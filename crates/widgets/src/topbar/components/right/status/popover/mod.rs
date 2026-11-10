pub mod battery;
pub mod network;
pub mod volume;

use gtk4::glib;
use gtk4::prelude::*;
use services::system::is_vpn_active;
use ui_kit::components::overlay::popover::tooltip::TooltipPopover;

#[derive(Clone)]
pub struct StatusPopovers {
    pub vpn_popover: gtk4::Popover,
    pub net_popover: gtk4::Popover,
    pub vol_popover: gtk4::Popover,
    pub bat_popover_opt: Option<gtk4::Popover>,
}

impl StatusPopovers {
    pub fn popdown_all(&self) {
        self.vpn_popover.popdown();
        self.net_popover.popdown();
        self.vol_popover.popdown();
        if let Some(ref pop) = self.bat_popover_opt {
            pop.popdown();
        }
    }
}

pub struct StatusPopoverContext<'a> {
    pub volume_icon: &'a gtk4::Image,
    pub network_container: &'a gtk4::Box,
    pub vpn_icon: &'a gtk4::Image,
    pub battery: &'a Option<gtk4::DrawingArea>,
}

pub fn setup_status_popover(context: StatusPopoverContext<'_>) -> StatusPopovers {
    let vpn_tooltip = TooltipPopover::new(context.vpn_icon, gtk4::PositionType::Bottom);
    let net_tooltip = TooltipPopover::new(context.network_container, gtk4::PositionType::Bottom);
    let vol_tooltip = TooltipPopover::new(context.volume_icon, gtk4::PositionType::Bottom);

    let (bat_tooltip_opt, bat_popover_opt) = if let Some(bat_area) = context.battery {
        let bat_pop = TooltipPopover::new(bat_area, gtk4::PositionType::Bottom);
        let pop = bat_pop.popover.clone();
        (Some(bat_pop), Some(pop))
    } else {
        (None, None)
    };

    let update_network_tooltip = network::build_network_update(&net_tooltip.popover);
    let update_volume_popover = volume::build_volume_update(context.volume_icon, &vol_tooltip.popover);
    let update_battery_popover = battery::build_battery_update(&bat_popover_opt);

    net_tooltip.attach_hover(context.network_container, Some(update_network_tooltip.clone()));
    vol_tooltip.attach_hover(context.volume_icon, Some(update_volume_popover.clone()));

    if let Some(bat_area) = context.battery {
        if let Some(bat_tt) = &bat_tooltip_opt {
            bat_tt.attach_hover(bat_area, Some(update_battery_popover.clone()));
        }
    }

    // Timer loop for periodic update
    let net_pop_t = net_tooltip.popover.clone();
    let vol_pop_t = vol_tooltip.popover.clone();
    let bat_pop_t = bat_popover_opt.clone();
    let update_net_t = update_network_tooltip.clone();
    let update_vol_t = update_volume_popover.clone();
    let update_bat_t = update_battery_popover.clone();
    let vpn_icon_timer = context.vpn_icon.clone();
    let bat_widget_timer = context.battery.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(2000), move || {
        if net_pop_t.is_visible() {
            update_net_t();
        }
        if vol_pop_t.is_visible() {
            update_vol_t();
        }
        if let Some(ref bp) = bat_pop_t {
            if bp.is_visible() {
                update_bat_t();
            }
        }

        let vpn_active = is_vpn_active();
        if vpn_icon_timer.is_visible() != vpn_active {
            vpn_icon_timer.set_visible(vpn_active);
        }

        if let Some(ref bat_area) = bat_widget_timer {
            bat_area.queue_draw();
        }

        glib::ControlFlow::Continue
    });

    StatusPopovers {
        vpn_popover: vpn_tooltip.popover,
        net_popover: net_tooltip.popover,
        vol_popover: vol_tooltip.popover,
        bat_popover_opt,
    }
}
