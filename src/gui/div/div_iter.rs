use gui::*;
use gui::div;

/// Iterator that computes the positions and sizes of space divisions based on the
/// parent or containing bounding box.
/// This is essentially the layout engine.
pub struct SpaceDivIter<'a, 'b: 'a, I>
where
    I: Iterator<Item = it::NodeId>,
{
    /// The arena to which the NodeIds refer.
    arena: &'a it::Arena<div::SpaceDiv<'b>>,
    /// The space divisions to compute the layout of.
    space_divs: I,
    /// The sum total of all the space division sizes.
    total_size: Size,
    /// The largest size of all space divisions.
    /// Basically each div's size `max`d, per axis.
    max_size: Size,
    /// The containing (parent) bounding box.
    bbox: Bbox,
    /// The scroll values of the parent bounding box.
    scroll: Point,
    /// The direction in with the layout grows.
    dir: div::Direction,
    /// The horizontal alignment.
    hori_align: div::Alignment,
    /// The vertical alignment.
    vert_align: div::Alignment,
    /// The horizontal overflow.
    hori_overflow: div::Overflow,
    /// The vertical overflow
    vert_overflow: div::Overflow,
    /// The current horizontal or vertical advance.
    cur: i32,
    /// The bottom-right coordinate of the last / previous division's bounding box.
    previous_end: Point,
}

impl<'a, 'b: 'a, I> SpaceDivIter<'a, 'b, I>
where
    I: Iterator<Item = it::NodeId>,
{
    pub fn new(
        arena: &'a it::Arena<div::SpaceDiv<'b>>,
        space_divs: I,
        total_size: Size,
        max_size: Size,
        bbox: Bbox,
        scroll: Point,
        dir: div::Direction,
        hori_align: div::Alignment,
        vert_align: div::Alignment,
        hori_overflow: div::Overflow,
        vert_overflow: div::Overflow,
    ) -> Self {
        Self {
            arena,
            space_divs,
            total_size,
            max_size,
            bbox,
            scroll,
            dir,
            hori_align,
            vert_align,
            hori_overflow,
            vert_overflow,
            cur: 0,
            previous_end: Point::new(0, 0),
        }
    }

    /// Compute visibility, overflow and scroll information for a div bbox.
    fn div_visibility(&self, div_bbox: Bbox) -> div::ComputedVisibility {
        if (self.hori_overflow != div::Overflow::Overflow
            || self.vert_overflow != div::Overflow::Overflow)
            && !self.bbox.contains_bbox(div_bbox/*.offset(-self.scroll)*/)
        {
            return div::ComputedVisibility::Invisible;
        }

        let do_axis = |overflow: div::Overflow,
                       (bbox_min, bbox_max): (i32, i32),
                       (parent_bbox_min, parent_bbox_max): (i32, i32),
                       max_size: i32, clip_parent_scroll: i32|
         -> div::AxisOverflowBehaviour {
            let min_val = max(bbox_min, parent_bbox_min);
            let max_val = min(bbox_max, parent_bbox_max);
            match overflow {
                div::Overflow::Clip => div::AxisOverflowBehaviour::Clip {
                    min: min_val,
                    max: max_val,
                },
                div::Overflow::Scroll => {
                    let size = parent_bbox_max - parent_bbox_min;
                    let scroll = max(max_size - size, 0);
                    if scroll > 0 {
                        div::AxisOverflowBehaviour::Scroll {
                            min: max(min_val - clip_parent_scroll, parent_bbox_min),
                            max: min(max_val - clip_parent_scroll, parent_bbox_max),
                            scroll,
                        }
                    } else {
                        div::AxisOverflowBehaviour::Clip {
                            min: min_val,
                            max: max_val,
                        }
                    }
                }
                div::Overflow::Overflow => div::AxisOverflowBehaviour::Overflow,
            }
        };

        let x = do_axis(
            self.hori_overflow,
            (div_bbox.min.x, div_bbox.max.x),
            (self.bbox.min.x, self.bbox.max.x),
            match self.dir {
                div::Direction::Horizontal => self.total_size.x,
                div::Direction::Vertical => self.max_size.x,
            },
            if self.dir == div::Direction::Horizontal { self.scroll.x } else { 0 },
        );
        let y = do_axis(
            self.vert_overflow,
            (div_bbox.min.y, div_bbox.max.y),
            (self.bbox.min.y, self.bbox.max.y),
            match self.dir {
                div::Direction::Horizontal => self.max_size.y,
                div::Direction::Vertical => self.total_size.y,
            },
            if self.dir == div::Direction::Vertical { self.scroll.y } else { 0 },
        );

        div::ComputedVisibility::Visible {
            bbox: div_bbox,
            x,
            y,
        }
    }
}

impl<'a, 'b: 'a, I> Iterator for SpaceDivIter<'a, 'b, I>
where
    I: Iterator<Item = it::NodeId>,
{
    type Item = (it::NodeId, div::ComputedVisibility);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(div_id) = self.space_divs.next() {
            let div = &self.arena[div_id].data;
            let size = div.size_pixels(self.bbox.size(), self.dir, self.previous_end);

            // Compute offset coordinates from top-left.
            let offset_x = match self.hori_align {
                div::Alignment::Min => 0,
                div::Alignment::Max => self.bbox.size().x - size.x,
                div::Alignment::Center => match self.dir {
                    div::Direction::Horizontal => self.bbox.size().x / 2 - self.total_size.x / 2,
                    div::Direction::Vertical => self.bbox.size().x / 2 - size.x / 2,
                },
            };
            let offset_y = match self.vert_align {
                div::Alignment::Min => 0,
                div::Alignment::Max => self.bbox.size().y - size.y,
                div::Alignment::Center => match self.dir {
                    div::Direction::Horizontal => self.bbox.size().y / 2 - size.y / 2,
                    div::Direction::Vertical => self.bbox.size().y / 2 - self.total_size.y / 2,
                },
            };

            let origin = match self.dir {
                div::Direction::Horizontal => Point::new(
                    self.bbox.min.x + self.cur + offset_x,
                    self.bbox.min.y + offset_y,
                ),
                div::Direction::Vertical => Point::new(
                    self.bbox.min.x + offset_x,
                    self.bbox.min.y + self.cur + offset_y,
                ),
            };
            self.previous_end = origin + size;

            // Div bbox, clip bbox and scroll amounts.
            let div_bbox = Bbox::with_size(origin, size);
            let visibility = self.div_visibility(div_bbox);

            // Compute directions and advance.
            let dir_x = match self.hori_align {
                div::Alignment::Max => -1,
                _ => 1,
            };
            let dir_y = match self.vert_align {
                div::Alignment::Max => -1,
                _ => 1,
            };

            self.cur += match self.dir {
                div::Direction::Horizontal => size.x * dir_x,
                div::Direction::Vertical => size.y * dir_y,
            };

            Some((div_id, visibility))
        } else {
            None
        }
    }
}
