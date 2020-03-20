#![allow(dead_code)]

use std::ffi::CString;
use std::os::raw::c_void;

use cgmath::prelude::*;
use cgmath::{Vector2, Vector3};
use gl;

use crate::shader::Shader;

// NOTE: without repr(C) the compiler may reorder the fields or use different padding/alignment than C.
// Depending on how you pass the data to OpenGL, this may be bad. In this case it's not strictly
// necessary though because of the `offset!` macro used below in setupMesh()
#[repr(C)]
pub struct Vertex {
    // position
    pub position: Vector3<f32>,
    // normal
    pub normal: Vector3<f32>,
    // texCoords
    pub tex_coords: Vector2<f32>,
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Vector3::zero(),
            normal: Vector3::zero(),
            tex_coords: Vector2::zero(),
        }
    }
}

#[derive(Clone)]
pub struct Texture {
    pub id: u32,
    pub type_str: String,
    pub path: String,
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

pub struct Mesh {
    /*  Mesh Data  */
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u32>>,
    pub textures: Vec<Texture>,
    pub vertex_array_object: u32,

    /*  Render data  */
    vertex_array_buffer: u32,
    element_array_buffer: u32,
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vertex_array_buffer);
            gl::DeleteBuffers(1, &self.element_array_buffer);
            gl::DeleteVertexArrays(1, &self.vertex_array_object);
        }
    }
}

impl Mesh {
    pub fn new_indexed(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<Texture>) -> Mesh {
        let mut mesh = Mesh {
            vertices,
            indices: Some(indices),
            textures,
            vertex_array_object: 0,
            vertex_array_buffer: 0,
            element_array_buffer: 0,
        };

        // now that we have all the required data, set the vertex buffers and its attribute pointers.
        unsafe { mesh.setup_mesh() }
        mesh
    }

    pub fn new_unindexed(vertices: Vec<Vertex>, textures: Vec<Texture>) -> Mesh {
        let mut mesh = Mesh {
            vertices,
            indices: None,
            textures,
            vertex_array_object: 0,
            vertex_array_buffer: 0,
            element_array_buffer: 0,
        };
        unsafe { mesh.setup_mesh() }
        mesh
    }

    /// render the mesh
    pub unsafe fn draw(&self, shader: &mut Shader) {
        //  shader.use_program();
        // bind appropriate textures
        let mut diffuse_number = 0;
        let mut specular_number = 0;
        for (i, texture) in self.textures.iter().enumerate() {
            gl::ActiveTexture(gl::TEXTURE0 + i as u32); // active proper texture unit before binding
                                                        // retrieve texture number (the N in diffuse_textureN)
            let name = &texture.type_str;
            let number = match name.as_str() {
                "diffuse_textures" => {
                    diffuse_number += 1;
                    diffuse_number
                }
                "specular_textures" => {
                    specular_number += 1;
                    specular_number
                }
                _ => panic!("unknown texture type"),
            };
            let number = number - 1;
            // now set the sampler to the correct texture unit
            let sampler = CString::new(format!("material.{}[{}]", name, number as i32)).unwrap();
            gl::Uniform1i(
                gl::GetUniformLocation(shader.id, sampler.as_ptr()),
                i as i32,
            );
            // and finally bind the texture
            gl::BindTexture(gl::TEXTURE_2D, texture.id);
        }
        shader.set_float(&CString::new("material.shininess").unwrap(), 32.0);
        shader.set_int(c_str!("number_of_specular_textures"), specular_number);
        shader.set_int(c_str!("number_of_diffuse_textures"), diffuse_number);

        // draw mesh
        gl::BindVertexArray(self.vertex_array_object);
        if let Some(indices) = &self.indices {
            gl::DrawElements(
                gl::TRIANGLES,
                indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        } else {
            // TODO: HACK
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }
        gl::BindVertexArray(0);

        // always good practice to set everything back to defaults once configured.
        gl::ActiveTexture(gl::TEXTURE0);
    }

    unsafe fn setup_mesh(&mut self) {
        // create buffers/arrays
        gl::GenVertexArrays(1, &mut self.vertex_array_object);
        gl::GenBuffers(1, &mut self.vertex_array_buffer);
        if self.indices.is_some() {
            gl::GenBuffers(1, &mut self.element_array_buffer);
        }

        gl::BindVertexArray(self.vertex_array_object);
        // load data into vertex buffers
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_array_buffer);
        // A great thing about structs with repr(C) is that their memory layout is sequential for all its items.
        // The effect is that we can simply pass a pointer to the struct and it translates perfectly to a glm::vec3/2 array which
        // again translates to 3/2 floats which translates to a byte array.
        let size = (self.vertices.len() * std::mem::size_of::<Vertex>()) as isize;
        let data = &self.vertices[0] as *const Vertex as *const c_void;
        gl::BufferData(gl::ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

        if let Some(indices) = &self.indices {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.element_array_buffer);
            let size = (indices.len() * std::mem::size_of::<u32>()) as isize;
            let data = &indices[0] as *const u32 as *const c_void;
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size, data, gl::STATIC_DRAW);
        }

        // set the vertex attribute pointers
        let size = std::mem::size_of::<Vertex>() as i32;
        // vertex Positions
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            size,
            offset_of!(Vertex, position) as *const c_void,
        );
        // vertex normals
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            size,
            offset_of!(Vertex, normal) as *const c_void,
        );
        // vertex texture coords
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            size,
            offset_of!(Vertex, tex_coords) as *const c_void,
        );

        gl::BindVertexArray(0);
    }
}
