use crate::graphics::model::Model;
use nalgebra::{Isometry3, Matrix4, Vector3};
use nphysics3d::object::{DefaultBodyHandle, DefaultColliderHandle};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightTag;

pub struct PhysicsBody {
    pub body_handle: DefaultBodyHandle,
    pub collider_handle: DefaultColliderHandle,
}

#[derive(Clone, PartialEq)]
pub struct Cube;

#[repr(C)]
#[derive()]
pub struct Transform {
    pub isometry: Isometry3<f32>,
    pub scale: Vector3<f32>,
    model_matrix: Matrix4<f32>, // Might be unecessary
}
// TODO: implement buildar macro or by hand (new builder struct)
impl Transform {
    pub fn from_position(position: Vector3<f32>) -> Self {
        let isometry = Isometry3::translation(position.x, position.y, position.z);
        let scale = Vector3::new(1.0, 1.0, 1.0);
        let model_matrix = isometry.to_homogeneous() * Matrix4::new_nonuniform_scaling(&scale);
        Transform {
            isometry,
            scale,
            model_matrix,
        }
    }

    pub fn new(isometry: Isometry3<f32>, scale: Vector3<f32>) -> Self {
        let model_matrix = isometry.to_homogeneous() * Matrix4::new_nonuniform_scaling(&scale);
        Transform {
            isometry,
            scale,
            model_matrix,
        }
    }

    // Safety: this should always be safe since the outgoing lifetime is bound to lifetime
    // to self which owns the underlying data
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.model_matrix.data.as_ptr() as *const u8,
                std::mem::size_of_val(&self.model_matrix),
            )
        }
    }

    pub fn translation(&self) -> Vector3<f32> {
        self.isometry.translation.vector
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub struct ModelHandle {
    pub id: usize,
}

pub struct AssetManager {
    pub asset_map: HashMap<ModelHandle, Model>,
}
impl AssetManager {
    pub fn new() -> Self {
        Self {
            asset_map: HashMap::new(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Selected;
