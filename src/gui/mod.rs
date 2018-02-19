pub mod main_window;
pub mod widget;
pub mod div;
pub mod view;

pub use self::view::View;

use self::widget::Widget;

use std::cmp::{max, min};

use nalgebra;
use nanovg;
use indextree as it;

pub type Point = nalgebra::Vector2<i32>;
pub type Size = nalgebra::Vector2<i32>;

fn point_min(a: Point, b: Point) -> Point {
    Point::new(min(a.x, b.x), min(a.y, b.y))
}

fn point_max(a: Point, b: Point) -> Point {
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
        (Bbox { min, max }).normalize()
    }

    pub fn with_size(origin: Point, size: Size) -> Self {
        Bbox::new(origin, origin + size)
    }

    pub fn normalize(mut self) -> Bbox {
        let Bbox { min: min_val, max: max_val } = self;
        self.min.x = min(min_val.x, max_val.x);
        self.min.y = min(min_val.y, max_val.y);
        self.max.x = max(min_val.x, max_val.x);
        self.max.y = max(min_val.y, max_val.y);
        self
    }

    pub fn size(&self) -> Size {
        self.max - self.min
    }

    /// Check whether this bounding box partially or completely contains the `other` bounding box.
    pub fn contains_bbox(self, other: Bbox) -> bool {
        let this = self.normalize();
        let other = other.normalize();
        let x = this.min.x <= other.max.x && this.max.x >= other.min.x;
        let y = this.min.y <= other.max.y && this.max.y >= other.min.y;
        x && y
    }

    /// Compute the bounding box made up by the area where `self` and `other` overlap.
    pub fn overlapping(self, other: Bbox) -> Option<Bbox> {
        if self.contains_bbox(other) {
            let min = point_max(self.min, other.min);
            let max = point_min(self.max, other.max);
            Some(Bbox::new(min, max))
        } else {
            None
        }
    }

    pub fn offset(mut self, p: Point) -> Bbox {
        self.min += p;
        self.max += p;
        self
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
        assert_eq!(
            b1.overlapping(b2),
            Some(Bbox::new(Point::new(2, 2), Point::new(4, 4)))
        );

        assert!(!b1.contains_bbox(b3));
        assert_eq!(b1.overlapping(b3), None);

        assert_eq!(
            b1.overlapping(b4),
            Some(Bbox::new(Point::new(0, 5), Point::new(8, 10)))
        );

        let b1 = Bbox::new(Point::new(10, 10), Point::new(20, 20));
        let b2 = b1.offset(Point::new(2, 5));
        assert_eq!(b2, Bbox::new(Point::new(12, 15), Point::new(22, 25)));

        let b2 = Bbox::new(Point::new(10, 10), Point::new(20, 5));
        assert!(b1.contains_bbox(b2));
        assert!(b1.contains_bbox(b2.offset(Point::new(-5, 0))));
    }
}
