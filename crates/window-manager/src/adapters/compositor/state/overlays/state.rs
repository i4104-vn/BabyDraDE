//! State owned by the compositor overlays.

use crate::adapters::compositor::state::launcher::LauncherState;
use crate::adapters::desktop::switcher::state::AltTabSwitcher;

/// The launcher and the Alt+Tab switcher share overlay semantics (they are
/// mutually exclusive and consume the same touch gestures), so their state is
/// owned together.
pub(crate) struct OverlayState {
    pub(crate) alt_tab_switcher: Option<AltTabSwitcher>,
    pub(crate) launcher: LauncherState,
}

impl OverlayState {
    pub(crate) fn new() -> Self {
        Self {
            alt_tab_switcher: None,
            launcher: LauncherState::new(),
        }
    }
}
