use crate::engine::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::shaders::{LightShader, ModelShader, OutLineShader, Shader, ShaderSys, SkyBoxShader};
use core::ffi::c_void;
use gl::types::*;
use legion::prelude::{Resources, Schedulable, World};
use std::ffi::CString;
use std::path::Path;

pub trait Renderer {
    fn init(&mut self, resources: &mut Resources);
    fn render_world(&mut self, world: &mut World, resources: &mut Resources);
}

/*
TODO: Potential refactor, Create a renderer pipeline where one can chain Renderer traits
for example the BasicRenderer -> SkyBoxRenderer -> PostProcessingRenderer
the chain can keep internal state to match stencil testing, framebuffering etc
*/

// Verticies for the simple rectangle where the off
// screen rendered textured, (this is in normalized device coordinates)
#[rustfmt::skip]
const QUAD_VERTICES: [f32; 24] = [
    // positions   // texCoords
    -1.0, 1.0,     0.0, 1.0,
    -1.0,-1.0,     0.0, 0.0, 
    1.0, -1.0,     1.0, 0.0, 
    -1.0, 1.0,     0.0, 1.0,
    1.0, -1.0,     1.0, 0.0, 
    1.0, 1.0,     1.0, 1.0,
];

pub struct BasicRenderer {
    shader_systems: Vec<Box<dyn Schedulable>>,
    frame_buffer: u32,
    frame_buffer_texture: u32,
    render_buffer: u32,
    quad_vao: u32,
    quad_vbo: u32,
    post_processing_shader: Option<Shader>,
}

impl Drop for BasicRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.quad_vao);
            gl::DeleteBuffers(1, &self.quad_vbo);
            gl::DeleteFramebuffers(1, &self.frame_buffer);
            gl::DeleteRenderbuffers(1, &self.render_buffer);
        }
    }
}

impl BasicRenderer {
    pub fn new() -> Self {
        BasicRenderer {
            shader_systems: Vec::new(),
            frame_buffer: 0,
            frame_buffer_texture: 0,
            render_buffer: 0,
            quad_vao: 0, // TODO: break these out to a postProcessing renderer
            quad_vbo: 0,
            post_processing_shader: None,
        }
    }

    unsafe fn set_up_frame_buffer(&mut self) -> Result<(), String> {
        // Generate framebuffer
        gl::GenFramebuffers(1, &mut self.frame_buffer);
        // Bind both read and write operations to the buffer
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.frame_buffer);
        // Create the texture the writes to the framebuffer will be stored in
        gl::GenTextures(1, &mut self.frame_buffer_texture);
        gl::BindTexture(gl::TEXTURE_2D, self.frame_buffer_texture);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            WINDOW_WIDTH as i32,
            WINDOW_HEIGHT as i32,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            std::ptr::null(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::BindTexture(gl::TEXTURE_2D, 0);

        // attach to framebuffer, note only the color attachment is needed for the texture
        // because that's the only thing the shaders will sample the rest will be stored
        // in a renderbuffer object, created below
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            self.frame_buffer_texture,
            0,
        );

        // create the render buffer
        gl::GenRenderbuffers(1, &mut self.render_buffer);
        gl::BindRenderbuffer(gl::RENDERBUFFER, self.render_buffer);
        // Both depth and stencil data is stored since they are not read in the shader
        gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH24_STENCIL8,
            WINDOW_WIDTH as i32,
            WINDOW_HEIGHT as i32,
        );
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
        // bind the renderbuffer to the frame buffer as depth and stencil attachment
        gl::FramebufferRenderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_STENCIL_ATTACHMENT,
            gl::RENDERBUFFER,
            self.render_buffer,
        );

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            return Err("FrameBuffer is not completed!".to_string());
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        Ok(())
    }

    unsafe fn set_up_screen_quad(&mut self) {
        gl::GenVertexArrays(1, &mut self.quad_vao);
        gl::GenBuffers(1, &mut self.quad_vbo);
        gl::BindVertexArray(self.quad_vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, self.quad_vbo);
        let size = (QUAD_VERTICES.len() * std::mem::size_of::<GLfloat>()) as isize;
        let data = &QUAD_VERTICES[0] as *const f32 as *const c_void;
        gl::BufferData(gl::ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

        let element_size = std::mem::size_of::<GLfloat>() as i32;
        let stride = element_size * 4;
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, std::ptr::null());

        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (2 * element_size) as *const c_void,
        );

        gl::BindVertexArray(0);
    }
}

