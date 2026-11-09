mod backends;
mod compositor;
pub mod config;
pub mod desktop;
pub mod i18n;
pub mod rendering;
pub mod settings {
    pub use crate::config::*;
}
pub mod system;
pub mod theme;
pub mod tray;
pub mod wallpaper;

pub use i18n::trans;

use std::{error::Error, ffi::OsString, process::Command};

use smithay::reexports::{calloop::EventLoop, wayland_server::Display};
use tracing::{info, warn};

use compositor::{BabydraState, CalloopData};

fn ensure_runtime_dir() {
    if std::env::var_os("XDG_RUNTIME_DIR").is_none() {
        let uid = std::env::var("UID")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "1000".into());
        let fallback = std::env::temp_dir().join(format!("babydrade-runtime-{}", uid));
        let _ = std::fs::create_dir_all(&fallback);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&fallback, std::fs::Permissions::from_mode(0o700));
        }

        std::env::set_var("XDG_RUNTIME_DIR", &fallback);
        info!("XDG_RUNTIME_DIR not set; using fallback: {:?}", fallback);
    }
}

pub fn run(test_app: Option<OsString>) -> Result<(), Box<dyn Error>> {
    ensure_runtime_dir();
    config::ensure_babydra_dirs();

    if std::env::var_os("WAYLAND_DISPLAY").is_none()
        && std::env::var_os("WAYLAND_SOCKET").is_none()
        && std::env::var_os("DISPLAY").is_none()
    {
        info!("No host display detected; starting autonomous Udev/DRM hardware backend for direct TTY scanout.");
    }

    let mut event_loop: EventLoop<CalloopData> = EventLoop::try_new()?;
    let display: Display<BabydraState> = Display::new()?;
    let display_handle = display.handle();
    let state = BabydraState::new(&mut event_loop, display);

    let mut data = CalloopData {
        state,
        display_handle,
    };

    backends::init(&mut event_loop, &mut data)?;

    // Export standard Wayland & desktop environment variables to compositor process and all child processes
    let socket_str = data.state.socket_name.to_string_lossy().to_string();
    std::env::set_var("WAYLAND_DISPLAY", &data.state.socket_name);
    std::env::set_var("ELECTRON_OZONE_PLATFORM_HINT", "auto");
    std::env::set_var("MOZ_ENABLE_WAYLAND", "1");
    std::env::set_var("QT_QPA_PLATFORM", "wayland;xcb");
    std::env::set_var("GDK_BACKEND", "wayland,x11,*");
    std::env::set_var("XDG_CURRENT_DESKTOP", "BabyDra");
    std::env::set_var("XDG_SESSION_DESKTOP", "BabyDra");
    std::env::set_var("XDG_SESSION_TYPE", "wayland");

    // Propagate Wayland socket and desktop session to DBus activation environment and systemd
    let _ = Command::new("dbus-update-activation-environment")
        .arg("--systemd")
        .arg(format!("WAYLAND_DISPLAY={}", socket_str))
        .arg("ELECTRON_OZONE_PLATFORM_HINT=auto")
        .arg("MOZ_ENABLE_WAYLAND=1")
        .arg("QT_QPA_PLATFORM=wayland;xcb")
        .arg("GDK_BACKEND=wayland,x11,*")
        .arg("XDG_CURRENT_DESKTOP=BabyDra")
        .arg("XDG_SESSION_DESKTOP=BabyDra")
        .arg("XDG_SESSION_TYPE=wayland")
        .spawn();

    // Autostart BabyDraDE widgets from unified binary (always runs by default, no env check)
    let exe_path = std::env::current_exe()
        .ok()
        .or_else(|| std::env::args_os().next().map(std::path::PathBuf::from))
        .unwrap_or_else(|| std::path::PathBuf::from("babydrade"));

    let widgets = ["desktop", "topbar"];
    for widget in widgets {
        match Command::new(&exe_path)
            .arg("__widget")
            .arg(widget)
            .env("WAYLAND_DISPLAY", &data.state.socket_name)
            .env("GDK_BACKEND", "wayland")
            .env("ELECTRON_OZONE_PLATFORM_HINT", "auto")
            .env("MOZ_ENABLE_WAYLAND", "1")
            .env("QT_QPA_PLATFORM", "wayland;xcb")
            .env("XDG_CURRENT_DESKTOP", "BabyDra")
            .spawn()
        {
            Ok(_child) => info!("Started BabyDraDE widget '{widget}' from unified binary"),
            Err(err) => warn!("Could not start BabyDraDE widget '{widget}': {err}"),
        }
    }

    // Only launch a test app if explicitly requested via CLI argument or BABYDRADE_TEST_APP environment variable.
    // Do NOT automatically spawn a terminal on startup.
    if let Some(app) = test_app.or_else(|| std::env::var_os("BABYDRADE_TEST_APP")) {
        match Command::new(&app)
            .env("WAYLAND_DISPLAY", &data.state.socket_name)
            .env("ELECTRON_OZONE_PLATFORM_HINT", "auto")
            .env("MOZ_ENABLE_WAYLAND", "1")
            .env("QT_QPA_PLATFORM", "wayland;xcb")
            .env("GDK_BACKEND", "wayland,x11,*")
            .spawn()
        {
            Ok(_child) => info!("Started test app: {}", app.to_string_lossy()),
            Err(err) => warn!(
                "Could not start test app '{}': {}",
                app.to_string_lossy(),
                err
            ),
        }
    }

    info!(
        "BabyDraDE is running. Wayland socket: {}",
        data.state.socket_name.to_string_lossy()
    );
    event_loop.run(None, &mut data, |_| {})?;
    Ok(())
}
