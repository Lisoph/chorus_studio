use std::borrow::Cow;

use glium;
use glium::{Blend, Display, DrawParameters, Frame, Program, Surface, VertexBuffer};
use glium::texture;
use glium::texture::{RawImage2d, Texture2d};
use glium::index::{NoIndices, PrimitiveType};

use rusttype::{Font, Point as GlyphPoint, PositionedGlyph, Scale as GlyphScale, ScaledGlyph};
use rusttype::gpu_cache::Cache as GlyphCache;

use nalgebra as na;

use gui::Bbox;
use asset_storage::{AssetStorage, FontId, TextureId};

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

#[derive(Clone, Copy)]
struct ColoredRectVertex {
    position: [f32; 2],
}

impl ColoredRectVertex {
    fn new(x: f32, y: f32) -> Self {
        Self { position: [x, y] }
    }
}

implement_vertex!(ColoredRectVertex, position);

#[derive(Clone, Copy)]
struct TexturedRectVertex {
    position: [f32; 2],
    texture_coords: [f32; 2],
}

impl TexturedRectVertex {
    fn new((x, y): (f32, f32), (u, v): (f32, f32)) -> Self {
        Self {
            position: [x, y],
            texture_coords: [u, v],
        }
    }
}

implement_vertex!(TexturedRectVertex, position, texture_coords);

#[derive(Clone, Copy)]
struct TextRectVertex {
    position: [f32; 2],
    texture_coords: [f32; 2],
    color: [f32; 4],
}

impl TextRectVertex {
    fn new((x, y): (f32, f32), (u, v): (f32, f32), color: Color) -> Self {
        Self {
            position: [x, y],
            texture_coords: [u, v],
            color: color.into(),
        }
    }
}

implement_vertex!(TextRectVertex, position, texture_coords, color);

const GLYPH_CACHE_WIDTH: u32 = 1024;
const GLYPH_CACHE_HEIGHT: u32 = 1024;

pub struct Renderer<'a> {
    display: &'a Display,
    storage: &'a AssetStorage<'a>,
    unit_color_rect_vbo: VertexBuffer<ColoredRectVertex>,
    unit_texture_rect_vbo: VertexBuffer<TexturedRectVertex>,
    program_rect_color: Program,
    program_rect_texture: Program,
    program_rect_text: Program,
    glyph_cache: GlyphCache,
    glyph_cache_texture: Texture2d,
}

impl<'a> Renderer<'a> {
    pub fn new(display: &'a Display, storage: &'a AssetStorage) -> Self {
        Self {
            display,
            storage,
            unit_color_rect_vbo: VertexBuffer::new(
                display,
                &[
                    ColoredRectVertex::new(0.0, 1.0),
                    ColoredRectVertex::new(0.0, 0.0),
                    ColoredRectVertex::new(1.0, 1.0),
                    ColoredRectVertex::new(1.0, 0.0),
                ],
            ).unwrap(),
            unit_texture_rect_vbo: VertexBuffer::new(
                display,
                &[
                    TexturedRectVertex::new((0.0, 1.0), (0.0, 1.0)),
                    TexturedRectVertex::new((0.0, 0.0), (0.0, 0.0)),
                    TexturedRectVertex::new((1.0, 1.0), (1.0, 1.0)),
                    TexturedRectVertex::new((1.0, 0.0), (1.0, 0.0)),
                ],
            ).unwrap(),
            program_rect_color: Program::from_source(
                display,
                include_str!("glsl/rect_color.vs"),
                include_str!("glsl/rect_color.fs"),
                None,
            ).unwrap(),
            program_rect_texture: Program::from_source(
                display,
                include_str!("glsl/rect_texture.vs"),
                include_str!("glsl/rect_texture.fs"),
                None,
            ).unwrap(),
            program_rect_text: Program::from_source(
                display,
                include_str!("glsl/rect_text.vs"),
                include_str!("glsl/rect_text.fs"),
                None,
            ).unwrap(),
            glyph_cache: GlyphCache::new(GLYPH_CACHE_WIDTH, GLYPH_CACHE_HEIGHT, 0.1, 0.1),
            glyph_cache_texture: Texture2d::with_format(
                display,
                RawImage2d {
                    data: Cow::Owned(vec![
                        0u8;
                        GLYPH_CACHE_WIDTH as usize * GLYPH_CACHE_HEIGHT as usize
                    ]),
                    width: GLYPH_CACHE_WIDTH,
                    height: GLYPH_CACHE_HEIGHT,
                    format: texture::ClientFormat::U8,
                },
                texture::UncompressedFloatFormat::U8,
                texture::MipmapsOption::NoMipmap,
            ).unwrap(),
        }
    }

