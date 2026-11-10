//! VPN status detection

use std::fs;

pub fn is_vpn_active() -> bool {
    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("tun")
                || name_str.starts_with("wg")
                || name_str.starts_with("tap")
                || name_str.starts_with("ppp")
            {
                return true;
            }
        }
    }
    false
}

