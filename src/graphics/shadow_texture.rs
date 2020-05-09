use super::lighting::{DirectionalLight, PointLight};
use legion::prelude::*;
use once_cell::sync::OnceCell;
use wgpu::Device;

pub const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: 2048,
    height: 2048,
    depth: 1,
};

pub struct ShadowTexture {
    texture: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl ShadowTexture {
    pub fn new(device: &Device) -> Self {
        // This is a texture array where each light gets its own layer
        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow map texture"),
            size: SHADOW_SIZE,
            array_layer_count: 16, // <- MAX LIGHTS
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2, // create cube texture for omnidirectional shadows
            format: SHADOW_FORMAT,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        });
        let shadow_view = shadow_texture.create_default_view();
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::LessEqual,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: Self::get_or_create_texture_layout(device),
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadow_view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Shadow texture bindgroup"),
        });

        Self {
            texture: shadow_texture,
            bind_group,
        }
    }

    pub fn update_lights_with_texture_view(&self, world: &mut World) {
        let light_query = <Write<DirectionalLight>>::query();
        for (i, mut light) in light_query.iter_mut(world).enumerate() {
            if light.target_view.is_some() {
                continue;
            }
            light.target_view = Some(self.texture.create_view(&wgpu::TextureViewDescriptor {
                format: SHADOW_FORMAT,
                dimension: wgpu::TextureViewDimension::D2,
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: i as u32,
                array_layer_count: 1,
            }));
        }
    }

    pub fn get_or_create_texture_layout(device: &Device) -> &wgpu::BindGroupLayout {
        static LAYOUT: OnceCell<wgpu::BindGroupLayout> = OnceCell::new();
        LAYOUT.get_or_init(|| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2Array,
                            component_type: wgpu::TextureComponentType::Float,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: true },
                    },
                ],
                label: Some("Shadow Texture layout"),
            })
        })
    }
}
