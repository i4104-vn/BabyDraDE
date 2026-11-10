use smithay::{
    backend::input::{
        Axis, AxisSource, GestureBeginEvent, GestureEndEvent, GestureHoldBeginEvent as HoldBegin,
        GestureHoldEndEvent as HoldEnd, GesturePinchBeginEvent as PinchBegin,
        GesturePinchEndEvent as PinchEnd, GesturePinchUpdateEvent as PinchUpdate,
        GestureSwipeBeginEvent as SwipeBegin, GestureSwipeEndEvent as SwipeEnd,
        GestureSwipeUpdateEvent as SwipeUpdate, InputBackend, PointerAxisEvent,
    },
    input::pointer::{
        AxisFrame, GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent,
        GesturePinchEndEvent, GesturePinchUpdateEvent, GestureSwipeBeginEvent,
        GestureSwipeEndEvent, GestureSwipeUpdateEvent,
    },
    utils::{Point, SERIAL_COUNTER},
};

use super::super::state::{BabydraState, TrackpadSwipeBuffer};
use super::keyboard::cycle_window_focus;

const WINDOW_SWITCH_FINGERS: u32 = 3;
const WINDOW_SWITCH_THRESHOLD: f64 = 100.0;

pub(crate) fn handle_axis<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: PointerAxisEvent<B>,
{
    state.cursor_visible = true;
    let source = event.source();
    let mut frame = AxisFrame::new(event.time_msec())
        .source(source)
        .relative_direction(Axis::Horizontal, event.relative_direction(Axis::Horizontal))
        .relative_direction(Axis::Vertical, event.relative_direction(Axis::Vertical));

    for axis in [Axis::Horizontal, Axis::Vertical] {
        let amount = event.amount(axis);
        let amount_v120 = event.amount_v120(axis);

        if let Some(amount) = amount {
            if source == AxisSource::Finger && amount == 0.0 {
                frame = frame.stop(axis);
            } else {
                frame = frame.value(axis, amount);
            }
        } else if let Some(amount_v120) = amount_v120 {
            // Provide a legacy pixel value as well as v120 for wheel-only backends.
            frame = frame.value(axis, amount_v120 / 8.0);
        }

        if let Some(amount_v120) = amount_v120 {
            frame = frame.v120(axis, amount_v120.round() as i32);
        }
    }

    if let Some(pointer) = state.seat.get_pointer() {
        pointer.axis(state, frame);
        pointer.frame(state);
    }
}

pub(crate) fn handle_swipe_begin<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: SwipeBegin<B> + GestureBeginEvent<B>,
{
    let fingers = event.fingers();
    let time = event.time_msec();

    if fingers == WINDOW_SWITCH_FINGERS && state.space.elements().count() > 1 {
        state.trackpad_swipe = Some(TrackpadSwipeBuffer {
            fingers,
            time,
            total_delta: Point::from((0.0, 0.0)),
            updates: Vec::new(),
        });
        return;
    }

    if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_swipe_begin(
            state,
            &GestureSwipeBeginEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time,
                fingers,
            },
        );
    }
}

pub(crate) fn handle_swipe_update<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: SwipeUpdate<B>,
{
    let delta = event.delta();
    let time = event.time_msec();

    if let Some(gesture) = state.trackpad_swipe.as_mut() {
        gesture.total_delta.x += delta.x;
        gesture.total_delta.y += delta.y;
        gesture.updates.push((delta, time));
        return;
    }

    if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_swipe_update(state, &GestureSwipeUpdateEvent { time, delta });
    }
}

pub(crate) fn handle_swipe_end<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: SwipeEnd<B> + GestureEndEvent<B>,
{
    let time = event.time_msec();
    let cancelled = event.cancelled();

    if let Some(gesture) = state.trackpad_swipe.take() {
        let horizontal_swipe = gesture.total_delta.x.abs() >= WINDOW_SWITCH_THRESHOLD
            && gesture.total_delta.x.abs() > gesture.total_delta.y.abs();

        if !cancelled && horizontal_swipe {
            let forward = gesture.total_delta.x < 0.0;
            if cycle_window_focus(state, forward, SERIAL_COUNTER.next_serial()) {
                return;
            }
        }

        replay_swipe(state, gesture, cancelled, time);
    } else if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_swipe_end(
            state,
            &GestureSwipeEndEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time,
                cancelled,
            },
        );
    }
}

