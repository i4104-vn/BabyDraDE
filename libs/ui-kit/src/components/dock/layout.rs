//! Dock geometry and pointer hit-testing.

use common::constants::launcher::{
    ICON, ICON_SELECTED, PANEL_PAD, PANEL_WIDTH, PANEL_X, SLOT, VISIBLE_SLOTS,
};

use super::DockSide;

/// Logical rectangle of the docked capsule.
#[derive(Clone, Copy, Debug)]
pub struct DockLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub center_y: f64,
}

impl DockLayout {
    pub fn new(count: usize, center_y: f64, screen_w: f64, side: DockSide) -> Self {
        let slots = count.clamp(1, VISIBLE_SLOTS) as f64;
        let height = slots * SLOT + PANEL_PAD * 2.0;
        let x = match side {
            DockSide::Left => PANEL_X,
            DockSide::Right => screen_w - PANEL_WIDTH,
        };
        Self {
            x,
            y: center_y - height / 2.0,
            width: PANEL_WIDTH,
            height,
            center_y,
        }
    }

    pub fn contains(&self, point: (f64, f64), slide_x: f64) -> bool {
        let x = self.x + slide_x;
        point.0 >= x
            && point.0 <= x + self.width
            && point.1 >= self.y
            && point.1 <= self.y + self.height
    }

    /// Centre and size of an icon at layout slot `rel`.
    pub fn icon_geometry(&self, rel: f64, selection_rel: f64) -> (f64, f64, f64) {
        let proximity = (1.0 - selection_rel.abs()).clamp(0.0, 1.0);
        let size = ICON + (ICON_SELECTED - ICON) * proximity * proximity;
        (self.x + self.width / 2.0, self.center_y + rel * SLOT, size)
    }

    /// Slot under a pointer position, or `None` outside the capsule.
    pub fn index_at(
        &self,
        point: (f64, f64),
        slide_x: f64,
        count: usize,
        position: f64,
    ) -> Option<usize> {
        if !self.contains(point, slide_x) {
            return None;
        }
        let visual = visual_center(count, position);
        let index = (visual + (point.1 - self.center_y) / SLOT).round();
        if index < 0.0 || index >= count as f64 {
            return None;
        }
        if (index - visual).abs() > (VISIBLE_SLOTS / 2) as f64 + 0.5 {
            return None;
        }
        Some(index as usize)
    }
}

/// Slot at the capsule centre. Short lists stay still; long lists scroll but
/// clamp at the ends so the first icon starts at the top and the last at the
/// bottom. Selection itself still wraps (`DockState::step_target`).
pub fn visual_center(count: usize, position: f64) -> f64 {
    if count <= VISIBLE_SLOTS {
        (count as f64 - 1.0) / 2.0
    } else {
        let half = (VISIBLE_SLOTS / 2) as f64;
        position.clamp(half, count as f64 - 1.0 - half)
    }
}
