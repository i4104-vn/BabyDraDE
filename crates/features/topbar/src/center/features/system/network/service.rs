//! Data source for the network indicator (read by `crate::center::poller`).

pub use services::system::network::{get_active_network_info, ActiveNetworkType};

/// One poll of the active network state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkSnapshot {
    pub connected: bool,
    pub kind: ActiveNetworkType,
    pub name: String,
    pub strength: u8,
}

/// Approximate Wi-Fi strength (0..100) from `iw`.
fn wifi_strength(interface: &str) -> u8 {
    let Ok(output) = std::process::Command::new("iw")
        .args(["dev", interface, "link"])
        .output()
    else {
        return 100;
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let Some(dbm) = text.lines().find_map(|line| {
        line.split("signal:")
            .nth(1)
            .and_then(|rest| rest.split_whitespace().next())
            .and_then(|value| value.parse::<i32>().ok())
    }) else {
        return 100;
    };
    ((dbm + 90) * 100 / 60).clamp(0, 100) as u8
}

/// Reads the active connection once.
pub fn read() -> NetworkSnapshot {
    let info = get_active_network_info();
    let strength = if info.network_type == ActiveNetworkType::Wifi {
        wifi_strength(&info.name)
    } else {
        0
    };
    NetworkSnapshot {
        connected: info.is_connected,
        kind: info.network_type,
        name: info.name,
        strength,
    }
}
