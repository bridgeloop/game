use std::time::Duration;

use cgmath::Deg;
use winit::window::Window;
use crate::Input;
use wgpu::{util::DeviceExt};

use crate::{Camera, camera::CameraUniform};

pub struct State {
	window: Window,
	size: winit::dpi::PhysicalSize<u32>,

	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	render_pipeline: wgpu::RenderPipeline,

	vertex_buffer: wgpu::Buffer,

	camera: Camera,
	camera_uniform: CameraUniform,
	camera_bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [f32; 3],
	color: [f32; 3],
}

impl State {
	pub fn new(window: Window) -> Result<Self, &'static str> {
		let size = window.inner_size();

		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			backends: wgpu::Backends::all(),
			..Default::default()
		});

		let surface = unsafe { instance.create_surface(&(window)) }.map_err(|_| "create_surface failed")?;

		// details about gpu
		let adapter = futures::executor::block_on(instance.request_adapter(
			&(wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: Some(&(surface)),
				force_fallback_adapter: false,
			}),
		)).ok_or("request_adapter failed")?;

		// gpu connection instance + queue
		let (device, queue) = futures::executor::block_on(adapter.request_device(
			&(wgpu::DeviceDescriptor {
				features: wgpu::Features::empty(),
				limits: wgpu::Limits::default(),
				label: None,
			}),
			None,
		)).map_err(|_| "request_device failed")?;

		let config = {
			let caps = surface.get_capabilities(&(adapter));
			wgpu::SurfaceConfiguration {
				usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
				format: caps
					.formats
					.into_iter()
					.find(|f| f.is_srgb())
					.ok_or("no srgb surface")?,
				width: size.width,
				height: size.height,
				present_mode: caps.present_modes[0],
				alpha_mode: caps.alpha_modes[0],
				view_formats: vec![],
			}
		};
		surface.configure(&(device), &(config));  

		let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

		let vertex_buffer = device.create_buffer_init(
			&(wgpu::util::BufferInitDescriptor {
				label: Some("Vertex Buffer"),
				contents: bytemuck::cast_slice(&[
					Vertex { position: [0.0, 0.5, -0.5], color: [0.0, 0.0, 0.1] }, // top
					Vertex { position: [-0.5, -0.5, -1.0], color: [0.0, 1.0, 0.0] }, // left
					Vertex { position: [-0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0] }, // right

					Vertex { position: [0.0, 0.5, -0.5], color: [1.0, 0.0, 0.0] }, // top
					Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] }, // left
					Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] }, // right

					Vertex { position: [0.0, 0.5, -0.5], color: [0.0, 1.0, 0.0] }, // top
					Vertex { position: [0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0] }, // left
					Vertex { position: [0.5, -0.5, -1.0], color: [0.0, 0.0, 1.0] }, // right

					Vertex { position: [0.0, 0.5, -0.5], color: [1.0, 0.0, 0.0] }, // top
					Vertex { position: [0.5, -0.5, -1.0], color: [0.0, 0.0, 1.0] }, // left
					Vertex { position: [-0.5, -0.5, -1.0], color: [0.0, 1.0, 0.0] }, // right
				]),
				usage: wgpu::BufferUsages::VERTEX,
			})
		);

		let camera = Camera::new(size, (0.0, 0.0, 0.0), Deg(-90.0), Deg(-20.0));

		let camera_bind_group_layout = &(device.create_bind_group_layout(&(wgpu::BindGroupLayoutDescriptor {
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
				}
			],
			label: Some("camera_bind_group_layout"),
		})));

		let render_pipeline_layout = device.create_pipeline_layout(&(wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[camera_bind_group_layout],
			push_constant_ranges: &[],
		}));
		let render_pipeline = 
			device.create_render_pipeline(&(wgpu::RenderPipelineDescriptor {
				label: Some("Render Pipeline"),
				layout: Some(&(render_pipeline_layout)),
				vertex: wgpu::VertexState {
					module: &(shader),
					entry_point: "vs_main",
					buffers: &[
						// index 0
						wgpu::VertexBufferLayout {
							array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
							step_mode: wgpu::VertexStepMode::Vertex,
							attributes: &(wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3]),
						},
					],
				},
				fragment: Some(wgpu::FragmentState {
					module: &(shader),
					entry_point: "fs_main",
					targets: &[Some(wgpu::ColorTargetState {
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
			}));

		let camera_uniform = CameraUniform::new(&(device));
		camera_uniform.set_view_projection_matrix(&(queue), &(camera));

		let camera_bind_group = device.create_bind_group(&(wgpu::BindGroupDescriptor {
			layout: camera_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: camera_uniform.as_entire_binding(),
				}
			],
			label: Some("camera_bind_group"),
		}));

		return Ok(Self {
			window,
			size,

			surface,
			device,
			queue,
			config,
			render_pipeline,

			vertex_buffer,

			camera,
			camera_uniform,
			camera_bind_group,
		});
	}

	pub fn window(&self) -> &Window {
		return &(self.window);
	}

	pub fn reconfigure(&mut self, new_size: Option<winit::dpi::PhysicalSize<u32>>) {
		let new_size = new_size.unwrap_or(self.size);
		assert!(new_size.width > 0 && new_size.height > 0);
		self.size = new_size;
		self.config.width = new_size.width;
		self.config.height = new_size.height;
		self.surface.configure(&(self.device), &(self.config));
		self.camera.reconfigure(new_size);
	}

	pub fn update(&mut self, input: &Input, dt: Duration) {
		self.camera.update(input, dt);
		self.camera_uniform.set_view_projection_matrix(&(self.queue), &(self.camera));
	}

	pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&(wgpu::TextureViewDescriptor::default()));
		let mut encoder = self.device.create_command_encoder(&(wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder"),
		}));
		{
			let mut render_pass = encoder.begin_render_pass(&(wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &(view),
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(Default::default()),
						store: true,
					},
				})],
				depth_stencil_attachment: None,
			}));
			render_pass.set_pipeline(&(self.render_pipeline));

			render_pass.set_bind_group(0, &(self.camera_bind_group), &[]);
			render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
			render_pass.draw(0..12, 0..1);
		}

		// submit will accept anything that implements IntoIter
		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();

		return Ok(());
	}
}