//! Input device configuration models.

/// Pointer / mouse configuration options loaded from `~/.babydra/configs/input.toml`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MouseConfig {
    /// Whether natural (reverse) scrolling direction is enabled for mouse wheels.
    pub(crate) natural_scroll: bool,
    /// Optional pointer acceleration speed (-1.0 to 1.0).
    pub(crate) speed: Option<f64>,
    /// Whether middle mouse button emulation (pressing left and right together) is enabled.
    pub(crate) middle_emulation: bool,
}

impl Default for MouseConfig {
    /// Creates default mouse configuration.
    ///
    /// By default:
    /// - `natural_scroll` is `false`.
    /// - `speed` is `None` (uses hardware default).
    /// - `middle_emulation` is `false`.
    fn default() -> Self {
        Self {
            natural_scroll: false,
            speed: None,
            middle_emulation: false,
        }
    }
}

impl From<services::models::MouseConfig> for MouseConfig {
    /// Converts a persistent [`services::models::MouseConfig`] from `configs/input.toml`
    /// into compositor runtime [`MouseConfig`].
    ///
    /// # Arguments
    ///
    /// * `cfg` - The deserialized mouse configuration from `configs/input.toml`.
    ///
    /// # Returns
    ///
    /// Returns the converted [`MouseConfig`].
    fn from(cfg: services::models::MouseConfig) -> Self {
        Self {
            natural_scroll: cfg.natural_scroll,
            speed: cfg.speed.map(|s| s.clamp(-1.0, 1.0)),
            middle_emulation: cfg.middle_emulation,
        }
    }
}

impl MouseConfig {
    /// Loads the mouse configuration directly from `~/.babydra/configs/input.toml` under `[mouse]`.
    ///
    /// # Returns
    ///
    /// Returns a resolved [`MouseConfig`] mapped from the shared [`services::models::MouseConfig`].
    pub(crate) fn load() -> Self {
        let shared_cfg = services::config::load_mouse_config();
        Self::from(shared_cfg)
    }
}

// Touchpad runtime configuration model.

pub(crate) struct TouchpadConfig {
    /// Whether tap-to-click is enabled (1-finger left click, 2-finger right click).
    pub(crate) tap_to_click: bool,
    /// Whether tap-and-drag is enabled (tap once then slide to drag).
    pub(crate) tap_drag: bool,
    /// Whether tap drag lock is enabled.
    pub(crate) tap_drag_lock: bool,
    /// Mapping of tap finger count to mouse buttons.
    pub(crate) button_map: smithay::reexports::input::TapButtonMap,
    /// Whether disable-while-typing (DWT) is active.
    pub(crate) dwt: bool,
    /// Whether natural (reverse) scrolling direction is enabled.
    pub(crate) natural_scroll: bool,
    /// Optional pointer acceleration speed (-1.0 to 1.0).
    pub(crate) speed: Option<f64>,
}

impl Default for TouchpadConfig {
    /// Creates default touchpad configuration suitable for modern laptops.
    ///
    /// By default:
    /// - `tap_to_click` is `true`.
    /// - `tap_drag` is `true`.
    /// - `tap_drag_lock` is `false`.
    /// - `button_map` is `LeftRightMiddle` (1 finger = Left, 2 fingers = Right, 3 fingers = Middle).
    /// - `dwt` is `true`.
    /// - `natural_scroll` is `false`.
    /// - `speed` is `None` (uses hardware default).
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

impl From<services::models::TouchpadConfig> for TouchpadConfig {
    /// Converts a persistent [`services::models::TouchpadConfig`] from `configs/input.toml`
    /// into compositor runtime [`TouchpadConfig`] with Smithay input types.
    ///
    /// # Arguments
    ///
    /// * `cfg` - The deserialized touchpad configuration from `configs/input.toml`.
    ///
    /// # Returns
    ///
    /// Returns the converted [`TouchpadConfig`].
    fn from(cfg: services::models::TouchpadConfig) -> Self {
        let button_map = match cfg.button_map.as_str() {
            "lmr" => smithay::reexports::input::TapButtonMap::LeftMiddleRight,
            _ => smithay::reexports::input::TapButtonMap::LeftRightMiddle,
        };

        Self {
            tap_to_click: cfg.tap_to_click,
            tap_drag: cfg.tap_drag,
            tap_drag_lock: cfg.tap_drag_lock,
            button_map,
            dwt: cfg.dwt,
            natural_scroll: cfg.natural_scroll,
            speed: cfg.speed.map(|s| s.clamp(-1.0, 1.0)),
        }
    }
}

impl TouchpadConfig {
    /// Loads the touchpad configuration directly from `~/.babydra/configs/input.toml` under `[touchpad]`.
    ///
    /// # Returns
    ///
    /// Returns a resolved [`TouchpadConfig`] mapped from the shared [`services::models::TouchpadConfig`].
    pub(crate) fn load() -> Self {
        let shared_cfg = services::config::load_touchpad_config();
        Self::from(shared_cfg)
    }
}
