//! Framework-free geometry value types.
//!
//! These are plain data containers: they know nothing about a window system,
//! rendering backend or protocol. Adapters convert their framework-specific
//! geometry to and from these types at the boundary.

/// A point in two-dimensional space.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Point<T> {
    pub(crate) x: T,
    pub(crate) y: T,
}

/// A two-dimensional extent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Size<T> {
    pub(crate) w: T,
    pub(crate) h: T,
}

/// An axis-aligned rectangle: an origin plus an extent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Rectangle<T> {
    pub(crate) loc: Point<T>,
    pub(crate) size: Size<T>,
}

impl<T> From<(T, T)> for Point<T> {
    fn from((x, y): (T, T)) -> Self {
        Self { x, y }
    }
}

impl<T> From<(T, T)> for Size<T> {
    fn from((w, h): (T, T)) -> Self {
        Self { w, h }
    }
}

impl<T> Rectangle<T> {
    /// Builds a rectangle from an origin and an extent.
    pub(crate) fn new(loc: Point<T>, size: Size<T>) -> Self {
        Self { loc, size }
    }
}

impl<T> std::ops::Add for Point<T>
where
    T: std::ops::Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_and_sizes_build_from_tuples() {
        assert_eq!(Point::from((3, 4)), Point { x: 3, y: 4 });
        assert_eq!(Size::from((5, 6)), Size { w: 5, h: 6 });
    }

    #[test]
    fn points_add_component_wise() {
        assert_eq!(
            Point::from((1, 2)) + Point::from((3, 4)),
            Point::from((4, 6))
        );
    }

    #[test]
    fn rectangles_combine_location_and_size() {
        let rect = Rectangle::new(Point::from((10, 20)), Size::from((30, 40)));
        assert_eq!(rect.loc, Point::from((10, 20)));
        assert_eq!(rect.size, Size::from((30, 40)));
    }
}
