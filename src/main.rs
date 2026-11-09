use std::{error::Error, ffi::OsString};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);

    // Multi-call internal widget dispatcher:
    // When invoked by the compositor as: babydrade __widget <name>
    if let Some(first_arg) = args.next() {
        if first_arg == "__widget" {
            let widget_name = args
                .next()
                .ok_or_else(|| std::io::Error::other("Missing widget name for __widget"))?;
            return widgets::run_widget(&widget_name.to_string_lossy());
        }

        // Normal CLI parsing for DE compositor
        let mut app: Option<OsString> = None;
        let mut current = Some(first_arg);

        while let Some(arg) = current.take().or_else(|| args.next()) {
            match arg.to_str() {
                Some("--help" | "-h") => {
                    println!("BabyDraDE — unified Wayland Desktop Environment");
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

        start_compositor(app)
    } else {
        start_compositor(None)
    }
}

fn start_compositor(app: Option<OsString>) -> Result<(), Box<dyn Error>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    if let Err(err) = infrastructure::run(app) {
        tracing::error!("BabyDraDE stopped with error: {err}");
        std::process::exit(1);
    }
    Ok(())
}
