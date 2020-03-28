use crate::shaders::{LightShader, ModelShader, OutLineShader, Shader, ShaderSys};
use legion::prelude::{Resources, Schedulable, World};

pub trait Renderer {
    fn init(&mut self, resources: &mut Resources);
    fn render_world(&mut self, world: &mut World, resources: &mut Resources);
}

pub struct BasicRenderer {
    shader_systems: Vec<Box<dyn Schedulable>>,
}

impl BasicRenderer {
    pub fn new() -> Self {
        BasicRenderer {
            shader_systems: Vec::new(),
        }
    }
}

impl Renderer for BasicRenderer {
    fn init(&mut self, resources: &mut Resources) {
        let shader = ModelShader(
            Shader::new("src/vertex_shader.shader", "src/fragment_shader.shader")
                .expect("Failed to create model shader"),
        );
        let light_shader = LightShader(
            Shader::new(
                "src/light_vertex_shader.shader",
                "src/light_fragment_shader.shader",
            )
            .expect("Failed to create Light shader"),
        );
        let outline_shader = OutLineShader(
            Shader::new("src/light_vertex_shader.shader", "src/outline_frag.shader")
                .expect("Failed to create OutLineShader"),
        );
        self.shader_systems.push(LightShader::get_system());
        self.shader_systems.push(ModelShader::get_system());
        self.shader_systems.push(OutLineShader::get_system());

        resources.insert(light_shader);
        resources.insert(shader);
        resources.insert(outline_shader);
        unsafe {
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::STENCIL_TEST);
        }
    }

    fn render_world(&mut self, world: &mut World, resources: &mut Resources) {
        self.shader_systems
            .iter_mut()
            .for_each(|system| system.run(world, resources))
    }
}
