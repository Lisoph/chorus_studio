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