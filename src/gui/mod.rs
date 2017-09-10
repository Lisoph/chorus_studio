pub mod window;
pub mod widgets;
pub mod renderer;

use std::cmp::{max, min};

use nalgebra;

use self::renderer::Painting;
use self::widgets::Widget;

pub type Point = nalgebra::Vector2<i32>;
pub type Size = nalgebra::Vector2<i32>;

#[derive(Clone, Copy, Debug)]
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
}

pub struct View {
    bbox: Bbox,
    space_div: SpaceDiv,
}

impl View {
    pub fn new(bbox: Bbox) -> Self {
        Self {
            bbox,
            space_div: SpaceDiv::full()
                .vertical()
                .vert_align(DivAlignment::Min)
                .build(),
        }
    }

    pub fn without_bbox() -> Self {
        let zero = Point::new(0, 0);
        Self::new(Bbox::new(zero, zero))
    }

    pub fn add_div(&mut self, div: SpaceDiv) {
        self.space_div.add_div(div);
    }

    pub fn bbox(&self) -> &Bbox {
        &self.bbox
    }

    pub fn set_bbox(&mut self, bbox: Bbox) {
        self.bbox = bbox;
    }

    /// Returns the root space div of this view.
    pub fn space_div(&self) -> &SpaceDiv {
        &self.space_div
    }

    pub fn draw(&self, painting: &mut Painting) {
        self.draw_div(&self.space_div, self.bbox, painting);
    }

    fn draw_div(&self, div: &SpaceDiv, div_bbox: Bbox, painting: &mut Painting) {
        if let Some(ref widget) = div.widget {
            widget.draw(div_bbox, painting);
        }

        for (div, bbox) in div.children(div_bbox) {
            self.draw_div(div, bbox, painting);
        }
    }
}

#[derive(Clone, Copy)]
pub struct DivUnitCalcData<'a> {
    pub div: &'a SpaceDiv,
    pub direction: DivDirection,
    pub parent_size: Size,
    pub remaining: i32,
}

pub enum DivUnit {
    /// Absolute pixels.
    Pixels(i32),
    /// Pixels relative to a parent.
    Relative(f32),
    /// Calculate the pixels dynamically through a closure.
    Calc(Box<Fn(DivUnitCalcData) -> i32>),
}

#[derive(Clone, Copy)]
pub enum DivDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy)]
pub enum DivAlignment {
    Min,
    Max,
    Center,
}

pub struct SpaceDiv {
    width: DivUnit,
    height: DivUnit,
    min_width: Option<DivUnit>,
    min_height: Option<DivUnit>,
    max_width: Option<DivUnit>,
    max_height: Option<DivUnit>,
    layout_dir: DivDirection,
    hori_align: DivAlignment,
    vert_align: DivAlignment,
    widget: Option<Box<Widget>>,
    child_divs: Vec<SpaceDiv>,
}

impl SpaceDiv {
    pub fn full() -> SpaceDivBuilder {
        SpaceDivBuilder::new()
            .width(DivUnit::Relative(1.0))
            .height(DivUnit::Relative(1.0))
    }

    // Compute the layout of the children.
    pub fn children(&self, self_bbox: Bbox) -> SpaceDivIter<::std::slice::Iter<SpaceDiv>> {
        let total_size = self.child_divs.iter().fold(Size::new(0, 0), |total, div| {
            total + div.size_pixels(self_bbox.size(), DivDirection::Vertical, Point::new(0, 0))
        });
        SpaceDivIter::new(
            self.child_divs.iter(),
            total_size,
            self_bbox,
            self.layout_dir,
            self.hori_align,
            self.vert_align,
        )
    }

    pub fn add_div(&mut self, div: SpaceDiv) {
        self.child_divs.push(div);
    }

    /// Compute this div's size in pixels.
    fn size_pixels(&self, parent_size: Size, parent_dir: DivDirection, last_origin: Point) -> Size {
        let calc_data = DivUnitCalcData {
            div: &self,
            direction: self.layout_dir,
            parent_size: parent_size,
            remaining: match parent_dir {
                DivDirection::Horizontal => max(parent_size.x - last_origin.x, 0),
                DivDirection::Vertical => max(parent_size.y - last_origin.y, 0),
            },
        };

        let w = match self.width {
            DivUnit::Pixels(pix) => pix,
            DivUnit::Relative(per) => (parent_size.x as f32 * per) as i32,
            DivUnit::Calc(ref f) => (*f)(calc_data),
        };
        let h = match self.height {
            DivUnit::Pixels(pix) => pix,
            DivUnit::Relative(per) => (parent_size.y as f32 * per) as i32,
            DivUnit::Calc(ref f) => (*f)(calc_data),
        };

        // Apply min and max width to width
        let w = if let Some(ref mw) = self.max_width {
            let mw = match *mw {
                DivUnit::Pixels(pix) => pix,
                DivUnit::Relative(per) => (parent_size.x as f32 * per) as i32,
                DivUnit::Calc(ref f) => (*f)(calc_data),
            };
            min(w, mw)
        } else {
            w
        };

        let w = if let Some(ref mw) = self.min_width {
            let mw = match *mw {
                DivUnit::Pixels(pix) => pix,
                DivUnit::Relative(per) => (parent_size.x as f32 * per) as i32,
                DivUnit::Calc(ref f) => (*f)(calc_data),
            };
            max(w, mw)
        } else {
            w
        };

        // Apply min and max height to height
        let h = if let Some(ref mh) = self.max_height {
            let mh = match *mh {
                DivUnit::Pixels(pix) => pix,
                DivUnit::Relative(per) => (parent_size.y as f32 * per) as i32,
                DivUnit::Calc(ref f) => (*f)(calc_data),
            };
            min(h, mh)
        } else {
            h
        };

        let h = if let Some(ref mh) = self.min_height {
            let mh = match *mh {
                DivUnit::Pixels(pix) => pix,
                DivUnit::Relative(per) => (parent_size.y as f32 * per) as i32,
                DivUnit::Calc(ref f) => (*f)(calc_data),
            };
            max(h, mh)
        } else {
            h
        };

        Size::new(w, h)
    }
}

