use game_loop::game_loop;

//use game_loop::winit::event::{Event, WindowEvent};
use game_loop::winit::event_loop::EventLoop;
use game_loop::winit::window::WindowBuilder;

mod game;
mod constants;

async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let game = game::Game::new(&window).await;

    game_loop(event_loop, window, game, 30, 0.1, |g| {
        g.game.update();
    }, |g| {
        match g.game.render() {
            Ok(_) => {}
            // Reconfigure the surface if lost
            Err(wgpu::SurfaceError::Lost) => g.game.reconfigure(),
            // The system is out of memory, we should probably quit
            Err(wgpu::SurfaceError::OutOfMemory) => g.exit(),
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        };
    }, |g, event| {
        if g.game.handle_event(event, &g.window) { g.exit() };
    });
}

fn main() {
    pollster::block_on(run());
}