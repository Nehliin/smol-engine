use nalgebra::{Matrix4, Unit, Vector3};
use nphysics3d::object::{DefaultBodyHandle, DefaultColliderHandle};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightTag;

pub struct PhysicsBody {
    pub body_handle: DefaultBodyHandle,
    pub collider_handle: DefaultColliderHandle,
}

pub struct Transform {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub angle: f32,
}
// TODO: implement buildar macro or by hand (new builder struct)
impl Transform {
    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_translation(&self.position)
            * Matrix4::from_axis_angle(&Unit::new_normalize(self.rotation), self.angle.to_radians())
            * Matrix4::new_nonuniform_scaling(&Vector3::new(
                self.scale.x,
                self.scale.y,
                self.scale.z,
            ))
    }
}

#[derive(Clone, PartialEq)]
pub struct Selected;
