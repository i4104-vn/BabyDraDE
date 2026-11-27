//! Dynamic Island composition root.
//!
//! ```text
//! island/
//! ├── engine/       # trait, context, handle, state, arbitration
//! ├── services/     # IPC and shared poller
//! ├── ui/           # capsule, notch, popover, transition
//! ├── features/     # one module per island feature
//! └── registry.rs   # the single feature registration point
//! ```

pub mod engine;
pub mod features;
pub mod registry;
pub mod services;
pub mod ui;

// Stable paths for shared services and widgets used by features and the shell.
pub use services::{ipc, poller};
pub use ui::capsule::IslandCapsule as DynamicIsland;
pub use ui::{IslandPopover, NotchWidget};

pub use engine::{
    IslandCtx, IslandFeature, IslandViewHandle, CAPSULE_HEIGHT, CAPSULE_WIDTH, PLAYER_CAPSULE_WIDTH,
};
