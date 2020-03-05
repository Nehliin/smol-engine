#version 330 core

#define MAX_TEXTURES 5

struct Material {
  sampler2D diffuse_textures[MAX_TEXTURES];
  sampler2D specular_textures[MAX_TEXTURES];
  float shininess;
};


struct DirectionalLight {
  vec3 direction;

  vec3 ambient;
  vec3 specular;
  vec3 diffuse;
};

struct PointLight {
  vec3 position;

  vec3 ambient;
  vec3 specular;
  vec3 diffuse;

  float constant;
  float linear;
  float quadratic;
};

struct SpotLight {
  vec3 direction;
  vec3 position;

  vec3 ambient;
  vec3 specular;
  vec3 diffuse;

  float cutoff;
  float outerCutOff;

  // strength
  float constant;
  float linear;
  float quadratic;
};


#define MAX_POINT_LIGHTS 10  
#define MAX_SPOT_LIGHTS 10

uniform PointLight pointLights[MAX_POINT_LIGHTS];
uniform SpotLight spotLights[MAX_SPOT_LIGHTS];

uniform DirectionalLight directional_light;

uniform Material material;

uniform vec3 viewPos;

uniform int number_of_point_lights;
uniform int number_of_spot_lights;
uniform int number_of_specular_textures; //part of material struct!
uniform int number_of_diffuse_textures; //part of material struct!

out vec4 FragColor;

in vec2 texCoords;
in vec3 normal;
in vec3 fragmentPosition;

float calculate_attenuation(vec3 light_position, float constant, float linear, float quadratic) {
   float distance = length(light_position - fragmentPosition);
   return 1.0 / (constant + (linear * distance) + quadratic * (distance * distance)); 
 }


vec3 calculate_directional_light(DirectionalLight light, vec3 normal) {
  vec3 result = vec3(0.0);

  vec3 direction_to_light = normalize(-light.direction);
  // ambient calc
  

  // specular calculation
  vec3 viewDir = normalize(viewPos - fragmentPosition);
  vec3 reflectDir = reflect(-direction_to_light, normal);

  float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
  float diff = max(dot(normal, direction_to_light), 0.0);
  for(int i = 0; i < number_of_specular_textures; ++i) {
    result += light.specular * spec * texture(material.specular_textures[i], texCoords).rgb;
  }
  for(int i = 0; i < number_of_diffuse_textures; ++i) {
    result +=  light.diffuse * diff * texture(material.diffuse_textures[i], texCoords).rgb;
    result += light.ambient * texture(material.diffuse_textures[i], texCoords).rgb;
  }
  

  return result;
}

vec3 calculate_point_light(PointLight light, vec3 normal) {

  vec3 direction_to_light = normalize(light.position - fragmentPosition);

  vec3 result = vec3(0.0);

  vec3 viewDir = normalize(viewPos - fragmentPosition);
  vec3 reflectDir = reflect(-direction_to_light, normal);

  float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
  float diff = max(dot(normal, direction_to_light), 0.0);
  float attenuation = calculate_attenuation(light.position, light.constant, light.linear, light.quadratic);
  
  for(int i = 0; i < number_of_specular_textures; ++i) {
    result += light.specular * attenuation * spec * texture(material.specular_textures[i], texCoords).rgb;
  }
  for(int i = 0; i < number_of_diffuse_textures; ++i) {
    result += light.diffuse * attenuation * diff * texture(material.diffuse_textures[i], texCoords).rgb;
    result += light.ambient * attenuation * texture(material.diffuse_textures[i], texCoords).rgb;
  }
  return result; 
}

vec3 calculate_spot_light(SpotLight light, vec3 normal) {

  vec3 direction_to_light = normalize(light.position - fragmentPosition);

  vec3 result = vec3(0.0);

  vec3 viewDir = normalize(viewPos - fragmentPosition);
  vec3 reflectDir = reflect(-direction_to_light, normal);

  float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
  float diff = max(dot(normal, direction_to_light), 0.0);
  float attenuation = calculate_attenuation(light.position, light.constant, light.linear, light.quadratic);

  float theta = dot(direction_to_light, normalize(-light.direction));
  float epsilon = light.cutoff - light.outerCutOff;
  float intensity = clamp((theta - light.outerCutOff) / epsilon, 0.0, 1.0);
  
  for(int i = 0; i < number_of_specular_textures; ++i) {
    result += light.specular * attenuation * spec * texture(material.specular_textures[i], texCoords).rgb;
  }
  for(int i = 0; i < number_of_diffuse_textures; ++i) {
    result += light.diffuse * attenuation * diff * texture(material.diffuse_textures[i], texCoords).rgb;
    result += light.ambient * attenuation * texture(material.diffuse_textures[i], texCoords).rgb;
  }
  result *= intensity;
  return result; 
}



void main() {

 
   vec3 norm = normalize(normal);
  // diffuse_1 = calculate_diffuse(lightDir, norm);
  // specular_1 = calculate_specular(lightDir, norm);
  
  // float theta = dot(lightDir, normalize(-light.direction));
  // float epsilon   = light.cutoff - light.outerCutOff;
  // float intensity = clamp((theta - light.outerCutOff) / epsilon, 0.0, 1.0);

  // diffuse_1 *= intensity;
  // specular_1 *= intensity;
  
  

  // if(directional_light.position.w == 0) {
  //   // directional light
  //   lightDir = normalize(-directional_light.position.xyz); 
  // } else {
  //   // non directional light
  //   lightDir = normalize(directional_light.position.xyz - fragmentPosition);
  // }

  // vec3 diffuse_2 = calculate_diffuse(lightDir, norm);

  // vec3 specular_2 = calculate_specular(lightDir, norm);


  // float attenuation = attenuation(light);
  // ambient *= attenuation;
  // diffuse_1 *= attenuation;
  // specular_1 *= attenuation;
  // vec3 result = (ambient + diffuse_1 + specular_1);
  vec3 result = calculate_directional_light(directional_light, norm);
  for(int i = 0; i < number_of_point_lights; i++) {
    result += calculate_point_light(pointLights[i], norm);
  }
  for(int i = 0; i < number_of_spot_lights; i++) {
    result += calculate_spot_light(spotLights[i], norm);
  }
  FragColor =  vec4(result, 1.0);
     
}