use std::path::Path;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::error::EventLoopError;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::args::Args;
use crate::resource::gltf::asset::file::FileSystemGltfAsset;
use crate::resource::gltf::asset::GltfAsset;
use crate::resource::gltf::loader::file::FileSystemGltfLoader;
use crate::resource::ibl::file::FileSystemIblEnvironmentLoader;
use crate::view::ViewSystem;

pub struct App {
    event_loop_proxy: EventLoopProxy<UserEvent>,
    args: Args,
    view_system: Option<ViewSystem>,
    last_render_time: std::time::Instant,
}

impl App {
    pub fn new(event_loop: &EventLoop<UserEvent>, args: Args) -> Self {
        App {
            event_loop_proxy: event_loop.create_proxy(),
            args,
            view_system: None,
            last_render_time: Instant::now(),
        }
    }

    pub fn create_event_loop() -> Result<EventLoop<UserEvent>, EventLoopError> {
        EventLoop::<UserEvent>::with_user_event().build()
    }

    fn create_window(event_loop: &ActiveEventLoop) -> Window {
        cfg_if::cfg_if! {
            if #[cfg(target_arch="wasm32")] {
                todo!()
            } else {
                event_loop.create_window(
                    Window::default_attributes(),
                ).unwrap()
            }
        }
    }

    async fn initialize_view_system(event_loop_proxy: EventLoopProxy<UserEvent>, window: Window) {
        let view_system = ViewSystem::from_window(window).await.unwrap();
        assert!(event_loop_proxy
            .send_event(UserEvent::ViewSystemReady(view_system))
            .is_ok());
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("Application resumed");

        let window = App::create_window(event_loop);
        let event_loop_proxy = self.event_loop_proxy.clone();
        let future = async move {
            App::initialize_view_system(event_loop_proxy, window).await;
        };

        cfg_if::cfg_if! {
            if #[cfg(target_arch="wasm32")] {
                wasm_bindgen_futures::spawn_local(future);
            } else {
                pollster::block_on(future);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let view_system = match &mut self.view_system {
            Some(view_system) => view_system,
            None => return,
        };

        if view_system.window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                view_system.render_system.set_view_dimensions(new_size)
            }
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let delta_time = now - self.last_render_time;

                match view_system.update_view(delta_time) {
                    Ok(_) => {}
                    Err(error) => {
                        if let Some(error) = error.downcast_ref::<wgpu::SurfaceError>() {
                            match error {
                                wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
                                    view_system.render_system.sync_view_dimensions()
                                }
                                wgpu::SurfaceError::OutOfMemory => {
                                    log::error!("OutOfMemory");
                                    event_loop.exit();
                                }
                                wgpu::SurfaceError::Timeout => {
                                    log::warn!("Surface timeout");
                                }
                            }
                        }
                    }
                }

                self.last_render_time = now;
                view_system.window.request_redraw();
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                view_system
                    .camera_controller
                    .handle_mouse_input(button, state);
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let view_system = match &mut self.view_system {
            Some(view_system) => view_system,
            None => return,
        };

        if let DeviceEvent::MouseMotion { delta } = event {
            view_system
                .camera_controller
                .handle_mouse_movement(delta.0 as f32, delta.1 as f32);
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        let UserEvent::ViewSystemReady(mut view_system) = event;

        log::info!("View system created");

        if let Some(ibl_environment) = &self.args.ibl_environment {
            let ibl_environment_loader = FileSystemIblEnvironmentLoader {
                paths: ibl_environment.clone(),
            };

            view_system
                .render_system
                .load_ibl_environment(&ibl_environment_loader)
                .unwrap();
        }

        if let Some(gltf_file_path) = &self.args.gltf {
            let asset = FileSystemGltfAsset::from_path(Path::new(gltf_file_path)).unwrap();
            let default_scene = asset.gltf().default_scene().unwrap();

            let mut gltf_loader = FileSystemGltfLoader::new(&asset);

            view_system
                .render_system
                .load_scene(&asset, default_scene.index(), &mut gltf_loader)
                .unwrap();
        }

        view_system.window.request_redraw();
        self.view_system = Some(view_system);
        self.last_render_time = std::time::Instant::now();
    }
}

pub enum UserEvent {
    ViewSystemReady(ViewSystem),
}
