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
    let spawn_cooldown = Duration::from_secs(1);
    
    'running: loop {
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
                        let mut new_vehicle = Vehicle::new(dir);
                        new_vehicle.check_vehicles_ahead(&vehicles, vehicles.len(), &traffic_light_system);
                        vehicles.push(new_vehicle);
                        last_spawn_time = Instant::now();
                    }
                }
                _ => {}
            }
        }
        
        traffic_light_system.update();
        traffic_light_system.update_congestion(&vehicles);
        
        for i in 0..vehicles.len() {
            let (vehicles_before, vehicles_after) = vehicles.split_at_mut(i);
            let (current_vehicle, remaining_vehicles) = vehicles_after.split_first_mut().unwrap();
            
            let other_vehicles = [vehicles_before, remaining_vehicles].concat();
            current_vehicle.check_vehicles_ahead(&other_vehicles, i, &traffic_light_system);
            current_vehicle.update();
            
            if current_vehicle.x < -100 || current_vehicle.x > WINDOW_WIDTH as i32 + 100 ||
               current_vehicle.y < -100 || current_vehicle.y > WINDOW_HEIGHT as i32 + 100 {
                vehicles.remove(i);
                break;
            }
        }
        
        canvas.set_draw_color(window::BACKGROUND_COLOR);
        canvas.clear();
        
        road::draw_intersection(&mut canvas)?;
        traffic_light_system.draw(&mut canvas)?;
        
        for vehicle in &vehicles {
            vehicle.draw(&mut canvas)?;
        }
        
        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    
    Ok(())
}