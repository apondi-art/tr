// vehicle.rs
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use rand::Rng;
use crate::window::{WINDOW_WIDTH, WINDOW_HEIGHT, CENTER_X, CENTER_Y};
use crate::traffic_light::{TrafficLightSystem, TrafficLightState};
use crate::traffic_light::ROAD_WIDTH;
// Lane width derived from road width
pub const LANE_WIDTH: u32 = ROAD_WIDTH / 2;

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Turn {
    Left,
    Right,
    Straight,
}

pub struct Vehicle {
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
    pub turn: Turn,
    pub speed: i32,
    pub width: u32,
    pub height: u32,
    pub color: Color,
    pub stopped: bool,
}

impl Vehicle {
    pub fn new(direction: Direction) -> Self {
        // Position and orientation of vehicle based on direction
        let (x, y, width, height) = match direction {
            Direction::North => (
                CENTER_X - LANE_WIDTH as i32 / 2, // Centered in left lane (going north)
                WINDOW_HEIGHT as i32 + 50,
                20,
                40,
            ),
            Direction::South => (
                CENTER_X + LANE_WIDTH as i32 / 2 - 20, // Centered in right lane (going south)
                -50,
                20, 
                40,
            ),
            Direction::East => (
                -50,
                CENTER_Y + LANE_WIDTH as i32 / 2 - 20, // Centered in bottom lane (going east)
                40,
                20,
            ),
            Direction::West => (
                WINDOW_WIDTH as i32 + 50,
                CENTER_Y - LANE_WIDTH as i32 / 2, // Centered in top lane (going west)
                40,
                20,
            ),
        };

        let turn = Turn::random();
        
        let color = match turn {
            Turn::Left => Color::RGB(255, 0, 0),     // Red for left
            Turn::Right => Color::RGB(0, 255, 0),    // Green for right
            Turn::Straight => Color::RGB(0, 0, 255), // Blue for straight
        };

        Vehicle {
            x,
            y,
            direction,
            turn,
            speed: 3,
            width,
            height,
            color,
            stopped: false,
        }
    }

    pub fn update(&mut self) {
        if !self.stopped {
            match self.direction {
                Direction::North => self.y -= self.speed,
                Direction::South => self.y += self.speed,
                Direction::East => self.x += self.speed,
                Direction::West => self.x -= self.speed,
            }
        }
    }
    
    pub fn check_traffic_light(&mut self, traffic_system: &TrafficLightSystem) {
        // Determine if light is red for this vehicle's direction
        let is_red = match self.direction {
            Direction::North => traffic_system.north_state == TrafficLightState::Red,
            Direction::South => traffic_system.south_state == TrafficLightState::Red,
            Direction::East => traffic_system.east_state == TrafficLightState::Red,
            Direction::West => traffic_system.west_state == TrafficLightState::Red,
        };
        
        if is_red {
            // Calculate distance to intersection
            let distance_to_intersection = match self.direction {
                Direction::North => self.y - (CENTER_Y - ROAD_WIDTH as i32 / 2),
                Direction::South => (CENTER_Y + ROAD_WIDTH as i32 / 2) - self.y - self.height as i32,
                Direction::East => (CENTER_X - ROAD_WIDTH as i32 / 2) - self.x - self.width as i32,
                Direction::West => self.x - (CENTER_X + ROAD_WIDTH as i32 / 2),
            };
            
            // Stop if approaching intersection but not yet in it
            self.stopped = distance_to_intersection < 60 && distance_to_intersection > 0;
        } else {
            // Green light, can proceed
            self.stopped = false;
        }
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.set_draw_color(self.color);
        canvas.fill_rect(Rect::new(self.x, self.y, self.width, self.height))
    }
}

impl Turn {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..3) {
            0 => Turn::Left,
            1 => Turn::Right,
            _ => Turn::Straight,
        }
    }
}