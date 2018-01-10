pub mod main_window;
pub mod widgets;

use self::widgets::Widget;

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
}

pub struct View<'a> {
    bbox: Bbox,
    arena: it::Arena<SpaceDiv<'a>>,
    root_div: it::NodeId,
    children: Vec<it::NodeId>,
    cache: RefCell<Option<Vec<(it::NodeId, Bbox)>>>,
}

impl<'a> View<'a> {
    pub fn new(bbox: Bbox) -> Self {
        let mut arena = it::Arena::new();
        let root_div = arena.new_node(SpaceDiv::full()
            .vertical()
            .vert_align(DivAlignment::Min)
            .build());
        Self {
            bbox,
            arena,
            root_div,
            children: Vec::new(),
            cache: RefCell::new(None),
        }
    }

    pub fn without_bbox() -> Self {
        let zero = Point::new(0, 0);
        Self::new(Bbox::new(zero, zero))
    }

    pub fn add_div(&mut self, parent: Option<it::NodeId>, div: SpaceDiv<'a>) -> it::NodeId {
        let node = self.arena.new_node(div);
        parent.unwrap_or(self.root_div).append(node, &mut self.arena);
        self.children.push(node);
        *self.cache.borrow_mut() = None;
        node
    }

    pub fn bbox(&self) -> &Bbox {
        &self.bbox
    }

    pub fn set_bbox(&mut self, bbox: Bbox) {
        if bbox != self.bbox {
            *self.cache.borrow_mut() = None;
        }

        self.bbox = bbox;
    }

    /// Returns the root space div of this view.
    pub fn space_div(&self) -> &SpaceDiv<'a> {
        &self.arena[self.root_div].data
    }

    pub fn draw(&self, frame: &nanovg::Frame) {
        if let Some(ref divs) = *self.divs() {
            for &(div, bbox) in divs.iter() {
                self.draw_div(&self.arena[div].data, bbox, frame);
            }
        }
    }

    fn draw_div(&self, div: &SpaceDiv, div_bbox: Bbox, frame: &nanovg::Frame) {
        if let Some(color) = div.background_color {
            frame.path(|path| {
                let origin = (div_bbox.min.x as f32, div_bbox.min.y as f32);
                let size = (div_bbox.size().x as f32, div_bbox.size().y as f32);
                path.rect(origin, size);
                path.fill(nanovg::FillStyle {
                    coloring_style: nanovg::ColoringStyle::Color(color.into()),
                    ..Default::default()
                });
            }, Default::default());
        }

        if let Some(ref widget) = div.widget {
            widget.draw(div_bbox, frame);
        }
    }

    /// Recursively visit all space divs of this view.
    fn visit_divs<F: FnMut(it::NodeId, Bbox)>(&self, id: it::NodeId, bbox: Bbox, visitor: &mut F) {
        visitor(id, bbox);
        for (c, bbox) in self.arena[id].data.children(&self.arena, id, bbox) {
            self.visit_divs(c, bbox, visitor);
        }
    }

    fn build_cache(&self) {
        let mut vec = self.cache.borrow_mut();
        let mut vec = vec.get_or_insert_with(|| Vec::with_capacity(64));
        vec.clear();
        self.visit_divs(self.root_div, self.bbox, &mut |div, bbox| vec.push((div, bbox)));
    }

    fn divs(&self) -> Ref<Option<Vec<(it::NodeId, Bbox)>>> {
        let is_none = { self.cache.borrow().is_none() };
        if is_none {
            self.build_cache();
        }
        self.cache.borrow()
    }
}

#[derive(Clone, Copy)]
pub struct DivUnitCalcData<'a> {
    pub div: &'a SpaceDiv<'a>,
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DivDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DivAlignment {
    Min,
    Max,
    Center,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DivOverflow {
    /// Nothing happens, the content just overflows.
    Overflow,
    /// The content gets cut off where it overflows.
    Clip,
    /// The content is scrollable.
    Scroll,
}

pub struct SpaceDiv<'a> {
    width: DivUnit,
    height: DivUnit,
    min_width: Option<DivUnit>,
    min_height: Option<DivUnit>,
    max_width: Option<DivUnit>,
    max_height: Option<DivUnit>,
    layout_dir: DivDirection,
    hori_align: DivAlignment,
    vert_align: DivAlignment,
    hori_overflow: DivOverflow,
    vert_overflow: DivOverflow,
    widget: Option<Box<Widget + 'a>>,
    background_color: Option<Color>,
}

impl<'a> SpaceDiv<'a> {
    pub fn full() -> SpaceDivBuilder<'a> {
        SpaceDivBuilder::new()
            .width(DivUnit::Relative(1.0))
            .height(DivUnit::Relative(1.0))
    }

    /// Compute the layout of the children divs.
    fn children(&self, arena: &'a it::Arena<SpaceDiv<'a>>, self_id: it::NodeId, self_bbox: Bbox) -> SpaceDivIter<it::Children<'a, SpaceDiv<'a>>> {
        let total_size = self_id.children(arena).fold(Size::new(0, 0), |total, div| {
            let div = &arena[div].data;
            total + div.size_pixels(self_bbox.size(), DivDirection::Vertical, Point::new(0, 0))
        });

        SpaceDivIter::new(
            arena,
            self_id.children(arena),
            total_size,
            self_bbox,
            self.layout_dir,
            self.hori_align,
            self.vert_align,
        )
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

impl<'a> Default for SpaceDiv<'a> {
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
            hori_overflow: DivOverflow::Overflow,
            vert_overflow: DivOverflow::Overflow,
            widget: None,
            background_color: None,
        }
    }
}

/// Iterator that computes the positions and sizes of space divisions based on the
/// parent or containing bounding box.
/// This is essentially the layout engine.
pub struct SpaceDivIter<'a, I>
    where
        I: Iterator<Item=it::NodeId>,
{
    /// The arena to which the NodeIds refer.
    arena: &'a it::Arena<SpaceDiv<'a>>,
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
        I: Iterator<Item=it::NodeId>,
{
    fn new(
        arena: &'a it::Arena<SpaceDiv<'a>>,
        space_divs: I,
        total_size: Size,
        bbox: Bbox,
        dir: DivDirection,
        hori_align: DivAlignment,
        vert_align: DivAlignment,
    ) -> Self {
        Self {
            arena,
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
        I: Iterator<Item=it::NodeId>,
{
    type Item = (it::NodeId, Bbox);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(div_id) = self.space_divs.next() {
            let div = &self.arena[div_id].data;
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
                    Point::new(self.bbox.min.x + self.cur + offset_x, self.bbox.min.y + offset_y)
                }
                DivDirection::Vertical => {
                    Point::new(self.bbox.min.x + offset_x, self.bbox.min.y + self.cur + offset_y)
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

            Some((div_id, div_bbox))
        } else {
            None
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

    pub fn hori_overflow(mut self, hori_overflow: DivOverflow) -> Self {
        self.current.hori_overflow = hori_overflow;
        self
    }

    pub fn vert_overflow(mut self, vert_overflow: DivOverflow) -> Self {
        self.current.vert_overflow = vert_overflow;
        self
    }

    pub fn widget(mut self, widget: Box<Widget + 'a>) -> Self {
        self.current.widget = Some(widget);
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
