use anyhow::Result;

use crate::data::transform::Transform;
use crate::error::Error;
use crate::render::buffer::allocator::{VertexBufferAllocator, VertexBufferSegmentDataSource};
use crate::render::buffer::IndexBuffer;
use crate::render::camera::Camera;
use crate::render::image::Image;
use crate::render::material::{Material, MetallicRoughnessUniform};
use crate::render::mesh::{Mesh, MeshInstance};
use crate::render::node::RenderNode;
use crate::render::pipeline::{RenderPipeline, RenderPipelineConfiguration};
use crate::render::primitive::Primitive;
use crate::render::sampler::Sampler;
use crate::render::shader::{ShaderModulePackage, ShaderTemplateConfiguration};
use crate::render::state::RenderSystemState;
use crate::render::storage::RenderSystemSceneStorage;
use crate::render::texture::Texture2DPackage;
use crate::resource::gltf::loader::GltfLoader;

pub struct SceneLoader<'a, T: GltfLoader> {
    state: &'a RenderSystemState,
    storage: &'a mut RenderSystemSceneStorage,
    gltf_loader: &'a mut T,
}

impl<'a, T: GltfLoader> SceneLoader<'a, T> {
    pub fn load(
        state: &'a RenderSystemState,
        storage: &'a mut RenderSystemSceneStorage,
        gltf_loader: &'a mut T,
        scene: &'a gltf::Scene<'a>,
    ) -> Result<()> {
        let mut scene_loader = Self {
            state,
            storage,
            gltf_loader,
        };
        scene_loader.load_scene(scene)?;

        Ok(())
    }

    fn load_scene(&mut self, scene: &'a gltf::Scene<'a>) -> Result<()> {
        log::debug!(
            "Loading glTF scene: {} - [{}]",
            scene.name().unwrap_or("<UNNAMED>"),
            scene.index(),
        );

        for node in scene.nodes() {
            self.load_node(&node, None)?;
        }

        self.state.queue.submit([]);

        Ok(())
    }

    fn load_node(
        &mut self,
        node: &gltf::Node,
        parent_transform_matrix: Option<cgmath::Matrix4<f32>>,
    ) -> Result<std::rc::Rc<RenderNode>> {
        if self.storage.node_registry.contains_key(&node.index()) {
            return Err(Error::new(format!(
                "A node with the given ID has already been registered: {}",
                node.index()
            ))
            .into());
        }

        let node_name_string = match node.name() {
            Some(name) => name.to_string(),
            None => "<UNNAMED>".to_string(),
        };

        log::debug!(
            "Loading glTF node: {} - [{}]",
            node_name_string,
            node.index()
        );

        let (translation, rotation, scale) = node.transform().decomposed();

        let local_transform = Transform {
            translation: translation.into(),
            rotation: rotation.into(),
            scale: scale.into(),
        };
        let local_transform_matrix = cgmath::Matrix4::from(local_transform);

        let global_transform_matrix = match parent_transform_matrix {
            Some(parent_transform_matrix) => parent_transform_matrix * local_transform_matrix,
            None => local_transform_matrix,
        };

        let mut children: Vec<std::rc::Rc<RenderNode>> = vec![];
        for child in node.children() {
            children.push(self.load_node(&child, Some(global_transform_matrix))?);
        }

        let mesh_instance = match node.mesh() {
            Some(mesh) => {
                let mesh_instance_name = format!(
                    "NODE_{node_name_string}_{}_MESH_{}_{}",
                    node.index(),
                    match mesh.name() {
                        Some(name) => name.to_string(),
                        None => "<UNNAMED>".to_string(),
                    },
                    mesh.index(),
                );

                let mesh = self.load_mesh(&mesh)?;

                Some(MeshInstance::from_device(
                    &self.state.device,
                    &self.state.queue,
                    &mesh_instance_name,
                    mesh,
                    global_transform_matrix,
                    &self.state.primitive_instance_bind_group_layout,
                ))
            }
            None => None,
        };

        /*
        // TODO: Add support for loading cameras
        match node.camera() {
            Some(camera) => return self.load_camera(&camera),
            None => {},
        }
        */

        let node = std::rc::Rc::new(RenderNode::new(
            node.index(),
            local_transform,
            children,
            mesh_instance,
            None,
        ));

        self.storage.node_registry.insert(node.id, node.clone());

        Ok(node)
    }

