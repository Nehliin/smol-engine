use glfw::Window;
use legion::prelude::*;
use wgpu::{
    BackendBit, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d,
    Instance, Limits, LoadOp, Operations, PowerPreference, PresentMode, Queue,
    RenderPassColorAttachmentDescriptor, RenderPassDepthStencilAttachmentDescriptor,
    RenderPassDescriptor, RequestAdapterOptions, ShaderStage, Surface, SwapChain,
    SwapChainDescriptor, TextureDimension, TextureFormat, TextureUsage, TextureView,
};

use super::{
    pass::{
        shadow_pass::ShadowPass, skybox_pass::SkyboxPass,
        water_environment_pass::WaterEnvironmentPass, water_surface_pass::WaterSurfacePass,
    },
    point_light::PointLightRaw,
    skybox_texture::SkyboxTexture,
    water_map::WaterEnviornmentMap,
    PointLight,
model::Model};
use crate::graphics::pass::light_object_pass::LightObjectPass;
use crate::graphics::pass::model_pass::ModelPass;
use crate::graphics::shadow_texture::ShadowTexture;
use crate::{
    assets::Assets,
    camera::{Camera, CameraUniform},
};
use crate::{components::Transform, graphics::Pass};
use smol_renderer::{LoadableTexture, Texture, UniformBindGroup};
use std::rc::Rc;
use std::sync::Arc;

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
    pub device: Device,
    queue: Queue,
    swap_chain_desc: SwapChainDescriptor,
    swap_chain: SwapChain,
    width: u32,
    height: u32,
    global_camera_uniforms: Arc<UniformBindGroup>,
    depth_texture: wgpu::Texture,
    depth_texture_view: TextureView,
    model_pass: ModelPass,
    light_pass: LightObjectPass,
    skybox_pass: SkyboxPass,
    shadow_pass: ShadowPass,
    water_pass: WaterEnvironmentPass,
    water_surface_pass: WaterSurfacePass,
}

impl WgpuRenderer {
    pub async fn new(window: &Window) -> Self {
        let (width, height) = window.get_size();

        let instance = Instance::new(BackendBit::PRIMARY);

        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to request adapter");

        let features = adapter.features();
        let mut limits = Limits::default();
        limits.max_bind_groups = 6;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features,
                    shader_validation: true,
                    limits,
                },
                None,
            )
            .await
            .expect("Your Gpu doesn't support this program :(");

        let swap_chain_desc = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: width as u32,
            height: height as u32,
            present_mode: PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let global_camera_uniforms = Arc::new(
            UniformBindGroup::builder()
                .add_binding::<CameraUniform>(ShaderStage::VERTEX)
                .unwrap()
                // TODO: is this really a global uniform?? can you use Rc and add to multiple passes?
                //.add_binding::<PointLightUniform>(ShaderStage::FRAGMENT)
                //.unwrap()
                .build(&device),
        );

        let depth_texture = create_depth_texture(&device, &swap_chain_desc);
        let depth_texture_view = depth_texture.create_default_view();

        let shadow_texture = Rc::new(ShadowTexture::allocate_texture(&device));
        let water_map = Rc::new(WaterEnviornmentMap::allocate_texture(&device));

        let water_pass = WaterEnvironmentPass::new(&device, water_map.clone()).unwrap();
        let shadow_pass = ShadowPass::new(&device, shadow_texture.clone()).unwrap();

        let water_surface_pass = WaterSurfacePass::new(&device).unwrap();

        let model_pass = ModelPass::new(
            &device,
            vec![Arc::clone(&global_camera_uniforms)],
            shadow_texture,
            water_map,
            swap_chain_desc.format,
        )
        .unwrap();
        let light_pass = LightObjectPass::new(
            &device,
            vec![Arc::clone(&global_camera_uniforms)],
            swap_chain_desc.format,
        )
        .unwrap();
        // TODO: should be handled as an asset instead
        let skybox_texture = SkyboxTexture::load_texture(&device, &queue, "skybox").unwrap();
        let skybox_pass = SkyboxPass::new(
            &device,
            vec![Arc::clone(&global_camera_uniforms)],
            swap_chain_desc.format,
            skybox_texture,
        )
        .unwrap();

        WgpuRenderer {
            surface,
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
            skybox_pass,
            global_camera_uniforms,
            shadow_pass,
            water_pass,
            water_surface_pass,
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
        self.global_camera_uniforms
            .update_buffer_data(
                &self.device,
                encoder,
                &CameraUniform {
                    view_matrix: *camera.get_view_matrix(),
                    projection: *camera.get_projection_matrix(),
                    view_pos: camera.get_vec_position(),
                },
            )
            .unwrap();
    }

    // THIS SHOULD NOT REQUIRE MUTABLE REF TO RESOURCES!
    pub fn render_frame(&mut self, world: &mut World, resources: &mut Resources) {
        let frame = self.swap_chain.get_next_frame().unwrap().output;
        let camera = resources.get::<Camera>().unwrap();
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Main CommandEncoder"),
            });
        self.update_camera_uniforms(&camera, &mut encoder);
        let mut asset_storage = resources.get_mut::<Assets<Model>>().unwrap();
        // TODO: This should be in an update method instead
        asset_storage
            .clear_load_queue(&self.device, &self.queue)
            .unwrap();
        drop(asset_storage);
        self.model_pass
            .update_uniform_data(&world, &resources, &self.device, &mut encoder);

        // move somewhere else this isn't as nice
        self.shadow_pass.update_lights_with_texture_view(world);
        let query = <(Read<PointLight>, Read<Transform>)>::query();
        for (light, transform) in query.iter(world) {
            let raw_light = PointLightRaw::from((&*light, transform.translation()));
            self.water_pass
                .update_uniforms(&self.device, &raw_light, &mut encoder);
            self.water_pass.render(
                &resources,
                world,
                &mut encoder,
                RenderPassDescriptor {
                    color_attachments: &[RenderPassColorAttachmentDescriptor {
                        attachment: &self.water_pass.water_map_view,
                        resolve_target: None,
                        ops: Operations {
                            // TODO: Are these sane defaults?
                            load: LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            }),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                },
            );

            self.shadow_pass
                .update_uniforms(&self.device, &raw_light, &mut encoder);
            self.shadow_pass.render(
                &resources,
                world,
                &mut encoder,
                RenderPassDescriptor {
                    color_attachments: &[],
                    depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                        attachment: light.target_view.as_ref().unwrap(),
                        depth_ops: Some(Operations {
                            load: LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                },
            );
        }

        self.water_pass.render(
            &resources,
            world,
            &mut encoder,
            RenderPassDescriptor {
                color_attachments: &[],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.water_pass.water_map_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            },
        );

        self.skybox_pass.render(
            &resources,
            world,
            &mut encoder,
            RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            },
        );

        self.water_surface_pass.update_uniform_data(
            world,
            &resources,
            &self.device,
            &mut encoder,
        );
        self.water_surface_pass.render(
            &resources,
            world,
            &mut encoder,
            RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            },
        );

        self.model_pass.render(
            &resources,
            world,
            &mut encoder,
            RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            },
        );
        self.light_pass.render(
            &resources,
            world,
            &mut encoder,
            RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            },
        );
        self.queue.submit(vec![encoder.finish()]);
    }
}
