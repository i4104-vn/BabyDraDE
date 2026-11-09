use std::{error::Error, ffi::OsString, fs, path::PathBuf};

include!(concat!(env!("OUT_DIR"), "/embedded_widgets.rs"));

fn unpack_embedded_widgets() {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"));
    let target_bin_dir = runtime_dir.join("babydrade").join("bin");

    for &(name, bytes) in EMBEDDED_WIDGETS {
        if bytes.is_empty() {
            continue;
        }
        // If the binary already exists in ./bin, no need to unpack
        if PathBuf::from("bin").join(name).is_file() {
            continue;
        }
        let _ = fs::create_dir_all(&target_bin_dir);
        let widget_path = target_bin_dir.join(name);
        if !widget_path.is_file() {
            if fs::write(&widget_path, bytes).is_ok() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&widget_path, fs::Permissions::from_mode(0o755));
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    unpack_embedded_widgets();

    let mut args = std::env::args_os().skip(1);
    let mut app: Option<OsString> = None;

    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--help" | "-h") => {
                println!("BabyDraDE — minimal Smithay compositor");
                println!("Usage: babydrade [--app EXECUTABLE]");
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

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    if let Err(err) = infrastructure::run(app) {
        tracing::error!("BabyDraDE stopped with error: {err}");
        std::process::exit(1);
    }
    Ok(())
}
