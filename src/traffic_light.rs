use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Instant;
use sdl2::rect::Rect;
use crate::vehicle::StopReason;
use crate::window::{CENTER_X, CENTER_Y};
use crate::vehicle::{Vehicle, Direction};

pub const ROAD_WIDTH: u32 = 100;
const TRAFFIC_LIGHT_DISTANCE: i32 = 20; // Distance from road edge
const LIGHT_POLE_WIDTH: u32 = 10;
const LIGHT_POLE_HEIGHT: u32 = 30;
const LIGHT_HEAD_WIDTH: u32 = 20;
const LIGHT_HEAD_HEIGHT: u32 = 40;
const LIGHT_RADIUS: i32 = 6;

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
    pub min_interval: u64,
    pub max_interval: u64,
    pub north_south_congestion: u32,
    pub east_west_congestion: u32,
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
            min_interval: 3,
            max_interval: 10,
            north_south_congestion: 0,
            east_west_congestion: 0,
        }
    }

    pub fn update_congestion(&mut self, vehicles: &[Vehicle]) {
        self.north_south_congestion = 0;
        self.east_west_congestion = 0;
        
        for vehicle in vehicles {
            if vehicle.stopped && vehicle.stop_reason == StopReason::TrafficLight {
                match vehicle.direction {
                    Direction::North | Direction::South => self.north_south_congestion += 1,
                    Direction::East | Direction::West => self.east_west_congestion += 1,
                }
            }
        }
        self.adapt_timing();
    }

    fn adapt_timing(&mut self) {
        const CONGESTION_THRESHOLD: u32 = 4;
        
        if self.north_state == TrafficLightState::Green {
            self.change_interval = if self.east_west_congestion >= CONGESTION_THRESHOLD {
                self.min_interval
            } else {
                5
            };
        } else {
            self.change_interval = if self.north_south_congestion >= CONGESTION_THRESHOLD {
                self.min_interval
            } else {
                5
            };
        }
        
        if self.north_south_congestion >= CONGESTION_THRESHOLD && 
           self.east_west_congestion >= CONGESTION_THRESHOLD {
            self.change_interval = 5;
        }
        
        if self.north_south_congestion < 2 && self.east_west_congestion < 2 {
            self.change_interval = self.max_interval;
        }
    }
    
    pub fn update(&mut self) {
        if self.last_change.elapsed().as_secs() >= self.change_interval {
            if self.north_state == TrafficLightState::Red {
                self.north_state = TrafficLightState::Green;
                self.south_state = TrafficLightState::Green;
                self.east_state = TrafficLightState::Red;
                self.west_state = TrafficLightState::Red;
            } else {
                self.north_state = TrafficLightState::Red;
                self.south_state = TrafficLightState::Red;
                self.east_state = TrafficLightState::Green;
                self.west_state = TrafficLightState::Green;
            }
            self.last_change = Instant::now();
        }
    }
    
    pub fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        // Northbound light (facing south)
        self.draw_traffic_light(
            canvas,
            CENTER_X - LIGHT_HEAD_WIDTH as i32 / 2,
            CENTER_Y - ROAD_WIDTH as i32 / 2 - TRAFFIC_LIGHT_DISTANCE - LIGHT_HEAD_HEIGHT as i32,
            false, // horizontal
            self.north_state
        )?;
        
        // Southbound light (facing north)
        self.draw_traffic_light(
            canvas,
            CENTER_X - LIGHT_HEAD_WIDTH as i32 / 2,
            CENTER_Y + ROAD_WIDTH as i32 / 2 + TRAFFIC_LIGHT_DISTANCE,
            false, // horizontal
            self.south_state
        )?;
        
        // Eastbound light (facing west)
        self.draw_traffic_light(
            canvas,
            CENTER_X + ROAD_WIDTH as i32 / 2 + TRAFFIC_LIGHT_DISTANCE,
            CENTER_Y - LIGHT_HEAD_WIDTH as i32 / 2,
            true, // vertical
            self.east_state
        )?;
        
        // Westbound light (facing east)
        self.draw_traffic_light(
            canvas,
            CENTER_X - ROAD_WIDTH as i32 / 2 - TRAFFIC_LIGHT_DISTANCE - LIGHT_HEAD_HEIGHT as i32,
            CENTER_Y - LIGHT_HEAD_WIDTH as i32 / 2,
            true, // vertical
            self.west_state
        )?;
        
        Ok(())
    }
    
    fn draw_traffic_light(
        &self,
        canvas: &mut Canvas<Window>,
        x: i32,
        y: i32,
        vertical: bool,
        state: TrafficLightState
    ) -> Result<(), String> {
        // Draw pole
        canvas.set_draw_color(Color::RGB(70, 70, 70));
        let pole_rect = if vertical {
            Rect::new(
                x + LIGHT_HEAD_HEIGHT as i32 / 2 - LIGHT_POLE_WIDTH as i32 / 2,
                y + LIGHT_HEAD_WIDTH as i32,
                LIGHT_POLE_WIDTH,
                LIGHT_POLE_HEIGHT
            )
        } else {
            Rect::new(
                x + LIGHT_HEAD_WIDTH as i32 / 2 - LIGHT_POLE_WIDTH as i32 / 2,
                y - LIGHT_POLE_HEIGHT as i32,
                LIGHT_POLE_WIDTH,
                LIGHT_POLE_HEIGHT
            )
        };
        canvas.fill_rect(pole_rect)?;
        
        // Draw light head
        canvas.set_draw_color(Color::RGB(40, 40, 40));
        let head_rect = Rect::new(x, y, 
            if vertical { LIGHT_HEAD_HEIGHT } else { LIGHT_HEAD_WIDTH },
            if vertical { LIGHT_HEAD_WIDTH } else { LIGHT_HEAD_HEIGHT }
        );
        canvas.fill_rect(head_rect)?;
        canvas.set_draw_color(Color::RGB(20, 20, 20));
        canvas.draw_rect(head_rect)?;
        
        // Calculate light positions
        let (red_pos, green_pos) = if vertical {
            // Vertical light (for east/west traffic)
            (
                (x + LIGHT_HEAD_HEIGHT as i32 / 2, y + LIGHT_HEAD_WIDTH as i32 / 3),
                (x + LIGHT_HEAD_HEIGHT as i32 / 2, y + 2 * LIGHT_HEAD_WIDTH as i32 / 3)
            )
        } else {
            // Horizontal light (for north/south traffic)
            (
                (x + LIGHT_HEAD_WIDTH as i32 / 3, y + LIGHT_HEAD_HEIGHT as i32 / 2),
                (x + 2 * LIGHT_HEAD_WIDTH as i32 / 3, y + LIGHT_HEAD_HEIGHT as i32 / 2)
            )
        };
        
        // Draw red light
        canvas.set_draw_color(if state == TrafficLightState::Red {
            Color::RGB(255, 0, 0)
        } else {
            Color::RGB(80, 0, 0)
        });
        self.draw_filled_circle(canvas, red_pos.0, red_pos.1, LIGHT_RADIUS)?;
        
        // Draw green light
        canvas.set_draw_color(if state == TrafficLightState::Green {
            Color::RGB(0, 255, 0)
        } else {
            Color::RGB(0, 80, 0)
        });
        self.draw_filled_circle(canvas, green_pos.0, green_pos.1, LIGHT_RADIUS)?;
        
        Ok(())
    }
    
    fn draw_filled_circle(
        &self, 
        canvas: &mut Canvas<Window>, 
        x: i32, 
        y: i32, 
        radius: i32
    ) -> Result<(), String> {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx*dx + dy*dy <= radius*radius {
                    canvas.draw_point((x + dx, y + dy))?;
                }
            }
        }
        Ok(())
    }
}