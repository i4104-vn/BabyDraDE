//! Geometry of the launcher capsule and of its slots.

use smithay::utils::{Logical, Point};

use super::{ICON, ICON_SELECTED, PANEL_PAD, PANEL_WIDTH, PANEL_X, SLOT, VISIBLE_SLOTS};

/// Logical rectangle of the capsule plus the vertical centre used to place
/// icons. Built from the app count, so the pill shrinks around short lists.
#[derive(Clone, Copy, Debug)]
pub(super) struct Panel {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) width: f64,
    pub(super) height: f64,
    pub(super) center_y: f64,
}

impl Panel {
    pub(super) fn new(count: usize, center_y: f64) -> Self {
        let slots = count.clamp(1, VISIBLE_SLOTS) as f64;
        let height = slots * SLOT + PANEL_PAD * 2.0;
        Self {
            x: PANEL_X,
            y: center_y - height / 2.0,
            width: PANEL_WIDTH,
            height,
            center_y,
        }
    }

    pub(super) fn contains(&self, point: Point<f64, Logical>) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    /// Centre and size of an icon at layout slot `rel`. `selection_rel` is the
    /// distance from the animated selection and drives the size bump.
    pub(super) fn icon_geometry(&self, rel: f64, selection_rel: f64) -> (f64, f64, f64) {
        let proximity = (1.0 - selection_rel.abs()).clamp(0.0, 1.0);
        let size = ICON + (ICON_SELECTED - ICON) * proximity * proximity;
        (self.x + self.width / 2.0, self.center_y + rel * SLOT, size)
    }

    /// Indices drawn for a list of `count` apps scrolling around `visual`.
    pub(super) fn visible_range(&self, count: usize, visual: f64) -> (usize, usize) {
        if count <= VISIBLE_SLOTS {
            (0, count)
        } else {
            let half = (VISIBLE_SLOTS / 2) as f64;
            (
                (visual - half - 1.0).floor().max(0.0) as usize,
                ((visual + half + 1.0).ceil() as usize).min(count),
            )
        }
    }
}

/// Slot at the capsule centre. Short lists stay put and only the highlight
/// moves; long lists scroll but stop before leaving empty rounded ends.
pub(super) fn visual_center(count: usize, position: f64) -> f64 {
    let count = count as f64;
    if count <= VISIBLE_SLOTS as f64 {
        (count - 1.0) / 2.0
    } else {
        let half = (VISIBLE_SLOTS / 2) as f64;
        position.clamp(half, count - 1.0 - half)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_lists_fit_the_panel_and_keep_icons_still() {
        let panel = Panel::new(5, 400.0);
        assert_eq!(panel.height, 5.0 * SLOT + PANEL_PAD * 2.0);
        assert_eq!(visual_center(5, 0.0), 2.0);
        assert_eq!(visual_center(5, 4.0), 2.0);
        assert_eq!(panel.visible_range(5, 2.0), (0, 5));
    }

    #[test]
    fn long_lists_clamp_the_visual_center_at_the_ends() {
        assert_eq!(visual_center(12, 0.0), 4.0);
        assert_eq!(visual_center(12, 5.0), 5.0);
        assert_eq!(visual_center(12, 11.0), 7.0);

        let panel = Panel::new(12, 400.0);
        assert_eq!(panel.visible_range(12, 4.0), (0, 9));
        assert_eq!(panel.visible_range(12, 11.0), (6, 12));
    }

    #[test]
    fn icon_growth_follows_the_selection_not_the_layout_slot() {
        let panel = Panel::new(12, 400.0);
        // Last slot: selected icon is at the bottom, still at full size.
        let (_, center_y, size) = panel.icon_geometry(4.0, 0.0);
        assert_eq!(center_y, 400.0 + 4.0 * SLOT);
        assert_eq!(size, ICON_SELECTED);

        let (_, _, size) = panel.icon_geometry(-4.0, 1.0);
        assert_eq!(size, ICON);
    }
}
