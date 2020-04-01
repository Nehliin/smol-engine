use legion::prelude::{Resources, World};

pub mod basic_renderer;
pub use basic_renderer::BasicRenderer;

pub trait Renderer {
    fn init(&mut self, resources: &mut Resources);
    fn render_world(&mut self, world: &mut World, resources: &mut Resources);
}
