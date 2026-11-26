//! Huawei MateBook E Go (gaokun3) accelerometer driver.
//!
//! This convertible has no IIO sysfs accelerometer: the sensor hangs off the
//! Qualcomm SSC/SLPI DSP and is published by an SSC-enabled `iio-sensor-proxy`
//! over D-Bus. Detection checks both the device-tree compatible string and the
//! `fastrpc-sdsp` transport, so the driver is inert on every other machine.
//!
//! The proxy may discover the sensor tens of seconds after boot, so probing
//! deliberately succeeds before a reading exists and claiming is retried on
//! the first read.

use std::path::Path;

use crate::modules::auto_rotate::{driver::RotationDriver, models::Gravity};

use super::proxy::ProxyClient;

const COMPATIBLE: &str = "huawei,gaokun3";
const FASTRPC_PATHS: [&str; 2] = ["/dev/fastrpc-sdsp", "/sys/class/misc/fastrpc-sdsp"];

/// Checks a NUL-separated device-tree `compatible` property.
fn has_gaokun3_compatible(data: &[u8]) -> bool {
    data.split(|byte| *byte == 0)
        .any(|entry| entry == COMPATIBLE.as_bytes())
}

pub(crate) struct Gaokun3Driver {
    client: ProxyClient,
}

impl Gaokun3Driver {
    /// Detects a gaokun3 with working SSC transport and opens the D-Bus proxy.
    pub(crate) fn probe() -> Option<Box<dyn RotationDriver>> {
        let compatible = std::fs::read("/proc/device-tree/compatible").ok()?;
        if !has_gaokun3_compatible(&compatible) {
            return None;
        }
        if !FASTRPC_PATHS.iter().any(|path| Path::new(path).exists()) {
            return None;
        }
        let client = ProxyClient::connect()?;
        Some(Box::new(Self { client }))
    }
}

impl RotationDriver for Gaokun3Driver {
    fn name(&self) -> &str {
        "gaokun3"
    }

    fn read(&mut self) -> Option<Gravity> {
        self.client.read()
    }
}

#[cfg(test)]
mod tests {
    use super::has_gaokun3_compatible;

    #[test]
    fn detects_gaokun3_in_device_tree_compatible() {
        let compatible = b"huawei,gaokun3\0qcom,sc8280xp\0";
        assert!(has_gaokun3_compatible(compatible));
    }

    #[test]
    fn rejects_other_device_tree_compatibles() {
        assert!(!has_gaokun3_compatible(b"qcom,sc8280xp\0other,device\0"));
    }
}
