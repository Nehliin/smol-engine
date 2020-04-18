use glfw::Window;
use legion::prelude::{Resources, World};
use wgpu::{
    Adapter, BackendBit, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Binding,
    BindingResource, BindingType, Buffer, BufferAddress, BufferDescriptor, BufferUsage, Color,
    CommandBuffer, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor, Extensions,
    Extent3d, LoadOp, PowerPreference, PresentMode, Queue, RenderPassColorAttachmentDescriptor,
    RenderPassDepthStencilAttachmentDescriptor, RenderPassDescriptor, RequestAdapterOptions,
    ShaderStage, StoreOp, Surface, SwapChain, SwapChainDescriptor, Texture, TextureComponentType,
    TextureDimension, TextureFormat, TextureUsage, TextureView, TextureViewDimension,
};

use crate::camera::Camera;
use crate::components::{AssetManager, ModelHandle};
use crate::graphics::model::Model;
use crate::graphics::pass::light_object_pass::LightObjectPass;
use crate::graphics::pass::model_pass::ModelPass;
use crate::graphics::uniform_bind_groups::CameraDataRaw;
use crate::graphics::{Pass, UniformBindGroup, UniformCameraData};

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
    device: Device,
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

        let camera_uniforms = UniformBindGroup::new(&mut device, ShaderStage::VERTEX);

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
            &device,
            vec![&layout, &camera_uniforms.bind_group_layout],
            swap_chain_desc.format,
        )
        .unwrap();
        let light_pass = LightObjectPass::new(
            &device,
            &layout,
            &camera_uniforms.bind_group_layout,
            swap_chain_desc.format,
        );

        // TODO: this must be moved
        let (sphere_model, cmd_buffer_0) =
            Model::load("light/untitled.obj", &device, &layout).unwrap();
        let (model, cmd_buffer) = Model::load("nanosuit/nanosuit.obj", &device, &layout).unwrap();
        let (cube_model, cmd_buffer_1) = Model::load("box/cube.obj", &device, &layout).unwrap();
        queue.submit(&cmd_buffer);
        queue.submit(&cmd_buffer_1);
        queue.submit(&cmd_buffer_0);
        let handle = ModelHandle { id: 0 };
        let handle_2 = ModelHandle { id: 1 };
        let handle_3 = ModelHandle { id: 2 };
        let mut asset_manager = AssetManager::new();
        asset_manager.asset_map.insert(handle, model);
        asset_manager.asset_map.insert(handle_3, sphere_model);
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

    pub fn render_frame(&mut self, world: &mut World, resources: &mut Resources) {
        let frame = self.swap_chain.get_next_texture().unwrap();
        let camera = resources.get::<Camera>().unwrap();
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Model render pass"),
            });
        self.update_camera_uniforms(&camera, &mut encoder);
        let asset_manager = resources.get::<AssetManager>().unwrap();
        self.model_pass
            .update_uniform_data(&world, &asset_manager, &self.device, &mut encoder);
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
                &asset_manager,
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
            //render_pass.set_bind_group(1, &self.camera_uniforms.bind_group, &[]);
            self.light_pass.render(
                &[&self.camera_uniforms.bind_group],
                &asset_manager,
                world,
                &mut render_pass,
            );
        }
        self.queue.submit(&[encoder.finish()]);
    }
}
