use crate::camera::{Camera, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::components::Selected;
use crate::components::{LightTag, Transform};
use crate::engine::{InputEvent, Time};
use crate::lighting::{DirectionalLight, PointLight};
use crate::model::Model;
use cgmath::vec3;
use glfw::{Action, Key};
use legion::prelude::*;
use std::collections::HashMap;

pub trait State {
    // resources??
    fn start(&mut self, world: &mut World, resources: &mut Resources);
    fn update(&mut self, world: &mut World, resources: &mut Resources); // -> transition
    fn stop(&mut self, world: &mut World, resources: &mut Resources);
    fn handle_event(
        &mut self,
        event: InputEvent,
        world: &mut World,
        resources: &mut Resources,
    ) -> bool;
}

pub struct BasicState {
    systems: Vec<Box<dyn Schedulable>>,
    first_mouse: bool,
    last_x: f32,
    last_y: f32,
    key_down_map: HashMap<Key, bool>,
}

impl BasicState {
    pub fn new() -> Self {
        BasicState {
            systems: Vec::new(),
            first_mouse: true,
            last_y: (WINDOW_HEIGHT / 2) as f32, // TODO: ugly
            last_x: (WINDOW_WIDTH / 2) as f32,  // TODO: ugly
            key_down_map: HashMap::new(),
        }
    }
}
const CAMERA_SPEED: f32 = 4.5;

impl State for BasicState {
    fn start(&mut self, world: &mut World, _resources: &mut Resources) {
        let light_positions = vec![
            vec3(0.7, 0.2, 2.0),
            vec3(2.3, -3.3, -4.0),
            vec3(-4.0, 2.0, -12.0),
            vec3(0.0, 0.0, -3.0),
        ];

        world.insert(
            (), // selected
            vec![(
                Transform {
                    position: vec3(0.0, -1.75, 0.0),
                    scale: vec3(0.2, 0.2, 0.2),
                    rotation: vec3(1.0, 1.0, 1.0),
                    angle: 0.0,
                },
                Model::new("nanosuit/nanosuit.obj"),
            )],
        );

        world.insert(
            (LightTag, ()),
            vec![(
                DirectionalLight::default().set_diffuse(vec3(0.0, 0.0, 1.0)),
                (),
            )],
        );

        world.insert(
            (LightTag, ()), // <--- maybe shader tag here?
            light_positions.iter().map(|&position| {
                (
                    Transform {
                        position,
                        scale: vec3(0.5, 0.5, 0.5),
                        rotation: vec3(1.0, 1.0, 1.0),
                        angle: 0.0,
                    },
                    Model::cube(),
                    PointLight::default(),
                )
            }),
        );

        let cube_positions = vec![
            vec3(0.0, -3.0, 0.0),
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
                        rotation: vec3(1.0, 1.0, 1.0),
                        angle: 0.0,
                    },
                )
            }),
        );

        let floor = Model::cube();
        let floor_transform = Transform {
            position: vec3(0.0, -5.0, -2.0),
            scale: vec3(0.1, 10.0, 10.0),
            rotation: vec3(0.0, 0.0, 1.0),
            angle: 90_f32,
        };

        world.insert((), vec![(floor, floor_transform)]);
    }

    fn update(&mut self, world: &mut World, resources: &mut Resources) {
        self.systems
            .iter_mut()
            .for_each(|system| system.run(world, resources));
    }

    fn stop(&mut self, _world: &mut World, _resources: &mut Resources) {
        unimplemented!()
    }

    // bool should be transition?
    fn handle_event(
        &mut self,
        event: InputEvent,
        _world: &mut World,
        resources: &mut Resources,
    ) -> bool {
        match event {
            InputEvent::KeyAction { key, action } => {
                if key == Key::Escape {
                    return true;
                }
                let time = resources.get::<Time>().unwrap();
                let mut camera = resources.get_mut::<Camera>().unwrap();
                if action == Action::Press {
                    self.key_down_map.insert(key, true);
                }

                if let Some(true) = self.key_down_map.get(&Key::W) {
                    camera.move_in_direction(CAMERA_SPEED * time.delta_time);
                }
                if let Some(true) = self.key_down_map.get(&Key::S) {
                    camera.move_in_direction(-CAMERA_SPEED * time.delta_time);
                }
                if let Some(true) = self.key_down_map.get(&Key::A) {
                    camera.move_sideways(-CAMERA_SPEED * time.delta_time);
                }
                if let Some(true) = self.key_down_map.get(&Key::D) {
                    camera.move_sideways(CAMERA_SPEED * time.delta_time);
                }
                if action == Action::Release {
                    self.key_down_map.insert(key, false);
                }

                false
            }
            InputEvent::CursorMovement { x_pos, y_pos } => {
                let mut camera = resources.get_mut::<Camera>().unwrap();
                let (xpos, ypos) = (x_pos as f32, y_pos as f32);
                if self.first_mouse {
                    self.last_x = xpos;
                    self.last_y = ypos;
                    self.first_mouse = false;
                }

                let mut xoffset = xpos - self.last_x;
                let mut yoffset = self.last_y - ypos; // reversed since y-coordinates go from bottom to top
                self.last_x = xpos;
                self.last_y = ypos;

                let sensitivity: f32 = 0.001; // change this value to your liking
                xoffset *= sensitivity;
                yoffset *= sensitivity;
                let yaw = camera.get_yaw();
                let pitch = camera.get_pitch();
                camera.set_yaw(xoffset + yaw);
                camera.set_pitch(yoffset + pitch);
                false
            }
        }
    }
}
