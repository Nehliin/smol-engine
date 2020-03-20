use cgmath::Vector3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightTag;

pub struct Transform {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub angle: f32,
}
