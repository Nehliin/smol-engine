use crate::assets::AssetManager;
use legion::prelude::World;
use wgpu::Device;
use wgpu::{CommandEncoder, RenderPassDescriptor};

pub mod light_object_pass;
pub mod model_pass;
pub mod shadow_pass;
pub mod skybox_pass;
pub mod ui_pass;

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
        asset_manager: &'encoder AssetManager,
        world: &World,
        encoder: &mut CommandEncoder,
        render_pass_descriptor: RenderPassDescriptor,
    );
}
