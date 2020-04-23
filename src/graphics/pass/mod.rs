use crate::assets::AssetManager;
use legion::prelude::World;
use wgpu::Device;
use wgpu::{BindGroup, CommandEncoder, RenderPass, VertexBufferDescriptor};

pub mod light_object_pass;
pub mod model_pass;

pub trait VBDesc {
    fn desc<'a>() -> VertexBufferDescriptor<'a>;
}

pub trait Pass {
    fn update_uniform_data(
        &self,
        world: &World,
        asset_manager: &AssetManager,
        device: &Device,
        encoder: &mut CommandEncoder,
    );
    fn render<'encoder>(
        &'encoder self,
        global_bind_groups: &[&'encoder BindGroup],
        asset_manager: &'encoder AssetManager,
        world: &World,
        render_pass: &mut RenderPass<'encoder>,
    );
}
