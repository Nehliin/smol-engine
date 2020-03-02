use cgmath::vec3;
use cgmath::Vector3;

pub struct DirectionalLight {
    pub(super) direction: Vector3<f32>,
    pub(super) ambient: Vector3<f32>,
    pub(super) specular: Vector3<f32>,
    pub(super) diffuse: Vector3<f32>,
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
}
