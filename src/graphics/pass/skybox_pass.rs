use super::Pass;
use crate::graphics::{skybox_texture::SkyboxTexture, Shader};
use glsl_to_spirv::ShaderType;
use wgpu::{
    BindGroupLayout, BlendDescriptor, ColorStateDescriptor, ColorWrite, CullMode, Device,
    FrontFace, IndexFormat, PipelineLayoutDescriptor, PrimitiveTopology,
    RasterizationStateDescriptor, RenderPipeline, RenderPipelineDescriptor, TextureFormat,
    VertexStateDescriptor,
};

pub struct SkyboxPass {
    render_pipeline: RenderPipeline,
    skybox_texture: SkyboxTexture,
}

impl SkyboxPass {
    pub fn new(
        device: &Device,
        incoming_layouts: Vec<&BindGroupLayout>,
        color_format: TextureFormat,
        skybox_texture: SkyboxTexture,
    ) -> Self {
        let vs_shader = Shader::new(
            &device,
            "src/shader_files/vs_skybox.shader",
            ShaderType::Vertex,
        )
        .unwrap();
        let fs_shader = Shader::new(
            &device,
            "src/shader_files/fs_skybox.shader",
            ShaderType::Fragment,
        )
        .unwrap();

        // cubemap texture bindgroups are part of incoming layouts
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &incoming_layouts,
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: vs_shader.get_descriptor(),
            fragment_stage: Some(fs_shader.get_descriptor()),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Cw,
                cull_mode: CullMode::None,
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
            depth_stencil_state: None,
            // The vertex data is hardcoded in the shader so
            // that's why there aren't any vertex_buffers added here
            // and why the index format is smaller
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            render_pipeline,
            skybox_texture,
        }
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
        global_bind_groups: &[&'encoder wgpu::BindGroup],
        _asset_manager: &'encoder crate::assets::AssetManager,
        _world: &legion::prelude::World,
        render_pass: &mut wgpu::RenderPass<'encoder>,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.skybox_texture.bind_group, &[]);
        render_pass.set_bind_group(1, global_bind_groups[0], &[]);
        // all vertex data is hardcoded into the shaders
        render_pass.draw(0..3 as u32, 0..1);
    }
}
