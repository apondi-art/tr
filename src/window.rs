use sdl2::{self, EventPump};
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;
pub const BACKGROUND_COLOR: Color = Color::RGB(200, 200, 200);  // Medium gray background
pub const CENTER_X: i32 = (WINDOW_WIDTH / 2) as i32;
pub const CENTER_Y: i32 = (WINDOW_HEIGHT / 2) as i32;

pub fn init() -> Result<(Canvas<Window>, EventPump), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    
    let window = video_subsystem
        .window("Clear Road Intersection", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    
    let canvas = window.into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    
    let event_pump = sdl_context.event_pump()?;
    
    Ok((canvas, event_pump))
}