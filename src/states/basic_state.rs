use crate::camera::Camera;
//use crate::components::Selected;
use crate::components::{AssetManager, Cube, LightTag, ModelHandle, Transform};
use crate::engine::{InputEvent, Time, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::physics::Physics;
use glfw::{Action, Key};
use legion::prelude::*;
use nalgebra::{Isometry3, Vector3};

use nphysics3d::object::BodyStatus;
use std::collections::HashMap;

use super::State;
use crate::graphics::model::Model;

pub struct BasicState {
    schedule: Option<Schedule>,
    first_mouse: bool,
    last_x: f32,
    last_y: f32,
    key_down_map: HashMap<Key, bool>,
}

impl BasicState {
    pub fn new() -> Self {
        BasicState {
            schedule: None,
            first_mouse: true,
            last_y: (WINDOW_HEIGHT / 2) as f32, // TODO: ugly
            last_x: (WINDOW_WIDTH / 2) as f32,  // TODO: ugly
            key_down_map: HashMap::new(),
        }
    }
}
const CAMERA_SPEED: f32 = 4.5;

impl State for BasicState {
    fn start(&mut self, world: &mut World, resources: &mut Resources) {
        let suit_handle = ModelHandle { id: 0 };
        let cube_handle = ModelHandle { id: 1 };
        /*let physicis = Physics::new(resources);
        let schedule = Schedule::builder().add_system(physicis.system).build();

        self.schedule = Some(schedule);

        let light_positions = vec![
            Vector3::new(0.7, 0.2, 2.0),
            Vector3::new(2.3, -3.3, -4.0),
            Vector3::new(-4.0, 2.0, -12.0),
            Vector3::new(0.0, 0.0, -3.0),
        ];

        world.insert(
            (), // selected
            vec![(
                Transform::new(
                    Isometry3::translation(0.0, -1.75, 0.0),
                    Vector3::new(0.2, 0.2, 0.2),
                ),
                Model::new("nanosuit/nanosuit.obj"),
            )],
        );

        world.insert(
            (LightTag, ()),
            vec![(
                DirectionalLight::default().set_diffuse(Vector3::new(0.0, 0.0, 1.0)),
                (),
            )],
        );

        world.insert(
            (LightTag, ()), // <--- maybe shader tag here?
            light_positions.iter().map(|&position| {
                (
                    Transform::new(
                        Isometry3::translation(position.x, position.y, position.z),
                        Vector3::new(0.5, 0.5, 0.5),
                    ),
                    Model::cube(),
                    PointLight::default(),
                )
            }),
        );

        let cube_positions = vec![
            Vector3::new(0.0, -3.0, 0.0),
            Vector3::new(2.0, 5.0, -15.0),
            Vector3::new(-1.5, -2.2, -2.5),
            Vector3::new(-3.8, -2.0, -12.0),
            Vector3::new(2.4, -0.4, -3.5),
            Vector3::new(-1.7, 3.0, -7.5),
            Vector3::new(1.3, -2.0, -2.5),
            Vector3::new(1.5, 2.0, -2.5),
            Vector3::new(1.5, 0.2, -1.5),
            Vector3::new(-1.3, 1.0, -1.5),
        ];

        world.insert(
            (),
            cube_positions.iter().map(|&position| {
                let transform = Transform::from_position(position);
                (
                    Model::cube(),
                    Physics::create_cube(resources, &transform, BodyStatus::Dynamic),
                    transform,
                )
            }),
        );

        let floor = Model::cube();
        let floor_transform = Transform::new(
            Isometry3::new(
                Vector3::new(0.0, -5.0, -2.0),
                Vector3::z() * 90.0_f32.to_radians(),
            ),
            Vector3::new(0.1, 10.0, 10.0),
        );

        world.insert(
            (),
            vec![(
                floor,
                Physics::create_cube(resources, &floor_transform, BodyStatus::Static),
                floor_transform,
            )],
        );*/

        world.insert(
            (suit_handle, ()), // selected
            vec![(Transform::new(
                Isometry3::translation(0.0, -1.75, 0.0),
                Vector3::new(0.2, 0.2, 0.2),
            ),)],
        );
        let floor_transform = Transform::new(
            Isometry3::new(
                Vector3::new(0.0, -5.0, -2.0),
                Vector3::z() * 90.0_f32.to_radians(),
            ),
            Vector3::new(0.1, 10.0, 10.0),
        );
        world.insert((cube_handle, ()), vec![(floor_transform,)]);

        let cube_positions = vec![
            Vector3::new(0.0, -3.0, 0.0),
            Vector3::new(2.0, 5.0, -15.0),
            Vector3::new(-1.5, -2.2, -2.5),
            Vector3::new(-3.8, -2.0, -12.0),
            Vector3::new(2.4, -0.4, -3.5),
            Vector3::new(-1.7, 3.0, -7.5),
            Vector3::new(1.3, -2.0, -2.5),
            Vector3::new(1.5, 2.0, -2.5),
            Vector3::new(1.5, 0.2, -1.5),
            Vector3::new(-1.3, 1.0, -1.5),
        ];

        world.insert(
            (cube_handle, ()),
            cube_positions.iter().map(|&position| {
                let transform = Transform::new(
                    Isometry3::translation(position.x, position.y, position.z),
                    Vector3::new(0.7, 0.7, 0.7),
                );
                (transform,)
            }),
        );
    }

    fn update(&mut self, world: &mut World, resources: &mut Resources) {
        /* self.schedule
        .as_mut()
        .expect("to be initializes")
        .execute(world, resources);*/
    }

    fn stop(&mut self, _world: &mut World, _resources: &mut Resources) {
        unimplemented!()
    }

    // bool should be transition?
    fn handle_event(
        &mut self,
        event: InputEvent,
        world: &mut World,
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
            InputEvent::MouseButton { button, action } => {
                /*   if action == Action::Press {
                    let transform = {
                        let camera = resources.get::<Camera>().unwrap();
                        Transform::from_position(Vector3::new(
                            camera.get_position().x,
                            camera.get_position().y,
                            camera.get_position().z - 3.0,
                        ))
                    };
                    world.insert(
                        (),
                        vec![(
                            Physics::create_sphere(resources, &transform, BodyStatus::Dynamic, 1.0),
                            transform,
                            Model::sphere(2.0),
                        )],
                    );
                }*/
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

                let sensitivity: f32 = 0.05; // change this value to your liking
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
