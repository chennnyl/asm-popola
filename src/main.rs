mod inter;
mod gfx;
mod render;
mod application;

use winit::event_loop::EventLoop;


fn main() {
    let mut application = application::Application::new();
    let event_loop = EventLoop::new().unwrap();

    event_loop.run_app(&mut application).unwrap();
}
