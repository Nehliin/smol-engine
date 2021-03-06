#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;
layout(location=2) in vec2 tex_coords;

layout(location=3) in mat4 model;

layout(location=0) out vec2 v_tex_coords;
layout(location=1) out vec3 normal;
layout(location=2) out vec3 fragment_position;
layout(location=3) out vec3 out_view_pos;


layout(set=3, binding=0)
uniform Uniforms {
    mat4 view;
    mat4 projection;
    vec3 view_pos;
};

const mat4 CONVERSION = mat4(
1.0, 0.0, 0.0, 0.0,
0.0, 1.0, 0.0, 0.0,
0.0, 0.0, 0.5, 0.0,
0.0, 0.0, 0.5, 1.0);

void main() {
    out_view_pos = view_pos;
    fragment_position = vec3(model * vec4(a_position, 1.0));
    v_tex_coords = tex_coords;
    normal = mat3(transpose(inverse(mat3(model)))) * a_normal; //make sure surface normals doesn't become fucked when scaling;
    gl_Position = CONVERSION * projection * view * vec4(fragment_position, 1.0);
}