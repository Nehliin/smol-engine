#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform texture2D t_water;
layout(set = 0, binding = 1) uniform sampler s_water;

layout(location = 0) out vec4 out_pos;

layout(set = 1, binding = 0) uniform HeightMapModelMatrix {
    mat4 matrix;
};


layout(set=2, binding=0)
uniform Uniforms {
    mat4 view;
    mat4 projection;
    vec3 view_pos;
};

const mat4 CONVERSION = mat4(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0
    );

void main() {
    vec4 info = texture(sampler2D(t_water, s_water), position.xy * 0.5 + 0.5);
    vec3 pos = vec3(position.xy, position.z + info.r);

    vec3 final_pos = vec3(matrix * vec4(pos, 1.0));

    out_pos = CONVERSION*  projection * view * vec4(final_pos, 1.0);
}