impl Renderer for BasicRenderer {
    fn init(&mut self, resources: &mut Resources) {
        let shader = ModelShader(
            Shader::new(
                "src/shader_files/vertex_shader.shader",
                "src/shader_files/fragment_shader.shader",
            )
            .expect("Failed to create model shader"),
        );
        let light_shader = LightShader(
            Shader::new(
                "src/shader_files/light_vertex_shader.shader",
                "src/shader_files/light_fragment_shader.shader",
            )
            .expect("Failed to create Light shader"),
        );
        let outline_shader = OutLineShader(
            Shader::new(
                "src/shader_files/light_vertex_shader.shader",
                "src/shader_files/outline_frag.shader",
            )
            .expect("Failed to create OutLineShader"),
        );

        self.shader_systems.push(LightShader::get_system());
        self.shader_systems.push(ModelShader::get_system());
        self.shader_systems.push(OutLineShader::get_system());
        let skybox_shader = SkyBoxShader::new(&Path::new("skybox"));
        self.shader_systems.push(SkyBoxShader::get_system());

        resources.insert(skybox_shader);
        resources.insert(light_shader);
        resources.insert(shader);
        resources.insert(outline_shader);

        unsafe {
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);
            gl::Enable(gl::MULTISAMPLE);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::STENCIL_TEST);
            self.set_up_frame_buffer().unwrap();
            self.set_up_screen_quad();
        }

        // Add post procesing shader:
        self.post_processing_shader = Some(
            Shader::new(
                "src/shader_files/post_vertex.shader",
                "src/shader_files/post_frag.shader",
            )
            .expect("Failed to create post processing shader"),
        );
        // Bind uniform value here since it never changes:
        unsafe {
            if let Some(post_shader) = &mut self.post_processing_shader {
                post_shader.use_program();
                post_shader.set_int(&CString::new("frame_buffer_texture").unwrap(), 0);
            }
        }
    }

    fn render_world(&mut self, world: &mut World, resources: &mut Resources) {
        if let Some(post_shader) = &mut self.post_processing_shader {
            // The framebuffer is first bound so all draw commands affect that framebuffer instead of the
            // default one
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.frame_buffer);
                gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
            }
            self.shader_systems
                .iter_mut()
                .for_each(|system| system.run(world, resources));

            // Bind the default framebuffer afterwards to draw the texture on a simple quad the size of the screen
            // The color buffer bit must be cleared here since that is not bound to the render buffer and is red by the
            // post processing shaders, each frame buffer must be individually cleared.
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                gl::ClearColor(1.0, 1.0, 1.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::Disable(gl::DEPTH_TEST);
                // activate post process shaders
                post_shader.use_program();

                // bind the quad where the texture is drawn
                gl::BindVertexArray(self.quad_vao);
                // (sampler2D uniform for the frame buffer texture
                //  is set once in the init function of the shader)
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, self.frame_buffer_texture);
                gl::DrawArrays(gl::TRIANGLES, 0, 6);

                // set everything back to normal
                gl::BindVertexArray(0);
                gl::BindTexture(gl::TEXTURE_2D, 0);
                gl::Enable(gl::DEPTH_TEST);
            }
        } else {
            self.shader_systems
                .iter_mut()
                .for_each(|system| system.run(world, resources));
        }
    }
}
