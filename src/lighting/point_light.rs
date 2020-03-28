#![allow(dead_code)]

use crate::components::Transform;
use crate::lighting::Strength;
use crate::shaders::Shader;
use nalgebra::Vector3;
use std::ffi::CString;

pub struct PointLight {
    pub(super) position: Vector3<f32>,
    pub(super) ambient: Vector3<f32>,
    pub(super) specular: Vector3<f32>,
    pub(super) diffuse: Vector3<f32>,
    pub(super) constant: f32,
    pub(super) linear: f32,
    pub(super) quadratic: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        let constant = 1.0;
        let linear = 0.09;
        let quadratic = 0.032;

        PointLight {
            position: Vector3::new(0.0, 0.0, 0.0),
            ambient: Vector3::new(0.1, 0.1, 0.1),
            specular: Vector3::new(1.0, 1.0, 1.0),
            diffuse: Vector3::new(1.0, 1.0, 1.0),
            constant,
            linear,
            quadratic,
        }
    }
}

impl PointLight {
    pub fn set_position(mut self, position: Vector3<f32>) -> Self {
        self.position = position;
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

    pub unsafe fn set_uniforms(&self, shader: &mut Shader, i: usize, transform: &Transform) {
        shader.set_float(
            &CString::new(format!("pointLights[{}].constant", i)).unwrap(),
            self.constant,
        );
        shader.set_float(
            &CString::new(format!("pointLights[{}].quadratic", i)).unwrap(),
            self.quadratic,
        );
        shader.set_float(
            &CString::new(format!("pointLights[{}].linear", i)).unwrap(),
            self.linear,
        );

        shader.set_vector3(
            &CString::new(format!("pointLights[{}].ambient", i)).unwrap(),
            &self.ambient,
        );
        shader.set_vector3(
            &CString::new(format!("pointLights[{}].diffuse", i)).unwrap(),
            &self.diffuse,
        );
        shader.set_vector3(
            &CString::new(format!("pointLights[{}].specular", i)).unwrap(),
            &self.specular,
        );
        shader.set_vector3(
            &CString::new(format!("pointLights[{}].position", i)).unwrap(),
            &transform.position,
        );
    }

    pub fn set_strength(mut self, strength: Strength) -> Self {
        let (constant, linear, quadratic) = strength.get_values();
        self.constant = constant;
        self.linear = linear;
        self.quadratic = quadratic;
        self
    }
}
