use graphics::State;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::{Window, WindowId};

mod graphics;
mod utils;

const APP_VERSION: u32 = make_version(0, 0, 1);
const APP_NAME: &str = "Project game";

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();
}

#[derive(Debug, Default)]
struct App {
    window: Option<Window>,
    state: Option<State>,
}

impl App {
    fn new() -> Self {
        Self::default()
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Project");

        let window = event_loop.create_window(window_attributes).expect("");

        let state = State::new(&window);

        self.window = Some(window);
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}

#[inline]
fn load_entry() -> ash::Entry {
    unsafe { ash::Entry::load() }.unwrap()
}

pub const fn make_version(major: u32, minor: u32, patch: u32) -> u32 {
    ((major) << 22) | ((minor) << 12) | (patch)
}
