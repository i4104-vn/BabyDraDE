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

        // Super + Tab -> Cycle focus forward
        if let Ok(combo) = KeyCombo::parse("Super + Tab") {
            self.register_command(combo, Command::CycleFocus(true));
        }

        // Super + Shift + Tab -> Cycle focus backward
        if let Ok(combo) = KeyCombo::parse("Super + Shift + Tab") {
            self.register_command(combo, Command::CycleFocus(false));
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
            Action::TargetFocusedWindowAndWorkspace(f, ws) => focused_window.map(|win| f(win, *ws)),
        }
    }

    /// Parses an action string (such as "launch:alacritty", "window:close", "workspace:switch:2", "focus:cycle").
    pub fn parse_action_str(action_str: &str) -> Option<Action> {
        let action_str = action_str.trim();

        if let Some(app) = action_str.strip_prefix("launch:") {
            return Some(Action::Static(
                SystemCommand::LaunchApp {
                    command_or_desktop_id: app.to_string(),
                }
                .into(),
            ));
        }

        match action_str {
            "window:close" => Some(Action::TargetFocusedWindow(|id| {
                WindowCommand::CloseWindow { id }.into()
            })),
            "window:fullscreen" => Some(Action::TargetFocusedWindow(|id| {
                WindowCommand::ToggleFullscreen { id }.into()
            })),
            "window:maximize" => Some(Action::TargetFocusedWindow(|id| {
                WindowCommand::ToggleMaximize { id }.into()
            })),
            "window:minimize" => Some(Action::TargetFocusedWindow(|id| {
                WindowCommand::MinimizeWindow { id }.into()
            })),
            "window:restore" => Some(Action::TargetFocusedWindow(|id| {
                WindowCommand::RestoreWindow { id }.into()
            })),
            "focus:cycle" => Some(Action::Static(Command::CycleFocus(true))),
            "focus:cycle_backward" => Some(Action::Static(Command::CycleFocus(false))),
            "system:quit" => Some(Action::Static(SystemCommand::QuitCompositor.into())),
            "system:reload" => Some(Action::Static(SystemCommand::ReloadConfig.into())),
            s if s.starts_with("workspace:switch:") => {
                let num: u32 = s.strip_prefix("workspace:switch:")?.parse().ok()?;
                Some(Action::Static(
                    WorkspaceCommand::SwitchWorkspace {
                        workspace_id: WorkspaceId::new(num),
                    }
                    .into(),
                ))
            }
            s if s.starts_with("workspace:move_window:") => {
                let num: u32 = s.strip_prefix("workspace:move_window:")?.parse().ok()?;
                let target = WorkspaceId::new(num);
                Some(Action::TargetFocusedWindowAndWorkspace(
                    |win_id, ws_id| {
                        WorkspaceCommand::MoveWindowToWorkspace {
                            window_id: win_id,
                            workspace_id: ws_id,
                        }
                        .into()
                    },
                    target,
                ))
            }
            _ => None,
        }
    }

    /// Loads keybindings from a string-to-string mapping (e.g. from TOML config).
    pub fn load_from_mapping(&mut self, mapping: &HashMap<String, String>) -> usize {
        let mut count = 0;
        for (combo_str, action_str) in mapping {
            if let Ok(combo) = KeyCombo::parse(combo_str) {
                if let Some(action) = Self::parse_action_str(action_str) {
                    self.register(combo, action);
                    count += 1;
                }
            }
        }
        count
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

        // Super + Tab (Cycle focus)
        let combo_tab = KeyCombo::parse("Super + Tab").unwrap();
        let cmd_tab = dispatcher.dispatch(&combo_tab, focused).unwrap();
        assert_eq!(cmd_tab, Command::CycleFocus(true));
    }

    #[test]
    fn test_load_from_mapping() {
        let mut dispatcher = InputDispatcher::new();
        let mut mapping = HashMap::new();
        mapping.insert("Super + Return".to_string(), "launch:foot".to_string());
        mapping.insert("Super + C".to_string(), "window:close".to_string());
        mapping.insert("Super + 3".to_string(), "workspace:switch:3".to_string());

        let loaded = dispatcher.load_from_mapping(&mapping);
        assert_eq!(loaded, 3);

        let focused = Some(WindowId::new(1));
        let cmd = dispatcher
            .dispatch(&KeyCombo::parse("Super + Return").unwrap(), focused)
            .unwrap();
        assert_eq!(
            cmd,
            SystemCommand::LaunchApp {
                command_or_desktop_id: "foot".into()
            }
            .into()
        );

        let cmd_c = dispatcher
            .dispatch(&KeyCombo::parse("Super + C").unwrap(), focused)
            .unwrap();
        assert_eq!(
            cmd_c,
            WindowCommand::CloseWindow {
                id: WindowId::new(1)
            }
            .into()
        );
    }
}
