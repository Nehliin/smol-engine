use crate::shader::{LightShader, ModelShader, Shader, ShaderSys};
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
        self.shader_systems.push(ModelShader::get_system());
        self.shader_systems.push(LightShader::get_system());
        resources.insert(light_shader);
        resources.insert(shader);
    }

    fn render_world(&mut self, world: &mut World, resources: &mut Resources) {
        self.shader_systems
            .iter_mut()
            .for_each(|system| system.run(world, resources))
    }
}
