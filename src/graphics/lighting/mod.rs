pub mod directional_light;
pub mod point_light;

pub use directional_light::DirectionalLight;
pub use point_light::PointLight;

pub trait Light {
    fn get_target_view(&self) -> &Option<wgpu::TextureView>;
}
