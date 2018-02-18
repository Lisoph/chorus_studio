use gui::*;
use gui::div;

use std::cell::{Ref, RefCell};

pub struct View<'a> {
    bbox: Bbox,
    arena: it::Arena<div::SpaceDiv<'a>>,
    root_div: it::NodeId,
    children: Vec<it::NodeId>,
    cache: RefCell<Option<Vec<(it::NodeId, div::ComputedVisibility)>>>,
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
        if let Some(ref divs) = *self.divs() {
            for &(div, vis) in divs.iter() {
                self.draw_div(&self.arena[div].data, vis, frame);
            }
        }
    }

    /// Draw a single div.
    fn draw_div(
        &self,
        div: &div::SpaceDiv,
        div_visibility: div::ComputedVisibility,
        frame: &nanovg::Frame,
    ) {
        let (div_bbox, x, y) = match div_visibility {
            div::ComputedVisibility::Invisible => return,
            div::ComputedVisibility::Visible { bbox, x, y } => (bbox, x, y),
        };

        // Build clip bbox
        let clip = {
            let mut clip = div_bbox;
            if let div::AxisOverflowBehaviour::Clip {
                min: min_val,
                max: max_val,
            } = x
            {
                clip.min.x = min_val;
                clip.max.x = max_val;
            }
            if let div::AxisOverflowBehaviour::Clip {
                min: min_val,
                max: max_val,
            } = y
            {
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
                nanovg::PathOptions {
                    scissor: Some(clip.into()),
                    ..Default::default()
                },
            );
        }

        // Draw scroll bars if necessary
        if let div::AxisOverflowBehaviour::Scroll(sx) = x {
            frame.path(|path| {
                let origin = ((div_bbox.min.x + div.scroll.x) as f32, div_bbox.min.y as f32);
                let size = (sx as f32, 16.0);
                path.rect(origin, size);
                path.fill(nanovg::FillStyle {
                    coloring_style: nanovg::ColoringStyle::Color(Color::red().into()),
                    .. Default::default()
                });
            }, Default::default());
        }

        // Draw the widget
        if let Some(ref widget) = div.widget {
            let mut widget = widget.borrow_mut();
            widget.update();
            widget.draw(div_bbox, clip, frame);
        }
    }

    /// Recursively visit all space divs of this view.
    fn visit_divs<F>(&self, id: it::NodeId, visibility: div::ComputedVisibility, visitor: &mut F)
    where
        F: FnMut(it::NodeId, div::ComputedVisibility),
    {
        if let div::ComputedVisibility::Visible { bbox, x, y } = visibility {
            visitor(id, visibility);

            let visible_children = self.arena[id].data.children(&self.arena, id, bbox).filter(
                |&(c, vis)| match vis {
                    div::ComputedVisibility::Visible { .. } => true,
                    div::ComputedVisibility::Invisible => false,
                },
            );

            for (c, vis) in visible_children {
                self.visit_divs(c, vis, visitor);
            }
        }
    }

    fn divs(&self) -> Ref<Option<Vec<(it::NodeId, div::ComputedVisibility)>>> {
        let is_none = { self.cache.borrow().is_none() };
        if is_none {
            // Build the cache
            let mut vec = self.cache.borrow_mut();
            let vec = vec.get_or_insert_with(|| Vec::with_capacity(16));
            vec.clear();
            self.visit_divs(
                self.root_div,
                div::ComputedVisibility::Visible {
                    bbox: self.bbox,
                    x: div::AxisOverflowBehaviour::Overflow,
                    y: div::AxisOverflowBehaviour::Overflow,
                },
                &mut |div, vis| vec.push((div, vis)),
            );
        }
        self.cache.borrow()
    }
}
