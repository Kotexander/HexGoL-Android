use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pos: [f32; 2],
}
impl Vertex {
    const fn new(pos: [f32; 2]) -> Self {
        Self { pos }
    }
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraTransform {
    scale: [f32; 2],
    offset: [f32; 2],
}
impl CameraTransform {
    fn new(scale: [f32; 2], offset: [f32; 2]) -> Self {
        Self { scale, offset }
    }
}

pub struct MeshBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}
impl MeshBuilder {
    pub fn new_hexagon(pos: [f32; 2], size: f32) -> Self {
        use std::f32::consts::FRAC_PI_3;

        let mut vertices = Vec::with_capacity(6);

        for i in 0..6 {
            let theta = i as f32 * FRAC_PI_3;
            vertices.push(Vertex::new([
                pos[0] + theta.cos() * size,
                pos[1] + theta.sin() * size,
            ]));
        }

        let indices = vec![0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5];

        Self { vertices, indices }
    }
    pub fn build(&self, ctx: &WgpuContext) -> Mesh {
        let vb = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let ib = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        Mesh::new(vb, ib, self.indices.len() as u32)
    }
}

pub struct Mesh {
    vb: wgpu::Buffer,
    ib: wgpu::Buffer,
    indices: u32,
}
impl Mesh {
    pub fn new(vb: wgpu::Buffer, ib: wgpu::Buffer, indices: u32) -> Self {
        Self { vb, ib, indices }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    scale: [f32; 2],
    offset: [f32; 2],
    color: [f32; 3],
}
impl Instance {
    pub fn new(offset: [f32; 2], scale: [f32; 2], color: [f32; 3]) -> Self {
        Self {
            scale,
            offset,
            color,
        }
    }

    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![1 => Float32x2, 2 => Float32x2, 3 => Float32x3];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}
pub struct InstancedMesh {
    mesh: Mesh,
    instance_buffer: wgpu::Buffer,
    num_instances: u32,
}
impl InstancedMesh {
    pub fn new(mesh: Mesh, ctx: &WgpuContext, instances: &[Instance]) -> Self {
        let instance_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(instances),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let num_instances = instances.len() as u32;

        Self {
            mesh,
            instance_buffer,
            num_instances,
        }
    }
    pub fn update(&mut self, ctx: &WgpuContext, instances: &[Instance]) {
        self.instance_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(instances),
                usage: wgpu::BufferUsages::VERTEX,
            });
        self.num_instances = instances.len() as u32;
    }
    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.mesh.vb.slice(..));
        render_pass.set_index_buffer(self.mesh.ib.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw_indexed(0..self.mesh.indices, 0, 0..self.num_instances);
    }
}

pub struct WgpuContext {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

struct Frame {
    output: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
}

pub struct Graphics {
    ctx: WgpuContext,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,

    frame: Option<Frame>,

    camera: CameraTransform,
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
}
impl Graphics {
    pub async fn new<W>(size: [u32; 2], window: &W) -> Self
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        // The instance is a handle to our GPU
        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);

        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let ctx = WgpuContext {
            instance,
            surface,
            device,
            queue,
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: ctx.surface.get_supported_formats(&adapter)[0],
            width: size[0],
            height: size[1],
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        ctx.surface.configure(&ctx.device, &config);

        let scale = 0.05;
        let camera = CameraTransform::new([1.0 * scale, 1.0 * scale], [0.0, 0.0]);
        let mut cb = camera.clone();
        cb.scale[1] *= size[0] as f32 / size[1] as f32;
        let camera_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[cb]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_transform_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("Transform bind group"),
                });

        let camera_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let shader = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));
        let render_pipeline_layout =
            ctx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&camera_transform_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let render_pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc(), Instance::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        let frame = None;

        Self {
            ctx,
            config,
            render_pipeline,
            frame,
            camera,
            camera_bind_group,
            camera_buffer,
        }
    }

    pub fn start_frame<'a>(&'a mut self) -> wgpu::RenderPass<'a> {
        let output = self.ctx.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.frame = Some(Frame {
            output,
            view,
            encoder,
        });

        let f = self.frame.as_mut().unwrap();

        let mut render_pass = f.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &f.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.01,
                        g: 0.01,
                        b: 0.01,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass
    }

    pub fn end_frame(&mut self) {
        let frame = self
            .frame
            .take()
            .expect("Ended frame without starting one!");
        self.ctx.queue.submit([frame.encoder.finish()]);
        frame.output.present();
    }

    pub fn context(&self) -> &WgpuContext {
        &self.ctx
    }

    pub fn resize(&mut self, new_size: [u32; 2]) {
        self.config.width = new_size[0];
        self.config.height = new_size[1];
        self.ctx.surface.configure(&self.ctx.device, &self.config);
    }

    pub fn update(&mut self) {
        let width = self.config.width as f32;
        let height = self.config.height as f32;

        let mut cb = self.camera.clone();
        cb.scale[1] *= width as f32 / height as f32;

        self.ctx
            .queue
            .write_buffer(&self.camera_buffer, 0, &bytemuck::cast_slice(&[cb]));
    }
}
