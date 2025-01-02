use anyhow::Result;

use crate::data::transform::Transform;
use crate::error::Error;
use crate::render::cubemap::CubeMap;
use crate::render::ibl::IblEnvironment;
use crate::render::lut::GgxLut;
use crate::render::primitive::Primitive;
use crate::render::scene::SceneLoader;
use crate::render::state::RenderSystemState;
use crate::render::storage::RenderSystemSceneStorage;
use crate::resource::gltf::asset::GltfAsset;
use crate::resource::gltf::loader::GltfLoader;
use crate::resource::ibl::IblEnvironmentLoader;

mod buffer;
mod camera;
mod cubemap;
mod equirectangular;
mod ibl;
mod image;
mod lut;
mod material;
mod mesh;
mod node;
mod pipeline;
mod primitive;
mod sampler;
mod scene;
mod shader;
mod skybox;
mod state;
mod storage;
mod texture;
mod view;

pub struct RenderSystem {
    state: RenderSystemState,
    storage: RenderSystemSceneStorage,
}

impl RenderSystem {
    pub async fn from_window(window: std::sync::Arc<winit::window::Window>) -> Result<Self> {
        let state = RenderSystemState::from_window(window).await?;

        Ok(Self {
            state,
            storage: Default::default(),
        })
    }

    pub fn sync_view_dimensions(&mut self) {
        self.set_view_dimensions(self.state.view_dimensions);
    }

    pub fn set_view_dimensions(&mut self, view_dimensions: winit::dpi::PhysicalSize<u32>) {
        if view_dimensions.width == 0 || view_dimensions.height == 0 {
            return;
        }

        self.state.set_view_dimensions(view_dimensions);
    }

    pub fn set_user_camera_transform(&mut self, transform: Transform) {
        self.state
            .view_environment
            .set_user_camera_transform(transform);
    }

    pub fn render(&mut self) -> Result<()> {
        let output = self.state.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("RENDER_SYSTEM_COMMAND_ENCODER"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("RENDER_SYSTEM_RENDER_PASS"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.state.depth_texture.gpu_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            for node in self.storage.node_registry.values() {
                let mesh_instance = match &node.mesh {
                    Some(mesh_instance) => mesh_instance,
                    None => continue,
                };

                for primitive in mesh_instance.mesh.primitives.iter() {
                    self.render_primitive(
                        primitive,
                        &mesh_instance.gpu_transform_bind_group,
                        &mut render_pass,
                    )?;
                }
            }

            self.state
                .skybox_renderer
                .render_skybox(self.state.view_environment.skybox(), &mut render_pass);
        }

        self.state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn render_primitive(
        &self,
        primitive: &Primitive,
        gpu_transform_bind_group: &wgpu::BindGroup,
        render_pass: &mut wgpu::RenderPass,
    ) -> Result<()> {
        render_pass.set_pipeline(&primitive.render_pipeline.gpu_pipeline);

        for buffer_segment in primitive.vertex_buffer.segments.iter() {
            let location = match buffer_segment.type_ {
                gltf::Semantic::Positions => 0,
                gltf::Semantic::Normals => 1,
                gltf::Semantic::Tangents => 2,
                gltf::Semantic::TexCoords(index) => {
                    match index {
                        0 => primitive.render_pipeline.config.get_tex_coord_0_location(),
                        1 => primitive.render_pipeline.config.get_tex_coord_1_location(),
                        _ => return Err(
                            Error::new(format!("The given primitive has a texture coordinate attribute with an index greater than 1: {index}")).into()
                        ),
                    }
                }
                gltf::Semantic::Colors(index) => {
                    match index {
                        0 => primitive.render_pipeline.config.get_color_0_location(),
                        _ => return Err(
                            Error::new(format!("The given primitive has a vertex color attribute with an index greater than 0: {index}")).into()
                        ),
                    }
                }
                _ => {
                    log::info!("Ignoring unsupported vertex attribute type: {:?}", buffer_segment.type_);
                    continue;
                }
            };

            let begin = buffer_segment.offset as u64;
            let end = (buffer_segment.offset + buffer_segment.length) as u64;

            render_pass.set_vertex_buffer(
                location,
                primitive.vertex_buffer.gpu_buffer.slice(begin..end),
            );
        }

        render_pass.set_bind_group(0, self.state.view_environment.bind_group(), &[]);
        render_pass.set_bind_group(1, gpu_transform_bind_group, &[]);
        render_pass.set_bind_group(2, &primitive.material.gpu_bind_group, &[]);

        match &primitive.index_buffer {
            Some(index_buffer) => {
                render_pass.set_index_buffer(index_buffer.gpu_buffer.slice(..), index_buffer.type_);
                render_pass.draw_indexed(0..(primitive.count as u32), 0, 0..1);
            }
            None => {
                render_pass.draw(0..(primitive.count as u32), 0..1);
            }
        }

        Ok(())
    }

    pub fn load_scene<T: GltfLoader>(
        &mut self,
        asset: &impl GltfAsset,
        scene_id: usize,
        gltf_loader: &mut T,
    ) -> Result<()> {
        let scene = asset.get_scene(scene_id)?;
        self.storage = Default::default();

        match SceneLoader::load(&self.state, &mut self.storage, gltf_loader, &scene) {
            Ok(_) => {}
            Err(error) => {
                self.storage = Default::default();
                return Err(error);
            }
        }

        Ok(())
    }

    pub fn load_ibl_environment(
        &mut self,
        ibl_environment_loader: &impl IblEnvironmentLoader,
    ) -> Result<()> {
        let equirectangular_skybox_image = ibl_environment_loader.load_equirectangular_skybox()?;

        let skybox_texture = self
            .state
            .equirectangular_to_cubemap_renderer
            .render_cubemap_texture(
                "IBL_ENVIRONMENT_SKYBOX_CUBEMAP",
                &equirectangular_skybox_image,
            )?;
        let skybox = self
            .state
            .skybox_renderer
            .create_skybox_from_texture(skybox_texture, "IBL_ENVIRONMENT_SKYBOX_CUBEMAP")?;

        let diffuse_cubemap = CubeMap::from_loader(
            &ibl_environment_loader.get_diffuse_cubemap_loader()?,
            "IBL_ENVIRONMENT_DIFFUSE_CUBEMAP",
            &self.state.device,
            &self.state.queue,
        )?;

        let specular_cubemap = CubeMap::from_loader(
            &ibl_environment_loader.get_specular_cubemap_loader()?,
            "IBL_ENVIRONMENT_SPECULAR_CUBEMAP",
            &self.state.device,
            &self.state.queue,
        )?;

        let ggx_lut = GgxLut::from_image(
            &ibl_environment_loader.load_ggx_lut(&GgxLut::default_path())?,
            "IBL_ENVIRONMENT_GGX_LUT",
            &self.state.device,
            &self.state.queue,
        );

        self.state
            .view_environment
            .set_ibl_environment(IblEnvironment {
                skybox,
                diffuse_cubemap,
                specular_cubemap,
                ggx_lut,
            });

        Ok(())
    }
}
