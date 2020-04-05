mod camera;
mod components;
mod engine;
mod graphics;
mod lighting;
pub mod macros;
mod mesh;
mod model;
mod physics;
mod shaders;
mod states;

use crate::engine::Engine;
//use crate::graphics::BasicRenderer;
use crate::states::BasicState;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    let mut engine = Engine::new(
        "Smol engine",
        Box::new(BasicState::new()),
        //   BasicRenderer::new(),
    );
    engine.run();
}
