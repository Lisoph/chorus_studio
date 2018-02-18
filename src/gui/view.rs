use gui::*;
use gui::div;

use std::cell::{Ref, RefCell};
use std::time::Instant;
use std::collections::HashMap;

struct CachedDiv {
    /// Node id of the cached div.
    id: it::NodeId,
    /// Visibility of the cached div.
    visibility: div::ComputedVisibility,
    /// The parent id and bbox of the cached div.
    parent: Option<(it::NodeId, Bbox)>,
}

pub struct View<'a> {
    bbox: Bbox,
    arena: it::Arena<div::SpaceDiv<'a>>,
    root_div: it::NodeId,
    children: Vec<it::NodeId>,
    cache: RefCell<Option<Vec<CachedDiv>>>,
    time_start: Instant,
    /// The scrollbars which need to be drawn per div.
    pending_scrollbars: RefCell<HashMap<it::NodeId, (Option<Bbox>, Option<Bbox>)>>,
}

impl<'a> View<'a> {
    pub fn new(bbox: Bbox) -> Self {
        let mut arena = it::Arena::new();
        let root_div = arena.new_node(
            div::SpaceDiv::full()
                .vertical()
                .vert_align(div::Alignment::Min)
                .build(),
        );
        Self {
            bbox,
            arena,
            root_div,
            children: Vec::new(),
            cache: RefCell::new(None),
            time_start: Instant::now(),
            pending_scrollbars: RefCell::new(HashMap::new()),
        }
    }

    pub fn without_bbox() -> Self {
        let zero = Point::new(0, 0);
        Self::new(Bbox::new(zero, zero))
    }

    pub fn add_div(&mut self, parent: Option<it::NodeId>, div: div::SpaceDiv<'a>) -> it::NodeId {
        let node = self.arena.new_node(div);
        parent
            .unwrap_or(self.root_div)
            .append(node, &mut self.arena);
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
    pub fn space_div(&self) -> &div::SpaceDiv<'a> {
        &self.arena[self.root_div].data
    }

    /// Draw the entire frame.
    pub fn draw(&self, frame: &nanovg::Frame) {
        self.pending_scrollbars.borrow_mut().clear();

        if let Some(ref divs) = *self.divs() {
            for cached_div in divs.iter() {
                self.draw_div(
                    &self.arena[cached_div.id].data,
                    cached_div.visibility,
                    cached_div.parent,
                    frame,
                );
            }
        }

        self.draw_scrollbars(frame);
    }

    fn draw_scrollbars(&self, frame: &nanovg::Frame) {
        let draw_bbox = |bbox: Bbox| {
            frame.path(
                |path| {
                    let origin = (bbox.min.x as f32, bbox.min.y as f32);
                    let size = (bbox.size().x as f32, bbox.size().y as f32);
                    path.rect(origin, size);
                    path.fill(nanovg::FillStyle {
                        coloring_style: nanovg::ColoringStyle::Color(
                            Color::rgba(0.0, 1.0, 0.0, 0.4).into(),
                        ),
                        ..Default::default()
                    });
                },
                Default::default(),
            );
        };

        for &(hori_bbox, vert_bbox) in self.pending_scrollbars.borrow().values() {
            if let Some(hori_bbox) = hori_bbox {
                draw_bbox(hori_bbox);
            }
            if let Some(vert_bbox) = vert_bbox {
                draw_bbox(vert_bbox);
            }
        }
    }

