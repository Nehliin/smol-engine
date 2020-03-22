use crate::mesh::Mesh;
use crate::mesh::Texture;
use crate::mesh::Vertex;
use crate::shaders::Shader;
use cgmath::vec2;
use cgmath::vec3;
use core::ffi::c_void;
use image::DynamicImage::*;
use image::GenericImage;
use std::path::Path;

// trash structure
#[rustfmt::skip]
pub const VERTICIES: [f32; 288] = [

// Back face
-0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  0.0, 0.0, // Bottom-let
0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  1.0, 1.0, // top-right
0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  1.0, 0.0, // bottom-right         
0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  1.0, 1.0, // top-right
-0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  0.0, 0.0, // bottom-let
-0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  0.0, 1.0, // top-let
// ront ace
-0.5, -0.5,  0.5,  0.0,  0.0, 1.0,  0.0, 0.0, // bottom-let
0.5, -0.5,  0.5,  0.0,  0.0, 1.0,  1.0, 0.0, // bottom-right
0.5,  0.5,  0.5,  0.0,  0.0, 1.0,  1.0, 1.0, // top-right
0.5,  0.5,  0.5,  0.0,  0.0, 1.0,  1.0, 1.0, // top-right
-0.5,  0.5,  0.5,  0.0,  0.0, 1.0,  0.0, 1.0, // top-let
-0.5, -0.5,  0.5,  0.0,  0.0, 1.0,  0.0, 0.0, // bottom-let
// Let ace
-0.5,  0.5,  0.5, -1.0,  0.0,  0.0,  1.0, 0.0, // top-right
-0.5,  0.5, -0.5, -1.0,  0.0,  0.0,  1.0, 1.0, // top-let
-0.5, -0.5, -0.5, -1.0,  0.0,  0.0,  0.0, 1.0, // bottom-let
-0.5, -0.5, -0.5, -1.0,  0.0,  0.0,  0.0, 1.0, // bottom-let
-0.5, -0.5,  0.5, -1.0,  0.0,  0.0,  0.0, 0.0, // bottom-right
-0.5,  0.5,  0.5, -1.0,  0.0,  0.0,  1.0, 0.0, // top-right
// Right ace
0.5,  0.5,  0.5,  1.0,  0.0,  0.0, 1.0, 0.0, // top-let
0.5, -0.5, -0.5,  1.0,  0.0,  0.0, 0.0, 1.0, // bottom-right
0.5,  0.5, -0.5,  1.0,  0.0,  0.0, 1.0, 1.0, // top-right         
0.5, -0.5, -0.5,  1.0,  0.0,  0.0, 0.0, 1.0, // bottom-right
0.5,  0.5,  0.5,  1.0,  0.0,  0.0, 1.0, 0.0, // top-let
0.5, -0.5,  0.5,  1.0,  0.0,  0.0, 0.0, 0.0, // bottom-let     
// Bottom ace
-0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  0.0, 1.0, // top-right
0.5, -0.5, -0.5,  0.0, -1.0,  0.0, 1.0, 1.0, // top-let
0.5, -0.5,  0.5,  0.0, -1.0,  0.0, 1.0, 0.0, // bottom-let
0.5, -0.5,  0.5,  0.0, -1.0,  0.0, 1.0, 0.0, // bottom-let
-0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  0.0, 0.0, // bottom-right
-0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  0.0, 1.0, // top-right
// Top ace
-0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  0.0, 1.0, // top-let
0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  1.0, 0.0, // bottom-right
0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  1.0, 1.0, // top-right     
0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  1.0, 0.0, // bottom-right
-0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  0.0, 1.0, // top-let
-0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  0.0, 0.0  // bottom-left     
];

const DIFFUSE_TEXTURE: &str = "container2.png";
const SPECULAR_TEXTURE: &str = "container2_specular.png";

pub struct Model {
    meshes: Vec<Mesh>,
    directory: String,
    textures_loaded: Vec<Texture>,
}

impl Model {
    pub fn new(path: &str) -> Self {
        let mut model = Model {
            meshes: Vec::new(),
            directory: String::new(),
            textures_loaded: Vec::new(),
        };

        model.load(path);
        model
    }

