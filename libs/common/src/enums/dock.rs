//! Which screen edge a docked surface attaches to.

/// Edge used by screen-edge docks (launcher on the left, switcher on the right).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockSide {
    Left,
    Right,
}
