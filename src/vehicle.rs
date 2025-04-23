use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use rand::Rng;
use crate::window::{WINDOW_WIDTH, WINDOW_HEIGHT, CENTER_X, CENTER_Y};
use crate::traffic_light::{TrafficLightSystem, TrafficLightState, ROAD_WIDTH};

pub const LANE_WIDTH: u32 = ROAD_WIDTH / 2;
const INTERSECTION_MARGIN: i32 = 15; // Area where we check for crossing vehicles
const SAFETY_DISTANCE: i32 = 45;

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

#[derive(Clone, Copy, PartialEq)]
pub enum StopReason {
    None,
    TrafficLight,
    VehicleAhead,
    IntersectionConflict,
}

#[derive(Clone)]
pub struct Vehicle {
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
    pub turn: Turn,
    pub width: u32,
    pub height: u32,
    pub color: Color,
    pub stopped: bool,
    pub stop_reason: StopReason,
    pub max_speed: i32,
    pub current_speed: i32,
    pub acceleration: i32,
    pub deceleration: i32,
    pub following_distance: i32,
    pub arrival_time: Option<u64>,
    pub global_tick: u32, 
}

impl Vehicle {
    pub fn new(direction: Direction) -> Self {
        let lane_offset = (LANE_WIDTH / 4) as i32;
        
        let (x, y, width, height) = match direction {
            Direction::North => (
                CENTER_X - LANE_WIDTH as i32 + lane_offset,
                WINDOW_HEIGHT as i32 + 50,
                20,
                40,
            ),
            Direction::South => (
                CENTER_X + lane_offset,
                -50,
                20, 
                40,
            ),
            Direction::East => (
                -50,
                CENTER_Y + lane_offset,
                40,
                20,
            ),
            Direction::West => (
                WINDOW_WIDTH as i32 + 50,
                CENTER_Y - LANE_WIDTH as i32 + lane_offset,
                40,
                20,
            ),
        };

        let turn = Turn::random();
        let color = match turn {
            Turn::Left => Color::RGB(255, 0, 0),
            Turn::Right => Color::RGB(0, 255, 0),
            Turn::Straight => Color::RGB(0, 0, 255),
        };

        Vehicle {
            x,
            y,
            direction,
            turn,
            width,
            height,
            color,
            stopped: false,
            stop_reason: StopReason::None,
            max_speed: 3,
            current_speed: 1,
            acceleration: 1,
            deceleration: 2,
            following_distance: 50,
            arrival_time: None,
            global_tick: 0,
        }
    }

    pub fn check_vehicles_ahead(
        &mut self,
        vehicles: &[Vehicle],
        current_index: usize,
        traffic_system: &TrafficLightSystem
    ) {
        // Reset previous stops
        if self.stop_reason == StopReason::VehicleAhead || 
           self.stop_reason == StopReason::IntersectionConflict {
            self.stopped = false;
            self.stop_reason = StopReason::None;
        }

        self.check_same_direction_vehicles(vehicles, current_index);
        self.check_intersection_conflicts(vehicles, current_index);
        self.check_traffic_light(traffic_system);
    }

    fn check_same_direction_vehicles(&mut self, vehicles: &[Vehicle], current_index: usize) {
        let mut closest_distance = i32::MAX;
        let mut closest_vehicle_stopped = false;

        for (i, other) in vehicles.iter().enumerate() {
            if i == current_index || other.direction != self.direction {
                continue;
            }

            let same_lane = match self.direction {
                Direction::North | Direction::South => 
                    (self.x - other.x).abs() < LANE_WIDTH as i32 / 2,
                Direction::East | Direction::West => 
                    (self.y - other.y).abs() < LANE_WIDTH as i32 / 2,
            };

            if !same_lane {
                continue;
            }

            let distance = match self.direction {
                Direction::North => self.y - other.y - other.height as i32,
                Direction::South => other.y - self.y - self.height as i32,
                Direction::East => other.x - self.x - self.width as i32,
                Direction::West => self.x - other.x - other.width as i32,
            };

            if distance > 0 && distance < closest_distance {
                closest_distance = distance;
                closest_vehicle_stopped = other.stopped;
            }
        }

        if closest_distance < self.following_distance {
            if closest_vehicle_stopped || closest_distance < 20 {
                self.current_speed = 0;
                self.stopped = true;
                self.stop_reason = StopReason::VehicleAhead;
            } else {
                let desired_speed = (closest_distance as f32 * 0.8).min(self.max_speed as f32) as i32;
                self.current_speed = desired_speed.max(1);
            }
        } else {
            self.current_speed = (self.current_speed + self.acceleration).min(self.max_speed);
        }
    }

