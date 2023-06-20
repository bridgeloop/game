use std::{io::{Cursor, BufReader}, path::Path};

use crate::texture;
use image;
use wgpu::util::DeviceExt;

fn load_bytes<T: AsRef<Path>>(file_name: T) -> Vec<u8> {
	return std::fs::read(file_name.as_ref()).unwrap();
}
fn load_string<T: AsRef<Path>>(file_name: T) -> String {
	return std::fs::read_to_string(file_name.as_ref()).unwrap();
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
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

pub struct Mesh {
    pub name: String,
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
	let obj_text = load_string(file_name);
	let obj_cursor = Cursor::new(obj_text);
	let mut obj_reader = BufReader::new(obj_cursor);

	let (models, obj_materials) = tobj::load_obj_buf(
		&mut(obj_reader),
		&(tobj::LoadOptions {
			triangulate: true,
			single_index: true,
			..Default::default()
		}),
		|p| {
			let p = format!("models/{}", p.to_str().unwrap()); // fixme
			let mat_text = load_string(p);
			tobj::load_mtl_buf(&mut(BufReader::new(Cursor::new(mat_text))))
		}
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
			name: m.name,
			diffuse_texture,
			bind_group,
		})
	}

	let meshes = models
		.into_iter()
		.map(|m| {
			let vertices = (0..m.mesh.positions.len() / 3)
				.map(|i| ModelVertex {
					position: [
						m.mesh.positions[i * 3],
						m.mesh.positions[i * 3 + 1],
						m.mesh.positions[i * 3 + 2],
					],
					tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
					normal: [
						m.mesh.normals[i * 3],
						m.mesh.normals[i * 3 + 1],
						m.mesh.normals[i * 3 + 2],
					],
				})
				.collect::<Vec<_>>();

			let vertex_buffer = device.create_buffer_init(&(wgpu::util::BufferInitDescriptor {
				label: Some(&(format!("{:?} Vertex Buffer", file_name))),
				contents: bytemuck::cast_slice(&(vertices)),
				usage: wgpu::BufferUsages::VERTEX,
			}));
			let index_buffer = device.create_buffer_init(&(wgpu::util::BufferInitDescriptor {
				label: Some(&(format!("{:?} Index Buffer", file_name))),
				contents: bytemuck::cast_slice(&(m.mesh.indices)),
				usage: wgpu::BufferUsages::INDEX,
			}));

			Mesh {
				name: file_name.to_string(),
				vertex_buffer,
				index_buffer,
				num_elements: m.mesh.indices.len() as u32,
				material: m.mesh.material_id.unwrap_or(0),
			}
		})
		.collect::<Vec<_>>();

	Model { meshes, materials }
}