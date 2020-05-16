#version 400 core

layout (location = 0) in vec4 vertex;
layout (location = 1) in vec4 bg_color;

uniform mat4 perspective;
uniform int with_radius;

out struct VS_OUT {
    vec4 bg_color;
} vs_out;

void main() {
    vec2 root = vertex.xy;
    vec2 dem = vertex.zw;

    vec2 position;
    position.x = (gl_VertexID == 0 || gl_VertexID == 1) ? 1.0 : 0.0;
    position.y = (gl_VertexID == 0 || gl_VertexID == 3) ? 0.0 : 1.0;

    vec2 vert = root + dem * position;
    gl_Position = perspective * vec4(vert, 0, 1);

    vs_out.bg_color = bg_color;
}
