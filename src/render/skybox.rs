use anyhow::Result;
use cgmath::{Vector3, Zero};

use crate::data::transform::Transform;
use crate::render::camera::Camera;
use crate::render::cubemap::CubeMap;
use crate::render::shader::ShaderModulePackage;

pub struct SkyboxRenderer {
    device: std::rc::Rc<wgpu::Device>,
    queue: std::rc::Rc<wgpu::Queue>,
    gpu_pipeline: wgpu::RenderPipeline,
    gpu_vertex_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl SkyboxRenderer {
    pub fn from_device(
        device: std::rc::Rc<wgpu::Device>,
        queue: std::rc::Rc<wgpu::Queue>,
        format: wgpu::TextureFormat,
        tera: &tera::Tera,
    ) -> Result<Self> {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SKYBOX_BIND_GROUP_LAYOUT"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("SKYBOX_RENDER_PIPELINE_LAYOUT"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader_module_package = ShaderModulePackage::from_templates(
            "skybox/skybox.vert",
            "skybox/skybox.frag",
            "SKYBOX",
            &device,
            tera,
            None,
        )?;

        let gpu_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SKYBOX_RENDER_PIPELINE"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module_package.vertex_shader_module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: (3 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    }],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module_package.fragment_shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let skybox_vertices: &[f32] = &[
            -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0,
            -1.0, 1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0,
            -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            1.0, 1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0,
            1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0,
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0,
            -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
        ];

        let gpu_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SKYBOX_VERTEX_BUFFER"),
            size: std::mem::size_of_val(skybox_vertices) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        queue.write_buffer(&gpu_vertex_buffer, 0, bytemuck::cast_slice(skybox_vertices));
        queue.submit([]);

        Ok(Self {
            device,
            queue,
            gpu_pipeline,
            gpu_vertex_buffer,
            bind_group_layout,
        })
    }

    pub fn create_skybox_from_texture(
        &self,
        gpu_texture: wgpu::Texture,
        name: &str,
    ) -> Result<Skybox> {
        let cubemap = CubeMap::from_texture(gpu_texture, name, &self.device)?;

        Ok(Skybox::from_device(
            &self.device,
            self.queue.clone(),
            name,
            cubemap,
            &self.bind_group_layout,
        ))
    }

    pub fn create_default_skybox(&self, name: &str) -> Result<Skybox> {
        let cubemap = CubeMap::create_default_cubemap(name, &self.device, &self.queue)?;

        Ok(Skybox::from_device(
            &self.device,
            self.queue.clone(),
            name,
            cubemap,
            &self.bind_group_layout,
        ))
    }

    pub fn render_skybox(&self, skybox: &Skybox, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.gpu_pipeline);
        render_pass.set_vertex_buffer(0, self.gpu_vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &skybox.gpu_bind_group, &[]);
        render_pass.draw(0..36, 0..1);
    }
}

pub struct Skybox {
    #[allow(dead_code)]
    pub cubemap: CubeMap,
    pub gpu_view_projection_uniform_buffer: wgpu::Buffer,
    pub gpu_bind_group: wgpu::BindGroup,
    queue: std::rc::Rc<wgpu::Queue>,
}

impl Skybox {
    fn from_device(
        device: &wgpu::Device,
        queue: std::rc::Rc<wgpu::Queue>,
        name: &str,
        cubemap: CubeMap,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let gpu_view_projection_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{name}_VIEW_PROJECTION_UNIFORM_BUFFER")),
            size: (4 * 4 * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let gpu_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{name}_BIND_GROUP")),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: gpu_view_projection_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cubemap.gpu_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&cubemap.gpu_sampler),
                },
            ],
        });

        Self {
            cubemap,
            gpu_view_projection_uniform_buffer,
            gpu_bind_group,
            queue,
        }
    }

    pub fn update_view_projection(
        &self,
        transform: Transform,
        projection_matrix: cgmath::Matrix4<f32>,
    ) {
        let transform = Transform {
            translation: Vector3::zero(),
            rotation: transform.rotation,
            scale: cgmath::Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        };

        let view_matrix = Camera::create_view_matrix_from_transform(transform);
        let view_projection_data: [[f32; 4]; 4] = (projection_matrix * view_matrix).into();

        self.queue.write_buffer(
            &self.gpu_view_projection_uniform_buffer,
            0,
            bytemuck::cast_slice(&[view_projection_data]),
        );
    }
}
