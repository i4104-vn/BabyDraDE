mod backends;
mod compositor;
pub mod rendering;

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

fn resolve_test_app(explicit: Option<OsString>) -> OsString {
    if let Some(app) = explicit {
        return app;
    }
    if let Some(app) = std::env::var_os("BABYDRADE_TEST_APP") {
        return app;
    }

    let candidates = [
        "foot",
        "alacritty",
        "kitty",
        "weston-terminal",
        "gnome-terminal",
        "xfce4-terminal",
        "konsole",
        "xterm",
        "urxvt",
        "st",
    ];

    if let Some(paths) = std::env::var_os("PATH") {
        for candidate in candidates {
            for dir in std::env::split_paths(&paths) {
                if dir.join(candidate).is_file() {
                    return OsString::from(candidate);
                }
            }
        }
    }

    OsString::from("foot")
}

pub fn run(test_app: Option<OsString>) -> Result<(), Box<dyn Error>> {
    ensure_runtime_dir();

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

    let app = resolve_test_app(test_app);
    match Command::new(&app).spawn() {
        Ok(_child) => info!("Started test app: {}", app.to_string_lossy()),
        Err(err) => warn!(
            "Could not start test app '{}': {}. Install a terminal (e.g. alacritty/foot) or set BABYDRADE_TEST_APP.",
            app.to_string_lossy(),
            err
        ),
    }

    info!(
        "BabyDraDE is running. Wayland socket: {}",
        data.state.socket_name.to_string_lossy()
    );
    event_loop.run(None, &mut data, |_| {})?;
    Ok(())
}
