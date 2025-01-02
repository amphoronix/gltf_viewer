use crate::app::App;
use crate::args::Args;

mod app;
pub mod args;
mod camera;
mod data;
mod error;
mod render;
mod resource;
mod view;

pub fn run(args: Args) {
    env_logger::init();

    let event_loop = App::create_event_loop().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = App::new(&event_loop, args);

    cfg_if::cfg_if! {
        if #[cfg(target_arch="wasm32")] {
            use winit::platform::web::EventLoopExtWebSys;
            event_loop.spawn_app(app);
        } else {
            event_loop.run_app(&mut app).unwrap();
        }
    }
}
