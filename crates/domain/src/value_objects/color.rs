use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorParseError {
    InvalidFormat,
    InvalidHexValue,
}

impl std::fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat => write!(
                f,
                "Invalid color format, expected #RGB, #RGBA, #RRGGBB, or #RRGGBBAA"
            ),
            Self::InvalidHexValue => write!(f, "Invalid hexadecimal character in color string"),
        }
    }
}

impl std::error::Error for ColorParseError {}

impl Color {
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const GREEN: Self = Self {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    pub fn to_rgba_f32(self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    pub fn to_rgba_u32(self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }

    pub fn from_hex(hex: &str) -> Result<Self, ColorParseError> {
        let hex = hex.trim().strip_prefix('#').unwrap_or(hex.trim());
        match hex.len() {
            // #RGB
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16)
                    .map_err(|_| ColorParseError::InvalidHexValue)?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16)
                    .map_err(|_| ColorParseError::InvalidHexValue)?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16)
                    .map_err(|_| ColorParseError::InvalidHexValue)?;
                Ok(Self::rgb(r, g, b))
            }
            // #RGBA
            4 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16)
                    .map_err(|_| ColorParseError::InvalidHexValue)?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16)
                    .map_err(|_| ColorParseError::InvalidHexValue)?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16)
                    .map_err(|_| ColorParseError::InvalidHexValue)?;
                let a = u8::from_str_radix(&hex[3..4].repeat(2), 16)
                    .map_err(|_| ColorParseError::InvalidHexValue)?;
                Ok(Self::rgba(r, g, b, a))
            }
            // #RRGGBB
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| ColorParseError::InvalidHexValue)?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| ColorParseError::InvalidHexValue)?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| ColorParseError::InvalidHexValue)?;
                Ok(Self::rgb(r, g, b))
            }
            // #RRGGBBAA
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| ColorParseError::InvalidHexValue)?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| ColorParseError::InvalidHexValue)?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| ColorParseError::InvalidHexValue)?;
                let a = u8::from_str_radix(&hex[6..8], 16)
                    .map_err(|_| ColorParseError::InvalidHexValue)?;
                Ok(Self::rgba(r, g, b, a))
            }
            _ => Err(ColorParseError::InvalidFormat),
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_constructors() {
        let c = Color::rgb(255, 128, 0);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 255);

        let with_a = c.with_alpha(100);
        assert_eq!(with_a.a, 100);
    }

    #[test]
    fn test_hex_parsing() {
        assert_eq!(Color::from_hex("#fff").unwrap(), Color::WHITE);
        assert_eq!(Color::from_hex("000000").unwrap(), Color::BLACK);
        assert_eq!(
            Color::from_hex("#ff000080").unwrap(),
            Color::rgba(255, 0, 0, 128)
        );
        assert!(Color::from_hex("#xyz").is_err());
        assert!(Color::from_hex("#12345").is_err());
    }

    #[test]
    fn test_color_conversions() {
        let c = Color::rgba(255, 0, 0, 255);
        let f = c.to_rgba_f32();
        assert_eq!(f, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(c.to_rgba_u32(), 0xff0000ff);
    }
}
