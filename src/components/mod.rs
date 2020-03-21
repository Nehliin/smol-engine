use cgmath::prelude::*;
use cgmath::Matrix4;
use cgmath::{Rad, Vector3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightTag;

pub struct Transform {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub angle: f32,
}
// TODO: implement buildar macro or by hand (new builder struct)
impl Transform {
    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position)
            * Matrix4::from_axis_angle(self.rotation.normalize(), Rad(self.angle.to_radians()))
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }
}

#[derive(Clone, PartialEq)]
pub struct Selected;
