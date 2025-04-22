use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use crate::window::{WINDOW_WIDTH, WINDOW_HEIGHT, CENTER_X, CENTER_Y};
use crate::road::{ROAD_WIDTH, LANE_WIDTH};

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
}

impl Vehicle {
    pub fn new(direction: Direction) -> Self {
        let (x, y, turn) = match direction {
            Direction::North => (
                CENTER_X + LANE_WIDTH as i32 / 4, // Right lane
                WINDOW_HEIGHT as i32 + 50,
                Turn::random()
            ),
            Direction::South => (
                CENTER_X - LANE_WIDTH as i32 / 4, // Left lane
                -50,
                Turn::random()
            ),
            Direction::East => (
                -50,
                CENTER_Y - LANE_WIDTH as i32 / 4,
                Turn::random()
            ),
            Direction::West => (
                WINDOW_WIDTH as i32 + 50,
                CENTER_Y + LANE_WIDTH as i32 / 4,
                Turn::random()
            ),
        };

        let color = match turn {
            Turn::Left => Color::RGB(255, 0, 0),    // Red for left
            Turn::Right => Color::RGB(0, 255, 0),   // Green for right
            Turn::Straight => Color::RGB(0, 0, 255),// Blue for straight
        };

        Vehicle {
            x,
            y,
            direction,
            turn,
            speed: 3,
            width: 20,
            height: 10,
            color,
        }
    }

    pub fn update(&mut self) {
        match self.direction {
            Direction::North => self.y -= self.speed,
            Direction::South => self.y += self.speed,
            Direction::East => self.x += self.speed,
            Direction::West => self.x -= self.speed,
        }
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.set_draw_color(self.color);
        canvas.fill_rect(Rect::new(self.x, self.y, self.width, self.height))
    }
}

impl Turn {
    fn random() -> Self {
        match rand::random::<u8>() % 3 {
            0 => Turn::Left,
            1 => Turn::Right,
            _ => Turn::Straight,
        }
    }
}