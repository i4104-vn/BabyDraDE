use std::fs;
use std::time::Instant;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InterfaceStats {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NetworkInfo {
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
    pub rx_speed_bps: f32, // bytes per second
    pub tx_speed_bps: f32, // bytes per second
    pub interfaces: Vec<InterfaceStats>,
}

#[derive(Debug, Clone)]
pub struct NetworkReader {
    last_read_time: Option<Instant>,
    last_totals: Option<(u64, u64)>, // (total_rx, total_tx)
}

impl Default for NetworkReader {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkReader {
    pub fn new() -> Self {
        let mut reader = Self {
            last_read_time: None,
            last_totals: None,
        };
        let _ = reader.read();
        reader
    }

    pub fn read(&mut self) -> NetworkInfo {
        let interfaces = Self::parse_proc_net_dev().unwrap_or_default();
        let now = Instant::now();

        let mut total_rx = 0u64;
        let mut total_tx = 0u64;

        for iface in &interfaces {
            // Exclude loopback "lo" from total network traffic calculation
            if iface.name != "lo" {
                total_rx = total_rx.saturating_add(iface.rx_bytes);
                total_tx = total_tx.saturating_add(iface.tx_bytes);
            }
        }

        let mut rx_speed_bps = 0.0f32;
        let mut tx_speed_bps = 0.0f32;

        if let (Some(last_time), Some((prev_rx, prev_tx))) = (self.last_read_time, self.last_totals) {
            let elapsed_secs = (now - last_time).as_secs_f32();
            if elapsed_secs > 0.001 {
                rx_speed_bps = (total_rx.saturating_sub(prev_rx) as f32) / elapsed_secs;
                tx_speed_bps = (total_tx.saturating_sub(prev_tx) as f32) / elapsed_secs;
            }
        }

        self.last_read_time = Some(now);
        self.last_totals = Some((total_rx, total_tx));

        NetworkInfo {
            total_rx_bytes: total_rx,
            total_tx_bytes: total_tx,
            rx_speed_bps,
            tx_speed_bps,
            interfaces,
        }
    }

    fn parse_proc_net_dev() -> Option<Vec<InterfaceStats>> {
        let content = fs::read_to_string("/proc/net/dev").ok()?;
        let mut list = Vec::new();

        for line in content.lines().skip(2) {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() != 2 {
                continue;
            }

            let name = parts[0].trim().to_string();
            let values: Vec<u64> = parts[1]
                .split_whitespace()
                .filter_map(|s| s.parse::<u64>().ok())
                .collect();

            if values.len() < 16 {
                continue;
            }

            let rx_bytes = values[0];
            let rx_packets = values[1];
            let rx_errors = values[2];
            let tx_bytes = values[8];
            let tx_packets = values[9];
            let tx_errors = values[10];

            list.push(InterfaceStats {
                name,
                rx_bytes,
                tx_bytes,
                rx_packets,
                tx_packets,
                rx_errors,
                tx_errors,
            });
        }

        Some(list)
    }
}
