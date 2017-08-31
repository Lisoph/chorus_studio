#version 330

uniform sampler2D tex;

in vec2 tex_coords;
in vec4 vertex_color;

out vec4 frag_color;

void main() {
    float alpha = texture(tex, tex_coords).r;
    frag_color = vec4(vertex_color.rgb, vertex_color.a * alpha);
}