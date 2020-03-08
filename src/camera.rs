#![allow(dead_code)]

use crate::{SRC_HEIGHT, SRC_WIDHT};
use cgmath::prelude::*;
use cgmath::{vec3, Vector3};
use cgmath::{Deg, Rad};
use cgmath::{Matrix4, Point3};

const UP: Vector3<f32> = vec3(0., 1., 0.);

pub struct Camera {
    direction: Vector3<f32>,
    position: Point3<f32>,
    view_matrix: Matrix4<f32>,
    projection_matrix: Matrix4<f32>,
    pitch: f32,
    yaw: f32,
}

impl Camera {
    pub fn new(position: Point3<f32>, direction: Vector3<f32>) -> Self {
        // what POINT should the camera look at?
        let view_target = position + direction;
        Camera {
            direction,
            position,
            view_matrix: Matrix4::look_at(position, view_target, UP),
            projection_matrix: cgmath::perspective(
                Deg(45.0),
                SRC_WIDHT as f32 / SRC_HEIGHT as f32,
                0.1,
                100.0,
            ),
            yaw: -90.0,
            pitch: 0.0,
        }
    }

    #[inline]
    pub fn set_direction(&mut self, direction: Vector3<f32>) {
        self.direction = direction;
        self.view_matrix = Matrix4::look_at(self.position, self.position + self.direction, UP);
    }

    #[inline]
    pub fn get_position(&self) -> Point3<f32> {
        self.position
    }

    #[inline]
    pub fn get_direction(&self) -> Vector3<f32> {
        self.direction
    }

    #[inline]
    pub fn move_in_direction(&mut self, amount: f32) {
        self.position += self.direction * amount;
        self.view_matrix = Matrix4::look_at(self.position, self.position + self.direction, UP);
    }

    #[inline]
    pub fn move_sideways(&mut self, amount: f32) {
        self.position += self.direction.cross(UP).normalize() * amount;
        self.view_matrix = Matrix4::look_at(self.position, self.position + self.direction, UP);
    }

    #[inline]
    pub fn get_view_matrix(&self) -> &Matrix4<f32> {
        &self.view_matrix
    }

    #[inline]
    pub fn get_projection_matrix(&self) -> &Matrix4<f32> {
        &self.projection_matrix
    }

    #[inline]
    pub fn set_pitch(&mut self, pitch: f32) {
        if pitch < -89.0 {
            self.pitch = -89.0;
        } else if 89.0 < pitch {
            self.pitch = 89.0;
        } else {
            self.pitch = pitch;
        }
        self.update_rotation();
    }

    #[inline]
    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
        self.update_rotation();
    }

    #[inline]
    pub fn get_pitch(&self) -> f32 {
        self.pitch
    }

    #[inline]
    pub fn get_yaw(&self) -> f32 {
        self.yaw
    }

    #[inline]
    fn update_rotation(&mut self) {
        self.direction.x = Rad(self.yaw).cos() * Rad(self.pitch).cos();
        self.direction.y = Rad(self.pitch).sin();
        self.direction.z = Rad(self.yaw).sin() * Rad(self.pitch).cos();
        self.direction.normalize();

        self.view_matrix = Matrix4::look_at(self.position, self.position + self.direction, UP);
    }
}
