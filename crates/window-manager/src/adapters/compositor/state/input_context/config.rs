//! `InputContext` config delegation methods.

macro_rules! config_methods {
    () => {
        fn move_state_mut(&mut self) -> &mut Option<MoveState> {
            &mut self.input.move_state
        }

        fn resize_state_mut(&mut self) -> &mut Option<ResizeState> {
            &mut self.input.resize_state
        }

        fn last_titlebar_click_mut(
            &mut self,
        ) -> &mut Option<(Instant, Point<f64, Logical>, Window)> {
            &mut self.input.last_titlebar_click
        }

        fn trackpad_swipe(&self) -> Option<&TrackpadSwipeBuffer> {
            self.input.trackpad_swipe.as_ref()
        }

        fn trackpad_swipe_mut(&mut self) -> &mut Option<TrackpadSwipeBuffer> {
            &mut self.input.trackpad_swipe
        }

        fn touch_gesture(&self) -> &TouchGestureBuffer {
            &self.input.touch_gesture
        }

        fn touch_gesture_mut(&mut self) -> &mut TouchGestureBuffer {
            &mut self.input.touch_gesture
        }

        fn overlay_touch_mut(&mut self) -> &mut Option<OverlayTouchState> {
            &mut self.input.overlay_touch
        }

        fn touch_slots(&self) -> &HashSet<TouchSlot> {
            &self.input.touch_slots
        }

        fn touch_slots_mut(&mut self) -> &mut HashSet<TouchSlot> {
            &mut self.input.touch_slots
        }

        fn pointer_pressed_buttons(&self) -> &HashSet<u32> {
            &self.input.pointer_pressed_buttons
        }

        fn pointer_pressed_buttons_mut(&mut self) -> &mut HashSet<u32> {
            &mut self.input.pointer_pressed_buttons
        }

        fn pointer_location(&self) -> Point<f64, Logical> {
            self.pointer_location
        }

        fn set_pointer_location(&mut self, position: Point<f64, Logical>) {
            self.pointer_location = position;
        }

        fn set_cursor_visible(&mut self, visible: bool) {
            self.cursor_visible = visible;
        }

        fn gesture_config(&self) -> &GesturePolicy {
            self.config.gesture_config()
        }

        fn touchpad_natural_scroll(&self) -> bool {
            self.config.touchpad_natural_scroll()
        }

        fn mouse_natural_scroll(&self) -> bool {
            self.config.mouse_natural_scroll()
        }

        fn reload_input_config(&mut self) {
            BabydraState::reload_input_config(self);
        }

        fn reload_config(&mut self) {
            BabydraState::reload_config(self);
        }

        fn shortcut_command(&self, mods: Mods, keysym: u32) -> Option<String> {
            self.config.shortcut_command(mods, keysym)
        }

        fn reload_shortcuts(&self) {
            self.config.reload_shortcuts();
        }

        fn launch_command(&self, command: String) {
            self.config.launch_command(command);
        }

        fn lock_screen(&self) {
            crate::adapters::desktop::lock::spawn();
        }

        fn trigger_screenshot(&self) {
            crate::adapters::desktop::screenshot::trigger();
        }
    };
}