impl Default for SpaceDiv {
    fn default() -> Self {
        Self {
            width: DivUnit::Relative(1.0),
            height: DivUnit::Relative(1.0),
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            layout_dir: DivDirection::Horizontal,
            hori_align: DivAlignment::Min,
            vert_align: DivAlignment::Min,
            widget: None,
            child_divs: Vec::new(),
        }
    }
}

/// Iterator that computes the positions and sizes of space divisions based on the
/// parent or containing bounding box.
/// This is essentially the layout engine.
pub struct SpaceDivIter<'a, I>
where
    I: Iterator<Item = &'a SpaceDiv>,
{
    /// The space divisions to compute the layout of.
    space_divs: I,
    /// The sum total of all the space division sizes.
    total_size: Size,
    /// The containing (parent) bounding box.
    bbox: Bbox,
    /// The direction in with the layout grows.
    dir: DivDirection,
    /// The horizontal alignment.
    hori_align: DivAlignment,
    /// The vertical alignment.
    vert_align: DivAlignment,
    /// The current horizontal or vertical advance.
    cur: i32,
    /// The bottom-right coordinate of the last / previous division's bounding box.
    previous_end: Point,
}

impl<'a, I> SpaceDivIter<'a, I>
where
    I: Iterator<Item = &'a SpaceDiv>,
{
    fn new(
        space_divs: I,
        total_size: Size,
        bbox: Bbox,
        dir: DivDirection,
        hori_align: DivAlignment,
        vert_align: DivAlignment,
    ) -> Self {
        Self {
            space_divs,
            total_size,
            bbox,
            dir,
            hori_align,
            vert_align,
            cur: 0,
            previous_end: Point::new(0, 0),
        }
    }
}

impl<'a, I> Iterator for SpaceDivIter<'a, I>
where
    I: Iterator<Item = &'a SpaceDiv>,
{
    type Item = (&'a SpaceDiv, Bbox);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(div) = self.space_divs.next() {
            let size = div.size_pixels(self.bbox.size(), self.dir, self.previous_end);

            // Compute offset coordinates from top-left.
            let offset_x = match self.hori_align {
                DivAlignment::Min => 0,
                DivAlignment::Max => self.bbox.size().x - size.x,
                DivAlignment::Center => match self.dir {
                    DivDirection::Horizontal => self.bbox.size().x / 2 - self.total_size.x / 2,
                    DivDirection::Vertical => self.bbox.size().x / 2 - size.x / 2,
                },
            };
            let offset_y = match self.vert_align {
                DivAlignment::Min => 0,
                DivAlignment::Max => self.bbox.size().y - size.y,
                DivAlignment::Center => match self.dir {
                    DivDirection::Horizontal => self.bbox.size().y / 2 - size.y / 2,
                    DivDirection::Vertical => self.bbox.size().y / 2 - self.total_size.y / 2,
                },
            };

            let origin = match self.dir {
                DivDirection::Horizontal => {
                    Point::new(self.cur + offset_x, self.bbox.min.y + offset_y)
                }
                DivDirection::Vertical => {
                    Point::new(self.bbox.min.x + offset_x, self.cur + offset_y)
                }
            };
            self.previous_end = origin + size;

            let div_bbox = Bbox::with_size(origin, size);

            // Compute directions and advance.
            let dir_x = match self.hori_align {
                DivAlignment::Max => -1,
                _ => 1,
            };
            let dir_y = match self.vert_align {
                DivAlignment::Max => -1,
                _ => 1,
            };

            self.cur += match self.dir {
                DivDirection::Horizontal => size.x * dir_x,
                DivDirection::Vertical => size.y * dir_y,
            };

            Some((div, div_bbox))
        } else {
            None
        }
    }
}

pub struct SpaceDivBuilder {
    current: SpaceDiv,
}

impl SpaceDivBuilder {
    pub fn new() -> Self {
        Self {
            current: Default::default(),
        }
    }

    pub fn width(mut self, width: DivUnit) -> Self {
        self.current.width = width;
        self
    }

    pub fn height(mut self, height: DivUnit) -> Self {
        self.current.height = height;
        self
    }

    pub fn min_width(mut self, min_width: DivUnit) -> Self {
        self.current.min_width = Some(min_width);
        self
    }

    pub fn min_height(mut self, min_height: DivUnit) -> Self {
        self.current.min_height = Some(min_height);
        self
    }

    pub fn max_width(mut self, max_width: DivUnit) -> Self {
        self.current.max_width = Some(max_width);
        self
    }

    pub fn max_height(mut self, max_height: DivUnit) -> Self {
        self.current.max_height = Some(max_height);
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.current.layout_dir = DivDirection::Horizontal;
        self
    }

    pub fn vertical(mut self) -> Self {
        self.current.layout_dir = DivDirection::Vertical;
        self
    }

    pub fn hori_align(mut self, hori_align: DivAlignment) -> Self {
        self.current.hori_align = hori_align;
        self
    }

    pub fn vert_align(mut self, vert_align: DivAlignment) -> Self {
        self.current.vert_align = vert_align;
        self
    }

    pub fn widget(mut self, widget: Box<Widget>) -> Self {
        self.current.widget = Some(widget);
        self
    }

    pub fn add_div(mut self, div: SpaceDiv) -> Self {
        self.current.add_div(div);
        self
    }

    pub fn build(self) -> SpaceDiv {
        self.current
    }
}
