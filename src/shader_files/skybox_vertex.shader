#version 330 core
layout (location = 0) in vec3 pos;

out vec3 TexCoords;

uniform mat4 projection;
uniform mat4 view;

void main() {
    TexCoords = pos;
    vec4 temp = projection * view * vec4(pos, 1.0);
    gl_Position = temp.xyww; // set the z coord so the perspective divsion will give the skybox max depth
}