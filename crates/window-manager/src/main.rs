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

    let mut spawned_children: Vec<std::process::Child> = Vec::new();

    // Only launch a test app if explicitly requested via CLI argument or BABYDRADE_TEST_APP environment variable.
    // Do NOT automatically spawn a terminal on startup.
    if let Some(app) = test_app.or_else(|| std::env::var_os("BABYDRADE_TEST_APP")) {
        let mut command = Command::new(&app);
        command
            .env("WAYLAND_DISPLAY", &data.state.socket_name)
            .env("ELECTRON_OZONE_PLATFORM_HINT", "auto")
            .env("MOZ_ENABLE_WAYLAND", "1")
            .env("QT_QPA_PLATFORM", "wayland;xcb")
            .env("GDK_BACKEND", "wayland,x11,*");

        #[cfg(target_os = "linux")]
        unsafe {
            use std::os::unix::process::CommandExt;
            command.pre_exec(|| {
                libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
                Ok(())
            });
        }

        match command.spawn() {
            Ok(child) => {
                info!("Started test app: {} (PID: {})", app.to_string_lossy(), child.id());
                spawned_children.push(child);
            }
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

    let run_res = event_loop.run(None, &mut data, |_| {});

    info!("BabyDraDE compositor stopping, terminating child widgets...");
    for mut child in spawned_children {
        let _ = child.kill();
        let _ = child.wait();
    }

    run_res?;
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
            let widget_str = widget_name.to_string_lossy();
            let clean_name = widget_str.strip_prefix("babydra-").unwrap_or(&widget_str);
            let bin_name = format!("babydra-{}", clean_name);
            let exe_path = std::env::current_exe().unwrap_or_default();
            let same_dir_bin = exe_path.with_file_name(&bin_name);
            let (cmd, cmd_args): (std::path::PathBuf, Vec<String>) = if same_dir_bin.is_file() {
                (same_dir_bin, Vec::new())
            } else {
                let same_dir_widgets = exe_path.with_file_name("babydra-widgets");
                if same_dir_widgets.is_file() {
                    (same_dir_widgets, vec![clean_name.to_string()])
                } else {
                    (std::path::PathBuf::from(&bin_name), Vec::new())
                }
            };
            let mut command = Command::new(cmd);
            for arg in cmd_args {
                command.arg(arg);
            }
            let status = command.status()?;
            std::process::exit(status.code().unwrap_or(0));
        }

        // CLI parsing for window manager
        let mut app: Option<OsString> = None;
        let mut current = Some(first_arg);

        while let Some(arg) = current.take().or_else(|| args.next()) {
            match arg.to_str() {
                Some("--help" | "-h") => {
                    println!("BabyDraDE Window Manager");
                    println!("Usage: babydra-window-manager [OPTIONS]");
                    println!();
                    println!("Options:");
                    println!("    --app EXECUTABLE       Launch a test application on startup");
                    println!("    --blur-apps APPS       Comma-separated list of app IDs to blur (e.g. 'foot,kitty')");
                    println!("    --blur-all             Enable background blur on all windows");
                    println!("    --no-blur              Disable background blur engine");
                    println!("    -h, --help             Show this help message");
                    println!();
                    println!("Shortcuts:");
                    println!("    Super + B              Toggle blur on the currently active window");
                    println!();
                    println!("Environment variables:");
                    println!("    BABYDRADE_BLUR_APPS    Comma-separated list of app IDs to blur");
                    println!("    BABYDRADE_BLUR_ALL     Set to 1 or true to blur all windows");
                    println!("    BABYDRADE_BLUR_RADIUS  Blur radius (default 18.0)");
                    println!("    BABYDRADE_BLUR_OPACITY Window opacity (default 0.68)");
                    return Ok(());
                }
                Some("--app") => {
                    app = args.next();
                    if app.is_none() {
                        return Err(std::io::Error::other("--app requires an executable name").into());
                    }
                }
                Some("--blur-apps") => {
                    if let Some(apps) = args.next() {
                        std::env::set_var("BABYDRADE_BLUR_APPS", apps);
                    } else {
                        return Err(std::io::Error::other("--blur-apps requires a comma-separated list of apps").into());
                    }
                }
                Some("--blur-all") => {
                    std::env::set_var("BABYDRADE_BLUR_ALL", "1");
                }
                Some("--no-blur") => {
                    std::env::set_var("BABYDRADE_BLUR_ALL", "0");
                    std::env::set_var("BABYDRADE_BLUR_APPS", "");
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

