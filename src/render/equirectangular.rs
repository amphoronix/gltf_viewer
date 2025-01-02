use anyhow::Result;
use half::vec::HalfFloatVecExt;

use crate::render::shader::ShaderModulePackage;

pub struct EquirectangularToCubeMapRenderer {
    device: std::rc::Rc<wgpu::Device>,
    queue: std::rc::Rc<wgpu::Queue>,
    gpu_pipeline: wgpu::RenderPipeline,
    source_bind_group_layout: wgpu::BindGroupLayout,
    face_direction_uv_mappings: [FaceDirectionUVMapping; 6],
}

impl EquirectangularToCubeMapRenderer {
    pub fn from_device(
        device: std::rc::Rc<wgpu::Device>,
        queue: std::rc::Rc<wgpu::Queue>,
        tera: &tera::Tera,
    ) -> Result<Self> {
        let source_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("EQUIRECTANGULAR_TO_CUBEMAP_BIND_GROUP_LAYOUT"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let cubemap_face_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("EQUIRECTANGULAR_TO_CUBEMAP_FACE_BIND_GROUP_LAYOUT"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("EQUIRECTANGULAR_TO_CUBEMAP_RENDER_PIPELINE_LAYOUT"),
                bind_group_layouts: &[&source_bind_group_layout, &cubemap_face_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader_module_package = ShaderModulePackage::from_templates(
            "equirectangular/fullscreen.vert",
            "equirectangular/equirectangular.frag",
            "EQUIRECTANGULAR_TO_CUBEMAP",
            &device,
            tera,
            None,
        )?;

        let gpu_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("EQUIRECTANGULAR_TO_CUBEMAP_RENDER_PIPELINE"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module_package.vertex_shader_module,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module_package.fragment_shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let face_direction_uv_mappings = [
            FaceDirectionUVMapping::from_uniform(
                FaceDirectionUVMappingUniform::positive_x(),
                "POSITIVE_X",
                &cubemap_face_bind_group_layout,
                &device,
                &queue,
            ),
            FaceDirectionUVMapping::from_uniform(
                FaceDirectionUVMappingUniform::negative_x(),
                "NEGATIVE_X",
                &cubemap_face_bind_group_layout,
                &device,
                &queue,
            ),
            FaceDirectionUVMapping::from_uniform(
                FaceDirectionUVMappingUniform::positive_y(),
                "POSITIVE_Y",
                &cubemap_face_bind_group_layout,
                &device,
                &queue,
            ),
            FaceDirectionUVMapping::from_uniform(
                FaceDirectionUVMappingUniform::negative_y(),
                "NEGATIVE_Y",
                &cubemap_face_bind_group_layout,
                &device,
                &queue,
            ),
            FaceDirectionUVMapping::from_uniform(
                FaceDirectionUVMappingUniform::positive_z(),
                "POSITIVE_Z",
                &cubemap_face_bind_group_layout,
                &device,
                &queue,
            ),
            FaceDirectionUVMapping::from_uniform(
                FaceDirectionUVMappingUniform::negative_z(),
                "NEGATIVE_Z",
                &cubemap_face_bind_group_layout,
                &device,
                &queue,
            ),
        ];

        Ok(Self {
            device,
            queue,
            gpu_pipeline,
            source_bind_group_layout,
            face_direction_uv_mappings,
        })
    }

    pub fn render_cubemap_texture(
        &self,
        name: &str,
        source_image: &image::Rgba32FImage,
    ) -> Result<wgpu::Texture> {
        let image_dimensions = source_image.dimensions();
        let image_size = wgpu::Extent3d {
            width: image_dimensions.0,
            height: image_dimensions.1,
            depth_or_array_layers: 1,
        };

        let image_data = Vec::<half::f16>::from_f32_slice(source_image);

        let gpu_source_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("EQUIRECTANGULAR_TEXTURE"),
            size: image_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &gpu_source_texture,
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

        self.queue.submit([]);

        let gpu_source_texture_view =
            gpu_source_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let gpu_source_texture_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let gpu_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("EQUIRECTANGULAR_TO_CUBEMAP_BIND_GROUP"),
            layout: &self.source_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&gpu_source_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&gpu_source_texture_sampler),
                },
            ],
        });

