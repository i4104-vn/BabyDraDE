//! `InputContext` devices delegation methods.

macro_rules! devices_methods {
    () => {
        fn has_keyboard(&self) -> bool {
            self.wayland.seat().get_keyboard().is_some()
        }

        fn keyboard_modifiers(&self) -> ModifiersState {
            match self.wayland.seat().get_keyboard() {
                Some(keyboard) => keyboard.modifier_state(),
                None => {
                    tracing::warn!("keyboard input event received but the seat has no keyboard");
                    ModifiersState::default()
                }
            }
        }

        fn keyboard_input<T, F>(
            &mut self,
            keycode: Keycode,
            state: KeyState,
            serial: Serial,
            time: u32,
            filter: F,
        ) -> Option<T>
        where
            F: FnOnce(&mut Self, &ModifiersState, KeysymHandle<'_>) -> FilterResult<T>,
        {
            if let Some(keyboard) = self.wayland.seat().get_keyboard() {
                keyboard.input::<T, F>(self, keycode, state, serial, time, filter)
            } else {
                tracing::warn!("keyboard input event received but the seat has no keyboard");
                None
            }
        }

        fn set_keyboard_focus(&mut self, focus: Option<WlSurface>, serial: Serial) {
            if let Some(keyboard) = self.wayland.seat().get_keyboard() {
                keyboard.set_focus(self, focus, serial);
            }
        }

        fn has_pointer(&self) -> bool {
            self.wayland.seat().get_pointer().is_some()
        }

        fn pointer_is_grabbed(&self) -> bool {
            match self.wayland.seat().get_pointer() {
                Some(pointer) => pointer.is_grabbed(),
                None => {
                    tracing::warn!("pointer state queried but the seat has no pointer");
                    false
                }
            }
        }

        fn pointer_motion(
            &mut self,
            focus: Option<(WlSurface, Point<f64, Logical>)>,
            event: &PointerMotionEvent,
        ) {
            if let Some(pointer) = self.wayland.seat().get_pointer() {
                pointer.motion(self, focus, event);
            } else {
                tracing::warn!("pointer motion event received but the seat has no pointer");
            }
        }

        fn pointer_button(&mut self, event: &ButtonEvent) {
            if let Some(pointer) = self.wayland.seat().get_pointer() {
                pointer.button(self, event);
            } else {
                tracing::warn!("pointer button event received but the seat has no pointer");
            }
        }

        fn pointer_axis(&mut self, frame: AxisFrame) {
            if let Some(pointer) = self.wayland.seat().get_pointer() {
                pointer.axis(self, frame);
            } else {
                tracing::warn!("pointer axis event received but the seat has no pointer");
            }
        }

        fn pointer_frame(&mut self) {
            if let Some(pointer) = self.wayland.seat().get_pointer() {
                pointer.frame(self);
            } else {
                tracing::warn!("pointer frame event received but the seat has no pointer");
            }
        }

        fn touch_down(
            &mut self,
            focus: Option<(WlSurface, Point<f64, Logical>)>,
            event: &TouchDownEvent,
        ) {
            if let Some(touch) = self.wayland.seat().get_touch() {
                touch.down(self, focus, event);
            }
        }

        fn touch_motion(
            &mut self,
            focus: Option<(WlSurface, Point<f64, Logical>)>,
            event: &TouchMotionEvent,
        ) {
            if let Some(touch) = self.wayland.seat().get_touch() {
                touch.motion(self, focus, event);
            }
        }

        fn touch_up(&mut self, event: &TouchUpEvent) {
            if let Some(touch) = self.wayland.seat().get_touch() {
                touch.up(self, event);
            }
        }

        fn touch_cancel(&mut self) {
            if let Some(touch) = self.wayland.seat().get_touch() {
                touch.cancel(self);
            }
        }

        fn touch_frame(&mut self) {
            if let Some(touch) = self.wayland.seat().get_touch() {
                touch.frame(self);
            }
        }

        fn gesture_swipe_begin(&mut self, event: &GestureSwipeBeginEvent) {
            if let Some(pointer) = self.wayland.seat().get_pointer() {
                pointer.gesture_swipe_begin(self, event);
            }
        }

        fn gesture_swipe_update(&mut self, event: &GestureSwipeUpdateEvent) {
            if let Some(pointer) = self.wayland.seat().get_pointer() {
                pointer.gesture_swipe_update(self, event);
            }
        }

        fn gesture_swipe_end(&mut self, event: &GestureSwipeEndEvent) {
            if let Some(pointer) = self.wayland.seat().get_pointer() {
                pointer.gesture_swipe_end(self, event);
            }
        }

        fn gesture_pinch_begin(&mut self, event: &GesturePinchBeginEvent) {
            if let Some(pointer) = self.wayland.seat().get_pointer() {
                pointer.gesture_pinch_begin(self, event);
            }
        }

        fn gesture_pinch_update(&mut self, event: &GesturePinchUpdateEvent) {
            if let Some(pointer) = self.wayland.seat().get_pointer() {
                pointer.gesture_pinch_update(self, event);
            }
        }

        fn gesture_pinch_end(&mut self, event: &GesturePinchEndEvent) {
            if let Some(pointer) = self.wayland.seat().get_pointer() {
                pointer.gesture_pinch_end(self, event);
            }
        }

        fn gesture_hold_begin(&mut self, event: &GestureHoldBeginEvent) {
            if let Some(pointer) = self.wayland.seat().get_pointer() {
                pointer.gesture_hold_begin(self, event);
            }
        }

        fn gesture_hold_end(&mut self, event: &GestureHoldEndEvent) {
            if let Some(pointer) = self.wayland.seat().get_pointer() {
                pointer.gesture_hold_end(self, event);
            }
        }
    };
}
