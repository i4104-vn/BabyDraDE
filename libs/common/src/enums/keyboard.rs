//! Global keyboard shortcut actions.

/// High-level compositor actions triggered by keyboard shortcuts.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KeyboardAction {
    /// Gracefully shut down the compositor loop.
    QuitCompositor,
    /// Send a close request to the top/active window.
    CloseActiveWindow,
    /// Spawn a new terminal emulator instance.
    SpawnTerminal,
    /// Show the integrated lock screen.
    LockScreen,
    /// Hot reload compositor and blur configuration from disk.
    ReloadConfig,
    /// Toggle maximize state on the active window.
    ToggleMaximize,
    /// Minimize the active window.
    Minimize,
    /// Toggle fullscreen state on the active window.
    ToggleFullscreen,
    /// Cycle focus between open windows.
    CycleWindowFocus { forward: bool },
    /// Step through the Alt+Tab window switcher overlay.
    AltTabStep { backwards: bool },
    /// Cancel the Alt+Tab switcher without changing focus.
    AltTabCancel,
    /// Commit the Alt+Tab switcher selection and focus chosen window.
    AltTabCommit,
}
