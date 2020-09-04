#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;
layout(location=2) in vec2 tex_coords;

layout(location=3) in mat4 model;

layout(location=0) out vec4 world_pos;
layout(location=1) out float projected_depth;

layout(set=0, binding=0) uniform LightSpaceMatrix {
    mat4 projection_matrix;
};


const mat4 CONVERSION = mat4(
1.0, 0.0, 0.0, 0.0,
0.0, 1.0, 0.0, 0.0,
0.0, 0.0, 0.5, 0.0,
0.0, 0.0, 0.5, 1.0);

void main() {
    world_pos =   model * vec4(a_position, 1.0);
    projected_depth = vec4(  projection_matrix * model * vec4(a_position, 1.0)).z;
    gl_Position = CONVERSION * projection_matrix *  model * vec4(a_position, 1.0);
}