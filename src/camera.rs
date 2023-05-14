use cgmath::{Vector3, Point3, Rad, InnerSpace, Deg};

use crate::Input;

#[derive(Debug)]
pub struct Camera {
	pub position: Point3<f32>,
	pub rot_x: Rad<f32>,
	pub rot_y: Rad<f32>,

	aspect: f32,
	fovy: Rad<f32>,
	znear: f32,
	zfar: f32,
}

impl Camera {
	pub fn new<
		V: Into<Point3<f32>>,
		Y: Into<Rad<f32>>,
		P: Into<Rad<f32>>,
	>(
		dimensions: winit::dpi::PhysicalSize<u32>,

		position: V,
		rot_x: Y,
		rot_y: P
	) -> Self {
		Self {
			position: position.into(),
			rot_x: rot_x.into(),
			rot_y: rot_y.into(),

			aspect: dimensions.width as f32 / dimensions.height as f32,
			fovy: Deg(45.0).into(),
			znear: 0.1,
			zfar: 100.0,
		}
	}
}

impl Camera {
	pub fn reconfigure(&mut self, dimensions: winit::dpi::PhysicalSize<u32>) {
		self.aspect = dimensions.width as f32 / dimensions.height as f32;
	}

	pub fn update(&mut self, input: &Input, dt: std::time::Duration) {
		let dt = dt.as_secs_f32();

		// Move forward/backward and left/right
		let (yaw_sin, yaw_cos) = self.rot_x.0.sin_cos();
		let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
		let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
		self.position += forward * (input.amount_forward - input.amount_backward) * input.speed * dt;
		self.position += right * (input.amount_right - input.amount_left) * input.speed * dt;

		// Move up/down. Since we don't use roll, we can just
		// modify the y coordinate directly.
		self.position.y += (input.amount_up - input.amount_down) * input.speed * dt;

		// Rotate
		self.rot_x += Rad(input.rotate_horizontal) * input.sensitivity * dt;

		let pitch = (self.rot_y + Rad(-input.rotate_vertical) * input.sensitivity * dt).0;
		let frac = std::f32::consts::FRAC_PI_2 - 0.0001;
		
		self.rot_y = Rad(pitch.clamp(-Rad(frac).0, Rad(frac).0));
	}
}

pub struct CameraUniform {
	buffer: wgpu::Buffer,
}
impl<'a> CameraUniform {
	const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
		1.0, 0.0, 0.0, 0.0,
		0.0, 1.0, 0.0, 0.0,
		0.0, 0.0, 0.5, 0.0,
		0.0, 0.0, 0.5, 1.0,
	);
	const SIZE: usize = std::mem::size_of::<[[f32; 4]; 4]>();

	pub fn new(device: &wgpu::Device) -> Self {
		Self {
			buffer: device.create_buffer(
				&(wgpu::BufferDescriptor {
					label: Some("Camera Buffer"),
					usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
					size: Self::SIZE as u64,
					mapped_at_creation: false,
				})
			),
		}
	}
	pub fn set_view_projection_matrix(&self, queue: &wgpu::Queue, camera: &Camera) {
		let (sin_yaw, cos_yaw) = camera.rot_x.0.sin_cos();
		let (sin_pitch, cos_pitch) = camera.rot_y.0.sin_cos();

		let target = Vector3::new(
			cos_pitch * cos_yaw,
			sin_pitch,
			cos_pitch * sin_yaw
		).normalize();

		let view = cgmath::Matrix4::look_to_rh(camera.position, target, Vector3::unit_y());
		let proj = cgmath::perspective(camera.fovy, camera.aspect, camera.znear, camera.zfar);
		let transformed_proj: [[f32; 4]; 4] = (Self::OPENGL_TO_WGPU_MATRIX * proj * view).into();
		queue.write_buffer(&(self.buffer), 0, bytemuck::cast_slice(&(transformed_proj)));
	}
	pub fn as_entire_binding(&self) -> wgpu::BindingResource {
		self.buffer.as_entire_binding()
	}
}