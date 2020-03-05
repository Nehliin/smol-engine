#![allow(dead_code)]

use crate::lighting::Strength;
use cgmath::vec3;
use cgmath::Vector3;

pub struct SpotLight {
    pub direction: Vector3<f32>,
    pub position: Vector3<f32>,
    pub(super) ambient: Vector3<f32>,
    pub(super) specular: Vector3<f32>,
    pub(super) diffuse: Vector3<f32>,

    pub(super) cutoff: f32,
    pub(super) outer_cutoff: f32,

    pub(super) constant: f32,
    pub(super) linear: f32,
    pub(super) quadratic: f32,
}

impl Default for SpotLight {
    fn default() -> Self {
        let constant = 1.0;
        let linear = 0.09;
        let quadratic = 0.032;

        SpotLight {
            direction: vec3(0.0, -1.0, 0.0),
            position: vec3(0.0, 0.0, 0.0),
            ambient: vec3(0.1, 0.1, 0.1),
            specular: vec3(1.0, 1.0, 1.0),
            diffuse: vec3(1.0, 1.0, 1.0),
            constant,
            linear,
            quadratic,
            cutoff: 12.5_f32.to_radians().cos(),
            outer_cutoff: 15.5_f32.to_radians().cos(),
        }
    }
}

impl SpotLight {
    pub fn set_direction(mut self, direction: Vector3<f32>) -> Self {
        self.direction = direction;
        self
    }
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

    pub fn set_cutoff(mut self, cutoff: f32) -> Self {
        self.cutoff = cutoff;
        self
    }

    pub fn set_outer_cutoff(mut self, outer_cutoff: f32) -> Self {
        self.outer_cutoff = outer_cutoff;
        self
    }

    pub fn set_strength(mut self, strength: Strength) -> Self {
        let (linear, quadratic) = strength.get_values();
        self.linear = linear;
        self.quadratic = quadratic;
        self
    }
}
