use crate::camera::Camera;
use crate::graphics::{Renderer, WgpuRenderer};
use crate::states::State;
use glfw::{Action, Context, Glfw, Key, MouseButton, Window, WindowEvent};
use legion::prelude::*;
use nalgebra::{Point3, Vector3};
use std::io::Write;
use std::sync::mpsc::Receiver;

//TODO: move this
pub enum InputEvent {
    KeyAction { key: Key, action: Action },
    CursorMovement { x_pos: f64, y_pos: f64 },
    MouseButton { button: MouseButton, action: Action },
}

pub struct Time {
    pub current_time: f32,
    pub delta_time: f32,
}

pub struct Engine {
    // renderer: R,
    tmp_wgpu_renderer: WgpuRenderer,
    current_state: Box<dyn State>,
    // ECS
    world: World,
    resources: Resources,
    //Window
    glfw: Glfw,
    window: Window,
    events: Receiver<(f64, WindowEvent)>,
}

pub const WINDOW_HEIGHT: u32 = 1200;
pub const WINDOW_WIDTH: u32 = 1600;

impl Engine {
    //noinspection ALL
    // TODO: make builder instead
    pub fn new(name: impl AsRef<str>, start_state: Box<dyn State>) -> Self {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        //    glfw.window_hint(glfw::WindowHint::Samples(Some(4))); <- better to do manually

        let (mut window, events) = glfw
            .create_window(
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                name.as_ref(),
                glfw::WindowMode::Windowed,
            )
            .expect("Failed to create window");

        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);
        window.set_cursor_mode(glfw::CursorMode::Disabled);
        window.set_framebuffer_size_polling(true);

        // ECS initialization
        let universe = Universe::new();
        let world = universe.create_world();

        let mut resources = Resources::default();
        resources.insert(Time {
            current_time: glfw.get_time() as f32,
            delta_time: 0.0,
        });
        let wgpu_renderer = futures::executor::block_on(WgpuRenderer::new(&window, &mut resources));

        let camera = Camera::new(
            Point3::new(0., 0., 3.),
            Vector3::new(0.0, 0.0, -1.0),
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        );
        resources.insert(camera);
        Engine {
            //       renderer,
            tmp_wgpu_renderer: wgpu_renderer,
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
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        while !self.window.should_close() {
            let current_frame = self.glfw.get_time() as f32;
            let delta_time = current_frame - last_frame;
            handle.write_fmt(format_args!("fps: {}\n", 1.0 / delta_time));
            last_frame = current_frame;
            {
                let mut time = self.resources.get_mut::<Time>().unwrap();
                time.delta_time = delta_time;
                time.current_time = current_frame;
            }
            self.process_events();
            self.current_state
                .update(&mut self.world, &mut self.resources);
            self.tmp_wgpu_renderer
                .render_frame(&mut self.world, &mut self.resources);
            self.glfw.poll_events();
        }
    }

    fn process_events(&mut self) {
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
                    self.tmp_wgpu_renderer.resize(width as u32, height as u32);
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
                glfw::WindowEvent::MouseButton(button, action, _) => {
                    self.current_state.handle_event(
                        InputEvent::MouseButton { button, action },
                        &mut self.world,
                        &mut self.resources,
                    );
                }
                _ => {}
            }
        }
    }
}
