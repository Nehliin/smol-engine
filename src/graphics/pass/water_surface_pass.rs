use crate::assets::AssetManager;
use crate::components::Transform;
use crate::graphics::heightmap::{HeightMap, HeightMapModelMatrix, HeightMapVertex};
use crate::graphics::Pass;
use anyhow::Result;
use legion::prelude::*;
use smol_renderer::{
    FragmentShader, RenderNode, SimpleTexture, TextureData, UniformBindGroup, VertexShader,
};
use wgpu::{CommandEncoder, Device, RenderPassDescriptor, ShaderStage};

// TODO WATER SURFACE PASS
// 1. create watersurface pass that from a plane uses a heightmap
// 2. load random height map and add reflection and refactoring from env map and skybox
pub struct WaterSurfacePass {
    render_node: RenderNode,
}

impl WaterSurfacePass {
    pub fn new(device: &Device) -> Result<WaterSurfacePass> {
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
            .set_default_rasterization_state()
            .set_default_depth_stencil_state()
            // height map
            .add_texture::<SimpleTexture>()
            .add_local_uniform_bind_group(
                UniformBindGroup::builder()
                    .add_binding::<HeightMapModelMatrix>(ShaderStage::VERTEX)?
                    .build(device),
            )
            .build(device)?;
        Ok(WaterSurfacePass { render_node })
    }
}

// TODO: the update and render can be written in to the same ECS query
impl Pass for WaterSurfacePass {
    fn update_uniform_data(
        &self,
        world: &World,
        _asset_manager: &AssetManager,
        device: &Device,
        encoder: &mut CommandEncoder,
    ) {
        let query = <(Read<Transform>, Read<HeightMap>)>::query();
        // Heightmap isn't send + sync....
        for (transform, _) in query.par_iter(world) {
            let model_matrix = transform.get_model_matrix();
            self.render_node
                .update(device, encoder, 1, &model_matrix)
                .unwrap();
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
        let query = <(Read<Transform>, Read<HeightMap>)>::query();
        for (_, height_map) in query.par_iter(world) {
            height_map.render(&mut runner);
        }
    }
}
