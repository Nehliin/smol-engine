use crate::camera::Camera;
use crate::components::Selected;
use crate::components::{LightTag, Transform};
//use crate::lighting::directional_light::DirectionalLight;
use crate::lighting::point_light::PointLight;
use crate::model::Model;
use crate::shaders::Shader;
use crate::shaders::ShaderSys;
use legion::prelude::*;
use std::ffi::CString;

pub struct ModelShader(pub Shader);

impl ShaderSys for ModelShader {
    fn get_system() -> Box<dyn Schedulable> {
        SystemBuilder::new("Model ShaderSystem")
            .write_resource::<ModelShader>() //TODO: EN samlad resurs med alla shaders istället??
            .read_resource::<Camera>()
            .with_query(
                <(Read<Transform>, Read<Model>)>::query()
                    .filter(tag::<Selected>() & !tag::<LightTag>()),
            )
            .with_query(
                <(Read<Transform>, Read<Model>)>::query()
                    .filter(!tag::<Selected>() & !tag::<LightTag>()),
            )
            .with_query(<(Read<Transform>, Read<PointLight>)>::query().filter(tag::<LightTag>()))
            .build(
                |_,
                 world,
                 (shader, camera),
                 (selected_model_query, non_selected_model_query, uniform_query)| unsafe {
                    // keep stencil value if anything fails, replace with
                    // value from stencil buffer if it passes
                    // (also implicit enable of depth test here, it's the engine default)
                    gl::StencilOp(gl::KEEP, gl::KEEP, gl::REPLACE);
                    // Stencil test should always succeed and set value to 1
                    gl::StencilFunc(gl::ALWAYS, 1, 0xFF);

                    shader.0.use_program();
                    shader.0.set_vector3(
                        &CString::new("viewPos").unwrap(),
                        &camera.get_vec_position(),
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
                    // Non selected models shouldn't write to the stencil buffer!
                    gl::StencilMask(0x00);
                    for (transform, model) in non_selected_model_query.iter(world) {
                        shader.0.set_mat4(
                            &CString::new("model").unwrap(),
                            &transform.get_model_matrix(),
                        );
                        model.draw(&mut shader.0);
                    }

                    //Selected models should write to the stencil buffer though
                    gl::StencilMask(0xFF);
                    for (transform, model) in selected_model_query.iter(world) {
                        shader.0.set_mat4(
                            &CString::new("model").unwrap(),
                            &transform.get_model_matrix(),
                        );
                        model.draw(&mut shader.0);
                    }
                },
            )
    }
}
