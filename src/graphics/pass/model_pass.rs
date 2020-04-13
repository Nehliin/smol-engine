use std::collections::HashMap;

use legion::prelude::*;
use nalgebra::{Matrix4, Vector3};
use wgpu::{
    BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Binding,
    BindingResource, BindingType, BlendDescriptor, Buffer, BufferAddress, BufferDescriptor,
    BufferUsage, Color, ColorStateDescriptor, ColorWrite, CommandBuffer, CommandEncoder,
    CommandEncoderDescriptor, CreateBufferMapped, CullMode, DepthStencilStateDescriptor, Device,
    FrontFace, IndexFormat, InputStepMode, LoadOp, PipelineLayoutDescriptor, PrimitiveTopology,
    ProgrammableStageDescriptor, Queue, RasterizationStateDescriptor, RenderPass,
    RenderPassColorAttachmentDescriptor, RenderPassDepthStencilAttachmentDescriptor,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStage, StoreOp,
    SwapChainDescriptor, SwapChainOutput, Texture, TextureComponentType, TextureFormat,
    TextureView, TextureViewDimension, VertexAttributeDescriptor, VertexBufferDescriptor,
    VertexFormat, VertexStateDescriptor,
};

use crate::components::{AssetManager, ModelHandle, Transform};
use crate::graphics::model::MeshVertex;
use crate::graphics::model::{DrawModel, InstanceData};
use crate::graphics::point_light::PointLightRaw;
//use crate::graphics::uniform_bind_groups::LightUniforms;
use crate::graphics::uniform_bind_groups::{CameraDataRaw, LightUniforms};
use crate::graphics::wgpu_renderer::DEPTH_FORMAT;
use crate::graphics::{PointLight, UniformBindGroup, UniformCameraData};

pub trait VBDesc {
    fn desc<'a>() -> VertexBufferDescriptor<'a>;
}

// make general over path later
fn load_shader() -> (Vec<u32>, Vec<u32>) {
    let vs_src = include_str!("../../shader_files/vertex.shader");
    let fs_src = include_str!("../../shader_files/fragment.shader");

    let vs_spirv = glsl_to_spirv::compile(vs_src, glsl_to_spirv::ShaderType::Vertex).unwrap();
    let fs_spirv = glsl_to_spirv::compile(fs_src, glsl_to_spirv::ShaderType::Fragment).unwrap();
    let vs_data = wgpu::read_spirv(vs_spirv).unwrap();
    let fs_data = wgpu::read_spirv(fs_spirv).unwrap();
    (vs_data, fs_data)
}

pub struct ModelPass {
    render_pipeline: RenderPipeline,
    light_uniforms: UniformBindGroup<LightUniforms>,
}

impl ModelPass {
    pub fn new(
        device: &mut Device,
        texture_layout: &BindGroupLayout,
        main_bind_group_layout: &BindGroupLayout,
        format: TextureFormat,
    ) -> Self {
        let (vs_data, fs_data) = load_shader();
        let vertex_shader = device.create_shader_module(&vs_data);
        let fragment_shader = device.create_shader_module(&fs_data);

        let light_uniforms = UniformBindGroup::new(device, ShaderStage::FRAGMENT);

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[
                &texture_layout,
                main_bind_group_layout,
                &light_uniforms.bind_group_layout,
            ],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: ProgrammableStageDescriptor {
                module: &vertex_shader,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fragment_shader,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format,
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

        Self {
            render_pipeline,
            light_uniforms,
        }
    }
    // fett hacky
    pub fn update_lights(&mut self, world: &mut World, device: &mut Device) -> Vec<CommandBuffer> {
        let query = <(Write<PointLight>, Read<Transform>)>::query();
        let mut command_buffers = Vec::new();
        for chunk in query.iter_chunks_mut(world) {
            let mut lights = chunk.components_mut::<PointLight>().unwrap();
            let positions = chunk.components::<Transform>().unwrap();
            let mut uniform_data = [PointLightRaw::from(PointLight::default()); 16];
            let mut lights_used = 0;
            lights
                .iter_mut()
                .zip(positions.iter())
                .enumerate()
                .for_each(|(i, (light, pos))| {
                    light.position = pos.translation(); // Should not be part of pointlight
                    uniform_data[i] = PointLightRaw::from(*light);
                    lights_used += 1;
                });

            command_buffers.push(self.light_uniforms.update(
                device,
                LightUniforms {
                    lights_used,
                    pad: [0; 3],
                    point_lights: uniform_data,
                },
            ));
        }
        command_buffers
    }

    pub fn update_instances(
        resources: &Resources,
        world: &mut World,
        encoder: &mut CommandEncoder,
        device: &mut Device,
    ) {
        let mut offsets = HashMap::new();
        let query = <(Read<Transform>, Tagged<ModelHandle>)>::query();
        let asset_manager = resources.get::<AssetManager>().unwrap();
        for chunk in query.iter_chunks(world) {
            let model = chunk.tag::<ModelHandle>().unwrap();
            let transforms = chunk.components::<Transform>().unwrap();
            let transforms = transforms
                .iter()
                .map(|trans| trans.as_bytes())
                .flatten()
                .copied()
                .collect::<Vec<u8>>();
            let offset = *offsets.get(model).unwrap_or(&0);
            let temp_buf = device.create_buffer_with_data(
                transforms.as_slice(),
                BufferUsage::VERTEX | BufferUsage::COPY_SRC,
            );
            let instance_buffer = &asset_manager.asset_map.get(model).unwrap().instance_buffer;
            encoder.copy_buffer_to_buffer(
                &temp_buf,
                0,
                instance_buffer,
                0,
                transforms.len() as u64,
            );
            offsets.insert(model.clone(), offset + transforms.len() as u64);
        }
    }

    pub fn render<'pass, 'encoder: 'pass>(
        &'encoder self,
        main_bind_group: &'encoder UniformBindGroup<CameraDataRaw>,
        asset_manager: &'encoder AssetManager,
        world: &'encoder mut World,
        render_pass: &'pass mut RenderPass<'encoder>,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        let query = <(Read<Transform>, Tagged<ModelHandle>)>::query();
        for chunk in query.iter_chunks(world) {
            // This is guarenteed to be the same for each chunk
            let model = chunk.tag::<ModelHandle>().unwrap();
            let transforms = chunk.components::<Transform>().unwrap();
            let model = asset_manager.asset_map.get(model).unwrap();
            render_pass.set_bind_group(2, &self.light_uniforms.bind_group, &[]);
            render_pass.draw_model_instanced(
                model,
                0..transforms.len() as u32, //TODO: must use the same offset map probably?
                &main_bind_group.bind_group,
            )
        }
    }
}
