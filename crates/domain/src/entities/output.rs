use crate::value_objects::{Rect, Size};
use serde::{Deserialize, Serialize};

/// Unique identifier for an output display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OutputId(pub u64);

impl OutputId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for OutputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Output({})", self.0)
    }
}

/// Physical output display screen representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Output {
    pub id: OutputId,
    pub name: String,
    pub geometry: Rect,
    pub scale: f64,
}

impl Output {
    pub fn new(id: OutputId, name: impl Into<String>, geometry: Rect, scale: f64) -> Self {
        Self {
            id,
            name: name.into(),
            geometry,
            scale: if scale <= 0.0 { 1.0 } else { scale },
        }
    }

    pub fn logical_size(&self) -> Size {
        let w = (self.geometry.width() as f64 / self.scale).round() as u32;
        let h = (self.geometry.height() as f64 / self.scale).round() as u32;
        Size::new(w, h)
    }

    pub fn logical_rect(&self) -> Rect {
        let size = self.logical_size();
        let x = (self.geometry.x() as f64 / self.scale).round() as i32;
        let y = (self.geometry.y() as f64 / self.scale).round() as i32;
        Rect::from_point_and_size(crate::value_objects::Point::new(x, y), size)
    }

    pub fn set_geometry(&mut self, geometry: Rect) {
        self.geometry = geometry;
    }

    pub fn set_scale(&mut self, scale: f64) {
        if scale > 0.0 {
            self.scale = scale;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_logical_geometry() {
        let out = Output::new(OutputId::new(1), "eDP-1", Rect::new(0, 0, 3840, 2160), 2.0);

        assert_eq!(out.logical_size(), Size::new(1920, 1080));
    }
}
