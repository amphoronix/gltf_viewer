use anyhow::Result;

use crate::error::Error;
use crate::render::camera::projection::PerspectiveCameraProjection;
use crate::render::camera::user::UserCamera;
use crate::render::camera::Camera;
use crate::render::cubemap::CubeMap;
use crate::render::equirectangular::EquirectangularToCubeMapRenderer;
use crate::render::ibl::IblEnvironment;
use crate::render::lut::GgxLut;
use crate::render::skybox::SkyboxRenderer;
use crate::render::texture::DepthTexture2DPackage;
use crate::render::view::ViewEnvironment;

pub struct RenderSystemState {
    #[allow(dead_code)]
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    #[allow(dead_code)]
    pub adapter: wgpu::Adapter,
    pub device: std::rc::Rc<wgpu::Device>,
    pub queue: std::rc::Rc<wgpu::Queue>,
    pub view_environment_bind_group_layout: std::rc::Rc<wgpu::BindGroupLayout>,
    pub primitive_instance_bind_group_layout: wgpu::BindGroupLayout,
    pub material_bind_group_layout: wgpu::BindGroupLayout,
    pub depth_texture: DepthTexture2DPackage,
    pub tera: tera::Tera,
    pub equirectangular_to_cubemap_renderer: EquirectangularToCubeMapRenderer,
    pub skybox_renderer: SkyboxRenderer,
    pub view_environment: ViewEnvironment,
    pub view_dimensions: winit::dpi::PhysicalSize<u32>,
}

impl RenderSystemState {
    pub async fn from_window(window: std::sync::Arc<winit::window::Window>) -> Result<Self> {
        let view_dimensions = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
        {
            Some(adapter) => adapter,
            None => return Err(Error::new(String::from("Failed to retrieve adapter.")).into()),
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                    ..Default::default()
                },
                None,
            )
            .await?;

        let device = std::rc::Rc::new(device);
        let queue = std::rc::Rc::new(queue);

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: view_dimensions.width,
            height: view_dimensions.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let view_environment_bind_group_layout = std::rc::Rc::new(device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("VIEW_ENVIRONMENT_BIND_GROUP_LAYOUT"),
                entries: &[
                    // Camera Uniform
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // IBL Diffuse Texture
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
                    // IBL Diffuse Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // IBL Specular Texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // IBL Specular Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // IBL GGX LUT
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // IBL GGX LUT Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            },
        ));

        let primitive_instance_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("PRIMITIVE_BIND_GROUP_LAYOUT"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let material_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("MATERIAL_BIND_GROUP_LAYOUT"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
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
                            view_dimension: wgpu::TextureViewDimension::D2,
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let depth_texture = RenderSystemState::create_depth_texture(
            &device,
            wgpu::Extent3d {
                width: view_dimensions.width.max(1),
                height: view_dimensions.height.max(1),
                depth_or_array_layers: 1,
            },
        );

        let tera = tera::Tera::new("shaders/**/*")?;

        let equirectangular_to_cubemap_renderer =
            EquirectangularToCubeMapRenderer::from_device(device.clone(), queue.clone(), &tera)?;

        let skybox_renderer =
            SkyboxRenderer::from_device(device.clone(), queue.clone(), surface_format, &tera)?;

        let skybox = skybox_renderer.create_default_skybox("IBL_ENVIRONMENT_SKYBOX_CUBEMAP")?;

        let ggx_lut_image = image::open(GgxLut::default_path())?.to_rgba32f();

        let ggx_lut =
            GgxLut::from_image(&ggx_lut_image, "IBL_ENVIRONMENT_GGX_LUT", &device, &queue);

        let ibl_environment = IblEnvironment {
            skybox,
            diffuse_cubemap: CubeMap::create_default_cubemap(
                "IBL_ENVIRONMENT_DIFFUSE_CUBEMAP",
                &device,
                &queue,
            )?,
            specular_cubemap: CubeMap::create_default_cubemap(
                "IBL_ENVIRONMENT_SPECULAR_CUBEMAP",
                &device,
                &queue,
            )?,
            ggx_lut,
        };

        let user_camera = UserCamera {
            camera: std::rc::Rc::new(Camera {
                projection: PerspectiveCameraProjection {
                    aspect_ratio: None,
                    fovy: cgmath::Deg(45.0).into(),
                    znear: 0.1,
                    zfar: 100.0,
                },
            }),
            transform: Default::default(),
        };

        let view_environment = ViewEnvironment::from_device(
            device.clone(),
            queue.clone(),
            surface_config.width as f32 / surface_config.height as f32,
            user_camera,
            ibl_environment,
            view_environment_bind_group_layout.clone(),
        );

        Ok(Self {
            instance,
            surface,
            surface_config,
            adapter,
            device,
            queue,
            view_environment_bind_group_layout,
            primitive_instance_bind_group_layout,
            material_bind_group_layout,
            depth_texture,
            tera,
            equirectangular_to_cubemap_renderer,
            skybox_renderer,
            view_environment,
            view_dimensions,
        })
    }

    pub fn set_view_dimensions(&mut self, view_dimensions: winit::dpi::PhysicalSize<u32>) {
        self.view_dimensions = view_dimensions;
        self.surface_config.width = view_dimensions.width;
        self.surface_config.height = view_dimensions.height;
        self.surface.configure(&self.device, &self.surface_config);
        self.depth_texture = RenderSystemState::create_depth_texture(
            &self.device,
            wgpu::Extent3d {
                width: view_dimensions.width.max(1),
                height: view_dimensions.height.max(1),
                depth_or_array_layers: 1,
            },
        );
        self.view_environment
            .set_aspect_ratio(view_dimensions.width as f32 / view_dimensions.height as f32);
    }

    fn create_depth_texture(device: &wgpu::Device, size: wgpu::Extent3d) -> DepthTexture2DPackage {
        let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("DEPTH_TEXTURE"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let gpu_texture_view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

        DepthTexture2DPackage {
            gpu_texture,
            gpu_texture_view,
        }
    }
}
