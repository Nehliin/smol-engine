use crate::graphics::Pass;
use crate::{
    components::{AssetManager, ModelHandle, Transform},
    graphics::model::MeshVertex,
    graphics::model::{DrawModel, InstanceData},
    graphics::pass::VBDesc,
    graphics::point_light::PointLightRaw,
    graphics::uniform_bind_groups::{CameraDataRaw, LightUniforms},
    graphics::wgpu_renderer::DEPTH_FORMAT,
    graphics::{PointLight, Shader, UniformBindGroup},
};
use anyhow::Result;
use futures::SinkExt;
use glsl_to_spirv::ShaderType;
use legion::prelude::*;
use nalgebra::{Matrix4, Vector3};
use std::collections::HashMap;
use wgpu::{
    BindGroup, BindGroupLayout, BlendDescriptor, BufferUsage, ColorStateDescriptor, ColorWrite,
    CommandEncoder, CullMode, Device, FrontFace, IndexFormat, PipelineLayoutDescriptor,
    PrimitiveTopology, RasterizationStateDescriptor, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderStage, TextureFormat, VertexStateDescriptor,
};
use zerocopy::AsBytes;

pub struct ModelPass {
    render_pipeline: RenderPipeline,
    light_uniforms: UniformBindGroup<LightUniforms>,
}

impl ModelPass {
    pub fn new(
        device: &Device,
        incoming_layouts: Vec<&BindGroupLayout>,
        color_format: TextureFormat,
    ) -> Result<Self> {
        let vs_shader = Shader::new(
            &device,
            "src/shader_files/vs_model.shader",
            ShaderType::Vertex,
        )?;
        let fs_shader = Shader::new(
            &device,
            "src/shader_files/fs_model.shader",
            ShaderType::Fragment,
        )?;

        let light_uniforms = UniformBindGroup::new(device, ShaderStage::FRAGMENT);
        let mut bind_group_layouts = incoming_layouts;
        bind_group_layouts.push(&light_uniforms.bind_group_layout);

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &bind_group_layouts,
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: vs_shader.get_descriptor(),
            fragment_stage: Some(fs_shader.get_descriptor()),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: color_format,
                alpha_blend: BlendDescriptor::REPLACE,
                color_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint32,
                vertex_buffers: &[MeshVertex::desc(), InstanceData::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Ok(Self {
            render_pipeline,
            light_uniforms,
        })
    }
    // TODO: fett hacky
    pub fn update_lights(&self, world: &World, device: &Device, encoder: &mut CommandEncoder) {
        let query = <(Read<PointLight>, Read<Transform>)>::query();
        for chunk in query.par_iter_chunks(world) {
            let lights = chunk.components::<PointLight>().unwrap();
            let positions = chunk.components::<Transform>().unwrap();
            let mut uniform_data =
                [PointLightRaw::from((PointLight::default(), Vector3::new(0.0, 0.0, 0.0))); 16];
            let mut lights_used = 0;
            lights
                .iter()
                .zip(positions.iter())
                .enumerate()
                .for_each(|(i, (light, pos))| {
                    uniform_data[i] = PointLightRaw::from((*light, pos.translation()));
                    lights_used += 1;
                });
            self.light_uniforms.update(
                device,
                &LightUniforms {
                    lights_used,
                    pad: [0; 3],
                    point_lights: uniform_data,
                },
                encoder,
            );
        }
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
        self.update_lights(world, device, encoder);
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
            let instance_buffer = &asset_manager.asset_map.get(model).unwrap().instance_buffer;
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
        render_pass.set_bind_group(1, global_bind_groups[0], &[]);
        render_pass.set_bind_group(2, &self.light_uniforms.bind_group, &[]);
        let mut offset_map = HashMap::new();
        let query =
            <(Read<Transform>, Tagged<ModelHandle>)>::query().filter(!component::<PointLight>());
        for chunk in query.par_iter_chunks(world) {
            // This is guarenteed to be the same for each chunk
            let model = chunk.tag::<ModelHandle>().unwrap();
            let offset = *offset_map.get(model).unwrap_or(&0);
            let transforms = chunk.components::<Transform>().unwrap();
            offset_map.insert(model.clone(), offset + transforms.len());
            let model = asset_manager.asset_map.get(model).unwrap();
            render_pass
                .draw_model_instanced(model, offset as u32..(offset + transforms.len()) as u32);
        }
    }
}
