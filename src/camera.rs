use cgmath::{Vector3, Point3, InnerSpace, Deg, Rad, Matrix4, Vector2};
use crate::input::Input;

#[derive(Debug)]
pub struct Camera {
	pub position: Option<Point3<f32>>,

	pub rot: Vector2<Deg<f32>>,

	aspect: f32,
	fovy: Deg<f32>,
	znear: f32,
	zfar: f32,
}

fn pitch_clamp(pitch: f32) -> Deg<f32> {
	const PITCH_LIM: f32 = 90.0 - 0.0001;
	return Deg(pitch.clamp(-PITCH_LIM, PITCH_LIM));
}
impl Camera {
	pub fn new(
		dimensions: winit::dpi::PhysicalSize<u32>
	) -> Self {
		return Self {
			position: None,
			rot: (Deg(90.0 /* 90deg because position updates use player's forward_right, where player's rot_x is 90deg */), Deg(0.0)).into(),

			aspect: dimensions.width as f32 / dimensions.height as f32,
			fovy: Deg(40.0),
			znear: 0.1,
			zfar: 100.0,
		};
	}

	pub fn reconfigure(&mut self, dimensions: winit::dpi::PhysicalSize<u32>) {
		self.aspect = dimensions.width as f32 / dimensions.height as f32;
		return;
	}

	pub fn set_pos(&mut self, pos: Point3<f32>) {
		self.position = Some(pos);
		return;
	}

	pub fn update_rot(&mut self, input: &Input, sf: f32) {
		// doesn't need dt because the input is not continuous.
		let (dx, dy) = input.mouse_moved;

		self.rot.x += Deg((dx / input.dots_per_deg) * sf);
		self.rot.y = pitch_clamp(self.rot.y.0 + (-dy / input.dots_per_deg * sf));

		return;
	}
}

pub struct CameraUniform {
	buffer: wgpu::Buffer,
}
impl<'a> CameraUniform {
	// https://github.com/sotrh/learn-wgpu/issues/478
	const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
		1.0, 0.0, 0.0, 0.0,
		0.0, 1.0, 0.0, 0.0,
		0.0, 0.0, 0.5, 0.5,
		0.0, 0.0, 0.0, 1.0,
	);
	const SIZE: usize = std::mem::size_of::<[[f32; 4]; 4]>();

	pub fn new(device: &wgpu::Device) -> Self {
		return Self {
			buffer: device.create_buffer(
				&(wgpu::BufferDescriptor {
					label: Some("Camera Buffer"),
					usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
					size: Self::SIZE as u64,
					mapped_at_creation: false,
				})
			),
		};
	}
	pub fn set_view_projection_matrix(&self, queue: &wgpu::Queue, camera: &Camera) {
		let (sin_yaw, cos_yaw) = Rad::from(camera.rot.x).0.sin_cos();
		let (sin_pitch, cos_pitch) = Rad::from(camera.rot.y).0.sin_cos();
		let target = Vector3::new(
			cos_pitch * cos_yaw,
			sin_pitch,
			cos_pitch * sin_yaw
		).normalize();

        let view = Matrix4::look_to_rh(camera.position.unwrap(), target, Vector3::unit_y());
		let proj = cgmath::perspective(camera.fovy, camera.aspect, camera.znear, camera.zfar);
		let transformed_proj: [[f32; 4]; 4] = (Self::OPENGL_TO_WGPU_MATRIX * proj * view).into();
		queue.write_buffer(&(self.buffer), 0, bytemuck::cast_slice(&(transformed_proj)));

		return;
	}
	pub fn as_entire_binding(&self) -> wgpu::BindingResource {
		return self.buffer.as_entire_binding();
	}
}