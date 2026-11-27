//! Single IPC entry point for the island (`org.babydra.Island`).
//!
//! Features register a callback once in `build()` via `ctx.on_trigger(...)`,
//! instead of each owning its own trigger thread-local and D-Bus server. GIO
//! (already a dependency) hosts the interface; calls are forwarded to the GTK
//! main context before the feature callbacks run.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

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

static INITIALIZED: AtomicBool = AtomicBool::new(false);

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

/// Starts the D-Bus service and the trigger dispatcher exactly once.
pub fn init() {
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        return;
    }

    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<&'static str>();
    glib::MainContext::default().spawn_local(async move {
        while let Some(topic) = receiver.recv().await {
            fire(topic);
        }
    });

    std::thread::Builder::new()
        .name("babydra-island-ipc".into())
        .spawn(move || {
            let context = glib::MainContext::new();
            let main_loop = glib::MainLoop::new(Some(&context), false);

            let services = context.with_thread_default(move || {
                let connection =
                    gio::bus_get_sync(gio::BusType::Session, None::<&gio::Cancellable>).ok()?;
                let node = gio::DBusNodeInfo::for_xml(INTROSPECTION_XML).ok()?;
                let interface = node.lookup_interface("org.babydra.Island")?;

                let tx = sender;
                let registration = connection
                    .register_object("/org/babydra/Island", &interface)
                    .method_call(move |_, _, _, _, method, _, invocation| {
                        let topic = match method {
                            "ShowClipboard" | "ToggleClipboard" => Some(CLIPBOARD),
                            "ShowPower" | "TogglePower" => Some(POWER),
                            "ShowRecording" => Some(RECORDING),
                            _ => None,
                        };
                        if let Some(topic) = topic {
                            let _ = tx.send(topic);
                        }
                        // GIO requires every method call to be answered.
                        invocation.return_value(None);
                    })
                    .build()
                    .ok()?;

                let owner = gio::bus_own_name_on_connection(
                    &connection,
                    "org.babydra.Island",
                    gio::BusNameOwnerFlags::NONE,
                    |_, _| {},
                    |_, _| {},
                );
                Some((connection, registration, owner))
            });

            // Keep the connection, registration and name owner alive for the
            // lifetime of the process by running the context's main loop.
            let Ok(Some(_services)) = services else {
                return;
            };
            main_loop.run();
        })
        .ok();
}