        let gpu_cubemap_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{name}_TEXTURE")),
            size: wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        for face_index in 0..6 {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("EQUIRECTANGULAR_TO_CUBEMAP_FACE_{face_index}_COMMAND_ENCODER"),
                });

            let texture_view = gpu_cubemap_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("{name}_FACE_{face_index}_TEXTURE_VIEW")),
                format: Some(wgpu::TextureFormat::Rgba16Float),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: face_index,
                array_layer_count: None,
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("EQUIRECTANGULAR_TO_CUBEMAP_FACE_{face_index}_RENDER_PASS"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 1.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                render_pass.set_pipeline(&self.gpu_pipeline);
                render_pass.set_bind_group(0, &gpu_bind_group, &[]);
                render_pass.set_bind_group(
                    1,
                    &self.face_direction_uv_mappings[face_index as usize].gpu_bind_group,
                    &[],
                );
                render_pass.draw(0..3, 0..1);
            }

            self.queue.submit(std::iter::once(encoder.finish()));
        }

        Ok(gpu_cubemap_texture)
    }
}

pub struct FaceDirectionUVMapping {
    pub gpu_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    pub gpu_uniform_buffer: wgpu::Buffer,
}

impl FaceDirectionUVMapping {
    pub fn from_uniform(
        uniform: FaceDirectionUVMappingUniform,
        name: &str,
        bind_group_layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let gpu_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!(
                "EQUIRECTANGULAR_TO_CUBEMAP_FACE_{name}_UNIFORM_BUFFER"
            )),
            size: std::mem::size_of::<FaceDirectionUVMappingUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        queue.write_buffer(&gpu_uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
        queue.submit([]);

        let gpu_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!(
                "EQUIRECTANGULAR_TO_CUBEMAP_FACE_{name}_BIND_GROUP"
            )),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: gpu_uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            gpu_bind_group,
            gpu_uniform_buffer,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FaceDirectionUVMappingUniform {
    pub direction: [f32; 3],
    _padding_1: u32,
    pub u_mapping: [f32; 3],
    _padding_2: u32,
    pub v_mapping: [f32; 3],
    _padding_3: u32,
}

impl FaceDirectionUVMappingUniform {
    pub fn positive_x() -> Self {
        FaceDirectionUVMappingUniform {
            direction: [1.0, 0.0, 0.0],
            _padding_1: 0,
            u_mapping: [0.0, 0.0, 1.0],
            _padding_2: 0,
            v_mapping: [0.0, 1.0, 0.0],
            _padding_3: 0,
        }
    }

    pub fn negative_x() -> Self {
        FaceDirectionUVMappingUniform {
            direction: [-1.0, 0.0, 0.0],
            _padding_1: 0,
            u_mapping: [0.0, 0.0, -1.0],
            _padding_2: 0,
            v_mapping: [0.0, 1.0, 0.0],
            _padding_3: 0,
        }
    }

    pub fn positive_y() -> Self {
        FaceDirectionUVMappingUniform {
            direction: [0.0, -1.0, 0.0],
            _padding_1: 0,
            u_mapping: [-1.0, 0.0, 0.0],
            _padding_2: 0,
            v_mapping: [0.0, 0.0, 1.0],
            _padding_3: 0,
        }
    }

    pub fn negative_y() -> Self {
        FaceDirectionUVMappingUniform {
            direction: [0.0, 1.0, 0.0],
            _padding_1: 0,
            u_mapping: [-1.0, 0.0, 0.0],
            _padding_2: 0,
            v_mapping: [0.0, 0.0, -1.0],
            _padding_3: 0,
        }
    }

    pub fn positive_z() -> Self {
        FaceDirectionUVMappingUniform {
            direction: [0.0, 0.0, 1.0],
            _padding_1: 0,
            u_mapping: [-1.0, 0.0, 0.0],
            _padding_2: 0,
            v_mapping: [0.0, 1.0, 0.0],
            _padding_3: 0,
        }
    }

    pub fn negative_z() -> Self {
        FaceDirectionUVMappingUniform {
            direction: [0.0, 0.0, -1.0],
            _padding_1: 0,
            u_mapping: [1.0, 0.0, 0.0],
            _padding_2: 0,
            v_mapping: [0.0, 1.0, 0.0],
            _padding_3: 0,
        }
    }
}
