// window.rs
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

// Window dimensions
pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

// Center coordinates
pub const CENTER_X: i32 = WINDOW_WIDTH as i32 / 2;
pub const CENTER_Y: i32 = WINDOW_HEIGHT as i32 / 2;

// Background color
pub const BACKGROUND_COLOR: Color = Color::RGB(100, 100, 100); // Gray

pub fn init() -> Result<(Canvas<Window>, sdl2::EventPump), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Traffic Simulation", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;

    let event_pump = sdl_context.event_pump()?;

    Ok((canvas, event_pump))
}