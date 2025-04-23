use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use rand::Rng;
use crate::window::{WINDOW_WIDTH, WINDOW_HEIGHT, CENTER_X, CENTER_Y};
use crate::traffic_light::{TrafficLightSystem, TrafficLightState, ROAD_WIDTH};

pub const LANE_WIDTH: u32 = ROAD_WIDTH / 2;
const INTERSECTION_MARGIN: i32 = 15;
const SAFETY_DISTANCE: i32 = 45;
const TURN_EXECUTION_ZONE: i32 = 5;

// Lane center positions
const NORTHBOUND_LANE_CENTER: i32 = CENTER_X - LANE_WIDTH as i32 / 4;
const SOUTHBOUND_LANE_CENTER: i32 = CENTER_X + LANE_WIDTH as i32 / 4;
const EASTBOUND_LANE_CENTER: i32 = CENTER_Y + LANE_WIDTH as i32 / 4;
const WESTBOUND_LANE_CENTER: i32 = CENTER_Y - LANE_WIDTH as i32 / 4;

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
    pub has_turned: bool,
    pub turn_executed: bool,
    pub target_x: i32,
    pub target_y: i32,
}

impl Vehicle {
    pub fn new(direction: Direction) -> Self {
        let (x, y, width, height, target_x, target_y) = match direction {
            Direction::North => (
                NORTHBOUND_LANE_CENTER - 10,
                WINDOW_HEIGHT as i32 + 50,
                20,
                40,
                NORTHBOUND_LANE_CENTER - 10,
                i32::MAX,
            ),
            Direction::South => (
                SOUTHBOUND_LANE_CENTER - 10,
                -50,
                20,
                40,
                SOUTHBOUND_LANE_CENTER - 10,
                i32::MAX,
            ),
            Direction::East => (
                -50,
                EASTBOUND_LANE_CENTER - 10,
                40,
                20,
                i32::MAX,
                EASTBOUND_LANE_CENTER - 10,
            ),
            Direction::West => (
                WINDOW_WIDTH as i32 + 50,
                WESTBOUND_LANE_CENTER - 10,
                40,
                20,
                i32::MAX,
                WESTBOUND_LANE_CENTER - 10,
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
            has_turned: false,
            turn_executed: false,
            target_x,
            target_y,
        }
    }

    pub fn check_vehicles_ahead(
        &mut self,
        vehicles: &[Vehicle],
        current_index: usize,
        traffic_system: &TrafficLightSystem
    ) {
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
                
                if closest_distance < 10 {
                    match self.direction {
                        Direction::North => self.y += 1,
                        Direction::South => self.y -= 1,
                        Direction::East => self.x -= 1,
                        Direction::West => self.x += 1,
                    }
                }
            } else {
                let desired_speed = (closest_distance as f32 * 0.8).min(self.max_speed as f32) as i32;
                self.current_speed = desired_speed.max(1);
            }
        } else {
            self.current_speed = (self.current_speed + self.acceleration).min(self.max_speed);
        }
    }

    fn check_intersection_conflicts(&mut self, vehicles: &[Vehicle], current_index: usize) {
        if self.stop_reason == StopReason::TrafficLight || !self.approaching_intersection() {
            if !self.approaching_intersection() {
                self.arrival_time = None;
            }
            return;
        }
        
        if self.arrival_time.is_none() {
            self.arrival_time = Some(self.global_tick as u64);
        }
        
        let mut should_stop = false;
        
        for (i, other) in vehicles.iter().enumerate() {
            if i == current_index || !other.approaching_intersection() && !other.in_intersection_area() {
                continue;
            }
            
            if other.stop_reason == StopReason::TrafficLight {
                continue;
            }
            
            if self.will_collide(other) {
                match (self.arrival_time, other.arrival_time) {
                    (Some(self_time), Some(other_time)) => {
                        if self_time > other_time {
                            should_stop = true;
                        } else if self_time == other_time {
                            should_stop = self.should_yield_to(other);
                        }
                    },
                    (None, Some(_)) => should_stop = true,
                    (Some(_), None) => should_stop = false,
                    _ => should_stop = self.should_yield_to(other),
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
        } else if self.stop_reason == StopReason::IntersectionConflict {
            self.stopped = false;
            self.stop_reason = StopReason::None;
        }
    }

    fn should_yield_to(&self, other: &Vehicle) -> bool {
        match (self.direction, other.direction) {
            (Direction::North, Direction::East) => true,
            (Direction::East, Direction::South) => true,
            (Direction::South, Direction::West) => true,
            (Direction::West, Direction::North) => true,
            
            (Direction::North, Direction::West) => false,
            (Direction::East, Direction::North) => false,
            (Direction::South, Direction::East) => false,
            (Direction::West, Direction::South) => false,
            
            (Direction::North, Direction::South) | (Direction::South, Direction::North) |
            (Direction::East, Direction::West) | (Direction::West, Direction::East) => {
                match (self.turn, other.turn) {
                    (Turn::Left, Turn::Straight) | (Turn::Left, Turn::Right) => true,
                    (Turn::Straight, Turn::Right) => true,
                    (Turn::Right, _) => false,
                    _ => self.distance_from_center() > other.distance_from_center()
                }
            },
            
            _ => self.distance_from_center() > other.distance_from_center()
        }
    }

    fn approaching_intersection(&self) -> bool {
        match self.direction {
            Direction::North => self.y <= CENTER_Y + ROAD_WIDTH as i32 / 2 + 80 &&
                               self.y > CENTER_Y + ROAD_WIDTH as i32 / 2,
            Direction::South => self.y >= CENTER_Y - ROAD_WIDTH as i32 / 2 - 80 &&
                               self.y < CENTER_Y - ROAD_WIDTH as i32 / 2,
            Direction::East => self.x <= CENTER_X + ROAD_WIDTH as i32 / 2 + 80 &&
                              self.x > CENTER_X + ROAD_WIDTH as i32 / 2,
            Direction::West => self.x >= CENTER_X - ROAD_WIDTH as i32 / 2 - 80 &&
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
        if self.in_intersection_area() && self.current_speed > 0 {
            return false;
        }
        
        if self.stopped && other.stopped && 
           self.stop_reason == StopReason::IntersectionConflict &&
           other.stop_reason == StopReason::IntersectionConflict {
            return self.global_tick > other.global_tick;
        }
        
        if (self.direction == Direction::North && other.direction == Direction::South) ||
           (self.direction == Direction::South && other.direction == Direction::North) {
            if (self.turn == Turn::Left && other.turn != Turn::Right) ||
               (other.turn == Turn::Left && self.turn != Turn::Right) {
                let self_time = self.time_to_intersection();
                let other_time = other.time_to_intersection();
                return (self_time - other_time).abs() < 20;
            }
        }
        
        if (self.direction == Direction::East && other.direction == Direction::West) ||
           (self.direction == Direction::West && other.direction == Direction::East) {
            if (self.turn == Turn::Left && other.turn != Turn::Right) ||
               (other.turn == Turn::Left && self.turn != Turn::Right) {
                let self_time = self.time_to_intersection();
                let other_time = other.time_to_intersection();
                return (self_time - other_time).abs() < 20;
            }
        }
        
        match (self.direction, other.direction) {
            (Direction::North, Direction::East) | (Direction::South, Direction::West) |
            (Direction::East, Direction::North) | (Direction::West, Direction::South) |
            (Direction::North, Direction::West) | (Direction::South, Direction::East) |
            (Direction::East, Direction::South) | (Direction::West, Direction::North) => {
                let self_time = self.time_to_intersection();
                let other_time = other.time_to_intersection();
                let collision_window = 15 + (self.current_speed + other.current_speed) / 2;
                (self_time - other_time).abs() < collision_window
            },
            _ => {
                let same_lane = match self.direction {
                    Direction::North | Direction::South => 
                        (self.x - other.x).abs() < LANE_WIDTH as i32 / 2,
                    Direction::East | Direction::West => 
                        (self.y - other.y).abs() < LANE_WIDTH as i32 / 2,
                };
                same_lane && self.bounding_box_collision(other)
            }
        }
    }

    pub fn bounding_box_collision(&self, other: &Vehicle) -> bool {
        let margin = 2;
        self.x < other.x + other.width as i32 - margin &&
        self.x + self.width as i32 - margin > other.x &&
        self.y < other.y + other.height as i32 - margin &&
        self.y + self.height as i32 - margin > other.y
    }

    fn time_to_intersection(&self) -> i32 {
        if self.current_speed <= 0 {
            return i32::MAX;
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
            let distance_to_intersection = match self.direction {
                Direction::North => self.y - (CENTER_Y + ROAD_WIDTH as i32 / 2),
                Direction::South => (CENTER_Y - ROAD_WIDTH as i32 / 2) - (self.y + self.height as i32),
                Direction::East => (CENTER_X - ROAD_WIDTH as i32 / 2) - self.x - self.width as i32,
                Direction::West => self.x - (CENTER_X + ROAD_WIDTH as i32 / 2),
            };
            
            if distance_to_intersection < 30 && distance_to_intersection > 0 {
                self.stopped = true;
                self.stop_reason = StopReason::TrafficLight;
            }
        }
    }

    fn in_turn_zone(&self) -> bool {
        match self.direction {
            Direction::North => (self.y - CENTER_Y).abs() <= TURN_EXECUTION_ZONE,
            Direction::South => (self.y - CENTER_Y).abs() <= TURN_EXECUTION_ZONE,
            Direction::East => (self.x - CENTER_X).abs() <= TURN_EXECUTION_ZONE,
            Direction::West => (self.x - CENTER_X).abs() <= TURN_EXECUTION_ZONE,
        }
    }

    fn adjust_lane_position(&mut self) {
        if self.in_intersection_area() || !self.has_turned {
            return;
        }

        match self.direction {
            Direction::North | Direction::South => {
                if (self.x - self.target_x).abs() > 1 {
                    self.x += if self.x < self.target_x { 1 } else { -1 };
                } else {
                    self.x = self.target_x;
                }
            },
            Direction::East | Direction::West => {
                if (self.y - self.target_y).abs() > 1 {
                    self.y += if self.y < self.target_y { 1 } else { -1 };
                } else {
                    self.y = self.target_y;
                }
            },
        }
    }

    pub fn handle_intersection_turn(&mut self) {
        if self.in_turn_zone() && !self.stopped && !self.turn_executed {
            self.turn_executed = true;

            match self.direction {
                Direction::North => {
                    match self.turn {
                        Turn::Left => {
                            self.direction = Direction::West;
                            self.x = CENTER_X - self.height as i32;
                            self.y = CENTER_Y - LANE_WIDTH as i32 / 2;
                            self.target_y = WESTBOUND_LANE_CENTER - 10;
                            std::mem::swap(&mut self.width, &mut self.height);
                        },
                        Turn::Right => {
                            self.direction = Direction::East;
                            self.x = CENTER_X;
                            self.y = CENTER_Y + LANE_WIDTH as i32 / 2;
                            self.target_y = EASTBOUND_LANE_CENTER - 10;
                            std::mem::swap(&mut self.width, &mut self.height);
                        },
                        Turn::Straight => {
                            self.target_x = NORTHBOUND_LANE_CENTER - 10;
                        }
                    }
                },
                Direction::South => {
                    match self.turn {
                        Turn::Left => {
                            self.direction = Direction::East;
                            self.x = CENTER_X;
                            self.y = CENTER_Y + LANE_WIDTH as i32 / 2;
                            self.target_y = EASTBOUND_LANE_CENTER - 10;
                            std::mem::swap(&mut self.width, &mut self.height);
                        },
                        Turn::Right => {
                            self.direction = Direction::West;
                            self.x = CENTER_X - self.height as i32;
                            self.y = CENTER_Y - LANE_WIDTH as i32 / 2;
                            self.target_y = WESTBOUND_LANE_CENTER - 10;
                            std::mem::swap(&mut self.width, &mut self.height);
                        },
                        Turn::Straight => {
                            self.target_x = SOUTHBOUND_LANE_CENTER - 10;
                        }
                    }
                },
                Direction::East => {
                    match self.turn {
                        Turn::Left => {
                            self.direction = Direction::North;
                            self.x = CENTER_X - LANE_WIDTH as i32 / 2;
                            self.y = CENTER_Y - self.height as i32;
                            self.target_x = NORTHBOUND_LANE_CENTER - 10;
                            std::mem::swap(&mut self.width, &mut self.height);
                        },
                        Turn::Right => {
                            self.direction = Direction::South;
                            self.x = CENTER_X + LANE_WIDTH as i32 / 2;
                            self.y = CENTER_Y;
                            self.target_x = SOUTHBOUND_LANE_CENTER - 10;
                            std::mem::swap(&mut self.width, &mut self.height);
                        },
                        Turn::Straight => {
                            self.target_y = EASTBOUND_LANE_CENTER - 10;
                        }
                    }
                },
                Direction::West => {
                    match self.turn {
                        Turn::Left => {
                            self.direction = Direction::South;
                            self.x = CENTER_X + LANE_WIDTH as i32 / 2;
                            self.y = CENTER_Y;
                            self.target_x = SOUTHBOUND_LANE_CENTER - 10;
                            std::mem::swap(&mut self.width, &mut self.height);
                        },
                        Turn::Right => {
                            self.direction = Direction::North;
                            self.x = CENTER_X - LANE_WIDTH as i32 / 2;
                            self.y = CENTER_Y - self.height as i32;
                            self.target_x = NORTHBOUND_LANE_CENTER - 10;
                            std::mem::swap(&mut self.width, &mut self.height);
                        },
                        Turn::Straight => {
                            self.target_y = WESTBOUND_LANE_CENTER - 10;
                        }
                    }
                },
            }
            self.has_turned = true;
            self.arrival_time = None;
        }
    }

    pub fn randomize_turn_if_needed(&mut self) {
        if self.approaching_intersection() && !self.in_intersection_area() && 
           !self.has_turned && !self.stopped {
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.3) {
                self.turn = Turn::random();
                self.color = match self.turn {
                    Turn::Left => Color::RGB(255, 0, 0),
                    Turn::Right => Color::RGB(0, 255, 0),
                    Turn::Straight => Color::RGB(0, 0, 255),
                };
            }
        }
    }

    pub fn check_traffic_and_vehicles(&mut self, vehicles: &[Vehicle], current_index: usize, traffic_system: &TrafficLightSystem) {
        self.check_traffic_light(traffic_system);
        
        if self.stop_reason != StopReason::TrafficLight {
            self.check_same_direction_vehicles(vehicles, current_index);
            
            if self.stop_reason != StopReason::VehicleAhead {
                self.check_intersection_conflicts(vehicles, current_index);
            }
        }
    }

    pub fn update(&mut self, vehicles: &[Vehicle], current_index: usize, traffic_system: &TrafficLightSystem) {
        self.global_tick += 1;
        
        self.randomize_turn_if_needed();
        self.check_traffic_and_vehicles(vehicles, current_index, traffic_system);
        
        if !self.stopped {
            self.handle_intersection_turn();
            
            match self.direction {
                Direction::North => self.y -= self.current_speed,
                Direction::South => self.y += self.current_speed,
                Direction::East => self.x += self.current_speed,
                Direction::West => self.x -= self.current_speed,
            }
            
            self.adjust_lane_position();
        } else if self.in_intersection_area() && self.stop_reason == StopReason::IntersectionConflict {
            if self.global_tick % 100 == 0 {
                self.stopped = false;
                self.stop_reason = StopReason::None;
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