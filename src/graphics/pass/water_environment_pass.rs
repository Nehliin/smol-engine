use super::{shadow_pass::LightSpaceMatrix, Pass};
use crate::{
    assets::Assets,
    assets::Handle,
    components::Transform,
    graphics::model::Model,
    graphics::{
        model::{DrawModel, InstanceData, MeshVertex},
        point_light::PointLightRaw,
        water_map::{WaterEnviornmentMap, WATERMAP_FORMAT},
        PointLight,
    },
};
use anyhow::Result;
use legion::prelude::*;
use smol_renderer::{FragmentShader, RenderNode, TextureData, UniformBindGroup, VertexShader};
use std::{collections::HashMap, rc::Rc};
use wgpu::{Device, ShaderStage};

pub struct WaterEnvironmentPass {
    render_node: RenderNode,
    water_map: Rc<TextureData<WaterEnviornmentMap>>,
    pub water_map_view: wgpu::TextureView,
}

impl WaterEnvironmentPass {
    pub fn new(
        device: &Device,
        water_map: Rc<TextureData<WaterEnviornmentMap>>,
    ) -> Result<WaterEnvironmentPass> {
        let render_node = RenderNode::builder()
            .add_vertex_buffer::<MeshVertex>()
            .add_vertex_buffer::<InstanceData>()
            .set_vertex_shader(VertexShader::new(
                device,
                "src/shader_files/vs_watermap.shader",
            )?)
            .set_fragment_shader(FragmentShader::new(
                device,
                "src/shader_files/fs_watermap.shader",
            )?)
            // This previously Culled front instead of back
            .set_default_rasterization_state()
            .add_default_color_state_desc(WATERMAP_FORMAT)
            .add_local_uniform_bind_group(
                UniformBindGroup::with_name("Water surface matrix")
                    .add_binding::<LightSpaceMatrix>(ShaderStage::VERTEX)?
                    .build(device),
            )
            .build(&device)?;

        let water_map_view = water_map.create_new_view(&wgpu::TextureViewDescriptor {
            label: Some("water map view"),
            format: WATERMAP_FORMAT,
            dimension: wgpu::TextureViewDimension::D2,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            array_layer_count: 1,
        });

        Ok(WaterEnvironmentPass {
            render_node,
            water_map,
            water_map_view,
        })
    }

    pub fn update_uniforms(
        &self,
        device: &Device,
        light: &PointLightRaw,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let light_space_matrix: LightSpaceMatrix = light.into();
        self.render_node
            .update(device, encoder, 0, &light_space_matrix)
            .unwrap();
    }
}

impl Pass for WaterEnvironmentPass {
    fn update_uniform_data(
        &self,
        _world: &World,
        _resources: &Resources,
        _device: &Device,
        _encoder: &mut wgpu::CommandEncoder,
    ) {
        todo!("not used but should be")
    }

    fn render<'encoder>(
        &'encoder self,
        resources: &'encoder Resources,
        world: &World,
        encoder: &mut wgpu::CommandEncoder,
        render_pass_descriptor: wgpu::RenderPassDescriptor,
    ) {
        let asset_storage = resources
            .get::<Assets<Model>>()
            .expect("Asset not registered");
        let mut runner = self.render_node.runner(encoder, render_pass_descriptor);
        let mut offset_map = HashMap::new();
        let query =
            <(Read<Transform>, Tagged<Handle<Model>>)>::query().filter(!component::<PointLight>());
        for chunk in query.iter_chunks(world) {
            // This is guaranteed to be the same for each chunk
            let handle = chunk.tag::<Handle<Model>>().unwrap();
            let offset = *offset_map.get(handle).unwrap_or(&0);
            let transforms = chunk.components::<Transform>().unwrap();
            offset_map.insert(handle.clone(), offset + transforms.len());
            let model = asset_storage.get(handle).unwrap();
            runner.draw_untextured(model, offset as u32..(offset + transforms.len()) as u32);
        }
    }
}
