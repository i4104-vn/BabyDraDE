mod backends;
mod compositor;
mod keymap;
pub mod rendering;
mod screenshot;

pub use common as enums;

use std::error::Error;

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

pub fn run() -> Result<(), Box<dyn Error>> {
    ensure_runtime_dir();
    if let Err(err) = gtk4::init() {
        warn!("GTK4 initialization failed: {err}; app icons and GTK-based rendering will be unavailable.");
    } else {
        info!("GTK4 initialized.");
    }
    services::config::ensure_babydra_dirs();

    std::thread::Builder::new()
        .name("babydra-keymap".to_string())
        .spawn(keymap::run)
        .map_err(|err| format!("failed to start keymap daemon: {err}"))?;

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

    // Export standard Wayland & desktop environment variables
    std::env::set_var("WAYLAND_DISPLAY", &data.state.socket_name);
    std::env::set_var("ELECTRON_OZONE_PLATFORM_HINT", "auto");
    std::env::set_var("MOZ_ENABLE_WAYLAND", "1");
    std::env::set_var("QT_QPA_PLATFORM", "wayland;xcb");
    std::env::set_var("GDK_BACKEND", "wayland,x11,*");
    std::env::set_var("XDG_CURRENT_DESKTOP", "BabyDra");
    std::env::set_var("XDG_SESSION_DESKTOP", "BabyDra");
    std::env::set_var("XDG_SESSION_TYPE", "wayland");

    info!(
        "BabyDraDE is running. Wayland socket: {}",
        data.state.socket_name.to_string_lossy()
    );

    // Spawn autostart commands configured in ~/.babydra/babydra.conf
    let autostart_cfg = services::config::load_autostart_config();
    let mut autostart_children: Vec<std::process::Child> = Vec::new();
    for cmd_str in &autostart_cfg.commands {
        let trimmed = cmd_str.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        info!("Spawning autostart command: {trimmed}");
        match std::process::Command::new("sh")
            .arg("-c")
            .arg(trimmed)
            .env("WAYLAND_DISPLAY", &data.state.socket_name)
            .spawn()
        {
            Ok(child) => autostart_children.push(child),
            Err(err) => warn!("Failed to spawn autostart command '{trimmed}': {err}"),
        }
    }

    let run_res = event_loop.run(None, &mut data, |_| {});

    info!("BabyDraDE compositor stopping, terminating autostart processes...");
    for mut child in autostart_children {
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

    run()
}
