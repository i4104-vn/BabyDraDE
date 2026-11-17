//! Window and titlebar interaction actions.

/// Actions triggered by titlebar buttons or gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Action {
    Close,
    Minimize,
    Maximize,
    Move,
}
