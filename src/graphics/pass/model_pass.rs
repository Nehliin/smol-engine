use crate::camera::Camera;
use crate::graphics::Pass;
use image::load;
use legion::prelude::{Resources, World};
use nalgebra::{Matrix, Matrix4, Vector3};
use std::slice::from_raw_parts_mut;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayoutBinding, BindGroupLayoutDescriptor, Binding,
    BindingResource, BindingType, BlendDescriptor, Buffer, BufferAddress, BufferUsage, Color,
    ColorStateDescriptor, ColorWrite, CommandEncoder, CommandEncoderDescriptor, CreateBufferMapped,
    CullMode, Device, FrontFace, IndexFormat, InputStepMode, LoadOp, PipelineLayoutDescriptor,
    PrimitiveTopology, ProgrammableStageDescriptor, RasterizationStateDescriptor,
    RenderPassColorAttachmentDescriptor, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderStage, StoreOp, SwapChainOutput, Texture, TextureFormat,
    TextureView, VertexAttributeDescriptor, VertexBufferDescriptor, VertexFormat,
};

// Pass associated type?
// this is the actual uniform buffer
// it's available for every shader invocation instead of copying it every time
#[repr(C)]
#[derive(Copy, Clone)]
struct VertexUniforms {
    view: Matrix4<f32>,
    projection: Matrix4<f32>,
}

impl VertexUniforms {
    fn new() -> Self {
        VertexUniforms {
            view: Matrix4::identity(),
            projection: Matrix4::identity(),
        }
    }
}

trait VBDesc {
    fn desc<'a>() -> VertexBufferDescriptor<'a>;
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InstanceRaw {
    model: Matrix4<f32>,
}

impl InstanceRaw {
    fn new(model: Matrix4<f32>) -> Self {
        InstanceRaw { model }
    }
}
const FLOAT_SIZE: BufferAddress = std::mem::size_of::<f32>() as BufferAddress;

impl VBDesc for InstanceRaw {
    fn desc<'a>() -> VertexBufferDescriptor<'a> {
        VertexBufferDescriptor {
            stride: std::mem::size_of::<InstanceRaw>() as BufferAddress,
            step_mode: InputStepMode::Instance,
            // Note that all of these attributes combined describe the matrix
            // we can't actually create a single attribute since the size limit
            // is for floating point values, thus we will need to create 4 rows manually
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    format: VertexFormat::Float4,
                    shader_location: 2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: FLOAT_SIZE * 4,
                    format: VertexFormat::Float4,
                    shader_location: 3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: FLOAT_SIZE * 4 * 2,
                    format: VertexFormat::Float4,
                    shader_location: 4,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: FLOAT_SIZE * 4 * 3,
                    format: VertexFormat::Float4,
                    shader_location: 5,
                },
            ],
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone)]
struct Vertex {
    position: Vector3<f32>,
    normal: Vector3<f32>,
}

impl VBDesc for Vertex {
    fn desc<'a>() -> VertexBufferDescriptor<'a> {
        VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: InputStepMode::Vertex,
            attributes: &[
                VertexAttributeDescriptor {
                    offset: 0,
                    format: VertexFormat::Float3,
                    shader_location: 0,
                },
                VertexAttributeDescriptor {
                    offset: FLOAT_SIZE * 3,
                    shader_location: 1,
                    format: VertexFormat::Float3,
                },
            ],
        }
    }
}

// make general over path later
fn load_shader() -> (Vec<u32>, Vec<u32>) {
    let vs_src = include_str!("../../shader_files/vs_test_shader.shader");
    let fs_src = include_str!("../../shader_files/fs_test_shader.shader");

    let vs_spirv = glsl_to_spirv::compile(vs_src, glsl_to_spirv::ShaderType::Vertex).unwrap();
    let fs_spirv = glsl_to_spirv::compile(fs_src, glsl_to_spirv::ShaderType::Fragment).unwrap();
    let vs_data = wgpu::read_spirv(vs_spirv).unwrap();
    let fs_data = wgpu::read_spirv(fs_spirv).unwrap();
    (vs_data, fs_data)
}

pub struct ModelPass<'a> {
    render_pipeline: RenderPipeline,
    uniforms: VertexUniforms, //<- filled from ecs
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
    staging_buffer: CreateBufferMapped<'a, VertexUniforms>, //instance_buffer: Buffer, // These are vertex attributes
                                                            //  depth_texture: Texture,
                                                            //depth_texture_view: TextureView
}

impl<'a> ModelPass<'a> {
    fn new(device: &'a mut wgpu::Device, format: TextureFormat) -> Self {
        let uniforms = VertexUniforms::new();
        // These might be better as methods on the vertexUniform struct
        let uniform_buffer = device
            .create_buffer_mapped(1, BufferUsage::UNIFORM | BufferUsage::COPY_DST)
            .fill_from_slice(&[&uniforms]);

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                bindings: &[
                    // This is the layout of the uniform buffer
                    BindGroupLayoutBinding {
                        binding: 0,
                        visibility: ShaderStage::VERTEX,
                        ty: BindingType::UniformBuffer { dynamic: false },
                    },
                ],
            });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            bindings: &[Binding {
                binding: 0,
                resource: BindingResource::Buffer {
                    buffer: &uniform_buffer,
                    range: 0..std::mem::size_of_val(&uniforms) as BufferAddress,
                },
            }],
        });

        let (vs_data, fs_data) = load_shader();
        let vertex_shader = device.create_shader_module(&vs_data);
        let fragment_shader = device.create_shader_module(&fs_data);

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&uniform_bind_group_layout],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: ProgrammableStageDescriptor {
                module: &vertex_shader,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fragment_shader,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::None, // TODO: change this
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format,
                alpha_blend: BlendDescriptor::REPLACE,
                color_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: IndexFormat::Uint16,
            vertex_buffers: &[Vertex::desc(), InstanceRaw::desc()],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        let staging_buffer = device.create_buffer_mapped(1, BufferUsage::COPY_DST);

        Self {
            render_pipeline,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            staging_buffer, //depth_texture: (),
                            //depth_texture_view: ()
        }
    }
}

impl<'a> Pass for ModelPass<'a> {
    fn update_uniforms(
        &mut self,
        world: &World,
        resources: &mut Resources,
        encoder: &mut CommandEncoder,
    ) {
        let camera = resources.get::<Camera>().expect("Camera to exist");
        self.uniforms.projection = *camera.get_projection_matrix();
        self.uniforms.view = *camera.get_view_matrix();

        let staging_buffer = self.staging_buffer.fill_from_slice(&[self.uniforms]);

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.uniform_buffer,
            0,
            std::mem::size_of::<VertexUniforms>() as BufferAddress,
        );
    }

    fn draw(&self, world: &World, frame: &SwapChainOutput, encoder: &mut CommandEncoder) {
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
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

        let query = <()>::query();

        render_pass.draw_model();
    }
}
