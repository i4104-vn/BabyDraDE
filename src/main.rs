//! BabyDraDE — Wayland Desktop Environment
//! Root entry point

use domain::{AppRegistry, CompositorBackend, Rect};
use infrastructure::{BabyDraCompositorState, WaylandServer, WinitBackend};
use presentation::CompositorRenderPipeline;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

fn main() -> anyhow::Result<()> {
    // 1. Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    info!("Starting BabyDraDE (Wayland Desktop Environment)...");

    // 2. Initialize display backend (Winit for development mode)
    let mut backend = WinitBackend::new();
    backend.initialize().map_err(|e| anyhow::anyhow!(e))?;
    let outputs = backend.outputs();
    let primary_screen = outputs
        .first()
        .map(|o| o.geometry)
        .unwrap_or(Rect::new(0, 0, 1920, 1080));

    info!(
        "Backend '{}' initialized with {} display output(s). Screen: {:?}",
        backend.name(),
        outputs.len(),
        primary_screen
    );

    // 3. Initialize Wayland server and bind socket
    let mut wayland_server = WaylandServer::new("me.babydra.compositor");
    wayland_server.register_core_globals();
    let socket = wayland_server
        .bind_socket()
        .map_err(|e| anyhow::anyhow!(e))?;
    info!("Wayland compositor listening on WAYLAND_DISPLAY={}", socket);

    // 4. Initialize desktop app scanner & launcher overlay
    let mut app_scanner = infrastructure::DesktopFileScanner::new();
    let apps = app_scanner.refresh().unwrap_or_default();
    info!("Scanned {} application(s) for App Launcher.", apps.len());

    let mut launcher = presentation::LauncherOverlay::new();
    launcher.open(&app_scanner);

    // 5. Initialize core compositor state & render pipeline
    let state = BabyDraCompositorState::new(primary_screen);
    let pipeline = CompositorRenderPipeline::new();

    // 6. Demonstrate frame rendering with active workspace
    let mut renderer = infrastructure::GlRenderer::new();
    let active_ws_windows = state.workspace_manager.current_workspace().windows();
    pipeline
        .render_frame_full(
            &mut renderer,
            &state.window_manager,
            Some(active_ws_windows),
            Some(&launcher),
            primary_screen,
            None,
        )
        .map_err(|e| anyhow::anyhow!(e))?;

    info!(
        "BabyDraDE ready. Socket='{}', Workspaces=4. Run apps with: WAYLAND_DISPLAY={} <app>",
        state.socket_name, socket
    );

    // For non-interactive/startup verification:
    info!("Compositor initialized successfully.");
    Ok(())
}
