//! Data source for the Bluetooth indicator (read by `crate::island::poller`).

/// Connected Bluetooth device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BtConnectedDevice {
    pub mac: String,
    pub name: String,
    pub battery: Option<u8>,
}

/// Reads connected devices from `bluetoothctl` (the workspace does not yet
/// expose a dedicated Bluetooth service).
pub fn read() -> Vec<BtConnectedDevice> {
    let Ok(output) = std::process::Command::new("bluetoothctl")
        .args(["devices", "Connected"])
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            if parts.next()? != "Device" {
                return None;
            }
            let mac = parts.next()?.to_string();
            let name = parts.collect::<Vec<_>>().join(" ");
            (!name.is_empty()).then_some(BtConnectedDevice {
                mac,
                name,
                battery: None,
            })
        })
        .collect()
}