    pub fn cube() -> Self {
        let texture_diffuse = Texture {
            id: unsafe { texture_from_file(DIFFUSE_TEXTURE, "") },
            type_str: "diffuse_textures".to_string(),
            path: DIFFUSE_TEXTURE.to_string(),
        };

        let texture_specular = Texture {
            id: unsafe { texture_from_file(SPECULAR_TEXTURE, "") },
            type_str: "specular_textures".to_string(),
            path: SPECULAR_TEXTURE.to_string(),
        };

        let verticies = VERTICIES
            .chunks_exact(8)
            .map(|chunk| Vertex {
                position: vec3(chunk[0], chunk[1], chunk[2]),
                normal: vec3(chunk[3], chunk[4], chunk[5]),
                tex_coords: vec2(chunk[6], chunk[7]),
            })
            .collect();
        let mesh = Mesh::new_unindexed(verticies, vec![texture_diffuse, texture_specular]);
        Model {
            meshes: vec![mesh],
            directory: String::new(),
            textures_loaded: Vec::new(),
        }
    }

    pub unsafe fn draw(&self, shader: &mut Shader) {
        self.meshes.iter().for_each(|mesh| {
            mesh.draw(shader);
        });
    }

    fn load(&mut self, path: &str) {
        let path = Path::new(path);
        if let Some(parent) = path.parent() {
            self.directory = parent.to_str().expect("incorrect parent directory").into();
        }

        let obj = tobj::load_obj(path);

        let (models, materials) = obj.unwrap();

        models.iter().for_each(|model| {
            let mesh = &model.mesh;
            let num_vertices = mesh.positions.len() / 3;

            let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);
            let indices = mesh.indices.clone(); // remove clone?

            let (positions, normals, tex_coords) =
                (&mesh.positions, &mesh.normals, &mesh.texcoords);

            for i in 0..num_vertices {
                vertices.push(Vertex {
                    position: vec3(positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]),
                    normal: vec3(normals[i * 3], normals[i * 3 + 1], normals[i * 3 + 2]),
                    tex_coords: vec2(tex_coords[i * 2], tex_coords[i * 2 + 1]),
                })
            }

            // materials
            let mut textures = Vec::new();
            if let Some(material_id) = mesh.material_id {
                let material = &materials[material_id];

                if !material.diffuse_texture.is_empty() {
                    let texture =
                        self.load_material_texture(&material.diffuse_texture, "diffuse_textures");
                    textures.push(texture);
                }

                if !material.specular_texture.is_empty() {
                    let texture =
                        self.load_material_texture(&material.specular_texture, "specular_textures");
                    textures.push(texture);
                }
            }

            self.meshes
                .push(Mesh::new_indexed(vertices, indices, textures));
        });
    }

    fn load_material_texture(&mut self, path: &str, type_name: &str) -> Texture {
        {
            let texture = self.textures_loaded.iter().find(|t| t.path == path);
            if let Some(texture) = texture {
                return texture.clone();
            }
        }
        let texture = Texture {
            id: unsafe { texture_from_file(path, &self.directory) },
            type_str: type_name.into(),
            path: path.into(),
        };
        self.textures_loaded.push(texture.clone());
        texture
    }
}
// TODO: use actual paths here!
pub unsafe fn texture_from_file(path: &str, directory: &str) -> u32 {
    let filename = if !directory.is_empty() {
        format!("{}/{}", directory, path)
    } else {
        path.to_string()
    };

    let mut texture_id = 0;
    gl::GenTextures(1, &mut texture_id);

    let img = image::open(&Path::new(&filename)).expect("Texture failed to load");
    let img = img.flipv();
    let format = match img {
        ImageLuma8(_) => gl::RED,
        ImageLumaA8(_) => gl::RG,
        ImageRgb8(_) => gl::RGB,
        ImageRgba8(_) => gl::RGBA,
    };

    let data = img.raw_pixels();

    gl::BindTexture(gl::TEXTURE_2D, texture_id);
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        format as i32,
        img.width() as i32,
        img.height() as i32,
        0,
        format,
        gl::UNSIGNED_BYTE,
        &data[0] as *const u8 as *const c_void,
    );
    gl::GenerateMipmap(gl::TEXTURE_2D);

    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_MIN_FILTER,
        gl::LINEAR_MIPMAP_LINEAR as i32,
    );
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

    texture_id
}
