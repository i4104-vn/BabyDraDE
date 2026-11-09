use gtk4::prelude::*;
use infrastructure::system::{get_active_output_name, get_volume_state};
use std::rc::Rc;
use ui_kit::components::overlay::popover::tooltip::{TooltipPopover, TooltipRow};
use ui_kit::icons::set_image_from_icon;

pub fn build_volume_update(vol_icon: &gtk4::Image, vol_popover: &gtk4::Popover) -> Rc<dyn Fn()> {
    let vol_icon_c = vol_icon.clone();
    let vol_popover_c = vol_popover.clone();

    Rc::new(move || {
        let (vol_pct, is_m) = get_volume_state();
        let icon_name = if is_m {
            "volume-mute"
        } else if vol_pct > 66.0 {
            "volume"
        } else if vol_pct > 33.0 {
            "volume-medium"
        } else {
            "volume-low"
        };
        set_image_from_icon(&vol_icon_c, icon_name, 14);

        let dev_name = get_active_output_name();

        let vol_str = if is_m {
            format!("Muted ({:.0}%)", vol_pct)
        } else {
            format!("{:.0}%", vol_pct)
        };

        let mut rows = vec![TooltipRow::new("Volume", &vol_str, None)];
        if let Some(ref dev) = dev_name {
            rows.push(TooltipRow::new("Device", dev, None));
        }

        let card = TooltipPopover::build_card("Audio Output", &rows);
        vol_popover_c.set_child(Some(&card));
    })
}
