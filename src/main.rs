use cgmath::Vector3;
use cgmath::{vec3, Deg};
use cgmath::{Matrix4, Point3};
use glfw::{Action, Context, Glfw, Key};

use std::ffi::CString;
use std::sync::mpsc::Receiver;

const SRC_WIDHT: u32 = 1600;
const SRC_HEIGHT: u32 = 1200;

mod camera;
//mod cube;
mod lighting;
pub mod macros;
mod mesh;
mod model;
mod shader;

use camera::Camera;
use lighting::directional_light::DirectionalLight;
use lighting::point_light::PointLight;
use lighting::spotlight::SpotLight;
use lighting::Lighting;
use model::Model;
use shader::Shader;

use legion::prelude::*;

use crate::lighting::{LightColor, PointLightTag, SpotLightTag, Strength};
use std::borrow::BorrowMut;

pub struct Transform {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
}

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[inline]
fn to_vec(point: &Point3<f32>) -> Vector3<f32> {
    Vector3::new(point.x, point.y, point.z)
}
// shader trait render is a method???? camera is a resource
unsafe fn render(shader: &mut Shader, camera: &Camera, world: &mut World) {
    shader.use_program();
    shader.set_vector3(
        &CString::new("viewPos").unwrap(),
        &to_vec(&camera.get_position()),
    );
    shader.set_mat4(
        &CString::new("projection").unwrap(),
        &camera.get_projection_matrix(),
    );
    shader.set_mat4(&CString::new("view").unwrap(), &camera.get_view_matrix());
    //shader.set_uniforms(&mut world);// ^---- alla light uniforms måste sättas här
    let query = <(Read<Transform>, Read<PointLight>)>::query().filter(tag::<Light>());
    let mut light_count = 0;
    for (i, (transform, point_light)) in query.iter(world).enumerate() {
        point_light.set_uniforms(shader, i, &transform);
        light_count += 1;
    }
    shader.set_int(
        &CString::new("number_of_point_lights").unwrap(),
        light_count,
    );
    //let query = <Read<DirectionalLight>>::query().filter(tag::<Light>());
    //if let Some(directional_light) = query.iter(world).next() {
    //  directional_light.set_uniforms(&mut shader);
    //}

    let query = <(Read<Transform>, Read<Model>)>::query().filter(!tag::<Light>());
    for (transform, model) in query.iter(world) {
        let transform_matrix = Matrix4::from_translation(transform.position)
            * Matrix4::from_nonuniform_scale(
                transform.scale.x,
                transform.scale.y,
                transform.scale.z,
            );
        shader.set_mat4(&CString::new("model").unwrap(), &transform_matrix);
        model.draw(shader);
    }
}

unsafe fn render_lights(shader: &mut Shader, camera: &Camera, world: &mut World) {
    shader.use_program();
    shader.set_mat4(
        &CString::new("projection").unwrap(),
        &camera.get_projection_matrix(),
    );
    shader.set_mat4(&CString::new("view").unwrap(), &camera.get_view_matrix());
    let query = <(Read<Transform>, Read<Model>)>::query().filter(tag::<Light>());
    for (transform, model) in query.iter(world) {
        let transform_matrix = Matrix4::from_translation(transform.position)
            * Matrix4::from_nonuniform_scale(
                transform.scale.x,
                transform.scale.y,
                transform.scale.z,
            );

        shader.set_mat4(&CString::new("model").unwrap(), &transform_matrix);
        model.draw(shader);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Light;

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
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }
    let universe = Universe::new();
    let mut world = universe.create_world();
    let mut light_shader = Shader::new(
        "src/light_vertex_shader.shader",
        "src/light_fragment_shader.shader",
    )
    .expect("Failed to create light shaders");
    let mut shader = Shader::new("src/vertex_shader.shader", "src/fragment_shader.shader").unwrap();
    let mut camera = Camera::new(Point3::new(0., 0., 3.), vec3(0., 0., -1.));
    world.insert(
        (),
        vec![(
            Transform {
                position: vec3(0.0, -1.75, 0.0),
                scale: vec3(0.2, 0.2, 0.2),
            },
            Model::new("nanosuit/nanosuit.obj"),
        )],
    );

    let light_positions = vec![
        vec3(0.7, 0.2, 2.0),
        vec3(2.3, -3.3, -4.0),
        vec3(-4.0, 2.0, -12.0),
        vec3(0.0, 0.0, -3.0),
    ];
    //world.insert((DirectionalLight), vec![(Direction(), LightColor::default())]);
    //world.insert((SpotLightTag), vec![(Transform::default(), LightColor::default(), Strength::Medium, CutOff::default())]);
    world.insert(
        (Light, ()), // <--- maybe shader tag here?
        light_positions.iter().map(|&position| {
            (
                Transform {
                    position,
                    scale: vec3(0.5, 0.5, 0.5),
                },
                Model::cube(),
                PointLight::default(),
            )
        }),
    );
    let cube_positions = vec![
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
    ];

    world.insert(
        (),
        cube_positions.iter().map(|&position| {
            (
                Model::cube(),
                Transform {
                    position,
                    scale: vec3(1.0, 1.0, 1.0),
                },
            )
        }),
    );

    //let mut lighting = Lighting::new();

    //light_positions.iter().for_each(|light_pos| {
    //  lighting
    //    .point_lights
    //  .push(PointLight::default().set_position(*light_pos));
    //});

    let mut first_mouse = true;
    let mut last_x = (SRC_WIDHT / 2) as f32;
    let mut last_y = (SRC_HEIGHT / 2) as f32;
    let mut last_frame = 0.0;

    while !window.should_close() {
        let current_frame = glfw.get_time() as f32;
        let delta_time = current_frame - last_frame;
        last_frame = current_frame;

        //  let mut camera = resources.get_mut::<Camera>().unwrap();

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
            render(&mut shader, &camera, &mut world);
            render_lights(&mut light_shader, &camera, &mut world);
            //shader.use_program();

            //lighting.set_uniforms(&mut shader);

            //lighting.draw(&camera.get_projection_matrix(), &camera.get_view_matrix());
        }
        //schedule.execute(&mut world, &mut resources);
        window.swap_buffers();
        glfw.poll_events();
    }
}

/**
    1. create render function using esc and querying
    shaders are resources
    Components:
    Strenght,
    Colours (ambient, diffuse, specular),
    position,
    direction,
    (mesh),

    log är en feature flag

    queries:
    en för varje ljus?
    tag är ljus typ
    // set uniforms
    format!(tag.tostring, [index], colour.ambient ...)

    set rätt uniforms
    rendrera


    shaders are resources
    model = component
    transforms = component

**/

const CAMERA_SPEED: f32 = 4.5;

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
        camera.move_sideways(-CAMERA_SPEED * delta_time);
    }

    if window.get_key(Key::D) == Action::Press {
        camera.move_sideways(CAMERA_SPEED * delta_time);
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
