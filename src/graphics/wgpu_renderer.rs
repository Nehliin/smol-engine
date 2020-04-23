use glfw::Window;
use legion::prelude::{Resources, World};
use wgpu::{
    Adapter, BackendBit, BindingResource, BindingType, Buffer, BufferAddress, BufferDescriptor,
    BufferUsage, Color, CommandBuffer, CommandEncoder, CommandEncoderDescriptor, Device,
    DeviceDescriptor, Extensions, Extent3d, LoadOp, PowerPreference, PresentMode, Queue,
    RenderPassColorAttachmentDescriptor, RenderPassDepthStencilAttachmentDescriptor,
    RenderPassDescriptor, RequestAdapterOptions, ShaderStage, StoreOp, Surface, SwapChain,
    SwapChainDescriptor, Texture, TextureComponentType, TextureDimension, TextureFormat,
    TextureUsage, TextureView, TextureViewDimension,
};

use crate::assets::{AssetManager, ModelHandle};
use crate::camera::Camera;
use crate::graphics::model::Model;
use crate::graphics::pass::light_object_pass::LightObjectPass;
use crate::graphics::pass::model_pass::ModelPass;
use crate::graphics::uniform_bind_groups::CameraDataRaw;
use crate::graphics::{Pass, UniformBindGroup, UniformCameraData};
use crossbeam_channel::{Receiver, Sender};

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

pub struct WgpuRenderer {
    surface: Surface,
    adapter: Adapter,
    pub device: Device,
    queue: Queue,
    swap_chain_desc: SwapChainDescriptor,
    swap_chain: SwapChain,
    width: u32,
    height: u32,
    camera_uniforms: UniformBindGroup<CameraDataRaw>,
    depth_texture: Texture,
    depth_texture_view: TextureView,
    model_pass: ModelPass,
    light_pass: LightObjectPass,
}

impl WgpuRenderer {
    pub async fn new(window: &Window) -> Self {
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
        .expect("Couldn't create wgpu adapter");

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

        let camera_uniforms = UniformBindGroup::new(&device, ShaderStage::VERTEX);

        let depth_texture = create_depth_texture(&device, &swap_chain_desc);
        let depth_texture_view = depth_texture.create_default_view();

        let model_pass = ModelPass::new(
            &device,
            vec![
                Model::get_or_create_texture_layout(&device),
                &camera_uniforms.bind_group_layout,
            ],
            swap_chain_desc.format,
        )
        .unwrap();
        let light_pass = LightObjectPass::new(
            &device,
            &Model::get_or_create_texture_layout(&device),
            &camera_uniforms.bind_group_layout,
            swap_chain_desc.format,
        );

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
            light_pass,
            camera_uniforms,
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

    fn update_camera_uniforms(&mut self, camera: &Camera, encoder: &mut CommandEncoder) {
        self.camera_uniforms.update(
            &mut self.device,
            &UniformCameraData {
                view_matrix: *camera.get_view_matrix(),
                projection: *camera.get_projection_matrix(),
                view_pos: camera.get_vec_position(),
            }
            .into(),
            encoder,
        )
    }

    // THIS SHOULD NOT REQUIRE MUTABLE REF TO RESOURCES!
    pub fn render_frame(&mut self, world: &World, resources: &mut Resources) {
        let frame = self.swap_chain.get_next_texture().unwrap();
        let camera = resources.get::<Camera>().unwrap();
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Model render pass"),
            });
        self.update_camera_uniforms(&camera, &mut encoder);
        let mut asset_storage = resources.get_mut::<AssetManager>().unwrap();
        // TODO: This should be in an update method instead
        let mut commands = asset_storage.clear_load_queue(&self.device);
        self.model_pass
            .update_uniform_data(&world, &asset_storage, &self.device, &mut encoder);
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
                &[&self.camera_uniforms.bind_group],
                &asset_storage,
                world,
                &mut render_pass,
            );
        }
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: LoadOp::Load,
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
                    depth_load_op: LoadOp::Load,
                    depth_store_op: StoreOp::Store,
                    clear_depth: 1.0,
                    stencil_load_op: LoadOp::Load,
                    stencil_store_op: StoreOp::Store,
                    clear_stencil: 0,
                }),
            });
            self.light_pass.render(
                &[&self.camera_uniforms.bind_group],
                &asset_storage,
                world,
                &mut render_pass,
            );
        }
        commands.push(encoder.finish());
        self.queue.submit(&commands);
    }
}
