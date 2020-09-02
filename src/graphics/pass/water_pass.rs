use super::Pass;
use crate::{
    assets::{AssetManager, ModelHandle},
    components::Transform,
    graphics::{
        model::{DrawModel, InstanceData, MeshVertex},
        water_map::{WaterMap, WATERMAP_FORMAT},
        PointLight,
    },
};
use anyhow::Result;
use legion::prelude::*;
use nalgebra::{Matrix4, Point3, Vector3, Orthographic3};
use smol_renderer::{
    FragmentShader, GpuData, RenderNode, TextureData, UniformBindGroup, VertexShader,
};
use std::{collections::HashMap, rc::Rc};
use wgpu::{Device, ShaderStage};
use once_cell::sync::Lazy;

static ORTHOGRAPHIC_PROJECTION: Lazy<Orthographic3<f32>> =
    Lazy::new(|| Orthographic3::new(-10.0, 10.0, -10.0, 10.0, 1.0, 100.0));



// defines the water level and possibly more things in the future
// the water map is captured looking straight down towards origin
// from the water level
pub struct WaterResource {
    pub level: f32,
}

#[repr(C)]
#[derive(Default, Clone, GpuData)]
pub struct WaterSurfaceSpaceMatrix {
    pub matrix: [[f32; 4]; 4],
}

impl From<&WaterResource> for WaterSurfaceSpaceMatrix {
    fn from(water: &WaterResource) -> Self {
        let view = Matrix4::look_at_rh(
            &Point3::new(0.0, water.level, 0.0),
            &Point3::new(0.0, 0.0, 0.0),
            &Vector3::y(),
        );
        let water_space_matrix = ORTHOGRAPHIC_PROJECTION.to_homogeneous() * view;
        let projection = dbg!(water_space_matrix)
            .as_slice()
            .chunks(4)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
            .collect::<Vec<[f32; 4]>>();
        WaterSurfaceSpaceMatrix {
            matrix: [projection[0], projection[1], projection[1], projection[3]],
        }
    }
}

pub struct WaterPass {
    render_node: RenderNode,
    water_map: Rc<TextureData<WaterMap>>,
    pub water_map_view: wgpu::TextureView,
}

impl WaterPass {
    pub fn new(device: &Device, water_map: Rc<TextureData<WaterMap>>) -> Result<WaterPass> {
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
            .set_depth_stencil_state(wgpu::DepthStencilStateDescriptor {
                format: WATERMAP_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            })
            .set_rasterization_state(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Front,
                depth_bias: 0,
                depth_bias_slope_scale: 2.0,
                depth_bias_clamp: 0.0,
            })
            .add_local_uniform_bind_group(
                UniformBindGroup::with_name("Water surface matrix")
                    .add_binding::<WaterSurfaceSpaceMatrix>(ShaderStage::VERTEX)?
                    .build(device),
            )
            .build(&device)?;

        let water_map_view = water_map.create_new_view(&wgpu::TextureViewDescriptor {
            label: Some("water map view"),
            format: WATERMAP_FORMAT,
            dimension: wgpu::TextureViewDimension::D2,
            aspect: wgpu::TextureAspect::DepthOnly,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            array_layer_count: 1,
        });

        Ok(WaterPass {
            render_node,
            water_map,
            water_map_view,
        })
    }

    pub fn update_uniforms(
        &self,
        device: &Device,
        water: &WaterResource,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let water_space_matrix: WaterSurfaceSpaceMatrix = water.into();
        self.render_node
            .update(device, encoder, 0, &water_space_matrix)
            .unwrap();
    }
}

impl Pass for WaterPass {
    fn update_uniform_data(
        &self,
        _world: &World,
        _asset_manager: &AssetManager,
        _device: &Device,
        _encoder: &mut wgpu::CommandEncoder,
    ) {
        todo!("not used but should be")
    }

    fn render<'encoder>(
        &'encoder self,
        asset_manager: &'encoder AssetManager,
        world: &World,
        encoder: &mut wgpu::CommandEncoder,
        render_pass_descriptor: wgpu::RenderPassDescriptor,
    ) {
        let mut runner = self.render_node.runner(encoder, render_pass_descriptor);

        let mut offset_map = HashMap::new();
        let query =
            <(Read<Transform>, Tagged<ModelHandle>)>::query().filter(!component::<PointLight>());
        for chunk in query.iter_chunks(world) {
            // This is guaranteed to be the same for each chunk
            let model = chunk.tag::<ModelHandle>().unwrap();
            let offset = *offset_map.get(model).unwrap_or(&0);
            let transforms = chunk.components::<Transform>().unwrap();
            offset_map.insert(model.clone(), offset + transforms.len());
            let model = asset_manager.get_model(model).unwrap();
            runner.draw_untextured(model, offset as u32..(offset + transforms.len()) as u32);
        }
    }
}
