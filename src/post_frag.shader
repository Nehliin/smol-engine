#version 330 core
out vec4 FragColor;
  
in vec2 outTexCoords;

uniform sampler2D frame_buffer_texture;

void main() { 
    FragColor = vec4(vec3(1.0 - texture(frame_buffer_texture, outTexCoords)), 1.0);
}