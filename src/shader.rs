use cgmath::{Point3, Vector4};
use gl::types::*;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs::File;
use std::io::Read as IoRead;

use crate::camera::Camera;
use crate::lighting::directional_light::DirectionalLight;
use crate::lighting::point_light::PointLight;
use crate::model::Model;
use crate::state::Light; // very wrong
use crate::to_vec;
use crate::Transform;
use cgmath::prelude::*;
use cgmath::{Matrix, Matrix4, Vector3};
use legion::prelude::*;

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

//Temps

pub trait ShaderSys {
    fn get_system() -> Box<dyn Schedulable>;
}

pub struct LightShader(pub Shader);

impl ShaderSys for LightShader {
    fn get_system() -> Box<dyn Schedulable> {
        SystemBuilder::new("Light ShaderSystem")
            .write_resource::<LightShader>()
            .read_resource::<Camera>()
            .with_query(
                <(Read<Transform>, Read<Model>, Read<PointLight>)>::query().filter(tag::<Light>()),
            )
            .build(|_, world, (shader, camera), model_query| unsafe {
                shader.0.use_program();
                shader.0.set_mat4(
                    &CString::new("projection").unwrap(),
                    &camera.get_projection_matrix(),
                );
                shader
                    .0
                    .set_mat4(&CString::new("view").unwrap(), &camera.get_view_matrix());
                for (transform, model, _light) in model_query.iter(world) {
                    let transform_matrix = Matrix4::from_translation(transform.position)
                        * Matrix4::from_nonuniform_scale(
                            transform.scale.x,
                            transform.scale.y,
                            transform.scale.z,
                        );
                    shader
                        .0
                        .set_mat4(&CString::new("model").unwrap(), &transform_matrix);
                    model.draw(&mut shader.0);
                }
            })
    }
}

pub struct ModelShader(pub Shader);

impl ShaderSys for ModelShader {
    fn get_system() -> Box<dyn Schedulable> {
        SystemBuilder::new("Model ShaderSystem")
            .write_resource::<ModelShader>() //TODO: EN samlad resurs med alla shaders istället??
            .read_resource::<Camera>()
            .with_query(<(Read<Transform>, Read<Model>)>::query().filter(!tag::<Light>()))
            .with_query(<(Read<Transform>, Read<PointLight>)>::query().filter(tag::<Light>()))
            .build(
                |_, world, (shader, camera), (model_query, uniform_query)| unsafe {
                    shader.0.use_program();
                    shader.0.set_vector3(
                        &CString::new("viewPos").unwrap(),
                        &to_vec(&camera.get_position()),
                    );
                    shader.0.set_mat4(
                        &CString::new("projection").unwrap(),
                        &camera.get_projection_matrix(),
                    );
                    shader
                        .0
                        .set_mat4(&CString::new("view").unwrap(), &camera.get_view_matrix());
                    //shader.0.set_uniforms(&mut world);// ^---- alla light uniforms måste sättas här

                    let mut light_count = 0;
                    for (i, (transform, point_light)) in uniform_query.iter(world).enumerate() {
                        point_light.set_uniforms(&mut shader.0, i, &transform);
                        light_count += 1;
                    }
                    shader.0.set_int(
                        &CString::new("number_of_point_lights").unwrap(),
                        light_count,
                    );
                    //  let query = <Read<DirectionalLight>>::query().filter(tag::<Light>());
                    //if let Some(directional_light) = query.iter(world).next() {
                    //directional_light.set_uniforms(&mut shader.0);
                    //}

                    //let query = <(Read<Transform>, Read<Model>)>::query().filter(!tag::<Light>());
                    for (transform, model) in model_query.iter(world) {
                        let transform_matrix = Matrix4::from_translation(transform.position)
                            * Matrix4::from_nonuniform_scale(
                                transform.scale.x,
                                transform.scale.y,
                                transform.scale.z,
                            );
                        shader
                            .0
                            .set_mat4(&CString::new("model").unwrap(), &transform_matrix);
                        model.draw(&mut shader.0);
                    }
                },
            )
    }
}

#[allow(dead_code)]
impl Shader {
    pub fn new(vertex_shader_path: &str, fragment_shader_path: &str) -> Result<Self, String> {
        let mut shader = Shader {
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

    pub unsafe fn use_program(&mut self) {
        gl::UseProgram(self.id);
    }
    /// utility uniform functions
    /// ------------------------------------------------------------------------
    pub unsafe fn set_bool(&mut self, name: &CStr, value: bool) {
        gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value as i32);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_int(&mut self, name: &CStr, value: i32) {
        gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_float(&mut self, name: &CStr, value: f32) {
        gl::Uniform1f(gl::GetUniformLocation(self.id, name.as_ptr()), value);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_vector3(&mut self, name: &CStr, value: &Vector3<f32>) {
        gl::Uniform3fv(
            gl::GetUniformLocation(self.id, name.as_ptr()),
            1,
            value.as_ptr(),
        );
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_vec3(&mut self, name: &CStr, x: f32, y: f32, z: f32) {
        gl::Uniform3f(gl::GetUniformLocation(self.id, name.as_ptr()), x, y, z);
    }

    pub unsafe fn set_vector4(&mut self, name: &CStr, value: &Vector4<f32>) {
        gl::Uniform4fv(
            gl::GetUniformLocation(self.id, name.as_ptr()),
            1,
            value.as_ptr(),
        );
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_vec4(&mut self, name: &CStr, x: f32, y: f32, z: f32, w: f32) {
        gl::Uniform4f(gl::GetUniformLocation(self.id, name.as_ptr()), x, y, z, w);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_mat4(&mut self, name: &CStr, mat: &Matrix4<f32>) {
        gl::UniformMatrix4fv(
            gl::GetUniformLocation(self.id, name.as_ptr()),
            1,
            gl::FALSE,
            mat.as_ptr(),
        );
    }

    unsafe fn compile_shader(
        &mut self,
        shader_type: GLenum,
        c_str_source: &CString,
    ) -> Result<GLuint, String> {
        let shader = gl::CreateShader(shader_type);
        gl::ShaderSource(shader, 1, &c_str_source.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);
        self.check_compile_errors(shader, Operation::from(shader_type))?;
        Ok(shader)
    }

    unsafe fn check_compile_errors(
        &mut self,
        shader: u32,
        operation: Operation,
    ) -> Result<(), String> {
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
