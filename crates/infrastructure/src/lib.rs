mod backends;
mod compositor;
pub mod rendering;
pub mod system;
pub mod theme;

use std::{
    collections::HashSet,
    error::Error,
    ffi::OsString,
    path::PathBuf,
    process::Command,
};

use smithay::reexports::{calloop::EventLoop, wayland_server::Display};
use tracing::{info, warn};

use compositor::{input::launcher::resolve_test_app, BabydraState, CalloopData};

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

fn autostart_widgets(socket_name: &str) {
    let mut launched_any = false;
    let mut candidate_dirs: Vec<PathBuf> = Vec::new();

    // 1. Current working directory ./bin
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join("bin");
        if p.is_dir() {
            candidate_dirs.push(p);
        }
    }

    // 2. Relative to current executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            if exe_dir.file_name().and_then(|n| n.to_str()) == Some("bin") {
                candidate_dirs.push(exe_dir.to_path_buf());
            } else {
                candidate_dirs.push(exe_dir.join("bin"));
                if let Some(parent) = exe_dir.parent().and_then(|p| p.parent()) {
                    candidate_dirs.push(parent.join("bin"));
                }
                candidate_dirs.push(exe_dir.to_path_buf());
            }
        }
    }

    // 3. Runtime directory fallback (where embedded widgets are extracted)
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let p = PathBuf::from(runtime_dir).join("babydrade").join("bin");
        if p.is_dir() {
            candidate_dirs.push(p);
        }
    }
    candidate_dirs.push(PathBuf::from("/tmp/babydrade/bin"));

    let mut launched_widgets = HashSet::new();

    for dir in candidate_dirs {
        if !dir.is_dir() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let file_name = entry.file_name().to_string_lossy().to_string();

                // Skip non-widgets (compositor itself, hidden files, or non-executables)
                if file_name == "babydrade"
                    || file_name == "babydrade.exe"
                    || file_name.starts_with('.')
                    || file_name.ends_with(".log")
                    || file_name.ends_with(".txt")
                    || file_name.ends_with(".gitkeep")
                {
                    continue;
                }

                if launched_widgets.contains(&file_name) {
                    continue;
                }

                match Command::new(&path)
                    .env("WAYLAND_DISPLAY", socket_name)
                    .env("GDK_BACKEND", "wayland")
                    .spawn()
                {
                    Ok(_child) => {
                        info!("Started BabyDraDE widget '{}': {:?}", file_name, path);
                        launched_widgets.insert(file_name);
                        launched_any = true;
                    }
                    Err(err) => {
                        warn!("Could not start widget at {:?}: {}", path, err);
                    }
                }
            }
        }
    }

    if !launched_any {
        warn!("No widget binaries found in bin/ or runtime directories to autostart.");
    }
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

    // Autostart all BabyDraDE widgets unconditionally (no env check, always runs by default)
    autostart_widgets(&data.state.socket_name);

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
