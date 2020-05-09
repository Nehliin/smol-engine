use glfw::Window;
use legion::prelude::*;
use wgpu::{
    Adapter, BackendBit, Color, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor,
    Extensions, Extent3d, Limits, LoadOp, PowerPreference, PresentMode, Queue,
    RenderPassColorAttachmentDescriptor, RenderPassDepthStencilAttachmentDescriptor,
    RenderPassDescriptor, RequestAdapterOptions, ShaderStage, StoreOp, Surface, SwapChain,
    SwapChainDescriptor, Texture, TextureDimension, TextureFormat, TextureUsage, TextureView,
};

use super::{
    lighting::PointLight,
    lighting::{
        directional_light::DirectionalLightRaw, point_light::PointLightRaw, DirectionalLight,
    },
    pass::{shadow_pass::ShadowPass, skybox_pass::SkyboxPass},
    skybox_texture::SkyboxTexture,
    uniform_bind_groups::{DirectionalLightUniforms, LightSpaceMatrix, PointLightUniforms},
};
use crate::assets::AssetManager;
use crate::camera::Camera;
use crate::graphics::model::Model;
use crate::graphics::pass::light_object_pass::LightObjectPass;
use crate::graphics::pass::model_pass::ModelPass;
use crate::graphics::shadow_texture::ShadowTexture;
use crate::graphics::uniform_bind_groups::CameraDataRaw;
use crate::{
    components::Transform,
    graphics::{Pass, UniformBindGroup, UniformCameraData},
};
use nalgebra::Vector3;

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
    pub device: Device,
    queue: Queue,
    swap_chain_desc: SwapChainDescriptor,
    swap_chain: SwapChain,
    width: u32,
    height: u32,
    camera_uniforms: UniformBindGroup<CameraDataRaw>,
    point_light_uniforms: UniformBindGroup<PointLightUniforms>,
    directional_light_uniforms: UniformBindGroup<DirectionalLightUniforms>,
    depth_texture: Texture,
    depth_texture_view: TextureView,
    model_pass: ModelPass,
    light_pass: LightObjectPass,
    skybox_pass: SkyboxPass,
    shadow_pass: ShadowPass,
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

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                extensions: Extensions {
                    anisotropic_filtering: false,
                },
                limits: Limits { max_bind_groups: 6 },
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
        let point_light_uniforms = UniformBindGroup::new(&device, ShaderStage::FRAGMENT);
        let directional_light_uniforms = UniformBindGroup::new(&device, ShaderStage::FRAGMENT);
        let depth_texture = create_depth_texture(&device, &swap_chain_desc);
        let depth_texture_view = depth_texture.create_default_view();

        let shadow_pass =
            ShadowPass::new(&device, vec![Model::get_or_create_texture_layout(&device)]).unwrap();

        let model_pass = ModelPass::new(
            &device,
            vec![
                Model::get_or_create_texture_layout(&device),
                &camera_uniforms.bind_group_layout,
                &point_light_uniforms.bind_group_layout,
                ShadowTexture::get_or_create_texture_layout(&device),
                &directional_light_uniforms.bind_group_layout,
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

        // TODO: should be handled as an asset instead
        let (skybox_texture, command_buffer) = SkyboxTexture::load(&device, "skybox").unwrap();
        queue.submit(&[command_buffer]);

        let skybox_pass = SkyboxPass::new(
            &device,
            vec![
                SkyboxTexture::get_bind_group_layout(&device),
                &camera_uniforms.bind_group_layout,
            ],
            swap_chain_desc.format,
            skybox_texture,
        );

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
            camera_uniforms,
            point_light_uniforms,
            shadow_pass,
            directional_light_uniforms,
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
            &self.device,
            &UniformCameraData {
                view_matrix: *camera.get_view_matrix(),
                projection: *camera.get_projection_matrix(),
                view_pos: camera.get_vec_position(),
            }
            .into(),
            encoder,
        )
    }

    pub fn update_lights(&self, world: &World, encoder: &mut CommandEncoder) {
        let query = <(Read<PointLight>, Read<Transform>)>::query();
        for chunk in query.par_iter_chunks(world) {
            let lights = chunk.components::<PointLight>().unwrap();
            let positions = chunk.components::<Transform>().unwrap();
            let mut uniform_data =
                [PointLightRaw::from((&PointLight::default(), Vector3::new(0.0, 0.0, 0.0))); 16];
            let mut lights_used = 0;
            lights
                .iter()
                .zip(positions.iter())
                .enumerate()
                .for_each(|(i, (light, pos))| {
                    uniform_data[i] = PointLightRaw::from((light, pos.translation()));
                    lights_used += 1;
                });
            self.point_light_uniforms.update(
                &self.device,
                &PointLightUniforms {
                    lights_used,
                    pad: [0; 3],
                    point_lights: uniform_data,
                },
                encoder,
            );
        }

        let query = <Read<DirectionalLight>>::query();
        let directional_light = query.iter(world).next().unwrap();
        self.directional_light_uniforms.update(
            &self.device,
            &DirectionalLightUniforms {
                directional_light: DirectionalLightRaw::from(&*dbg!(directional_light)),
            },
            encoder,
        );
    }

    // THIS SHOULD NOT REQUIRE MUTABLE REF TO RESOURCES!
    pub fn render_frame(&mut self, world: &mut World, resources: &mut Resources) {
        let frame = self.swap_chain.get_next_texture().unwrap();
        let camera = resources.get::<Camera>().unwrap();
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Model render pass"),
            });
        self.update_camera_uniforms(&camera, &mut encoder);
        self.update_lights(world, &mut encoder);
        let mut asset_storage = resources.get_mut::<AssetManager>().unwrap();
        // TODO: This should be in an update method instead
        let mut commands = asset_storage.clear_load_queue(&self.device);
        self.model_pass
            .update_uniform_data(&world, &asset_storage, &self.device, &mut encoder);
        // move somewhere else this isn't as nice
        self.shadow_pass
            .shadow_texture
            .update_lights_with_texture_view(world);
        let query = <(Read<PointLight>, Read<Transform>)>::query();
        for (light, transform) in query.iter(world) {
            let raw_light = PointLightRaw::from((&*light, transform.translation()));
            self.shadow_pass
                .update_uniforms(&self.device, &raw_light, &mut encoder);
            self.shadow_pass.render(
                // this shit doesn't work for some reaso
                &mut encoder,
                &[],
                &*light,
                world,
                &asset_storage,
            );
        }
        let directional_light_query = <Read<DirectionalLight>>::query();
        for light in directional_light_query.iter(world) {
            let raw_light = DirectionalLightRaw::from(&*light);
            self.shadow_pass
                .update_uniforms(&self.device, &raw_light, &mut encoder);
            self.shadow_pass.render(
                // this shit doesn't work for some reaso
                &mut encoder,
                &[],
                &*light,
                world,
                &asset_storage,
            );
        }
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: None,
            });
            self.skybox_pass.render(
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
                    depth_load_op: LoadOp::Clear,
                    depth_store_op: StoreOp::Store,
                    clear_depth: 1.0,
                    stencil_load_op: LoadOp::Clear,
                    stencil_store_op: StoreOp::Store,
                    clear_stencil: 0,
                }),
            });
            self.model_pass.render(
                &[
                    &self.camera_uniforms.bind_group,
                    &self.point_light_uniforms.bind_group,
                    &self.shadow_pass.shadow_texture.bind_group,
                    &self.directional_light_uniforms.bind_group,
                ],
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
