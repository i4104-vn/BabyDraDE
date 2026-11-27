//! Network monitoring services.
//!
//! The active link is discovered through sysfs and its SSID is queried from
//! that very same interface, so a tooltip never pairs one interface with a
//! network name read from another one.

/// Network counter sampling.
pub mod stats;

#[cfg(test)]
mod tests;

pub use stats::{InterfaceStats, NetworkInfo, NetworkReader};

use std::fs;
use std::path::Path;
use std::process::Command;

/// Kind of the active network connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveNetworkType {
    Wifi,
    Ethernet,
    Disconnected,
}

/// Details of the active network interface.
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveNetworkInfo {
    pub network_type: ActiveNetworkType,
    pub name: String,
    pub ip_address: String,
    pub is_connected: bool,
    pub icon_name: String,
}

/// Formats a byte rate as a human-readable string.
///
/// # Arguments
/// * `bytes_per_sec` - Transfer rate in bytes per second.
///
/// # Returns
/// A string such as `"1.5 MB/s"`, `"512 KB/s"` or `"64 B/s"`.
pub fn format_speed(bytes_per_sec: f64) -> String {
    match true {
        _ if bytes_per_sec >= 1_048_576.0 => {
            format!("{:.1} MB/s", bytes_per_sec / 1_048_576.0)
        }
        _ if bytes_per_sec >= 1024.0 => format!("{:.0} KB/s", bytes_per_sec / 1024.0),
        _ => format!("{:.0} B/s", bytes_per_sec),
    }
}

/// Detects the active network interface and its IP address.
///
/// Wi-Fi interfaces are detected from sysfs markers (with the conventional
/// `wl*` name as a fallback) and take precedence over wired links. The
/// detected interface name is passed on to the SSID lookup.
///
/// # Returns
/// Connection type, interface name (SSID for Wi-Fi), IP address and display
/// icon.
pub fn get_active_network_info() -> ActiveNetworkInfo {
    // Check interfaces in /sys/class/net
    let mut wifi_iface = None;
    let mut wired_iface = None;

    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if is_virtual_interface(&name) {
                continue;
            }

            let path = entry.path();
            let is_up = fs::read_to_string(path.join("operstate"))
                .map(|state| state.trim() == "up")
                .unwrap_or(false);
            if !is_up {
                continue;
            }

            if is_wifi_interface(&path, &name) {
                wifi_iface = Some(name);
                break;
            }
            if wired_iface.is_none() {
                wired_iface = Some(name);
            }
        }
    }

    let active = wifi_iface
        .map(|name| (name, true))
        .or_else(|| wired_iface.map(|name| (name, false)));
    let Some((iface, is_wifi)) = active else {
        return ActiveNetworkInfo {
            network_type: ActiveNetworkType::Disconnected,
            name: "Disconnected".to_string(),
            ip_address: "".to_string(),
            is_connected: false,
            icon_name: "wifi".to_string(),
        };
    };

    let ip = get_interface_ip(&iface).unwrap_or_else(|| "127.0.0.1".to_string());
    if is_wifi {
        let ssid = get_wifi_ssid(&iface).unwrap_or_else(|| "Wi-Fi".to_string());
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
}

/// Returns whether a name belongs to a loopback, bridge or container device.
///
/// These interfaces never represent the user's active link and must be
/// skipped consistently during discovery.
fn is_virtual_interface(name: &str) -> bool {
    name == "lo"
        || name.starts_with("docker")
        || name.starts_with("veth")
        || name.starts_with("virbr")
        || name.starts_with("br-")
        || name.starts_with("cni")
        || name.starts_with("flannel")
        || name.starts_with("dummy")
}

/// Returns whether a sysfs interface directory belongs to a Wi-Fi device.
///
/// The `wireless` and `phy80211` markers are authoritative; the `wl*` name
/// prefix is only used as a fallback for kernels/driver setups that expose
/// neither marker.
fn is_wifi_interface(path: &Path, name: &str) -> bool {
    path.join("wireless").exists() || path.join("phy80211").exists() || name.starts_with("wl")
}

/// Reads the IPv4 address assigned to `iface`.
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

/// Cleans a raw SSID, returning `None` when no usable text remains.
///
/// Trims surrounding whitespace and quotes (some frontends quote SSIDs) and
/// drops control characters, including stray NUL bytes.
fn sanitize_ssid(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let unquoted = trimmed
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(trimmed);
    let cleaned: String = unquoted.chars().filter(|c| !c.is_control()).collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

/// Runs a helper command and parses its stdout with `parse`.
///
/// Command failures, empty output and invalid UTF-8 are handled gracefully:
/// stdout is decoded lossily and the parser decides whether it is usable.
fn run_ssid_command<F>(program: &str, args: &[&str], parse: F) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse(&String::from_utf8_lossy(&output.stdout))
}

/// Extracts the first usable line from `iwgetid -r <iface>` output.
fn parse_iwgetid_output(output: &str) -> Option<String> {
    output.lines().find_map(sanitize_ssid)
}

/// Extracts the `SSID:` field from `iw dev <iface> link` output.
fn parse_iw_link_ssid(output: &str) -> Option<String> {
    for line in output.lines() {
        let Some(value) = line.trim().strip_prefix("SSID:") else {
            continue;
        };
        if let Some(ssid) = sanitize_ssid(value) {
            return Some(ssid);
        }
    }
    None
}

/// Extracts the active SSID from `nmcli -t -f active,ssid dev wifi` output.
fn parse_nmcli_active_ssid(output: &str) -> Option<String> {
    for line in output.lines() {
        let mut fields = line.splitn(2, ':');
        let Some(active) = fields.next() else {
            continue;
        };
        if active.trim() != "yes" {
            continue;
        }
        let Some(ssid) = fields.next() else {
            continue;
        };
        // Terse nmcli output escapes colons inside SSIDs as `\:`.
        if let Some(ssid) = sanitize_ssid(&ssid.replace("\\:", ":")) {
            return Some(ssid);
        }
    }
    None
}

/// Resolves the SSID of the active Wi-Fi interface.
///
/// # Fallback order
/// 1. `iwgetid -r <iface>` — raw SSID, scoped to the detected interface.
/// 2. `iw dev <iface> link` — parses the `SSID:` line of the associated link.
/// 3. `nmcli -t -f active,ssid dev wifi` — NetworkManager's active Wi-Fi
///    network, used only when neither interface-scoped command works.
fn get_wifi_ssid(iface: &str) -> Option<String> {
    if let Some(ssid) = run_ssid_command("iwgetid", &["-r", iface], parse_iwgetid_output) {
        return Some(ssid);
    }
    if let Some(ssid) = run_ssid_command("iw", &["dev", iface, "link"], parse_iw_link_ssid) {
        return Some(ssid);
    }
    run_ssid_command(
        "nmcli",
        &["-t", "-f", "active,ssid", "dev", "wifi"],
        parse_nmcli_active_ssid,
    )
}
