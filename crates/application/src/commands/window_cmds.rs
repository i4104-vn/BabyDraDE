use domain::{Point, Size, WindowId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowCommand {
    OpenWindow {
        id: WindowId,
        title: String,
        app_id: Option<String>,
        initial_size: Size,
    },
    CloseWindow {
        id: WindowId,
    },
    MoveWindow {
        id: WindowId,
        dx: i32,
        dy: i32,
    },
    SetWindowPosition {
        id: WindowId,
        position: Point,
    },
    ResizeWindow {
        id: WindowId,
        new_size: Size,
    },
    FocusWindow {
        id: WindowId,
    },
    ToggleMaximize {
        id: WindowId,
    },
    MinimizeWindow {
        id: WindowId,
    },
    RestoreWindow {
        id: WindowId,
    },
    ToggleFullscreen {
        id: WindowId,
    },
}
