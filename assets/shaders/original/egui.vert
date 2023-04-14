#version 450
layout(push_constant) uniform Push {
    mat4 matrix;
    vec4 data0;
    vec4 data1;
    vec4 data2;
    vec4 data3;
} push;

layout(location = 0) in vec2 i_pos;
layout(location = 1) in vec2 i_uv;
layout(location = 2) in vec4 i_col_srgba;

layout(location = 0) out vec4 o_col_rgba_in_gamma;
layout(location = 1) out vec2 o_uv;

void main() {
    o_uv = i_uv;
    o_col_rgba_in_gamma = i_col_srgba / 255.0;
    gl_Position = push.matrix * vec4(i_pos.xy, 0.0, 1.0);
}