//! Dock selection, presence and hold-to-repeat state.

use animation::{Easing, Tween, FADE_IN_DURATION, FADE_OUT_DURATION, SCROLL_DURATION};
use common::constants::{launcher::PANEL_WIDTH, KEY_REPEAT_DELAY_MS, KEY_REPEAT_INTERVAL_MS};
use std::time::{Duration, Instant};

use super::DockSide;

/// Next dock position, wrapping straight to the first/last entry at a boundary.
fn step_target(current: f64, delta: f64, count: usize) -> (f64, bool) {
    let raw = current + delta;
    let n = count as f64;
    if raw >= n {
        (0.0, true)
    } else if raw < 0.0 {
        (n - 1.0, true)
    } else {
        (raw, false)
    }
}

/// Selection, presence and hold-to-repeat state shared by both docks.
pub struct DockState {
    pub position: Tween,
    pub presence: Tween,
    pub selected: Option<usize>,
    pub closing: bool,
    held_direction: Option<i32>,
    repeat_started: Option<Instant>,
    last_repeat: Option<Instant>,
}

impl Default for DockState {
    fn default() -> Self {
        Self::new()
    }
}

impl DockState {
    pub fn new() -> Self {
        Self {
            position: Tween::settled(0.0),
            presence: Tween::settled(0.0),
            selected: None,
            closing: false,
            held_direction: None,
            repeat_started: None,
            last_repeat: None,
        }
    }

    pub fn position(&self) -> f64 {
        self.position.value()
    }

    pub fn presence(&self) -> f64 {
        self.presence.value()
    }

    pub fn is_closing(&self) -> bool {
        self.closing
    }

    pub fn open(&mut self, index: usize, animate: bool) {
        self.closing = false;
        self.selected = Some(index);
        self.position = Tween::settled(index as f64);
        if animate {
            self.presence
                .retarget(1.0, FADE_IN_DURATION, Easing::EaseOutCubic);
        } else {
            self.presence = Tween::settled(1.0);
        }
    }

    pub fn begin_close(&mut self) {
        if self.closing {
            return;
        }
        self.closing = true;
        self.stop_key_repeat();
        self.presence
            .retarget(0.0, FADE_OUT_DURATION, Easing::EaseOutCubic);
    }

    pub fn finish_close(&mut self) {
        self.closing = false;
        self.selected = None;
        self.position = Tween::settled(0.0);
        self.presence = Tween::settled(0.0);
    }

    pub fn slide_offset(&self, side: DockSide) -> f64 {
        let offset = (1.0 - self.presence().clamp(0.0, 1.0)) * PANEL_WIDTH;
        match side {
            DockSide::Left => -offset,
            DockSide::Right => offset,
        }
    }

    pub fn move_selection(&mut self, count: usize, delta: i32, duration: Duration) {
        if count == 0 || self.closing {
            return;
        }
        let (next, wrapped) = step_target(self.position.target(), delta as f64, count);
        self.selected = Some(next as usize);
        if wrapped {
            self.position.snap_to(next);
        } else {
            self.position.retarget(next, duration, Easing::EaseOutCubic);
        }
    }

    pub fn select(&mut self, index: usize, duration: Duration) {
        self.selected = Some(index);
        self.position
            .retarget(index as f64, duration, Easing::EaseOutCubic);
    }

    pub fn repeat_move(&mut self, count: usize, direction: i32) {
        if count == 0 || self.closing {
            return;
        }
        let (next, wrapped) = step_target(self.position.target(), direction as f64, count);
        self.selected = Some(next as usize);
        if wrapped {
            self.position.snap_to(next);
        } else {
            self.position.retarget(
                next,
                Duration::from_millis(KEY_REPEAT_INTERVAL_MS),
                Easing::Linear,
            );
        }
    }

    pub fn scroll_by(&mut self, count: usize, slots: f64) {
        if count == 0 || self.closing {
            return;
        }
        let (next, wrapped) = step_target(self.position.target(), slots.round(), count);
        self.selected = Some(next as usize);
        if wrapped {
            self.position.snap_to(next);
        } else {
            self.position
                .retarget(next, SCROLL_DURATION, Easing::EaseOutCubic);
        }
    }

    /// First key press starts repeat tracking; backend repeats are ignored.
    pub fn key_pressed(&mut self, direction: i32) -> bool {
        if self.closing || self.held_direction == Some(direction) {
            return false;
        }
        self.held_direction = Some(direction);
        let now = Instant::now();
        self.repeat_started = Some(now);
        self.last_repeat = Some(now);
        true
    }

    pub fn key_released(&mut self, direction: i32) {
        if self.held_direction == Some(direction) {
            self.stop_key_repeat();
        }
    }

    pub fn stop_key_repeat(&mut self) {
        self.held_direction = None;
        self.repeat_started = None;
        self.last_repeat = None;
    }

    pub fn tick_key_repeat(&mut self, count: usize) {
        let Some(direction) = self.held_direction else {
            return;
        };
        let Some(started) = self.repeat_started else {
            return;
        };
        let now = Instant::now();
        if now.duration_since(started) >= Duration::from_millis(KEY_REPEAT_DELAY_MS)
            && now.duration_since(self.last_repeat.unwrap_or(started))
                >= Duration::from_millis(KEY_REPEAT_INTERVAL_MS)
        {
            self.last_repeat = Some(now);
            self.repeat_move(count, direction);
        }
    }

    /// Cache tick for animation frames; `u64::MAX` when both tweens rest.
    pub fn animation_tick(&self) -> u64 {
        let presence = self.presence.is_animating();
        let position = self.position.is_animating();
        match (presence, position) {
            (true, true) => self.presence.frame_tick().max(self.position.frame_tick()),
            (true, false) => self.presence.frame_tick(),
            (false, true) => self.position.frame_tick(),
            (false, false) => u64::MAX,
        }
    }
}
