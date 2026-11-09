pub mod popover;
pub mod render;

use gtk4::prelude::*;
use popover::{setup_status_popover, StatusPopoverContext};

pub fn create_status_icons() -> gtk4::Box {
    let (
        status_box,
        status_button,
        separator,
        vol_icon,
        net_container,
        vpn_icon,
        bat_widget,
    ) = render::build_status_row();

    let _popovers = setup_status_popover(StatusPopoverContext {
        volume_icon: &vol_icon,
        network_container: &net_container,
        vpn_icon: &vpn_icon,
        battery: &bat_widget,
    });

    status_box.append(&status_button);
    status_box.append(&separator);
    status_box
}

