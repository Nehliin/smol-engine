use super::PointLight;
use crate::graphics::point_light::PointLightRaw;
use nalgebra::{Matrix4, Vector3};
use std::marker::PhantomData;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Binding, BindingResource, BindingType, Buffer, BufferAddress,
    BufferDescriptor, BufferUsage, CommandEncoder, Device, ShaderStage,
};
use zerocopy::AsBytes;
use zerocopy::FromBytes;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UniformCameraData {
    pub view_matrix: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    pub view_pos: Vector3<f32>,
}
#[repr(C)]
#[derive(AsBytes, Default, FromBytes)]
pub struct CameraDataRaw {
    pub view_matrix: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub view_pos: [f32; 3],
}

impl From<UniformCameraData> for CameraDataRaw {
    fn from(data: UniformCameraData) -> Self {
        let test = data.view_matrix.as_slice();
        let view_matrix = test
            .chunks(4)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
            .collect::<Vec<[f32; 4]>>();
        let projection = data
            .projection
            .as_slice()
            .chunks(4)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
            .collect::<Vec<[f32; 4]>>();
        Self {
            view_matrix: [
                view_matrix[0],
                view_matrix[1],
                view_matrix[2],
                view_matrix[3],
            ],
            projection: [projection[0], projection[1], projection[2], projection[3]],
            view_pos: [data.view_pos.x, data.view_pos.y, data.view_pos.z],
        }
    }
}

impl Default for UniformCameraData {
    fn default() -> Self {
        Self {
            view_pos: Vector3::identity(),
            projection: Matrix4::identity(),
            view_matrix: Matrix4::identity(),
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, AsBytes)]
pub struct LightUniforms {
    pub lights_used: i32,
    pub pad: [i32; 3],
    pub(crate) point_lights: [PointLightRaw; 16],
}

#[repr(C)]
#[derive(Debug, AsBytes, Default)]
pub struct LightSpaceMatrix {
    pub light_space_matrix: [[f32; 4]; 4],
}

impl From<&PointLightRaw> for LightSpaceMatrix {
    fn from(light: &PointLightRaw) -> Self {
        Self {
            light_space_matrix: light.light_space_matrix,
        }
    }
}

pub struct UniformBindGroup<T> {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
    _marker: PhantomData<T>,
}

impl<T: AsBytes> UniformBindGroup<T> {
    pub fn new(device: &Device, visibility: ShaderStage) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Uniform buffer"),
            size: std::mem::size_of::<T>() as u64,
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                // This is the layout of the uniform buffer
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
            ],
            label: Some("Uniform Bind group layout"),
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[Binding {
                binding: 0,
                resource: BindingResource::Buffer {
                    buffer: &buffer,
                    range: 0..std::mem::size_of::<T>() as BufferAddress,
                },
            }],
            label: Some("Uniform bind group"),
        });
        Self {
            bind_group,
            buffer,
            bind_group_layout,
            _marker: PhantomData::default(),
        }
    }

    pub fn update(&self, device: &Device, data: &T, encoder: &mut CommandEncoder) {
        let staging_buffer = device.create_buffer_with_data(data.as_bytes(), BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.buffer,
            0,
            std::mem::size_of::<T>() as BufferAddress,
        );
    }
}
