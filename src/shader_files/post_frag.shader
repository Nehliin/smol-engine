#version 330 core
out vec4 FragColor;
  
in vec2 outTexCoords;

uniform float time;
uniform sampler2D frame_buffer_texture;

void main() { 
    //FragColor = texture(frame_buffer_texture, outTexCoords);
    FragColor = texture(frame_buffer_texture, outTexCoords);
    //FragColor = vec4(FragColor.r*sin(time), FragColor.g * cos(time), FragColor.b * cos(time), 1.0);
    //FragColor = vec4(vec3(1.0 - texture(frame_buffer_texture, outTexCoords)), 1.0);
}