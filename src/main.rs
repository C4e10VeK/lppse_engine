use graphics::State;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::windows::{Color, WindowExtWindows};
use winit::window::{Window, WindowButtons, WindowId};

mod graphics;
mod utils;

const APP_MAJOR_VERSION: &str = env!("CARGO_PKG_VERSION_MAJOR");
const APP_MINOR_VERSION: &str = env!("CARGO_PKG_VERSION_MINOR");
const APP_PATCH_VERSION: &str = env!("CARGO_PKG_VERSION_PATCH");

const APP_NAME: &str = env!("CARGO_PKG_NAME");

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
        let window_attributes = Window::default_attributes()
            .with_title(APP_NAME)
            .with_transparent(true)
            .with_resizable(false)
            .with_enabled_buttons(WindowButtons::MINIMIZE | WindowButtons::CLOSE);

        let window = event_loop.create_window(window_attributes).expect("");

        let state = State::new(&window);

        self.window = Some(window);
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let window = match self.window.as_mut() {
            None => return,
            Some(window) => window
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                println!("redraw requested called: {:?}", window)
            }
            _ => {}
        }
    }
}
