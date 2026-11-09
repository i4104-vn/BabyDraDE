use gtk4::prelude::*;
use infrastructure::system::{format_speed, get_active_network_info, ActiveNetworkType, NetworkReader};
use std::cell::RefCell;
use std::rc::Rc;
use ui_kit::components::overlay::popover::tooltip::{TooltipPopover, TooltipRow};

fn get_speed_color_class(bytes_per_sec: f64) -> &'static str {
    if bytes_per_sec > 1_048_576.0 {
        "speed-high"
    } else if bytes_per_sec > 102_400.0 {
        "speed-medium"
    } else {
        "speed-low"
    }
}

pub fn build_network_update(net_popover: &gtk4::Popover) -> Rc<dyn Fn()> {
    let net_popover_c = net_popover.clone();
    let net_reader = Rc::new(RefCell::new(NetworkReader::new()));

    Rc::new(move || {
        let active_net = get_active_network_info();
        let net_info = net_reader.borrow_mut().read();

        let rx_cls = get_speed_color_class(net_info.rx_speed_bps as f64);
        let tx_cls = get_speed_color_class(net_info.tx_speed_bps as f64);

        let rows = if !active_net.is_connected {
            vec![TooltipRow::new("Status", "Disconnected", None)]
        } else {
            let type_label = match active_net.network_type {
                ActiveNetworkType::Ethernet => "Ethernet",
                ActiveNetworkType::Wifi => "Wi-Fi",
                ActiveNetworkType::Disconnected => "Network",
            };
            let name_key = match active_net.network_type {
                ActiveNetworkType::Wifi => "SSID",
                _ => "Connection",
            };

            vec![
                TooltipRow::new("Type", type_label, None),
                TooltipRow::new(name_key, &active_net.name, None),
                TooltipRow::new("IP Address", &active_net.ip_address, None),
                TooltipRow::new(
                    "Download",
                    &format!("↓ {}", format_speed(net_info.rx_speed_bps as f64)),
                    Some(rx_cls),
                ),
                TooltipRow::new(
                    "Upload",
                    &format!("↑ {}", format_speed(net_info.tx_speed_bps as f64)),
                    Some(tx_cls),
                ),
            ]
        };

        let card = TooltipPopover::build_card("Network Connection", &rows);
        net_popover_c.set_child(Some(&card));
    })
}
