use cgmath::Vector3;
use cgmath::{vec3, Deg};
use cgmath::{Matrix4, Point3};

mod camera;
//mod cube;
mod engine;
mod graphics;
mod lighting;
pub mod macros;
mod mesh;
mod model;
mod shader;
mod state;

use crate::engine::Engine;
use crate::graphics::BasicRenderer;
use crate::state::BasicState;

pub struct Transform {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
}

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[inline]
fn to_vec(point: &Point3<f32>) -> Vector3<f32> {
    Vector3::new(point.x, point.y, point.z)
}

fn main() {
    let mut engine = Engine::new(
        "Smol engine",
        Box::new(BasicState::new()),
        Box::new(BasicRenderer::new()),
    );
    engine.run();
}
