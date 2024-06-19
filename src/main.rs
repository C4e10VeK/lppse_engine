// TODO: remove this for check unused
#![allow(dead_code)]

use application::App;
use winit::event_loop::EventLoop;
#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;

mod application;
mod graphics;
mod utils;

const APP_MAJOR_VERSION: &str = env!("CARGO_PKG_VERSION_MAJOR");
const APP_MINOR_VERSION: &str = env!("CARGO_PKG_VERSION_MINOR");
const APP_PATCH_VERSION: &str = env!("CARGO_PKG_VERSION_PATCH");
const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn main() {
    init_logger();

    let event_loop = create_event_loop();
    let mut app = App::new();

    log::info!("Begin launch");
    event_loop.run_app(&mut app).unwrap();
    log::info!("end launch");
}

#[inline]
fn init_logger() {
    let env_log = env_logger::Env::new().filter_or("LPPS_LOG", "DEBUG");

    env_logger::init_from_env(env_log);
}

#[cfg(target_os = "linux")]
#[inline]
fn create_event_loop() -> EventLoop<()> {
    EventLoop::builder().with_x11().build().unwrap()
}

#[cfg(target_os = "windows")]
#[inline]
fn create_event_loop() -> EventLoop<()> {
    EventLoop::new().unwrap()
}
