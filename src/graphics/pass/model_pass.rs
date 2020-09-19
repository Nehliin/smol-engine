use std::{collections::HashMap, rc::Rc, sync::Arc};

use anyhow::Result;
use legion::prelude::*;
use nalgebra::Vector3;
use smol_renderer::{
    FragmentShader, GpuData, RenderNode, SimpleTexture, TextureData, UniformBindGroup, VertexShader,
};
use wgpu::{CommandEncoder, Device, RenderPassDescriptor, ShaderStage, TextureFormat};

use crate::{
    assets::Assets,
    assets::Handle,
    graphics::{model::Model, Pass},
};
use crate::{
    components::Transform,
    graphics::PointLight,
    graphics::{model::MeshVertex, shadow_texture::ShadowTexture},
    graphics::{
        model::{DrawModel, InstanceData},
        point_light::PointLightRaw,
    },
};

pub struct ModelPass {
    //todo: maybe solve in another way instead of Rc (weak ptr)?
    shadow_texture: Rc<TextureData<ShadowTexture>>,
    render_node: RenderNode,
}

pub const MAX_POINT_LIGHTS: u32 = 16;

#[repr(C)]
#[derive(Debug, GpuData, Clone)]
pub struct PointLightsUniforms {
    lights_used: i32,
    _pad: [i32; 3],
    lights: [PointLightRaw; MAX_POINT_LIGHTS as usize],
}

impl ModelPass {
    pub fn new(
        device: &Device,
        global_uniforms: Vec<Arc<UniformBindGroup>>,
        shadow_texture: Rc<TextureData<ShadowTexture>>,
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
            .add_default_color_state_desc(color_format)
            .set_default_depth_stencil_state()
            .set_default_rasterization_state()
            .add_local_uniform_bind_group(
                UniformBindGroup::with_name("Point light uniform")
                    .add_binding::<PointLightsUniforms>(ShaderStage::FRAGMENT)?
                    .build(device),
            )
            .add_shared_uniform_bind_group(global_uniforms[0].clone())
            .build(&device)?;

        Ok(Self {
            render_node,
            shadow_texture,
        })
    }

    pub fn update_lights(&self, device: &Device, world: &World, encoder: &mut CommandEncoder) {
        let query = <(Read<PointLight>, Read<Transform>)>::query();
        // TODO: only runs once unecessary loop
        for chunk in query.par_iter_chunks(world) {
            let lights = chunk.components::<PointLight>().unwrap();
            let positions = chunk.components::<Transform>().unwrap();
            let mut uniform_data =
                [PointLightRaw::from((&PointLight::default(), Vector3::new(0.0, 0.0, 0.0)));
                    MAX_POINT_LIGHTS as usize];
            let mut lights_used = 0;
            lights
                .iter()
                .zip(positions.iter())
                .enumerate()
                .for_each(|(i, (light, pos))| {
                    uniform_data[i] = PointLightRaw::from((light, pos.translation()));
                    lights_used += 1;
                });
            let data = &PointLightsUniforms {
                lights_used,
                _pad: [0; 3],
                lights: uniform_data,
            };
            self.render_node.update(device, encoder, 0, data).unwrap();
        }
    }
}

impl Pass for ModelPass {
    fn update_uniform_data(
        &self,
        world: &World,
        resources: &Resources,
        device: &Device,
        encoder: &mut CommandEncoder,
    ) {
        self.update_lights(device, world, encoder);
        let asset_storage = resources
            .get::<Assets<Model>>()
            .expect("Asset not registerd");
        let mut offsets = HashMap::new();
        let query = <(Read<Transform>, Tagged<Handle<Model>>)>::query();
        for chunk in query.par_iter_chunks(world) {
            let model = chunk.tag::<Handle<Model>>().unwrap();
            let transforms = chunk.components::<Transform>().unwrap();
            let model_matrices = transforms
                .iter()
                .map(|trans| InstanceData::new(trans.get_model_matrix()))
                .collect::<Vec<InstanceData>>();
            let offset = *offsets.get(model).unwrap_or(&0);
            let instance_buffer = &asset_storage.get(model).unwrap().instance_buffer;
            instance_buffer.update(device, encoder, &model_matrices);
            offsets.insert(model.clone(), offset + model_matrices.len() as u64);
        }
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
            .expect("Asset not registerd");
        let mut runner = self.render_node.runner(encoder, render_pass_descriptor);
        runner.set_texture_data(2, &self.shadow_texture);
        let mut offset_map = HashMap::new();
        let query =
            <(Read<Transform>, Tagged<Handle<Model>>)>::query().filter(!component::<PointLight>());
        for chunk in query.par_iter_chunks(world) {
            // This is guaranteed to be the same for each chunk
            let model = chunk.tag::<Handle<Model>>().unwrap();
            let offset = *offset_map.get(model).unwrap_or(&0);
            let transforms = chunk.components::<Transform>().unwrap();
            offset_map.insert(model.clone(), offset + transforms.len());
            let model = asset_storage.get(model).unwrap();
            runner.draw_model_instanced(model, offset as u32..(offset + transforms.len()) as u32);
        }
    }
}
