mod camera;
mod cube;
mod model;
mod resources;
mod texture;

use bytemuck::Pod;
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

fn print_mat4(mat: &glam::Mat4) {
    println!(
        "[[{:8.4}, {:8.4}, {:8.4}, {:8.4}]",
        mat.x_axis[0], mat.y_axis[0], mat.z_axis[0], mat.w_axis[0]
    );
    println!(
        " [{:8.4}, {:8.4}, {:8.4}, {:8.4}]",
        mat.x_axis[1], mat.y_axis[1], mat.z_axis[1], mat.w_axis[1]
    );
    println!(
        " [{:8.4}, {:8.4}, {:8.4}, {:8.4}]",
        mat.x_axis[2], mat.y_axis[2], mat.z_axis[2], mat.w_axis[2]
    );
    println!(
        " [{:8.4}, {:8.4}, {:8.4}, {:8.4}]]",
        mat.x_axis[3], mat.y_axis[3], mat.z_axis[3], mat.w_axis[3]
    );
    println!("x_axis: {:?}", mat.x_axis);
    println!("y_axis: {:?}", mat.y_axis);
    println!("z_axis: {:?}", mat.z_axis);
    println!("w_axis: {:?}", mat.w_axis);
}

const DEFAULT_WINDOW_SIZE: PhysicalSize<u32> = PhysicalSize {
    width: 1920,
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
    model_meshes: Vec<model::Mesh>,
    projectors: Vec<camera::Projector>,
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
            features: wgpu::Features::POLYGON_MODE_LINE
                | wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER,
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
            eye: [0.0, -64.0, 0.0].into(),
            target: glam::Vec3::ZERO,
            up: glam::Vec3::Z,
            aspect: config.width as f32 / config.height as f32,
            fovy,
            znear: 0.1,
            zfar: 100.0,
        };
        let mat = camera.build_view_projection_matrix();

        let camera_controller = camera::CameraController::new(0.2);
        let camera_uniform = camera::CameraUniform {
            view_proj: [
                mat.x_axis.into(),
                mat.y_axis.into(),
                mat.z_axis.into(),
                mat.w_axis.into(),
            ],
        };

        //let mut camera_uniform = camera::CameraUniform::new();
        //camera_uniform.update_view_proj(&camera);

        let img1 = resources::load_texture("0001.png", &device, &queue)
            .await
            .unwrap();
        let mat1 = model::Material::new("0001", img1, &device, &texture_bind_group_layout);
        let img2 = resources::load_texture("0002.png", &device, &queue)
            .await
            .unwrap();
        let mat2 = model::Material::new("0002", img2, &device, &texture_bind_group_layout);
        let img3 = resources::load_texture("0003.png", &device, &queue)
            .await
            .unwrap();
        let mat3 = model::Material::new("0003", img3, &device, &texture_bind_group_layout);
        let projectors = vec![
            camera::Projector {
                pos: [7.0, -40.0, 5.0].into(),
                rot: camera::EulerDegreesXYZ([84.0, 2.0, 2.0]).into(),
                aspect: config.width as f32 / config.height as f32,
                fovy,
                znear: 0.1,
                zfar: 100.0,
                material: mat1,
            },
            camera::Projector {
                pos: [0.0, -31.0, 3.0].into(),
                rot: camera::EulerDegreesXYZ([78.0, 2.0, -4.0]).into(),
                aspect: config.width as f32 / config.height as f32,
                fovy,
                znear: 0.1,
                zfar: 100.0,
                material: mat2,
            },
            camera::Projector {
                pos: [-8.0, -33.0, 12.0].into(),
                rot: camera::EulerDegreesXYZ([69.0, 4.0, 5.0]).into(),
                aspect: config.width as f32 / config.height as f32,
                fovy,
                znear: 0.1,
                zfar: 100.0,
                material: mat3,
            },
        ];

        let projector_uniforms: Vec<camera::CameraUniform> =
            projectors.iter().map(camera::CameraUniform::from).collect();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let projector_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Projector Buffer"),
            contents: bytemuck::cast_slice(projector_uniforms.as_slice()),
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
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
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: projector_buffer.as_entire_binding(),
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
                buffers: &[model::ModelVertex::desc()],
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
        let bb_model = cube::Billboard::new("test_bb", &device).into();
        let model_meshes = resources::load_meshes("projection_objects_zup.obj", &device)
            .await
            .expect("Failed to load meshes");

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
            meshes: vec![cube_model, plane_model, bb_model],
            materials: vec![material],
            model_meshes,
            projectors,
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
        //render_pass.draw_mesh(&self.meshes[1], &self.materials[1], &self.camera_bind_group);
        self.projectors.iter()
            .for_each(|proj| self.model_meshes.iter().for_each(|m| {
                render_pass.draw_mesh(m, &proj.material, &self.camera_bind_group)
            }));
        //self.model_meshes
        //    .iter()
        //    .for_each(|m| render_pass.draw_mesh(m, &self.materials[0], &self.camera_bind_group));
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
