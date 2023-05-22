use cgmath::{Vector3, Point3, InnerSpace, Deg, Rad, Matrix4};
use crate::{input::Input, player::Player};

#[derive(Debug)]
pub struct Camera {
	pub position: Option<Point3<f32>>,
	pub target: Option<Point3<f32>>,

	pub rot_y: Deg<f32>,

	aspect: f32,
	fovy: Deg<f32>,
	znear: f32,
	zfar: f32,
}

fn pitch_clamp(pitch: f32) -> Deg<f32> {
	const PITCH_LIM: f32 = 90.0 - (0.0001 * 10.0);
	return Deg(pitch.clamp(-PITCH_LIM, PITCH_LIM));
}
impl Camera {
	pub fn new(
		dimensions: winit::dpi::PhysicalSize<u32>,
		rot_y: Deg<f32>,
	) -> Self {
		return Self {
			position: None,
			target: None,
			rot_y: pitch_clamp(rot_y.0),

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

	pub fn update_pos(&mut self, player: &Player) {
		let mut target = player.position + (player.forward_right().1 * (0.1 /* player half width */ + 0.05 /* additional offset */));
		target.y += 0.25; // player half height

		let (sin_yaw, cos_yaw) = Rad::from(player.rot_x).0.sin_cos();
		let (sin_pitch, cos_pitch) = Rad::from(self.rot_y).0.sin_cos();

		let v = Vector3::new(
			cos_pitch * cos_yaw,
			sin_pitch,
			cos_pitch * sin_yaw
		).normalize();

		self.position = Some(target - v);
		self.target = Some(target);

		return;
	}

	pub fn update_rot(&mut self, input: &Input, sf: f32) {
		// doesn't need dt because the input is not continuous.
		let (_, dy) = input.mouse_moved;

		self.rot_y = pitch_clamp(self.rot_y.0 + ((-dy / input.dots_per_deg) * sf));

		return;
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
		/*let (sin_yaw, cos_yaw) = Rad::from(camera.rot.x).0.sin_cos();
		let (sin_pitch, cos_pitch) = Rad::from(camera.rot.y).0.sin_cos();
		let target = Vector3::new(
			cos_pitch * cos_yaw,
			sin_pitch,
			cos_pitch * sin_yaw
		).normalize();*/

        let view = Matrix4::look_to_rh(camera.position.unwrap(), camera.target.unwrap() - camera.position.unwrap(), Vector3::unit_y());
		let proj = cgmath::perspective(camera.fovy, camera.aspect, camera.znear, camera.zfar);
		let transformed_proj: [[f32; 4]; 4] = (Self::OPENGL_TO_WGPU_MATRIX * proj * view).into();
		queue.write_buffer(&(self.buffer), 0, bytemuck::cast_slice(&(transformed_proj)));

		return;
	}
	pub fn as_entire_binding(&self) -> wgpu::BindingResource {
		return self.buffer.as_entire_binding();
	}
}