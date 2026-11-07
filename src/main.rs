//! BabyDraDE — Wayland Desktop Environment
//! Root entry point

use domain::{
    AppRegistry, CanvasRenderer, Color, DesktopEntry, Point, Rect, Size, WindowCommand, WindowId,
    WorkspaceCommand,
};
use infrastructure::{BabyDraCompositorState, SoftwareRenderer, WaylandServer};
use presentation::{CanvasBackground, CompositorRenderPipeline, DesktopCanvas};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

fn main() -> anyhow::Result<()> {
    // 1. Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    info!("Starting BabyDraDE (Wayland Desktop Environment)...");

    // 2. Initialize display backend and Desktop Canvas surface
    let mut renderer = SoftwareRenderer::auto_detect();
    let screen_rect = Rect::new(0, 0, renderer.width, renderer.height);
    let desktop_canvas = DesktopCanvas::new(screen_rect, CanvasBackground::default());

    info!(
        "Desktop Canvas initialized: {}x{} ({})",
        renderer.width,
        renderer.height,
        if renderer.fb_file.is_some() {
            "/dev/fb0 hardware display connected"
        } else {
            "Memory surface mode"
        }
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
    let mut apps = app_scanner.refresh().unwrap_or_default();
    if apps.is_empty() {
        // Fallback default applications for instant testing
        apps = vec![
            DesktopEntry::new("alacritty", "Alacritty Terminal", "alacritty"),
            DesktopEntry::new("firefox", "Firefox Web Browser", "firefox"),
            DesktopEntry::new("files", "File Manager", "thunar"),
            DesktopEntry::new("editor", "Text Editor", "gedit"),
            DesktopEntry::new("settings", "System Settings", "gnome-control-center"),
        ];
    }
    info!("App Launcher ready with {} application(s).", apps.len());

    let mut launcher = presentation::LauncherOverlay::new();

    // 5. Initialize core compositor state & render pipeline
    let mut state = BabyDraCompositorState::new(screen_rect);
    let pipeline = CompositorRenderPipeline::new();

    // 6. Pre-populate initial demo floating windows for immediate testing
    let win_term = state.allocate_window_id();
    state.handle_command(
        WindowCommand::OpenWindow {
            id: win_term,
            title: "Alacritty — Terminal".into(),
            app_id: Some("alacritty".into()),
            initial_size: Size::new(620, 380),
        }
        .into(),
    );
    state.handle_command(
        WindowCommand::SetWindowPosition {
            id: win_term,
            position: Point::new(80, 70),
        }
        .into(),
    );

    let win_info = state.allocate_window_id();
    state.handle_command(
        WindowCommand::OpenWindow {
            id: win_info,
            title: "BabyDraDE — System Info & Controls".into(),
            app_id: Some("babydrade".into()),
            initial_size: Size::new(520, 340),
        }
        .into(),
    );
    state.handle_command(
        WindowCommand::SetWindowPosition {
            id: win_info,
            position: Point::new(420, 180),
        }
        .into(),
    );

    // Check if running in non-interactive verification mode (CI / test checks)
    if std::env::var("BABYDRADE_VERIFY_ONLY").is_ok()
        || std::env::args().any(|a| a == "--verify-only")
    {
        info!("Running single-frame verification mode...");
        render_composite_frame(
            &mut renderer,
            &pipeline,
            &desktop_canvas,
            &state,
            &launcher,
            &socket,
        )?;
        info!("Compositor initialized successfully.");
        return Ok(());
    }

    info!(
        "BabyDraDE running. Workspaces=4, Socket='{}'. Press 'q' or 'Esc' to exit.",
        socket
    );

    // 7. Non-blocking input receiver thread
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        use std::io::Read;
        let mut stdin = std::io::stdin();
        let mut buf = [0u8; 16];
        while let Ok(n) = stdin.read(&mut buf) {
            if n == 0 {
                break;
            }
            for &b in &buf[..n] {
                if tx.send(b).is_err() {
                    return;
                }
            }
        }
    });

    let mut running = true;
    let mut last_render = Instant::now();

    // 8. Main Compositor Event & Render Loop (~40 FPS)
    while running && state.is_running {
        // Handle input events from channel
        while let Ok(byte) = rx.try_recv() {
            match byte {
                // 'q' or 'Q' -> Close focused window
                b'q' | b'Q' => {
                    if let Some(focused) = state.window_manager.focused_window_id() {
                        state.handle_command(WindowCommand::CloseWindow { id: focused }.into());
                    } else {
                        running = false;
                    }
                }
                // 'd' | 'D' -> Toggle App Launcher
                b'd' | b'D' => {
                    launcher.toggle(&app_scanner);
                }
                // Enter / Return -> Open terminal or launch app
                b'\r' | b'\n' => {
                    if launcher.visible {
                        if let Some(entry) = launcher.matched_entries.get(launcher.selected_index) {
                            let new_id = state.allocate_window_id();
                            state.handle_command(
                                WindowCommand::OpenWindow {
                                    id: new_id,
                                    title: entry.name.clone(),
                                    app_id: Some(entry.exec.clone()),
                                    initial_size: Size::new(560, 360),
                                }
                                .into(),
                            );
                        }
                        launcher.close();
                    } else {
                        // Open new terminal window
                        let new_id = state.allocate_window_id();
                        state.handle_command(
                            WindowCommand::OpenWindow {
                                id: new_id,
                                title: format!("Terminal #{}", new_id),
                                app_id: Some("alacritty".into()),
                                initial_size: Size::new(600, 380),
                            }
                            .into(),
                        );
                    }
                }
                // Tab -> Cycle window focus (MRU)
                b'\t' => {
                    let wins: Vec<WindowId> = state
                        .workspace_manager
                        .current_workspace()
                        .windows()
                        .to_vec();
                    if !wins.is_empty() {
                        let cur_focused = state.window_manager.focused_window_id();
                        let next_win = if let Some(cur) = cur_focused {
                            let idx = wins.iter().position(|&w| w == cur).unwrap_or(0);
                            wins[(idx + 1) % wins.len()]
                        } else {
                            wins[0]
                        };
                        state.handle_command(WindowCommand::FocusWindow { id: next_win }.into());
                    }
                }
                // 'm' | 'M' -> Toggle maximize
                b'm' | b'M' => {
                    if let Some(focused) = state.window_manager.focused_window_id() {
                        state.handle_command(WindowCommand::ToggleMaximize { id: focused }.into());
                    }
                }
                // 'h' | 'H' -> Minimize
                b'h' | b'H' => {
                    if let Some(focused) = state.window_manager.focused_window_id() {
                        state.handle_command(WindowCommand::MinimizeWindow { id: focused }.into());
                    }
                }
                // '1'..='4' -> Switch workspaces
                b'1'..=b'4' => {
                    let ws_id = (byte - b'0') as u32;
                    state.handle_command(
                        WorkspaceCommand::SwitchWorkspace {
                            workspace_id: ws_id,
                        }
                        .into(),
                    );
                }
                // Arrow navigation / window moving (ANSI escape sequences)
                b'A' => {
                    // Up
                    if let Some(focused) = state.window_manager.focused_window_id() {
                        state.handle_command(
                            WindowCommand::MoveWindow {
                                id: focused,
                                dx: 0,
                                dy: -25,
                            }
                            .into(),
                        );
                    }
                }
                b'B' => {
                    // Down
                    if let Some(focused) = state.window_manager.focused_window_id() {
                        state.handle_command(
                            WindowCommand::MoveWindow {
                                id: focused,
                                dx: 0,
                                dy: 25,
                            }
                            .into(),
                        );
                    }
                }
                b'C' => {
                    // Right
                    if let Some(focused) = state.window_manager.focused_window_id() {
                        state.handle_command(
                            WindowCommand::MoveWindow {
                                id: focused,
                                dx: 25,
                                dy: 0,
                            }
                            .into(),
                        );
                    }
                }
                b'D' => {
                    // Left
                    if let Some(focused) = state.window_manager.focused_window_id() {
                        state.handle_command(
                            WindowCommand::MoveWindow {
                                id: focused,
                                dx: -25,
                                dy: 0,
                            }
                            .into(),
                        );
                    }
                }
                // Esc -> Close launcher or exit compositor
                0x1b => {
                    if launcher.visible {
                        launcher.close();
                    } else {
                        running = false;
                    }
                }
                _ => {}
            }
        }

        // Render frame
        if last_render.elapsed() >= Duration::from_millis(25) {
            render_composite_frame(
                &mut renderer,
                &pipeline,
                &desktop_canvas,
                &state,
                &launcher,
                &socket,
            )?;
            last_render = Instant::now();
        }

        thread::sleep(Duration::from_millis(10));
    }

    info!("BabyDraDE shut down cleanly.");
    Ok(())
}

