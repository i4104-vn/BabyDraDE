//! Huawei MateBook E Go (`linux-gaokun` / `huawei,gaokun3`) accelerometer driver.
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

use crate::sensors::rotation::{Gravity, RotationDriver};

use super::proxy::ProxyClient;

const COMPATIBLE_PREFIX: &[u8] = b"huawei,gaokun";
const DEVICE_TREE_PATHS: [&str; 2] = [
    "/proc/device-tree/compatible",
    "/sys/firmware/devicetree/base/compatible",
];
const FASTRPC_PATHS: [&str; 3] = [
    "/dev/fastrpc-sdsp",
    "/dev/fastrpc-sdsp0",
    "/sys/class/misc/fastrpc-sdsp",
];

/// Checks a NUL-separated device-tree `compatible` property for any gaokun model.
fn has_gaokun_compatible(data: &[u8]) -> bool {
    data.split(|byte| *byte == 0)
        .any(|entry| entry.starts_with(COMPATIBLE_PREFIX))
}

/// Reads the device-tree compatible property from standard Linux paths.
fn read_compatible() -> Option<Vec<u8>> {
    DEVICE_TREE_PATHS
        .iter()
        .find_map(|path| std::fs::read(path).ok())
}

pub(crate) struct Gaokun3Driver {
    client: ProxyClient,
}

impl Gaokun3Driver {
    /// Detects a MateBook E Go (`gaokun`) with SSC transport and opens the D-Bus proxy.
    pub(crate) fn probe() -> Option<Box<dyn RotationDriver>> {
        let compatible = read_compatible()?;
        if !has_gaokun_compatible(&compatible) {
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
    use super::has_gaokun_compatible;

    #[test]
    fn detects_gaokun_variants_in_device_tree_compatible() {
        assert!(has_gaokun_compatible(b"huawei,gaokun3\0qcom,sc8280xp\0"));
        assert!(has_gaokun_compatible(b"huawei,gaokun2\0qcom,sc8280xp\0"));
        assert!(has_gaokun_compatible(b"huawei,gaokun\0qcom,sc8280xp\0"));
    }

    #[test]
    fn rejects_other_device_tree_compatibles() {
        assert!(!has_gaokun_compatible(b"qcom,sc8280xp\0other,device\0"));
    }
}
