use anyhow::Result;
use nalgebra::Vector2;
use once_cell::sync::OnceCell;
use smol_renderer::{
    FragmentShader, GpuData, LoadableTexture, RenderNode, SimpleTexture, Texture, TextureData,
    TextureShaderLayout, UniformBindGroup, VertexBuffer, VertexShader,
};
use wgpu::{
    BufferAddress, Device, ShaderStage, TextureFormat, VertexAttributeDescriptor, VertexFormat,
};

#[repr(C)]
#[derive(GpuData, Debug)]
pub struct UiVertex {
    position: Vector2<f32>,
    uv: Vector2<f32>,
    color: u32,
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

fn upload_font_textures(
    fonts: &imgui::FontAtlasRefMut,
    device: &Device,
) -> TextureData<UiTexture> {
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
        let mut fonts = ctx.fonts();
        let ui_texture = upload_font_textures(fonts, device);
        let mut textures = imgui::Textures::new();
        let atlas = textures.insert(ui_texture);
        fonts.tex_id = atlas;
        Ok(UiPass {
            render_node,
            textures,
        })
    }

    // nextup start with rendering
}
