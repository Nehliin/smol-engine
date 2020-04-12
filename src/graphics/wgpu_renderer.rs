use glfw::{Glfw, Window};
use legion::prelude::{Resources, World};
use nalgebra::Matrix4;
use wgpu::{
    Adapter, BackendBit, BindGroup, BindGroupDescriptor, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, Binding, BindingResource, BindingType, Buffer,
    BufferAddress, BufferDescriptor, BufferUsage, Color, CommandBuffer, CommandEncoderDescriptor,
    Device, DeviceDescriptor, Extensions, Extent3d, LoadOp, PowerPreference, PresentMode, Queue,
    RenderPassColorAttachmentDescriptor, RenderPassDepthStencilAttachmentDescriptor,
    RenderPassDescriptor, RequestAdapterOptions, ShaderStage, StoreOp, Surface, SwapChain,
    SwapChainDescriptor, Texture, TextureComponentType, TextureDimension, TextureFormat,
    TextureUsage, TextureView, TextureViewDimension,
};

use crate::camera::Camera;
use crate::components::{AssetManager, ModelHandle};
use crate::graphics::model::Model;
use crate::graphics::pass::model_pass::ModelPass;
use crate::graphics::{Pass, Renderer};
use std::ops::Deref;

//type RenderPass = Box<dyn Pass>;

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
fn create_depth_texture(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
) -> wgpu::Texture {
    let desc = wgpu::TextureDescriptor {
        label: None,
        size: Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        },
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    };
    device.create_texture(&desc)
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UniformCameraData {
    pub view_matrix: Matrix4<f32>,
    pub projection: Matrix4<f32>,
}

pub struct UniformBindGroup {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}

impl UniformBindGroup {
    pub fn new(device: &mut Device) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Uniform buffer"),
            size: std::mem::size_of::<UniformCameraData>() as u64,
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                // This is the layout of the uniform buffer
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
            ],
            label: Some("Uniform Bind group layout"),
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[Binding {
                binding: 0,
                resource: BindingResource::Buffer {
                    buffer: &buffer,
                    range: 0..std::mem::size_of::<UniformCameraData>() as BufferAddress,
                },
            }],
            label: Some("Uniform bind group"),
        });
        Self {
            bind_group,
            buffer,
            bind_group_layout,
        }
    }

    pub fn update(&self, device: &mut Device, data: UniformCameraData) -> CommandBuffer {
        let data = unsafe {
            std::slice::from_raw_parts(
                &data as *const UniformCameraData as *const u8,
                std::mem::size_of::<UniformCameraData>(),
            )
        };
        let staging_buffer = device.create_buffer_with_data(data, BufferUsage::COPY_SRC);

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("UniformCameraData staging buffer"),
        });

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.buffer,
            0,
            std::mem::size_of::<UniformCameraData>() as BufferAddress,
        );
        encoder.finish()
    }
}

pub struct WgpuRenderer {
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    swap_chain_desc: SwapChainDescriptor,
    swap_chain: SwapChain,
    width: u32,
    height: u32,
    uniform_bind_group: UniformBindGroup,
    depth_texture: Texture,
    depth_texture_view: TextureView,
    model_pass: ModelPass,
    // Storing all the render passes
    // render_passes: Vec<RenderPass>,
}

impl WgpuRenderer {
    pub async fn new(window: &Window, resources: &mut Resources) -> Self {
        let (width, height) = window.get_size();

        let surface = Surface::create(window);
        let adapter = Adapter::request(
            &RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            },
            BackendBit::PRIMARY,
        )
        .await
        .expect("Couln't create wgpu adapter");

        let (mut device, mut queue) = adapter
            .request_device(&DeviceDescriptor {
                extensions: Extensions {
                    anisotropic_filtering: false,
                },
                limits: Default::default(),
            })
            .await;

        let swap_chain_desc = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: width as u32,
            height: height as u32,
            present_mode: PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let uniform_bind_group = UniformBindGroup::new(&mut device);

        let depth_texture = create_depth_texture(&device, &swap_chain_desc);
        let depth_texture_view = depth_texture.create_default_view();

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                // diffuse texture
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Float,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: true },
                },
                // specular texutre
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Float,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: true },
                },
            ],
            label: Some("Texture tmp layout"),
        });

        let model_pass = ModelPass::new(
            &mut device,
            &layout,
            &uniform_bind_group.bind_group_layout,
            swap_chain_desc.format,
        );
        let (model, cmd_buffer) =
            Model::load("nanosuit/nanosuit.obj", &mut device, &layout).unwrap();
        let (cube_model, cmd_buffer_1) = Model::load("box/cube.obj", &mut device, &layout).unwrap();
        queue.submit(&cmd_buffer);
        queue.submit(&cmd_buffer_1);
        let handle = ModelHandle { id: 0 };
        let handle_2 = ModelHandle { id: 1 };
        let mut asset_manager = AssetManager::new();
        asset_manager.asset_map.insert(handle, model);
        asset_manager.asset_map.insert(handle_2, cube_model);
        resources.insert(asset_manager);

        WgpuRenderer {
            surface,
            adapter,
            device,
            queue,
            swap_chain_desc,
            swap_chain,
            width: width as u32,
            height: height as u32,
            depth_texture,
            depth_texture_view,
            model_pass,
            uniform_bind_group,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.height = height;
        self.width = width;

        self.swap_chain_desc.width = width;
        self.swap_chain_desc.height = height;

        self.depth_texture = create_depth_texture(&self.device, &self.swap_chain_desc);
        self.depth_texture_view = self.depth_texture.create_default_view();

        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_desc);
    }

    pub fn render_frame(&mut self, world: &mut World, resources: &mut Resources) {
        let frame = self.swap_chain.get_next_texture().unwrap();
        let mut command_buffers = Vec::new();
        let camera = resources.get::<Camera>().unwrap();
        let commands = self.uniform_bind_group.update(
            &mut self.device,
            UniformCameraData {
                view_matrix: *camera.get_view_matrix(),
                projection: *camera.get_projection_matrix(),
            },
        );
        command_buffers.push(commands);
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Model render pass"),
            });
        let asset_manager = resources.get::<AssetManager>().unwrap();
        ModelPass::update_instances(resources, world, &mut encoder, &mut self.device);
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture_view,
                    depth_load_op: LoadOp::Clear,
                    depth_store_op: StoreOp::Store,
                    clear_depth: 1.0,
                    stencil_load_op: LoadOp::Clear,
                    stencil_store_op: StoreOp::Store,
                    clear_stencil: 0,
                }),
            });

            self.model_pass.render(
                &mut self.uniform_bind_group,
                &asset_manager,
                world,
                &mut render_pass,
            );
        }
        command_buffers.push(encoder.finish());
        self.queue.submit(&command_buffers);
    }
}
