use legion::prelude::{Resources, World};
use wgpu::{CommandBuffer, CommandEncoder, VertexBufferDescriptor};
use wgpu::{Device, SwapChainOutput};

pub mod light_object_pass;
pub mod model_pass;

pub trait VBDesc {
    fn desc<'a>() -> VertexBufferDescriptor<'a>;
}

pub trait Pass {
    //fn new(device: &mut Device) -> Box<Self>;
    fn update_uniforms(
        &mut self,
        world: &World,
        resources: &mut Resources,
        device: &mut Device,
    ) -> CommandBuffer;
    fn draw(
        &self,
        world: &World,
        resources: &mut Resources,
        frame: &SwapChainOutput,
        encoder: &mut CommandEncoder,
    );
}
