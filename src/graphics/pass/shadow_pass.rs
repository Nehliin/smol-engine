use crate::graphics::lighting::Light;
use super::{Pass, VBDesc};
use crate::{
    assets::{AssetManager, ModelHandle},
    components::Transform,
    graphics::{
        lighting::point_light::PointLightRaw,
        lighting::PointLight,
        model::{DrawModel, InstanceData, MeshVertex},
        shadow_texture::{ShadowTexture, SHADOW_FORMAT},
        uniform_bind_groups::LightSpaceMatrix,
        Shader, UniformBindGroup,
    },
};
use anyhow::Result;
use legion::prelude::World;
use legion::prelude::*;
use std::collections::HashMap;
use wgpu::{BindGroup, BindGroupLayout, CommandBuffer, Device};

// TODO:
// Sample the shadow textures in the model pass (add them as an internal the bindgroup)
// modify the model_pass shaders
// Don't make the point light contian the target_view, that should be a separate component
// Update the resize method
pub struct ShadowPass {
    render_pipeline: wgpu::RenderPipeline,
    light_projection_uniforms: UniformBindGroup<LightSpaceMatrix>,
    pub shadow_texture: ShadowTexture,
}

impl ShadowPass {
    pub fn new(device: &Device, incoming_layouts: Vec<&BindGroupLayout>) -> Result<Self> {
        let vs_shader = Shader::new(
            &device,
            "src/shader_files/vs_shadow.shader",
            glsl_to_spirv::ShaderType::Vertex,
        )?;
        let fs_shader = Shader::new(
            &device,
            "src/shader_files/fs_shadow.shader",
            glsl_to_spirv::ShaderType::Fragment,
        )?;

        let light_projection = UniformBindGroup::new(&device, wgpu::ShaderStage::VERTEX);
        let mut bind_group_layouts = incoming_layouts;
        bind_group_layouts.push(&light_projection.bind_group_layout);

        let shadow_texture = ShadowTexture::new(device);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &bind_group_layouts,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: vs_shader.get_descriptor(),
            fragment_stage: Some(fs_shader.get_descriptor()),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Front,
                depth_bias: 0, // Biliniear filtering
                depth_bias_slope_scale: 2.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: SHADOW_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[MeshVertex::desc(), InstanceData::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Ok(Self {
            render_pipeline,
            light_projection_uniforms: light_projection,
            shadow_texture,
        })
    }

    pub fn update_uniforms<'a, T>(
        &self,
        device: &Device,
        light: &'a T,
        encoder: &mut wgpu::CommandEncoder,
    ) where
        &'a T: Into<LightSpaceMatrix>,
    {
        let matrix = light.into();
        self.light_projection_uniforms
            .update(device, &matrix, encoder);
    }

    pub fn render<T: Light>(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        global_bindgroups: &[&wgpu::BindGroup],
        light: &T,
        world: &World,
        asset_manager: &AssetManager,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: light.get_target_view().as_ref().unwrap(),
                depth_load_op: wgpu::LoadOp::Clear,
                depth_store_op: wgpu::StoreOp::Store,
                clear_depth: 1.0,
                stencil_load_op: wgpu::LoadOp::Clear,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_stencil: 0,
            }),
        });

        pass.set_pipeline(&self.render_pipeline);
        // 0 = model texture bindgroup set in the draw call
        // 1 = light projections
        pass.set_bind_group(1, &self.light_projection_uniforms.bind_group, &[]);

        let mut offset_map = HashMap::new();
        let query =
            <(Read<Transform>, Tagged<ModelHandle>)>::query().filter(!component::<PointLight>());
        for chunk in query.iter_chunks(world) {
            // This is guaranteed to be the same for each chunk
            let model = chunk.tag::<ModelHandle>().unwrap();
            let offset = *offset_map.get(model).unwrap_or(&0);
            let transforms = chunk.components::<Transform>().unwrap();
            offset_map.insert(model.clone(), offset + transforms.len());
            let model = asset_manager.get_model(model).unwrap();
            pass.draw_model_instanced(model, offset as u32..(offset + transforms.len()) as u32);
        }
    }
}
