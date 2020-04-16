use crate::components::{AssetManager, ModelHandle, Transform};
use crate::graphics::model::{DrawModel, InstanceData, MeshVertex, Model};
use crate::graphics::pass::VBDesc;
use crate::graphics::uniform_bind_groups::CameraDataRaw;
use crate::graphics::wgpu_renderer::DEPTH_FORMAT;
use crate::graphics::{PointLight, Shader, UniformBindGroup};
use glsl_to_spirv::ShaderType;
use legion::prelude::*;
use wgpu::{
    BindGroupLayout, BlendDescriptor, ColorStateDescriptor, ColorWrite, CullMode, Device,
    FrontFace, IndexFormat, PipelineLayoutDescriptor, PrimitiveTopology,
    RasterizationStateDescriptor, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    TextureFormat, VertexStateDescriptor,
};

pub struct LightObjectPass {
    render_pipeline: RenderPipeline,
}

impl LightObjectPass {
    pub fn new(
        device: &Device,
        texture_layout: &BindGroupLayout,
        main_bind_group_layout: &BindGroupLayout,
        format: TextureFormat,
    ) -> Self {
        let vs_shader = Shader::new(
            &device,
            "src/shader_files/vs_light.shader",
            ShaderType::Vertex,
        )
        .unwrap();
        let fs_shader = Shader::new(
            &device,
            "src/shader_files/fs_light.shader",
            ShaderType::Fragment,
        )
        .unwrap();

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[texture_layout, main_bind_group_layout],
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

    pub fn render<'pass, 'encoder: 'pass>(
        &'encoder self,
        main_bind_group: &'encoder UniformBindGroup<CameraDataRaw>,
        asset_manager: &'encoder AssetManager,
        world: &'encoder mut World,
        render_pass: &'pass mut RenderPass<'encoder>,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        let query =
            <(Read<Transform>, Tagged<ModelHandle>)>::query().filter(component::<PointLight>());
        for chunk in query.iter_chunks(world) {
            let model_handle = chunk.tag::<ModelHandle>().unwrap();
            let model = asset_manager.asset_map.get(model_handle).unwrap();
            let transforms = chunk.components::<Transform>().unwrap();
            render_pass.draw_model_instanced(
                model,
                0..transforms.len() as u32,
                &main_bind_group.bind_group,
            )
        }
    }
}
