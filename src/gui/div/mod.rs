pub mod div_iter;
pub use self::div_iter::*;

use super::*;

use std::cell::{Cell, RefCell};

#[derive(Clone, Copy)]
pub struct UnitCalcData<'a, 'b: 'a> {
    pub div: &'a SpaceDiv<'b>,
    pub direction: Direction,
    pub parent_size: Size,
    pub remaining: i32,
}

pub enum Unit {
    /// Absolute pixels.
    Pixels(i32),
    /// Pixels relative to a parent.
    Relative(f32),
    /// Calculate the pixels dynamically through a closure.
    Calc(Box<Fn(UnitCalcData) -> i32>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Min,
    Max,
    Center,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    /// Nothing happens, the content just overflows.
    Overflow,
    /// The content gets cut off where it overflows.
    Clip,
    /// The content is scrollable.
    Scroll,
}

#[derive(Clone, Copy)]
pub enum ComputedVisibility {
    Visible {
        bbox: Bbox,
        x: AxisOverflowBehaviour,
        y: AxisOverflowBehaviour,
    },
    Invisible,
}

#[derive(Clone, Copy)]
pub enum AxisOverflowBehaviour {
    Clip { min: i32, max: i32 },
    Scroll { min: i32, max: i32, scroll: i32 },
    Overflow,
}

impl AxisOverflowBehaviour {
    pub fn min_max(&self) -> Option<(i32, i32)> {
        match *self {
            AxisOverflowBehaviour::Clip { min, max }
            | AxisOverflowBehaviour::Scroll { min, max, .. } => Some((min, max)),
            _ => None,
        }
    }

    pub fn scroll(&self) -> Option<i32> {
        if let AxisOverflowBehaviour::Scroll { scroll, .. } = *self {
            Some(scroll)
        } else {
            None
        }
    }
}

pub struct SpaceDiv<'a> {
    pub width: Unit,
    pub height: Unit,
    pub min_width: Option<Unit>,
    pub min_height: Option<Unit>,
    pub max_width: Option<Unit>,
    pub max_height: Option<Unit>,
    pub layout_dir: Direction,
    pub hori_align: Alignment,
    pub vert_align: Alignment,
    pub hori_overflow: Overflow,
    pub vert_overflow: Overflow,
    pub widget: Option<RefCell<Box<Widget + 'a>>>,
    pub background_color: Option<Color>,
    pub scroll: Cell<Point>,
}

impl<'a> SpaceDiv<'a> {
    pub fn full() -> SpaceDivBuilder<'a> {
        SpaceDivBuilder::new()
            .width(Unit::Relative(1.0))
            .height(Unit::Relative(1.0))
    }

    /// Compute the layout of the children divs.
    pub fn children<'b, 'c>(
        &'b self,
        arena: &'b it::Arena<SpaceDiv<'c>>,
        self_id: it::NodeId,
        self_bbox: Bbox,
    ) -> div::SpaceDivIter<'b, 'c, it::Children<SpaceDiv<'c>>> {
        let mut total_size = Size::new(0, 0);
        let mut max_size = Size::new(0, 0);
        self_id.children(arena).for_each(|div| {
            let div = &arena[div].data;
            let div_size = div.size_pixels(self_bbox.size(), Direction::Vertical, Point::new(0, 0));
            total_size += div_size;
            max_size = point_max(max_size, div_size);
        });

        div::SpaceDivIter::new(
            arena,
            self_id.children(arena),
            total_size,
            max_size,
            self_bbox,
            self.scroll.get(),
            self.layout_dir,
            self.hori_align,
            self.vert_align,
            self.hori_overflow,
            self.vert_overflow,
        )
    }

    /// Compute this div's size in pixels.
    pub fn size_pixels(
        &self,
        parent_size: Size,
        parent_dir: Direction,
        last_origin: Point,
    ) -> Size {
        let calc_data = UnitCalcData {
            div: &self,
            direction: self.layout_dir,
            parent_size: parent_size,
            remaining: match parent_dir {
                Direction::Horizontal => max(parent_size.x - last_origin.x, 0),
                Direction::Vertical => max(parent_size.y - last_origin.y, 0),
            },
        };

        let w = match self.width {
            Unit::Pixels(pix) => pix,
            Unit::Relative(per) => (parent_size.x as f32 * per) as i32,
            Unit::Calc(ref f) => (*f)(calc_data),
        };
        let h = match self.height {
            Unit::Pixels(pix) => pix,
            Unit::Relative(per) => (parent_size.y as f32 * per) as i32,
            Unit::Calc(ref f) => (*f)(calc_data),
        };

        // Apply min and max width to width
        let w = if let Some(ref mw) = self.max_width {
            let mw = match *mw {
                Unit::Pixels(pix) => pix,
                Unit::Relative(per) => (parent_size.x as f32 * per) as i32,
                Unit::Calc(ref f) => (*f)(calc_data),
            };
            min(w, mw)
        } else {
            w
        };

        let w = if let Some(ref mw) = self.min_width {
            let mw = match *mw {
                Unit::Pixels(pix) => pix,
                Unit::Relative(per) => (parent_size.x as f32 * per) as i32,
                Unit::Calc(ref f) => (*f)(calc_data),
            };
            max(w, mw)
        } else {
            w
        };

        // Apply min and max height to height
        let h = if let Some(ref mh) = self.max_height {
            let mh = match *mh {
                Unit::Pixels(pix) => pix,
                Unit::Relative(per) => (parent_size.y as f32 * per) as i32,
                Unit::Calc(ref f) => (*f)(calc_data),
            };
            min(h, mh)
        } else {
            h
        };

        let h = if let Some(ref mh) = self.min_height {
            let mh = match *mh {
                Unit::Pixels(pix) => pix,
                Unit::Relative(per) => (parent_size.y as f32 * per) as i32,
                Unit::Calc(ref f) => (*f)(calc_data),
            };
            max(h, mh)
        } else {
            h
        };

        Size::new(w, h)
    }
}

