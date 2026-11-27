//! Huawei MateBook E Go (`huawei,gaokun3`) accelerometer driver.
//!
//! This convertible has no IIO sysfs accelerometer and no usable
//! `net.hadess.SensorProxy` device: the sensor hangs off the Qualcomm
//! SSC/SLPI DSP and is only reachable through `libssc` over QMI/QRTR. A working
//! userspace path is `ssccli --sensor accelerometer`, which streams the
//! acceleration as `X=.. Y=.. Z=.. m/s²` lines on stdout.
//!
//! Detection checks both the device-tree compatible string and the
//! `fastrpc-sdsp` transport, so the driver is inert on every other machine.
//! The sensor may only appear tens of seconds after boot, so probing succeeds
//! as soon as `ssccli` starts and an absent reading is reported as a flat
//! sample (which the debounce tracker ignores) instead of a hardware failure.

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::sensors::{Gravity, RotationDriver};

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
/// Override the helper binary location for testing or custom installs.
const SENSOR_CLI_ENV: &str = "BABYDRADE_SSCCLI";
/// Extra whitespace-separated arguments, e.g. `--debug` to trace QMI traffic.
const SENSOR_CLI_ARGS_ENV: &str = "BABYDRADE_SSCCLI_ARGS";
const SENSOR_CLI: &str = "ssccli";
/// `ssccli` exits after this many seconds; keep it alive for the session.
const CLI_TIMEOUT_SECONDS: &str = "2147483647";
/// Below half of Earth gravity the sample is considered flat/noise.
const MIN_MAGNITUDE: f64 = 4.9;

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

/// Parses one `KEY=value` field from an `ssccli` measurement line.
fn parse_field(line: &str, key: &str) -> Option<f64> {
    line.split_once(key)?
        .1
        .split_whitespace()
        .next()?
        .parse::<f64>()
        .ok()
}

/// Converts one `ssccli` measurement line into a normalized gravity vector.
///
/// libssc already applies the DSP mount matrix, so x and y are screen
/// relative. The z axis is only used to reject flat/noisy samples.
fn parse_measurement(line: &str) -> Option<Gravity> {
    let x = parse_field(line, "X=")?;
    let y = parse_field(line, "Y=")?;
    let z = parse_field(line, "Z=")?;
    let magnitude = (x * x + y * y + z * z).sqrt();
    if magnitude < MIN_MAGNITUDE {
        return Some(Gravity { x: 0.0, y: 0.0 });
    }
    Some(Gravity {
        x: x / magnitude,
        y: y / magnitude,
    })
}

pub(crate) struct Gaokun3Driver {
    child: Child,
    latest: Arc<Mutex<Option<Gravity>>>,
}

impl Gaokun3Driver {
    /// Detects a MateBook E Go with SSC transport and starts its sensor stream.
    pub(crate) fn probe() -> Option<Box<dyn RotationDriver>> {
        let compatible = read_compatible()?;
        if !has_gaokun_compatible(&compatible) {
            return None;
        }
        if !FASTRPC_PATHS.iter().any(|path| Path::new(path).exists()) {
            return None;
        }

        let cli = std::env::var_os(SENSOR_CLI_ENV).unwrap_or_else(|| SENSOR_CLI.into());
        let extra_args = std::env::var(SENSOR_CLI_ARGS_ENV).unwrap_or_default();
        let mut command = Command::new(cli);
        command
            .arg("--sensor")
            .arg("accelerometer")
            .arg("--timeout")
            .arg(CLI_TIMEOUT_SECONDS);
        command.args(extra_args.split_whitespace());
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .inspect_err(|error| tracing::warn!(%error, "cannot start {SENSOR_CLI} for gaokun3"))
            .ok()?;
        let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take()) else {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        };
        if thread::Builder::new()
            .name("babydra-gaokun3-log".into())
            .spawn(move || {
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => tracing::warn!(line = line.trim(), "gaokun3 sensor helper"),
                    }
                }
            })
            .is_err()
        {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }

        let latest = Arc::new(Mutex::new(None));
        let reader_slot = Arc::clone(&latest);
        if thread::Builder::new()
            .name("babydra-gaokun3".into())
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {
                            if let Some(gravity) = parse_measurement(&line) {
                                if let Ok(mut slot) = reader_slot.lock() {
                                    *slot = Some(gravity);
                                }
                            }
                        }
                    }
                }
            })
            .is_err()
        {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }

        Some(Box::new(Self { child, latest }))
    }
}

impl RotationDriver for Gaokun3Driver {
    fn name(&self) -> &str {
        "gaokun3"
    }

    fn read(&mut self) -> Option<Gravity> {
        let sample = self.latest.lock().ok().and_then(|mut slot| slot.take());
        if sample.is_some() {
            return sample;
        }
        // The sensor can appear long after boot; staying flat keeps the driver
        // manager from abandoning this machine's only working sensor.
        if self.child.try_wait().ok().flatten().is_none() {
            return Some(Gravity { x: 0.0, y: 0.0 });
        }
        None
    }
}

impl Drop for Gaokun3Driver {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::{has_gaokun_compatible, parse_measurement};
    use crate::sensors::Gravity;

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

    #[test]
    fn parses_ssccli_accelerometer_line() {
        let line = "Accelerometer sensor measurement: X=9.806650 Y=0.000000 Z=0.000000 m/s²\n";
        assert_eq!(parse_measurement(line), Some(Gravity { x: 1.0, y: 0.0 }));
    }

    #[test]
    fn reports_flat_when_magnitude_is_below_gravity() {
        let line = "Accelerometer sensor measurement: X=0.100000 Y=-0.200000 Z=0.300000 m/s²\n";
        assert_eq!(parse_measurement(line), Some(Gravity { x: 0.0, y: 0.0 }));
    }

    #[test]
    fn ignores_lines_without_acceleration_fields() {
        assert_eq!(
            parse_measurement("Unable to open accelerometer sensor\n"),
            None
        );
    }
}
