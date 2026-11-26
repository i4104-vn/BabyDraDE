//! Generic `iio-sensor-proxy` accelerometer driver.

use crate::modules::auto_rotate::{driver::RotationDriver, models::Gravity};

use super::proxy::ProxyClient;

pub(crate) struct SensorProxyDriver {
    client: ProxyClient,
}

impl SensorProxyDriver {
    /// Connects to the proxy and verifies that a reading is available.
    ///
    /// # Returns
    /// A boxed driver, or `None` when the proxy has no usable accelerometer.
    pub(crate) fn probe() -> Option<Box<dyn RotationDriver>> {
        let mut client = ProxyClient::connect()?;
        // `read()` claims the sensor itself, so a sensor appearing later is
        // covered without a second explicit claim here.
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
