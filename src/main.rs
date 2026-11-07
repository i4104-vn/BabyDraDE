//! BabyDraDE — Wayland Desktop Environment
//! Root entry point

use application::{WindowCommand, WorkspaceCommand};
use domain::{
    AppRegistry, CanvasRenderer, Color, Cursor, DesktopEntry, Point, Rect, Size, WindowId,
    WorkspaceId,
};
use infrastructure::{BabyDraCompositorState, SoftwareRenderer, WaylandServer};
use presentation::{
    draw_cursor_pointer, CanvasBackground, CompositorRenderPipeline, DesktopCanvas, TitlebarButton,
    TitlebarHit,
};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Clone)]
enum AppInputEvent {
    Key(u8),
    Mouse {
        dx: i32,
        dy: i32,
        left: bool,
        right: bool,
        middle: bool,
    },
}

struct DragState {
    window_id: WindowId,
    offset_x: i32,
    offset_y: i32,
}

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

    // Initialize interactive desktop mouse cursor at center of screen
    let mut cursor = Cursor::new(Point::new(
        renderer.width as i32 / 2,
        renderer.height as i32 / 2,
    ));
    cursor.visible = true;

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
            &cursor,
        )?;
        info!("Compositor initialized successfully.");
        return Ok(());
    }

    info!(
        "BabyDraDE running. Workspaces=4, Socket='{}'. Mouse and keyboard interactive.",
        socket
    );

    // 7. Non-blocking input receiver threads (Keyboard stdin + Hardware Mouse /dev/input/mice)
    let (tx, rx) = mpsc::channel();

    // Stdin keyboard reader thread
    let tx_kbd = tx.clone();
    thread::spawn(move || {
        use std::io::Read;
        let mut stdin = std::io::stdin();
        let mut buf = [0u8; 16];
        while let Ok(n) = stdin.read(&mut buf) {
            if n == 0 {
                break;
            }
            for &b in &buf[..n] {
                if tx_kbd.send(AppInputEvent::Key(b)).is_err() {
                    return;
                }
            }
        }
    });

    // Hardware Mouse reader thread (/dev/input/mice)
    let tx_mouse = tx;
    thread::spawn(move || {
        use std::fs::File;
        use std::io::Read;

        let mouse_paths = ["/dev/input/mice", "/dev/input/mouse0"];
        let mut mouse_file = None;
        for path in mouse_paths {
            if let Ok(file) = File::open(path) {
                info!("Hardware mouse input connected: {}", path);
                mouse_file = Some(file);
                break;
            }
        }

        if let Some(mut file) = mouse_file {
            let mut packet = [0u8; 3];
            while file.read_exact(&mut packet).is_ok() {
                let btn = packet[0];
                let raw_dx = packet[1] as i8;
                let raw_dy = packet[2] as i8;

                let left = (btn & 0x01) != 0;
                let right = (btn & 0x02) != 0;
                let middle = (btn & 0x04) != 0;

                // PS/2 Y is inverted (up is positive), screen Y is positive downwards
                let dx = raw_dx as i32;
                let dy = -(raw_dy as i32);

                if tx_mouse
                    .send(AppInputEvent::Mouse {
                        dx,
                        dy,
                        left,
                        right,
                        middle,
                    })
                    .is_err()
                {
                    return;
                }
            }
        } else {
            warn!("Could not open /dev/input/mice. Mouse interaction available via simulated events.");
        }
    });

    let mut running = true;
    let mut needs_redraw = true;
    let mut active_drag: Option<DragState> = None;
    let mut prev_left = false;
    let mut prev_right = false;
    let mut last_render = Instant::now();
    let mut last_clock_tick = Instant::now();

    // 8. Main Compositor Event & Render Loop (~60 FPS Dirty Frame Rendering)
    while running && state.is_running {
        // Handle input events from channel
        while let Ok(event) = rx.try_recv() {
            match event {
                AppInputEvent::Key(byte) => {
                    match byte {
                        // 'q' or 'Q' -> Close focused window
                        b'q' | b'Q' => {
                            if let Some(focused) = state.window_manager.focused_window_id() {
                                state.handle_command(WindowCommand::CloseWindow { id: focused }.into());
                                needs_redraw = true;
                            } else {
                                running = false;
                            }
                        }
                        // 'd' | 'D' -> Toggle App Launcher
                        b'd' | b'D' => {
                            launcher.toggle(&app_scanner);
                            needs_redraw = true;
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
                            needs_redraw = true;
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
                                needs_redraw = true;
                            }
                        }
                        // 'm' | 'M' -> Toggle maximize
                        b'm' | b'M' => {
                            if let Some(focused) = state.window_manager.focused_window_id() {
                                state.handle_command(WindowCommand::ToggleMaximize { id: focused }.into());
                                needs_redraw = true;
                            }
                        }
                        // 'h' | 'H' -> Minimize
                        b'h' | b'H' => {
                            if let Some(focused) = state.window_manager.focused_window_id() {
                                state.handle_command(WindowCommand::MinimizeWindow { id: focused }.into());
                                needs_redraw = true;
                            }
                        }
                        // '1'..='4' -> Switch workspaces
                        b'1'..=b'4' => {
                            let ws_id = (byte - b'0') as u32;
                            state.handle_command(
                                WorkspaceCommand::SwitchWorkspace {
                                    workspace_id: WorkspaceId::new(ws_id),
                                }
                                .into(),
                            );
                            needs_redraw = true;
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
                                needs_redraw = true;
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
                                needs_redraw = true;
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
                                needs_redraw = true;
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
                                needs_redraw = true;
                            }
                        }
                        // Esc -> Close launcher or exit compositor
                        0x1b => {
                            if launcher.visible {
                                launcher.close();
                                needs_redraw = true;
                            } else {
                                running = false;
                            }
                        }
                        _ => {}
                    }
                }
                AppInputEvent::Mouse {
                    dx,
                    dy,
                    left,
                    right,
                    middle: _,
                } => {
                    // 1. Move cursor position and clamp to screen bounds
                    let new_x = (cursor.position.x + dx).clamp(0, renderer.width as i32 - 1);
                    let new_y = (cursor.position.y + dy).clamp(0, renderer.height as i32 - 1);
                    let new_point = Point::new(new_x, new_y);
                    if new_point != cursor.position {
                        cursor.set_position(new_point);
                        needs_redraw = true;
                    }

                    // 2. Window dragging in progress
                    if let Some(ref drag) = active_drag {
                        if left {
                            let target_x = new_x - drag.offset_x;
                            let target_y = (new_y - drag.offset_y).max(30);
                            state.handle_command(
                                WindowCommand::SetWindowPosition {
                                    id: drag.window_id,
                                    position: Point::new(target_x, target_y),
                                }
                                .into(),
                            );
                            needs_redraw = true;
                        } else {
                            active_drag = None;
                        }
                    }

                    // 3. Mouse Left Click down
                    if left && !prev_left {
                        needs_redraw = true;

                        if launcher.visible {
                            // Check click in app launcher
                            let popup_w = 460i32;
                            let popup_h = 320i32;
                            let popup_x = (screen_rect.width() as i32 - popup_w) / 2;
                            let popup_y = (screen_rect.height() as i32 - popup_h) / 2;
                            let popup_rect = Rect::new(popup_x, popup_y, popup_w as u32, popup_h as u32);

                            if popup_rect.contains_point(new_point) {
                                let item_y = new_point.y - (popup_y + 55);
                                if item_y >= 0 {
                                    let clicked_idx = (item_y / 36) as usize;
                                    if let Some(entry) = launcher.matched_entries.get(clicked_idx) {
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
                                        launcher.close();
                                    }
                                }
                            } else {
                                launcher.close();
                            }
                        } else if new_point.y < 30 {
                            // Top Bar clicked
                            if new_point.x <= 110 {
                                // Click BabyDraDE logo -> Toggle Launcher
                                launcher.toggle(&app_scanner);
                            } else {
                                // Workspaces 1..4 buttons
                                for ws in 1..=4 {
                                    let ws_x = 130 + (ws as i32 - 1) * 45;
                                    let ws_rect = Rect::new(ws_x - 6, 2, 40, 26);
                                    if ws_rect.contains_point(new_point) {
                                        state.handle_command(
                                            WorkspaceCommand::SwitchWorkspace {
                                                workspace_id: WorkspaceId::new(ws as u32),
                                            }
                                            .into(),
                                        );
                                        break;
                                    }
                                }
                            }
                        } else {
                            // Windows hit test in reverse z-order (topmost first)
                            let cur_ws_wins = state
                                .workspace_manager
                                .current_workspace()
                                .windows()
                                .to_vec();
                            let mut hit_info = None;

                            for win in state.window_manager.windows().collect::<Vec<_>>().into_iter().rev() {
                                if !win.visible || win.is_minimized() || !cur_ws_wins.contains(&win.id) {
                                    continue;
                                }

                                let titlebar_h = pipeline.titlebar_decorator.theme.height;
                                let full_win_rect = Rect::new(
                                    win.geometry.x(),
                                    win.geometry.y(),
                                    win.geometry.width(),
                                    win.geometry.height() + titlebar_h,
                                );

                                if full_win_rect.contains_point(new_point) {
                                    hit_info = Some((win.id, win.geometry));
                                    break;
                                }
                            }

                            if let Some((win_id, win_geo)) = hit_info {
                                state.handle_command(WindowCommand::FocusWindow { id: win_id }.into());

                                let hit = pipeline.titlebar_decorator.hit_test(&win_geo, new_point);
                                match hit {
                                    TitlebarHit::Button(TitlebarButton::Close) => {
                                        state.handle_command(WindowCommand::CloseWindow { id: win_id }.into());
                                    }
                                    TitlebarHit::Button(TitlebarButton::Maximize) => {
                                        state.handle_command(WindowCommand::ToggleMaximize { id: win_id }.into());
                                    }
                                    TitlebarHit::Button(TitlebarButton::Minimize) => {
                                        state.handle_command(WindowCommand::MinimizeWindow { id: win_id }.into());
                                    }
                                    TitlebarHit::DragTitlebar => {
                                        active_drag = Some(DragState {
                                            window_id: win_id,
                                            offset_x: new_point.x - win_geo.x(),
                                            offset_y: new_point.y - win_geo.y(),
                                        });
                                    }
                                    TitlebarHit::None => {}
                                }
                            }
                        }
                    }

                    // 4. Mouse Right Click -> Toggle Launcher
                    if right && !prev_right {
                        launcher.toggle(&app_scanner);
                        needs_redraw = true;
                    }

                    prev_left = left;
                    prev_right = right;
                }
            }
        }

        // Periodic clock / idle pulse every 1 second
        if last_clock_tick.elapsed() >= Duration::from_millis(1000) {
            needs_redraw = true;
            last_clock_tick = Instant::now();
        }

        // Frame rate limiter (~60 FPS) and Dirty-Frame redrawing (completely eliminates idle flicker)
        if needs_redraw && last_render.elapsed() >= Duration::from_millis(16) {
            render_composite_frame(
                &mut renderer,
                &pipeline,
                &desktop_canvas,
                &state,
                &launcher,
                &socket,
                &cursor,
            )?;
            needs_redraw = false;
            last_render = Instant::now();
        }

        thread::sleep(Duration::from_millis(5));
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
    cursor: &Cursor,
) -> Result<(), domain::RenderError> {
    let screen_rect = canvas.bounds;
    let active_ws_windows = state.workspace_manager.current_workspace().windows();

    // 1. Render background and window decorations via Clean Architecture pipeline (into memory buffer only)
    pipeline.render_layers(
        renderer,
        &state.window_manager,
        Some(active_ws_windows),
        None,
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
                    state.workspace_manager.active_workspace_id().as_u32()
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
                &format!("Mouse: ({}, {})", cursor.position.x, cursor.position.y),
                Point::new(client_rect.x() + 15, client_rect.y() + 95),
                14.0,
                Color::rgb(100, 255, 180),
            )?;
            renderer.draw_text(
                "----------------------------------------------",
                Point::new(client_rect.x() + 15, client_rect.y() + 115),
                14.0,
                Color::rgb(60, 65, 85),
            )?;
            renderer.draw_text(
                "Controls: Mouse drag titlebar to move windows",
                Point::new(client_rect.x() + 15, client_rect.y() + 135),
                14.0,
                Color::rgb(140, 220, 255),
            )?;
            renderer.draw_text(
                "          Click buttons [_] [□] [×] to control",
                Point::new(client_rect.x() + 15, client_rect.y() + 155),
                14.0,
                Color::rgb(140, 220, 255),
            )?;
            renderer.draw_text(
                "          [Enter] Terminal  |  [d] Launcher",
                Point::new(client_rect.x() + 15, client_rect.y() + 175),
                14.0,
                Color::rgb(140, 220, 255),
            )?;
            renderer.draw_text(
                "          [Tab] Cycle Focus |  [1..4] Workspaces",
                Point::new(client_rect.x() + 15, client_rect.y() + 195),
                14.0,
                Color::rgb(140, 220, 255),
            )?;
            renderer.draw_text(
                "          [Esc] Exit DE to TTY",
                Point::new(client_rect.x() + 15, client_rect.y() + 215),
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

    // Top Bar items: Logo (clickable launcher button)
    renderer.draw_rect(Rect::new(4, 3, 105, 24), Color::rgb(35, 40, 58))?;
    renderer.draw_text(" BabyDraDE ", Point::new(10, 8), 14.0, Color::rgb(120, 190, 255))?;

    let cur_ws = state.workspace_manager.active_workspace_id().as_u32();
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

    let hint = "Mouse Drag: Move  |  Buttons: [_][□][×]  |  [Enter] Term  |  [Esc] Quit";
    renderer.draw_text(
        hint,
        Point::new(screen_rect.width() as i32 - 620, 8),
        14.0,
        Color::rgb(160, 165, 180),
    )?;

    // 4. Render Launcher Overlay if visible
    if launcher.visible {
        launcher.render(renderer, screen_rect)?;
    }

    // 5. Render Mouse Cursor Pointer (Topmost layer!)
    if cursor.visible {
        draw_cursor_pointer(renderer, cursor.position.x, cursor.position.y)?;
    }

    // 6. Flush canvas frame to physical display hardware ONCE
    renderer.render_frame()?;

    Ok(())
}
