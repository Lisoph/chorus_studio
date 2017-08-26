use gui::Bbox;

use asset_storage::{AssetStorage, TextureId};

use glium::{Display, Frame, VertexBuffer, IndexBuffer, Program, Surface, DrawParameters, Blend};
use glium::texture::CompressedSrgbTexture2d;
use glium::index::PrimitiveType;
use glium::backend::Facade;
use glium::uniforms::EmptyUniforms;

use nalgebra as na;

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

pub struct Renderer<'a> {
    display: &'a Display,
    storage: &'a AssetStorage,
    unit_color_rect_vbo: VertexBuffer<ColoredRectVertex>,
    unit_texture_rect_vbo: VertexBuffer<TexturedRectVertex>,
    unit_rect_ibo: IndexBuffer<u8>,
    program_rect_color: Program,
    program_rect_texture: Program,
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
            unit_rect_ibo: IndexBuffer::new(
                display,
                PrimitiveType::TriangleStrip,
                &[0, 1, 2, 1, 2, 3],
            ).unwrap(),
            program_rect_color: Program::from_source(
                display,
                SHADER_VERTEX_SRC_COLOR,
                SHADER_FRAGMENT_SRC_COLOR,
                None,
            ).unwrap(),
            program_rect_texture: Program::from_source(
                display,
                SHADER_VERTEX_SRC_TEXTURE,
                SHADER_FRAGMENT_SRC_TEXTURE,
                None,
            ).unwrap(),
        }
    }

    pub fn do_painting(&mut self, mut painting: Painting) {
        painting.frame.clear_color_srgb(0.2, 0.4, 0.8, 1.0);

        for cmd in painting.commands.iter() {
            cmd.draw(self, &mut painting.frame, self.storage);
        }

        painting.frame.finish().unwrap();
    }

    fn display_size(&self) -> (f32, f32) {
        let (w, h) = self.display.get_framebuffer_dimensions();
        (w as f32, h as f32)
    }
}

pub struct Painting {
    frame: Frame,
    commands: Vec<Box<DrawCommand>>,
}

impl Painting {
    pub fn new(display: &Display) -> Self {
        Self {
            frame: display.draw(),
            commands: Vec::new(),
        }
    }

    pub fn draw<C: DrawCommand + 'static>(&mut self, command: C) {
        self.commands.push(Box::new(command));
    }
}

pub trait DrawCommand {
    fn draw(&self, renderer: &Renderer, frame: &mut Frame, storage: &AssetStorage);
}

pub struct ColoredRectangle(pub Bbox, pub Color);

impl DrawCommand for ColoredRectangle {
    fn draw(&self, renderer: &Renderer, frame: &mut Frame, _storage: &AssetStorage) {
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
                &renderer.unit_rect_ibo,
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
    fn draw(&self, renderer: &Renderer, frame: &mut Frame, storage: &AssetStorage) {
        if let Some(texture) = storage.texture(self.1) {
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
                    &renderer.unit_rect_ibo,
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

const SHADER_VERTEX_SRC_COLOR: &str = r#"
#version 330

uniform vec2 origin, size;
uniform vec2 frame_size;

in vec2 position;

void main() {
    vec2 pos = position * size + origin;
    pos = pos / (frame_size / 2.0) - 1.0;
    gl_Position = vec4(vec2(pos.x, -pos.y), 0.0, 1.0);
}

"#;

const SHADER_VERTEX_SRC_TEXTURE: &str = r#"
#version 330

uniform vec2 origin, size;
uniform vec2 frame_size;

in vec2 position;
in vec2 texture_coords;

out vec2 tex_coords;

void main() {
    vec2 pos = position * size + origin;
    pos = pos / (frame_size / 2.0) - 1.0;
    gl_Position = vec4(vec2(pos.x, -pos.y), 0.0, 1.0);
    tex_coords = texture_coords;
}

"#;

const SHADER_FRAGMENT_SRC_COLOR: &str = r#"
#version 330

uniform vec4 color;
out vec4 frag_color;

void main() {
    frag_color = color;
}

"#;

const SHADER_FRAGMENT_SRC_TEXTURE: &str = r#"
#version 330

uniform sampler2D tex;

in vec2 tex_coords;

out vec4 frag_color;

void main() {
    frag_color = texture(tex, tex_coords);
}

"#;
