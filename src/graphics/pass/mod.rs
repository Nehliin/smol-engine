use crate::assets::AssetManager;
use legion::prelude::World;
use wgpu::Device;
use wgpu::{BindGroup, CommandEncoder, RenderPass, VertexBufferDescriptor};

pub mod light_object_pass;
pub mod model_pass;
pub mod shadow_pass;
pub mod skybox_pass;


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
