//! Desktop theme and window decoration styling foundation for BabyDraDE.

#[derive(Debug, Clone, Copy)]
pub struct BorderTheme {
    /// Border thickness in physical pixels
    pub thickness: f32,
    /// Radius for rounded corners in physical pixels (0.0 for sharp corners)
    pub radius: f32,
    /// RGB start color for border gradient (normalized 0.0 - 1.0)
    pub start_color: [f32; 3],
    /// RGB end color for border gradient (normalized 0.0 - 1.0)
    pub end_color: [f32; 3],
    /// Angle of the linear gradient in radians (0.0 is left to right)
    pub gradient_angle: f32,
}

impl Default for BorderTheme {
    fn default() -> Self {
        Self {
            thickness: 6.0,
            radius: 8.0,
            // MagmaWM-inspired neon cyan glow
            start_color: [0.880, 1.0, 1.0],
            end_color: [0.580, 0.921, 0.921],
            gradient_angle: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DesktopTheme {
    /// Desktop canvas clear color [R, G, B, A]
    pub background_color: [f32; 4],
    /// Window border and corner styling
    pub border: BorderTheme,
}

impl Default for DesktopTheme {
    fn default() -> Self {
        Self {
            // MagmaWM-inspired sleek dark canvas foundation
            background_color: [0.10, 0.10, 0.10, 1.0],
            border: BorderTheme::default(),
        }
    }
}

impl DesktopTheme {
    /// MagmaWM-style cyan neon glow over a dark slate background
    pub fn magma_cyan() -> Self {
        Self::default()
    }

    /// Modern dark slate theme with royal blue borders
    pub fn dark_slate() -> Self {
        Self {
            background_color: [0.08, 0.09, 0.11, 1.0],
            border: BorderTheme {
                thickness: 5.0,
                radius: 10.0,
                start_color: [0.45, 0.65, 0.95],
                end_color: [0.30, 0.40, 0.80],
                gradient_angle: 0.25,
            },
        }
    }

    /// Emerald theme with vibrant green borders
    pub fn emerald() -> Self {
        Self {
            background_color: [0.07, 0.10, 0.08, 1.0],
            border: BorderTheme {
                thickness: 6.0,
                radius: 8.0,
                start_color: [0.35, 0.95, 0.65],
                end_color: [0.15, 0.70, 0.45],
                gradient_angle: 0.0,
            },
        }
    }
}
