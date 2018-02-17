pub mod main_window;
pub mod widget;
pub mod div;
pub mod view;

pub use self::view::View;

use self::widget::Widget;

use std::cmp::{max, min};
use std::cell::{RefCell, Ref};

use nalgebra;
use nanovg;
use indextree as it;

pub type Point = nalgebra::Vector2<i32>;
pub type Size = nalgebra::Vector2<i32>;

pub fn point_min(a: Point, b: Point) -> Point {
    Point::new(min(a.x, b.x), min(a.y, b.y))
}

pub fn point_max(a: Point, b: Point) -> Point {
    Point::new(max(a.x, b.x), max(a.y, b.y))
}

#[derive(Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn white() -> Self {
        Color::rgba(1.0, 1.0, 1.0, 1.0)
    }

    pub fn red() -> Self {
        Color::rgba(1.0, 0.0, 0.0, 1.0)
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Into<nanovg::Color> for Color {
    fn into(self) -> nanovg::Color {
        nanovg::Color::new(self.r, self.g, self.b, self.a)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bbox {
    pub min: Point,
    pub max: Point,
}

impl Bbox {
    pub fn new(min: Point, max: Point) -> Self {
        Bbox { min, max }
    }

    pub fn with_size(origin: Point, size: Size) -> Self {
        Bbox::new(origin, origin + size)
    }

    pub fn size(&self) -> Size {
        self.max - self.min
    }

    /// Check whether this bounding box partially or completely contains the `other` bounding box.
    pub fn contains_bbox(&self, other: Bbox) -> bool {
        let between = |x, min, max| x >= min && x <= max;
        let x = between(other.min.x, self.min.x, self.max.x)
            || between(other.max.x, self.min.x, self.max.x);
        let y = between(other.min.y, self.min.y, self.max.y)
            || between(other.max.y, self.min.y, self.max.y);
        x && y
    }

    /// Compute the bounding box made up by the area where `self` and `other` overlap.
    pub fn overlapping(&self, other: Bbox) -> Option<Bbox> {
        if self.contains_bbox(other) {
            let min = point_max(self.min, other.min);
            let max = point_min(self.max, other.max);
            Some(Bbox::new(min, max))
        } else {
            None
        }
    }
}

impl Into<nanovg::Scissor> for Bbox {
    fn into(self) -> nanovg::Scissor {
        let size = self.size();
        nanovg::Scissor::Rect {
            x: self.min.x as f32,
            y: self.min.y as f32,
            width: size.x as f32,
            height: size.y as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbox() {
        let b1 = Bbox::new(Point::new(0, 0), Point::new(10, 10));
        let b2 = Bbox::new(Point::new(2, 2), Point::new(4, 4));
        let b3 = Bbox::new(Point::new(20, 20), Point::new(25, 25));
        let b4 = Bbox::new(Point::new(0, 5), Point::new(8, 15));

        assert!(b1.contains_bbox(b2));
        assert_eq!(b1.overlapping(b2), Some(Bbox::new(Point::new(2, 2), Point::new(4, 4))));

        assert!(!b1.contains_bbox(b3));
        assert_eq!(b1.overlapping(b3), None);

        assert_eq!(b1.overlapping(b4), Some(Bbox::new(Point::new(0, 5), Point::new(8, 10))));
    }
}