    fn load_mesh(&mut self, mesh: &gltf::Mesh) -> Result<std::rc::Rc<Mesh>> {
        let mesh_log_name = format!(
            "{} - [{}]",
            mesh.name().unwrap_or("<UNNAMED>"),
            mesh.index(),
        );

        if let Some(mesh) = self.storage.mesh_registry.get(&mesh.index()) {
            log::debug!("Skipping duplicate load of glTF mesh: {mesh_log_name}");
            return Ok(mesh.clone());
        }

        log::debug!("Loading glTF mesh: {mesh_log_name}");

        let mut loaded_primitives = Vec::<std::rc::Rc<Primitive>>::new();

        let mesh_name_string = match mesh.name() {
            Some(name) => name.to_string(),
            None => "<UNNAMED>".to_string(),
        };
        let mesh_label_prefix = format!("MESH_{mesh_name_string}_{}", mesh.index());

        for primitive in mesh.primitives() {
            log::debug!(
                "Loading glTF primitve {} for glTF mesh: {mesh_log_name}",
                primitive.index()
            );

            loaded_primitives.push(std::rc::Rc::new(self.load_primitive(
                &primitive,
                format!("{mesh_label_prefix}_PRIMITIVE_{}", primitive.index()),
            )?));
        }

        let loaded_mesh = std::rc::Rc::new(Mesh {
            primitives: loaded_primitives,
        });
        self.storage
            .mesh_registry
            .insert(mesh.index(), loaded_mesh.clone());

        Ok(loaded_mesh)
    }

    fn load_primitive(
        &mut self,
        primitive: &gltf::Primitive,
        label_prefix: String,
    ) -> Result<Primitive> {
        let mut vertex_buffer_allocator =
            VertexBufferAllocator::new(format!("{label_prefix}_VERTEX_BUFFER"));

        let topology = match primitive.mode() {
            gltf::mesh::Mode::Triangles => wgpu::PrimitiveTopology::TriangleList,
            _ => {
                return Err(Error::new(format!(
                    "The given primitive uses an unsupported topology: {:?}",
                    primitive.mode()
                ))
                .into())
            }
        };

        let mut has_position = false;
        let mut has_normal = false;
        let mut has_tangent = false;
        let mut has_tex_coord_0 = false;
        let mut has_tex_coord_1 = false;
        let mut has_color_0 = false;

        for (semantic, _) in primitive.attributes() {
            match semantic {
                gltf::Semantic::Positions => has_position = true,
                gltf::Semantic::Normals => has_normal = true,
                gltf::Semantic::Tangents => has_tangent = true,
                gltf::Semantic::TexCoords(index) => {
                    match index {
                        0 => has_tex_coord_0 = true,
                        1 => has_tex_coord_1 = true,
                        _ => return Err(
                            Error::new(format!("The given primitive has a texture coordinate attribute with an index greater than 1: {index}")).into()
                        ),
                    }
                }
                gltf::Semantic::Colors(index) => {
                    match index {
                        0 => has_color_0 = true,
                        _ => return Err(
                            Error::new(format!("The given primitive has a vertex color attribute with an index greater than 0: {index}")).into()
                        ),
                    }
                }
                _ => {}
            }
        }

        if !has_position {
            return Err(
                Error::new("The given primitive has no position attribute.".to_string()).into(),
            );
        }

        for (semantic, accessor) in primitive.attributes() {
            match semantic {
                gltf::Semantic::Positions
                | gltf::Semantic::Normals
                | gltf::Semantic::Tangents
                | gltf::Semantic::Colors(_)
                | gltf::Semantic::TexCoords(_) => {
                    vertex_buffer_allocator.add_segment(
                        semantic,
                        VertexBufferSegmentDataSource::Accessor {
                            id: accessor.index(),
                            length: (accessor.count() * accessor.size()) as u64,
                        },
                    );
                }
                _ => {}
            }
        }

        let vertex_buffer = vertex_buffer_allocator.finish(
            &self.state.device,
            &self.state.queue,
            self.gltf_loader,
        )?;

        let index_buffer = match primitive.indices() {
            Some(accessor) => {
                let length = accessor.count() * accessor.size();

                let type_ = match accessor.data_type() {
                    gltf::accessor::DataType::U16 => wgpu::IndexFormat::Uint16,
                    gltf::accessor::DataType::U32 => wgpu::IndexFormat::Uint32,
                    _ => return Err(Error::new(format!(
                        "The index buffer for the given primitive uses an invalid data type: {:?}",
                        accessor.data_type()
                    ))
                    .into()),
                };

                let gpu_buffer = self.state.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("{label_prefix}_INDEX_BUFFER")),
                    size: length as u64,
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                let data = self
                    .gltf_loader
                    .load_bytes_from_accessor(accessor.index())?;

                self.state.queue.write_buffer(&gpu_buffer, 0, data);
                self.state.queue.submit([]);

                Some(IndexBuffer { gpu_buffer, type_ })
            }
            None => None,
        };

