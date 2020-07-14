use super::pass::model_pass::MAX_POINT_LIGHTS;
use once_cell::sync::OnceCell;
use smol_renderer::{Texture, TextureData, TextureShaderLayout};
use wgpu::Device;

pub const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: 1024,
    height: 1024,
    depth: MAX_POINT_LIGHTS,
};

pub struct ShadowCubeTexture;

impl TextureShaderLayout for ShadowCubeTexture {
    const VISIBILITY: wgpu::ShaderStage = wgpu::ShaderStage::FRAGMENT;
    fn get_layout(device: &wgpu::Device) -> &'static wgpu::BindGroupLayout {
        static LAYOUT: OnceCell<wgpu::BindGroupLayout> = OnceCell::new();
        LAYOUT.get_or_init(|| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry::new(
                        0,
                        Self::VISIBILITY,
                        wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::CubeArray,
                            component_type: wgpu::TextureComponentType::Float,
                        },
                    ),
                    wgpu::BindGroupLayoutEntry::new(
                        1,
                        Self::VISIBILITY,
                        wgpu::BindingType::Sampler { comparison: true },
                    ),
                ],
                label: Some("Shadow Cube Texture layout"),
            })
        })
    }
}

impl Texture for ShadowCubeTexture {
    fn allocate_texture(device: &Device) -> TextureData<Self>
    where
        Self: TextureShaderLayout,
    {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow cube texture"),
            size: SHADOW_SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SHADOW_FORMAT,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        });

        let view = texture.create_default_view();
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow cube sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: Self::get_layout(device),
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Shadow texture bindgroup"),
        });
        TextureData::new(bind_group, texture, vec![view], sampler)
    }
}
