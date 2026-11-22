//! Environment and toolkit bootstrap for the compositor process.

use std::ffi::OsStr;
use tracing::{info, warn};

use crate::platform::brand;
use tracing_subscriber::fmt::writer::MakeWriterExt;

/// Initializes tracing to stdout and the per-user log file.
pub fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let log_dir = services::config::get_logs_dir();
    let file_writer = std::fs::create_dir_all(&log_dir).ok().and_then(|_| {
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

/// Ensures `XDG_RUNTIME_DIR` exists before any Wayland socket is created.
pub fn ensure_runtime_dir() {
    if std::env::var_os("XDG_RUNTIME_DIR").is_none() {
        let uid = std::env::var("UID")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "1000".into());
        let fallback = std::env::temp_dir().join(format!("{}-runtime-{uid}", brand::ID));
        if let Err(err) = std::fs::create_dir_all(&fallback) {
            warn!("cannot create runtime dir {}: {err}", fallback.display());
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&fallback, std::fs::Permissions::from_mode(0o700));
        }

        // SAFETY: called before GTK, Wayland sockets and any spawned thread.
        std::env::set_var("XDG_RUNTIME_DIR", &fallback);
        info!("XDG_RUNTIME_DIR not set; using fallback: {:?}", fallback);
    }
}

/// Exports Wayland and desktop environment variables before GTK starts.
pub fn export_env(socket_name: &OsStr) {
    // SAFETY: called from `app::run` before GTK initialization or threads.
    std::env::set_var("WAYLAND_DISPLAY", socket_name);
    // SAFETY: see above.
    std::env::set_var("ELECTRON_OZONE_PLATFORM_HINT", "auto");
    // SAFETY: see above.
    std::env::set_var("MOZ_ENABLE_WAYLAND", "1");
    // SAFETY: see above.
    std::env::set_var("QT_QPA_PLATFORM", "wayland;xcb");
    // SAFETY: see above.
    std::env::set_var("GDK_BACKEND", "wayland,x11,*");
    // SAFETY: see above.
    std::env::set_var("XDG_CURRENT_DESKTOP", brand::DISPLAY_NAME);
    // SAFETY: see above.
    std::env::set_var("XDG_SESSION_DESKTOP", brand::DISPLAY_NAME);
    // SAFETY: see above.
    std::env::set_var("XDG_SESSION_TYPE", "wayland");
}

/// Initializes GTK after the environment is fully configured.
pub fn init_gtk() {
    match gtk4::init() {
        Ok(()) => info!("GTK4 initialized."),
        Err(err) => warn!(
            "GTK4 initialization failed: {err}; app icons and GTK-based rendering will be unavailable."
        ),
    }
}