fn replay_swipe(
    state: &mut BabydraState,
    gesture: TrackpadSwipeBuffer,
    cancelled: bool,
    end_time: u32,
) {
    let Some(pointer) = state.seat.get_pointer() else {
        return;
    };

    pointer.gesture_swipe_begin(
        state,
        &GestureSwipeBeginEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: gesture.time,
            fingers: gesture.fingers,
        },
    );
    for (delta, time) in gesture.updates {
        pointer.gesture_swipe_update(state, &GestureSwipeUpdateEvent { time, delta });
    }
    pointer.gesture_swipe_end(
        state,
        &GestureSwipeEndEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: end_time,
            cancelled,
        },
    );
}

pub(crate) fn handle_pinch_begin<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: PinchBegin<B> + GestureBeginEvent<B>,
{
    if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_pinch_begin(
            state,
            &GesturePinchBeginEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time: event.time_msec(),
                fingers: event.fingers(),
            },
        );
    }
}

pub(crate) fn handle_pinch_update<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: PinchUpdate<B>,
{
    if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_pinch_update(
            state,
            &GesturePinchUpdateEvent {
                time: event.time_msec(),
                delta: event.delta(),
                scale: event.scale(),
                rotation: event.rotation(),
            },
        );
    }
}

pub(crate) fn handle_pinch_end<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: PinchEnd<B> + GestureEndEvent<B>,
{
    if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_pinch_end(
            state,
            &GesturePinchEndEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time: event.time_msec(),
                cancelled: event.cancelled(),
            },
        );
    }
}

pub(crate) fn handle_hold_begin<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: HoldBegin<B> + GestureBeginEvent<B>,
{
    if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_hold_begin(
            state,
            &GestureHoldBeginEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time: event.time_msec(),
                fingers: event.fingers(),
            },
        );
    }
}

pub(crate) fn handle_hold_end<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: HoldEnd<B> + GestureEndEvent<B>,
{
    if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_hold_end(
            state,
            &GestureHoldEndEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time: event.time_msec(),
                cancelled: event.cancelled(),
            },
        );
    }
}

/// Touchpad configuration options loaded from environment or defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct TouchpadConfig {
    pub tap_to_click: bool,
    pub tap_drag: bool,
    pub tap_drag_lock: bool,
    pub button_map: smithay::reexports::input::TapButtonMap,
    pub dwt: bool,
    pub natural_scroll: bool,
    pub speed: Option<f64>,
}

impl Default for TouchpadConfig {
    fn default() -> Self {
        Self {
            tap_to_click: true,
            tap_drag: true,
            tap_drag_lock: false,
            button_map: smithay::reexports::input::TapButtonMap::LeftRightMiddle,
            dwt: true,
            natural_scroll: false,
            speed: None,
        }
    }
}

impl TouchpadConfig {
    pub fn from_env() -> Self {
        let tap_to_click = std::env::var("BABYDRADE_TAP_TO_CLICK")
            .map(|v| v != "0" && v != "false" && v != "no")
            .unwrap_or(true);

        let tap_drag = std::env::var("BABYDRADE_TAP_DRAG")
            .map(|v| v != "0" && v != "false" && v != "no")
            .unwrap_or(true);

        let tap_drag_lock = std::env::var("BABYDRADE_TAP_DRAG_LOCK")
            .map(|v| v == "1" || v == "true" || v == "yes")
            .unwrap_or(false);

        let button_map = match std::env::var("BABYDRADE_TAP_BUTTON_MAP").as_deref() {
            Ok("lmr") => smithay::reexports::input::TapButtonMap::LeftMiddleRight,
            _ => smithay::reexports::input::TapButtonMap::LeftRightMiddle,
        };

        let dwt = std::env::var("BABYDRADE_TOUCHPAD_DWT")
            .map(|v| v != "0" && v != "false" && v != "no")
            .unwrap_or(true);

        let natural_scroll = std::env::var("BABYDRADE_NATURAL_SCROLL")
            .map(|v| v == "1" || v == "true" || v == "yes")
            .unwrap_or(false);

        let speed = std::env::var("BABYDRADE_TOUCHPAD_SPEED")
            .or_else(|_| std::env::var("BABYDRADE_POINTER_SPEED"))
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .map(|s| s.clamp(-1.0, 1.0));

        Self {
            tap_to_click,
            tap_drag,
            tap_drag_lock,
            button_map,
            dwt,
            natural_scroll,
            speed,
        }
    }
}

