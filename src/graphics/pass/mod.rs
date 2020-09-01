use legion::prelude::*;
use wgpu::Device;
use wgpu::{CommandEncoder, RenderPassDescriptor};

pub mod light_object_pass;
pub mod model_pass;
pub mod shadow_pass;
pub mod skybox_pass;
pub mod water_pass;

pub trait Pass {
    fn update_uniform_data(
        &self,
        world: &World,
        resources: &Resources,
        device: &Device,
        encoder: &mut CommandEncoder,
    );
    fn render<'encoder>(
        &'encoder self,
        resources: &'encoder Resources,
        world: &World,
        encoder: &mut CommandEncoder,
        render_pass_descriptor: RenderPassDescriptor,
    );
}
