use gtk4::prelude::*;
use services::system::BatteryReader;
use std::rc::Rc;
use ui_kit::components::overlay::popover::tooltip::{TooltipPopover, TooltipRow};

pub fn build_battery_update(bat_popover_opt: &Option<gtk4::Popover>) -> Rc<dyn Fn()> {
    let bat_popover_c = bat_popover_opt.clone();

    Rc::new(move || {
        if let Some(ref bat_pop) = bat_popover_c {
            let mut reader = BatteryReader::new();
            if let Some(info) = reader.read() {
                let mut rows = Vec::new();

                rows.push(TooltipRow::new(
                    "Level",
                    &format!("{}%", info.capacity),
                    None,
                ));
                rows.push(TooltipRow::new("State", &format!("{}", info.status), None));
                if let Some(w) = info.power_watts {
                    rows.push(TooltipRow::new("Power Draw", &format!("{:.1} W", w), None));
                }
                if let (Some(now), Some(full)) = (info.energy_now_wh, info.energy_full_wh) {
                    rows.push(TooltipRow::new(
                        "Capacity",
                        &format!("{:.1} / {:.1} Wh", now, full),
                        None,
                    ));
                }

                let title = "Power & Battery";
                let card = TooltipPopover::build_card(title, &rows);
                bat_pop.set_child(Some(&card));
            }
        }
    })
}
