use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use super::window::{CENTER_X, CENTER_Y, WINDOW_WIDTH, WINDOW_HEIGHT};

pub const ROAD_WIDTH: u32 = 200;  // Wider roads for better visibility
pub const LANE_WIDTH: u32 = ROAD_WIDTH / 2;
const DASH_LENGTH: i32 = 30;  // Longer dashes
const GAP_LENGTH: i32 = 20;   // More space between dashes

pub fn draw_intersection(canvas: &mut Canvas<Window>) -> Result<(), String> {
   
    // Draw black road surfaces
    draw_road_surfaces(canvas)?;
    // Draw road surfaces (clear/empty - just shows background)
    draw_lane_markers(canvas)
}
fn draw_road_surfaces(canvas: &mut Canvas<Window>) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0)); // Pure black roads
    
    // Horizontal road
    canvas.fill_rect(sdl2::rect::Rect::new(
        0,
        CENTER_Y - (ROAD_WIDTH as i32 / 2),
        WINDOW_WIDTH,
        ROAD_WIDTH
    ))?;
    
    // Vertical road
    canvas.fill_rect(sdl2::rect::Rect::new(
        CENTER_X - (ROAD_WIDTH as i32 / 2),
        0,
        ROAD_WIDTH,
        WINDOW_HEIGHT
    ))?;
    
    Ok(())
}

fn draw_lane_markers(canvas: &mut Canvas<Window>) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(255, 255, 255)); // White markers

    // Horizontal center divider (east-west)
    let mut x = 0;
    while x < WINDOW_WIDTH as i32 {
        canvas.draw_line(
            (x, CENTER_Y),
            (x + DASH_LENGTH, CENTER_Y)
        )?;
        x += DASH_LENGTH + GAP_LENGTH;
    }

    // Vertical center divider (north-south)
    let mut y = 0;
    while y < WINDOW_HEIGHT as i32 {
        canvas.draw_line(
            (CENTER_X, y),
            (CENTER_X, y + DASH_LENGTH)
        )?;
        y += DASH_LENGTH + GAP_LENGTH;
    }

    // Lane boundaries (solid lines)
    canvas.draw_line(
        (0, CENTER_Y - (ROAD_WIDTH as i32 / 2)),
        (WINDOW_WIDTH as i32, CENTER_Y - (ROAD_WIDTH as i32 / 2))
    )?;
    
    canvas.draw_line(
        (0, CENTER_Y + (ROAD_WIDTH as i32 / 2)),
        (WINDOW_WIDTH as i32, CENTER_Y + (ROAD_WIDTH as i32 / 2))
    )?;
    
    canvas.draw_line(
        (CENTER_X - (ROAD_WIDTH as i32 / 2), 0),
        (CENTER_X - (ROAD_WIDTH as i32 / 2), WINDOW_HEIGHT as i32)
    )?;
    
    canvas.draw_line(
        (CENTER_X + (ROAD_WIDTH as i32 / 2), 0),
        (CENTER_X + (ROAD_WIDTH as i32 / 2), WINDOW_HEIGHT as i32)
    )?;

    Ok(())
}