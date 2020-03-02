use cgmath::{vec3, Deg};
use cgmath::{Matrix4, Point3};
use glfw::{Action, Context, Key};

use std::ffi::CString;
use std::sync::mpsc::Receiver;

const SRC_WIDHT: u32 = 1600;
const SRC_HEIGHT: u32 = 1200;

mod camera;
mod lighting;
mod macros;
mod mesh;
mod model;
mod shader;

use camera::Camera;
use lighting::directional_light::DirectionalLight;
use lighting::point_light::PointLight;
use lighting::Lighting;
use model::Model;
use shader::Shader;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw
        .create_window(
            SRC_WIDHT,
            SRC_HEIGHT,
            "Smol-Engine",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create window");

    window.make_current();
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_cursor_mode(glfw::CursorMode::Disabled);
    window.set_framebuffer_size_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    let mut shader_program =
        Shader::new("src/vertex_shader.shader", "src/fragment_shader.shader").unwrap();

    let light_positions = vec![
        vec3(0.7, 0.2, 2.0),
        vec3(2.3, -3.3, -4.0),
        vec3(-4.0, 2.0, -12.0),
        vec3(0.0, 0.0, -3.0),
    ];
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }
    let mut lighting = Lighting::new();
    lighting.set_directional_light(
        DirectionalLight::default()
            .set_diffuse(vec3(0.0, 1.0, 0.0))
            .set_direction(vec3(-0.2, -1.0, -0.3)),
    );
    light_positions.iter().for_each(|light_pos| {
        lighting.add_point_light(PointLight::default().set_position(*light_pos));
    });

    let nano_suite_model = Model::new("nanosuit/nanosuit.obj");

    //let model_matrix = Matrix4::from_angle_x(Deg(-55.0)); //* Matrix::identity();
    let mut camera = Camera::new(Point3::new(0., 0., 3.), vec3(0., 0., -1.));
    let projection_matrix =
        cgmath::perspective(Deg(45.0), SRC_WIDHT as f32 / SRC_HEIGHT as f32, 0.1, 100.0);
    //let test = CString::new("offset").unwrap();
    //let light_position = vec3(1.2, 1.0, 2.0);
    /* let cube_positions = vec![
        vec3(0.0, 0.0, 0.0),
        vec3(2.0, 5.0, -15.0),
        vec3(-1.5, -2.2, -2.5),
        vec3(-3.8, -2.0, -12.0),
        vec3(2.4, -0.4, -3.5),
        vec3(-1.7, 3.0, -7.5),
        vec3(1.3, -2.0, -2.5),
        vec3(1.5, 2.0, -2.5),
        vec3(1.5, 0.2, -1.5),
        vec3(-1.3, 1.0, -1.5),
    ];*/
    let mut first_mouse = true;
    let mut last_x = (SRC_WIDHT / 2) as f32;
    let mut last_y = (SRC_HEIGHT / 2) as f32;
    let mut last_frame = 0.0;
    while !window.should_close() {
        let current_frame = glfw.get_time() as f32;
        let delta_time = current_frame - last_frame;
        last_frame = current_frame;
        process_events(
            &events,
            &mut camera,
            &mut first_mouse,
            &mut last_x,
            &mut last_y,
        );
        process_input(&mut window, &mut camera, delta_time);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // let time = glfw.get_time() as f32;

            shader_program.use_program();
            shader_program.set_vec3(
                &CString::new("viewPos").unwrap(),
                camera.get_position().x,
                camera.get_position().y,
                camera.get_position().z,
            );

            shader_program.set_float(&CString::new("material.shininess").unwrap(), 32.0);
            lighting.set_uniforms(&mut shader_program);
            shader_program.set_mat4(&CString::new("projection").unwrap(), &projection_matrix);
            shader_program.set_mat4(&CString::new("view").unwrap(), &camera.get_view_matrix());

            //    shader_program.set_float(&test, time.sin());
            let mut model = Matrix4::<f32>::from_translation(vec3(0.0, -1.75, 0.0)); // translate it down so it's at the center of the scene
            model = model * Matrix4::from_scale(0.2); // it's a bit too big for our scene, so scale it down
            shader_program.set_mat4(c_str!("model"), &model);
            nano_suite_model.draw(&mut shader_program);

            lighting.draw(&projection_matrix, &camera.get_view_matrix());
        }
        window.swap_buffers();
        glfw.poll_events();
    }
}

const CAMERA_SPEED: f32 = 2.5;

fn process_input(window: &mut glfw::Window, camera: &mut Camera, delta_time: f32) {
    if window.get_key(Key::Escape) == Action::Press {
        window.set_should_close(true);
    }

    if window.get_key(Key::W) == Action::Press {
        camera.move_in_direction(CAMERA_SPEED * delta_time);
    }

    if window.get_key(Key::S) == Action::Press {
        camera.move_in_direction(-CAMERA_SPEED * delta_time);
    }

    if window.get_key(Key::A) == Action::Press {
        camera.move_sidways(-CAMERA_SPEED * delta_time);
    }

    if window.get_key(Key::D) == Action::Press {
        camera.move_sidways(CAMERA_SPEED * delta_time);
    }
}

fn process_events(
    events: &Receiver<(f64, glfw::WindowEvent)>,
    camera: &mut Camera,
    first_mouse: &mut bool,
    last_x: &mut f32,
    last_y: &mut f32,
) {
    glfw::flush_messages(events).for_each(|(_, event)| match event {
        glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
            gl::Viewport(0, 0, width, height)
        },
        glfw::WindowEvent::CursorPos(xpos, ypos) => {
            let (xpos, ypos) = (xpos as f32, ypos as f32);
            if *first_mouse {
                *last_x = xpos;
                *last_y = ypos;
                *first_mouse = false;
            }

            let mut xoffset = xpos - *last_x;
            let mut yoffset = *last_y - ypos; // reversed since y-coordinates go from bottom to top
            *last_x = xpos;
            *last_y = ypos;

            let sensitivity: f32 = 0.001; // change this value to your liking
            xoffset *= sensitivity;
            yoffset *= sensitivity;

            camera.set_yaw(xoffset + camera.get_yaw());
            camera.set_pitch(yoffset + camera.get_pitch())
        }
        _ => {}
    })
}
