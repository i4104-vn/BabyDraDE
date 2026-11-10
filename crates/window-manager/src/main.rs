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

pub fn run(test_app: Option<OsString>) -> Result<(), Box<dyn Error>> {
    ensure_runtime_dir();
    services::config::ensure_babydra_dirs();

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

    // Autostart BabyDraDE widgets binary
    let exe_path = std::env::current_exe()
        .ok()
        .or_else(|| std::env::args_os().next().map(std::path::PathBuf::from))
        .unwrap_or_else(|| std::path::PathBuf::from("window-manager"));

    let widgets_exe = exe_path.with_file_name("widgets");
    let cmd = if widgets_exe.exists() {
        widgets_exe
    } else {
        std::path::PathBuf::from("widgets")
    };

    let widgets = ["desktop", "topbar"];
    for widget in widgets {
        match Command::new(&cmd)
            .arg(widget)
            .env("WAYLAND_DISPLAY", &data.state.socket_name)
            .env("GDK_BACKEND", "wayland")
            .env("ELECTRON_OZONE_PLATFORM_HINT", "auto")
            .env("MOZ_ENABLE_WAYLAND", "1")
            .env("QT_QPA_PLATFORM", "wayland;xcb")
            .env("XDG_CURRENT_DESKTOP", "BabyDra")
            .spawn()
        {
            Ok(_child) => info!("Started BabyDraDE widget '{widget}'"),
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

fn main() -> Result<(), Box<dyn Error>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    let mut args = std::env::args_os().skip(1);

    // Multi-call dispatch: if invoked with __widget, forward to widgets executable
    if let Some(first_arg) = args.next() {
        if first_arg == "__widget" {
            let widget_name = args
                .next()
                .ok_or_else(|| std::io::Error::other("Missing widget name for __widget"))?;
            let exe_path = std::env::current_exe().unwrap_or_default();
            let widgets_exe = exe_path.with_file_name("widgets");
            let cmd = if widgets_exe.exists() {
                widgets_exe
            } else {
                std::path::PathBuf::from("widgets")
            };
            let status = Command::new(cmd).arg(widget_name).status()?;
            std::process::exit(status.code().unwrap_or(0));
        }

        // CLI parsing for window manager
        let mut app: Option<OsString> = None;
        let mut current = Some(first_arg);

        while let Some(arg) = current.take().or_else(|| args.next()) {
            match arg.to_str() {
                Some("--help" | "-h") => {
                    println!("BabyDraDE Window Manager");
                    println!("Usage: window-manager [--app EXECUTABLE]");
                    println!("BABYDRADE_TEST_APP may also select the test app.");
                    return Ok(());
                }
                Some("--app") => {
                    app = args.next();
                    if app.is_none() {
                        return Err(std::io::Error::other("--app requires an executable name").into());
                    }
                }
                _ => {
                    return Err(std::io::Error::other(format!(
                        "unknown argument: {}",
                        arg.to_string_lossy()
                    ))
                    .into());
                }
            }
        }

        run(app)
    } else {
        run(None)
    }
}
