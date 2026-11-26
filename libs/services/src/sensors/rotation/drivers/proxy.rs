//! Shared D-Bus client for `iio-sensor-proxy`.
//!
//! `net.hadess.SensorProxy` runs on the system D-Bus and publishes
//! `HasAccelerometer`, `AccelerometerOrientation` and `ClaimAccelerometer`.
//! Patched proxies may also expose a raw `Accelerate` tuple. Both are consumed
//! here so every D-Bus based driver shares one implementation.

use gio::glib::Variant;

use crate::sensors::rotation::Gravity;

const PROXY_NAME: &str = "net.hadess.SensorProxy";
const PROXY_PATH: &str = "/net/hadess/SensorProxy";
const PROXY_IFACE: &str = "net.hadess.SensorProxy";
const EARTH_GRAVITY: f64 = 9.80665;
const FLAT_GRAVITY: Gravity = Gravity { x: 0.0, y: 0.0 };

/// Maps iio-sensor-proxy's orientation enum to a gravity vector.
///
/// The signs follow upstream `orientation_calc()`: positive X is `left-up`,
/// positive Y is `bottom-up`.
fn gravity_from_orientation(value: &str) -> Option<Gravity> {
    match value {
        "normal" => Some(Gravity { x: 0.0, y: -1.0 }),
        "bottom-up" => Some(Gravity { x: 0.0, y: 1.0 }),
        "left-up" => Some(Gravity { x: 1.0, y: 0.0 }),
        "right-up" => Some(Gravity { x: -1.0, y: 0.0 }),
        _ => None,
    }
}

/// Checks whether `net.hadess.SensorProxy` responds on the given bus.
fn proxy_responds(connection: &gio::DBusConnection) -> bool {
    connection
        .call_sync(
            Some(PROXY_NAME),
            PROXY_PATH,
            "org.freedesktop.DBus.Properties",
            "Get",
            Some(&Variant::from((PROXY_IFACE, "HasAccelerometer"))),
            None,
            gio::DBusCallFlags::NONE,
            500,
            gio::Cancellable::NONE,
        )
        .is_ok()
}

pub(crate) struct ProxyClient {
    connection: gio::DBusConnection,
    claimed: bool,
}

impl ProxyClient {
    /// Connects to the bus hosting `iio-sensor-proxy`, preferring the system bus.
    ///
    /// # Returns
    /// A connected client, or `None` if neither system nor session bus is available.
    pub(crate) fn connect() -> Option<Self> {
        let system = gio::bus_get_sync(gio::BusType::System, gio::Cancellable::NONE).ok();
        if let Some(conn) = system {
            if proxy_responds(&conn) {
                return Some(Self {
                    connection: conn,
                    claimed: false,
                });
            }
            if let Ok(session) = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE) {
                if proxy_responds(&session) {
                    return Some(Self {
                        connection: session,
                        claimed: false,
                    });
                }
            }
            // Keep system bus so drivers like gaokun3 can wait for late activation.
            return Some(Self {
                connection: conn,
                claimed: false,
            });
        }
        let session = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()?;
        Some(Self {
            connection: session,
            claimed: false,
        })
    }

    /// Calls a method on the proxy interface.
    fn call(&self, method: &str, timeout: i32) -> Option<Variant> {
        self.connection
            .call_sync(
                Some(PROXY_NAME),
                PROXY_PATH,
                PROXY_IFACE,
                method,
                None,
                None,
                gio::DBusCallFlags::NONE,
                timeout,
                gio::Cancellable::NONE,
            )
            .ok()
    }

    /// Reads a property from the proxy interface.
    fn property(&self, name: &str, timeout: i32) -> Option<Variant> {
        let reply = self
            .connection
            .call_sync(
                Some(PROXY_NAME),
                PROXY_PATH,
                "org.freedesktop.DBus.Properties",
                "Get",
                Some(&Variant::from((PROXY_IFACE, name))),
                None,
                gio::DBusCallFlags::NONE,
                timeout,
                gio::Cancellable::NONE,
            )
            .ok()?;
        reply.child_value(0).get::<Variant>()
    }

    /// Returns whether the proxy currently reports an accelerometer.
    pub(crate) fn has_accelerometer(&self) -> bool {
        self.property("HasAccelerometer", 500)
            .and_then(|v| v.get::<bool>())
            .unwrap_or(false)
    }

    /// Claims the accelerometer, starting proxy polling.
    ///
    /// Claiming an absent sensor is allowed by upstream; it lets a sensor that
    /// appears later start updating without a second claim.
    fn claim(&mut self) -> bool {
        if self.claimed {
            return true;
        }
        if self.call("ClaimAccelerometer", 1000).is_some() {
            self.claimed = true;
        }
        self.claimed
    }

    /// Reads the current gravity, preferring raw values when available.
    ///
    /// Returns `Some(FLAT_GRAVITY)` when the proxy is healthy but the device is
    /// flat (`"undefined"`) or still initializing, so the driver manager does not
    /// misclassify normal idle states as hardware failures.
    pub(crate) fn read(&mut self) -> Option<Gravity> {
        if !self.claim() {
            return None;
        }
        if let Some(value) = self.property("Accelerate", 500) {
            if let (Some(x), Some(y)) = (
                value.child_value(0).get::<f64>(),
                value.child_value(1).get::<f64>(),
            ) {
                return Some(Gravity {
                    x: x / EARTH_GRAVITY,
                    y: y / EARTH_GRAVITY,
                });
            }
        }
        let Some(value) = self.property("AccelerometerOrientation", 500) else {
            self.claimed = false;
            return None;
        };
        let orientation = value.get::<String>()?;
        Some(gravity_from_orientation(&orientation).unwrap_or(FLAT_GRAVITY))
    }
}

#[cfg(test)]
mod tests {
    use super::gravity_from_orientation;
    use crate::sensors::rotation::Gravity;

    #[test]
    fn proxy_orientation_maps_to_gravity() {
        assert_eq!(
            gravity_from_orientation("normal"),
            Some(Gravity { x: 0.0, y: -1.0 })
        );
        assert_eq!(
            gravity_from_orientation("bottom-up"),
            Some(Gravity { x: 0.0, y: 1.0 })
        );
        assert_eq!(
            gravity_from_orientation("left-up"),
            Some(Gravity { x: 1.0, y: 0.0 })
        );
        assert_eq!(
            gravity_from_orientation("right-up"),
            Some(Gravity { x: -1.0, y: 0.0 })
        );
    }

    #[test]
    fn undefined_orientation_is_ignored() {
        assert_eq!(gravity_from_orientation("undefined"), None);
        assert_eq!(gravity_from_orientation("sideways"), None);
    }
}
