use crate::camera::Camera;
use crate::graphics::Renderer;
use crate::state::State;
use cgmath::{vec3, Point3};
use glfw::{Action, Context, Glfw, Key, Window, WindowEvent};
use legion::prelude::*;
use std::sync::mpsc::Receiver;

//TODO: move this
pub enum InputEvent {
    KeyAction { key: Key, action: Action },
    CursorMovement { x_pos: f64, y_pos: f64 },
}

pub struct Time {
    pub time: f32,
}

pub struct Engine {
    renderer: Box<dyn Renderer>,
    current_state: Box<dyn State>,
    // ECS
    world: World,
    resources: Resources,
    //Window
    glfw: Glfw,
    window: Window,
    events: Receiver<(f64, WindowEvent)>,
}

pub const WINDOW_HEIGHT: u32 = 800;
pub const WINDOW_WIDTH: u32 = 1200;

impl Engine {
    // TODO: make builder instead
    pub fn new(
        name: impl AsRef<str>,
        start_state: Box<dyn State>,
        mut renderer: Box<dyn Renderer>,
    ) -> Self {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        #[cfg(target_os = "macos")]
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        let (mut window, events) = glfw
            .create_window(
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                name.as_ref(),
                glfw::WindowMode::Windowed,
            )
            .expect("Failed to create window");

        window.make_current();
        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_cursor_mode(glfw::CursorMode::Disabled);
        window.set_framebuffer_size_polling(true);

        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        // ECS initialization
        let universe = Universe::new();
        let world = universe.create_world();

        let mut resources = Resources::default();
        resources.insert(Time {
            time: glfw.get_time() as f32,
        });
        renderer.init(&mut resources);
        println!("RENDERER INITIALIZED!");
        let camera = Camera::new(
            Point3::new(0., 0., 3.),
            vec3(0., 0., -1.),
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        );
        resources.insert(camera);
        Engine {
            renderer,
            current_state: start_state,
            world,
            resources,
            glfw,
            window,
            events,
        }
    }
    // Run the main game loop
    pub fn run(&mut self) {
        let mut last_frame = 0.0;
        self.current_state
            .start(&mut self.world, &mut self.resources);
        while !self.window.should_close() {
            unsafe {
                gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
            }
            let current_frame = self.glfw.get_time() as f32;
            let delta_time = current_frame - last_frame;
            last_frame = current_frame;
            {
                let mut time = self.resources.get_mut::<Time>().unwrap();
                time.time = delta_time;
            }
            self.process_events();
            self.current_state
                .update(&mut self.world, &mut self.resources);
            self.renderer
                .render_world(&mut self.world, &mut self.resources);
            self.window.swap_buffers();
            self.glfw.poll_events();
        }
    }

    fn process_events(&mut self) {
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
                    gl::Viewport(0, 0, width, height)
                },
                glfw::WindowEvent::Key(key, _, action, _) => {
                    if self.current_state.handle_event(
                        InputEvent::KeyAction { key, action },
                        &mut self.world,
                        &mut self.resources,
                    ) {
                        self.window.set_should_close(true);
                    }
                }
                glfw::WindowEvent::CursorPos(x_pos, y_pos) => {
                    self.current_state.handle_event(
                        InputEvent::CursorMovement { x_pos, y_pos },
                        &mut self.world,
                        &mut self.resources,
                    );
                }
                _ => {}
            }
        }
    }
}
