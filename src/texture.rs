pub struct Texture {
	pub texture: wgpu::Texture,
	pub view: wgpu::TextureView,
	pub sampler: wgpu::Sampler,
}

impl Texture {
	pub fn solid(
		device: &wgpu::Device,
		queue: &wgpu::Queue,
		rgba: u32,
		label: Option<&str>
	) -> Self {
		let size = wgpu::Extent3d {
			width: 1,
			height: 1,
			depth_or_array_layers: 1,
		};
		let texture = device.create_texture(&(wgpu::TextureDescriptor {
			label,
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			view_formats: &[],
		}));

		queue.write_texture(
			wgpu::ImageCopyTexture {
				aspect: wgpu::TextureAspect::All,
				texture: &(texture),
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
			},
			&(rgba.to_be_bytes()),
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(1 * 4),
				rows_per_image: Some(1),
			},
			size
		);

		let view = texture.create_view(&(wgpu::TextureViewDescriptor::default()));
		let sampler = device.create_sampler(&(wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		}));
		
		return Self { texture, view, sampler };
	}

	pub fn from_image(
		device: &wgpu::Device,
		queue: &wgpu::Queue,
		img: &image::DynamicImage,
		label: Option<&str>
	) -> Self {
		let rgba = img.to_rgba8();
		let dimensions = rgba.dimensions();

		let size = wgpu::Extent3d {
			width: dimensions.0,
			height: dimensions.1,
			depth_or_array_layers: 1,
		};
		let texture = device.create_texture(&(wgpu::TextureDescriptor {
			label,
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			view_formats: &[],
		}));

		queue.write_texture(
			wgpu::ImageCopyTexture {
				aspect: wgpu::TextureAspect::All,
				texture: &(texture),
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
			},
			&(rgba),
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(dimensions.0 * 4),
				rows_per_image: Some(dimensions.1),
			},
			size
		);

		let view = texture.create_view(&(wgpu::TextureViewDescriptor::default()));
		let sampler = device.create_sampler(&(wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		}));
		
		return Self { texture, view, sampler };
	}
}
