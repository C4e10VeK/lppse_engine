use application::App;
use winit::event_loop::EventLoop;

mod application;
mod graphics;
mod utils;

const APP_MAJOR_VERSION: &str = env!("CARGO_PKG_VERSION_MAJOR");
const APP_MINOR_VERSION: &str = env!("CARGO_PKG_VERSION_MINOR");
const APP_PATCH_VERSION: &str = env!("CARGO_PKG_VERSION_PATCH");

const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn main() {
    init_logger();

    log::info!("Begin launch");
    let event_loop = EventLoop::builder().build().unwrap();
    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();
    log::info!("end launch");
}

#[inline]
fn init_logger() {
    let env_log = env_logger::Env::new().filter("LPPS_LOG");

    env_logger::init_from_env(env_log);
}
