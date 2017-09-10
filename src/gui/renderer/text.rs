use super::*;

use gui::Bbox;
use asset_storage::FontId;

use rusttype::{Font, Point as GlyphPoint, PositionedGlyph, Scale as GlyphScale, ScaledGlyph};

use glium;

pub struct Text {
    bbox: Bbox,
    font: FontId,
    color: Color,
    scale: GlyphScale,
    /// The string to render. Expected to already be unicode-normalized.
    text: String,
}

impl Text {
    /// Construct a new text render-command.
    /// The `text` is expected to already be unicode-normalized.
    pub fn new(bbox: Bbox, font: FontId, color: Color, scale: GlyphScale, text: String) -> Self {
        Self {
            bbox,
            font,
            color,
            scale,
            text,
        }
    }

    /// Layout the glyphs like a paragraph with wrapping lines.
    fn layout_paragraph<'a>(&'a self, font: &'a Font, dpi_factor: f32) -> Vec<PositionedGlyph<'a>> {
        let mut glyphs = Vec::new();
        for mut word in ParagraphGlyphs::new(&self, &font, dpi_factor) {
            glyphs.append(&mut word);
        }
        glyphs
    }
}

impl DrawCommand for Text {
    fn draw(&self, renderer: &mut Renderer, frame: &mut Frame) {
        if let Some(font) = renderer.storage.borrow().font(self.font) {
            let glyphs = self.layout_paragraph(font, renderer.dpi_factor());
            for glyph in glyphs.iter() {
                renderer
                    .glyph_cache
                    .queue_glyph(self.font.font_id(), glyph.clone());
            }

            let vertices = {
                let glyph_cache = &mut renderer.glyph_cache;
                let texture = &mut renderer.glyph_cache_texture;

                glyph_cache
                    .cache_queued(|rect, data| {
                        texture.main_level().write(
                            glium::Rect {
                                left: rect.min.x,
                                bottom: rect.min.y,
                                width: rect.width(),
                                height: rect.height(),
                            },
                            texture::RawImage2d {
                                data: Cow::Borrowed(data),
                                width: rect.width(),
                                height: rect.height(),
                                format: texture::ClientFormat::U8,
                            },
                        );
                    })
                    .unwrap();

                let vertices = glyphs
                    .iter()
                    .map(|g| glyph_cache.rect_for(self.font.font_id(), g))
                    .filter(Result::is_ok)
                    .map(Result::unwrap)
                    .filter(Option::is_some)
                    .map(Option::unwrap)
                    .map(|(tex, pos)| {
                        [
                            TextRectVertex::new(
                                (pos.min.x as f32, pos.max.y as f32),
                                (tex.min.x, tex.max.y),
                                self.color,
                            ),
                            TextRectVertex::new(
                                (pos.min.x as f32, pos.min.y as f32),
                                (tex.min.x, tex.min.y),
                                self.color,
                            ),
                            TextRectVertex::new(
                                (pos.max.x as f32, pos.min.y as f32),
                                (tex.max.x, tex.min.y),
                                self.color,
                            ),
                            TextRectVertex::new(
                                (pos.max.x as f32, pos.min.y as f32),
                                (tex.max.x, tex.min.y),
                                self.color,
                            ),
                            TextRectVertex::new(
                                (pos.max.x as f32, pos.max.y as f32),
                                (tex.max.x, tex.max.y),
                                self.color,
                            ),
                            TextRectVertex::new(
                                (pos.min.x as f32, pos.max.y as f32),
                                (tex.min.x, tex.max.y),
                                self.color,
                            ),
                        ]
                    });

                let vertices = {
                    let mut verts: Vec<TextRectVertex> = Vec::new();
                    for v in vertices {
                        verts.extend(&v);
                    }
                    verts
                };
                vertices
            };

            let vertices = VertexBuffer::new(renderer.window.display(), &vertices).unwrap();
            let origin: na::Vector2<f32> = na::convert(self.bbox.min);
            let origin: [f32; 2] = origin.into();

            let uniforms = uniform! {
                frame_size: renderer.display_size(),
                origin: origin,
                tex: renderer.glyph_cache_texture.sampled(),
            };

            frame
                .draw(
                    &vertices,
                    &NoIndices(PrimitiveType::TrianglesList),
                    &renderer.program_rect_text,
                    &uniforms,
                    &DrawParameters {
                        blend: Blend::alpha_blending(),
                        scissor: Some(glium::Rect {
                            left: self.bbox.min.x as u32,
                            bottom: renderer.display_size().1 as u32 - self.bbox.max.y as u32,
                            width: self.bbox.size().x as u32,
                            height: self.bbox.size().y as u32,
                        }),
                        ..Default::default()
                    },
                )
                .unwrap();
        }
    }
}

