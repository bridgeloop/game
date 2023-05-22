use cgmath::{Deg, Rad, Point3, EuclideanSpace};
use winit::window::Window;
use wgpu::{util::DeviceExt, BufferAddress};

use crate::{camera::*, Input, player::Player};

pub struct State {
	input: Input,

	window: Window,
	size: winit::dpi::PhysicalSize<u32>,
	fullscreen: bool,
	focused: bool,

	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	render_pipeline: wgpu::RenderPipeline,

	plane_buffer: wgpu::Buffer,
	build_buffer: wgpu::Buffer,

	player: Player,
	camera: Camera,
	camera_uniform: CameraUniform,
	camera_bind_group: wgpu::BindGroup,

	depth_view: wgpu::TextureView,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [f32; 3],
	color: [f32; 3],
}

fn depth_view(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
	let desc = wgpu::TextureDescriptor {
		label: Some("texture_descriptor"),
		size: wgpu::Extent3d {
			width: config.width,
			height: config.height,
			depth_or_array_layers: 1,
		},
		mip_level_count: 1,
		sample_count: 1,
		dimension: wgpu::TextureDimension::D2,
		format: wgpu::TextureFormat::Depth32Float,
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
		view_formats: &[],
	};
	let texture = device.create_texture(&(desc));
	return texture.create_view(&(wgpu::TextureViewDescriptor::default()));
}

