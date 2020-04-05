use legion::prelude::{Resources, World};
use wgpu::{CommandEncoder, CreateBufferMapped};
use wgpu::{Device, SwapChainOutput, Texture};

pub mod model_pass;

pub trait Pass {
    //fn new(device: &mut Device) -> Box<Self>;
    fn update_uniforms(
        &mut self,
        world: &World,
        resources: &mut Resources,
        encoder: &mut CommandEncoder,
    );
    fn draw(&self, world: &World, frame: &SwapChainOutput, encoder: &mut CommandEncoder);
}
