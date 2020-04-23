use crate::graphics::pass::VBDesc;
use crate::graphics::texture::Texture;
use anyhow::Result;
use nalgebra::{Matrix4, Vector3};
use once_cell::sync::OnceCell;
use std::ops::Range;
use std::path::Path;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Binding, BindingResource, BindingType, Buffer, BufferAddress,
    BufferDescriptor, BufferUsage, CommandBuffer, Device, InputStepMode, ShaderStage,
    TextureComponentType, TextureViewDimension, VertexAttributeDescriptor, VertexBufferDescriptor,
    VertexFormat,
};
use zerocopy::AsBytes;

const INDEX_BUFFER_SIZE: u64 = 16_000;

#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes)]
pub struct MeshVertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coords: [f32; 2],
}

impl VBDesc for MeshVertex {
    fn desc<'a>() -> VertexBufferDescriptor<'a> {
        VertexBufferDescriptor {
            stride: std::mem::size_of::<MeshVertex>() as BufferAddress,
            step_mode: InputStepMode::Vertex,
            attributes: &[
                VertexAttributeDescriptor {
                    offset: 0,
                    format: VertexFormat::Float3,
                    shader_location: 0,
                },
                VertexAttributeDescriptor {
                    offset: std::mem::size_of::<Vector3<f32>>() as BufferAddress,
                    format: VertexFormat::Float3,
                    shader_location: 1,
                },
                VertexAttributeDescriptor {
                    offset: (std::mem::size_of::<Vector3<f32>>() * 2) as BufferAddress,
                    format: VertexFormat::Float2,
                    shader_location: 2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct InstanceData {
    model_matrix: Matrix4<f32>,
}
const ROW_SIZE: BufferAddress = (std::mem::size_of::<f32>() * 4) as BufferAddress;

impl VBDesc for InstanceData {
    fn desc<'a>() -> VertexBufferDescriptor<'a> {
        VertexBufferDescriptor {
            stride: std::mem::size_of::<InstanceData>() as BufferAddress,
            step_mode: InputStepMode::Instance,
            attributes: &[
                VertexAttributeDescriptor {
                    offset: 0,
                    format: VertexFormat::Float4,
                    shader_location: 3,
                },
                VertexAttributeDescriptor {
                    offset: ROW_SIZE,
                    format: VertexFormat::Float4,
                    shader_location: 4,
                },
                VertexAttributeDescriptor {
                    offset: ROW_SIZE * 2,
                    format: VertexFormat::Float4,
                    shader_location: 5,
                },
                VertexAttributeDescriptor {
                    offset: ROW_SIZE * 3,
                    format: VertexFormat::Float4,
                    shader_location: 6,
                },
            ],
        }
    }
}

pub struct Material {
    pub diffuse_texture: Texture,
    pub specular_texture: Texture,
    pub bind_group: BindGroup,
}

pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub material: usize,
    pub num_indexes: u32,
}

pub struct Model {
    pub instance_buffer: Buffer,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Model {
    // TODO: Create a trait for this?
    pub fn get_or_create_texture_layout(device: &Device) -> &'static BindGroupLayout {
        static LAYOUT: OnceCell<BindGroupLayout> = OnceCell::new();
        LAYOUT.get_or_init(|| {
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                bindings: &[
                    // diffuse texture
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::SampledTexture {
                            multisampled: false,
                            dimension: TextureViewDimension::D2,
                            component_type: TextureComponentType::Float,
                        },
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::Sampler { comparison: true },
                    },
                    // specular texutre
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::SampledTexture {
                            multisampled: false,
                            dimension: TextureViewDimension::D2,
                            component_type: TextureComponentType::Float,
                        },
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::Sampler { comparison: true },
                    },
                ],
                label: Some("Texture layout"),
            })
        })
    }

    pub fn load(path: impl AsRef<Path>, device: &Device) -> Result<(Self, Vec<CommandBuffer>)> {
        let (obj_models, obj_materials) = tobj::load_obj(path.as_ref())?;
        let current_folder = path.as_ref().parent().unwrap_or_else(|| {
            panic!(
                "There must exist a parent folder for object {:?}",
                path.as_ref()
            )
        });

        let mut command_buffers = Vec::with_capacity(obj_materials.len() * 2);
        let mut materials = Vec::with_capacity(obj_materials.len());

        for material in obj_materials {
            let diffuse_path = material.diffuse_texture;
            let mut specular_path = material.specular_texture;
            //let ambient_path = material.ambient_texture; TODO: Should this be handled?
            if specular_path.is_empty() {
                specular_path = diffuse_path.clone(); // TODO: WORST HACK EVER
            }
            let (diffuse_texture, diffuse_commands) =
                Texture::load(&device, current_folder.join(diffuse_path))?;
            let (specular_texture, specular_command) =
                Texture::load(&device, current_folder.join(specular_path))?;

            let layout = Self::get_or_create_texture_layout(device);

            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                layout,
                bindings: &[
                    Binding {
                        binding: 0,
                        resource: BindingResource::TextureView(&diffuse_texture.view),
                    },
                    Binding {
                        binding: 1,
                        resource: BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                    Binding {
                        binding: 2,
                        resource: BindingResource::TextureView(&specular_texture.view),
                    },
                    Binding {
                        binding: 3,
                        resource: BindingResource::Sampler(&specular_texture.sampler),
                    },
                ],
                label: None,
            });

            materials.push(Material {
                diffuse_texture,
                specular_texture,
                bind_group,
            });
            command_buffers.push(diffuse_commands);
            command_buffers.push(specular_command);
        }

        let mut meshes = Vec::new();
        for m in obj_models {
            let mut vertices = Vec::new();
            for i in 0..m.mesh.positions.len() / 3 {
                vertices.push(MeshVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                });
            }
            let vertex_buffer =
                device.create_buffer_with_data(&vertices.as_bytes(), BufferUsage::VERTEX);

            let index_buffer =
                device.create_buffer_with_data(&m.mesh.indices.as_bytes(), BufferUsage::INDEX);

            meshes.push(Mesh {
                vertex_buffer,
                index_buffer,
                material: m.mesh.material_id.unwrap_or(0),
                num_indexes: m.mesh.indices.len() as u32,
            });
        }
        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Instance buffer"),
            size: INDEX_BUFFER_SIZE, //TODO: reallocate is if it's changed and minimize data
            usage: BufferUsage::VERTEX | BufferUsage::COPY_DST,
        });
        Ok((
            Model {
                meshes,
                materials,
                instance_buffer,
            },
            command_buffers,
        ))
    }
}

pub trait DrawModel<'a> {
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instance_buffer: &'a Buffer,
        instances: Range<u32>,
    );

    fn draw_model_instanced(&mut self, model: &'a Model, instances: Range<u32>);
}

impl<'a> DrawModel<'a> for wgpu::RenderPass<'a> {
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instance_buffer: &'a Buffer,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
        self.set_vertex_buffer(1, &instance_buffer, 0, 0);
        self.set_index_buffer(&mesh.index_buffer, 0, 0);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.draw_indexed(0..mesh.num_indexes, 0, instances);
    }

    fn draw_model_instanced(&mut self, model: &'a Model, instances: Range<u32>) {
        let instance_buffer = &model.instance_buffer;
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, instance_buffer, instances.clone());
        }
    }
}
