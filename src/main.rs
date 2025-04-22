// main.rs
mod window;
mod road;
mod traffic_light;
mod vehicle;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Duration, Instant};
use vehicle::{Vehicle, Direction};
use crate::window::{WINDOW_WIDTH, WINDOW_HEIGHT};
use rand::Rng;

fn main() -> Result<(), String> {
    let (mut canvas, mut event_pump) = window::init()?;
    let mut traffic_light_system = traffic_light::TrafficLightSystem::new();
    let mut vehicles: Vec<Vehicle> = Vec::new();
    let mut last_spawn_time = Instant::now();
    let spawn_cooldown = Duration::from_secs(1); // 1 second between spawns
    
    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                Event::KeyDown { keycode: Some(key), .. } if last_spawn_time.elapsed() > spawn_cooldown => {
                    let direction = match key {
                        Keycode::Up => Some(Direction::South),
                        Keycode::Down => Some(Direction::North),
                        Keycode::Left => Some(Direction::East),
                        Keycode::Right => Some(Direction::West),
                        Keycode::R => {
                            let mut rng = rand::thread_rng();
                            Some(match rng.gen_range(0..4) {
                                0 => Direction::North,
                                1 => Direction::South,
                                2 => Direction::East,
                                _ => Direction::West,
                            })
                        },
                        _ => None,
                    };
                    
                    if let Some(dir) = direction {
                        vehicles.push(Vehicle::new(dir));
                        last_spawn_time = Instant::now();
                    }
                }
                _ => {}
            }
        }
        
        // Update traffic lights
        traffic_light_system.update();
        
        // First check traffic lights for all vehicles
        for vehicle in &mut vehicles {
            vehicle.check_traffic_light(&traffic_light_system);
        }
        
        // Then update vehicle positions
        for vehicle in &mut vehicles {
            vehicle.update();
        }
        
        // Remove vehicles that are out of bounds
        vehicles.retain(|v| {
            v.x > -50 && v.x < (window::WINDOW_WIDTH as i32 + 50) &&
            v.y > -50 && v.y < (window::WINDOW_HEIGHT as i32 + 50)
        });
        
        // Clear screen
        canvas.set_draw_color(window::BACKGROUND_COLOR);
        canvas.clear();
        
        // Draw intersection
        road::draw_intersection(&mut canvas)?;
        
        // Draw traffic lights
        traffic_light_system.draw(&mut canvas)?;
        
        // Draw vehicles
        for vehicle in &vehicles {
            vehicle.draw(&mut canvas)?;
        }
        
        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60)); // ~60 FPS
    }
    
    Ok(())
}