pub mod window {
    use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

    use wgpu::{Backends, Instance, InstanceDescriptor, RequestAdapterOptions, util::DeviceExt};

    
    
    const SHADER: &str = r#"
    struct VertexInput {
        @location(0) position: vec3<f32>,
        @location(1) color: vec3<f32>,
    };
    
    struct VertexOutput {
        @builtin(position) clip_position: vec4<f32>,
        @location(0) color: vec3<f32>,
    };
    
    @vertex
    fn vs_main(
        model: VertexInput,
    ) -> VertexOutput {
        var out: VertexOutput;
        out.color = model.color;
        out.clip_position = vec4<f32>(model.position, 1.0);
        return out;
    }
    
    // Fragment shader
    
    @fragment
    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
        return vec4<f32>(in.color, 1.0);
    }

    "#;
    
    struct State {
        surface: wgpu::Surface,
        device: wgpu::Device,
        queue: wgpu::Queue,
        config: wgpu::SurfaceConfiguration,
        size: winit::dpi::PhysicalSize<u32>,
        window: winit::window::Window,
        render_pipeline: wgpu::RenderPipeline,
        vertex_buffer: wgpu::Buffer,
        num_vertices: u32,
    }
    
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    struct Vertex {
        position: [f32; 3],
        color: [f32; 3],
    }
    unsafe impl bytemuck::Pod for Vertex {}
    unsafe impl bytemuck::Zeroable for Vertex {}

    const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5,  0.5, 0.0], color: [1.0, 0.0, 0.0] }, // Top-left
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] }, // Bottom-left
    Vertex { position: [ 0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] }, // Bottom-right

    Vertex { position: [ 0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] }, // Bottom-right
    Vertex { position: [ 0.5,  0.5, 0.0], color: [1.0, 1.0, 0.0] }, // Top-right
    Vertex { position: [-0.5,  0.5, 0.0], color: [1.0, 0.0, 0.0] }, // Top-left
    ];

    impl Vertex {
        const ATTRIBS: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
    
        fn desc() -> wgpu::VertexBufferLayout<'static> {
            use std::mem;
    
            wgpu::VertexBufferLayout {
                array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &Self::ATTRIBS,
            }
        }
    }
     
    impl State {
        fn update(&mut self) {}
        fn input(&mut self, _event: &WindowEvent) -> bool {
            false
        }
        fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
            let output = self.surface.get_current_texture()?;
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                render_pass.set_pipeline(&self.render_pipeline); // 2.
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.draw(0..self.num_vertices, 0..1);
            }

            self.queue.submit(std::iter::once(encoder.finish()));
            output.present();

            Ok(())
        }
        fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
            if new_size.width == 0 && new_size.height == 0 {
                return;
            }
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }

        async fn new(window: winit::window::Window) -> Self {
            let size = window.inner_size();
            let num_vertices = VERTICES.len() as u32;

            let instance = Instance::new(InstanceDescriptor {
                backends: Backends::PRIMARY,
                ..InstanceDescriptor::default()
            });

            let surface = unsafe { instance.create_surface(&window) }.unwrap();

            let options = RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            };

            let adapter = instance.request_adapter(&options).await;

            let adapter = match adapter {
                Some(adapter) => adapter,
                None => instance
                    .request_adapter(&wgpu::RequestAdapterOptions::default())
                    .await
                    .expect("Failed to find any suitable adapter"),
            };
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: wgpu::Features::empty(),
                        // Just in case I want wasm support later
                        limits: if cfg!(target_arch = "wasm32") {
                            wgpu::Limits::downlevel_webgl2_defaults()
                        } else {
                            wgpu::Limits::default()
                        },
                        label: None,
                    },
                    None,
                )
                .await
                .unwrap();

            let surface_caps = surface.get_capabilities(&adapter);

            let surface_format = surface_caps
                .formats
                .iter()
                .copied()
                .filter(|f| f.is_srgb())
                .next()
                .unwrap_or(surface_caps.formats[0]);

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
            };

            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(SHADER.into()),
            });

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
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    // 3.
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        // 4.
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
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

            let vertex_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );

            surface.configure(&device, &config);

            Self {
                window,
                surface,
                device,
                queue,
                config,
                size,
                render_pipeline,
                vertex_buffer,
                num_vertices
            }
        }
    }

    pub async fn run(title: &str) {
        env_logger::init();
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .build(&event_loop)
            .expect("Window could not be created");

        let mut state = State::new(window).await;

        let _ = event_loop.run(move |event, _, control_flow| match event {
            Event::RedrawRequested(window_id) if window_id == state.window.id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => control_flow.set_exit(),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // redraw loop
                // state.window.request_redraw();
            }
            Event::WindowEvent { window_id, event } if window_id == state.window.id() => {
                if !state.input(&event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        } => {
                            println!("Closed window!");
                            control_flow.set_exit();
                        }
                        WindowEvent::Resized(physical_size) => {
                            state.resize(physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(*new_inner_size);
                        }
                        _ => {}
                    };
                };
            }
            _ => (),
        });
    }
}
// 325 lines for square