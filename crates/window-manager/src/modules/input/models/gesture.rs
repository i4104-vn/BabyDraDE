//! Input gesture decision models.

/// A decision the core has made; the adapter decides how to perform it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowIntent {
    /// Swallow the event without further action (already handled live).
    Consumed,
    CycleFocus {
        forward: bool,
    },
    Maximize,
    Minimize,
    Fullscreen,
    Close,
    /// Synthesise a middle click.
    MiddleClick,
}
