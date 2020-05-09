use legion::prelude::{Resources, World};

pub mod model;
pub mod pass;
pub mod shader;
pub mod shadow_texture;
pub mod skybox_texture;
pub mod texture;
pub mod wgpu_renderer;
pub mod lighting;

pub use shader::Shader;
pub use shader::ShaderLoadError;
mod uniform_bind_groups;

pub use pass::Pass;

pub use wgpu_renderer::WgpuRenderer;

pub use uniform_bind_groups::{UniformBindGroup, UniformCameraData};
//pub mod basic_renderer;
//pub use basic_renderer::BasicRenderer;
use glfw::{Glfw, Window};
// This trait isn't object safe and shouldn't need to be, I can't see any use case for a
// heterogenus renderer collection
pub trait Renderer {
    // This will set glfw window hints specific to the renderer used
    fn set_window_hints(glfw: &mut Glfw);
    // This will initalize the renderer equivalent to ::new() on open gl renderer
    // This also takes a window where the open gl renderer can for example load
    // gl symbols, the wgpu renderer will create surface and devices here etc
    fn new(window: &Window, resources: &mut Resources) -> Self;
    // called when a resize event is discovered
    fn resize(&mut self, width: u32, height: u32);
    // actually renders the frame
    fn render_frame(&mut self, world: &mut World, resources: &mut Resources);
}
