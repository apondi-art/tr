// traffic_light.rs
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Instant;
use super::window::{CENTER_X, CENTER_Y};

// Constants for traffic light dimensions
pub const ROAD_WIDTH: u32 = 100;
const TRAFFIC_LIGHT_WIDTH: u32 = 15;
const TRAFFIC_LIGHT_HEIGHT: u32 = 40;
const LIGHT_RADIUS: i32 = 5;

#[derive(Clone, Copy, PartialEq)]
pub enum TrafficLightState {
    Red,
    Green,
}

pub struct TrafficLightSystem {
    pub north_state: TrafficLightState,
    pub south_state: TrafficLightState,
    pub east_state: TrafficLightState,
    pub west_state: TrafficLightState,
    pub last_change: Instant,
    pub change_interval: u64,
}

impl TrafficLightSystem {
    pub fn new() -> Self {
        TrafficLightSystem {
            north_state: TrafficLightState::Red,
            south_state: TrafficLightState::Red,
            east_state: TrafficLightState::Green,
            west_state: TrafficLightState::Green,
            last_change: Instant::now(),
            change_interval: 5,
        }
    }
    
    pub fn update(&mut self) {
        if self.last_change.elapsed().as_secs() >= self.change_interval {
            if self.north_state == TrafficLightState::Red {
                // Switch to north-south green, east-west red
                self.north_state = TrafficLightState::Green;
                self.south_state = TrafficLightState::Green;
                self.east_state = TrafficLightState::Red;
                self.west_state = TrafficLightState::Red;
            } else {
                // Switch to north-south red, east-west green
                self.north_state = TrafficLightState::Red;
                self.south_state = TrafficLightState::Red;
                self.east_state = TrafficLightState::Green;
                self.west_state = TrafficLightState::Green;
            }
            self.last_change = Instant::now();
        }
    }
    
    pub fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        // Draw North traffic light - on the dotted line at the north approach
        draw_traffic_light(
            canvas, 
            CENTER_X, // Position it on the center line (dotted line)
            CENTER_Y - ROAD_WIDTH as i32 / 2 - TRAFFIC_LIGHT_HEIGHT as i32,
            self.north_state
        )?;
        
        // Draw South traffic light - on the dotted line at the south approach
        draw_traffic_light(
            canvas, 
            CENTER_X, // Position it on the center line (dotted line)
            CENTER_Y + ROAD_WIDTH as i32 / 2,
            self.south_state
        )?;
        
        // Draw East traffic light - on the dotted line at the east approach
        draw_traffic_light(
            canvas, 
            CENTER_X - ROAD_WIDTH as i32 / 2 - TRAFFIC_LIGHT_HEIGHT as i32,
            CENTER_Y, // Position it on the center line (dotted line)
            self.east_state
        )?;
        
        // Draw West traffic light - on the dotted line at the west approach
        draw_traffic_light(
            canvas, 
            CENTER_X + ROAD_WIDTH as i32 / 2,
            CENTER_Y, // Position it on the center line (dotted line)
            self.west_state
        )?;
        
        Ok(())
    }
}

fn draw_traffic_light(
    canvas: &mut Canvas<Window>, 
    x: i32, 
    y: i32, 
    state: TrafficLightState
) -> Result<(), String> {
    // Draw traffic light box
    canvas.set_draw_color(Color::RGB(50, 50, 50)); // Dark gray for the traffic light housing
    
    // Adjust orientation based on position
    let is_vertical = x < CENTER_X - ROAD_WIDTH as i32 / 4 || x > CENTER_X + ROAD_WIDTH as i32 / 4;
    
    let (width, height) = if is_vertical {
        (TRAFFIC_LIGHT_HEIGHT, TRAFFIC_LIGHT_WIDTH) // Horizontal orientation
    } else {
        (TRAFFIC_LIGHT_WIDTH, TRAFFIC_LIGHT_HEIGHT) // Vertical orientation
    };
    
    canvas.fill_rect(sdl2::rect::Rect::new(x, y, width, height))?;
    
    // Draw border
    canvas.set_draw_color(Color::RGB(80, 80, 80));
    canvas.draw_rect(sdl2::rect::Rect::new(x, y, width, height))?;
    
    // Calculate light positions
    let (red_x, red_y, green_x, green_y) = if is_vertical {
        let center_y = y + height as i32 / 2;
        (
            x + width as i32 / 4,     // Red position
            center_y,
            x + 3 * width as i32 / 4, // Green position
            center_y
        )
    } else {
        let center_x = x + width as i32 / 2;
        (
            center_x,
            y + height as i32 / 4,     // Red position
            center_x,
            y + 3 * height as i32 / 4  // Green position
        )
    };
    
    // Draw red light (dimmed if not active)
    if state == TrafficLightState::Red {
        canvas.set_draw_color(Color::RGB(255, 0, 0)); // Bright red
    } else {
        canvas.set_draw_color(Color::RGB(100, 0, 0)); // Dim red
    }
    draw_filled_circle(canvas, red_x, red_y, LIGHT_RADIUS)?;
    
    // Draw green light (dimmed if not active)
    if state == TrafficLightState::Green {
        canvas.set_draw_color(Color::RGB(0, 255, 0)); // Bright green
    } else {
        canvas.set_draw_color(Color::RGB(0, 100, 0)); // Dim green
    }
    draw_filled_circle(canvas, green_x, green_y, LIGHT_RADIUS)?;
    
    Ok(())
}

fn draw_filled_circle(canvas: &mut Canvas<Window>, x: i32, y: i32, radius: i32) -> Result<(), String> {
    // Simple circle drawing implementation
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx*dx + dy*dy <= radius*radius {
                canvas.draw_point((x + dx, y + dy))?;
            }
        }
    }
    Ok(())
}