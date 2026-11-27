//! Single IPC entry point for the island (`org.babydra.Island`).
//!
//! Features register a callback once in `build()` via `ctx.on_trigger(...)`,
//! instead of each owning its own trigger thread-local and D-Bus server. GIO
//! (already a dependency) hosts the interface; calls are forwarded to the GTK
//! main context before the feature callbacks run.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

type Callback = Box<dyn Fn()>;

thread_local! {
    static TRIGGERS: RefCell<HashMap<&'static str, Vec<Callback>>> = RefCell::new(HashMap::new());
}

/// Topics exposed over D-Bus.
const CLIPBOARD: &str = "clipboard";
const POWER: &str = "power";
const RECORDING: &str = "recording";

const INTROSPECTION_XML: &str = r#"
<node>
  <interface name='org.babydra.Island'>
    <method name='ShowClipboard'/>
    <method name='ToggleClipboard'/>
    <method name='ShowPower'/>
    <method name='TogglePower'/>
    <method name='ShowRecording'/>
  </interface>
</node>"#;

/// Registration is considered initialized only after the D-Bus object is live.
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static ATTEMPTS: AtomicU32 = AtomicU32::new(0);
const MAX_INIT_ATTEMPTS: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(2);

enum Command {
    Trigger(&'static str),
    Retry,
}

/// Registers a feature callback for its trigger topic.
pub fn register_trigger(topic: &'static str, callback: impl Fn() + 'static) {
    TRIGGERS.with(|triggers| {
        triggers
            .borrow_mut()
            .entry(topic)
            .or_default()
            .push(Box::new(callback));
    });
}

/// Fires a feature trigger from local code (hardcoded shortcuts).
pub fn trigger(topic: &'static str) {
    fire(topic);
}

fn fire(topic: &str) {
    TRIGGERS.with(|triggers| {
        if let Ok(triggers) = triggers.try_borrow() {
            if let Some(callbacks) = triggers.get(topic) {
                for callback in callbacks {
                    callback();
                }
            }
        }
    });
}

/// Starts the D-Bus service and the trigger dispatcher at most once while it
/// stays registered. A failed registration is retried a bounded number of times.
pub fn init() {
    if INITIALIZED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }
    let attempt = ATTEMPTS.fetch_add(1, Ordering::SeqCst) + 1;
    if attempt > MAX_INIT_ATTEMPTS {
        tracing::warn!("giving up on island D-Bus registration after {MAX_INIT_ATTEMPTS} attempts");
        INITIALIZED.store(false, Ordering::SeqCst);
        return;
    }

    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Command>();
    let retry_tx = sender.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Some(command) = receiver.recv().await {
            match command {
                Command::Trigger(topic) => fire(topic),
                Command::Retry => {
                    // Once the failed worker has released INITIALIZED, try again
                    // on the GTK main context after a short backoff.
                    let _ = glib::timeout_add_local_once(RETRY_DELAY, || {
                        init();
                    });
                }
            }
        }
    });

    if let Err(error) = std::thread::Builder::new()
        .name("babydra-island-ipc".into())
        .spawn(move || {
            let context = glib::MainContext::new();
            let main_loop = glib::MainLoop::new(Some(&context), false);

            let services = context
                .with_thread_default(move || {
                    let connection =
                        gio::bus_get_sync(gio::BusType::Session, None::<&gio::Cancellable>)
                            .map_err(|error| tracing::warn!("island D-Bus connect failed: {error}"))
                            .ok()?;
                    let node = gio::DBusNodeInfo::for_xml(INTROSPECTION_XML)
                        .map_err(|error| tracing::warn!("island introspection XML failed: {error}"))
                        .ok()?;
                    let interface = node.lookup_interface("org.babydra.Island").or_else(|| {
                        tracing::warn!("island D-Bus interface missing from introspection");
                        None
                    })?;

                    let tx = sender;
                    let registration = connection
                        .register_object("/org/babydra/Island", &interface)
                        .method_call(move |_, _, _, _, method, _, invocation| {
                            let topic = match method {
                                "ShowClipboard" | "ToggleClipboard" => {
                                    Some(Command::Trigger(CLIPBOARD))
                                }
                                "ShowPower" | "TogglePower" => Some(Command::Trigger(POWER)),
                                "ShowRecording" => Some(Command::Trigger(RECORDING)),
                                _ => None,
                            };
                            if let Some(topic) = topic {
                                let _ = tx.send(topic);
                            }
                            // GIO requires every method call to be answered.
                            invocation.return_value(None);
                        })
                        .build()
                        .map_err(|error| {
                            tracing::warn!("island D-Bus registration failed: {error}");
                        })
                        .ok()?;

                    let owner = gio::bus_own_name_on_connection(
                        &connection,
                        "org.babydra.Island",
                        gio::BusNameOwnerFlags::NONE,
                        |_, _| {},
                        |_, _| {},
                    );
                    Some((connection, registration, owner))
                })
                .ok()
                .flatten();

            // Keep the connection, registration and name owner alive for the
            // lifetime of the process by running the context's main loop.
            if services.is_none() {
                INITIALIZED.store(false, Ordering::SeqCst);
                if attempt < MAX_INIT_ATTEMPTS {
                    let _ = retry_tx.send(Command::Retry);
                } else {
                    tracing::warn!("island D-Bus service unavailable; triggers stay disabled");
                }
                return;
            }
            main_loop.run();
        })
    {
        tracing::warn!("cannot start island IPC thread: {error}");
        INITIALIZED.store(false, Ordering::SeqCst);
        if attempt < MAX_INIT_ATTEMPTS {
            let _ = glib::timeout_add_local_once(RETRY_DELAY, init);
        }
    }
}
