use anyhow::Result;

use crate::data::transform::Transform;
use crate::error::Error;
use crate::render::camera::user::UserCamera;
use crate::render::camera::{Camera, CameraInstance, CameraUniform};
use crate::render::ibl::IblEnvironment;
use crate::render::node::RenderNode;
use crate::render::skybox::Skybox;

pub struct ViewEnvironment {
    aspect_ratio: f32,
    active_camera: Option<ViewEnvironmentCamera>,
    user_camera: UserCamera,
    ibl_environment: IblEnvironment,
    gpu_camera_uniform_buffer: wgpu::Buffer,
    view_environment_bind_group_layout: std::rc::Rc<wgpu::BindGroupLayout>,
    gpu_view_environment_bind_group: wgpu::BindGroup,
    device: std::rc::Rc<wgpu::Device>,
    queue: std::rc::Rc<wgpu::Queue>,
}

impl ViewEnvironment {
    pub fn from_device(
        device: std::rc::Rc<wgpu::Device>,
        queue: std::rc::Rc<wgpu::Queue>,
        aspect_ratio: f32,
        user_camera: UserCamera,
        ibl_environment: IblEnvironment,
        view_environment_bind_group_layout: std::rc::Rc<wgpu::BindGroupLayout>,
    ) -> Self {
        let gpu_camera_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("CAMERA_UNIFORM_BUFFER"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let gpu_view_environment_bind_group = ViewEnvironment::create_view_environment_bind_group(
            &device,
            &view_environment_bind_group_layout,
            &gpu_camera_uniform_buffer,
            &ibl_environment,
        );

        let object = Self {
            aspect_ratio,
            active_camera: None,
            user_camera,
            ibl_environment,
            gpu_camera_uniform_buffer,
            view_environment_bind_group_layout,
            gpu_view_environment_bind_group,
            device,
            queue,
        };

        object.update_uniform_buffers();
        object
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.gpu_view_environment_bind_group
    }

    pub fn skybox(&self) -> &Skybox {
        &self.ibl_environment.skybox
    }

    pub fn set_user_camera_transform(&mut self, transform: Transform) {
        let update_ibl_environment_view_projection = self.active_camera.is_none()
            && self.user_camera.transform.rotation != transform.rotation;

        self.user_camera.transform = transform;

        let projection_matrix = cgmath::Matrix4::from(
            self.user_camera
                .camera
                .create_projection_matrix(self.aspect_ratio),
        );

        self.update_camera_view_projection(
            self.user_camera.transform,
            projection_matrix * self.user_camera.create_view_matrix(),
        );

        if update_ibl_environment_view_projection {
            self.ibl_environment
                .skybox
                .update_view_projection(self.user_camera.transform, projection_matrix);
        }
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        if self.aspect_ratio == aspect_ratio {
            return;
        }

        self.aspect_ratio = aspect_ratio;

        if self
            .get_camera_definition()
            .projection
            .aspect_ratio
            .is_some()
        {
            return;
        }

        self.update_uniform_buffers();
    }

    pub fn set_active_camera(&mut self, active_camera: Option<ViewEnvironmentCamera>) {
        match &active_camera {
            Some(active_camera) => match &self.active_camera {
                Some(current_active_camera) if current_active_camera.id() == active_camera.id() => {
                    return
                }
                _ => {}
            },
            None if self.active_camera.is_none() => return,
            _ => {}
        }

        self.active_camera = active_camera;
        self.update_uniform_buffers();
    }

    pub fn set_ibl_environment(&mut self, ibl_environment: IblEnvironment) {
        self.ibl_environment = ibl_environment;
        self.ibl_environment
            .skybox
            .update_view_projection(self.get_camera_transform(), self.get_projection_matrix());
        self.gpu_view_environment_bind_group = self.recreate_view_environment_bind_group();
    }

    fn update_uniform_buffers(&self) {
        let camera_transform = self.get_camera_transform();
        let projection_matrix = self.get_projection_matrix();

        self.update_camera_view_projection(
            camera_transform,
            projection_matrix * self.get_camera_view_matrix(),
        );

        self.ibl_environment
            .skybox
            .update_view_projection(camera_transform, projection_matrix);
    }

    fn update_camera_view_projection(
        &self,
        transform: Transform,
        view_projection_matrix: cgmath::Matrix4<f32>,
    ) {
        self.queue.write_buffer(
            &self.gpu_camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[CameraUniform::new(
                cgmath::Point3 {
                    x: transform.translation.x,
                    y: transform.translation.y,
                    z: transform.translation.z,
                },
                view_projection_matrix,
            )]),
        );
    }

    fn get_camera_definition(&self) -> std::rc::Rc<Camera> {
        match &self.active_camera {
            Some(view_environment_camera) => {
                view_environment_camera.camera_instance().camera.clone()
            }
            None => self.user_camera.camera.clone(),
        }
    }

    fn get_camera_transform(&self) -> Transform {
        match &self.active_camera {
            Some(view_environment_camera) => view_environment_camera
                .camera_instance()
                .global_transform_matrix
                .into(),
            None => self.user_camera.transform,
        }
    }

    fn get_camera_view_matrix(&self) -> cgmath::Matrix4<f32> {
        match &self.active_camera {
            Some(view_environment_camera) => view_environment_camera
                .camera_instance()
                .create_view_matrix(),
            None => self.user_camera.create_view_matrix(),
        }
    }

    fn get_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        self.get_camera_definition()
            .create_projection_matrix(self.aspect_ratio)
            .into()
    }

    fn recreate_view_environment_bind_group(&self) -> wgpu::BindGroup {
        ViewEnvironment::create_view_environment_bind_group(
            &self.device,
            &self.view_environment_bind_group_layout,
            &self.gpu_camera_uniform_buffer,
            &self.ibl_environment,
        )
    }

    fn create_view_environment_bind_group(
        device: &wgpu::Device,
        view_environment_bind_group_layout: &wgpu::BindGroupLayout,
        gpu_camera_uniform_buffer: &wgpu::Buffer,
        ibl_environment: &IblEnvironment,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("VIEW_ENVIRONMENT_BIND_GROUP"),
            layout: view_environment_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: gpu_camera_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &ibl_environment.diffuse_cubemap.gpu_texture_view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(
                        &ibl_environment.diffuse_cubemap.gpu_sampler,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(
                        &ibl_environment.specular_cubemap.gpu_texture_view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(
                        &ibl_environment.specular_cubemap.gpu_sampler,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(
                        &ibl_environment.ggx_lut.gpu_texture_view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(&ibl_environment.ggx_lut.gpu_sampler),
                },
            ],
        })
    }
}

pub struct ViewEnvironmentCamera {
    render_node: std::rc::Rc<RenderNode>,
}

impl ViewEnvironmentCamera {
    pub fn from_render_node(render_node: std::rc::Rc<RenderNode>) -> Result<Self> {
        if render_node.camera.is_none() {
            return Err(Error::new(String::from(
                "The given render node does not have a camera instance",
            ))
            .into());
        }

        Ok(Self { render_node })
    }

    pub fn id(&self) -> usize {
        self.render_node.id
    }

    pub fn camera_instance(&self) -> &CameraInstance {
        self.render_node.camera.as_ref().unwrap()
    }
}
