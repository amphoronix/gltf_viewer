use anyhow::Result;

use crate::error::Error;
use crate::resource::cubemap::CubeMapLoader;

pub struct CubeMap {
    #[allow(dead_code)]
    pub gpu_texture: wgpu::Texture,
    pub gpu_texture_view: wgpu::TextureView,
    pub gpu_sampler: wgpu::Sampler,
}

impl CubeMap {
    pub fn from_texture(
        gpu_texture: wgpu::Texture,
        name: &str,
        device: &wgpu::Device,
    ) -> Result<Self> {
        if gpu_texture.dimension() != wgpu::TextureDimension::D2 {
            return Err(Error::new(format!(
                "The given texture does not have the required dimension (required=D2): {:?}",
                gpu_texture.dimension(),
            ))
            .into());
        }

        if gpu_texture.depth_or_array_layers() != 6 {
            return Err(
                Error::new(
                    format!(
                        "The given texture does not have the required number of depth/array layers (required=6): {}",
                        gpu_texture.depth_or_array_layers(),
                    )
                ).into()
            );
        }

        let gpu_texture_view = gpu_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("{name}_TEXTURE_VIEW")),
            format: Some(gpu_texture.format()),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(gpu_texture.mip_level_count()),
            base_array_layer: 0,
            array_layer_count: Some(6),
        });

        let gpu_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{name}_SAMPLER")),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Ok(Self {
            gpu_texture,
            gpu_texture_view,
            gpu_sampler,
        })
    }

    pub fn from_loader(
        loader: &impl CubeMapLoader,
        name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Self> {
        let (width, height) = loader.face_dimensions();
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 6,
        };

        let mip_level_count = loader.mip_level_count();
        let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{name}_TEXTURE")),
            size: texture_size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        for mip_level in 0..mip_level_count {
            let face_width = width / 2_u32.pow(mip_level);
            let face_height = height / 2_u32.pow(mip_level);

            CubeMap::write_to_face(
                &gpu_texture,
                mip_level,
                0,
                face_width,
                face_height,
                bytemuck::cast_slice(loader.load_positive_x_face(mip_level)?),
                queue,
            );
            CubeMap::write_to_face(
                &gpu_texture,
                mip_level,
                1,
                face_width,
                face_height,
                bytemuck::cast_slice(loader.load_negative_x_face(mip_level)?),
                queue,
            );
            CubeMap::write_to_face(
                &gpu_texture,
                mip_level,
                2,
                face_width,
                face_height,
                bytemuck::cast_slice(loader.load_positive_y_face(mip_level)?),
                queue,
            );
            CubeMap::write_to_face(
                &gpu_texture,
                mip_level,
                3,
                face_width,
                face_height,
                bytemuck::cast_slice(loader.load_negative_y_face(mip_level)?),
                queue,
            );
            CubeMap::write_to_face(
                &gpu_texture,
                mip_level,
                4,
                face_width,
                face_height,
                bytemuck::cast_slice(loader.load_positive_z_face(mip_level)?),
                queue,
            );
            CubeMap::write_to_face(
                &gpu_texture,
                mip_level,
                5,
                face_width,
                face_height,
                bytemuck::cast_slice(loader.load_negative_z_face(mip_level)?),
                queue,
            );
        }

        queue.submit([]);

        CubeMap::from_texture(gpu_texture, name, device)
    }

    pub fn create_default_cubemap(
        name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<CubeMap> {
        let image_data: [u8; 4] = [255, 255, 255, 255];
        let image_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 6,
        };

        let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{name}_TEXTURE")),
            size: image_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        for layer in 0..6 {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &gpu_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(&image_data),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
        }

        queue.submit([]);

        CubeMap::from_texture(gpu_texture, name, device)
    }

    fn write_to_face(
        gpu_texture: &wgpu::Texture,
        mip_level: u32,
        face_index: u32,
        width: u32,
        height: u32,
        data: &[u8],
        queue: &wgpu::Queue,
    ) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: gpu_texture,
                mip_level,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: face_index,
                },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * (std::mem::size_of::<half::f16>() as u32) * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}
