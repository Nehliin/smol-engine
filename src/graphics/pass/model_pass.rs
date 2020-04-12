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
use crate::graphics::wgpu_renderer::{UniformBindGroup, DEPTH_FORMAT};

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

/*#[repr(C)]
#[derive(Copy, Clone)]
struct InstanceData {
    pub model: Matrix4<f32>,
}

struct InstanceBindGroup {
    buffer: Buffer,
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
}

const MAX_INSTANCE_LENGTH: usize = 100;

impl InstanceBindGroup {
    pub fn new(device: &mut Device) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (MAX_INSTANCE_LENGTH * std::mem::size_of::<InstanceData>()) as u64,
            usage: BufferUsage::STORAGE_READ | BufferUsage::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                // This is the layout of the uniform buffer
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
            ],
            label: Some("Instance Bind group layout"),
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[Binding {
                binding: 0,
                resource: BindingResource::Buffer {
                    buffer: &buffer,
                    range: 0..(std::mem::size_of::<InstanceData>() * MAX_INSTANCE_LENGTH)
                        as BufferAddress,
                },
            }],
            label: Some("Uniform bind group"),
        });
        Self {
            bind_group,
            buffer,
            bind_group_layout,
        }
    }
    // Store staging buffer within the pass instead? use mem replace in that case
    pub fn update(&self, device: &mut Device, transform_data: &[Transform]) -> CommandBuffer {
        let instance_data = transform_data
            .iter()
            .map(|trans| InstanceData {
                model: trans.get_model_matrix(),
            })
            .collect::<Vec<InstanceData>>();
        let staging_buffer =
            device.create_buffer_with_data(instance_data.as_bytes(), BufferUsage::COPY_SRC);
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("UniformCameraData staging buffer"),
        });

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.buffer,
            0,
            std::mem::size_of_val(&instance_data) as BufferAddress,
        );
        encoder.finish()
    }
}*/

pub struct ModelPass {
    render_pipeline: RenderPipeline,
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

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&texture_layout, main_bind_group_layout],
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
                cull_mode: CullMode::Back, // TODO: change this
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

        Self { render_pipeline }
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
        main_bind_group: &'encoder UniformBindGroup,
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
            render_pass.draw_model_instanced(
                model,
                0..transforms.len() as u32, //TODO: must use the same offset map probably?
                &main_bind_group.bind_group,
            )
        }
    }
}
