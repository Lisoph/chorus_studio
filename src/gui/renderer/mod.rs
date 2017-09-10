pub mod text;

use std::borrow::Cow;
use std::rc::Rc;
use std::cell::RefCell;

use glium::{Blend, Display, DrawParameters, Frame, Program, Surface, VertexBuffer};
use glium::texture;
use glium::texture::{RawImage2d, Texture2d};
use glium::index::{NoIndices, PrimitiveType};

use rusttype::gpu_cache::Cache as GlyphCache;

use nalgebra as na;

use gui::Bbox;
use gui::window::Window;
use asset_storage::{AssetStorage, TextureId};

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

const GLYPH_CACHE_BASE_WIDTH: u32 = 1024;
const GLYPH_CACHE_BASE_HEIGHT: u32 = 1024;

pub struct Renderer<'a, 'b, 'c>
where
    'b: 'a,
{
    window: &'a Window<'b>,
    storage: Rc<RefCell<AssetStorage<'c>>>,
    unit_color_rect_vbo: VertexBuffer<ColoredRectVertex>,
    unit_texture_rect_vbo: VertexBuffer<TexturedRectVertex>,
    program_rect_color: Program,
    program_rect_texture: Program,
    program_rect_text: Program,
    glyph_cache: GlyphCache,
    glyph_cache_texture: Texture2d,
}

impl<'a, 'b, 'c> Renderer<'a, 'b, 'c> {
    pub fn new(window: &'a Window<'b>, storage: Rc<RefCell<AssetStorage<'c>>>) -> Self {
        let dpi_factor = window.window().hidpi_factor().round() as u32;

        Self {
            window,
            storage,
            unit_color_rect_vbo: VertexBuffer::new(
                window.display(),
                &[
                    ColoredRectVertex::new(0.0, 1.0),
                    ColoredRectVertex::new(0.0, 0.0),
                    ColoredRectVertex::new(1.0, 1.0),
                    ColoredRectVertex::new(1.0, 0.0),
                ],
            ).unwrap(),
            unit_texture_rect_vbo: VertexBuffer::new(
                window.display(),
                &[
                    TexturedRectVertex::new((0.0, 1.0), (0.0, 1.0)),
                    TexturedRectVertex::new((0.0, 0.0), (0.0, 0.0)),
                    TexturedRectVertex::new((1.0, 1.0), (1.0, 1.0)),
                    TexturedRectVertex::new((1.0, 0.0), (1.0, 0.0)),
                ],
            ).unwrap(),
            program_rect_color: Program::from_source(
                window.display(),
                include_str!("glsl/rect_color.vs"),
                include_str!("glsl/rect_color.fs"),
                None,
            ).unwrap(),
            program_rect_texture: Program::from_source(
                window.display(),
                include_str!("glsl/rect_texture.vs"),
                include_str!("glsl/rect_texture.fs"),
                None,
            ).unwrap(),
            program_rect_text: Program::from_source(
                window.display(),
                include_str!("glsl/rect_text.vs"),
                include_str!("glsl/rect_text.fs"),
                None,
            ).unwrap(),
            glyph_cache: GlyphCache::new(
                GLYPH_CACHE_BASE_WIDTH * dpi_factor,
                GLYPH_CACHE_BASE_HEIGHT * dpi_factor,
                0.1,
                0.1,
            ),
            glyph_cache_texture: Texture2d::with_format(
                window.display(),
                RawImage2d {
                    data: Cow::Owned(vec![
                        0u8;
                        {
                            let width = GLYPH_CACHE_BASE_WIDTH as usize * dpi_factor as usize;
                            let height = GLYPH_CACHE_BASE_HEIGHT as usize * dpi_factor as usize;
                            width * height
                        }
                    ]),
                    width: GLYPH_CACHE_BASE_WIDTH * dpi_factor,
                    height: GLYPH_CACHE_BASE_HEIGHT * dpi_factor,
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
        let (w, h) = self.window.display().get_framebuffer_dimensions();
        (w as f32, h as f32)
    }

    fn dpi_factor(&self) -> f32 {
        self.window.window().hidpi_factor()
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
        if let Some(texture) = renderer.storage.borrow().texture(self.1) {
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
