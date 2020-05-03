#version 450

layout(location=0) in vec2 v_tex_coords;
layout(location=1) in vec3 normal;
layout(location=2) in vec3 fragment_position;
layout(location=3) in vec3 view_pos;

layout(location=0) out vec4 f_color;

struct PointLight {
    vec3 position;
    vec3 ambient;
    vec3 specular;
    vec3 diffuse;
    float constant;
    float linear;
    float quadratic;
    mat4 projection; // used for shadowmapping
};
// handle multiple textures?
layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform sampler s_diffuse;
layout(set = 0, binding = 2) uniform texture2D t_specular;
layout(set = 0, binding = 3) uniform sampler s_specular;


layout(set = 3, binding = 0) uniform texture2DArray t_shadow;
layout(set = 3, binding = 1) uniform samplerShadow s_shadow;

const int MAX_POINT_LIGHTS = 16;
layout(set=2, binding=0) uniform PointLights {
    int lights_used;
    PointLight pointLights[MAX_POINT_LIGHTS];
};


float fetch_shadow(int light_id, vec4 homo_coords) {
    if (homo_coords.w <= 0.0) {
        return 1.0;
    }
    const vec2 flip_correction = vec2(0.5, -0.5);
    vec4 light_local = vec4(
        homo_coords.xy * flip_correction/homo_coords.w + 0.5,
        light_id,
        homo_coords.z / homo_coords.w
    );

    return texture(sampler2DArrayShadow(t_shadow, s_shadow), light_local);
}


float calculate_attenuation(vec3 light_position, float constant, float linear, float quadratic) {
    float distance = length(light_position - fragment_position);
    return 1.0 / (constant + (linear * distance) + quadratic * (distance * distance));
}


vec3 calculate_point_light(PointLight light, vec3 normal) {

    vec3 direction_to_light = normalize(light.position - fragment_position);

    vec3 result = vec3(0.0);

    vec3 viewDir = normalize(view_pos - fragment_position);
    vec3 halfwayDir = normalize(direction_to_light + viewDir);

    float spec = pow(max(dot(normal, halfwayDir), 0.0), 32.0);
    float diff = max(dot(normal, direction_to_light), 0.0);
    float attenuation = calculate_attenuation(light.position, light.constant, light.linear, light.quadratic);


    result += light.specular * attenuation * spec * texture(sampler2D(t_specular, s_specular), v_tex_coords).rgb;


    result += light.diffuse * attenuation * diff * texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords).rgb;


    result += light.ambient * attenuation * texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords).rgb;

    return result;
}

void main() {

    vec3 norm = normalize(normal);
    vec3 result = vec3(0.0);
    for(int i = 0; i < lights_used; i++) {
        float shadow_value = fetch_shadow(i, pointLights[i].projection * vec4(fragment_position, 0.0));
        result += shadow_value * calculate_point_light(pointLights[i], norm);
    }
    //vec3 result = calculate_point_light(pointLights[0], norm);
    //f_color = vec4(vec3(1.0,0.09,0.032), 1.0);//vec4(result, 1.0);//+ texture(sampler2D(t_specular,s_specular), v_tex_coords);
     f_color = vec4(result ,1.0);
    //f_color = vec4(vec3(pointLights.constant, pointLights.linear, pointLights.quadratic), 1.0);
}