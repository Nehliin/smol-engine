use crate::shaders::Shader;
use nalgebra::Vector3;
use std::ffi::CString;

pub struct DirectionalLight {
    pub(super) direction: Vector3<f32>,
    pub(super) ambient: Vector3<f32>,
    pub(super) specular: Vector3<f32>,
    pub(super) diffuse: Vector3<f32>,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        DirectionalLight {
            direction: Vector3::new(0.0, -1.0, 0.0),
            ambient: Vector3::new(0.1, 0.1, 0.1),
            specular: Vector3::new(1.0, 1.0, 1.0),
            diffuse: Vector3::new(1.0, 1.0, 1.0),
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
