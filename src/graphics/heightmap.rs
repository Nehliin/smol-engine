use crate::assets::AssetLoader;
use anyhow::Result;
use nalgebra::Matrix4;
use smol_renderer::{
    GpuData, ImmutableVertexData, LoadableTexture, RenderNodeRunner, SimpleTexture, TextureData,
    VertexBuffer,
};
use std::path::Path;
use std::path::PathBuf;
use wgpu::{Buffer, BufferUsage, Device, Queue, VertexAttributeDescriptor, VertexFormat};

#[repr(C)]
#[derive(GpuData, Debug)]
// todo: Add UVs and normals?
pub struct HeightMapVertex {
    position: [f32; 3],
}

impl VertexBuffer for HeightMapVertex {
    const STEP_MODE: wgpu::InputStepMode = wgpu::InputStepMode::Vertex;

    fn get_attributes<'a>() -> &'a [wgpu::VertexAttributeDescriptor] {
        &[VertexAttributeDescriptor {
            offset: 0,
            format: VertexFormat::Float3,
            shader_location: 0,
        }]
    }
}
// No instance buffer, use uniform instead
pub struct HeightMap {
    pub vertex_buffer: ImmutableVertexData<HeightMapVertex>,
    pub index_buffer: Buffer,
    pub number_of_indices: u32,
    pub height_map: TextureData<SimpleTexture>,
}

// This is the data layout for the model matrix bindgroup that's part of the render pass
// It's not actually owned by the height map itself and is generated from the transform component
#[repr(C)]
#[derive(GpuData, Debug, Clone, Default)]
pub struct HeightMapModelMatrix {
    pub model_matrix: Matrix4<f32>,
}

const WIDTH_SEGMENTS: f32 = 512.0;
const HEIGHT_SEGMENTS: f32 = 512.0;
const WIDTH: f32 = 10.0;
const HEIGHT: f32 = 10.0;

impl HeightMap {
    pub fn load(device: &Device, queue: &Queue, path: impl AsRef<Path>) -> Result<HeightMap> {
        let height_map = SimpleTexture::load_texture(device, queue, path)?;
        // algorithm from Three.js plane buffer geometry
        let half_width = WIDTH / 2.0;
        let half_height = HEIGHT / 2.0;

        let grid_x = WIDTH_SEGMENTS.floor() as u32;
        let grid_y = HEIGHT_SEGMENTS.floor() as u32;

        let grid_x1 = grid_x + 1;
        let grid_y1 = grid_y + 1;

        let segment_width = WIDTH / grid_x as f32;
        let segment_height = HEIGHT / grid_y as f32;

        let mut vertices = Vec::with_capacity((grid_x1 * grid_y1) as usize);

        for i in 0..grid_y1 {
            let y = i as f32 * segment_height - half_height;
            for j in 0..grid_x1 {
                let x = j as f32 * segment_width - half_width;
                vertices.push(HeightMapVertex {
                    position: [x, -y, 0.0],
                });
            }
        }
        // indices
        let mut indices = Vec::with_capacity((grid_x * grid_y) as usize);
        for i in 0..grid_y {
            for j in 0..grid_x {
                let a = j + grid_x1 * i;
                let b = j + grid_x1 * (i + 1);
                let c = (j + 1) + grid_x1 * (i + 1);
                let d = (j + 1) + grid_x1 * i;

                indices.push(a);
                indices.push(b);
                indices.push(d);

                indices.push(b);
                indices.push(c);
                indices.push(d);
            }
        }
        let number_of_indices = indices.len() as u32;
        let indices =
            unsafe { std::slice::from_raw_parts(indices.as_ptr() as *const u8, indices.len() * 4) };

        let index_buffer = device.create_buffer_with_data(&indices, BufferUsage::INDEX);

        let vertex_buffer = VertexBuffer::allocate_immutable_buffer(device, &vertices);

        Ok(HeightMap {
            height_map,
            vertex_buffer,
            index_buffer,
            number_of_indices,
        })
    }

    pub fn render<'a, 'b>(&'b self, runner: &mut RenderNodeRunner<'a, 'b>) {
        runner.set_texture_data(0, &self.height_map);
        runner.set_vertex_buffer_data(0, &self.vertex_buffer);
        runner.set_index_buffer(self.index_buffer.slice(..));
        runner.draw_indexed(0..self.number_of_indices, 0, 0..1);
    }
}

impl AssetLoader for HeightMap {
    fn load(path: &PathBuf, device: &Device, queue: &Queue) -> Result<HeightMap> {
        HeightMap::load(device, queue, path)
    }

    fn extension() -> &'static str {
        "png"
    }
}
