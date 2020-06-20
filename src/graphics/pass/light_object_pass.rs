use crate::assets::{AssetManager, ModelHandle};
use crate::components::Transform;
use crate::graphics::model::{DrawModel, InstanceData, MeshVertex};
use crate::graphics::wgpu_renderer::DEPTH_FORMAT;
use crate::graphics::{Pass, PointLight};
use anyhow::Result;
use legion::prelude::*;
use smol_renderer::{FragmentShader, RenderNode, SimpleTexture, UniformBindGroup, VertexShader};
use std::sync::Arc;
use wgpu::{
    BindGroup, BindGroupLayout, BlendDescriptor, ColorStateDescriptor, ColorWrite, CommandEncoder,
    CullMode, Device, FrontFace, IndexFormat, PipelineLayoutDescriptor, PrimitiveTopology,
    RasterizationStateDescriptor, RenderPass, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, TextureFormat, VertexStateDescriptor,
};

pub struct LightObjectPass {
    render_node: RenderNode,
}

impl LightObjectPass {
    pub fn new(
        device: &Device,
        global_uniforms: Vec<Arc<UniformBindGroup>>,
        color_format: TextureFormat,
    ) -> Result<Self> {
        let render_node = RenderNode::builder()
            .add_vertex_buffer::<MeshVertex>()
            .add_vertex_buffer::<InstanceData>()
            .set_vertex_shader(VertexShader::new(
                device,
                "src/shader_files/vs_light.shader",
            )?)
            .set_fragment_shader(FragmentShader::new(
                device,
                "src/shader_files/fs_light.shader",
            )?)
            .add_default_color_state_desc(color_format)
            .set_default_depth_stencil_state()
            .set_default_rasterization_state()
            .add_shared_uniform_bind_group(global_uniforms[0])
            //.attach_global_uniform_bind_group(uniform)
            .build(&device)?;
        Ok(LightObjectPass { render_node })
    }
}

impl Pass for LightObjectPass {
    fn update_uniform_data(
        &self,
        _world: &World,
        _asset_manager: &AssetManager,
        _device: &Device,
        _encoder: &mut CommandEncoder,
    ) {
        todo!("Think if it's worth to update the specific model matrixes for this pass here");
    }

    fn render<'encoder>(
        &'encoder self,
        asset_manager: &'encoder AssetManager,
        world: &World,
        encoder: &mut CommandEncoder,
        render_pass_descriptor: RenderPassDescriptor,
    ) {
        let mut runner = self.render_node.runner(encoder, render_pass_descriptor);
        let query =
            <(Read<Transform>, Tagged<ModelHandle>)>::query().filter(component::<PointLight>());
        for chunk in query.par_iter_chunks(world) {
            let model_handle = chunk.tag::<ModelHandle>().unwrap();
            let model = asset_manager.get_model(model_handle).unwrap();
            let transforms = chunk.components::<Transform>().unwrap();
            runner.draw_untextured(model, 0..transforms.len() as u32)
        }
    }
}
