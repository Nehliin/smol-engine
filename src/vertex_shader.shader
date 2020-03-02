#version 330 core
layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aTexCoords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec3 fragmentPosition; //vertex position in world coordinates
out vec3 normal; //vertex surface normal
out vec2 texCoords;

void main() {

  fragmentPosition = vec3(model * vec4(pos, 1.0));
  // this also fixed issue where surfaces lightning was static because the model changed but the normals where in local coordinates not world coordinates
  normal = mat3(transpose(inverse(model))) * aNormal; //make sure surface normals doesn't become fucked when scaling
  texCoords = aTexCoords;
  gl_Position = projection * view * vec4(fragmentPosition, 1.0);
}