impl State {
	pub fn new(window: Window, input: Input) -> Result<Self, &'static str> {
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
				present_mode: wgpu::PresentMode::AutoNoVsync, // caps.present_modes[0],
				alpha_mode: caps.alpha_modes[0],
				view_formats: vec![],
			}
		};
		surface.configure(&(device), &(config));  

		let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

		let plane_buffer = device.create_buffer_init(
			&(wgpu::util::BufferInitDescriptor {
				label: Some("plane_buffer"),
				usage: wgpu::BufferUsages::VERTEX,
				contents: bytemuck::cast_slice(&[
					Vertex { position: [0.0, 0.0, 0.0], color: [0.0, 1.0, 1.0] }, // top
					Vertex { position: [5.0, 0.0, -5.0], color: [0.0, 1.0, 0.0] }, // left
					Vertex { position: [0.0, 0.0, -5.0], color: [0.0, 1.0, 0.0] }, // right

					Vertex { position: [0.0, 0.0, 0.0], color: [0.0, 1.0, 0.0] }, // top
					Vertex { position: [5.0, 0.0, -0.0], color: [0.0, 1.0, 0.0] }, // left
					Vertex { position: [5.0, 0.0, -5.0], color: [0.0, 1.0, 0.0] }, // right
				]),
			})
		);
		let build_buffer = device.create_buffer(&(wgpu::BufferDescriptor {
			label: Some("build_buffer"),
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			size: std::mem::size_of::<Vertex>() as u64 * 6,
			mapped_at_creation: false,
		}));

		let player = Player {
			position: (1.0, 0.25, -1.0).into(),
			rot_x: Deg(0.0),
			buffer: device.create_buffer(&(wgpu::BufferDescriptor {
				label: Some("player_buffer"),
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
				size: std::mem::size_of::<Vertex>() as u64 * 6,
				mapped_at_creation: false,
			})),
		};
		let mut camera = Camera::new(size, Deg(0.0));
		camera.update_pos(&(player));

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
			label: Some("render_pipeline_layout"),
			bind_group_layouts: &[camera_bind_group_layout],
			push_constant_ranges: &[],
		}));
		let render_pipeline = device.create_render_pipeline(&(wgpu::RenderPipelineDescriptor {
			label: Some("render_pipeline"),
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
				cull_mode: None, // should use Some(wgpu::Face::Back) once i 3d objects (rather than 2d ones) in space
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
				conservative: false,
			},
			depth_stencil: Some(wgpu::DepthStencilState {
				format: wgpu::TextureFormat::Depth32Float,
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
		}));

		let depth_view = depth_view(&(device), &(config));

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
			input,

			fullscreen: false,
			focused: false,
			window,
			size,

			surface,
			device,
			queue,
			config,
			render_pipeline,

			plane_buffer,
			build_buffer,

			player,
			camera,
			camera_uniform,
			camera_bind_group,

			depth_view,
		});
	}

	pub fn window(&self) -> &Window {
		return &(self.window);
	}

	pub fn set_focus(&mut self, to: bool) {
		assert!((!self.fullscreen && !to) || self.fullscreen);
		assert!(to != self.focused);

		let window = self.window();
		if to {
			drop(window
				.set_cursor_grab(winit::window::CursorGrabMode::Confined)
				.or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Locked)));
			window.set_cursor_visible(false);
		} else {
			window.set_cursor_grab(winit::window::CursorGrabMode::None).expect("failed to unlock cursor");
			window.set_cursor_visible(true);
		}

		self.focused = to;
		return;
	}
	pub fn is_focused(&self) -> bool {
		return self.focused;
	}

	pub fn set_fullscreen(&mut self, to: bool) {
		assert!(to != self.fullscreen);
		use std::cmp::Ordering;

		self.fullscreen = to;
		
		let window = self.window();
		if to {
			fn area(size: winit::dpi::PhysicalSize<u32>) -> u32 {
				size.width * size.height
			}
			let video_modes = window.current_monitor().expect("no monitor detected").video_modes();
			let video_mode = video_modes.max_by(|x, y| {
				if area(x.size()) > area(y.size()) {
					return Ordering::Greater;
				} else if area(x.size()) < area(y.size()) {
					return Ordering::Less;
				}
				if x.refresh_rate_millihertz() > y.refresh_rate_millihertz() {
					return Ordering::Greater;
				} else {
					return Ordering::Less;
				}
			}).expect("no video modes");
			window.set_fullscreen(
				Some(winit::window::Fullscreen::Exclusive(video_mode.clone()))
			);
			self.set_focus(true);
		} else {
			window.set_fullscreen(None);
			if self.focused {
				self.set_focus(false);
			}
		}

		return;
	}
	pub fn is_fullscreen(&self) -> bool {
		return self.fullscreen;
	}

	pub fn reconfigure(&mut self, new_size: Option<winit::dpi::PhysicalSize<u32>>) {
		let new_size = new_size.unwrap_or(self.size);
		assert!(new_size.width > 0 && new_size.height > 0);
		self.size = new_size;
		self.config.width = new_size.width;
		self.config.height = new_size.height;
		self.surface.configure(&(self.device), &(self.config));
		self.depth_view = depth_view(&(self.device), &(self.config));
		self.camera.reconfigure(new_size);

		return;
	}

	pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		self.camera_uniform.set_view_projection_matrix(&(self.queue), &(self.camera));
		
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&(wgpu::TextureViewDescriptor::default()));
		let mut encoder = self.device.create_command_encoder(&(wgpu::CommandEncoderDescriptor {
			label: Some("encoder"),
		}));

		let mut render_pass = encoder.begin_render_pass(&(wgpu::RenderPassDescriptor {
			label: Some("render_pass"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &(view),
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(Default::default()),
					store: true,
				},
			})],
			depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
				view: &(self.depth_view),
				depth_ops: Some(wgpu::Operations {
					load: wgpu::LoadOp::Clear(1.0),
					store: true,
				}),
				stencil_ops: None,
			}),
		}));
		render_pass.set_pipeline(&(self.render_pipeline));

		// camera
		render_pass.set_bind_group(0, &(self.camera_bind_group), &[]);

		// player
		fn rot_rect(w: f32, h: f32, r: Rad<f32>) -> [Point3<f32>; 6] {
			use cgmath::Transform;

			let hw = w / 2.0;
			let hh = h / 2.0;
			let mut vertices: [Point3<f32>; 4] = [
				[0.0, -hh, -hw].into(),
				[0.0, -hh, hw].into(),
				[0.0, hh, hw].into(),
				[0.0, hh, -hw].into(),
			];

			// Create the rotation matrix around the y-axis
			let rotation_matrix = cgmath::Matrix4::from_axis_angle(cgmath::Vector3::unit_y(), -r);

			// Apply the rotation to each vertex
			for (_, v) in vertices.iter_mut().enumerate() {
				*v = rotation_matrix.transform_point(*v);
			}

			return [
				vertices[0],
				vertices[1],
				vertices[2],
				vertices[2],
				vertices[3],
				vertices[0],
			];
		}
		// (USED TO) render the player in-front of the camera
		// this should really be done the other way
		// around though; the camera should be placed
		// *behind the player*, and the camera should
		// be affected by collision to prevent it going
		// inside of walls, etc.
		// (the camera should rotate around the player,
		//  and the player should also rotate so that
		//  its back is facing the camera.)
		self.queue.write_buffer(&(self.player.buffer), 0, bytemuck::cast_slice(&(rot_rect(
			0.2, 0.5, Rad::from(self.player.rot_x)
		).map(|point| Vertex { position: (self.player.position + point.to_vec()).into(), color: [1.0, 1.0, 1.0] }))));
		render_pass.set_vertex_buffer(0, self.player.buffer.slice(..));
		render_pass.draw(0..6, 0..1);

		// build grid
		self.queue.write_buffer(&(self.build_buffer), 0, bytemuck::cast_slice(&[
			Vertex { position: [0.0, 1.0, 0.0], color: [1.0, 0.0, 0.0] },
			Vertex { position: [0.0, 0.0, 0.0], color: [1.0, 0.0, 0.0] },
			Vertex { position: [1.0, 0.0, 0.0], color: [1.0, 0.0, 0.0] },

			Vertex { position: [0.0, 1.0, 0.0], color: [1.0, 0.0, 0.0] },
			Vertex { position: [1.0, 0.0, 0.0], color: [1.0, 0.0, 0.0] },
			Vertex { position: [1.0, 1.0, 0.0], color: [1.0, 0.0, 0.0] },
		]));
		render_pass.set_vertex_buffer(0, self.build_buffer.slice(..));
		render_pass.draw(0..6, 0..1);

		// plane
		render_pass.set_vertex_buffer(0, self.plane_buffer.slice(..));
		let n_vertices = (
			self.plane_buffer.size() / std::mem::size_of::<Vertex>() as BufferAddress
		) as u32;
		render_pass.draw(0..n_vertices, 0..1);

		drop(render_pass);

		// submit will accept anything that implements IntoIter
		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();

		return Ok(());
	}

	pub fn update_camera(&mut self, dt: f32, sf: f32) {
		self.player.update_pos(&(self.input), dt);
		self.player.update_rot(&(self.input), sf);

		self.camera.update_rot(&(self.input), sf);
		self.camera.update_pos(&(self.player));
		return;
	}
	pub fn process_key(&mut self, key: winit::event::VirtualKeyCode, state: winit::event::ElementState) {
		return self.input.process_key(key, state);
	}
	pub fn process_mouse_motion(&mut self, delta: (f64, f64)) {
		return self.input.process_mouse_motion(delta);
	}
}