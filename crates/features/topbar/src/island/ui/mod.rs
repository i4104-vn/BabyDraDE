//! Island UI: capsule assembly, shared notch, popovers and transitions.

pub mod capsule;
pub mod popover;

mod motion;

// Compatibility re-exports: the notch and transition modules moved under
// `motion`, but their original paths (`island::ui::notch`, `::transition`)
// remain public.
pub use motion::{notch, transition};

pub use motion::notch::NotchWidget;
pub use popover::IslandPopover;
