#version 400 core

uniform vec2 resolution;
uniform mat4 perspective;
uniform int with_radius;

in struct VS_OUT {
    vec4 bg_color;
} vs_out;


out vec4 color;

void main() {
    color = vs_out.bg_color;
}