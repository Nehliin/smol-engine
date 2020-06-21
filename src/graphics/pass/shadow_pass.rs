use super::Pass;
use crate::{
    assets::{AssetManager, ModelHandle},
    components::Transform,
    graphics::{
        model::{DrawModel, InstanceData, MeshVertex},
        point_light::PointLightRaw,
        shadow_texture::{ShadowTexture, SHADOW_FORMAT},
        PointLight,
    },
};
use anyhow::Result;
use legion::prelude::World;
use legion::prelude::*;
use smol_renderer::{
    FragmentShader, GpuData, RenderNode, TextureData, UniformBindGroup, VertexShader,
};
use std::{collections::HashMap, rc::Rc, sync::Arc};
use wgpu::{BindGroup, BindGroupLayout, CommandBuffer, Device, ShaderStage};

#[repr(C)]
#[derive(Default, Clone, GpuData)]
pub struct LightSpaceMatrix {
    pub light_space_matrix: [[f32; 4]; 4],
}

impl From<&PointLightRaw> for LightSpaceMatrix {
    fn from(light: &PointLightRaw) -> Self {
        LightSpaceMatrix {
            light_space_matrix: light.light_space_matrix,
        }
    }
}

// TODO:
// Sample the shadow textures in the model pass (add them as an internal the bindgroup)
// modify the model_pass shaders
// Don't make the point light contian the target_view, that should be a separate component
// Update the resize method
pub struct ShadowPass {
    render_node: RenderNode,
    shadow_texture: Rc<TextureData<ShadowTexture>>,
}

impl ShadowPass {
    pub fn new(device: &Device, shadow_texture: Rc<TextureData<ShadowTexture>>) -> Result<Self> {
        let render_node = RenderNode::builder()
            .add_vertex_buffer::<MeshVertex>()
            .add_vertex_buffer::<InstanceData>()
            .set_vertex_shader(VertexShader::new(
                device,
                "src/shader_files/vs_shadow.shader",
            )?)
            .set_fragment_shader(FragmentShader::new(
                device,
                "src/shader_files/fs_shadow.shader",
            )?)
            // shadow texture
            .add_texture::<ShadowTexture>()
            .set_depth_stencil_state(wgpu::DepthStencilStateDescriptor {
                format: SHADOW_FORMAT,
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
                depth_bias: 0, // Biliniear filtering
                depth_bias_slope_scale: 2.0,
                depth_bias_clamp: 0.0,
            })
            .add_local_uniform_bind_group(
                UniformBindGroup::builder()
                    .add_binding::<LightSpaceMatrix>(ShaderStage::FRAGMENT)?
                    .build(device),
            )
            .build(&device)?;

        Ok(Self {
            render_node,
            shadow_texture,
        })
    }

    // This is very ugly it's probably better do decouple these
    // either use events to give new lights a view immedietly
    // or separate them completely in different components
    pub fn update_light_with_texture_view(&self, world: &mut World) {
        let light_query = <Write<PointLight>>::query();
        for (i, mut light) in light_query.iter_mut(world).enumerate() {
            if light.target_view.is_some() {
                continue;
            }
            light.target_view = Some(self.shadow_texture.create_new_view(
                &wgpu::TextureViewDescriptor {
                    format: SHADOW_FORMAT,
                    dimension: wgpu::TextureViewDimension::D2,
                    aspect: wgpu::TextureAspect::DepthOnly,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: i as u32,
                    array_layer_count: 1,
                },
            ));
        }
    }

    pub fn update_uniforms(
        &self,
        device: &Device,
        light: &PointLightRaw,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.render_node.update(device, encoder, 0, &light.into());
    }
}

impl Pass for ShadowPass {
    fn update_uniform_data(
        &self,
        world: &World,
        asset_manager: &AssetManager,
        device: &Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        todo!("Not used but should be")
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
