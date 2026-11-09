use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box, Label};
use infrastructure::system::SystemService;
use ui_kit::{PillBox, VerticalSeparator};

pub struct StatusPill {
    pill: PillBox,
}

impl StatusPill {
    pub fn new() -> Self {
        let pill = PillBox::with_extra_class(5, "status-pill");

        // 1. Loa (Volume)
        let volume_label = Label::builder()
            .label("󰕾")
            .css_classes(["status-icon"])
            .tooltip_text("Volume: 80%")
            .build();
        pill.append(&volume_label);

        // 2. Wi-Fi
        let wifi_label = Label::builder()
            .label("󰤨")
            .css_classes(["status-icon"])
            .tooltip_text("Wi-Fi: Connected")
            .build();
        pill.append(&wifi_label);

        // 3. Pin (Battery)
        let battery_label = Label::builder()
            .label("󰁹")
            .css_classes(["status-icon"])
            .tooltip_text("Battery: 100% (Plugged In)")
            .build();
        pill.append(&battery_label);

        // 4. Vertical separator
        let sep = VerticalSeparator::new();
        pill.append(sep.widget());

        // 5. 24h Clock: HH:mm
        let initial_time = Self::format_current_time();
        let clock_label = Label::builder()
            .label(&initial_time)
            .css_classes(["clock-label"])
            .tooltip_text("Current Time (24h)")
            .build();
        pill.append(&clock_label);

        Self::start_update_timer(clock_label, battery_label);

        Self { pill }
    }

    pub fn widget(&self) -> &Box {
        self.pill.widget()
    }

    fn format_current_time() -> String {
        glib::DateTime::now_local()
            .and_then(|dt| dt.format("%H:%M"))
            .map(|s| s.to_string())
            .unwrap_or_else(|_| "00:00".to_string())
    }

    fn start_update_timer(clock: Label, battery: Label) {
        let mut sys = SystemService::new();

        glib::timeout_add_seconds_local(1, move || {
            // Update 24h clock
            let time_str = Self::format_current_time();
            clock.set_label(&time_str);

            // Periodically check battery status
            if let Some(bat_info) = sys.battery.read() {
                let pct = bat_info.capacity;
                let bat_icon = match pct {
                    81..=100 => "󰁹",
                    51..=80 => "󰂀",
                    21..=50 => "󰁾",
                    _ => "󰂃",
                };
                battery.set_label(bat_icon);
                battery.set_tooltip_text(Some(&format!("Battery: {}% ({:?})", pct, bat_info.status)));
            }

            glib::ControlFlow::Continue
        });
    }
}
