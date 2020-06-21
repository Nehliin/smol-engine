#version 450
layout (location = 0) in vec3 tex_coords;
layout (location = 0) out vec4 f_color;

layout (set=1, binding=0) uniform textureCube t_cubemap;
layout (set=1, binding=1) uniform sampler s_cubemap;

void main() {
    f_color = texture(samplerCube(t_cubemap, s_cubemap), tex_coords);
}