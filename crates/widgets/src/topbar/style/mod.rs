mod css;

pub const TOPBAR_CSS: &str = concat!(
    include_str!("css/base.css"),
    "\n",
    include_str!("css/left.css"),
    "\n",
    include_str!("css/island.css"),
    "\n",
    include_str!("css/right.css"),
);

pub fn load_topbar_style() {
    // 1. Load base reusable UI Kit styles
    ui_kit::load_ui_kit_styles();

    // 2. Load topbar-specific styles
    ui_kit::style::append_custom_css(TOPBAR_CSS);
}
