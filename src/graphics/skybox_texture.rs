use image::{DynamicImage, GenericImage};
use once_cell::sync::OnceCell;
use smol_renderer::{LoadableTexture, RenderError, TextureData, TextureShaderLayout};
use std::path::Path;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Binding, BindingResource, BindingType, CommandEncoderDescriptor, Device,
    Extent3d, FilterMode, Origin3d, Queue, Sampler, ShaderStage, TextureAspect,
    TextureComponentType, TextureCopyView, TextureDataLayout, TextureDescriptor, TextureDimension,
    TextureFormat, TextureView, TextureViewDescriptor, TextureViewDimension,
};

const REQUIRED_SKYBOX_TEXTURES: usize = 6;

pub struct SkyboxTexture {
    pub texture: wgpu::Texture,
    pub texture_view: TextureView,
    pub sampler: Sampler,
    pub bind_group: BindGroup, // the texture is the buffer in this case
}

impl TextureShaderLayout for SkyboxTexture {
    const VISIBILITY: ShaderStage = ShaderStage::FRAGMENT;
    fn get_layout(device: &Device) -> &'static BindGroupLayout {
        static LAYOUT: OnceCell<BindGroupLayout> = OnceCell::new();
        LAYOUT.get_or_init(|| {
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                bindings: &[
                    BindGroupLayoutEntry::new(
                        0,
                        Self::VISIBILITY,
                        BindingType::SampledTexture {
                            multisampled: false,
                            dimension: TextureViewDimension::Cube,
                            component_type: TextureComponentType::Float,
                        },
                    ),
                    BindGroupLayoutEntry::new(
                        1,
                        Self::VISIBILITY,
                        BindingType::Sampler { comparison: true },
                    ),
                ],
                label: Some("Skybox Texture layout"),
            })
        })
    }
}

impl LoadableTexture for SkyboxTexture {
    fn load_texture(
        device: &Device,
        queue: &Queue,
        dir_path: impl AsRef<Path>,
    ) -> Result<TextureData<Self>, RenderError> {
        let directory_iterator = std::fs::read_dir(dir_path.as_ref())?;

        let mut paths = directory_iterator
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| !path.is_dir())
            .collect::<Vec<_>>();

        assert!(
            paths.len() == REQUIRED_SKYBOX_TEXTURES,
            format!(
                "Skybox texture directory {:?} doesn't contain exacty 6 images",
                dir_path.as_ref()
            )
        );
        // sort the paths in order (skybox textures are ordered)
        paths.sort();
        let images = paths
            .iter()
            .map(image::open)
            .collect::<Result<Vec<DynamicImage>, _>>()?;

        let (width, height) = images.first().unwrap().dimensions();

        let texture_size = Extent3d {
            width,
            height,
            depth: REQUIRED_SKYBOX_TEXTURES as u32,
        };

        let (texture, texture_view, sampler) = Self::create_texture_data(device, texture_size);

        let mut command_encoder =
            device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        let texture_copy_view = TextureCopyView {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
        };

        let texture_data_layout = TextureDataLayout {
            offset: 0,
            bytes_per_row: 4 * width,
            rows_per_image: 0,
        };
        let texture_data = images
            .iter()
            .map(|img| img.to_rgba())
            .flat_map(|img_buffer| img_buffer.to_vec())
            .collect::<Vec<u8>>();

        queue.write_texture(
            texture_copy_view,
            &texture_data,
            texture_data_layout,
            texture_size,
        );

        let bind_group = Self::create_bind_group(device, &texture_view, &sampler);

        Ok(TextureData::new(
            bind_group,
            texture,
            vec![texture_view],
            sampler,
        ))
    }
}

impl SkyboxTexture {
    fn create_texture_data(
        device: &Device,
        texture_extent: Extent3d,
    ) -> (wgpu::Texture, TextureView, Sampler) {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Skybox Texture"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        let texture_view = texture.create_view(&TextureViewDescriptor {
            format: TextureFormat::Rgba8Unorm,
            dimension: TextureViewDimension::Cube,
            aspect: TextureAspect::default(),
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            array_layer_count: REQUIRED_SKYBOX_TEXTURES as u32,
            label: Some("Skybox texture view"),
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None,
            label: Some("Skybox Sampler"),
            ..Default::default()
        });

        (texture, texture_view, sampler)
    }

    fn create_bind_group(
        device: &Device,
        texture_view: &TextureView,
        sampler: &Sampler,
    ) -> BindGroup {
        let bind_group_layout = Self::get_layout(device);
        device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::TextureView(texture_view),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
            label: Some("Skybox bindgroup"),
        })
    }
}
