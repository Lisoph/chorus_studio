use gui::*;
use gui::div;

use std::cell::{Ref, RefCell};
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
    /// The horizontal scrollbars which need to be drawn per div.
    pending_hori_scrollbars: RefCell<HashMap<it::NodeId, Bbox>>,
    /// The vertical scrollbars which need to be drawn per div.
    pending_vert_scrollbars: RefCell<HashMap<it::NodeId, Bbox>>,
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
            pending_hori_scrollbars: RefCell::new(HashMap::new()),
            pending_vert_scrollbars: RefCell::new(HashMap::new()),
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
    pub fn space_div(&self, id: it::NodeId) -> &div::SpaceDiv<'a> {
        &self.arena[id].data
    }

    /// Draw the entire frame.
    pub fn draw(&self, frame: &nanovg::Frame) {
        self.pending_hori_scrollbars.borrow_mut().clear();
        self.pending_vert_scrollbars.borrow_mut().clear();

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
        let draw_bbox = |bbox: &Bbox| {
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

        self.pending_hori_scrollbars
            .borrow()
            .values()
            .chain(self.pending_vert_scrollbars.borrow().values())
            .for_each(draw_bbox);
    }

    fn handle_div_scroll(
        &self,
        (mut div_bbox, x, y): (Bbox, div::AxisOverflowBehaviour, div::AxisOverflowBehaviour),
        parent: Option<(it::NodeId, Bbox)>,
    ) -> (Bbox, Bbox) {
        let mut clip = div_bbox;
        if let Some((parent, parent_bbox)) = parent {
            // Handle parent div bbox and clip scroll offset
            let scroll = self.arena[parent].data.scroll.get();
            if x.scroll().is_some() {
                div_bbox.min.x -= scroll.x;
                div_bbox.max.x -= scroll.x;
                clip.min.x = max(clip.min.x - scroll.x, parent_bbox.min.x);
                clip.max.x = min(clip.max.x - scroll.x, parent_bbox.max.x);
            } else if let Some((min_val, max_val)) = x.min_max() {
                clip.min.x = min_val;
                clip.max.x = max_val;
            }
            if y.scroll().is_some() {
                div_bbox.min.y -= scroll.y;
                div_bbox.max.y -= scroll.y;
                clip.min.y = max(clip.min.y - scroll.y, parent_bbox.min.y);
                clip.max.y = min(clip.max.y - scroll.y, parent_bbox.max.y);
            } else if let Some((min_val, max_val)) = y.min_max() {
                clip.min.y = min_val;
                clip.max.y = max_val;
            }

            // Generate scrollbars
            if let Some(sx) = x.scroll() {
                let width = max(parent_bbox.size().x - sx, 8);
                let height = 16;
                let origin = Point::new(parent_bbox.min.x + scroll.x, parent_bbox.max.y - height);
                let size = Point::new(width, height);
                let bbox = Bbox::with_size(origin, size);
                let mut bars = self.pending_hori_scrollbars.borrow_mut();
                let _ = bars.entry(parent).or_insert(bbox);
            }
            if let Some(sy) = y.scroll() {
                let width = 16;
                let height = max(parent_bbox.size().y - sy, 8);
                let origin = Point::new(parent_bbox.max.x - width, parent_bbox.min.y + scroll.y);
                let size = Point::new(width, height);
                let bbox = Bbox::with_size(origin, size);
                let mut bars = self.pending_vert_scrollbars.borrow_mut();
                let _ = bars.entry(parent).or_insert(bbox);
            }
        }

        (div_bbox.normalize(), clip.normalize())
    }

    /// Draw a single div.
    fn draw_div(
        &self,
        div: &div::SpaceDiv,
        div_visibility: div::ComputedVisibility,
        parent: Option<(it::NodeId, Bbox)>,
        frame: &nanovg::Frame,
    ) {
        let (div_bbox, x, y) = match div_visibility {
            div::ComputedVisibility::Invisible => return,
            div::ComputedVisibility::Visible { bbox, x, y } => (bbox, x, y),
        };

        let (div_bbox, clip) = self.handle_div_scroll((div_bbox, x, y), parent);

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
                nanovg::PathOptions {
                    scissor: parent.map(|(_, bbox)| bbox.into()),
                    ..Default::default()
                },
            );
        }

        // Draw the widget
        if let Some(ref widget) = div.widget {
            let mut widget = widget.borrow_mut();
            widget.update();
            widget.draw(div_bbox, clip, frame);
        }
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

            for (c, vis) in self.arena[id].data.children(&self.arena, id, bbox) {
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
