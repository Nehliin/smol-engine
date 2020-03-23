use crate::shaders::Shader;
use cgmath::vec3;
use cgmath::Vector3;
use std::ffi::CString;

pub struct DirectionalLight {
    pub direction: Vector3<f32>,
    pub ambient: Vector3<f32>,
    pub specular: Vector3<f32>,
    pub diffuse: Vector3<f32>,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        DirectionalLight {
            direction: vec3(0.0, -1.0, 0.0),
            ambient: vec3(0.1, 0.1, 0.1),
            specular: vec3(1.0, 1.0, 1.0),
            diffuse: vec3(1.0, 1.0, 1.0),
        }
    }
}

impl DirectionalLight {
    pub fn set_direction(mut self, direction: Vector3<f32>) -> Self {
        self.direction = direction;
        self
    }

    pub fn set_ambient(mut self, ambient: Vector3<f32>) -> Self {
        self.ambient = ambient;
        self
    }

    pub fn set_specular(mut self, specular: Vector3<f32>) -> Self {
        self.specular = specular;
        self
    }

    pub fn set_diffuse(mut self, diffuse: Vector3<f32>) -> Self {
        self.diffuse = diffuse;
        self
    }

    pub unsafe fn set_uniforms(&self, shader: &mut Shader) {
        shader.set_vector3(
            &CString::new("directional_light.direction").unwrap(),
            &self.direction,
        );
        shader.set_vector3(
            &CString::new("directional_light.ambient").unwrap(),
            &self.ambient,
        );
        shader.set_vector3(
            &CString::new("directional_light.specular").unwrap(),
            &self.specular,
        );
        shader.set_vector3(
            &CString::new("directional_light.diffuse").unwrap(),
            &self.diffuse,
        );
    }
}
