use nalgebra::{geometry::Orthographic3, Matrix4, Point3, Vector3};
use once_cell::sync::Lazy;
use smol_renderer::GpuData;

static DIRECTIONAL_PROJECTION: Lazy<Orthographic3<f32>> =
    Lazy::new(|| Orthographic3::new(-10.0, 10.0, -10.0, 10.0, 1.0, 100.0));

pub struct PointLight {
    pub ambient: Vector3<f32>,
    pub specular: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
    pub target_view: Option<wgpu::TextureView>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, GpuData)]
pub struct PointLightRaw {
    position: [f32; 3],
    _pad: f32,
    ambient: [f32; 3],
    _pad1: f32,
    specular: [f32; 3],
    _pad2: f32,
    diffuse: [f32; 3],
    constant: f32,
    linear: f32,
    quadratic: f32,
    _pad3: f32,
    _pad4: f32,
    pub light_space_matrix: [[f32; 4]; 4], //todo are these really necessary if you don't use as bytes anyways?
}

#[repr(C)]
#[derive(GpuData)]
pub struct PointLightUniform {
    pub lights_used: i32,
    pub _pad: [i32; 3],
    pub point_lights: [PointLightRaw; 16],
}

impl From<(&PointLight, Vector3<f32>)> for PointLightRaw {
    fn from((light, position): (&PointLight, Vector3<f32>)) -> Self {
        let view = Matrix4::look_at_rh(
            &Point3::new(position.x, position.y, position.z),
            &Point3::new(0.0, 0.0, 0.0),
            &Vector3::y(),
        );
        let light_space_matrix = DIRECTIONAL_PROJECTION.to_homogeneous() * view;
        let projection = light_space_matrix
            .as_slice()
            .chunks(4)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
            .collect::<Vec<[f32; 4]>>();

        PointLightRaw {
            position: [position.x, position.y, position.z],
            ambient: [light.ambient.x, light.ambient.y, light.ambient.z],
            specular: [light.specular.x, light.specular.y, light.specular.z],
            diffuse: [light.diffuse.x, light.diffuse.y, light.diffuse.z],
            constant: light.constant,
            linear: light.linear,
            quadratic: light.quadratic,
            light_space_matrix: [projection[0], projection[1], projection[2], projection[3]],
            _pad: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            _pad3: 0.0,
            _pad4: 0.0,
        }
    }
}

impl Default for PointLight {
    fn default() -> Self {
        let constant = 1.0;
        let linear = 0.09;
        let quadratic = 0.032;

        PointLight {
            ambient: Vector3::new(0.01, 0.01, 0.01),
            specular: Vector3::new(1.0, 1.0, 1.0),
            diffuse: Vector3::new(1.0, 1.0, 1.0),
            constant,
            linear,
            quadratic,
            target_view: None,
        }
    }
}
