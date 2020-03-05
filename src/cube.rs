use crate::mesh::Texture;
use crate::model::texture_from_file;
use crate::shader::Shader;
use cgmath::{vec3, Vector3, Zero};
use gl::types::*;
use glfw::ffi::glfwSetWindowMaximizeCallback;
use std::ffi::{c_void, CStr, CString};

#[rustfmt::skip]
pub const VERTICIES: [f32; 288] = [
    // positions          // normals           // texture coords
    -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  0.0, 0.0,
    0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  1.0, 0.0,
    0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  1.0, 1.0,
    0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  1.0, 1.0,
    -0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  0.0, 1.0,
    -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  0.0, 0.0,

    -0.5, -0.5,  0.5,  0.0,  0.0, 1.0,   0.0, 0.0,
    0.5, -0.5,  0.5,  0.0,  0.0, 1.0,   1.0, 0.0,
    0.5,  0.5,  0.5,  0.0,  0.0, 1.0,   1.0, 1.0,
    0.5,  0.5,  0.5,  0.0,  0.0, 1.0,   1.0, 1.0,
    -0.5,  0.5,  0.5,  0.0,  0.0, 1.0,   0.0, 1.0,
    -0.5, -0.5,  0.5,  0.0,  0.0, 1.0,   0.0, 0.0,

    -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,  1.0, 0.0,
    -0.5,  0.5, -0.5, -1.0,  0.0,  0.0,  1.0, 1.0,
    -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,  0.0, 1.0,
    -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,  0.0, 1.0,
    -0.5, -0.5,  0.5, -1.0,  0.0,  0.0,  0.0, 0.0,
    -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,  1.0, 0.0,

    0.5,  0.5,  0.5,  1.0,  0.0,  0.0,  1.0, 0.0,
    0.5,  0.5, -0.5,  1.0,  0.0,  0.0,  1.0, 1.0,
    0.5, -0.5, -0.5,  1.0,  0.0,  0.0,  0.0, 1.0,
    0.5, -0.5, -0.5,  1.0,  0.0,  0.0,  0.0, 1.0,
    0.5, -0.5,  0.5,  1.0,  0.0,  0.0,  0.0, 0.0,
    0.5,  0.5,  0.5,  1.0,  0.0,  0.0,  1.0, 0.0,

    -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  0.0, 1.0,
    0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  1.0, 1.0,
    0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  1.0, 0.0,
    0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  1.0, 0.0,
    -0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  0.0, 0.0,
    -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  0.0, 1.0,

    -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  0.0, 1.0,
    0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  1.0, 1.0,
    0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  1.0, 0.0,
    0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  1.0, 0.0,
    -0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  0.0, 0.0,
    -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  0.0, 1.0
];

pub struct Cube {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
    texture_diffuse: Texture, //path ?
    texture_specular: Texture,

    vertex_array_object: u32,
    vertex_array_buffer: u32,
}

const diffuse_texture: &str = "container2.png";
const specular_texture: &str = "container2_specular.png";

impl Cube {
    pub fn new() -> Self {
        let texture_diffuse = Texture {
            id: unsafe { texture_from_file(diffuse_texture, "") },
            type_str: "diffuse_textures".to_string(),
            path: diffuse_texture.to_string(),
        };

        let texture_specular = Texture {
            id: unsafe { texture_from_file(specular_texture, "") },
            type_str: "specular_textures".to_string(),
            path: specular_texture.to_string(),
        };

        let mut cube = Cube {
            position: Vector3::zero(),
            texture_diffuse,
            texture_specular,
            scale: vec3(1.0, 1.0, 1.0),
            vertex_array_object: 0,
            vertex_array_buffer: 0,
        };
        unsafe {
            cube.set_up_cube();
        }
        cube
    }

    unsafe fn set_up_cube(&mut self) {
        gl::GenBuffers(1, &mut self.vertex_array_buffer);
        gl::GenVertexArrays(1, &mut self.vertex_array_object);

        gl::BindVertexArray(self.vertex_array_object);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_array_buffer);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (VERTICIES.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
            &VERTICIES[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            8 * std::mem::size_of::<GLfloat>() as GLsizei,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            8 * std::mem::size_of::<GLfloat>() as GLsizei,
            (3 * std::mem::size_of::<GLfloat>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            8 * std::mem::size_of::<GLfloat>() as GLsizei,
            (6 * std::mem::size_of::<GLfloat>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(2);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    pub fn set_position(mut self, position: Vector3<f32>) -> Self {
        self.position = position;
        self
    }

    pub unsafe fn draw(&self, shader: &mut Shader) {
        shader.use_program();
        shader.set_int(&CString::new("number_of_diffuse_textures").unwrap(), 1);
        shader.set_int(&CString::new("number_of_specular_textures").unwrap(), 1);
        gl::ActiveTexture(gl::TEXTURE0); // active proper texture unit before binding
                                         // retrieve texture number (the N in diffuse_textureN)
                                         // now set the sampler to the correct texture unit
        let sampler =
            CString::new(format!("material.{}[{}]", self.texture_diffuse.type_str, 0)).unwrap();
        gl::Uniform1i(gl::GetUniformLocation(shader.id, sampler.as_ptr()), 0);
        // and finally bind the texture
        gl::BindTexture(gl::TEXTURE_2D, self.texture_diffuse.id);
        gl::ActiveTexture(gl::TEXTURE1); // active proper texture unit before binding
                                         // retrieve texture number (the N in diffuse_textureN)
                                         // now set the sampler to the correct texture unit
        let sampler = CString::new(format!(
            "material.{}[{}]",
            self.texture_specular.type_str, 0
        ))
        .unwrap();
        gl::Uniform1i(gl::GetUniformLocation(shader.id, sampler.as_ptr()), 1);
        // and finally bind the texture
        gl::BindTexture(gl::TEXTURE_2D, self.texture_specular.id);
        gl::BindVertexArray(self.vertex_array_object);
        gl::DrawArrays(gl::TRIANGLES, 0, 36);
        gl::BindVertexArray(0);
        gl::ActiveTexture(gl::TEXTURE0);
    }
}

impl Drop for Cube {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vertex_array_buffer);
            gl::DeleteVertexArrays(1, &self.vertex_array_object);
        }
    }
}
