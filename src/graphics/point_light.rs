use nalgebra::Vector3;
use zerocopy::AsBytes;
use zerocopy::FromBytes;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PointLight {
    pub position: Vector3<f32>,
    pub ambient: Vector3<f32>,
    pub specular: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, AsBytes)]
pub struct PointLightRaw {
    position: [f32; 3],
    pad: f32,
    amibent: [f32; 3],
    pad1: f32,
    specular: [f32; 3],
    pad2: f32,
    diffuse: [f32; 3],
    constant: f32,
    linear: f32,
    quadratic: f32,
    pad3: f32,
    pad4: f32,
}

impl From<PointLight> for PointLightRaw {
    fn from(light: PointLight) -> Self {
        PointLightRaw {
            position: [light.position.x, light.position.y, light.position.z],
            amibent: [light.ambient.x, light.ambient.y, light.ambient.z],
            specular: [light.specular.x, light.specular.y, light.specular.z],
            diffuse: [light.diffuse.x, light.diffuse.y, light.diffuse.z],
            constant: light.constant,
            linear: light.linear,
            quadratic: light.quadratic,
            pad: 0.0,
            pad1: 0.0,
            pad2: 0.0,
            pad3: 0.0,
            pad4: 0.0,
        }
    }
}

impl Default for PointLight {
    fn default() -> Self {
        let constant = 1.0;
        let linear = 0.09;
        let quadratic = 0.032;

        PointLight {
            position: Vector3::new(0.0, 0.0, 0.0),
            ambient: Vector3::new(0.1, 0.1, 0.1),
            specular: Vector3::new(1.0, 1.0, 1.0),
            diffuse: Vector3::new(1.0, 1.0, 1.0),
            constant,
            linear,
            quadratic,
        }
    }
}