    fn check_intersection_conflicts(&mut self, vehicles: &[Vehicle], current_index: usize) {
        // If not approaching intersection area, reset arrival time
        if !self.approaching_intersection() {
            self.arrival_time = None;
            return;
        }
        
        // Record arrival time when first approaching intersection
        if self.arrival_time.is_none() {
            self.arrival_time = Some(self.global_tick as u64);
        }
        
        // Check for potential collisions
        let mut should_stop = false;
        
        for (i, other) in vehicles.iter().enumerate() {
            if i == current_index {
                continue;
            }
            
            // Only check vehicles approaching or in the intersection
            if !other.approaching_intersection() && !other.in_intersection_area() {
                continue;
            }
            
            // If we detect a collision path
            if self.will_collide(other) {
                // Use arrival time to determine priority
                match (self.arrival_time, other.arrival_time) {
                    (Some(self_time), Some(other_time)) => {
                        // The vehicle that arrived later should yield
                        if self_time > other_time {
                            should_stop = true;
                        }
                    },
                    (None, Some(_)) => {
                        // If we don't have arrival time but other does, we yield
                        should_stop = true;
                    },
                    _ => {
                        // If neither has arrival time yet, use position to decide
                        if self.distance_from_center() > other.distance_from_center() {
                            should_stop = true;
                        }
                    }
                }
                
                if should_stop {
                    break;
                }
            }
        }
        
        if should_stop {
            self.current_speed = 0;
            self.stopped = true;
            self.stop_reason = StopReason::IntersectionConflict;
        }
    }

    // New method to detect if vehicles are approaching the intersection
    fn approaching_intersection(&self) -> bool {
        let margin = 80; // Larger margin to detect approaching vehicles
        
        match self.direction {
            Direction::North => self.y <= CENTER_Y + ROAD_WIDTH as i32 / 2 + margin &&
                               self.y > CENTER_Y + ROAD_WIDTH as i32 / 2,
            Direction::South => self.y >= CENTER_Y - ROAD_WIDTH as i32 / 2 - margin &&
                               self.y < CENTER_Y - ROAD_WIDTH as i32 / 2,
            Direction::East => self.x <= CENTER_X + ROAD_WIDTH as i32 / 2 + margin &&
                              self.x > CENTER_X + ROAD_WIDTH as i32 / 2,
            Direction::West => self.x >= CENTER_X - ROAD_WIDTH as i32 / 2 - margin &&
                              self.x < CENTER_X - ROAD_WIDTH as i32 / 2,
        }
    }
    
    fn distance_from_center(&self) -> f32 {
        let dx = self.x + self.width as i32 / 2 - CENTER_X;
        let dy = self.y + self.height as i32 / 2 - CENTER_Y;
        ((dx * dx + dy * dy) as f32).sqrt()
    }

    fn in_intersection_area(&self) -> bool {
        match self.direction {
            Direction::North => self.y <= CENTER_Y + ROAD_WIDTH as i32 / 2 + INTERSECTION_MARGIN &&
                               self.y >= CENTER_Y - ROAD_WIDTH as i32 / 2 - INTERSECTION_MARGIN,
            Direction::South => self.y >= CENTER_Y - ROAD_WIDTH as i32 / 2 - INTERSECTION_MARGIN &&
                               self.y <= CENTER_Y + ROAD_WIDTH as i32 / 2 + INTERSECTION_MARGIN,
            Direction::East => self.x <= CENTER_X + ROAD_WIDTH as i32 / 2 + INTERSECTION_MARGIN &&
                              self.x >= CENTER_X - ROAD_WIDTH as i32 / 2 - INTERSECTION_MARGIN,
            Direction::West => self.x >= CENTER_X - ROAD_WIDTH as i32 / 2 - INTERSECTION_MARGIN &&
                              self.x <= CENTER_X + ROAD_WIDTH as i32 / 2 + INTERSECTION_MARGIN,
        }
    }

