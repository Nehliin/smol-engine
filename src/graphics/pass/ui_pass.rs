use super::Pass;
use crate::assets::AssetManager;
use anyhow::Result;
use imgui::{
    internal::RawWrapper, BackendFlags, DrawCmd, DrawCmdParams, DrawIdx, DrawVert, TextureId,
};
use legion::prelude::World;
use nalgebra::Vector2;
use once_cell::sync::OnceCell;
use smol_renderer::{
    FragmentShader, GpuData, ImmutableVertexData, LoadableTexture, RenderNode, SimpleTexture,
    Texture, TextureData, TextureShaderLayout, UniformBindGroup, VertexBuffer, VertexBufferData,
    VertexShader,
};
use wgpu::{
    Buffer, BufferAddress, BufferUsage, CommandEncoder, Device, RenderPassDescriptor, ShaderStage,
    TextureFormat, VertexAttributeDescriptor, VertexFormat,
};

#[repr(C)]
#[derive(GpuData, Debug)]
pub struct UiVertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [u8; 4],
}

impl VertexBuffer for UiVertex {
    const STEP_MODE: wgpu::InputStepMode = wgpu::InputStepMode::Vertex;

    fn get_attributes<'a>() -> &'a [wgpu::VertexAttributeDescriptor] {
        &[
            VertexAttributeDescriptor {
                offset: 0,
                format: VertexFormat::Float2,
                shader_location: 0,
            },
            VertexAttributeDescriptor {
                offset: std::mem::size_of::<Vector2<f32>>() as BufferAddress,
                format: VertexFormat::Float2,
                shader_location: 1,
            },
            VertexAttributeDescriptor {
                offset: (std::mem::size_of::<Vector2<f32>>() * 2) as BufferAddress,
                format: VertexFormat::Uint,
                shader_location: 2,
            },
        ]
    }
}

impl From<DrawVert> for UiVertex {
    fn from(imgui_draw_vert: DrawVert) -> Self {
        // Should be safe because of repr C and exact same layout
        unsafe { std::mem::transmute(imgui_draw_vert) }
    }
}
#[repr(C)]
#[derive(GpuData)]
pub struct ViewMatrix {
    matrix: [[f32; 4]; 4],
}

pub struct UiTexture;

/*impl UiTexture {
    fn new()
}*/

impl TextureShaderLayout for UiTexture {
    const VISIBILITY: ShaderStage = ShaderStage::FRAGMENT;

    fn get_layout(device: &Device) -> &'static wgpu::BindGroupLayout {
        static LAYOUT: OnceCell<wgpu::BindGroupLayout> = OnceCell::new();
        LAYOUT.get_or_init(|| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry::new(
                        0,
                        Self::VISIBILITY,
                        wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Float,
                        },
                    ),
                    wgpu::BindGroupLayoutEntry::new(
                        1,
                        Self::VISIBILITY,
                        wgpu::BindingType::Sampler { comparison: false },
                    ),
                ],
                label: Some("Ui Texture layout"),
            })
        })
    }
}

impl Texture for UiTexture {
    fn allocate_texture(device: &Device) -> TextureData<Self>
    where
        Self: TextureShaderLayout,
    {
        todo!("The ugliest hack since ever")
    }
}

fn upload_font_textures(fonts: &imgui::FontAtlasRefMut, device: &Device) -> TextureData<UiTexture> {
    let texture_atlas = fonts.build_rgba32_texture();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Font atlas texture"),
        size: wgpu::Extent3d {
            width: texture_atlas.width,
            height: texture_atlas.height,
            depth: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2, // create cube texture for omnidirectional shadows
        format: TextureFormat::Bgra8UnormSrgb,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    });
    let view = texture.create_default_view();
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("font atlas sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        lod_min_clamp: -100.0,
        lod_max_clamp: 100.0,
        compare: None,
        ..Default::default()
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: UiTexture::get_layout(device),
        bindings: &[
            wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::Binding {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: Some("font atlas texture bindgroup"),
    });
    TextureData::new(bind_group, texture, vec![view], sampler)
}

pub struct UiPass {
    render_node: RenderNode,
    textures: imgui::Textures<TextureData<UiTexture>>,
    font_texture: TextureData<UiTexture>,
    vertex_buffer: Option<ImmutableVertexData<UiVertex>>,
    index_buffer: Option<Buffer>,
}

