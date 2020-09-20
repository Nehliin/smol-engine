#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform texture2D t_water;
layout(set = 0, binding = 1) uniform sampler s_water;

layout(set = 1, binding = 0) uniform HeightMapModelMatrix {
    mat4 matrix;
};

void main() {
    vec4 info = texture(sampler2D(t_water, s_water), position.xy * 0.5 + 0.5);
    vec3 pos = vec3(position.xy, position.z + info.r);

    gl_Position = matrix * vec4(pos, 1.0);
}