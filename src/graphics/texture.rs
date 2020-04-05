use anyhow::Result;
use image::{DynamicImage, GenericImage};
use wgpu::{
    AddressMode, BufferCopyView, BufferUsage, CommandBuffer, CompareFunction, Device, Extent3d,
    FilterMode, Origin3d, Sampler, SamplerDescriptor, TextureCopyView, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsage, TextureView,
};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler,
}

impl Texture {
    pub fn from_bytes(device: &Device, bytes: &[u8]) -> Result<(Self, CommandBuffer)> {
        let img = image::load_from_memory(bytes)?;
        //P  let img = img.flipv();
        Texture::from_image(device, &img)
    }

    pub fn from_image(device: &Device, img: &DynamicImage) -> Result<(Self, CommandBuffer)> {
        let rgba = img.as_rgba8().unwrap(); // handle formats properly
        let (width, height) = img.dimensions();

        let size = Extent3d {
            width,
            height,
            depth: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb, // handle formats properly
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        });
        // Generate buffer + Bindbuffer + fill it with data
        let buffer = device
            .create_buffer_mapped(rgba.len(), BufferUsage::COPY_SRC)
            .fill_from_slice(&rgba);

        let mut command_encoder = device.create_command_encoder(&Default::default());
        // Encode a command that sends the data to the gpu so it can be bound to the texture in the shaders
        command_encoder.copy_buffer_to_texture(
            BufferCopyView {
                buffer: &buffer,
                offset: 0,
                row_pitch: 4 * width, // each pixel stored as 4 bytes?
                image_height: height,
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
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: -100.0, // related to mipmaps
            lod_max_clamp: 100.0,  // related to mipmaps
            compare_function: CompareFunction::Always,
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
