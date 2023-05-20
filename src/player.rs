use cgmath::{Point3, Deg};

pub struct Player {
	pub position: Point3<f32>,

	pub buffer: wgpu::Buffer,
}