use cgmath::Vector4;
use gl::types::*;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;

use cgmath::prelude::*;
use cgmath::{Matrix, Matrix4, Vector3};

#[derive(Debug, Clone, Copy)]
enum Operation {
    VertexCompilation,
    FragmentCompilation,
    ShaderLinking,
}

impl From<GLenum> for Operation {
    fn from(value: GLenum) -> Operation {
        match value {
            gl::VERTEX_SHADER => Operation::VertexCompilation,
            gl::FRAGMENT_SHADER => Operation::FragmentCompilation,
            _ => panic!("unknown shader_compilation"),
        }
    }
}

pub struct Shader {
    pub id: GLuint,
}

#[allow(dead_code)]
impl Shader {
    pub fn new(vertex_shader_path: &str, fragment_shader_path: &str) -> Result<Self, String> {
        let shader = Shader {
            id: unsafe { gl::CreateProgram() },
        };
        let mut vertex_shader_file = File::open(vertex_shader_path)
            .map_err(|err| format!("Error: Couldn't open vertex shader file, {}", err))?;
        let mut fragment_shader_file = File::open(fragment_shader_path)
            .map_err(|err| format!("Error: Couldn't open fragment shader file, {}", err))?;
        let mut vertex_code = String::new();
        let mut fragment_code = String::new();

        vertex_shader_file
            .read_to_string(&mut vertex_code)
            .map_err(|err| format!("Error: Couldn't read vertex shader file, {}", err))?;
        fragment_shader_file
            .read_to_string(&mut fragment_code)
            .map_err(|err| format!("Error: Couldn't read fragment shader file, {}", err))?;

        let v_shader_code = CString::new(vertex_code.as_bytes())
            .map_err(|err| format!("Error: Coudln't create cstring, {}", err))?;
        let f_shader_code = CString::new(fragment_code.as_bytes())
            .map_err(|err| format!("Error: Coudln't create cstring, {}", err))?;

        unsafe {
            let vertex_shader = shader.compile_shader(gl::VERTEX_SHADER, &v_shader_code)?;
            let fragment_shader = shader.compile_shader(gl::FRAGMENT_SHADER, &f_shader_code)?;

            gl::AttachShader(shader.id, vertex_shader);
            gl::AttachShader(shader.id, fragment_shader);
            gl::LinkProgram(shader.id);
            shader.check_compile_errors(shader.id, Operation::ShaderLinking)?;
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }
        Ok(shader)
    }

    pub unsafe fn use_program(&self) {
        gl::UseProgram(self.id);
    }

    /// utility uniform functions
    /// ------------------------------------------------------------------------
    pub unsafe fn set_bool(&self, name: &CStr, value: bool) {
        gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value as i32);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_int(&self, name: &CStr, value: i32) {
        gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_float(&self, name: &CStr, value: f32) {
        gl::Uniform1f(gl::GetUniformLocation(self.id, name.as_ptr()), value);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_vector3(&self, name: &CStr, value: &Vector3<f32>) {
        gl::Uniform3fv(
            gl::GetUniformLocation(self.id, name.as_ptr()),
            1,
            value.as_ptr(),
        );
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_vec3(&self, name: &CStr, x: f32, y: f32, z: f32) {
        gl::Uniform3f(gl::GetUniformLocation(self.id, name.as_ptr()), x, y, z);
    }

    pub unsafe fn set_vector4(&self, name: &CStr, value: &Vector4<f32>) {
        gl::Uniform4fv(
            gl::GetUniformLocation(self.id, name.as_ptr()),
            1,
            value.as_ptr(),
        );
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_vec4(&self, name: &CStr, x: f32, y: f32, z: f32, w: f32) {
        gl::Uniform4f(gl::GetUniformLocation(self.id, name.as_ptr()), x, y, z, w);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_mat4(&self, name: &CStr, mat: &Matrix4<f32>) {
        gl::UniformMatrix4fv(
            gl::GetUniformLocation(self.id, name.as_ptr()),
            1,
            gl::FALSE,
            mat.as_ptr(),
        );
    }

    unsafe fn compile_shader(
        &self,
        shader_type: GLenum,
        c_str_source: &CString,
    ) -> Result<GLuint, String> {
        let shader = gl::CreateShader(shader_type);
        gl::ShaderSource(shader, 1, &c_str_source.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);
        self.check_compile_errors(shader, Operation::from(shader_type))?;
        Ok(shader)
    }

    unsafe fn check_compile_errors(&self, shader: u32, operation: Operation) -> Result<(), String> {
        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(1024);
        info_log.set_len(1024); // subtract 1 to skip the trailing null character
        match operation {
            Operation::VertexCompilation | Operation::FragmentCompilation => {
                gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
                if success != gl::TRUE as GLint {
                    gl::GetShaderInfoLog(
                        shader,
                        1024,
                        std::ptr::null_mut(),
                        info_log.as_mut_ptr() as *mut GLchar,
                    );

                    let stripped_log = info_log
                        .into_iter()
                        .take_while(|c| *c as char != '\u{0}')
                        .collect::<Vec<u8>>();
                    Err(format!(
                        "ERROR::SHADER_COMPILATION_ERROR of type: {:?}\n{}\n",
                        operation,
                        String::from_utf8_lossy(&stripped_log) //.unwrap_or("Failed to convert compilation err to str"),
                    ))
                } else {
                    Ok(())
                }
            }
            Operation::ShaderLinking => {
                gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success);
                if success != gl::TRUE as GLint {
                    gl::GetProgramInfoLog(
                        shader,
                        1024,
                        std::ptr::null_mut(),
                        info_log.as_mut_ptr() as *mut GLchar,
                    );
                    Err(format!(
                        "ERROR::PROGRAM_LINKING_ERROR of type: {:?}\n{}\n",
                        operation,
                        std::str::from_utf8(&info_log)
                            .unwrap_or("Failed to convert link err to str")
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }
}
