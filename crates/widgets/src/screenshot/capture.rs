//! Screenshot capture via grim and filesystem persistence.

use std::path::PathBuf;
use std::process::Command;

/// Resolves the default file path to save screenshots in `~/Pictures/Screenshots/`.
pub fn get_screenshot_path() -> PathBuf {
    let pictures_dir = dirs::picture_dir().unwrap_or_else(|| {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home.join("Pictures")
    });
    let screenshots_dir = pictures_dir.join("Screenshots");
    let _ = std::fs::create_dir_all(&screenshots_dir);

    let datetime = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    screenshots_dir.join(format!("Screenshot_{}.png", datetime))
}

/// Invokes `grim` to capture the Wayland display screen losslessly and save it to a temporary file.
pub fn capture_screen() -> Option<String> {
    let temp_path = "/tmp/babydra-screenshot-temp.png";
    let _ = std::fs::remove_file(temp_path);

    let status = Command::new("grim")
        .args(["-t", "png", "-l", "1", temp_path])
        .status();

    match status {
        Ok(s) if s.success() => Some(temp_path.to_string()),
        _ => None,
    }
}

/// Performs a fullscreen screenshot capture, directly preserving 100% bit-exact PNG file from grim.
pub fn capture_fullscreen() -> bool {
    if let Some(temp_path) = capture_screen() {
        let save_path = get_screenshot_path();
        if std::fs::copy(&temp_path, &save_path).is_ok() {
            let notif_title = services::trans("screenshot.full_saved_title");
            let notif_msg =
                services::trans("screenshot.saved_msg").replace("{}", &format!("{:?}", save_path));

            send_notification(&notif_title, &notif_msg);
            let _ = std::fs::remove_file(temp_path);
            return true;
        }
        let _ = std::fs::remove_file(temp_path);
    }
    false
}

/// Dispatches desktop notification.
pub fn send_notification(title: &str, body: &str) {
    let _ = Command::new("notify-send")
        .args(["-a", "BabyDra", title, body])
        .spawn();
}