impl UiPass {
    pub fn new(
        ctx: &mut imgui::Context,
        device: &Device,
        color_format: TextureFormat,
    ) -> Result<Self> {
        let render_node = RenderNode::builder()
            .add_vertex_buffer::<UiVertex>()
            .set_vertex_shader(VertexShader::new(device, "src/shader_files/vs_ui.shader")?)
            .set_fragment_shader(FragmentShader::new(
                device,
                "src/shader_files/fs_ui.shader",
            )?)
            .add_local_uniform_bind_group(
                UniformBindGroup::builder()
                    .add_binding::<ViewMatrix>(ShaderStage::VERTEX)?
                    .build(device),
                // använd custom texxture
            )
            .add_texture::<UiTexture>()
            .add_default_color_state_desc(color_format)
            .set_rasterization_state(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            })
            .build(device)?;
        ctx.io_mut()
            .backend_flags
            .insert(BackendFlags::RENDERER_HAS_VTX_OFFSET);
        let mut fonts = ctx.fonts();
        let ui_texture = upload_font_textures(&fonts, device);
        let mut textures = imgui::Textures::new();
        //let atlas = textures.insert(ui_texture);
        //fonts.tex_id = atlas;
        Ok(UiPass {
            render_node,
            font_texture: ui_texture,
            textures,
            vertex_buffer: None,
            index_buffer: None,
        })
    }

    fn lookup_texture(&self, texture_id: TextureId) -> Result<&TextureData<UiTexture>, ()> {
        if texture_id.id() == usize::MAX {
            Ok(&self.font_texture)
        } else if let Some(texture) = self.textures.get(texture_id) {
            Ok(texture)
        } else {
            Err(())
        }
    }

    fn upload_vertex_buffer(&mut self, device: &wgpu::Device, vtx_buffer: &[DrawVert]) {
        // very inefficent
        let buffer_data = vtx_buffer
            .iter()
            .copied()
            .map(|vert| vert.into())
            .collect::<Vec<UiVertex>>();

        self.vertex_buffer = Some(VertexBuffer::allocate_immutable_buffer(
            device,
            &buffer_data,
        ));
    }

    fn upload_index_buffer(&mut self, device: &wgpu::Device, idx_buffer: &[DrawIdx]) {
        let data = unsafe {
            std::slice::from_raw_parts(idx_buffer.as_ptr() as *const u8, idx_buffer.len() * 2)
        };
        let index_buffer = device.create_buffer_with_data(data, BufferUsage::INDEX);
        self.index_buffer = Some(index_buffer);
    }

    fn render<'encoder>(
        &'encoder mut self,
        device: &Device,
        draw_data: &imgui::DrawData,
        encoder: &mut CommandEncoder,
        render_pass_descriptor: RenderPassDescriptor,
    ) {
        let runner = self.render_node.runner(encoder, render_pass_descriptor);
        let fb_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let fb_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];
        if !(fb_width > 0.0 && fb_height > 0.0) {
            panic!("Wierd UI frame size");
        }

        let left = draw_data.display_pos[0];
        let right = draw_data.display_pos[0] + draw_data.display_size[0];
        let top = draw_data.display_pos[1];
        let bottom = draw_data.display_pos[1] + draw_data.display_size[1];
        let matrix = [
            [(2.0 / (right - left)), 0.0, 0.0, 0.0],
            [0.0, (2.0 / (top - bottom)), 0.0, 0.0],
            [0.0, 0.0, -1.0, 0.0],
            [
                (right + left) / (left - right),
                (top + bottom) / (bottom - top),
                0.0,
                1.0,
            ],
        ];
        let clip_off = draw_data.display_pos;
        let clip_scale = draw_data.framebuffer_scale;

        // update pass data
        for draw_list in draw_data.draw_lists() {
            self.upload_vertex_buffer(device, draw_list.vtx_buffer());
            self.upload_index_buffer(device, draw_list.idx_buffer());
            for cmd in draw_list.commands() {
                match cmd {
                    DrawCmd::Elements {
                        count,
                        cmd_params:
                            DrawCmdParams {
                                clip_rect,
                                texture_id,
                                vtx_offset,
                                idx_offset,
                                ..
                            },
                    } => {
                        let clip_rect = [
                            (clip_rect[0] - clip_off[0]) * clip_scale[0],
                            (clip_rect[1] - clip_off[1]) * clip_scale[1],
                            (clip_rect[2] - clip_off[0]) * clip_scale[0],
                            (clip_rect[3] - clip_off[1]) * clip_scale[1],
                        ];

                        self.slice.start = idx_offset as u32;
                        self.slice.end = self.slice.start + count as u32;
                        self.slice.base_vertex = vtx_offset as u32;

                        if clip_rect[0] < fb_width
                            && clip_rect[1] < fb_height
                            && clip_rect[2] >= 0.0
                            && clip_rect[3] >= 0.0
                        {
                            /*let scissor = Rect {
                                x: f32::max(0.0, clip_rect[0]).floor() as u16,
                                y: f32::max(0.0, clip_rect[1]).floor() as u16,
                                w: (clip_rect[2] - clip_rect[0]).abs().ceil() as u16,
                                h: (clip_rect[3] - clip_rect[1]).abs().ceil() as u16,
                            };*/
                            let tex = self.lookup_texture(texture_id)?;
                            // #[cfg(feature = "directx")]
                            //{
                            //   let constants = constants::Constants { matrix };
                            // encoder.update_constant_buffer(&self.constants, &constants);
                            //}
                            self.render_node
                                .update(device, encoder, 1, &ViewMatrix { matrix })
                                .unwrap();

                            let pass = self.render_node.runner(encoder, render_pass_descriptor);
                            pass.set_vertex_buffer_data(0, self.vertex_buffer.unwrap());
                            pass.set_index_buffer(self.index_buffer.unwrap().slice(..));
                            pass.set_texture_data(0, tex);

                            pass.draw_indexed(idx_offset..idx_offset + count, vtx_offset);
                        }
                    }
                    DrawCmd::ResetRenderState => (), // TODO
                    DrawCmd::RawCallback { callback, raw_cmd } => unsafe {
                        callback(draw_list.raw(), raw_cmd)
                    },
                }
            }
        }
    }
    // nextup start with rendering
}