struct ParagraphGlyphs<'a> {
    last_glyph: Option<ScaledGlyph<'a>>,
    caret: GlyphPoint<f32>,
    advance_y: f32,
    dpi_factor: f32,
    word: Vec<PositionedGlyph<'a>>,
    text: &'a Text,
    font: &'a Font<'a>,
    chars: ::std::str::Chars<'a>,
    done: bool,
}

impl<'a> ParagraphGlyphs<'a> {
    fn new(text: &'a Text, font: &'a Font, dpi_factor: f32) -> Self {
        let v_metrics = font.v_metrics(text.scale);
        let advance_y = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

        Self {
            last_glyph: None,
            caret: GlyphPoint {
                x: 0.0,
                y: advance_y,
            },
            advance_y,
            dpi_factor,
            word: Vec::new(),
            text,
            font,
            chars: text.text.chars(),
            done: false,
        }
    }

    /// Checks whether the glyphs in `self.word` are overflowing the bbox,
    /// and repositions them accordingly.
    fn handle_word_wrap(&mut self) {
        if self.caret.x > self.text.bbox.size().x as f32 {
            let indent = { self.word.iter().next().map(|g| g.position().x) };
            if let Some(indent) = indent {
                for g in self.word.iter_mut() {
                    self.caret = g.position() + ::rusttype::Vector::<f32> {
                        x: -indent as f32,
                        y: self.advance_y as f32,
                    };
                    *g = g.clone().into_unpositioned().positioned(self.caret);
                }

                // Re-add the last glyph's advance_width, if there was a last glyph.
                self.caret.x += self.last_glyph
                    .clone()
                    .map(|g| g.h_metrics().advance_width)
                    .unwrap_or(0.0);
            }
        }
    }

    fn scale(&self) -> GlyphScale {
        GlyphScale {
            x: self.text.scale.x * self.dpi_factor,
            y: self.text.scale.y * self.dpi_factor,
        }
    }
}

impl<'a> Iterator for ParagraphGlyphs<'a> {
    type Item = Vec<PositionedGlyph<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let mut glyphs = Vec::new();

        while let Some(c) = self.chars.next() {
            if c.is_control() {
                self.handle_word_wrap();
                match c {
                    '\n' => {
                        self.caret.x = 0.0;
                        self.caret.y += self.advance_y;
                    }
                    _ => {}
                }
                glyphs.append(&mut self.word);
                return Some(glyphs);
            } else {
                if let Some(g) = self.font.glyph(c) {
                    let g = g.scaled(self.scale());
                    self.word.push(g.clone().positioned(self.caret));

                    self.caret.x += g.h_metrics().advance_width;

                    if let Some(ref last_glyph) = self.last_glyph {
                        self.caret.x +=
                            self.font
                                .pair_kerning(self.scale(), g.id(), last_glyph.id());
                    }
                    self.last_glyph = Some(g.clone());

                    if c.is_whitespace() {
                        self.handle_word_wrap();
                        glyphs.append(&mut self.word);
                        return Some(glyphs);
                    }
                }
            }
        }

        self.handle_word_wrap();
        glyphs.append(&mut self.word);
        self.done = true;
        Some(glyphs)
    }
}
