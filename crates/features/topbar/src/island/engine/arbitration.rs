//! Pure arbitration rules: request purging and winner selection.

use std::time::Instant;

use super::ViewRecord;

/// Purges stale flags and marks live views as requested.
/// Returns the number of live views (used to show the side brackets).
pub(crate) fn refresh_requests(views: &[ViewRecord], now: Instant, hovered: bool) -> usize {
    let mut alive_count = 0usize;
    for view in views {
        let alive = {
            let feature = view.feature.borrow();
            feature.is_alive()
                || feature.is_popover_open()
                || (hovered && feature.hover_keep() && view.state.requested.get())
        };
        view.state.purge(now);
        if alive {
            view.state.requested.set(true);
            alive_count += 1;
        } else if !view.state.has_timeout(now) && !view.state.override_active.get() {
            view.state.deactivate();
        }
    }
    alive_count
}

/// 1. An open popover is never hijacked.
/// 2. Otherwise the most recent explicit override wins.
/// 3. Otherwise priority wins, tie-broken by most recent request.
pub(crate) fn select_winner(views: &[ViewRecord], now: Instant) -> Option<usize> {
    if let Some((index, _)) = views
        .iter()
        .enumerate()
        .find(|(_, view)| view.feature.borrow().is_popover_open())
    {
        return Some(index);
    }

    let mut best_override: Option<(u64, usize)> = None;
    for (index, view) in views.iter().enumerate() {
        if view.state.override_active.get() && view.state.purge(now) {
            let sequence = view.state.request_seq.get();
            if best_override.is_none_or(|(best, _)| sequence > best) {
                best_override = Some((sequence, index));
            }
        }
    }
    if let Some((_, index)) = best_override {
        return Some(index);
    }

    let mut best: Option<(u32, u64, usize)> = None;
    for (index, view) in views.iter().enumerate() {
        if !view.state.purge(now) || !view.state.requested.get() {
            continue;
        }
        let key = (
            view.feature.borrow().priority(),
            view.state.request_seq.get(),
            index,
        );
        if best.is_none_or(|value| key > value) {
            best = Some(key);
        }
    }
    best.map(|(_, _, index)| index)
}
