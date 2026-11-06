use std::collections::HashMap;

use domain::{KeyCombo, WindowId, WorkspaceId};

use crate::commands::{Command, SystemCommand, WindowCommand, WorkspaceCommand};

#[derive(Debug, Clone)]
pub enum Action {
    Static(Command),
    TargetFocusedWindow(fn(WindowId) -> Command),
    TargetFocusedWindowAndWorkspace(fn(WindowId, WorkspaceId) -> Command, WorkspaceId),
}

/// Dispatches raw key events to high-level Desktop Environment commands.
pub struct InputDispatcher {
    bindings: HashMap<KeyCombo, Action>,
}

impl InputDispatcher {
    pub fn new() -> Self {
        let mut dispatcher = Self {
            bindings: HashMap::new(),
        };
        dispatcher.register_defaults();
        dispatcher
    }

    pub fn register(&mut self, combo: KeyCombo, action: Action) {
        self.bindings.insert(combo, action);
    }

    pub fn register_command(&mut self, combo: KeyCombo, command: impl Into<Command>) {
        self.register(combo, Action::Static(command.into()));
    }

    /// Registers the default BabyDraDE keybindings specified in planning.md.
    pub fn register_defaults(&mut self) {
        // Super + Return -> Terminal
        if let Ok(combo) = KeyCombo::parse("Super + Return") {
            self.register_command(
                combo,
                SystemCommand::LaunchApp {
                    command_or_desktop_id: "alacritty".into(),
                },
            );
        }

        // Super + Q -> Close focused window
        if let Ok(combo) = KeyCombo::parse("Super + Q") {
            self.register(
                combo,
                Action::TargetFocusedWindow(|id| WindowCommand::CloseWindow { id }.into()),
            );
        }

        // Super + F -> Toggle fullscreen
        if let Ok(combo) = KeyCombo::parse("Super + F") {
            self.register(
                combo,
                Action::TargetFocusedWindow(|id| WindowCommand::ToggleFullscreen { id }.into()),
            );
        }

        // Super + M -> Toggle maximize
        if let Ok(combo) = KeyCombo::parse("Super + M") {
            self.register(
                combo,
                Action::TargetFocusedWindow(|id| WindowCommand::ToggleMaximize { id }.into()),
            );
        }

        // Super + H -> Minimize
        if let Ok(combo) = KeyCombo::parse("Super + H") {
            self.register(
                combo,
                Action::TargetFocusedWindow(|id| WindowCommand::MinimizeWindow { id }.into()),
            );
        }

        // Super + Shift + Q -> Quit compositor
        if let Ok(combo) = KeyCombo::parse("Super + Shift + Q") {
            self.register_command(combo, SystemCommand::QuitCompositor);
        }

        // Super + D -> App Launcher
        if let Ok(combo) = KeyCombo::parse("Super + D") {
            self.register_command(
                combo,
                SystemCommand::LaunchApp {
                    command_or_desktop_id: "babydra-launcher".into(),
                },
            );
        }

        // Super + 1..9 -> Switch Workspace
        for i in 1..=9 {
            if let Ok(combo) = KeyCombo::parse(&format!("Super + {i}")) {
                self.register_command(
                    combo,
                    WorkspaceCommand::SwitchWorkspace {
                        workspace_id: WorkspaceId::new(i),
                    },
                );
            }
        }

        // Super + Shift + 1..9 -> Move window to Workspace
        for i in 1..=9 {
            if let Ok(combo) = KeyCombo::parse(&format!("Super + Shift + {i}")) {
                let ws_id = WorkspaceId::new(i);
                self.register(
                    combo,
                    Action::TargetFocusedWindowAndWorkspace(
                        |win_id, target_ws| {
                            WorkspaceCommand::MoveWindowToWorkspace {
                                window_id: win_id,
                                workspace_id: target_ws,
                            }
                            .into()
                        },
                        ws_id,
                    ),
                );
            }
        }
    }

    /// Dispatches key event against registered keybindings.
    pub fn dispatch(&self, combo: &KeyCombo, focused_window: Option<WindowId>) -> Option<Command> {
        let action = self.bindings.get(combo)?;
        match action {
            Action::Static(cmd) => Some(cmd.clone()),
            Action::TargetFocusedWindow(f) => focused_window.map(f),
            Action::TargetFocusedWindowAndWorkspace(f, ws) => {
                focused_window.map(|win| f(win, *ws))
            }
        }
    }
}

impl Default for InputDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_dispatcher_default_bindings() {
        let dispatcher = InputDispatcher::new();
        let focused = Some(WindowId::new(42));

        // Super + Return
        let combo = KeyCombo::parse("Super + Return").unwrap();
        let cmd = dispatcher.dispatch(&combo, focused).unwrap();
        assert_eq!(
            cmd,
            SystemCommand::LaunchApp {
                command_or_desktop_id: "alacritty".into()
            }
            .into()
        );

        // Super + Q
        let combo_q = KeyCombo::parse("Super + Q").unwrap();
        let cmd_q = dispatcher.dispatch(&combo_q, focused).unwrap();
        assert_eq!(
            cmd_q,
            WindowCommand::CloseWindow {
                id: WindowId::new(42)
            }
            .into()
        );

        // Super + 2
        let combo_ws2 = KeyCombo::parse("Super + 2").unwrap();
        let cmd_ws2 = dispatcher.dispatch(&combo_ws2, focused).unwrap();
        assert_eq!(
            cmd_ws2,
            WorkspaceCommand::SwitchWorkspace {
                workspace_id: WorkspaceId::new(2)
            }
            .into()
        );
    }
}