/// Configure a newly connected input device if it is a touchpad / trackpad.
pub(crate) fn handle_device_added<D: std::any::Any>(device: &mut D) {
    if let Some(libinput_device) = (device as &mut dyn std::any::Any)
        .downcast_mut::<smithay::reexports::input::Device>()
    {
        let config = TouchpadConfig::from_env();
        configure_touchpad(libinput_device, &config);
    }
}

/// Log when an input device is disconnected.
pub(crate) fn handle_device_removed<D: std::any::Any>(device: &D) {
    if let Some(libinput_device) =
        (device as &dyn std::any::Any).downcast_ref::<smithay::reexports::input::Device>()
    {
        tracing::info!(
            "Input device removed: '{}' ({})",
            libinput_device.name(),
            libinput_device.sysname()
        );
    }
}

/// Configure touchpad settings: tap-to-click, tap-and-drag, button mapping,
/// clickfinger, natural scrolling, disable-while-typing, and speed.
pub(crate) fn configure_touchpad(
    device: &mut smithay::reexports::input::Device,
    config: &TouchpadConfig,
) {
    use smithay::reexports::input::ClickMethod;
    use tracing::{info, warn};

    let name = device.name().to_string();
    let sysname = device.sysname().to_string();

    let tap_finger_count = device.config_tap_finger_count();
    if tap_finger_count > 0 {
        info!(
            "Trackpad detected: '{}' ({}), supported tap fingers: {}",
            name, sysname, tap_finger_count
        );

        // 1. Tap-to-click (enabled by default so users can tap without pressing physical buttons)
        if config.tap_to_click {
            match device.config_tap_set_enabled(true) {
                Ok(()) => info!("Tap-to-click successfully enabled on '{}'", name),
                Err(err) => warn!("Failed to enable tap-to-click on '{}': {:?}", name, err),
            }

            // 2. Tap-and-drag (tap then hold and move to drag)
            if let Err(err) = device.config_tap_set_drag_enabled(config.tap_drag) {
                warn!("Failed to set tap-and-drag on '{}': {:?}", name, err);
            }

            // 3. Tap drag lock
            if let Err(err) = device.config_tap_set_drag_lock_enabled(config.tap_drag_lock) {
                warn!("Failed to set tap drag lock on '{}': {:?}", name, err);
            }

            // 4. Tap button map (1 finger = Left, 2 fingers = Right, 3 fingers = Middle)
            if let Err(err) = device.config_tap_set_button_map(config.button_map) {
                warn!("Failed to set tap button map on '{}': {:?}", name, err);
            }
        } else {
            let _ = device.config_tap_set_enabled(false);
            info!(
                "Tap-to-click disabled on '{}' via BABYDRADE_TAP_TO_CLICK=0",
                name
            );
        }

        // 5. Click method: clickfinger (1 finger = left click, 2 fingers = right click anywhere on pad)
        if device
            .config_click_methods()
            .contains(&ClickMethod::Clickfinger)
        {
            if let Err(err) = device.config_click_set_method(ClickMethod::Clickfinger) {
                warn!("Failed to set Clickfinger method on '{}': {:?}", name, err);
            }
        }

        // 6. Disable while typing (DWT)
        if device.config_dwt_is_available() {
            let _ = device.config_dwt_set_enabled(config.dwt);
        }

        // 7. Natural scrolling
        if device.config_scroll_has_natural_scroll() {
            let _ = device.config_scroll_set_natural_scroll_enabled(config.natural_scroll);
        }

        // 8. Middle click emulation
        if device.config_middle_emulation_is_available() {
            let _ = device.config_middle_emulation_set_enabled(true);
        }

        // 9. Pointer acceleration / speed
        if device.config_accel_is_available() {
            if let Some(speed) = config.speed {
                let _ = device.config_accel_set_speed(speed);
            }
        }
    } else {
        tracing::debug!(
            "Input device '{}' ({}) does not support tapping",
            name,
            sysname
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::reexports::input::TapButtonMap;

    #[test]
    fn test_touchpad_config_defaults() {
        let config = TouchpadConfig::default();
        assert!(config.tap_to_click);
        assert!(config.tap_drag);
        assert!(!config.tap_drag_lock);
        assert_eq!(config.button_map, TapButtonMap::LeftRightMiddle);
        assert!(config.dwt);
        assert!(!config.natural_scroll);
        assert_eq!(config.speed, None);
    }

    #[test]
    fn test_handle_device_non_libinput() {
        struct MockDevice {
            _id: u32,
        }
        let mut mock = MockDevice { _id: 42 };
        // Should not panic on non-libinput devices (e.g. Winit devices)
        handle_device_added(&mut mock);
        handle_device_removed(&mock);
    }
}
