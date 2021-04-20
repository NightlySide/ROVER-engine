use vertex::Vertex;
use wgpu::util::DeviceExt;
use winit::{event::*, window::Window};
use cgmath::*;
use super::{camera, uniform, vertex, light, texture, terrain};

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
    pub light_render_pipeline: wgpu::RenderPipeline,
    // buffers
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub light_buffer: wgpu::Buffer,
    // bind groups
    pub uniform_bind_group: wgpu::BindGroup,
    pub light_bind_group: wgpu::BindGroup,
    // uniforms
    pub uniforms: uniform::Uniforms,
    // textures & materials
    pub depth_texture: texture::Texture,
    // pub debug_material: vertex::Material,
    // lights
    pub light: light::Light,
    // camera
    pub camera: camera::Camera,
    pub projection: camera::Projection,
    pub camera_controller: camera::CameraController,
    // states
    //pub mouse_pressed: bool,
    pub mouse_capture: bool,
    // data
    pub num_index: u32,
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

        // camera
        let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = camera::Projection::new(swap_chain_desc.width, swap_chain_desc.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(5.0, 0.6);

        // uniforms
        let mut uniforms = uniform::Uniforms::new();
        uniforms.update_view_proj(&camera, &projection);

        // light
        let light = light::Light {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
        };

        // data
        let chunk = terrain::Chunk::new();
        let (vertices, indices) = chunk.create_mesh();
        let vertices: &[vertex::ColorVertex] = &vertices.as_slice();
        let indices: &[u16] = &indices.as_slice();
        let num_index = indices.len() as u32;

        // buffers
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("Indices Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsage::INDEX,
            }
        );
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        // bind groups layouts
        // let texture_bind_group_layout = texture::create_bind_group_layout(&device);
        let uniform_bind_group_layout = uniform::create_bind_group_layout(&device);
        let light_bind_group_layout = light::create_bind_group_layout(&device);

        // bind groups
        let uniform_bind_group = uniform::create_bind_group(
            &device, 
            &uniform_bind_group_layout, 
            &uniform_buffer
        );
        let light_bind_group = light::create_bind_group(
            &device, 
            &light_bind_group_layout, 
            &light_buffer
        );

        // texture
        let depth_texture = texture::Texture::create_depth_texture(&device, &swap_chain_desc, "depth_texture");

        // rendering pipelines
        let render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    //&texture_bind_group_layout,
                    &uniform_bind_group_layout,
                    //&light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

            State::create_render_pipeline(
                &device,
                &layout,
                swap_chain_desc.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[vertex::ColorVertex::desc()],
                wgpu::include_spirv!("shaders/simple.vert.spv"),
                wgpu::include_spirv!("shaders/simple.frag.spv"),
            )
        };
        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[
                    &uniform_bind_group_layout, 
                    &light_bind_group_layout
                ],
                push_constant_ranges: &[],
            });

            State::create_render_pipeline(
                &device,
                &layout,
                swap_chain_desc.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[vertex::ColorVertex::desc()],
                wgpu::include_spirv!("shaders/light.vert.spv"),
                wgpu::include_spirv!("shaders/light.frag.spv"),
            )
        };

        // returning the new state
        State {
            // swap chain
            surface,
            device,
            queue,
            swap_chain_desc,
            swap_chain,
            size,
            // rendering pipeline
            render_pipeline,
            light_render_pipeline,
            // buffers
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            light_buffer,
            // bind groups
            uniform_bind_group,
            light_bind_group,
            // uniforms
            uniforms,
            // textures & materials
            depth_texture,
            // debug_material,
            // lights
            light,
            // camera
            camera,
            projection,
            camera_controller,
            // states,
            //mouse_pressed: false,
            mouse_capture: false,
            // data
            num_index,
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

    fn create_render_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        color_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        vs_src: wgpu::ShaderModuleDescriptor,
        fs_src: wgpu::ShaderModuleDescriptor,
    ) -> wgpu::RenderPipeline {
        // loading shaders
        let vs_module = device.create_shader_module(&vs_src);
        let fs_module = device.create_shader_module(&fs_src);
    
        // returning the pipeling
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            // vertex shader
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: vertex_layouts,
            },
            // fragment shader
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: color_format,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            // creating faces from triangles
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::Back,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            // setting the depth stencil
            depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                // Setting this to true requires Features::DEPTH_CLAMPING
                clamp_depth: false,
            }),
            // multisampling
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.projection.resize(new_size.width, new_size.height);
        self.swap_chain_desc.width = new_size.width;
        self.swap_chain_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.swap_chain_desc, "depth_texture");
    }

    pub fn window_input(&mut self, window: &winit::window::Window, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                ..
            } => {
                if !self.mouse_capture {
                    window.set_cursor_grab(true).unwrap();
                    window.set_cursor_visible(false);
                    self.mouse_capture = true;
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    pub fn device_input(&mut self, window: &winit::window::Window, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::Key(KeyboardInput {
                virtual_keycode: Some(key),
                state,
                ..
            }) => {
                if !self.camera_controller.process_keyboard(*key, *state) {
                    if *key == VirtualKeyCode::Escape && self.mouse_capture {
                        self.mouse_capture = false;
                        window.set_cursor_grab(false).unwrap();
                        window.set_cursor_visible(true);
                        println!("Only ungrabbing cursor from window");
                        return true;
                    } 
                }
                false
            },
            DeviceEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            DeviceEvent::Button {
                button: 1, // Left Mouse Button
                state: _,
            } => {
                //self.mouse_pressed = *state == ElementState::Pressed;
                // capture mouse
                true
            }
            DeviceEvent::MouseMotion { delta } => {
                if self.mouse_capture {
                    self.camera_controller.process_mouse(delta.0, delta.1);
                }
                true
            }
            _ => false,
        }
    }

    // updating loop
    pub fn update(&mut self, dt: std::time::Duration) {
        // updating the camera
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.uniforms
            .update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );

        // Update the light
        let old_position: cgmath::Vector3<_> = self.light.position.into();
        self.light.position =
            (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                * old_position)
                .into();
        self.queue
            .write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light]));
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        // rendering things
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_index, 0, 0..1);

        // render lightt

        // we need to drop the render pass in order to avoid a memory leak
        drop(render_pass); // the commands has already be sent to the encoder
    
        // send the command encoded to the queue
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
    
        Ok(())
    }
}