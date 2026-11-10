//! BabyDraDE Multi-call Widgets Executable

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    let mut args = std::env::args().skip(1);
    let mut widget_name = args.next().unwrap_or_else(|| "topbar".to_string());
    if widget_name == "__widget" {
        widget_name = args.next().unwrap_or_else(|| "topbar".to_string());
    }
    let clean_name = widget_name.strip_prefix("babydra-").unwrap_or(&widget_name);
    widgets::run_widget(clean_name)
}

