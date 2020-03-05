#![allow(dead_code)]

use crate::cube::VERTICIES;
use crate::lighting::directional_light::DirectionalLight;
use crate::lighting::spotlight::SpotLight;
use crate::shader::Shader;
use cgmath::Matrix4;
use core::ffi::c_void;
use gl::types::*;
use point_light::PointLight;
use std::ffi::CString; // TODO: Ska ej hämtas här

pub mod directional_light;
pub mod point_light;
pub mod spotlight;

pub enum Strength {
    Weak,

    Medium,

    Strong,
}

impl Strength {
    pub fn get_values(&self) -> (f32, f32) {
        match self {
            Strength::Weak => (0.22, 0.2),
            Strength::Medium => (0.09, 0.032),
            Strength::Strong => (0.045, 0.0075),
        }
    }
}

pub struct Lighting {
    directional_light: Option<DirectionalLight>,
    pub point_lights: Vec<PointLight>,
    pub spotlights: Vec<SpotLight>,

    shader: Shader,

    vertex_array_object: u32,
    vertex_array_buffer: u32,
}

impl Drop for Lighting {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vertex_array_buffer);
            gl::DeleteVertexArrays(1, &self.vertex_array_object);
        }
    }
}

impl Lighting {
    pub fn new() -> Self {
        let lamp_shader = Shader::new(
            "src/light_vertex_shader.shader",
            "src/light_fragment_shader.shader",
        )
        .expect("Failed to create light shaders");
        let mut lighting = Lighting {
            directional_light: None,
            point_lights: Vec::new(),
            spotlights: Vec::new(),
            shader: lamp_shader,
            vertex_array_buffer: 0,
            vertex_array_object: 0,
        };
        unsafe {
            lighting.setup_lights();
        }
        lighting
    }

    unsafe fn setup_lights(&mut self) {
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

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    pub fn set_directional_light(&mut self, light: DirectionalLight) {
        self.directional_light = Some(light);
    }

    pub unsafe fn set_uniforms(&self, shader: &mut Shader) {
        shader.set_int(
            &CString::new("number_of_point_lights").unwrap(),
            self.point_lights.len() as i32,
        );

        shader.set_int(
            &CString::new("number_of_spot_lights").unwrap(),
            self.spotlights.len() as i32,
        );

        for (i, light) in self.point_lights.iter().enumerate() {
            // set all uniforms
            shader.set_float(
                &CString::new(format!("pointLights[{}].constant", i)).unwrap(),
                light.constant,
            );
            shader.set_float(
                &CString::new(format!("pointLights[{}].quadratic", i)).unwrap(),
                light.quadratic,
            );
            shader.set_float(
                &CString::new(format!("pointLights[{}].linear", i)).unwrap(),
                light.linear,
            );

            shader.set_vector3(
                &CString::new(format!("pointLights[{}].ambient", i)).unwrap(),
                &light.ambient,
            );
            shader.set_vector3(
                &CString::new(format!("pointLights[{}].diffuse", i)).unwrap(),
                &light.diffuse,
            );
            shader.set_vector3(
                &CString::new(format!("pointLights[{}].specular", i)).unwrap(),
                &light.specular,
            );
            shader.set_vector3(
                &CString::new(format!("pointLights[{}].position", i)).unwrap(),
                &light.position,
            );
        }
        for (i, light) in self.spotlights.iter().enumerate() {
            // set all uniforms
            shader.set_float(
                &CString::new(format!("spotLights[{}].constant", i)).unwrap(),
                light.constant,
            );
            shader.set_float(
                &CString::new(format!("spotLights[{}].quadratic", i)).unwrap(),
                light.quadratic,
            );
            shader.set_float(
                &CString::new(format!("spotLights[{}].linear", i)).unwrap(),
                light.linear,
            );

            shader.set_float(
                &CString::new(format!("spotLights[{}].cutoff", i)).unwrap(),
                light.cutoff,
            );
            shader.set_float(
                &CString::new(format!("spotLights[{}].outerCutOff", i)).unwrap(),
                light.outer_cutoff,
            );

            shader.set_vector3(
                &CString::new(format!("spotLights[{}].ambient", i)).unwrap(),
                &light.ambient,
            );
            shader.set_vector3(
                &CString::new(format!("spotLights[{}].diffuse", i)).unwrap(),
                &light.diffuse,
            );
            shader.set_vector3(
                &CString::new(format!("spotLights[{}].specular", i)).unwrap(),
                &light.specular,
            );
            shader.set_vector3(
                &CString::new(format!("spotLights[{}].position", i)).unwrap(),
                &light.position,
            );
            shader.set_vector3(
                &CString::new(format!("spotLights[{}].direction", i)).unwrap(),
                &light.direction,
            );
        }
        if let Some(directional_light) = &self.directional_light {
            shader.set_vector3(
                &CString::new("directional_light.ambient").unwrap(),
                &directional_light.ambient,
            );
            shader.set_vector3(
                &CString::new("directional_light.diffuse").unwrap(),
                &directional_light.diffuse,
            );
            shader.set_vector3(
                &CString::new("directional_light.specular").unwrap(),
                &directional_light.specular,
            );
            shader.set_vector3(
                &CString::new("directional_light.direction").unwrap(),
                &directional_light.direction,
            );
        }
    }
    // TODO: This should not be here! decouple cube and light!
    pub unsafe fn draw(&mut self, projection_matrix: &Matrix4<f32>, view_matrix: &Matrix4<f32>) {
        self.shader.use_program();
        gl::BindVertexArray(self.vertex_array_object);
        for light in self.point_lights.iter() {
            self.shader
                .set_mat4(&CString::new("projection").unwrap(), &projection_matrix);
            self.shader
                .set_mat4(&CString::new("view").unwrap(), &view_matrix);
            let model_matrix = Matrix4::from_translation(light.position) * Matrix4::from_scale(0.2);
            self.shader
                .set_mat4(&CString::new("model").unwrap(), &model_matrix);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }
        // for light in self.spotlights.iter() {
        //     self.shader
        //         .set_mat4(&CString::new("projection").unwrap(), &projection_matrix);
        //     self.shader
        //         .set_mat4(&CString::new("view").unwrap(), &view_matrix);
        //     let model_matrix = Matrix4::from_translation(light.position) * Matrix4::from_scale(0.2);
        //     self.shader
        //         .set_mat4(&CString::new("model").unwrap(), &model_matrix);
        //     gl::DrawArrays(gl::TRIANGLES, 0, 36);
        // }
        gl::BindVertexArray(0);
    }
}

// impl drop here
