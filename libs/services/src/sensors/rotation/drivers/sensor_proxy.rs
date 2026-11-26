//! Generic `iio-sensor-proxy` accelerometer driver.

use crate::sensors::rotation::{Gravity, RotationDriver};

use super::proxy::ProxyClient;

pub(crate) struct SensorProxyDriver {
    client: ProxyClient,
}

impl SensorProxyDriver {
    /// Connects to the proxy and verifies that an accelerometer is available.
    ///
    /// # Returns
    /// A boxed driver, or `None` when the proxy has no usable accelerometer.
    pub(crate) fn probe() -> Option<Box<dyn RotationDriver>> {
        let mut client = ProxyClient::connect()?;
        if !client.has_accelerometer() {
            return None;
        }
        client.read()?;
        Some(Box::new(Self { client }))
    }
}

impl RotationDriver for SensorProxyDriver {
    fn name(&self) -> &str {
        "sensor-proxy"
    }

    fn read(&mut self) -> Option<Gravity> {
        self.client.read()
    }
}
