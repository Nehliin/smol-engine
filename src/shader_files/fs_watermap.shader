#version 450

layout(location=0) in vec4 world_pos;
layout(location=1) in float projected_depth;

layout(location=0) out vec4 fragment;

void main() {
    fragment = vec4(world_pos.xyz, projected_depth); 
}