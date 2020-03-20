use crate::camera::Camera;
use crate::components::{LightTag, Transform};
use crate::lighting::point_light::PointLight;
use crate::model::Model;
use crate::shaders::Shader;
use crate::shaders::ShaderSys;
use legion::prelude::*;
use std::ffi::CString;

pub struct LightShader(pub Shader);

impl ShaderSys for LightShader {
    fn get_system() -> Box<dyn Schedulable> {
        SystemBuilder::new("Light ShaderSystem")
            .write_resource::<LightShader>()
            .read_resource::<Camera>()
            .with_query(
                <(Read<Transform>, Read<Model>, Read<PointLight>)>::query()
                    .filter(tag::<LightTag>()),
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
                    shader.0.set_mat4(
                        &CString::new("model").unwrap(),
                        &transform.get_model_matrix(),
                    );
                    model.draw(&mut shader.0);
                }
            })
    }
}
