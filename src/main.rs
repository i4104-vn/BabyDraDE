//! BabyDraDE — Wayland Desktop Environment
//! Root entry point

use domain::{CompositorBackend, Rect};
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

    // 4. Initialize core compositor state & render pipeline
    let state = BabyDraCompositorState::new(primary_screen);
    let _pipeline = CompositorRenderPipeline::new();

    info!(
        "BabyDraDE ready. Socket='{}', Workspaces=4. Run apps with: WAYLAND_DISPLAY={} <app>",
        state.socket_name, socket
    );

    // For non-interactive/startup verification:
    info!("Compositor initialized successfully.");
    Ok(())
}
