use crate::assets::{AssetManager, ModelHandle};
use crate::components::Transform;
use crate::graphics::model::{DrawModel, InstanceData, MeshVertex};
use crate::graphics::pass::VBDesc;
use crate::graphics::wgpu_renderer::DEPTH_FORMAT;
use crate::graphics::{Pass, lighting::PointLight, Shader};
use glsl_to_spirv::ShaderType;
use legion::prelude::*;
use wgpu::{
    BindGroup, BindGroupLayout, BlendDescriptor, ColorStateDescriptor, ColorWrite, CommandEncoder,
    CullMode, Device, FrontFace, IndexFormat, PipelineLayoutDescriptor, PrimitiveTopology,
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
        global_bind_groups: &[&'encoder BindGroup],
        asset_manager: &'encoder AssetManager,
        world: &World,
        render_pass: &mut RenderPass<'encoder>,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(1, global_bind_groups[0], &[]);
        let query =
            <(Read<Transform>, Tagged<ModelHandle>)>::query().filter(component::<PointLight>());
        for chunk in query.par_iter_chunks(world) {
            let model_handle = chunk.tag::<ModelHandle>().unwrap();
            let model = asset_manager.get_model(model_handle).unwrap();
            let transforms = chunk.components::<Transform>().unwrap();
            render_pass.draw_model_instanced(model, 0..transforms.len() as u32)
        }
    }
}
