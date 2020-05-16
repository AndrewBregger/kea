#version 400 core

uniform sampler2D texs[8];
uniform mat4 perspective;

in struct VEC_IN {
    vec4 fg_color;
    vec2 tex_info;
    float tex_id;
} vs_out;

layout (location = 0, index = 0) out vec4 color0;
layout (location = 0, index = 1) out vec4 color1;

void main() {
    int tex_id = int(vs_out.tex_id);

    vec2 uv = vs_out.tex_info.xy;
    vec3 color = texture(texs[tex_id], uv).rgb;
    color0 = vec4(vs_out.fg_color.xyz, color.r);
    color1 = vec4(color, color.r);
}
