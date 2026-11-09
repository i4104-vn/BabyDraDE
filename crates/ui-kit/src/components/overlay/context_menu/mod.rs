//! Context Menu module for BabyDraDE UI Kit.

pub mod builder;
pub mod items;

pub use builder::{ContextMenu, ContextMenuBuilder};
pub use items::{
    create_danger_btn, create_danger_item, create_footer_box, create_footer_btn,
    create_group_header, create_menu_box, create_menu_for, create_menu_full, create_menu_item,
    create_menu_item_gicon, create_menu_item_resolved, create_menu_popover, create_menu_sens,
    create_menu_sep, create_menu_shortcut, create_menu_text, create_submenu_item,
    create_submenu_popover, resolve_icon,
};

