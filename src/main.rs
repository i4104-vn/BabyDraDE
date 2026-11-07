//! BabyDraDE — Wayland Desktop Environment
//! Core Window Management & Compositor Bootstrap

use application::{WindowCommand, WorkspaceCommand};
use domain::{Point, Rect, Size, SnapPosition, WindowId, WorkspaceId};
use infrastructure::{BabyDraCompositorState, WaylandServer};
use std::io::{self, BufRead, Write};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

fn main() -> anyhow::Result<()> {
    // 1. Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    println!("\x1b[1;36m==========================================================\x1b[0m");
    println!("\x1b[1;36m       BabyDraDE — Wayland Floating Window Manager        \x1b[0m");
    println!("\x1b[1;36m==========================================================\x1b[0m");

    // 2. Initialize Compositor State & Screen Geometry
    let screen_w = std::env::var("BABYDRADE_WIDTH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1920);
    let screen_h = std::env::var("BABYDRADE_HEIGHT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1080);

    let screen_rect = Rect::new(0, 0, screen_w, screen_h);
    let mut state = BabyDraCompositorState::new(screen_rect);

    // Set work area (leaving 30px for top status bar)
    state
        .window_manager
        .set_work_area(Rect::new(0, 30, screen_w, screen_h.saturating_sub(30)));

    info!(
        "Screen initialized: {}x{}, Work Area: {}x{} at (0, 30)",
        screen_w,
        screen_h,
        state.window_manager.work_area().width(),
        state.window_manager.work_area().height()
    );

    // 3. Initialize Wayland Server & bind compositor socket
    let mut wayland_server = WaylandServer::new("me.babydra.compositor");
    wayland_server.register_core_globals();
    let socket = wayland_server
        .bind_socket()
        .map_err(|e| anyhow::anyhow!(e))?;
    info!("Wayland compositor listening on socket: '{}'", socket);

    // 4. Pre-populate initial demo floating windows
    let win_term = state.allocate_window_id();
    state.handle_command(
        WindowCommand::OpenWindow {
            id: win_term,
            title: "Alacritty — Terminal".into(),
            app_id: Some("alacritty".into()),
            initial_size: Size::new(800, 520),
        }
        .into(),
    );

    let win_browser = state.allocate_window_id();
    state.handle_command(
        WindowCommand::OpenWindow {
            id: win_browser,
            title: "Firefox Web Browser".into(),
            app_id: Some("firefox".into()),
            initial_size: Size::new(960, 640),
        }
        .into(),
    );

    let win_files = state.allocate_window_id();
    state.handle_command(
        WindowCommand::OpenWindow {
            id: win_files,
            title: "File Manager (Thunar)".into(),
            app_id: Some("thunar".into()),
            initial_size: Size::new(700, 480),
        }
        .into(),
    );

    // Focus terminal initially
    state.handle_command(WindowCommand::FocusWindow { id: win_term }.into());

    // Print initial desktop state
    print_desktop_status(&state, &socket);

    // Check if running in verification mode
    if std::env::var("BABYDRADE_VERIFY_ONLY").is_ok()
        || std::env::args().any(|a| a == "--verify-only")
    {
        info!("Verification mode complete: 3 demo windows initialized in active workspace.");
        return Ok(());
    }

    // 5. Interactive Window Management Shell
    println!("\x1b[1;32mInteractive Window Manager Shell Ready.\x1b[0m");
    println!("Type \x1b[1;33mhelp\x1b[0m for available commands, or \x1b[1;33mquit\x1b[0m to exit.\n");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    print!("babydrade> ");
    stdout.flush()?;

    for line_res in stdin.lock().lines() {
        let line = match line_res {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            print!("babydrade> ");
            stdout.flush()?;
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "help" | "?" => {
                println!("\x1b[1;34mAvailable Window Management Commands:\x1b[0m");
                println!("  list | ls                : Display all windows and stacking z-order");
                println!("  open <title> [app_id]    : Open a new window (cascade placement)");
                println!("  close <id>               : Close window (MRU focus fallback)");
                println!("  focus <id>               : Focus window (raises to top of z-order)");
                println!("  raise <id>               : Raise window to top of stack");
                println!("  lower <id>               : Lower window to bottom of stack");
                println!("  move <id> <dx> <dy>      : Move window by delta x, y");
                println!("  pos <id> <x> <y>         : Set window absolute position");
                println!("  resize <id> <w> <h>      : Resize window dimensions");
                println!("  max <id>                 : Toggle window maximize");
                println!("  snap <id> <left|right>   : Snap window to 50% split (or top|bottom)");
                println!("  min <id>                 : Minimize window");
                println!("  restore <id>             : Restore window to normal state");
                println!("  full <id>                : Toggle fullscreen mode");
                println!("  ws <1..4>                : Switch active workspace");
                println!("  send <id> <1..4>         : Move window to workspace");
                println!("  quit | exit | q          : Quit compositor");
            }

            "list" | "ls" | "status" => {
                print_desktop_status(&state, &socket);
            }

            "open" => {
                let title = if parts.len() > 1 {
                    parts[1..].join(" ")
                } else {
                    "Untitled Window".to_string()
                };
                let id = state.allocate_window_id();
                state.handle_command(
                    WindowCommand::OpenWindow {
                        id,
                        title: title.clone(),
                        app_id: None,
                        initial_size: Size::new(720, 480),
                    }
                    .into(),
                );
                println!(
                    "\x1b[32m[+] Window #{} ('{}') opened and placed on workspace {}.\x1b[0m",
                    id,
                    title,
                    state.workspace_manager.active_workspace_id().as_u32()
                );
            }

            "close" => {
                if let Some(id) = parse_id(&parts, 1, &state) {
                    state.handle_command(WindowCommand::CloseWindow { id }.into());
                    println!("\x1b[33m[-] Window #{} closed.\x1b[0m", id);
                } else {
                    println!("\x1b[31m[!] Usage: close <window_id>\x1b[0m");
                }
            }

            "focus" => {
                if let Some(id) = parse_id(&parts, 1, &state) {
                    state.handle_command(WindowCommand::FocusWindow { id }.into());
                    println!("\x1b[32m[*] Window #{} focused (raised to top).\x1b[0m", id);
                } else {
                    println!("\x1b[31m[!] Usage: focus <window_id>\x1b[0m");
                }
            }

            "raise" => {
                if let Some(id) = parse_id(&parts, 1, &state) {
                    state.handle_command(WindowCommand::RaiseWindow { id }.into());
                    println!("\x1b[32m[^] Window #{} raised to top of z-order stack.\x1b[0m", id);
                } else {
                    println!("\x1b[31m[!] Usage: raise <window_id>\x1b[0m");
                }
            }

            "lower" => {
                if let Some(id) = parse_id(&parts, 1, &state) {
                    state.handle_command(WindowCommand::LowerWindow { id }.into());
                    println!("\x1b[33m[v] Window #{} lowered to bottom of z-order stack.\x1b[0m", id);
                } else {
                    println!("\x1b[31m[!] Usage: lower <window_id>\x1b[0m");
                }
            }

            "move" => {
                if parts.len() >= 4 {
                    if let (Some(id), Ok(dx), Ok(dy)) = (
                        parse_id(&parts, 1, &state),
                        parts[2].parse::<i32>(),
                        parts[3].parse::<i32>(),
                    ) {
                        state.handle_command(WindowCommand::MoveWindow { id, dx, dy }.into());
                        if let Some(win) = state.window_manager.get_window(id) {
                            println!(
                                "\x1b[32m[*] Window #{} moved to ({}, {}).\x1b[0m",
                                id, win.geometry.origin.x, win.geometry.origin.y
                            );
                        }
                    }
                } else {
                    println!("\x1b[31m[!] Usage: move <window_id> <dx> <dy>\x1b[0m");
                }
            }

            "pos" => {
                if parts.len() >= 4 {
                    if let (Some(id), Ok(x), Ok(y)) = (
                        parse_id(&parts, 1, &state),
                        parts[2].parse::<i32>(),
                        parts[3].parse::<i32>(),
                    ) {
                        state.handle_command(
                            WindowCommand::SetWindowPosition {
                                id,
                                position: Point::new(x, y),
                            }
                            .into(),
                        );
                        println!("\x1b[32m[*] Window #{} positioned at ({}, {}).\x1b[0m", id, x, y);
                    }
                } else {
                    println!("\x1b[31m[!] Usage: pos <window_id> <x> <y>\x1b[0m");
                }
            }

            "resize" => {
                if parts.len() >= 4 {
                    if let (Some(id), Ok(w), Ok(h)) = (
                        parse_id(&parts, 1, &state),
                        parts[2].parse::<u32>(),
                        parts[3].parse::<u32>(),
                    ) {
                        state.handle_command(
                            WindowCommand::ResizeWindow {
                                id,
                                new_size: Size::new(w, h),
                            }
                            .into(),
                        );
                        println!("\x1b[32m[*] Window #{} resized to {}x{}.\x1b[0m", id, w, h);
                    }
                } else {
                    println!("\x1b[31m[!] Usage: resize <window_id> <width> <height>\x1b[0m");
                }
            }

            "max" => {
                if let Some(id) = parse_id(&parts, 1, &state) {
                    state.handle_command(WindowCommand::ToggleMaximize { id }.into());
                    if let Some(win) = state.window_manager.get_window(id) {
                        println!(
                            "\x1b[32m[*] Window #{} maximize toggled. State: {:?}, Geometry: {:?}\x1b[0m",
                            id, win.state, win.geometry
                        );
                    }
                } else {
                    println!("\x1b[31m[!] Usage: max <window_id>\x1b[0m");
                }
            }

            "snap" => {
                if parts.len() >= 3 {
                    if let Some(id) = parse_id(&parts, 1, &state) {
                        let pos = match parts[2].to_lowercase().as_str() {
                            "left" | "l" => Some(SnapPosition::LeftHalf),
                            "right" | "r" => Some(SnapPosition::RightHalf),
                            "top" | "t" => Some(SnapPosition::TopHalf),
                            "bottom" | "b" => Some(SnapPosition::BottomHalf),
                            _ => None,
                        };
                        if let Some(p) = pos {
                            state.handle_command(WindowCommand::SnapWindow { id, position: p }.into());
                            if let Some(win) = state.window_manager.get_window(id) {
                                println!(
                                    "\x1b[32m[*] Window #{} snapped to {:?}: {:?}\x1b[0m",
                                    id, p, win.geometry
                                );
                            }
                        } else {
                            println!("\x1b[31m[!] Unknown snap position. Use: left, right, top, bottom\x1b[0m");
                        }
                    }
                } else {
                    println!("\x1b[31m[!] Usage: snap <window_id> <left|right|top|bottom>\x1b[0m");
                }
            }

            "min" => {
                if let Some(id) = parse_id(&parts, 1, &state) {
                    state.handle_command(WindowCommand::MinimizeWindow { id }.into());
                    println!("\x1b[33m[*] Window #{} minimized.\x1b[0m", id);
                } else {
                    println!("\x1b[31m[!] Usage: min <window_id>\x1b[0m");
                }
            }

            "restore" => {
                if let Some(id) = parse_id(&parts, 1, &state) {
                    state.handle_command(WindowCommand::RestoreWindow { id }.into());
                    println!("\x1b[32m[*] Window #{} restored.\x1b[0m", id);
                } else {
                    println!("\x1b[31m[!] Usage: restore <window_id>\x1b[0m");
                }
            }

            "full" => {
                if let Some(id) = parse_id(&parts, 1, &state) {
                    state.handle_command(WindowCommand::ToggleFullscreen { id }.into());
                    println!("\x1b[32m[*] Window #{} fullscreen toggled.\x1b[0m", id);
                } else {
                    println!("\x1b[31m[!] Usage: full <window_id>\x1b[0m");
                }
            }

            "ws" => {
                if parts.len() >= 2 {
                    if let Ok(num) = parts[1].parse::<u32>() {
                        if num >= 1 && num <= 4 {
                            state.handle_command(
                                WorkspaceCommand::SwitchWorkspace {
                                    workspace_id: WorkspaceId::new(num),
                                }
                                .into(),
                            );
                            println!("\x1b[32m[*] Switched to Workspace {}.\x1b[0m", num);
                        } else {
                            println!("\x1b[31m[!] Workspaces are 1..4.\x1b[0m");
                        }
                    }
                } else {
                    println!("\x1b[31m[!] Usage: ws <1..4>\x1b[0m");
                }
            }

            "send" => {
                if parts.len() >= 3 {
                    if let (Some(id), Ok(ws)) = (parse_id(&parts, 1, &state), parts[2].parse::<u32>()) {
                        state.handle_command(
                            WorkspaceCommand::MoveWindowToWorkspace {
                                window_id: id,
                                workspace_id: WorkspaceId::new(ws),
                            }
                            .into(),
                        );
                        println!("\x1b[32m[*] Moved Window #{} to Workspace {}.\x1b[0m", id, ws);
                    }
                } else {
                    println!("\x1b[31m[!] Usage: send <window_id> <workspace_id>\x1b[0m");
                }
            }

            "quit" | "exit" | "q" => {
                println!("Exiting BabyDraDE...");
                break;
            }

            unknown => {
                println!(
                    "\x1b[31mUnknown command: '{}'. Type 'help' for commands.\x1b[0m",
                    unknown
                );
            }
        }

        print!("babydrade> ");
        stdout.flush()?;
    }

    info!("BabyDraDE shut down cleanly.");
    Ok(())
}

fn parse_id(parts: &[&str], idx: usize, state: &BabyDraCompositorState) -> Option<WindowId> {
    if parts.len() > idx {
        if let Ok(num) = parts[idx].parse::<u64>() {
            return Some(WindowId::new(num));
        }
    }
    // Fallback: return currently focused window ID
    state.window_manager.focused_window_id()
}

/// Formats and prints the complete Window Manager state & z-order tree to stdout
fn print_desktop_status(state: &BabyDraCompositorState, socket: &str) {
    let cur_ws = state.workspace_manager.active_workspace_id().as_u32();
    let cur_ws_wins = state.workspace_manager.current_workspace().windows();
    let focused_id = state.window_manager.focused_window_id();

    println!("\x1b[1;36m┌────────────────────────────────────────────────────────────────────────┐\x1b[0m");
    println!(
        "\x1b[1;36m│\x1b[0m \x1b[1mBabyDraDE Window Manager\x1b[0m | WAYLAND_DISPLAY=\x1b[32m{}\x1b[0m",
        socket
    );
    println!(
        "\x1b[1;36m│\x1b[0m Work Area: \x1b[33m{}x{}\x1b[0m (Total Display: {}x{})",
        state.window_manager.work_area().width(),
        state.window_manager.work_area().height(),
        state.window_manager.screen_rect().width(),
        state.window_manager.screen_rect().height()
    );
    println!("\x1b[1;36m├────────────────────────────────────────────────────────────────────────┤\x1b[0m");

    let mut ws_line = String::from("  Workspaces: ");
    for ws in 1..=4 {
        if ws == cur_ws {
            ws_line.push_str(&format!(" \x1b[1;44;37m[{}]\x1b[0m ", ws));
        } else {
            ws_line.push_str(&format!(" [{}] ", ws));
        }
    }
    println!("{}", ws_line);
    println!("\x1b[1;36m├────────────────────────────────────────────────────────────────────────┤\x1b[0m");
    println!("\x1b[1m  Window Stacking Order (Z-Order, Bottom -> Top):\x1b[0m");

    let stacked: Vec<_> = state
        .window_manager
        .windows_stacked()
        .filter(|w| cur_ws_wins.contains(&w.id))
        .collect();

    if stacked.is_empty() {
        println!("    (No windows open in this workspace)");
    } else {
        for (idx, win) in stacked.iter().enumerate() {
            let is_focused = focused_id == Some(win.id);
            let focus_badge = if is_focused {
                "\x1b[1;32m[FOCUSED]\x1b[0m"
            } else {
                "\x1b[90m[UNFOCUSED]\x1b[0m"
            };

            let state_badge = match win.state {
                domain::WindowState::Normal => "\x1b[36mNORMAL\x1b[0m",
                domain::WindowState::Maximized { .. } => "\x1b[33mMAXIMIZED\x1b[0m",
                domain::WindowState::Minimized => "\x1b[31mMINIMIZED\x1b[0m",
                domain::WindowState::Fullscreen { .. } => "\x1b[35mFULLSCREEN\x1b[0m",
                domain::WindowState::Snapped { position, .. } => match position {
                    SnapPosition::LeftHalf => "\x1b[34mSNAP-LEFT\x1b[0m",
                    SnapPosition::RightHalf => "\x1b[34mSNAP-RIGHT\x1b[0m",
                    SnapPosition::TopHalf => "\x1b[34mSNAP-TOP\x1b[0m",
                    SnapPosition::BottomHalf => "\x1b[34mSNAP-BOTTOM\x1b[0m",
                },
            };

            println!(
                "   \x1b[1;33m{}.\x1b[0m [ID \x1b[1m{}\x1b[0m] \"{}\" {} {} at ({}, {}) {}x{}",
                idx + 1,
                win.id.as_u64(),
                win.title,
                focus_badge,
                state_badge,
                win.geometry.origin.x,
                win.geometry.origin.y,
                win.geometry.size.width,
                win.geometry.size.height,
            );
        }
    }
    println!("\x1b[1;36m└────────────────────────────────────────────────────────────────────────┘\x1b[0m\n");
}
