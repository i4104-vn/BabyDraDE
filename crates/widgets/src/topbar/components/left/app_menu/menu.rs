//! Context menu for active window lifecycle actions

use gtk4::Button;
use infrastructure::system::{
    close_window_by_app, get_app_display_name, minimize_active_window,
    toggle_fullscreen_active_window, ActiveWindowInfo,
};
use ui_kit::overlay::context_menu::ContextMenu;

/// Builds and displays the context menu for the currently focused application
pub fn show_window_action_menu(anchor: &Button, info: &ActiveWindowInfo) {
    if info.is_empty() {
        return;
    }

    let display_title = get_app_display_name(&info.app_id, &info.title);
    let app_id_close = info.app_id.clone();
    let title_close = info.title.clone();

    let menu = ContextMenu::new(anchor)
        .item_sensitive(&display_title, "info", false, || {})
        .separator()
        .item("Minimize Window", "eye-off", || {
            minimize_active_window();
        })
        .item("Toggle Fullscreen", "display", || {
            toggle_fullscreen_active_window();
        })
        .separator()
        .item("Close Window", "close", move || {
            close_window_by_app(&app_id_close, &title_close);
        });
    menu.popup();
}

