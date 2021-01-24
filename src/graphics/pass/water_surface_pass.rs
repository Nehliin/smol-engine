use std::sync::Arc;

use crate::graphics::Pass;
use crate::{
    assets::Assets,
    graphics::heightmap::{HeightMap, HeightMapModelMatrix, HeightMapVertex},
};
use crate::{assets::Handle, components::Transform};
use anyhow::Result;
use legion::prelude::*;
use smol_renderer::{FragmentShader, RenderNode, SimpleTexture, UniformBindGroup, VertexShader};
use wgpu::{CommandEncoder, CullMode, Device, RenderPassDescriptor, ShaderStage, TextureFormat};

// TODO WATER SURFACE PASS
// 1. create watersurface pass that from a plane uses a heightmap
// 2. load random height map and add reflection and refactoring from env map and skybox
pub struct WaterSurfacePass {
    render_node: RenderNode,
}

impl WaterSurfacePass {
    pub fn new(
        device: &Device,
        format: TextureFormat,
        global_uniforms: Vec<Arc<UniformBindGroup>>,
    ) -> Result<WaterSurfacePass> {
        let render_node = RenderNode::builder()
            .add_vertex_buffer::<HeightMapVertex>()
            .set_vertex_shader(VertexShader::new(
                device,
                "src/shader_files/vs_watersurface.shader",
            )?)
            .set_fragment_shader(FragmentShader::new(
                device,
                "src/shader_files/fs_watersurface.shader",
            )?)
            .set_rasterization_state(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: CullMode::None,
                ..Default::default()
             /*   depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,*/
            })
            .add_default_color_state_desc(format)
            .set_default_depth_stencil_state()
            // height map
            .add_texture::<SimpleTexture>()
            .add_local_uniform_bind_group(
                UniformBindGroup::builder()
                    .add_binding::<HeightMapModelMatrix>(ShaderStage::VERTEX)?
                    .build(device),
            )
            .add_shared_uniform_bind_group(global_uniforms[0].clone())
            .build(device)?;
        Ok(WaterSurfacePass { render_node })
    }
}

// TODO: the update and render can be written in to the same ECS query
impl Pass for WaterSurfacePass {
    fn update_uniform_data(
        &self,
        world: &World,
        _resources: &Resources,
        device: &Device,
        encoder: &mut CommandEncoder,
    ) {
        let query = <(Read<Transform>, Tagged<Handle<HeightMap>>)>::query();
        for (transform, _) in query.iter(world) {
            let model_matrix = HeightMapModelMatrix {
                model_matrix: transform.get_model_matrix(),
            };
            self.render_node
                .update(device, encoder, 0, &model_matrix)
                .unwrap();
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
            .get::<Assets<HeightMap>>()
            .expect("asset not registered");
        let mut runner = self.render_node.runner(encoder, render_pass_descriptor);
        let query = <(Read<Transform>, Tagged<Handle<HeightMap>>)>::query();
        for (_, handle) in query.iter(world) {
            let height_map = asset_storage.get(handle).unwrap();
            height_map.render(&mut runner);
        }
    }
}
