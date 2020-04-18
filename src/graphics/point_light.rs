use nalgebra::Vector3;
use zerocopy::AsBytes;

#[derive(Copy, Clone, Debug)]
pub struct PointLight {
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
}

impl From<(PointLight, Vector3<f32>)> for PointLightRaw {
    fn from((light, position): (PointLight, Vector3<f32>)) -> Self {
        PointLightRaw {
            position: [position.x, position.y, position.z],
            ambient: [light.ambient.x, light.ambient.y, light.ambient.z],
            specular: [light.specular.x, light.specular.y, light.specular.z],
            diffuse: [light.diffuse.x, light.diffuse.y, light.diffuse.z],
            constant: light.constant,
            linear: light.linear,
            quadratic: light.quadratic,
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
        }
    }
}
