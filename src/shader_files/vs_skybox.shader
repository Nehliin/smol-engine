#version 450
layout (location = 0) out vec3 tex_coords;

layout (set=1, binding=0)
uniform Uniforms {
    mat4 view;
    mat4 projection;
    vec3 view_pos;
};

const mat4 CONVERSION = mat4(
1.0, 0.0, 0.0, 0.0,
0.0, 1.0, 0.0, 0.0,
0.0, 0.0, 0.5, 0.0,
0.0, 0.0, 0.5, 1.0);

void main() {
    vec4 pos = vec4(0.0);
    switch(gl_VertexIndex) {
        case 0: pos = vec4(-1.0, -1.0, 0.0, 1.0); break;
        case 1: pos = vec4( 3.0, -1.0, 0.0, 1.0); break;
        case 2: pos = vec4(-1.0,  3.0, 0.0, 1.0); break;
    }
    // NOTE IF CONVERSION IS ADDED HERE IN THE CALCULATIONS
    // THE FOV gets really wierd for the skybox and it looks odd
    // for example if you move around the camera the skybox "follows"
    // one could try to redo it more like in the old openGL shaders later on.
    // But it's important to remember when the conversion is precalculated for all camera
    // uniforms because things will be wierd
    mat3 invModelView = transpose(mat3(view));
    vec3 unprojected = (inverse(projection) * pos).xyz;
    tex_coords = invModelView * unprojected;
    gl_Position = pos;
}