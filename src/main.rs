use std::{error::Error, ffi::OsString};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let mut app: Option<OsString> = None;

    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            println!("BabyDraDE — minimal Smithay compositor");
            println!("Usage: babydrade [--app EXECUTABLE]");
            println!("BABYDRADE_TEST_APP may also select the test app.");
            return Ok(());
        }

        if arg == "--app" {
            app = args.next();
            if app.is_none() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "--app requires an executable name",
                )
                .into());
            }
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("unknown argument: {}", arg.to_string_lossy()),
            )
            .into());
        }
    }

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    infrastructure::run(app)
}