/// Renders the full desktop scene graph onto the Desktop Canvas
fn render_composite_frame(
    renderer: &mut SoftwareRenderer,
    pipeline: &CompositorRenderPipeline,
    canvas: &DesktopCanvas,
    state: &BabyDraCompositorState,
    launcher: &presentation::LauncherOverlay,
    socket: &str,
) -> Result<(), domain::RenderError> {
    let screen_rect = canvas.bounds;
    let active_ws_windows = state.workspace_manager.current_workspace().windows();

    // 1. Render background and window decorations via Clean Architecture pipeline
    pipeline.render_frame_full(
        renderer,
        &state.window_manager,
        Some(active_ws_windows),
        if launcher.visible { Some(launcher) } else { None },
        screen_rect,
        None,
    )?;

    // 2. Render Window Client Area contents (Simulated interactive app textures)
    for win in state.window_manager.windows() {
        if !win.visible || win.is_minimized() || !active_ws_windows.contains(&win.id) {
            continue;
        }

        let client_rect = pipeline.titlebar_decorator.client_rect(&win.geometry);
        if client_rect.width() < 50 || client_rect.height() < 50 {
            continue;
        }

        if win.title.contains("Terminal") {
            // Terminal client interior
            renderer.draw_rect(client_rect, Color::rgb(14, 16, 22))?;
            renderer.draw_text(
                "BabyDraDE Wayland Terminal [alacritty]",
                Point::new(client_rect.x() + 15, client_rect.y() + 15),
                14.0,
                Color::rgb(100, 180, 240),
            )?;
            renderer.draw_text(
                "babydra@de:~$ echo $WAYLAND_DISPLAY",
                Point::new(client_rect.x() + 15, client_rect.y() + 45),
                14.0,
                Color::WHITE,
            )?;
            renderer.draw_text(
                socket,
                Point::new(client_rect.x() + 15, client_rect.y() + 70),
                14.0,
                Color::rgb(120, 220, 140),
            )?;
            renderer.draw_text(
                "babydra@de:~$ uname -srm",
                Point::new(client_rect.x() + 15, client_rect.y() + 100),
                14.0,
                Color::WHITE,
            )?;
            renderer.draw_text(
                "Linux 6.8.0-generic x86_64",
                Point::new(client_rect.x() + 15, client_rect.y() + 125),
                14.0,
                Color::rgb(200, 205, 215),
            )?;
            renderer.draw_text(
                "babydra@de:~$ _",
                Point::new(client_rect.x() + 15, client_rect.y() + 160),
                14.0,
                Color::rgb(100, 255, 120),
            )?;
        } else if win.title.contains("System Info") {
            // System info dashboard interior
            renderer.draw_rect(client_rect, Color::rgb(22, 24, 34))?;
            renderer.draw_text(
                "BabyDraDE v0.1.0 (Wayland Floating DE)",
                Point::new(client_rect.x() + 15, client_rect.y() + 15),
                14.0,
                Color::rgb(255, 215, 0),
            )?;
            renderer.draw_text(
                &format!(
                    "Active Workspace: [{}] / 4",
                    state.workspace_manager.current_workspace_id()
                ),
                Point::new(client_rect.x() + 15, client_rect.y() + 45),
                14.0,
                Color::WHITE,
            )?;
            renderer.draw_text(
                &format!("Display: {}x{} Framebuffer", screen_rect.width(), screen_rect.height()),
                Point::new(client_rect.x() + 15, client_rect.y() + 70),
                14.0,
                Color::rgb(180, 190, 205),
            )?;
            renderer.draw_text(
                "----------------------------------------------",
                Point::new(client_rect.x() + 15, client_rect.y() + 95),
                14.0,
                Color::rgb(60, 65, 85),
            )?;
            renderer.draw_text(
                "Controls: [Enter] Terminal  |  [d] Launcher",
                Point::new(client_rect.x() + 15, client_rect.y() + 120),
                14.0,
                Color::rgb(140, 220, 255),
            )?;
            renderer.draw_text(
                "          [Tab] Cycle Focus |  [Arrows] Move",
                Point::new(client_rect.x() + 15, client_rect.y() + 145),
                14.0,
                Color::rgb(140, 220, 255),
            )?;
            renderer.draw_text(
                "          [m] Maximize      |  [q] Close",
                Point::new(client_rect.x() + 15, client_rect.y() + 170),
                14.0,
                Color::rgb(140, 220, 255),
            )?;
            renderer.draw_text(
                "          [1..4] Workspaces |  [Esc] Exit DE",
                Point::new(client_rect.x() + 15, client_rect.y() + 195),
                14.0,
                Color::rgb(255, 120, 120),
            )?;
        }
    }

    // 3. Render Top Status Bar
    let bar_rect = Rect::new(0, 0, screen_rect.width(), 30);
    renderer.draw_rect(bar_rect, Color::rgb(18, 20, 28))?;
    renderer.draw_rect(
        Rect::new(0, 29, screen_rect.width(), 1),
        Color::rgb(40, 45, 62),
    )?;

    // Top Bar items: Logo, Workspace indicator, hints
    renderer.draw_text(" BabyDraDE ", Point::new(10, 8), 14.0, Color::rgb(120, 190, 255))?;

    let cur_ws = state.workspace_manager.current_workspace_id();
    for ws in 1..=4 {
        let ws_x = 130 + (ws as i32 - 1) * 45;
        let is_cur = ws == cur_ws;
        if is_cur {
            renderer.draw_rect(Rect::new(ws_x - 4, 4, 36, 22), Color::rgb(60, 100, 200))?;
            renderer.draw_text(&format!("[{}]", ws), Point::new(ws_x, 8), 14.0, Color::WHITE)?;
        } else {
            renderer.draw_text(&format!(" {} ", ws), Point::new(ws_x, 8), 14.0, Color::rgb(120, 125, 140))?;
        }
    }

    let hint = "Shortcuts: [Enter] Terminal  [d] Launcher  [1..4] WS  [Esc] Quit";
    renderer.draw_text(
        hint,
        Point::new(screen_rect.width() as i32 - 580, 8),
        14.0,
        Color::rgb(160, 165, 180),
    )?;

    // 4. Flush canvas frame to physical display hardware
    renderer.render_frame()?;

    Ok(())
}
