mod camera;
mod cube;
mod model;
mod resources;
mod texture;

use log::{info, warn};
use model::DrawModel;
use model::Vertex;
use wgpu::util::DeviceExt;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

const DEFAULT_WINDOW_SIZE: PhysicalSize<u32> = PhysicalSize {
    width: 1080,
    height: 1080,
};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

struct Instance {
    position: glam::Vec3,
    rotation: glam::Quat,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        let mat = glam::Mat4::from_rotation_translation(self.rotation, self.position);
        InstanceRaw {
            model: [
                mat.x_axis.into(),
                mat.y_axis.into(),
                mat.z_axis.into(),
                mat.w_axis.into(),
            ],
        }
    }
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    clear_color: wgpu::Color,
    pipeline: wgpu::RenderPipeline,
    camera: camera::Camera,
    camera_controller: camera::CameraController,
    camera_uniform: camera::CameraUniform,
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    depth_texture: texture::Texture,
    materials: Vec<model::Material>,
    meshes: Vec<model::Mesh>,
}

impl State {
    async fn new(window: Window) -> Self {
        // Ensure neither width nor height is 0
        let size = match window.inner_size() {
            PhysicalSize {
                height: 0,
                width: 0,
            } => DEFAULT_WINDOW_SIZE,
            size => size,
        };

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter_options = &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };
        let adapter = instance.request_adapter(adapter_options).await.unwrap();

        let descriptor = &wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),//wgpu::Features::POLYGON_MODE_LINE,
            limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
            label: None,
        };
        let (device, queue) = adapter.request_device(descriptor, None).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let sensor_size = 24_f32;
        let focal_length = 50_f32;
        let fovy = 2.0 * ((sensor_size / focal_length) * 0.5).atan();
        let camera = camera::Camera {
            eye: [-6.0, 6.0, 6.0].into(),
            target: [0.0, 0.0, 0.0].into(),
            up: glam::Vec3::Y,
            aspect: config.width as f32 / config.height as f32,
            fovy,
            znear: 0.1,
            zfar: 100.0,
        };

        let camera_controller = camera::CameraController::new(0.2);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
            ],
            label: Some("camera_bind_group"),
        });

        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let pipeline_layout_desc = &wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        };
        let pipeline_layout = device.create_pipeline_layout(pipeline_layout_desc);

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[cube::SimpleVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let texture = resources::load_texture("image_projection_test_square.png", &device, &queue)
            .await
            .unwrap();
        let material = model::Material::new(
            "image_projection",
            texture,
            &device,
            &texture_bind_group_layout,
        );
        let cube_model = cube::Cube::new("test_cube", &device).into();
        let plane_model = cube::Plane::new("test_plane", &device).into();

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            clear_color,
            pipeline,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            depth_texture,
            meshes: vec![cube_model, plane_model],
            materials: vec![material],
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.camera.aspect = self.config.width as f32 / self.config.height as f32;
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        if self.camera_controller.process_events(event) {
            self.window.request_redraw();
            return true;
        }
        false
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        self.window.request_redraw();
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
        let render_pass_desc = &wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        };

        let mut render_pass = encoder.begin_render_pass(render_pass_desc);
        render_pass.set_pipeline(&self.pipeline);
        //self.meshes
        //    .iter()
        //    .take(2)
        //    .for_each(|m| render_pass.draw_mesh(m, &self.materials[0], &self.camera_bind_group));
        render_pass.draw_mesh(&self.meshes[1], &self.materials[0], &self.camera_bind_group);
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new()
        .with_title("Learn WGPU")
        .with_inner_size(DEFAULT_WINDOW_SIZE)
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowBuilderExtWebSys;
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm_example")?;
                let canvas = window.canvas().unwrap();
                info!("Canvas: [{}, {}]", canvas.width(), canvas.height());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(window).await;

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match &event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::KeyboardInput { event, .. } => {
                            if event.state.is_pressed() {
                                match event.physical_key {
                                    PhysicalKey::Code(KeyCode::KeyX) => {
                                        elwt.exit();
                                    }
                                    PhysicalKey::Code(KeyCode::Escape) => {
                                        elwt.exit();
                                    }
                                    _ => {}
                                }
                            }
                        }
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(e) => eprintln!("Some unhandled error {:?}", e),
                            }
                        }
                        WindowEvent::Resized(physical_size) => {
                            info!("Resize: {:?}", physical_size);
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { .. } => {
                            state.resize(state.window().inner_size());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        })
        .unwrap();
}
