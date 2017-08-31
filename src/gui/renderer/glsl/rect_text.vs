#version 330

uniform vec2 origin;
uniform vec2 frame_size;

in vec2 position;
in vec2 texture_coords;
in vec4 color;

out vec2 tex_coords;
out vec4 vertex_color;

void main() {
    vec2 pos = (position + origin) / (frame_size / 2.0) - 1.0;
    gl_Position = vec4(vec2(pos.x, -pos.y), 0.0, 1.0);
    tex_coords = texture_coords;
    vertex_color = color;
}