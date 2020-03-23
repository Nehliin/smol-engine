#version 330 core
out vec4 FragColor;
  
in vec2 outTexCoords;

uniform float time;
uniform sampler2D frame_buffer_texture;

void main() { 
    FragColor = texture(frame_buffer_texture, outTexCoords);
    // Apply gamma correction:
    float gamma = 2.2;
    FragColor.rgb = pow(FragColor.rgb, vec3(1.0/gamma));
}