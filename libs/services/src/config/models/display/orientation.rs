//! Parsing and formatting of fixed display rotations.

/// Fixed rotation of a physical display output.
///
/// The string form matches both the config value and `wlr-randr --transform`,
/// so the compositor and the greeter share one vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonitorOrientation {
    /// No rotation (`normal` or `0`).
    Normal,
    /// Quarter turn clockwise (`90`).
    Rotate90,
    /// Half turn (`180`).
    Rotate180,
    /// Three quarter turns clockwise (`270`).
    Rotate270,
    /// Horizontally mirrored (`flipped`).
    Flipped,
    /// Mirrored then rotated (`flipped-90`).
    Flipped90,
    /// Mirrored then half turned (`flipped-180`).
    Flipped180,
    /// Mirrored then rotated three quarters (`flipped-270`).
    Flipped270,
}

impl MonitorOrientation {
    /// Parses a configured orientation value.
    ///
    /// # Arguments
    /// * `value` - Config value such as `normal`, `90` or `flipped-90`.
    ///
    /// # Returns
    /// The matching orientation, or `None` when the value is unknown.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "normal" | "0" => Some(Self::Normal),
            "90" | "left" => Some(Self::Rotate90),
            "180" | "inverted" => Some(Self::Rotate180),
            "270" | "right" => Some(Self::Rotate270),
            "flipped" => Some(Self::Flipped),
            "flipped-90" => Some(Self::Flipped90),
            "flipped-180" => Some(Self::Flipped180),
            "flipped-270" => Some(Self::Flipped270),
            _ => None,
        }
    }

    /// Returns the config/`wlr-randr` string form of this orientation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Rotate90 => "90",
            Self::Rotate180 => "180",
            Self::Rotate270 => "270",
            Self::Flipped => "flipped",
            Self::Flipped90 => "flipped-90",
            Self::Flipped180 => "flipped-180",
            Self::Flipped270 => "flipped-270",
        }
    }
}
