use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::graphics::GraphicsState;
use crate::APP_NAME;

#[derive(Debug, Default)]
pub struct App {
    graphics_state: Option<GraphicsState>,
    window: Option<Window>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title(APP_NAME);

        let window = event_loop
            .create_window(window_attributes)
            .expect("Error while create window");

        let graphics_state = GraphicsState::new(&window);

        self.window = Some(window);
        self.graphics_state = Some(graphics_state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let window = match self.window.as_ref() {
            None => return,
            Some(window) => window,
        };
        let graphics_state = match self.graphics_state.as_mut() {
            None => return,
            Some(state) => state,
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                graphics_state.resize(size);
            }
            WindowEvent::RedrawRequested => {
                graphics_state.render();
                window.request_redraw();
            }
            _ => {}
        }
    }
}
