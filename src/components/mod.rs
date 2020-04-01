use nalgebra::{Isometry3, Matrix4, Vector3};
use nphysics3d::object::{DefaultBodyHandle, DefaultColliderHandle};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightTag;

pub struct PhysicsBody {
    pub body_handle: DefaultBodyHandle,
    pub collider_handle: DefaultColliderHandle,
}

pub struct Transform {
    pub isometry: Isometry3<f32>,
    pub scale: Vector3<f32>,
}
// TODO: implement buildar macro or by hand (new builder struct)
impl Transform {
    pub fn from_position(position: Vector3<f32>) -> Self {
        Transform {
            isometry: Isometry3::translation(position.x, position.y, position.z),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn new(isometry: Isometry3<f32>, scale: Vector3<f32>) -> Self {
        Transform { isometry, scale }
    }

    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        self.isometry.to_homogeneous()
            * Matrix4::new_nonuniform_scaling(&Vector3::new(
                self.scale.x,
                self.scale.y,
                self.scale.z,
            ))
    }

    pub fn translation(&self) -> Vector3<f32> {
        self.isometry.translation.vector
    }
}

#[derive(Clone, PartialEq)]
pub struct Selected;
