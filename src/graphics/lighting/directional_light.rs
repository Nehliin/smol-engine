use super::Light;
use nalgebra::{Matrix4, Orthographic3, Point3, Vector3};
use once_cell::sync::Lazy;
use zerocopy::AsBytes;

static DIRECTIONAL_PROJECTION: Lazy<Orthographic3<f32>> =
    Lazy::new(|| Orthographic3::new(-10.0, 10.0, -10.0, 10.0, 1.0, 100.0));

#[derive(Debug)]
pub struct DirectionalLight {
    pub ambient: Vector3<f32>,
    pub specular: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub target_view: Option<wgpu::TextureView>,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        DirectionalLight {
            direction: -Vector3::y(),
            ambient: Vector3::new(0.01, 0.01, 0.01),
            specular: Vector3::new(1.0, 1.0, 1.0),
            diffuse: Vector3::new(1.0, 1.0, 1.0),
            target_view: None,
        }
    }
}

impl Light for DirectionalLight {
    fn get_target_view(&self) -> &Option<wgpu::TextureView> {
        &self.target_view
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, AsBytes)]
pub struct DirectionalLightRaw {
    ambient: [f32; 3],
    _pad1: f32,
    specular: [f32; 3],
    _pad2: f32,
    diffuse: [f32; 3],
    _pad3: f32,
    direction: [f32; 3],
    pub light_space_matrix: [[f32; 4]; 4], //todo are these really necessary if you don't use as bytes anyways?
}

impl From<&DirectionalLight> for DirectionalLightRaw {
    fn from(light: &DirectionalLight) -> Self {
        let dir = -light.direction;
        let position = Point3::new(dir.x, dir.y, dir.z) * 10.0;
        let light_view = Matrix4::look_at_rh(&position, &Point3::origin(), &Vector3::y());
        let light_space_matrix = DIRECTIONAL_PROJECTION.to_homogeneous() * light_view;
        let light_space_matrix = light_space_matrix
            .as_slice()
            .chunks(4)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
            .collect::<Vec<[f32; 4]>>();

        DirectionalLightRaw {
            ambient: [light.ambient.x, light.ambient.y, light.ambient.z],
            specular: [light.specular.x, light.specular.y, light.specular.z],
            diffuse: [light.diffuse.x, light.diffuse.y, light.diffuse.z],
            direction: [light.direction.x, light.direction.y, light.direction.z],
            light_space_matrix: [
                light_space_matrix[0],
                light_space_matrix[1],
                light_space_matrix[2],
                light_space_matrix[3],
            ],
            _pad1: 0.0,
            _pad2: 0.0,
            _pad3: 0.0,
        }
    }
}
