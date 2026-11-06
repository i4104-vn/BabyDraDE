use application::{
    Command, DesktopEnvironmentEvent, InputDispatcher, SystemCommand, WindowCommand, WindowManager,
    WorkspaceCommand,
};
use domain::{Rect, WindowId};

pub const DEFAULT_WAYLAND_SOCKET: &str = "me.babydra.compositor";

/// Core state of the BabyDraDE compositor.
pub struct BabyDraCompositorState {
    pub socket_name: String,
    pub window_manager: WindowManager,
    pub workspace_manager: application::WorkspaceManager,
    pub input_dispatcher: InputDispatcher,
    pub is_running: bool,
    next_window_id: u64,
}

impl BabyDraCompositorState {
    pub fn new(screen_rect: Rect) -> Self {
        Self {
            socket_name: DEFAULT_WAYLAND_SOCKET.to_string(),
            window_manager: WindowManager::new(screen_rect),
            workspace_manager: application::WorkspaceManager::new(4),
            input_dispatcher: InputDispatcher::new(),
            is_running: true,
            next_window_id: 1,
        }
    }

    pub fn allocate_window_id(&mut self) -> WindowId {
        let id = WindowId::new(self.next_window_id);
        self.next_window_id += 1;
        id
    }

    pub fn handle_command(&mut self, cmd: Command) -> Option<DesktopEnvironmentEvent> {
        match cmd {
            Command::Window(win_cmd) => match win_cmd {
                WindowCommand::OpenWindow {
                    id,
                    title,
                    app_id,
                    initial_size,
                } => {
                    let ev = self
                        .window_manager
                        .open_window(id, title, app_id, initial_size);
                    self.workspace_manager.add_window_to_current(id);
                    Some(ev)
                }
                WindowCommand::CloseWindow { id } => {
                    let (ev, _) = self.window_manager.close_window(id);
                    self.workspace_manager.remove_window(id);
                    ev
                }
                WindowCommand::MoveWindow { id, dx, dy } => {
                    self.window_manager.move_window(id, dx, dy)
                }
                WindowCommand::SetWindowPosition { id, position } => {
                    self.window_manager.set_window_position(id, position)
                }
                WindowCommand::ResizeWindow { id, new_size } => {
                    self.window_manager.resize_window(id, new_size)
                }
                WindowCommand::FocusWindow { id } => self.window_manager.focus_window(id),
                WindowCommand::ToggleMaximize { id } => self.window_manager.toggle_maximize(id),
                WindowCommand::MinimizeWindow { id } => {
                    let (ev, _) = self.window_manager.minimize_window(id);
                    ev
                }
                WindowCommand::RestoreWindow { id } => self.window_manager.restore_window(id),
                WindowCommand::ToggleFullscreen { id } => self.window_manager.toggle_fullscreen(id),
            },
            Command::Workspace(ws_cmd) => match ws_cmd {
                WorkspaceCommand::SwitchWorkspace { workspace_id } => {
                    self.workspace_manager.switch_workspace(workspace_id)
                }
                WorkspaceCommand::MoveWindowToWorkspace {
                    window_id,
                    workspace_id,
                } => {
                    self.workspace_manager
                        .move_window_to_workspace(window_id, workspace_id);
                    None
                }
                WorkspaceCommand::NextWorkspace => self.workspace_manager.next_workspace(),
                WorkspaceCommand::PreviousWorkspace => self.workspace_manager.previous_workspace(),
            },
            Command::System(sys_cmd) => match sys_cmd {
                SystemCommand::QuitCompositor => {
                    self.is_running = false;
                    Some(DesktopEnvironmentEvent::CompositorExiting)
                }
                SystemCommand::LaunchApp {
                    command_or_desktop_id,
                } => {
                    tracing::info!("Launching client app: {}", command_or_desktop_id);
                    None
                }
                SystemCommand::ReloadConfig => {
                    tracing::info!("Reloading configuration");
                    None
                }
            },
            Command::CycleFocus(forward) => {
                if let Some(next_id) = self.window_manager.cycle_focus(forward) {
                    Some(DesktopEnvironmentEvent::WindowFocused { id: next_id })
                } else {
                    None
                }
            }
        }
    }
}
