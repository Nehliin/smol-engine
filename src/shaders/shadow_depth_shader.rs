use crate::camera::{Camera, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::components::{LightTag, Transform};
use crate::lighting::DirectionalLight;
use crate::model::Model;
use crate::shaders::Shader;
use crate::shaders::ShaderSys;

use cgmath::{vec3, Matrix4, Point3};
use legion::prelude::*;
use std::ffi::CString;

const SHADOW_WIDTH: i32 = 1024;
const SHADOW_HEIGHT: i32 = 1024;

const NEAR_PLANE: f32 = 1.0;
const FAR_PLANE: f32 = 7.5;

pub struct ShadowDepthShader {
    depth_map_frame_buffer: u32,
    depth_map_texture: u32,
    shader: Shader,
    light_projection_matrix: Matrix4<f32>,
}

impl Drop for ShadowDepthShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.depth_map_frame_buffer);
            gl::DeleteTextures(1, &self.depth_map_texture);
        }
    }
}

impl ShadowDepthShader {
    pub fn new() -> Self {
        let mut depth_map_frame_buffer = 0;
        let mut depth_map_texture = 0;
        let light_projection_matrix =
            cgmath::ortho(-10.0, 10.0, -10.0, 10.0, NEAR_PLANE, FAR_PLANE);
        // Set up framebuffer and texture where the depthmap will be stored
        unsafe {
            gl::GenFramebuffers(1, &mut depth_map_frame_buffer);
            gl::GenTextures(1, &mut depth_map_texture);

            gl::BindTexture(gl::TEXTURE_2D, depth_map_texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH_COMPONENT as i32,
                SHADOW_WIDTH,
                SHADOW_HEIGHT,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                std::ptr::null(),
            );

            gl::TextureParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TextureParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TextureParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TextureParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

            // attach texture as framebuffers depth buffer
            // since only the the depth buffer is needed the draw buffer
            // and read buffer is explicitly disabled otherwise the framebuffer isn't
            // completed
            gl::BindFramebuffer(gl::FRAMEBUFFER, depth_map_frame_buffer);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                depth_map_texture,
                0,
            );
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        ShadowDepthShader {
            depth_map_frame_buffer,
            depth_map_texture,
            shader: Shader::new(
                "shader_files/depth_map_vertex.shader",
                "shader_files/depth_map_frag.shader",
            )
            .expect("Failed to create depth map shader"),
            light_projection_matrix,
        }
    }
}

impl ShaderSys for ShadowDepthShader {
    fn get_system() -> Box<dyn Schedulable> {
        SystemBuilder::new("ShadowDepth ShaderSystem")
            .write_resource::<Self>() //TODO: EN samlad resurs med alla shaders istället??
            .read_resource::<Camera>()
            .with_query(<Read<DirectionalLight>>::query().filter(tag::<LightTag>()))
            .with_query(<(Read<Transform>, Read<Model>)>::query())
            .build(
                |_,
                 world,
                 (shadow_depth_shader, camera),
                 (directional_light_query, object_query)| unsafe {
                    // viewport must match the shadow resolution otherwise
                    // the scaling will be wierd probably
                    gl::Viewport(0, 0, SHADOW_WIDTH, SHADOW_HEIGHT);
                    shadow_depth_shader.shader.use_program();
                    // (only a single direcitonal light at a time is supported for the moment)
                    for directional_light in directional_light_query.iter(world) {
                        // set the view matrix equal to the look at matrix from  the oposite direction of the directional light times 100 to get some distance
                        let source_vec = -directional_light.direction * 100.0;
                        let directional_light_source =
                            Point3::new(source_vec.x, source_vec.y, -source_vec.z);
                        // with the direction of the directional light
                        let light_view_matrix = Matrix4::look_at_dir(
                            directional_light_source,
                            directional_light.direction,
                            vec3(0.0, 1.0, 0.0),
                        );
                        // calculate the light space transformation matrix so all cooridnates can be converted to the coordinate space with the light as the origin
                        let light_space_matrix =
                            shadow_depth_shader.light_projection_matrix * light_view_matrix;
                        shadow_depth_shader.shader.set_mat4(
                            &CString::new("light_space_matrix").unwrap(),
                            &light_space_matrix,
                        );
                    }

                    // Render the scene with the depth map shader that only tranforms the vertex coordinates to the light space
                    // and mesures the depth of them
                    // FUCK THIS WONT WORK! THE SHADOW DEPTH SHADER NEEDS TO BE SET UP BEFORE AND THEN RENDER THE REST OF
                    // THE RELEVANT PIPELINE THEN IT NEEDS TO RERUN WITH THE  REGULAR PIPELINE
                    for (transform, _) in object_query.iter(world) {
                        shadow_depth_shader.shader.set_mat4(
                            &CString::new("model").unwrap(),
                            &transform.get_model_matrix(),
                        );
                    }
                    // Reset the viewport
                    gl::Viewport(0, 0, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);
                },
            )
    }
}
