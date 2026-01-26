// Vaporwave Overlay - wgpu/Rust port
// Creates a transparent fullscreen overlay with vaporwave shader effects

use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId, WindowLevel, WindowAttributes},
};

#[cfg(target_os = "macos")]
use cocoa::base::id;
#[cfg(target_os = "macos")]
use objc::runtime::YES;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    time: f32,
    opacity: f32,
    window_seed: f32,
    _padding: f32,
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    start_time: Instant,
    window: &'a Window,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::PostMultiplied,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let uniforms = Uniforms {
            time: 0.0,
            opacity: 1.0,
            window_seed: 42.0,
            _padding: 0.0,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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
            cache: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            uniform_buffer,
            uniform_bind_group,
            start_time: Instant::now(),
            window,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f32() * 1000.0;
        let uniforms = Uniforms {
            time: elapsed,
            opacity: 1.0,
            window_seed: 42.0,
            _padding: 0.0,
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
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
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw(0..4, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

struct App<'a> {
    state: Option<State<'a>>,
    window: Option<&'a Window>,
}

impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            state: None,
            window: None,
        }
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let monitor = event_loop.primary_monitor().or_else(|| {
            event_loop.available_monitors().next()
        });

        let size = monitor
            .as_ref()
            .map(|m| m.size())
            .unwrap_or(winit::dpi::PhysicalSize::new(1920, 1080));

        let window_attributes = WindowAttributes::default()
            .with_title("Vaporwave Overlay")
            .with_inner_size(size)
            .with_position(winit::dpi::PhysicalPosition::new(0, 0))
            .with_decorations(false)
            .with_transparent(true)
            .with_window_level(WindowLevel::AlwaysOnTop);

        let window = Box::leak(Box::new(
            event_loop.create_window(window_attributes).unwrap()
        ));

        // macOS-specific: make window click-through and float above all
        #[cfg(target_os = "macos")]
        {
            use winit::raw_window_handle::HasWindowHandle;
            use objc::msg_send;
            use objc::sel;
            use objc::sel_impl;

            if let Ok(handle) = window.window_handle() {
                if let winit::raw_window_handle::RawWindowHandle::AppKit(appkit) = handle.as_raw() {
                    unsafe {
                        let ns_view: id = appkit.ns_view.as_ptr() as id;
                        let ns_window: id = msg_send![ns_view, window];
                        if !ns_window.is_null() {
                            let _: () = msg_send![ns_window, setIgnoresMouseEvents: YES];
                            // CanJoinAllSpaces | Stationary | FullScreenAuxiliary
                            let behavior: u64 = (1 << 0) | (1 << 4) | (1 << 8);
                            let _: () = msg_send![ns_window, setCollectionBehavior: behavior];
                            let _: () = msg_send![ns_window, setOpaque: cocoa::base::NO];
                            let _: () = msg_send![ns_window, setHasShadow: cocoa::base::NO];
                        }
                    }
                }
            }
        }

        self.window = Some(window);
        self.state = Some(pollster::block_on(State::new(window)));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = &mut self.state {
                    state.resize(physical_size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("Render error: {:?}", e),
                    }
                }
                if let Some(window) = self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
