use crate::value_objects::Size;
use serde::{Deserialize, Serialize};

/// Constraints on window dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SizeConstraints {
    pub min_size: Option<Size>,
    pub max_size: Option<Size>,
    pub aspect_ratio: Option<(u32, u32)>,
}

impl SizeConstraints {
    pub const UNCONSTRAINED: Self = Self {
        min_size: None,
        max_size: None,
        aspect_ratio: None,
    };

    pub fn new(min_size: Option<Size>, max_size: Option<Size>) -> Self {
        Self {
            min_size,
            max_size,
            aspect_ratio: None,
        }
    }

    pub fn with_aspect_ratio(mut self, width: u32, height: u32) -> Self {
        if width > 0 && height > 0 {
            self.aspect_ratio = Some((width, height));
        }
        self
    }

    /// Clamps and adjusts a requested size to conform to constraints.
    pub fn clamp(&self, requested: Size) -> Size {
        let mut width = requested.width;
        let mut height = requested.height;

        // Apply minimum size
        if let Some(min) = self.min_size {
            width = width.max(min.width);
            height = height.max(min.height);
        }

        // Apply maximum size
        if let Some(max) = self.max_size {
            if max.width > 0 {
                width = width.min(max.width);
            }
            if max.height > 0 {
                height = height.min(max.height);
            }
        }

        // Apply aspect ratio if present
        if let Some((ar_w, ar_h)) = self.aspect_ratio {
            let target_height = (width as u64 * ar_h as u64 / ar_w as u64) as u32;
            height = target_height;

            // Re-check min/max height bounds if necessary
            if let Some(min) = self.min_size {
                if height < min.height {
                    height = min.height;
                    width = (height as u64 * ar_w as u64 / ar_h as u64) as u32;
                }
            }
            if let Some(max) = self.max_size {
                if max.height > 0 && height > max.height {
                    height = max.height;
                    width = (height as u64 * ar_w as u64 / ar_h as u64) as u32;
                }
            }
        }

        Size::new(width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_constraints_clamping() {
        let constraints =
            SizeConstraints::new(Some(Size::new(200, 150)), Some(Size::new(1200, 900)));

        // Within bounds
        assert_eq!(constraints.clamp(Size::new(800, 600)), Size::new(800, 600));

        // Too small -> clamped to min
        assert_eq!(constraints.clamp(Size::new(100, 100)), Size::new(200, 150));

        // Too large -> clamped to max
        assert_eq!(
            constraints.clamp(Size::new(2000, 1500)),
            Size::new(1200, 900)
        );
    }

    #[test]
    fn test_aspect_ratio_constraint() {
        let constraints = SizeConstraints::UNCONSTRAINED.with_aspect_ratio(16, 9);
        let clamped = constraints.clamp(Size::new(1600, 1000));
        assert_eq!(clamped, Size::new(1600, 900));
    }
}
