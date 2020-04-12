#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;
layout(location=2) in vec2 tex_coords;

layout(location=3) in mat4 model;

layout(location=0) out vec2 v_tex_coords;

layout(set=1, binding=0)
uniform Uniforms {
    mat4 view;
    mat4 projection;
};

const mat4 CONVERSION = mat4(
1.0, 0.0, 0.0, 0.0,
0.0, 1.0, 0.0, 0.0,
0.0, 0.0, 0.5, 0.0,
0.0, 0.0, 0.5, 1.0);

const mat4 test = mat4(    1.0, 0.0, 0.0, 0.0,
0.0, 1.0, 0.0, -1.0,
0.0, 0.0, 1.0, 0.0,
0.0, 0.0, 0.0, 1.0);

void main() {
    v_tex_coords = tex_coords;
    gl_Position = CONVERSION * projection * view * model * vec4(a_position, 1.0);
}