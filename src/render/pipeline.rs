use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};
use super::vertex::Vertex;

pub struct State {
    // swap chain
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swap_chain_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    // rendering pipeline
    pub render_pipeline: wgpu::RenderPipeline,
    // buffers
    pub vertex_buffer: wgpu::Buffer,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        // getting the window size
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();

        // device and queue from adapter
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None, // Trace path
        ).await.unwrap();

        // swap chain
        let (swap_chain_desc, swap_chain) = State::create_swap_chain(&size, &surface, &device, &adapter);

        // rendering pipeline
        let render_pipeline = State::create_render_pipeline(&device, &swap_chain_desc);

        // buffers
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(super::vertex::VERTICES),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );

        State {
            surface,
            device,
            queue,
            swap_chain_desc,
            swap_chain,
            size,
            render_pipeline,
            vertex_buffer,
        }
    }

    pub fn create_swap_chain(size: &winit::dpi::PhysicalSize<u32>, surface: &wgpu::Surface, device: &wgpu::Device, adapter: &wgpu::Adapter) -> (wgpu::SwapChainDescriptor, wgpu::SwapChain) {
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(surface, &swap_chain_desc);

        (swap_chain_desc, swap_chain)
    }

    pub fn create_render_pipeline(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor) -> wgpu::RenderPipeline {
        // loading shaders
        let vs_module = device.create_shader_module(&wgpu::include_spirv!("shaders/simple.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("shaders/simple.frag.spv"));

        // creating rendering pipeline
        let render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[
                    // to fill !!!
                    super::vertex::ColorVertex::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: swap_chain_desc.format,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0, 
                    alpha_to_coverage_enabled: false,
                },
            });

            render_pipeline
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.swap_chain_desc.width = new_size.width;
        self.swap_chain_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
    }

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false // events were not processed here!
    }

    pub fn update(&mut self) {
        // updating loop
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        // full rendering process
        let frame = self.swap_chain.get_current_frame()?.output;

        // commands encoder to send to the gpu
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // creating a render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.078,
                            g: 0.078,
                            b: 0.121,
                            a: 1.0,
                        }),
                        store: true,
                    }
                }
            ],
            depth_stencil_attachment: None,
        });

        // rendering things
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..super::vertex::VERTICES.len() as u32, 0..1);

        // we need to drop the render pass in order to avoid a memory leak
        drop(render_pass); // the commands has already be sent to the encoder
    
        // send the command encoded to the queue
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
    
        Ok(())
    }
}