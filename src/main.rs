mod camera;
mod components;
mod engine;
mod graphics;
mod lighting;
pub mod macros;
mod mesh;
mod model;
mod shaders;
mod state;

use crate::engine::Engine;
use crate::graphics::BasicRenderer;
use crate::state::BasicState;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    let mut engine = Engine::new(
        "Smol engine",
        Box::new(BasicState::new()),
        Box::new(BasicRenderer::new()),
    );
    engine.run();
}