    fn will_collide(&self, other: &Vehicle) -> bool {
        // Different collision logic based on direction pairs
        match (self.direction, other.direction) {
            // Perpendicular directions
            (Direction::North, Direction::East) | (Direction::South, Direction::West) |
            (Direction::East, Direction::North) | (Direction::West, Direction::South) => {
                // Calculate time to intersection for both vehicles
                let self_time = self.time_to_intersection();
                let other_time = other.time_to_intersection();
                
                // If times are close, they will collide
                (self_time - other_time).abs() < 15
            },
            // Opposite directions don't collide unless turning
            (Direction::North, Direction::South) | (Direction::South, Direction::North) |
            (Direction::East, Direction::West) | (Direction::West, Direction::East) => {
                false // Opposite directions don't collide in standard lanes
            },
            // Same direction - check if they're in the same lane
            _ => {
                let same_lane = match self.direction {
                    Direction::North | Direction::South => 
                        (self.x - other.x).abs() < LANE_WIDTH as i32 / 2,
                    Direction::East | Direction::West => 
                        (self.y - other.y).abs() < LANE_WIDTH as i32 / 2,
                };
                
                // Fix: Use bounding box collision check instead of recursive call
                same_lane && self.bounding_box_collision(other)
            }
        }
    }

    // Add this helper method for simple bounding box collision
    fn bounding_box_collision(&self, other: &Vehicle) -> bool {
        self.x < other.x + other.width as i32 &&
        self.x + self.width as i32 > other.x &&
        self.y < other.y + other.height as i32 &&
        self.y + self.height as i32 > other.y
    }

    // Calculate time to reach center of intersection
    fn time_to_intersection(&self) -> i32 {
        if self.current_speed <= 0 {
            return i32::MAX; // Will never reach at current speed or if stopped
        }
        
        match self.direction {
            Direction::North => (self.y - CENTER_Y).max(1) / self.current_speed,
            Direction::South => (CENTER_Y - self.y).max(1) / self.current_speed,
            Direction::East => (CENTER_X - self.x).max(1) / self.current_speed,
            Direction::West => (self.x - CENTER_X).max(1) / self.current_speed,
        }
    }

    pub fn check_traffic_light(&mut self, traffic_system: &TrafficLightSystem) {
        let is_red = match self.direction {
            Direction::North => traffic_system.north_state == TrafficLightState::Red,
            Direction::South => traffic_system.south_state == TrafficLightState::Red,
            Direction::East => traffic_system.east_state == TrafficLightState::Red,
            Direction::West => traffic_system.west_state == TrafficLightState::Red,
        };
        
        if self.stop_reason == StopReason::TrafficLight {
            self.stopped = false;
            self.stop_reason = StopReason::None;
        }
        
        if is_red {
            // Adjusted distance calculation to stop vehicles closer to the intersection
            let distance_to_intersection = match self.direction {
                // For northbound vehicles, stop right at the bottom edge of the intersection
                Direction::North => self.y - (CENTER_Y + ROAD_WIDTH as i32 / 2),
                // For southbound vehicles, stop right at the top edge of the intersection
                Direction::South => (CENTER_Y - ROAD_WIDTH as i32 / 2) - (self.y + self.height as i32),
                // These are working correctly for east/west, but let's maintain consistency
                Direction::East => (CENTER_X - ROAD_WIDTH as i32 / 2) - self.x - self.width as i32,
                Direction::West => self.x - (CENTER_X + ROAD_WIDTH as i32 / 2),
            };
            
            // Adjust the detection range to be smaller (previously 60)
            // This will make vehicles stop closer to the actual intersection
            if distance_to_intersection < 30 && distance_to_intersection > 0 {
                self.stopped = true;
                self.stop_reason = StopReason::TrafficLight;
            }
        }
    }

    pub fn update(&mut self) {
        self.global_tick += 1;
        
        if !self.stopped {
            match self.direction {
                Direction::North => self.y -= self.current_speed,
                Direction::South => self.y += self.current_speed,
                Direction::East => self.x += self.current_speed,
                Direction::West => self.x -= self.current_speed,
            }
        }
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        if self.x > -100 && self.x < WINDOW_WIDTH as i32 + 100 &&
           self.y > -100 && self.y < WINDOW_HEIGHT as i32 + 100 {
            canvas.set_draw_color(self.color);
            canvas.fill_rect(Rect::new(self.x, self.y, self.width, self.height))?;
        }
        Ok(())
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