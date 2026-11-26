//! Built-in accelerometer drivers.
//!
//! Register new drivers in [`crate::modules::auto_rotate::driver::DriverFactory`].

pub(crate) mod iio;
pub(crate) mod proxy;
pub(crate) mod sensor_proxy;
