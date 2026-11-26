//! Built-in accelerometer drivers.
//!
//! Register new drivers in `super::driver::DriverFactory`.

pub(crate) mod gaokun3;
pub(crate) mod iio;
pub(crate) mod proxy;
pub(crate) mod sensor_proxy;
