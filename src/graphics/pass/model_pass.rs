use crate::assets::{AssetManager, ModelHandle};
use crate::graphics::Pass;
use crate::{
    components::Transform,
    graphics::{shadow_texture::ShadowTexture, model::MeshVertex},
    graphics::model::{DrawModel, InstanceData},
    graphics::wgpu_renderer::DEPTH_FORMAT,
    graphics::PointLight,
};
use anyhow::Result;
use glsl_to_spirv::ShaderType;
use legion::prelude::*;
use nalgebra::{Matrix4, Vector3};
use smol_renderer::{SimpleTexture, RenderNode, VertexShader, FragmentShader};
use std::collections::HashMap;
use wgpu::{
    BindGroup, BindGroupLayout, BlendDescriptor, BufferUsage, ColorStateDescriptor, ColorWrite,
    CommandEncoder, CullMode, Device, FrontFace, IndexFormat, PipelineLayoutDescriptor,
    PrimitiveTopology, RasterizationStateDescriptor, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderStage, TextureFormat, VertexStateDescriptor,
};

pub struct ModelPass {
    render_node: RenderNode,
}

impl ModelPass {
    pub fn new(
        device: &Device,
        incoming_layouts: Vec<&BindGroupLayout>,
        color_format: TextureFormat,
    ) -> Result<Self> {
        let render_node = RenderNode::builder()
            .add_vertex_buffer::<MeshVertex>()
            .add_vertex_buffer::<InstanceData>()
            .set_vertex_shader(VertexShader::new(device, "src/shader_files/vs_model.shader")?)
            .set_fragment_shader(FragmentShader::new(device, "src/shader_files/fs_model.shader")?)
            // diffuse
            .add_texture::<SimpleTexture>(ShaderStage::FRAGMENT)
            // specular
            .add_texture::<SimpleTexture>(ShaderStage::FRAGMENT)
            // shadow texture
            .add_texture::<ShadowTexture>(ShaderStage::FRAGMENT)
            .set_default_depth_stencil_state()
            .set_default_rasterization_state()
            .add_shared_uniform_bind_group(shared_uniform)
            //.attach_global_uniform_bind_group(uniform)
            .build(&device, color_format);
         
        Ok(Self { render_node})
    }
}

impl Pass for ModelPass {
    fn update_uniform_data(
        &self,
        world: &World,
        asset_manager: &AssetManager,
        device: &Device,
        encoder: &mut CommandEncoder,
    ) {
        let mut offsets = HashMap::new();
        let query = <(Read<Transform>, Tagged<ModelHandle>)>::query();
        for chunk in query.par_iter_chunks(world) {
            let model = chunk.tag::<ModelHandle>().unwrap();
            let transforms = chunk.components::<Transform>().unwrap();
            let model_matrices = transforms
                .iter()
                .map(|trans| trans.get_model_matrix())
                .collect::<Vec<Matrix4<f32>>>();
            // Safety the vector is owned within the same scope so this slice is also valid within
            // the same scope
            let data = unsafe {
                std::slice::from_raw_parts(
                    model_matrices.as_ptr() as *const u8,
                    model_matrices.len() * std::mem::size_of::<Matrix4<f32>>(),
                )
            };
            let offset = *offsets.get(model).unwrap_or(&0);
            let temp_buf =
                device.create_buffer_with_data(data, BufferUsage::VERTEX | BufferUsage::COPY_SRC);
            let instance_buffer = &asset_manager.get_model(model).unwrap().instance_buffer;
            encoder.copy_buffer_to_buffer(&temp_buf, 0, instance_buffer, offset, data.len() as u64);
            offsets.insert(model.clone(), offset + model_matrices.len() as u64);
        }
    }

    fn render<'encoder>(
        &'encoder self,
        global_bind_groups: &[&'encoder BindGroup],
        asset_manager: &'encoder AssetManager,
        world: &World,
        render_pass: &mut RenderPass<'encoder>,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        // Bindgroup 0 is for the model textures set in the drawcall
        // 1 = camera uniforms
        render_pass.set_bind_group(1, global_bind_groups[0], &[]);
        // 2 = light uniforms
        render_pass.set_bind_group(2, global_bind_groups[1], &[]);
        // 3 = shadow texture uniforms
        render_pass.set_bind_group(3, global_bind_groups[2], &[]);
        let mut offset_map = HashMap::new();
        let query =
            <(Read<Transform>, Tagged<ModelHandle>)>::query().filter(!component::<PointLight>());
        for chunk in query.par_iter_chunks(world) {
            // This is guaranteed to be the same for each chunk
            let model = chunk.tag::<ModelHandle>().unwrap();
            let offset = *offset_map.get(model).unwrap_or(&0);
            let transforms = chunk.components::<Transform>().unwrap();
            offset_map.insert(model.clone(), offset + transforms.len());
            let model = asset_manager.get_model(model).unwrap();
            render_pass
                .draw_model_instanced(model, offset as u32..(offset + transforms.len()) as u32);
        }
    }
}
