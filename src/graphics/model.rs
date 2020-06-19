use anyhow::Result;
use nalgebra::{Matrix4, Vector3};
use once_cell::sync::OnceCell;
use smol_renderer::Texture;
use smol_renderer::{
    GpuData, ImmutableVertexData, LoadableTexture, MutableVertexData, RenderNodeRunner,
    SimpleTexture, TextureData, VertexBuffer,
};
use std::ops::Range;
use std::path::Path;
use wgpu::{
    Buffer, BufferAddress, BufferUsage, CommandBuffer, Device, VertexAttributeDescriptor,
    VertexFormat,
};

const INDEX_BUFFER_SIZE: u64 = 16_000;

#[repr(C)]
#[derive(GpuData, Debug)]
pub struct MeshVertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coords: [f32; 2],
}

impl VertexBuffer for MeshVertex {
    const STEP_MODE: wgpu::InputStepMode = wgpu::InputStepMode::Vertex;

    fn get_attributes<'a>() -> &'a [wgpu::VertexAttributeDescriptor] {
        &[
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
        ]
    }
}

#[repr(C)]
#[derive(GpuData, Debug, Clone)]
pub struct InstanceData {
    model_matrix: Matrix4<f32>,
}

impl InstanceData {
    pub fn new(model_matrix: Matrix4<f32>) -> Self {
        InstanceData { model_matrix }
    }
}

impl Default for InstanceData {
    fn default() -> Self {
        InstanceData {
            model_matrix: Matrix4::identity(),
        }
    }
}

const ROW_SIZE: BufferAddress = (std::mem::size_of::<f32>() * 4) as BufferAddress;

impl VertexBuffer for InstanceData {
    const STEP_MODE: wgpu::InputStepMode = wgpu::InputStepMode::Instance;

    fn get_attributes<'a>() -> &'a [wgpu::VertexAttributeDescriptor] {
        &[
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
        ]
    }
}
// TODO: This should be its own texture type
pub struct Material {
    pub diffuse_texture: TextureData<SimpleTexture>,
    pub specular_texture: TextureData<SimpleTexture>,
}

pub struct Mesh {
    pub vertex_buffer: ImmutableVertexData<MeshVertex>,
    pub index_buffer: Buffer,
    pub material: usize,
    pub num_indexes: u32,
}

pub struct Model {
    pub instance_buffer: MutableVertexData<InstanceData>,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Model {
    pub fn load(path: impl AsRef<Path>, device: &Device) -> Result<(Self, Vec<CommandBuffer>)> {
        let (obj_models, obj_materials) = tobj::load_obj(path.as_ref(), true)?;
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
                SimpleTexture::load_texture(&device, current_folder.join(diffuse_path))?;
            let (specular_texture, specular_command) =
                SimpleTexture::load_texture(&device, current_folder.join(specular_path))?;

            materials.push(Material {
                diffuse_texture,
                specular_texture,
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
            let vertex_buffer = VertexBuffer::allocate_immutable_buffer(device, &vertices);

            let indicies = unsafe {
                std::slice::from_raw_parts(
                    m.mesh.indices.as_ptr() as *const u8,
                    m.mesh.indices.len() * 4,
                )
            };

            let index_buffer = device.create_buffer_with_data(&indicies, BufferUsage::INDEX);

            meshes.push(Mesh {
                vertex_buffer,
                index_buffer,
                material: m.mesh.material_id.unwrap_or(0),
                num_indexes: m.mesh.indices.len() as u32,
            });
        }
        let instance_buffer_len = INDEX_BUFFER_SIZE as usize / std::mem::size_of::<InstanceData>();
        println!("INSTANCE BUFFER LEN: {}", instance_buffer_len);
        let buffer_data = vec![InstanceData::default(); instance_buffer_len];
        let instance_buffer = VertexBuffer::allocate_mutable_buffer(device, &buffer_data);
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
// TODO: rethink if this is even needed anymore?
pub trait DrawModel<'b> {
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instance_buffer: &'b MutableVertexData<InstanceData>,
        instances: Range<u32>,
    );

    fn draw_untextured(&mut self, model: &'b Model, instances: Range<u32>);

    fn draw_model_instanced(&mut self, model: &'b Model, instances: Range<u32>);
}

impl<'a, 'b> DrawModel<'b> for RenderNodeRunner<'a, 'b> {
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instance_buffer: &'b MutableVertexData<InstanceData>,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer_data(0, &mesh.vertex_buffer);
        self.set_vertex_buffer_data(1, instance_buffer);
        self.set_index_buffer(&mesh.index_buffer, 0, 0);
        self.set_texture_data(0, &material.diffuse_texture);
        self.set_texture_data(1, &material.specular_texture);
        self.draw_indexed(0..mesh.num_indexes, 0, instances);
    }

    fn draw_untextured(&mut self, model: &'b Model, instances: Range<u32>) {
        let instance_buffer = &model.instance_buffer;
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.set_vertex_buffer_data(0, &mesh.vertex_buffer);
            self.set_vertex_buffer_data(1, instance_buffer);
            self.set_index_buffer(&mesh.index_buffer, 0, 0);
        }
    }

    fn draw_model_instanced(&mut self, model: &'b Model, instances: Range<u32>) {
        let instance_buffer = &model.instance_buffer;
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, instance_buffer, instances.clone());
        }
    }
}
