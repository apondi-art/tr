// road.rs
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use crate::window::{WINDOW_WIDTH, WINDOW_HEIGHT, CENTER_X, CENTER_Y};
use crate::traffic_light::ROAD_WIDTH;

pub fn draw_intersection(canvas: &mut Canvas<Window>) -> Result<(), String> {
    // Draw horizontal road
    canvas.set_draw_color(Color::RGB(50, 50, 50)); // Dark gray for road
    canvas.fill_rect(Rect::new(0, CENTER_Y - ROAD_WIDTH as i32 / 2, WINDOW_WIDTH, ROAD_WIDTH))?;
    
    // Draw vertical road
    canvas.fill_rect(Rect::new(CENTER_X - ROAD_WIDTH as i32 / 2, 0, ROAD_WIDTH, WINDOW_HEIGHT))?;
    
    // Draw lane markings
    draw_road_markings(canvas)?;
    
    Ok(())
}

fn draw_road_markings(canvas: &mut Canvas<Window>) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(255, 255, 0)); // Yellow for road markings
    
    // Horizontal road center line
    let dash_length = 20;
    let gap_length = 20;
    let total_dashes = (WINDOW_WIDTH / (dash_length + gap_length)) as i32 + 1;
    
    for i in 0..total_dashes {
        let x = i * (dash_length + gap_length) as i32;
        canvas.fill_rect(Rect::new(
            x, 
            CENTER_Y - 1, // Center line thickness of 2
            dash_length, 
            2
        ))?;
    }
    
    // Vertical road center line
    let total_dashes_vert = (WINDOW_HEIGHT / (dash_length + gap_length)) as i32 + 1;
    
    for i in 0..total_dashes_vert {
        let y = i * (dash_length + gap_length) as i32;
        canvas.fill_rect(Rect::new(
            CENTER_X - 1, // Center line thickness of 2
            y, 
            2, 
            dash_length
        ))?;
    }
    
    // Draw stop lines at intersection
    canvas.set_draw_color(Color::RGB(255, 255, 255)); // White for stop lines
    
    // North stop line
    canvas.fill_rect(Rect::new(
        CENTER_X - ROAD_WIDTH as i32 / 2,
        CENTER_Y - ROAD_WIDTH as i32 / 2 - 5,
        ROAD_WIDTH,
        3
    ))?;
    
    // South stop line
    canvas.fill_rect(Rect::new(
        CENTER_X - ROAD_WIDTH as i32 / 2,
        CENTER_Y + ROAD_WIDTH as i32 / 2 + 2,
        ROAD_WIDTH,
        3
    ))?;
    
    // East stop line
    canvas.fill_rect(Rect::new(
        CENTER_X - ROAD_WIDTH as i32 / 2 - 5,
        CENTER_Y - ROAD_WIDTH as i32 / 2,
        3,
        ROAD_WIDTH
    ))?;
    
    // West stop line
    canvas.fill_rect(Rect::new(
        CENTER_X + ROAD_WIDTH as i32 / 2 + 2,
        CENTER_Y - ROAD_WIDTH as i32 / 2,
        3,
        ROAD_WIDTH
    ))?;
    
    Ok(())
}