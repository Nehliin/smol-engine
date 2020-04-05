use crate::graphics::texture::Texture;
use glfw::Window;
use nalgebra::{Matrix4, Point3, RowVector3, RowVector4, Vector3};

use crate::camera::Camera;
use crate::engine::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::shaders::Shader;
use wgpu::{
    read_spirv, Adapter, BindGroup, BindGroupDescriptor, BindGroupLayoutBinding,
    BindGroupLayoutDescriptor, Binding, BindingResource, BindingType, BlendDescriptor, Buffer,
    BufferAddress, BufferUsage, Color, ColorStateDescriptor, ColorWrite, CommandEncoderDescriptor,
    CullMode, Device, DeviceDescriptor, Extensions, Extent3d, FrontFace, IndexFormat,
    InputStepMode, LoadOp, PipelineLayoutDescriptor, PresentMode, PrimitiveTopology,
    ProgrammableStageDescriptor, Queue, RasterizationStateDescriptor,
    RenderPassColorAttachmentDescriptor, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderStage, StoreOp, Surface, SwapChain,
    SwapChainDescriptor, Texture as WgpuTexture, TextureDescriptor, TextureFormat, TextureUsage,
    TextureViewDimension, VertexAttributeDescriptor, VertexBufferDescriptor, VertexFormat,
};
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, -0.49240386, 0.0],
        tex_coords: [1.0 - 0.4131759, 1.0 - 0.00759614],
    }, // A
    Vertex {
        position: [-0.49513406, -0.06958647, 0.0],
        tex_coords: [1.0 - 0.0048659444, 1.0 - 0.43041354],
    }, // B
    Vertex {
        position: [-0.21918549, 0.44939706, 0.0],
        tex_coords: [1.0 - 0.28081453, 1.0 - 0.949397057],
    }, // C
    Vertex {
        position: [0.35966998, 0.3473291, 0.0],
        tex_coords: [1.0 - 0.85967, 1.0 - 0.84732911],
    }, // D
    Vertex {
        position: [0.44147372, -0.2347359, 0.0],
        tex_coords: [1.0 - 0.9414737, 1.0 - 0.2652641],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Uniforms {
    view_projection: Matrix4<f32>,
}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_projection: Matrix4::identity(),
        }
    }

    fn get_correction_matrix() -> Matrix4<f32> {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        Matrix4::from_rows(&[
            RowVector4::new(1.0, 0.0, 0.0, 0.0),
            RowVector4::new(0.0, -1.0, 0.0, 0.0),
            RowVector4::new(0.0, 0.0, 0.5, 0.0),
            RowVector4::new(0.0, 0.0, 0.5, 1.0),
        ]
        )
    }

    fn update_view_projection(&mut self, camera: &Camera) {
        self.view_projection = //Self::get_correction_matrix()
             camera.get_projection_matrix()
            * camera.get_view_matrix();
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    // this is really nice structure
    fn desc<'a>() -> VertexBufferDescriptor<'a> {
        use std::mem;
        /*
        step_mode tells the pipeline how often it should move to the next vertex.
        This seems redundant in our case, but we can specify wgpu::InputStepMode::Instance
         if we only want the change vertices when we start drawing a new instance.
        */
        VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as BufferAddress,
            step_mode: InputStepMode::Vertex,
            attributes: &[
                VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float3,
                },
                VertexAttributeDescriptor {
                    offset: mem::size_of::<Vector3<f32>>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float2,
                },
            ],
        }
    }
}

pub struct WgpuRenderer {
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    swap_chain_desc: SwapChainDescriptor,
    swap_chain: SwapChain,
    render_pipeline: RenderPipeline,
    width: u32,
    height: u32,
    uniforms: Uniforms,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
    diffuse_texture: Texture,
    diffuse_bind_group: BindGroup,
    index_buffer: Buffer,
    vertex_buffer: Buffer,
}

