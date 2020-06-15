use nalgebra::geometry::Perspective3;
use nalgebra::{Matrix4, Point3, Vector3};
use smol_renderer::GpuData;

//const UP: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);

pub struct Camera {
    direction: Vector3<f32>,
    position: Point3<f32>,
    view_matrix: Matrix4<f32>,
    projection_matrix: Perspective3<f32>,
    pitch: f32,
    yaw: f32,
}

#[repr(C)]
#[derive(GpuData)]
pub struct CameraUniform {
    view_matrix: Matrix4<f32>,
    projection: Matrix4<f32>,
    view_pos: Vector3<f32>,
}

impl From<Camera> for CameraUniform {
    fn from(camera: Camera) -> Self {
        assert!(std::mem::size_of::<Matrix4<f32>>() == std::mem::size_of::<[[f32;4];4]>());
        assert!(std::mem::size_of::<Vector3<f32>>() == std::mem::size_of::<[f32;3]>());
        CameraUniform {
            view_matrix: camera.view_matrix,
            projection: camera.get_projection_matrix(),
            view_pos: camera.get_vec_position()
        }
    }
}

#[inline]
fn to_vec(point: &Point3<f32>) -> Vector3<f32> {
    Vector3::new(point.x, point.y, point.z)
}

impl Camera {
    pub fn new(
        position: Point3<f32>,
        direction: Vector3<f32>,
        window_width: u32,
        window_height: u32,
    ) -> Self {
        // what POINT should the camera look at?
        let view_target = position + direction;
        Camera {
            direction,
            position,
            view_matrix: Matrix4::look_at_rh(&position, &view_target, &Vector3::new(0.0, 1.0, 0.0)),
            projection_matrix: Perspective3::new(
                window_width as f32 / window_height as f32,
                45.0,
                0.1,
                100.0,
            ),
            yaw: -90.0,
            pitch: 0.0,
        }
    }

    #[inline]
    pub fn get_vec_position(&self) -> Vector3<f32> {
        to_vec(&self.position)
    }

    #[inline]
    pub fn move_in_direction(&mut self, amount: f32) {
        self.position += self.direction * amount;
        self.view_matrix = Matrix4::look_at_rh(
            &self.position,
            &(self.position + self.direction),
            &Vector3::new(0.0, 1.0, 0.0),
        );
    }

    #[inline]
    pub fn move_sideways(&mut self, amount: f32) {
        self.position += self
            .direction
            .cross(&Vector3::new(0.0, 1.0, 0.0))
            .normalize()
            * amount;
        self.view_matrix = Matrix4::look_at_rh(
            &self.position,
            &(self.position + self.direction),
            &Vector3::new(0.0, 1.0, 0.0),
        );
    }

    #[inline]
    pub fn get_view_matrix(&self) -> &Matrix4<f32> {
        &self.view_matrix
    }

    #[inline]
    pub fn get_projection_matrix(&self) -> &Matrix4<f32> {
        &self.projection_matrix.as_matrix()
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
        self.direction.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        self.direction.y = self.pitch.to_radians().sin();
        self.direction.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();
        self.direction.normalize_mut();

        self.view_matrix = Matrix4::look_at_rh(
            &self.position,
            &(self.position + self.direction),
            &Vector3::new(0.0, 1.0, 0.0),
        );
    }
}
