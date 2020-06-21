use std::sync::Arc;

use anyhow::Result;
use legion::prelude::World;
use smol_renderer::{FragmentShader, RenderNode, TextureData, UniformBindGroup, VertexShader};
use wgpu::{CommandEncoder, Device, RenderPassDescriptor, TextureFormat};

use crate::{assets::AssetManager, graphics::skybox_texture::SkyboxTexture};

use super::Pass;

pub struct SkyboxPass {
    render_node: RenderNode,
    skybox_texture: TextureData<SkyboxTexture>,
}

impl SkyboxPass {
    pub fn new(
        device: &Device,
        global_unifroms: Vec<Arc<UniformBindGroup>>,
        color_format: TextureFormat,
        skybox_texture: TextureData<SkyboxTexture>,
    ) -> Result<Self> {
        let render_node = RenderNode::builder()
            .set_vertex_shader(VertexShader::new(
                device,
                "src/shader_files/vs_skybox.shader",
            )?)
            .set_fragment_shader(FragmentShader::new(
                device,
                "src/shader_files/fs_skybox.shader",
            )?)
            .add_shared_uniform_bind_group(global_unifroms[0].clone())
            .add_texture::<SkyboxTexture>()
            .add_default_color_state_desc(color_format)
            .set_default_rasterization_state() // THIS WAS PREVIOUSLY CW NOT CCW
            .build(device)?;
        Ok(Self {
            render_node,
            skybox_texture,
        })
    }
}

impl Pass for SkyboxPass {
    fn update_uniform_data(
        &self,
        _world: &legion::prelude::World,
        _asset_manager: &crate::assets::AssetManager,
        _device: &Device,
        _encoder: &mut wgpu::CommandEncoder,
    ) {
        todo!()
    }

    fn render<'encoder>(
        &'encoder self,
        _asset_manager: &'encoder AssetManager,
        _world: &World,
        encoder: &mut CommandEncoder,
        render_pass_descriptor: RenderPassDescriptor,
    ) {
        let mut runner = self.render_node.runner(encoder, render_pass_descriptor);
        runner.set_texture_data(0, &self.skybox_texture);
        // all vertex data is hardcoded into the shaders
        runner.draw(0..3 as u32, 0..1);
    }
}