impl WgpuRenderer {
    pub fn new(window: &Window) -> Self {
        let (width, height) = window.get_size();

        let surface = Surface::create(window);
        let adapter = Adapter::request(&RequestAdapterOptions {
            ..Default::default()
        })
        .expect("Couldn't create wgpu adapter");

        let (device, mut queue) = adapter.request_device(&DeviceDescriptor {
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

        // swap chain handles the swapping of buffers unlike what glfw previously did when using opengl
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let diffuse_bytes = include_bytes!("../../happy-tree.png");
        let (diffuse_texture, command_buffer) =
            Texture::from_bytes(&device, diffuse_bytes).unwrap();
        // This submits the command that copys the texture data to the gpu
        queue.submit(&[command_buffer]);
        // the bindgrouplayout defines the premissions ish?
        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                bindings: &[
                    BindGroupLayoutBinding {
                        binding: 0,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::SampledTexture {
                            multisampled: false,
                            dimension: TextureViewDimension::D2,
                        },
                    },
                    BindGroupLayoutBinding {
                        binding: 1,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::Sampler,
                    },
                ],
            });
        // this defines the layout in memory in memory so the shader know how to read
        // the texture data
        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture.view),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });
        let camera = Camera::new(
            Point3::new(0., 0., 3.),
            Vector3::new(0.0, 0.0, -1.0),
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        );
        let mut uniforms = Uniforms::new();
        uniforms.update_view_projection(&camera);
        // this is the data
        let uniform_buffer = device
            .create_buffer_mapped(1, BufferUsage::UNIFORM | BufferUsage::COPY_DST)
            .fill_from_slice(&[uniforms]);
        // this defines the data layout
        let uniforms_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                bindings: &[BindGroupLayoutBinding {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::UniformBuffer { dynamic: false },
                }],
            });
        // put them together
        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &uniforms_bind_group_layout,
            bindings: &[Binding {
                binding: 0,
                resource: BindingResource::Buffer {
                    buffer: &uniform_buffer,
                    range: 0..std::mem::size_of_val(&uniforms) as BufferAddress,
                },
            }],
        });

        let vs_src = include_str!("../shader_files/wgpu.vert");
        let fs_src = include_str!("../shader_files/wgpu.frag");

        let vertex_spirv =
            glsl_to_spirv::compile(vs_src, glsl_to_spirv::ShaderType::Vertex).unwrap();
        let fragment_spirv =
            glsl_to_spirv::compile(fs_src, glsl_to_spirv::ShaderType::Fragment).unwrap();

        let vs_data = wgpu::read_spirv(vertex_spirv).unwrap();
        let fs_data = wgpu::read_spirv(fragment_spirv).unwrap();

        let vs_module = device.create_shader_module(&vs_data);
        let fs_module = device.create_shader_module(&fs_data);
        // This specifies the memorylayout for the shaders I think, ex uniforms and in/out variables
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&texture_bind_group_layout, &uniforms_bind_group_layout],
        });

        // given a layout one can create a render pipeline which defines the buffers, shaders used.
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::Back,
                depth_bias: 0,               // todo: look this up
                depth_bias_slope_scale: 0.0, // todo: look in opengl tutorial regaring the depth buffer I think this is related to it
                depth_bias_clamp: 0.0,
            }),
            color_states: &[ColorStateDescriptor {
                format: swap_chain_desc.format,
                color_blend: BlendDescriptor::REPLACE,
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            primitive_topology: PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            index_format: IndexFormat::Uint16,
            vertex_buffers: &[Vertex::desc()], // this is where vertex buffer descriptions fit
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        // This should obviously not be created here or stored in the renderer itself
        // This is equivalent to GenBuffers + BindBuffer and adding data
        let vertex_buffer = device
            .create_buffer_mapped(VERTICES.len(), BufferUsage::VERTEX)
            .fill_from_slice(VERTICES);
        // I think all "create_buffer_mapped" are a special function that's only really used for these hardcoded scenarios
        // not sure though
        let index_buffer = device
            .create_buffer_mapped(INDICES.len(), BufferUsage::INDEX)
            .fill_from_slice(INDICES);

        WgpuRenderer {
            surface,
            adapter,
            device,
            uniform_bind_group,
            uniforms,
            uniform_buffer,
            queue,
            diffuse_texture,
            diffuse_bind_group,
            vertex_buffer,
            index_buffer,
            render_pipeline,
            swap_chain_desc,
            swap_chain,
            width: width as u32,
            height: height as u32,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.height = height;
        self.width = width;

        self.swap_chain_desc.width = width;
        self.swap_chain_desc.height = height;

        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_desc);
    }

    pub fn render(&mut self) {
        let frame = self.swap_chain.get_next_texture();

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
            todo: 0 // lol
        });

        {
            /*
             It looks like you describe where you should render with attachements in the render pass
             and then attach the pipeline to the actual render pass, the pipeline defines shaders etc

             you fill buffers and add vertex attribute descriptions on the device itself
             probably the exakt same for uniforms (yup
            */
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
                depth_stencil_attachment: None,
            });
            // equivalent to use_program I think?
            render_pass.set_pipeline(&self.render_pipeline);

            //let num_verticies = VERTICES.len() as u32;
            let num_indexes = INDICES.len() as u32;

            // set bindgroup is basically setting uniforms
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);

            // this is what is done in thight loops and reading from the different objects
            // vertex buffers potentially a component? Complicated when instance rendering perhaps
            // this basically is BindVertexBuffer
            render_pass.set_vertex_buffers(0, &[(&self.vertex_buffer, 0)]);
            // only possible to have one at a time
            render_pass.set_index_buffer(&self.index_buffer, 0);
            // base_vertex is like an offset where to start so one can store multiple different indexes in the same buffer
            render_pass.draw_indexed(0..num_indexes, 0, 0..1);
        }
        // submit the commands to the graphics card.
        self.queue.submit(&[encoder.finish()])
    }
}
