mod backends;
mod compositor;
mod extensions;
mod keymap;
pub mod rendering;

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

    // The clipboard capture thread reports back through calloop so the
    // compositor can take ownership of the selection on the main thread.
    let (clipboard_tx, clipboard_rx) =
        smithay::reexports::calloop::channel::channel::<Option<String>>();
    data.state.clipboard_sender = Some(clipboard_tx);
    event_loop
        .handle()
        .insert_source(clipboard_rx, |event, _, data| {
            if let smithay::reexports::calloop::channel::Event::Msg(mime) = event {
                data.state.complete_clipboard_capture(mime);
            }
        })
        .expect("failed to register clipboard completion channel");

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

fn init_tracing() {
    use tracing_subscriber::fmt::writer::MakeWriterExt;

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    // Keep the TTY output but also persist logs so crashes on a bare console
    // can be inspected afterwards.
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let log_dir = std::path::PathBuf::from(home).join(".babydra").join("logs");
    let file_writer = std::fs::create_dir_all(&log_dir)
        .ok()
        .and_then(|_| {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_dir.join("window-manager.log"))
                .ok()
        });

    let subscriber = tracing_subscriber::fmt().with_env_filter(filter);
    match file_writer {
        Some(file) => {
            let _ = subscriber.with_writer(std::io::stdout.and(file)).try_init();
        }
        None => {
            let _ = subscriber.try_init();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    init_tracing();

    // The screenshot editor needs a GTK main context of its own. Running it in
    // a thread of the compositor process is impossible because the default
    // GLib main context is already acquired there, so it runs in a subprocess.
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 && args[1] == "--screenshot-editor" {
        extensions::screenshot::open_editor(args[2].clone());
        return Ok(());
    }
    if args.len() == 2 && args[1] == "--clipboard-picker" {
        extensions::clipboard::open_picker();
        return Ok(());
    }

    run()
}
