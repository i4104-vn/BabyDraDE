pub mod indicators;
pub mod popover;
pub mod render;

use gtk4::prelude::*;
use popover::{setup_status_popover, StatusPopoverContext};
use crate::topbar::components::right::control_center::{create_cc_window, WindowSlot};

pub fn create_status_icons(
    app: &gtk4::Application,
    cc_slot: WindowSlot,
    cal_slot: WindowSlot,
) -> gtk4::Box {
    let (
        status_box,
        status_button,
        separator,
        vol_icon,
        net_container,
        vpn_icon,
        bat_widget,
    ) = render::build_status_row();

    let popovers = setup_status_popover(StatusPopoverContext {
        volume_icon: &vol_icon,
        network_container: &net_container,
        vpn_icon: &vpn_icon,
        battery: &bat_widget,
    });

    let app_c = app.clone();
    let cc_inner = cc_slot.clone();
    let cal_inner = cal_slot.clone();
    let vol_icon_c = vol_icon.clone();
    let popovers_c = popovers;

    status_button.connect_clicked(move |_| {
        popovers_c.popdown_all();

        if let Some(win) = cal_inner.borrow().clone() {
            win.close();
        }
        if let Some(win) = cc_inner.borrow().clone() {
            win.close();
        } else {
            let new_win = create_cc_window(&app_c, cc_inner.clone(), vol_icon_c.clone());
            if let Ok(mut borrow) = cc_inner.try_borrow_mut() {
                *borrow = Some(new_win);
            }
        }
    });

    let clock_button = super::clock::create_clock_widget(app, cc_slot, cal_slot);

    status_box.append(&status_button);
    status_box.append(&separator);
    status_box.append(&clock_button);
    status_box
}

