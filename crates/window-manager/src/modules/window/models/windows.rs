//! Typed window actions crossing the compositor boundary.
//!
//! Feature apps send `protocol::WindowAction`; the bridge maps it to this
//! internal enum. Unknown actions are rejected by the protocol codec before
//! they ever reach the compositor.

/// A window-management command requested from outside the compositor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowAction {
    /// Close the selected window.
    Close,
    /// Minimize the selected window.
    Minimize,
    /// Toggle maximize on the selected window.
    Maximize,
    /// Toggle fullscreen on the selected window.
    Fullscreen,
}
