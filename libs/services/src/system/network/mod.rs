pub mod stats;

pub use stats::{InterfaceStats, NetworkInfo, NetworkReader};

use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveNetworkType {
    Wifi,
    Ethernet,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveNetworkInfo {
    pub network_type: ActiveNetworkType,
    pub name: String,
    pub ip_address: String,
    pub is_connected: bool,
    pub icon_name: String,
}

pub fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1_048_576.0 {
        format!("{:.1} MB/s", bytes_per_sec / 1_048_576.0)
    } else if bytes_per_sec >= 1024.0 {
        format!("{:.0} KB/s", bytes_per_sec / 1024.0)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

pub fn get_active_network_info() -> ActiveNetworkInfo {
    // Check interfaces in /sys/class/net
    let mut active_iface = None;
    let mut is_wifi = false;

    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "lo"
                || name.starts_with("docker")
                || name.starts_with("veth")
                || name.starts_with("virbr")
            {
                continue;
            }

            // Check if operstate is up
            let operstate_path = entry.path().join("operstate");
            if let Ok(state) = fs::read_to_string(operstate_path) {
                if state.trim() == "up" {
                    let is_w = name.starts_with("wl");
                    active_iface = Some(name);
                    is_wifi = is_w;
                    if is_wifi {
                        break;
                    }
                }
            }
        }
    }

    if let Some(iface) = active_iface {
        // Read IP address
        let ip = get_interface_ip(&iface).unwrap_or_else(|| "127.0.0.1".to_string());
        if is_wifi {
            let ssid = get_wifi_ssid().unwrap_or_else(|| "Wi-Fi".to_string());
            ActiveNetworkInfo {
                network_type: ActiveNetworkType::Wifi,
                name: ssid,
                ip_address: ip,
                is_connected: true,
                icon_name: "wifi".to_string(),
            }
        } else {
            ActiveNetworkInfo {
                network_type: ActiveNetworkType::Ethernet,
                name: "Ethernet".to_string(),
                ip_address: ip,
                is_connected: true,
                icon_name: "ethernet".to_string(),
            }
        }
    } else {
        ActiveNetworkInfo {
            network_type: ActiveNetworkType::Disconnected,
            name: "Disconnected".to_string(),
            ip_address: "".to_string(),
            is_connected: false,
            icon_name: "wifi".to_string(),
        }
    }
}

fn get_interface_ip(iface: &str) -> Option<String> {
    if let Ok(output) = Command::new("ip")
        .args(["-4", "addr", "show", iface])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("inet ") {
                if let Some(ip_cidr) = trimmed.split_whitespace().nth(1) {
                    if let Some(ip) = ip_cidr.split('/').next() {
                        return Some(ip.to_string());
                    }
                }
            }
        }
    }
    None
}

fn get_wifi_ssid() -> Option<String> {
    if let Ok(output) = Command::new("iwgetid").arg("-r").output() {
        let ssid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !ssid.is_empty() {
            return Some(ssid);
        }
    }
    None
}
