use crate::camera::Camera;
use crate::components::{LightTag, Selected, Transform};
use crate::model::Model;
use crate::shaders::Shader;
use crate::shaders::ShaderSys;
use cgmath::prelude::*;
use cgmath::Matrix4;
use cgmath::Rad;
use legion::prelude::*;
use std::ffi::CString;

pub struct OutLineShader(pub Shader);

impl ShaderSys for OutLineShader {
    fn get_system() -> Box<dyn Schedulable> {
        SystemBuilder::new("OutLine ShaderSystem")
            .write_resource::<OutLineShader>() //TODO: EN samlad resurs med alla shaders istället??
            .read_resource::<Camera>()
            .with_query(
                <(Read<Transform>, Read<Model>)>::query()
                    .filter(tag::<Selected>() & !tag::<LightTag>()), // <-- try to remove if there are issues
            )
            .build(|_, world, (shader, camera), model_query| unsafe {
                // keep stencil value if anything fails, replace with
                // value from stencil buffer if it passes
                gl::StencilFunc(gl::NOTEQUAL, 1, 0xFF);
                gl::StencilMask(0x00);
                gl::Disable(gl::DEPTH_TEST);
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
                for (transform, model) in model_query.iter(world) {
                    let transform_matrix = Matrix4::from_translation(transform.position)
                        * Matrix4::from_axis_angle(
                            transform.rotation.normalize(),
                            Rad(transform.angle.to_radians()),
                        )
                        * Matrix4::from_nonuniform_scale(
                            transform.scale.x * 1.1,
                            transform.scale.y * 1.1,
                            transform.scale.z * 1.1,
                        );
                    shader
                        .0
                        .set_mat4(&CString::new("model").unwrap(), &transform_matrix);
                    model.draw(&mut shader.0);
                }
                gl::StencilMask(0xFF);
                gl::StencilFunc(gl::ALWAYS, 1, 0xFF);
                gl::Enable(gl::DEPTH_TEST);
            })
    }
}
