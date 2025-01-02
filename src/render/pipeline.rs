pub struct RenderPipeline {
    pub config: RenderPipelineConfiguration,
    pub gpu_pipeline: wgpu::RenderPipeline,
}

impl RenderPipeline {
    pub fn from_config(
        config: RenderPipelineConfiguration,
        name: String,
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        vertex_shader_module: &wgpu::ShaderModule,
        fragment_shader_module: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
    ) -> Self {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{name}_RENDER_PIPELINE_LAYOUT")),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        let vertex_buffer_layout_builder =
            RenderPipeline::create_vertex_buffer_layout_builder(&config);

        let gpu_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{name}_RENDER_PIPELINE")),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: vertex_shader_module,
                entry_point: "vs_main",
                buffers: &vertex_buffer_layout_builder.build(),
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: fragment_shader_module,
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
                topology: config.topology,
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
                depth_compare: wgpu::CompareFunction::Less,
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

        Self {
            config,
            gpu_pipeline,
        }
    }

    fn create_vertex_buffer_layout_builder(
        config: &RenderPipelineConfiguration,
    ) -> VertexBufferLayoutBuilder {
        let mut builder: VertexBufferLayoutBuilder = Default::default();

        builder.add(VertexBufferLayoutBuilderEntry {
            array_stride: (3 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: vec![wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        });

        if config.has_normal {
            builder.add(VertexBufferLayoutBuilderEntry {
                array_stride: (3 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: vec![wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                }],
            });

            if config.has_tangent {
                builder.add(VertexBufferLayoutBuilderEntry {
                    array_stride: (4 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: vec![wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 2,
                        format: wgpu::VertexFormat::Float32x4,
                    }],
                });
            }
        }

        if config.has_tex_coord_0 {
            builder.add(VertexBufferLayoutBuilderEntry {
                array_stride: (2 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: vec![wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: config.get_tex_coord_0_location(),
                    format: wgpu::VertexFormat::Float32x2,
                }],
            });
        }

        if config.has_tex_coord_1 {
            builder.add(VertexBufferLayoutBuilderEntry {
                array_stride: (2 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: vec![wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: config.get_tex_coord_1_location(),
                    format: wgpu::VertexFormat::Float32x2,
                }],
            });
        }

        if config.has_color_0 {
            builder.add(VertexBufferLayoutBuilderEntry {
                array_stride: (4 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: vec![wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: config.get_color_0_location(),
                    format: wgpu::VertexFormat::Float32x4,
                }],
            });
        }

        builder
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RenderPipelineConfiguration {
    pub has_normal: bool,
    pub has_tangent: bool,
    pub has_tex_coord_0: bool,
    pub has_tex_coord_1: bool,
    pub has_color_0: bool,
    pub topology: wgpu::PrimitiveTopology,
}

impl RenderPipelineConfiguration {
    pub fn get_tex_coord_0_location(&self) -> u32 {
        match self.has_tex_coord_0 {
            true => self.get_base_location_offset(),
            false => 0,
        }
    }

    pub fn get_tex_coord_1_location(&self) -> u32 {
        match self.has_tex_coord_1 {
            true => self.get_base_location_offset() + 1,
            false => 0,
        }
    }

    pub fn get_color_0_location(&self) -> u32 {
        if !self.has_color_0 {
            return 0;
        }

        let base_offset = self.get_base_location_offset();

        if self.has_tex_coord_1 {
            base_offset + 2
        } else if self.has_tex_coord_0 {
            base_offset + 1
        } else {
            base_offset
        }
    }

    fn get_base_location_offset(&self) -> u32 {
        if self.has_tangent {
            3
        } else if self.has_normal {
            2
        } else {
            1
        }
    }
}

#[derive(Default)]
struct VertexBufferLayoutBuilder {
    entries: Vec<VertexBufferLayoutBuilderEntry>,
}

impl VertexBufferLayoutBuilder {
    pub fn add(&mut self, entry: VertexBufferLayoutBuilderEntry) {
        self.entries.push(entry);
    }

    pub fn build(&self) -> Vec<wgpu::VertexBufferLayout> {
        self.entries
            .iter()
            .map(|entry| wgpu::VertexBufferLayout {
                array_stride: entry.array_stride,
                step_mode: entry.step_mode,
                attributes: &entry.attributes,
            })
            .collect()
    }
}

struct VertexBufferLayoutBuilderEntry {
    array_stride: wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode,
    attributes: Vec<wgpu::VertexAttribute>,
}
