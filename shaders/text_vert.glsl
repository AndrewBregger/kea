#version 400 core

layout (location = 0) in vec4 vertex;
layout (location = 1) in vec4 fg_color;
layout (location = 2) in vec4 tex_info;
layout (location = 3) in float tex_id;

// uniform vec4 perspective;
uniform mat4 perspective;

out struct VEC_IN {
    vec4 fg_color;
    vec2 tex_info;
    float tex_id;
} vs_out;

void main() {
    vec2 root = vertex.xy;
    vec2 dem = vertex.zw;

    vec2 uv_root = tex_info.xy;
    vec2 uv_dem = tex_info.zw;

    vec2 position;
    position.x = (gl_VertexID == 0 || gl_VertexID == 1) ? 0.0 : 1.0;
    position.y = (gl_VertexID == 0 || gl_VertexID == 3) ? 0.0 : 1.0;

    vec2 vert  = root + dem * position;
    vec2 uv    = uv_root + uv_dem * position;

    // gl_Position = vec4(offset + proj * vert, 0, 1);
    gl_Position = perspective * vec4(vert, 0, 1);
    vs_out.fg_color = fg_color;
    vs_out.tex_info = uv;
    vs_out.tex_id = tex_id;
}