    /// Draw a single div.
    fn draw_div(
        &self,
        div: &div::SpaceDiv,
        div_visibility: div::ComputedVisibility,
        parent_bbox: Option<(it::NodeId, Bbox)>,
        frame: &nanovg::Frame,
    ) {
        let (div_bbox, x, y) = match div_visibility {
            div::ComputedVisibility::Invisible => return,
            div::ComputedVisibility::Visible { bbox, x, y } => (bbox, x, y),
        };

        // Build clip bbox
        let clip = {
            let mut clip = div_bbox;
            if let Some((min_val, max_val)) = x.min_max() {
                clip.min.x = min_val;
                clip.max.x = max_val;
            }
            if let Some((min_val, max_val)) = y.min_max() {
                clip.min.y = min_val;
                clip.max.y = max_val;
            }
            clip
        };

        // Draw background color if we have one
        if let Some(color) = div.background_color {
            frame.path(
                |path| {
                    let origin = (clip.min.x as f32, clip.min.y as f32);
                    let size = (clip.size().x as f32, clip.size().y as f32);
                    path.rect(origin, size);
                    path.fill(nanovg::FillStyle {
                        coloring_style: nanovg::ColoringStyle::Color(color.into()),
                        ..Default::default()
                    });
                },
                Default::default(),
            );
        }

        // Generate scrollbars
        if let Some((parent, parent_bbox)) = parent_bbox {
            if let Some(sx) = x.scroll() {
                let width = max(parent_bbox.size().x - sx, 8);
                let height = 16;
                let origin = Point::new(
                    parent_bbox.min.x + div.scroll.get().x,
                    parent_bbox.max.y - height,
                );
                let size = Point::new(width, height);
                let bbox = Bbox::with_size(origin, size);
                let mut bars = self.pending_scrollbars.borrow_mut();
                bars.entry(parent).or_insert((None, None)).0 = Some(bbox);
            }
            if let Some(sy) = y.scroll() {
                let width = 16;
                let height = max(parent_bbox.size().y - sy, 8);
                let origin = Point::new(
                    parent_bbox.max.x - width,
                    parent_bbox.min.y + div.scroll.get().y,
                );
                let size = Point::new(width, height);
                let bbox = Bbox::with_size(origin, size);
                let mut bars = self.pending_scrollbars.borrow_mut();
                bars.entry(parent).or_insert((None, None)).1 = Some(bbox);
            }
        }

        let scrolled_div_bbox = {
            let mut bbox = div_bbox;
            let scroll = div.scroll.get();
            if let div::AxisOverflowBehaviour::Scroll { .. } = x {
                bbox.min.x -= scroll.x;
                bbox.max.x -= scroll.x;
            }
            if let div::AxisOverflowBehaviour::Scroll { .. } = y {
                bbox.min.y -= scroll.y;
                bbox.max.y -= scroll.y;
            }
            bbox
        };

        // Draw the widget
        if let Some(ref widget) = div.widget {
            let mut widget = widget.borrow_mut();
            widget.update();
            widget.draw(scrolled_div_bbox, clip, frame);
        }

        let max_scroll = {
            let x = x.scroll().unwrap_or(0);
            let y = y.scroll().unwrap_or(0);
            Point::new(x, y)
        };

        // Scroll right 1 pixel per frame for testing the scrolling.
        let delta = self.time_start.elapsed();
        let delta = (delta.as_secs() * 1000 + delta.subsec_millis() as u64) as f32 / 1000.0;
        let delta = ((delta.sin() * 0.5 + 0.5) * max_scroll.x as f32) as i32;
        div.scroll.set(point_min(Point::new(delta, 0), max_scroll));
    }

    /// Recursively visit all space divs of this view.
    fn visit_divs<F>(
        &self,
        id: it::NodeId,
        visibility: div::ComputedVisibility,
        parent: Option<(it::NodeId, Bbox)>,
        visitor: &mut F,
    ) where
        F: FnMut(CachedDiv),
    {
        if let div::ComputedVisibility::Visible { bbox, .. } = visibility {
            visitor(CachedDiv {
                id,
                visibility,
                parent,
            });

            let visible_children = self.arena[id].data.children(&self.arena, id, bbox);

            for (c, vis) in visible_children {
                self.visit_divs(c, vis, Some((id, bbox)), visitor);
            }
        }
    }

    fn divs(&self) -> Ref<Option<Vec<CachedDiv>>> {
        let is_none = { self.cache.borrow().is_none() };
        if is_none {
            // Build the cache
            let mut vec = self.cache.borrow_mut();
            let vec = vec.get_or_insert_with(|| Vec::with_capacity(32));
            vec.clear();
            self.visit_divs(
                self.root_div,
                div::ComputedVisibility::Visible {
                    bbox: self.bbox,
                    x: div::AxisOverflowBehaviour::Overflow,
                    y: div::AxisOverflowBehaviour::Overflow,
                },
                None,
                &mut |cached_div| vec.push(cached_div),
            );
        }
        self.cache.borrow()
    }
}
