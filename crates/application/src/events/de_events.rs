use domain::{Point, Size, Window, WindowId, WindowState, WorkspaceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DesktopEnvironmentEvent {
    WindowCreated {
        window: Window,
    },
    WindowClosed {
        id: WindowId,
    },
    WindowMoved {
        id: WindowId,
        new_position: Point,
    },
    WindowResized {
        id: WindowId,
        new_size: Size,
    },
    WindowFocused {
        id: WindowId,
    },
    WindowStateChanged {
        id: WindowId,
        state: WindowState,
    },
    WorkspaceSwitched {
        from: WorkspaceId,
        to: WorkspaceId,
    },
    CompositorExiting,
}
