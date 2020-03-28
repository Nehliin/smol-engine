use crate::camera::Camera;
use crate::shaders::Shader;
use crate::shaders::ShaderSys;
use cgmath::Matrix3;
use gl::types::*;
use image::DynamicImage::*;
use image::GenericImage;
use legion::prelude::*;
use std::ffi::c_void;
use std::ffi::CString;
use std::path::Path;

// Note the lack of texture coordinates here
// OpenGL uses local coordinates to figure out where
// each texel should be mapped to for cubetextures
#[rustfmt::skip]
const SKYBOX_VERTICIES: [f32; 108] = [
     // positions          
     -1.0,  1.0, -1.0,
     -1.0, -1.0, -1.0,
      1.0, -1.0, -1.0,
      1.0, -1.0, -1.0,
      1.0,  1.0, -1.0,
     -1.0,  1.0, -1.0,
 
     -1.0, -1.0,  1.0,
     -1.0, -1.0, -1.0,
     -1.0,  1.0, -1.0,
     -1.0,  1.0, -1.0,
     -1.0,  1.0,  1.0,
     -1.0, -1.0,  1.0,
 
      1.0, -1.0, -1.0,
      1.0, -1.0,  1.0,
      1.0,  1.0,  1.0,
      1.0,  1.0,  1.0,
      1.0,  1.0, -1.0,
      1.0, -1.0, -1.0,
 
     -1.0, -1.0,  1.0,
     -1.0,  1.0,  1.0,
      1.0,  1.0,  1.0,
      1.0,  1.0,  1.0,
      1.0, -1.0,  1.0,
     -1.0, -1.0,  1.0,
 
     -1.0,  1.0, -1.0,
      1.0,  1.0, -1.0,
      1.0,  1.0,  1.0,
      1.0,  1.0,  1.0,
     -1.0,  1.0,  1.0,
     -1.0,  1.0, -1.0,
 
     -1.0, -1.0, -1.0,
     -1.0, -1.0,  1.0,
      1.0, -1.0, -1.0,
      1.0, -1.0, -1.0,
     -1.0, -1.0,  1.0,
      1.0, -1.0,  1.0
];

pub struct SkyBoxShader {
    skybox_path: &'static Path,
    cube_vao: u32,
    cube_vbo: u32,
    shader: Shader,
    texture: u32,
}

impl SkyBoxShader {
    pub fn new(skybox_path: &'static Path) -> Self {
        let mut skybox = SkyBoxShader {
            skybox_path,
            cube_vao: 0,
            cube_vbo: 0,
            shader: Shader::new(
                "src/shader_files/skybox_vertex.shader",
                "src/shader_files/skybox_frag.shader",
            )
            .expect("Failed to create Skybox shader"),
            texture: 0,
        };
        unsafe {
            skybox.generate_cube();
            /*skybox.texture = SkyBoxShader::loadCubemap(&vec![
                "skybox/1.jpg",
                "skybox/2.jpg",
                "skybox/3.jpg",
                "skybox/4.jpg",
                "skybox/5.jpg",
                "skybox/6.jpg",
            ]);*/
            skybox.load_textures().unwrap();
            skybox.shader.use_program();
            skybox
                .shader
                .set_int(&CString::new("skybox_texture").unwrap(), 0);
        }
        skybox
    }

    unsafe fn generate_cube(&mut self) {
        gl::GenVertexArrays(1, &mut self.cube_vao);
        gl::GenBuffers(1, &mut self.cube_vbo);

        gl::BindVertexArray(self.cube_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.cube_vbo);

        let size = (SKYBOX_VERTICIES.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr;
        let data = &SKYBOX_VERTICIES[0] as *const f32 as *const c_void;

        gl::BufferData(gl::ARRAY_BUFFER, size, data, gl::STATIC_DRAW);
        let stride = 3 * std::mem::size_of::<GLfloat>() as GLsizei;

        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null());

        gl::BindVertexArray(0);
    }
    /*
        This expects the textures to be sorted by name
    */
    unsafe fn load_textures(&mut self) -> Result<(), String> {
        let mut paths = Vec::new();

        let directory_iterator = std::fs::read_dir(self.skybox_path)
            .map_err(|err| format!("Failed to find directory: {}", err))?;
        for dir_entry in directory_iterator {
            let path = dir_entry
                .map_err(|err| format!("Couldn't view dir entry: {}", err))?
                .path();
            if !path.is_dir() {
                paths.push(path);
            }
        }
        if paths.len() != 6 {
            return Err("Skybox texture directory doesn't contain exactly 6 images".to_string());
        }
        paths.sort();
        dbg!(&paths);
        gl::GenTextures(1, &mut self.texture);
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.texture);

        for (i, path) in paths.iter().enumerate() {
            let img =
                image::open(&path).map_err(|err| format!("Texture failed to load: {}", err))?;
            //   let img = img.flipv();
            let data = img.raw_pixels();
            gl::TexImage2D(
                gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                0,
                gl::RGB as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                &data[0] as *const u8 as *const c_void,
            );
        }
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MAG_FILTER,
            gl::LINEAR as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_R,
            gl::CLAMP_TO_EDGE as i32,
        );

        gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);
        Ok(())
    }
}

impl Drop for SkyBoxShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.cube_vbo);
            gl::DeleteVertexArrays(1, &self.cube_vao);
            gl::DeleteTextures(1, &self.texture);
        }
    }
}

impl ShaderSys for SkyBoxShader {
    fn get_system() -> Box<dyn Schedulable> {
        SystemBuilder::new("Skybox ShaderSystem")
            .write_resource::<Self>() //TODO: EN samlad resurs med alla shaders istället??
            .read_resource::<Camera>()
            .build(|_, _world, (skybox_shader, camera), _| unsafe {
                gl::DepthMask(gl::FALSE);
                // The skybox has the max depth possible (1.0) because of how
                // the fragment shader calculations are done, thus the depth func need
                // to chekc GL_LEQUAL so fragments resulting ClearColor doesn't
                // make the skybox test to fail
                gl::DepthFunc(gl::LEQUAL);
                skybox_shader.shader.use_program();

                skybox_shader.shader.set_mat4(
                    &CString::new("projection").unwrap(),
                    &camera.get_projection_matrix(),
                );
                let mut view_matrix = *camera.get_view_matrix();
                view_matrix.w[0] = 0.0;
                view_matrix.w[1] = 0.0;
                view_matrix.w[2] = 0.0;
                skybox_shader
                    .shader
                    .set_mat4(&CString::new("view").unwrap(), &view_matrix);

                gl::BindVertexArray(skybox_shader.cube_vao);
                gl::ActiveTexture(gl::TEXTURE0);
                // texture uniform are bound in constructor since it never changes
                gl::BindTexture(gl::TEXTURE_CUBE_MAP, skybox_shader.texture);
                gl::DrawArrays(gl::TRIANGLES, 0, 36);

                gl::DepthMask(gl::TRUE);
                gl::DepthFunc(gl::LESS);
                gl::BindVertexArray(0);
            })
    }
}
