use half::vec::HalfFloatVecExt;

pub struct GgxLut {
    #[allow(dead_code)]
    pub gpu_texture: wgpu::Texture,
    pub gpu_texture_view: wgpu::TextureView,
    pub gpu_sampler: wgpu::Sampler,
}

impl GgxLut {
    pub fn default_path() -> std::path::PathBuf {
        std::path::PathBuf::from("resources/lut_ggx.png")
    }

    pub fn from_image(
        source_image: &image::Rgba32FImage,
        name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let image_dimensions = source_image.dimensions();
        let image_size = wgpu::Extent3d {
            width: image_dimensions.0,
            height: image_dimensions.1,
            depth_or_array_layers: 1,
        };

        let image_data = Vec::<half::f16>::from_f32_slice(source_image);

        let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size: image_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &gpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&image_data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * (std::mem::size_of::<u16>() as u32) * image_dimensions.0),
                rows_per_image: Some(image_dimensions.1),
            },
            image_size,
        );

        let gpu_texture_view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let gpu_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            gpu_texture,
            gpu_texture_view,
            gpu_sampler,
        }
    }
}
