//! Bridge from the desktop feature to the compositor bus.
//!
//! Win+F5 is handled by the compositor, which publishes a config reload event.
//! The desktop subscribes here so its wallpaper can re-read the config without
//! watching files or polling the config store itself.

use std::cell::RefCell;

use services::protocol::{Bus, Event, Incoming, CONFIG_RELOAD_TOPIC};

/// Stable peer name registered with the router.
const OWNER: &str = "org.babydrade.desktop";

thread_local! {
    static BUS: RefCell<Option<Bus>> = const { RefCell::new(None) };
}

/// Connects on first use and subscribes to config reload events.
///
/// # Returns
/// `true` when a bus client is ready; failures are retried on the next poll
/// because the desktop may start before the compositor socket is available.
fn ensure_bus() -> bool {
    BUS.with(|slot| {
        let mut bus = slot.borrow_mut();
        if bus.is_none() {
            match Bus::connect(OWNER) {
                Ok(client) => {
                    if let Err(err) = client.subscribe(CONFIG_RELOAD_TOPIC) {
                        tracing::warn!("cannot subscribe to config reload: {err}");
                    }
                    *bus = Some(client);
                }
                Err(err) => {
                    tracing::debug!("protocol bus not ready yet: {err}");
                    return false;
                }
            }
        }
        true
    })
}

/// Consumes pending bus traffic and reports whether config was reloaded.
///
/// # Returns
/// `true` when at least one config reload event arrived since the last call.
pub fn take_reload() -> bool {
    if !ensure_bus() {
        return false;
    }
    BUS.with(|slot| {
        let bus = slot.borrow();
        let Some(client) = bus.as_ref() else {
            return false;
        };
        client.drain().into_iter().any(|incoming| {
            matches!(incoming, Incoming::Event(message)
                if message.decode::<Event>().ok() == Some(Event::ConfigReloaded))
        })
    })
}