    pub fn do_painting(&mut self, mut painting: Painting) {
        painting.frame.clear_color_srgb(0.2, 0.4, 0.8, 1.0);

        for cmd in painting.commands.iter() {
            cmd.draw(self, &mut painting.frame);
        }

        painting.frame.finish().unwrap();
    }

    fn display_size(&self) -> (f32, f32) {
        let (w, h) = self.display.get_framebuffer_dimensions();
        (w as f32, h as f32)
    }

    fn dpi_factor(&self) -> f32 {
        // TODO: Implement
        1.0
    }
}

pub struct Painting<'a> {
    frame: Frame,
    commands: Vec<Box<DrawCommand + 'a>>,
}

impl<'a> Painting<'a> {
    pub fn new(display: &Display) -> Self {
        Self {
            frame: display.draw(),
            commands: Vec::new(),
        }
    }

    pub fn draw<C: DrawCommand + 'a>(&mut self, command: C) {
        self.commands.push(Box::new(command));
    }
}

pub trait DrawCommand {
    fn draw(&self, renderer: &mut Renderer, frame: &mut Frame);
}

pub struct ColoredRectangle(pub Bbox, pub Color);

impl DrawCommand for ColoredRectangle {
    fn draw(&self, renderer: &mut Renderer, frame: &mut Frame) {
        let origin: na::Vector2<f32> = na::convert(self.0.min);
        let origin: [f32; 2] = origin.into();

        let size: na::Vector2<f32> = na::convert(self.0.size());
        let size: [f32; 2] = size.into();

        let uniforms = uniform! {
            origin: origin,
            size: size,
            frame_size: renderer.display_size(),
            color: Into::<[f32; 4]>::into(self.1),
        };

        frame
            .draw(
                &renderer.unit_color_rect_vbo,
                &NoIndices(PrimitiveType::TriangleStrip),
                &renderer.program_rect_color,
                &uniforms,
                &DrawParameters {
                    blend: Blend::alpha_blending(),
                    ..Default::default()
                },
            )
            .unwrap();
    }
}

pub struct TexturedRectangle(pub Bbox, pub TextureId);

impl DrawCommand for TexturedRectangle {
    fn draw(&self, renderer: &mut Renderer, frame: &mut Frame) {
        if let Some(texture) = renderer.storage.texture(self.1) {
            let origin: na::Vector2<f32> = na::convert(self.0.min);
            let origin: [f32; 2] = origin.into();

            let size: na::Vector2<f32> = na::convert(self.0.size());
            let size: [f32; 2] = size.into();

            let uniforms = uniform! {
                origin: origin,
                size: size,
                frame_size: renderer.display_size(),
                tex: texture
            };

            frame
                .draw(
                    &renderer.unit_texture_rect_vbo,
                    &NoIndices(PrimitiveType::TriangleStrip),
                    &renderer.program_rect_texture,
                    &uniforms,
                    &DrawParameters {
                        blend: Blend::alpha_blending(),
                        ..Default::default()
                    },
                )
                .unwrap();
        }
    }
}

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
    fn layout_paragraph<'a>(&'a self, font: &'a Font) -> Vec<PositionedGlyph<'a>> {
        let mut glyphs = Vec::new();
        for mut word in ParagraphGlyphs::new(&self, &font) {
            glyphs.append(&mut word);
        }
        glyphs
    }
}

impl DrawCommand for Text {
    fn draw(&self, renderer: &mut Renderer, frame: &mut Frame) {
        if let Some(font) = renderer.storage.font(self.font) {
            let glyphs = self.layout_paragraph(font);
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

            let vertices = VertexBuffer::new(renderer.display, &vertices).unwrap();
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
    word: Vec<PositionedGlyph<'a>>,
    text: &'a Text,
    font: &'a Font<'a>,
    chars: ::std::str::Chars<'a>,
    done: bool,
}

impl<'a> ParagraphGlyphs<'a> {
    fn new(text: &'a Text, font: &'a Font) -> Self {
        let v_metrics = font.v_metrics(text.scale);
        let advance_y = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

        Self {
            last_glyph: None,
            caret: GlyphPoint {
                x: 0.0,
                y: advance_y,
            },
            advance_y,
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
                    let g = g.scaled(self.text.scale);
                    self.word.push(g.clone().positioned(self.caret));

                    self.caret.x += g.h_metrics().advance_width;

                    if let Some(ref last_glyph) = self.last_glyph {
                        self.caret.x +=
                            self.font
                                .pair_kerning(self.text.scale, g.id(), last_glyph.id());
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
