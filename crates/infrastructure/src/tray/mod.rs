//! System Tray Support

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayItem {
    pub service: String,
    pub path: String,
    pub icon_name: String,
    pub title: String,
}

pub fn get_tray_items() -> Vec<TrayItem> {
    // Scaffold for active status notifier items
    Vec::new()
}

pub fn activate_item(_service: &str, _path: &str, _x: i32, _y: i32, _is_menu: bool) {
}
