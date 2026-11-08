use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};
use infrastructure::system::SystemService;

pub struct StatusPill {
    container: Box,
    clock_label: Label,
    #[allow(dead_code)]
    volume_label: Label,
    #[allow(dead_code)]
    wifi_label: Label,
    battery_label: Label,
}

impl StatusPill {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(5)
            .css_classes(["pill-container", "status-pill"])
            .build();

        // 1. Loa (Volume)
        let volume_label = Label::builder()
            .label("󰕾")
            .css_classes(["status-icon"])
            .tooltip_text("Volume: 80%")
            .build();
        container.append(&volume_label);

        // 2. Wi-Fi
        let wifi_label = Label::builder()
            .label("󰤨")
            .css_classes(["status-icon"])
            .tooltip_text("Wi-Fi: Connected")
            .build();
        container.append(&wifi_label);

        // 3. Pin (Battery)
        let battery_label = Label::builder()
            .label("󰁹")
            .css_classes(["status-icon"])
            .tooltip_text("Battery: 100% (Plugged In)")
            .build();
        container.append(&battery_label);

        // 4. Vertical separator
        let sep = Label::builder()
            .label("|")
            .css_classes(["separator"])
            .build();
        container.append(&sep);

        // 5. 24h Clock: HH:mm
        let initial_time = Self::format_current_time();
        let clock_label = Label::builder()
            .label(&initial_time)
            .css_classes(["clock-label"])
            .tooltip_text("Current Time (24h)")
            .build();
        container.append(&clock_label);

        let pill = Self {
            container,
            clock_label,
            volume_label,
            wifi_label,
            battery_label,
        };

        pill.start_update_timer();
        pill
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    fn format_current_time() -> String {
        glib::DateTime::now_local()
            .and_then(|dt| dt.format("%H:%M"))
            .map(|s| s.to_string())
            .unwrap_or_else(|_| "00:00".to_string())
    }

    fn start_update_timer(&self) {
        let clock = self.clock_label.clone();
        let battery = self.battery_label.clone();
        let mut sys = SystemService::new();

        glib::timeout_add_seconds_local(1, move || {
            // Update 24h clock
            let time_str = Self::format_current_time();
            clock.set_label(&time_str);

            // Periodically check battery status
            if let Some(bat_info) = sys.battery.read() {
                let pct = bat_info.capacity;
                let bat_icon = if pct > 80 {
                    "󰁹"
                } else if pct > 50 {
                    "󰂀"
                } else if pct > 20 {
                    "󰁾"
                } else {
                    "󰂃"
                };
                battery.set_label(bat_icon);
                battery.set_tooltip_text(Some(&format!("Battery: {}% ({:?})", pct, bat_info.status)));
            }

            glib::ControlFlow::Continue
        });
    }
}
