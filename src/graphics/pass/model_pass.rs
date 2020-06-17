use crate::assets::{AssetManager, ModelHandle};
use crate::graphics::Pass;
use crate::{
    components::Transform,
    graphics::wgpu_renderer::DEPTH_FORMAT,
    graphics::PointLight,
    graphics::{model::MeshVertex, shadow_texture::ShadowTexture},
    graphics::{
        model::{DrawModel, InstanceData},
        point_light::PointLightRaw,
    },
};
use anyhow::Result;
use glsl_to_spirv::ShaderType;
use legion::prelude::*;
use nalgebra::{Matrix4, Vector3};
use smol_renderer::{
    FragmentShader, GpuData, RenderNode, SimpleTexture, UniformBindGroup, VertexShader,
};
use std::{collections::HashMap, sync::Arc};
use wgpu::{
    BindGroup, BindGroupLayout, BlendDescriptor, BufferUsage, ColorStateDescriptor, ColorWrite,
    CommandEncoder, CullMode, Device, FrontFace, IndexFormat, PipelineLayoutDescriptor,
    PrimitiveTopology, RasterizationStateDescriptor, RenderPass, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderStage, TextureFormat, VertexStateDescriptor,
};

pub struct ModelPass {
    render_node: RenderNode,
}

pub const MAX_POINT_LIGHTS: u32 = 16;
#[derive(Debug, Clone)]
pub struct PointLightsUniforms {
    lights_used: i32,
    _pad: [i32; 3],
    lights: [PointLightRaw; MAX_POINT_LIGHTS as usize],
}

unsafe impl GpuData for PointLightsUniforms {
    fn as_raw_bytes(&self) -> &[u8] {
        // THIS MIGHT BE FUCKED 
        let total_size = std::mem::size_of::<i32>() * 4
            + std::mem::size_of::<PointLightRaw>() * MAX_POINT_LIGHTS;
        unsafe {
            std::slice::from_raw_parts(
                smol_renderer as *const Self as *const u8,
                total_size,
            )
        }
    }
}

impl ModelPass {
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
                "src/shader_files/vs_model.shader",
            )?)
            .set_fragment_shader(FragmentShader::new(
                device,
                "src/shader_files/fs_model.shader",
            )?)
            // diffuse
            .add_texture::<SimpleTexture>()
            // specular
            .add_texture::<SimpleTexture>()
            // shadow texture
            .add_texture::<ShadowTexture>()
            .set_default_depth_stencil_state()
            .set_default_rasterization_state()
            .add_shared_uniform_bind_group(global_uniforms[0])
            .add_local_uniform_bind_group(
                UniformBindGroup::builder()
                    .add_binding::<PointLightsUniforms>(ShaderStage::FRAGMENT)?
                    .build(device),
            )
            //.attach_global_uniform_bind_group(uniform)
            .build(&device, color_format)?;

        Ok(Self { render_node })
    }

    pub fn update_lights(&self, world: &World, encoder: &mut CommandEncoder) {
        let query = <(Read<PointLight>, Read<Transform>)>::query();
        // TODO: only runs once unecessary loop
        for chunk in query.par_iter_chunks(world) {
            let lights = chunk.components::<PointLight>().unwrap();
            let positions = chunk.components::<Transform>().unwrap();
            let mut uniform_data =
                [PointLightRaw::from((&PointLight::default(), Vector3::new(0.0, 0.0, 0.0))); MAX_POINT_LIGHTS];
            let mut lights_used = 0;
            lights
                .iter()
                .zip(positions.iter())
                .enumerate()
                .for_each(|(i, (light, pos))| {
                    uniform_data[i] = PointLightRaw::from((light, pos.translation()));
                    lights_used += 1;
                });
            self.render_node.update(
                &self.device,
                encoder,
                1,
                &PointLightsUniforms {
                    lights_used,
                    _pad: [0; 3],
                    lights: uniform_data,
                },
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
        self.update_lights(world, encoder);

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
        asset_manager: &'encoder AssetManager,
        world: &World,
        encoder: &mut CommandEncoder,
        render_pass_descriptor: RenderPassDescriptor,
    ) {
        let mut runner = self.render_node.runner(encoder, render_pass_descriptor);

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
            runner.draw_model_instanced(model, offset as u32..(offset + transforms.len()) as u32);
        }
    }
}
