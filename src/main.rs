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
        
        // Update traffic lights
        traffic_light_system.update();
        traffic_light_system.update_congestion(&vehicles);
        
        // Process vehicles with safer index handling
       // In your main.rs, update the vehicle processing loop:

// Process vehicles with safer index handling
let mut i = 0;
while i < vehicles.len() {
    // Create a temporary copy of other vehicles for collision checking
    let mut other_vehicles = Vec::new();
    for (j, vehicle) in vehicles.iter().enumerate() {
        if j != i {
            other_vehicles.push(vehicle.clone());
        }
    }
    
    // Store the previous position and state
    let prev_x = vehicles[i].x;
    let prev_y = vehicles[i].y;
    let prev_stopped = vehicles[i].stopped;
    
    // Check for collisions and update vehicle
    vehicles[i].check_vehicles_ahead(&other_vehicles, 0, &traffic_light_system);
   
vehicles[i].update(&other_vehicles, 0, &traffic_light_system);
    
    // Check if vehicle overlaps with any other vehicle after movement
    let mut has_overlap = false;
    for j in 0..vehicles.len() {
        if i != j && vehicles[i].bounding_box_collision(&vehicles[j]) {
            has_overlap = true;
            break;
        }
    }
    
    // If there's overlap, revert to previous position
    if has_overlap {
        vehicles[i].x = prev_x;
        vehicles[i].y = prev_y;
        vehicles[i].stopped = prev_stopped;
    }
    
    // Remove vehicles that have left the screen
    if vehicles[i].x < -100 || vehicles[i].x > WINDOW_WIDTH as i32 + 100 ||
       vehicles[i].y < -100 || vehicles[i].y > WINDOW_HEIGHT as i32 + 100 {
        vehicles.remove(i);
    } else {
        i += 1;
    }
}
        
        // Render everything
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