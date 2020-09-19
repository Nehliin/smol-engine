use std::sync::Arc;

use anyhow::Result;
use legion::prelude::*;
use smol_renderer::{FragmentShader, RenderNode, UniformBindGroup, VertexShader};
use wgpu::{CommandEncoder, Device, RenderPassDescriptor, TextureFormat};

use crate::graphics::{Pass, PointLight};
use crate::{assets::Assets, components::Transform, graphics::model::Model};
use crate::{
    assets::Handle,
    graphics::model::{DrawModel, InstanceData, MeshVertex},
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
            .add_shared_uniform_bind_group(global_uniforms[0].clone())
            //.attach_global_uniform_bind_group(uniform)
            .build(&device)?;
        Ok(LightObjectPass { render_node })
    }
}

impl Pass for LightObjectPass {
    fn update_uniform_data(
        &self,
        _world: &World,
        _resources: &Resources,
        _device: &Device,
        _encoder: &mut CommandEncoder,
    ) {
        todo!("Think if it's worth to update the specific model matrixes for this pass here");
    }

    fn render<'encoder>(
        &'encoder self,
        resources: &'encoder Resources,
        world: &World,
        encoder: &mut CommandEncoder,
        render_pass_descriptor: RenderPassDescriptor,
    ) {
        let asset_storage = resources
            .get::<Assets<Model>>()
            .expect("asset type not registered");
        let mut runner = self.render_node.runner(encoder, render_pass_descriptor);
        let query =
            <(Read<Transform>, Tagged<Handle<Model>>)>::query().filter(component::<PointLight>());
        for chunk in query.par_iter_chunks(world) {
            let model_handle = chunk.tag::<Handle<Model>>().unwrap();
            let model = asset_storage.get(model_handle).unwrap();
            let transforms = chunk.components::<Transform>().unwrap();
            runner.draw_untextured(model, 0..transforms.len() as u32)
        }
    }
}
