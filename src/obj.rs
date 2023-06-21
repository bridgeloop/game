use std::{path::Path, assert_eq};

use crate::texture;
use image;
use wgpu::util::DeviceExt;

fn load_bytes<T: AsRef<Path>>(file_name: T) -> Vec<u8> {
	return std::fs::read(file_name.as_ref()).unwrap();
}
fn load_texture(
	file_name: &str,
	device: &wgpu::Device,
	queue: &wgpu::Queue,
) -> texture::Texture {
	let bytes = load_bytes(file_name);
	return texture::Texture::from_image(device, queue, &(image::load_from_memory(&(bytes)).unwrap()), Some(file_name));
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	pub position: [f32; 3],
	pub tex_coords: [f32; 2],
}

pub struct Material {
	pub diffuse_texture: texture::Texture,
	pub bind_group: wgpu::BindGroup,
}

pub struct Mesh {
	pub vertex_buffer: wgpu::Buffer,
	pub index_buffer: wgpu::Buffer,
	pub num_elements: u32,
	pub material: usize,
}

pub struct Model {
	pub meshes: Vec<Mesh>,
	pub materials: Vec<Material>,
}

pub fn load_obj(
	file_name: &str,
	device: &wgpu::Device,
	queue: &wgpu::Queue,
	layout: &wgpu::BindGroupLayout,
) -> Model {
	let (models, obj_materials) = tobj::load_obj(
		file_name,
		&(tobj::GPU_LOAD_OPTIONS)
	).unwrap();

	let mut materials = Vec::new();
	for m in obj_materials.unwrap() {
		let x = format!("models/ruby/{}", m.diffuse_texture.unwrap()); // fixme
		let diffuse_texture = load_texture(&(x), device, queue);
		let bind_group = device.create_bind_group(&(wgpu::BindGroupDescriptor {
			layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&(diffuse_texture.view)),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&(diffuse_texture.sampler)),
				},
			],
			label: None,
		}));

		materials.push(Material {
			diffuse_texture,
			bind_group,
		})
	}

	// models is a Vec of struct { mesh: Mesh, name: String }
	let mut meshes = Vec::<Mesh>::new();
	for model in models {
		let mesh = model.mesh;

		let (positions, _) = mesh.positions.as_chunks::<3>();
		let (texcoords, _) = mesh.texcoords.as_chunks::<2>();
		assert_eq!(positions.len(), texcoords.len());

		let mut vertices = Vec::<Vertex>::new();
		for index in 0..positions.len() {
			vertices.push(Vertex {
				position: positions[index],
				tex_coords: [texcoords[index][0], 1.0 - texcoords[index][1]],
			});
		}

		let vertex_buffer = device.create_buffer_init(&(wgpu::util::BufferInitDescriptor {
			label: Some(&(format!("{:?} vertex buffer", file_name))),
			contents: bytemuck::cast_slice(&(vertices)),
			usage: wgpu::BufferUsages::VERTEX,
		}));
		let index_buffer = device.create_buffer_init(&(wgpu::util::BufferInitDescriptor {
			label: Some(&(format!("{:?} index buffer", file_name))),
			contents: bytemuck::cast_slice(&(mesh.indices)),
			usage: wgpu::BufferUsages::INDEX,
		}));

		meshes.push(Mesh {
			vertex_buffer,
			index_buffer,
			num_elements: mesh.indices.len() as u32,
			material: mesh.material_id.unwrap(),
		});
	}

	Model { meshes, materials }
}