        let material = self.load_material(&primitive.material())?;

        let count = match primitive.indices() {
            Some(accessor) => accessor.count(),
            None => match primitive.attributes().next() {
                Some((_, accessor)) => accessor.count(),
                None => return Err(Error::new(String::from(
                    "Unable to determine the number of vertices to render for the given primitive.",
                ))
                .into()),
            },
        };

        let render_pipeline_config = RenderPipelineConfiguration {
            has_normal,
            has_tangent,
            has_tex_coord_0,
            has_tex_coord_1,
            has_color_0,
            topology,
        };
        let render_pipeline = self.get_render_pipeline(&render_pipeline_config)?;

        Ok(Primitive {
            vertex_buffer,
            index_buffer,
            material,
            count,
            render_pipeline,
        })
    }

    fn load_material(&mut self, material: &gltf::Material) -> Result<std::rc::Rc<Material>> {
        let material_log_name = format!(
            "{} - [{}]",
            material.name().unwrap_or("<UNNAMED>"),
            match material.index() {
                Some(index) => index.to_string(),
                None => String::from("<DEFAULT>"),
            },
        );

        if let Some(material) = self.storage.material_registry.get(&material.index()) {
            log::debug!("Skipping duplicate load of glTF material: {material_log_name}");
            return Ok(material.clone());
        }

        log::debug!("Loading glTF material: {material_log_name}");

        let material_name_string = match material.name() {
            Some(name) => name.to_string(),
            None => "<UNNAMED>".to_string(),
        };
        let material_label = format!(
            "MATERIAL_{material_name_string}_{}",
            match material.index() {
                Some(index) => index.to_string(),
                None => String::from("<DEFAULT>"),
            },
        );

        let gpu_metallic_roughness_uniform_buffer =
            self.state.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("{material_label}_METALLIC_ROUGHNESS_UNIFORM_BUFFER"),
                size: std::mem::size_of::<MetallicRoughnessUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

        let base_color_texture = match material.pbr_metallic_roughness().base_color_texture() {
            Some(texture_info) => {
                self.load_texture(&texture_info, wgpu::TextureFormat::Rgba8UnormSrgb)?
            }
            None => self.load_default_texture(),
        };

        let metallic_roughness_texture = match material
            .pbr_metallic_roughness()
            .metallic_roughness_texture()
        {
            Some(texture_info) => {
                self.load_texture(&texture_info, wgpu::TextureFormat::Rgba8Unorm)?
            }
            None => self.load_default_texture(),
        };

        let gpu_bind_group = self
            .state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("{material_label}_BIND_GROUP")),
                layout: &self.state.material_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: gpu_metallic_roughness_uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &base_color_texture.gpu_texture_view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(
                            &base_color_texture.sampler.gpu_sampler,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(
                            &metallic_roughness_texture.gpu_texture_view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(
                            &metallic_roughness_texture.sampler.gpu_sampler,
                        ),
                    },
                ],
            });

        let loaded_material = std::rc::Rc::new(Material::new(
            material.pbr_metallic_roughness().base_color_factor(),
            base_color_texture,
            material.pbr_metallic_roughness().metallic_factor(),
            material.pbr_metallic_roughness().roughness_factor(),
            metallic_roughness_texture,
            gpu_metallic_roughness_uniform_buffer,
            gpu_bind_group,
            &self.state.queue,
        ));

        self.storage
            .material_registry
            .insert(material.index(), loaded_material.clone());

        Ok(loaded_material)
    }

    fn load_texture(
        &mut self,
        texture_info: &gltf::texture::Info,
        format: wgpu::TextureFormat,
    ) -> Result<std::rc::Rc<Texture2DPackage>> {
        let texture = texture_info.texture();

        let texture_log_name = format!(
            "{} - [{}]",
            texture.name().unwrap_or("<UNNAMED>"),
            texture.index(),
        );

        if let Some(texture) = self.storage.texture_registry.get(&texture.index()) {
            log::debug!("Skipping duplicate load of glTF texture: {texture_log_name}");
            return Ok(texture.clone());
        }

        log::debug!("Loading glTF texture: {texture_log_name}");

        let loaded_image = self.load_image(&texture.source())?;
        let image_dimensions = loaded_image.dimensions();
        let image_size = wgpu::Extent3d {
            width: image_dimensions.0,
            height: image_dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture_name_string = match texture.name() {
            Some(name) => name.to_string(),
            None => "<UNNAMED>".to_string(),
        };
        let texture_label = format!("TEXTURE_{texture_name_string}_{}", texture.index());

        let gpu_texture = self.state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&texture_label),
            size: image_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.state.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &gpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            loaded_image.data(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * image_dimensions.0),
                rows_per_image: Some(image_dimensions.1),
            },
            image_size,
        );

        let gpu_texture_view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.load_sampler(&texture.sampler())?;

        let loaded_texture = std::rc::Rc::new(Texture2DPackage {
            gpu_texture,
            gpu_texture_view,
            sampler,
        });

        self.storage
            .texture_registry
            .insert(texture.index(), loaded_texture.clone());

        Ok(loaded_texture)
    }

    fn load_default_texture(&mut self) -> std::rc::Rc<Texture2DPackage> {
        if let Some(default_texture) = &self.storage.default_texture {
            log::debug!("Skipping duplicate load of default glTF texture.");
            return default_texture.clone();
        }

        log::debug!("Loading default glTF texture.");

        let image_data: [u8; 4] = [255, 255, 255, 255];
        let image_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let texture_label = String::from("DEFAULT_TEXTURE");

        let gpu_texture = self.state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&texture_label),
            size: image_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.state.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &gpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&image_data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            image_size,
        );

        let gpu_texture_view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = std::rc::Rc::new(Sampler {
            gpu_sampler: self.state.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }),
        });

        let default_texture = std::rc::Rc::new(Texture2DPackage {
            gpu_texture,
            gpu_texture_view,
            sampler,
        });
        self.storage.default_texture = Some(default_texture.clone());

        default_texture
    }

    fn load_image(&mut self, image: &gltf::Image) -> Result<std::rc::Rc<Image>> {
        let image_log_name = format!(
            "{} - [{}]",
            image.name().unwrap_or("<UNNAMED>"),
            image.index(),
        );

        if let Some(texture) = self.storage.image_registry.get(&image.index()) {
            log::debug!("Skipping duplicate load of glTF image: {image_log_name}");
            return Ok(texture.clone());
        }

        log::debug!("Loading glTF image: {image_log_name}");

        let loaded_image = std::rc::Rc::new(Image::from_rgba_image(
            self.gltf_loader.load_image(image.index())?,
        ));

        self.storage
            .image_registry
            .insert(image.index(), loaded_image.clone());

        Ok(loaded_image)
    }

    fn load_sampler(&mut self, sampler: &gltf::texture::Sampler) -> Result<std::rc::Rc<Sampler>> {
        let sampler_log_name = format!(
            "{} - [{}]",
            sampler.name().unwrap_or("<UNNAMED>"),
            match sampler.index() {
                Some(index) => index.to_string(),
                None => String::from("<DEFAULT>"),
            },
        );

        if let Some(sampler) = self.storage.sampler_registry.get(&sampler.index()) {
            log::debug!("Skipping duplicate load of glTF sampler: {sampler_log_name}");
            return Ok(sampler.clone());
        }

        log::debug!("Loading glTF sampler: {sampler_log_name}");

        let address_mode_u = match sampler.wrap_s() {
            gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::Repeat,
            gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
        };

        let address_mode_v = match sampler.wrap_t() {
            gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::Repeat,
            gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
        };

        let mag_filter = match sampler.mag_filter() {
            Some(mag_filter) => match mag_filter {
                gltf::texture::MagFilter::Linear => wgpu::FilterMode::Linear,
                gltf::texture::MagFilter::Nearest => wgpu::FilterMode::Nearest,
            },
            None => wgpu::FilterMode::Linear,
        };

        let min_filter = match sampler.min_filter() {
            Some(min_filter) => match min_filter {
                gltf::texture::MinFilter::Linear
                | gltf::texture::MinFilter::LinearMipmapLinear
                | gltf::texture::MinFilter::LinearMipmapNearest => wgpu::FilterMode::Linear,
                _ => wgpu::FilterMode::Nearest,
            },
            None => wgpu::FilterMode::Linear,
        };

        let mipmap_filter = match sampler.min_filter() {
            Some(min_filter) => match min_filter {
                gltf::texture::MinFilter::Linear
                | gltf::texture::MinFilter::LinearMipmapLinear
                | gltf::texture::MinFilter::NearestMipmapLinear => wgpu::FilterMode::Linear,
                _ => wgpu::FilterMode::Nearest,
            },
            None => wgpu::FilterMode::Linear,
        };

        let loaded_sampler = std::rc::Rc::new(Sampler {
            gpu_sampler: self.state.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u,
                address_mode_v,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter,
                min_filter,
                mipmap_filter,
                ..Default::default()
            }),
        });

        self.storage
            .sampler_registry
            .insert(sampler.index(), loaded_sampler.clone());

        Ok(loaded_sampler)
    }

    fn load_camera(&mut self, camera: &gltf::Camera) -> Result<std::rc::Rc<Camera>> {
        log::debug!(
            "Loading glTF camera: {} - [{}]",
            camera.name().unwrap_or("<UNNAMED>"),
            camera.index()
        );
        todo!("Add support for loading glTF cameras.");
    }

    fn get_render_pipeline(
        &mut self,
        render_pipeline_config: &RenderPipelineConfiguration,
    ) -> Result<std::rc::Rc<RenderPipeline>> {
        if let Some(render_pipeline) = self
            .storage
            .render_pipeline_registry
            .get(render_pipeline_config)
        {
            return Ok(render_pipeline.clone());
        }

        let shader_template_config =
            ShaderTemplateConfiguration::from_render_pipeline_config(render_pipeline_config);

        let shader_module_package = self.get_shader_module_package(&shader_template_config)?;

        log::debug!(
            "Creating render pipeline for config: {:?}",
            render_pipeline_config
        );

        let render_pipeline = std::rc::Rc::new(RenderPipeline::from_config(
            *render_pipeline_config,
            format!(
                "RENDER_PIPELINE_{}",
                self.storage.render_pipeline_registry.len()
            ),
            &self.state.device,
            &[
                &self.state.view_environment_bind_group_layout,
                &self.state.primitive_instance_bind_group_layout,
                &self.state.material_bind_group_layout,
            ],
            &shader_module_package.vertex_shader_module,
            &shader_module_package.fragment_shader_module,
            self.state.surface_config.format,
        ));
        self.storage
            .render_pipeline_registry
            .insert(*render_pipeline_config, render_pipeline.clone());

        Ok(render_pipeline)
    }

    fn get_shader_module_package(
        &mut self,
        shader_template_config: &ShaderTemplateConfiguration,
    ) -> Result<std::rc::Rc<ShaderModulePackage>> {
        let module_name_prefix = format!(
            "SHADER_MODULE_PACKAGE_{}",
            self.storage.shader_module_package_registry.len()
        );

        if let Some(shader_module_package) = self
            .storage
            .shader_module_package_registry
            .get(shader_template_config)
        {
            return Ok(shader_module_package.clone());
        }

        let shader_module_package = std::rc::Rc::new(ShaderModulePackage::from_templates(
            "primitive/primitive.vert",
            "primitive/primitive.frag",
            &module_name_prefix,
            &self.state.device,
            &self.state.tera,
            Some(shader_template_config),
        )?);

        self.storage
            .shader_module_package_registry
            .insert(*shader_template_config, shader_module_package.clone());

        Ok(shader_module_package)
    }
}
