use anyhow::Result;
use image::{DynamicImage, GenericImage};
use std::path::Path;
use wgpu::{
    AddressMode, BufferCopyView, BufferUsage, CommandBuffer, CommandEncoderDescriptor,
    CompareFunction, Device, Extent3d, FilterMode, Origin3d, Sampler, SamplerDescriptor,
    TextureCopyView, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage, TextureView,
};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler,
}

impl Texture {
    pub fn load(device: &Device, path: impl AsRef<Path>) -> Result<(Self, CommandBuffer)> {
        let img = image::open(path)?;
        let img = img.flipv();
        Texture::from_image(device, &img)
    }

    pub fn from_image(device: &Device, img: &DynamicImage) -> Result<(Self, CommandBuffer)> {
        let rgba = img.to_rgba(); // handle formats properly
        let (width, height) = img.dimensions();

        let size = Extent3d {
            width,
            height,
            depth: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb, // handle formats properly
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        });
        // Generate buffer + Bindbuffer + fill it with data
        let buffer = device.create_buffer_with_data(&rgba.to_vec(), BufferUsage::COPY_SRC);

        let mut command_encoder =
            device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        // Encode a command that sends the data to the gpu so it can be bound to the texture in the shaders
        command_encoder.copy_buffer_to_texture(
            BufferCopyView {
                buffer: &buffer,
                offset: 0,
                bytes_per_row: 4 * width,
                rows_per_image: 0,
            },
            TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: Origin3d::ZERO,
            },
            size,
        );
        // final buffer of the commands needed to send the texture to the GPU
        // So it can be used in the shaders
        let command_buffer = command_encoder.finish();

        let view = texture.create_default_view();
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: -100.0, // related to mipmaps
            lod_max_clamp: 100.0,  // related to mipmaps
            compare: CompareFunction::Always,
        });

        Ok((
            Self {
                texture,
                view,
                sampler,
            },
            command_buffer,
        ))
    }
}
