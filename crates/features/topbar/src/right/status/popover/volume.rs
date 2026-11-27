use gtk4::prelude::*;
use services::system::{get_active_output_name, get_volume_state};
use std::rc::Rc;
use ui_kit::components::overlay::popover::tooltip::{TooltipPopover, TooltipRow};
use ui_kit::icons::set_image_from_icon;

/// Keeps long output-device names from stretching the tooltip card.
const MAX_DEVICE_CHARS: usize = 32;

/// Trims and shortens a device name with a trailing ellipsis when needed.
fn compact_device_name(name: &str) -> String {
    let name = name.trim();
    if name.chars().count() <= MAX_DEVICE_CHARS {
        return name.to_string();
    }
    let mut shortened: String = name
        .chars()
        .take(MAX_DEVICE_CHARS.saturating_sub(1))
        .collect();
    shortened.push('…');
    shortened
}

pub fn build_volume_update(vol_icon: &gtk4::Image, vol_popover: &gtk4::Popover) -> Rc<dyn Fn()> {
    let vol_icon_c = vol_icon.clone();
    let vol_popover_c = vol_popover.clone();

    Rc::new(move || {
        let (vol_pct, is_m) = get_volume_state();
        let icon_name = match true {
            _ if is_m => "volume-mute",
            _ if vol_pct > 66.0 => "volume",
            _ if vol_pct > 33.0 => "volume-medium",
            _ => "volume-low",
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
            rows.push(TooltipRow::new("Device", &compact_device_name(dev), None));
        }

        let card = TooltipPopover::build_card("Audio Output", &rows);
        vol_popover_c.set_child(Some(&card));
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_device_names_are_kept_intact() {
        assert_eq!(compact_device_name("  Built-in Audio  "), "Built-in Audio");
        assert_eq!(compact_device_name("HDMI"), "HDMI");
    }

    #[test]
    fn long_device_names_are_shortened_with_ellipsis() {
        let long = "A very long output device name that should not stretch the tooltip card";
        let compact = compact_device_name(long);
        assert_eq!(compact.chars().count(), MAX_DEVICE_CHARS);
        assert!(compact.ends_with('…'));
        assert!(long.starts_with(&compact[..compact.len() - '…'.len_utf8()]));
    }
}