impl<'a> Default for SpaceDiv<'a> {
    fn default() -> Self {
        Self {
            width: Unit::Relative(1.0),
            height: Unit::Relative(1.0),
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            layout_dir: Direction::Horizontal,
            hori_align: Alignment::Min,
            vert_align: Alignment::Min,
            hori_overflow: Overflow::Overflow,
            vert_overflow: Overflow::Overflow,
            widget: None,
            background_color: None,
            scroll: Cell::new(Point::new(0, 0)),
        }
    }
}

pub struct SpaceDivBuilder<'a> {
    current: SpaceDiv<'a>,
}

impl<'a> SpaceDivBuilder<'a> {
    pub fn new() -> Self {
        Self {
            current: Default::default(),
        }
    }

    pub fn width(mut self, width: div::Unit) -> Self {
        self.current.width = width;
        self
    }

    pub fn height(mut self, height: div::Unit) -> Self {
        self.current.height = height;
        self
    }

    pub fn min_width(mut self, min_width: div::Unit) -> Self {
        self.current.min_width = Some(min_width);
        self
    }

    pub fn min_height(mut self, min_height: div::Unit) -> Self {
        self.current.min_height = Some(min_height);
        self
    }

    pub fn max_width(mut self, max_width: div::Unit) -> Self {
        self.current.max_width = Some(max_width);
        self
    }

    pub fn max_height(mut self, max_height: div::Unit) -> Self {
        self.current.max_height = Some(max_height);
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.current.layout_dir = Direction::Horizontal;
        self
    }

    pub fn vertical(mut self) -> Self {
        self.current.layout_dir = Direction::Vertical;
        self
    }

    pub fn hori_align(mut self, hori_align: div::Alignment) -> Self {
        self.current.hori_align = hori_align;
        self
    }

    pub fn vert_align(mut self, vert_align: div::Alignment) -> Self {
        self.current.vert_align = vert_align;
        self
    }

    pub fn hori_overflow(mut self, hori_overflow: div::Overflow) -> Self {
        self.current.hori_overflow = hori_overflow;
        self
    }

    pub fn vert_overflow(mut self, vert_overflow: div::Overflow) -> Self {
        self.current.vert_overflow = vert_overflow;
        self
    }

    pub fn widget(mut self, widget: Box<Widget + 'a>) -> Self {
        self.current.widget = Some(RefCell::new(widget));
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.current.background_color = Some(color);
        self
    }

    pub fn build(self) -> SpaceDiv<'a> {
        self.current
    }
}
