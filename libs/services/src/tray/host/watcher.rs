//! Tray host watcher.

use super::discovery::refresh;
use super::registration::{register_watcher, request_watcher_name};
use super::*;

pub(super) const WATCHER_XML: &str = r#"
<node>
  <interface name="org.kde.StatusNotifierWatcher">
    <method name="RegisterStatusNotifierItem">
      <arg type="s" name="identifier" direction="in"/>
    </method>
    <method name="RegisterStatusNotifierHost">
      <arg type="s" name="service" direction="in"/>
    </method>
    <signal name="StatusNotifierItemRegistered">
      <arg type="s" name="service"/>
    </signal>
    <signal name="StatusNotifierItemUnregistered">
      <arg type="s" name="service"/>
    </signal>
    <signal name="StatusNotifierHostRegistered"/>
    <property name="RegisteredStatusNotifierItems" type="as" access="read"/>
    <property name="IsStatusNotifierHostRegistered" type="b" access="read"/>
    <property name="ProtocolVersion" type="i" access="read"/>
  </interface>
</node>
"#;

pub(super) fn state() -> &'static Mutex<HostState> {
    HOST_STATE.get_or_init(|| Mutex::new(HostState::default()))
}

/// Entry point of the dedicated tray host thread.
pub(crate) fn run() {
    let context = glib::MainContext::new();
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);

    // The connection and every D-Bus callback must be tied to the same main
    // context. `with_thread_default` acquires the context for the duration of
    // the closure, so the main loop can only be started afterwards.
    let setup = context.with_thread_default(|| {
        let Some(connection) =
            gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()
        else {
            tracing::debug!("no session bus available; tray icons are disabled");
            let _ = sender.send(None);
            return;
        };

        let owns_watcher = request_watcher_name(&connection);
        if let Err(err) = register_watcher(&connection) {
            tracing::warn!("failed to export status notifier watcher: {err}");
        }
        if !owns_watcher {
            // Another panel already owns the watcher name. Items registered
            // with it are still picked up by the bus scan below.
            tracing::debug!("status notifier watcher name is owned by another host");
        }

        // `timeout_add_local_full` always targets the *global* default context,
        // which belongs to GTK. Attach the source to our own context explicitly.
        let timer_connection = connection.clone();
        let refresh_source = glib::timeout_source_new(
            REFRESH_INTERVAL,
            Some("babydra-tray-refresh"),
            glib::Priority::DEFAULT,
            move || {
                refresh(&timer_connection);
                glib::ControlFlow::Continue
            },
        );
        refresh_source.attach(Some(&context));

        let _ = sender.send(Some(connection));
    });
    if let Err(err) = setup {
        tracing::warn!("failed to configure tray host main context: {err}");
        return;
    }

    let Ok(Some(_connection)) = receiver.recv() else {
        return;
    };
    // Keep the connection alive: GIO closes it when the last reference drops.
    let main_loop = glib::MainLoop::new(Some(&context), false);
    main_loop.run();
}

