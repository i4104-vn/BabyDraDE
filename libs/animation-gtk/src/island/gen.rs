//! Animation generation counter for stale-callback cancellation.

use std::cell::Cell;

thread_local! {
    static CURRENT_ANIM_GEN: Cell<u64> = const { Cell::new(0) };
}

/// Returns the next generation token used to cancel stale animations.
pub fn next_anim_gen() -> u64 {
    CURRENT_ANIM_GEN.with(|g| {
        let next = g.get() + 1;
        g.set(next);
        next
    })
}

/// Returns the most recently issued animation generation token.
pub fn current_anim_gen() -> u64 {
    CURRENT_ANIM_GEN.with(|g| g.get())
}
