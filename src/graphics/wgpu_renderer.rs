use crate::graphics::{Pass, Renderer};
use glfw::{Glfw, Window};
use legion::prelude::{Resources, World};
use wgpu::{
    Adapter, CommandEncoderDescriptor, Device, DeviceDescriptor, Extensions, PresentMode, Queue,
    RequestAdapterOptions, Surface, SwapChain, SwapChainDescriptor, TextureFormat, TextureUsage,
};

type RenderPass = Box<dyn Pass>;

pub struct WgpuRenderer {
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    swap_chain_desc: SwapChainDescriptor,
    swap_chain: SwapChain,
    width: u32,
    height: u32,

    // Storing all the render passes
    render_passes: Vec<RenderPass>,
}

impl Renderer for WgpuRenderer {
    fn set_window_hints(glfw: &mut Glfw) {
        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
    }

    fn new(window: &Window, resource: &mut Resources) -> Self {
        let (width, height) = window.get_size();

        let surface = Surface::create(window);
        let adapter = Adapter::request(&RequestAdapterOptions {
            ..Default::default()
        })
        .expect("Couln't create wgpu adapter");

        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            extensions: Extensions {
                anisotropic_filtering: false,
            },
            limits: Default::default(),
        });

        let swap_chain_desc = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: width as u32,
            height: height as u32,
            present_mode: PresentMode::Vsync,
        };

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let render_passes = Vec::new();

        WgpuRenderer {
            surface,
            adapter,
            device,
            queue,
            swap_chain_desc,
            swap_chain,
            width: width as u32,
            height: height as u32,
            render_passes,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.height = height;
        self.width = width;

        self.swap_chain_desc.width = width;
        self.swap_chain_desc.height = height;

        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_desc);
    }

    fn render_frame(&mut self, world: &mut World, _resources: &mut Resources) {
        let frame = self.swap_chain.get_next_texture();

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { todo: 0 });

        for render_pass in self.render_passes {
            render_pass.update_uniforms(world, &mut encoder);
            render_pass.draw(world, &mut encoder);
        }
        self.queue.submit(&[encoder.finish()]);
    }
}
