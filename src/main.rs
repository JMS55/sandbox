mod sandbox;

use pixels::{wgpu::Surface, Pixels, SurfaceTexture};
use sandbox::{Particle, ParticleType, Sandbox};
use std::time::{Duration, Instant};
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub const SIMULATION_WIDTH: usize = 600;
pub const SIMULATION_HEIGHT: usize = 400;
const DISPLAY_WIDTH: f64 = SIMULATION_WIDTH as f64 * 2.0;
const DISPLAY_HEIGHT: f64 = SIMULATION_HEIGHT as f64 * 2.0;
const TARGET_TIME_PER_UPDATE: Duration = Duration::from_nanos(16666670);

fn main() {
    let mut sandbox = Sandbox::new();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Sandbox")
        .with_inner_size(LogicalSize::new(DISPLAY_WIDTH, DISPLAY_HEIGHT))
        .build(&event_loop)
        .unwrap();
    let screen_size = window.inner_size();

    let surface = Surface::create(&window);
    let surface_texture = SurfaceTexture::new(screen_size.width, screen_size.height, surface);
    let mut pixels = Pixels::new(
        SIMULATION_WIDTH as u32,
        SIMULATION_HEIGHT as u32,
        surface_texture,
    )
    .unwrap();

    let mut last_update = Instant::now() - TARGET_TIME_PER_UPDATE;
    let mut paused = false;

    let mut selected_particle = None;
    let mut brush_size = 3;
    let mut should_place_particles = false;

    let mut hidpi_factor = window.scale_factor();
    let mut prev_cursor_position = LogicalPosition::<f64>::new(0.0, 0.0);
    let mut curr_cursor_position = LogicalPosition::<f64>::new(0.0, 0.0);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::ScaleFactorChanged {
                    scale_factor,
                    new_inner_size,
                } => {
                    hidpi_factor = scale_factor;
                    pixels.resize(new_inner_size.width, new_inner_size.height);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    prev_cursor_position = curr_cursor_position;
                    curr_cursor_position = position.to_logical(hidpi_factor);
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if button == MouseButton::Left {
                        should_place_particles = state == ElementState::Pressed;
                    }
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed {
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::Escape) => *control_flow = ControlFlow::Exit,
                            Some(VirtualKeyCode::Back) => sandbox = Sandbox::new(),
                            Some(VirtualKeyCode::Space) => paused = !paused,
                            Some(VirtualKeyCode::Equals) => {
                                if brush_size < 10 {
                                    brush_size += 1
                                }
                            }
                            Some(VirtualKeyCode::Minus) => {
                                if brush_size > 1 {
                                    brush_size -= 1
                                }
                            }
                            Some(VirtualKeyCode::D) => {
                                selected_particle = None;
                            }
                            Some(VirtualKeyCode::S) => {
                                selected_particle = Some(ParticleType::Sand);
                            }
                            Some(VirtualKeyCode::W) => {
                                selected_particle = Some(ParticleType::Water);
                            }
                            Some(VirtualKeyCode::A) => {
                                selected_particle = Some(ParticleType::Acid);
                            }
                            Some(VirtualKeyCode::I) => {
                                selected_particle = Some(ParticleType::Iridium);
                            }
                            Some(VirtualKeyCode::R) => {
                                selected_particle = Some(ParticleType::Replicator);
                            }
                            Some(VirtualKeyCode::P) => {
                                selected_particle = Some(ParticleType::Plant);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },

            Event::MainEventsCleared => {
                if should_place_particles {
                    let p1x = clamp(prev_cursor_position.x, 0.0, DISPLAY_WIDTH - 2.0) / 2.0;
                    let p1y = clamp(prev_cursor_position.y, 0.0, DISPLAY_WIDTH - 2.0) / 2.0;
                    let p2x = clamp(curr_cursor_position.x, 0.0, DISPLAY_WIDTH - 2.0) / 2.0;
                    let p2y = clamp(curr_cursor_position.y, 0.0, DISPLAY_WIDTH - 2.0) / 2.0;
                    let n = (p1x - p2y).abs().max((p1y - p2y).abs()) as usize;
                    for step in 0..(n + 1) {
                        let t = if n == 0 { 0.0 } else { step as f64 / n as f64 };
                        let x = (p1x + t * (p2x - p1x)).round() as usize;
                        let y = (p1y + t * (p2y - p1y)).round() as usize;
                        for x in x..(x + brush_size) {
                            for y in y..(y + brush_size) {
                                if x < SIMULATION_WIDTH && y < SIMULATION_HEIGHT {
                                    match selected_particle {
                                        Some(selected_particle) => {
                                            if sandbox.cells[x][y].is_none() {
                                                sandbox.cells[x][y] =
                                                    Some(Particle::new(selected_particle));
                                            }
                                        }
                                        None => sandbox.cells[x][y] = None,
                                    }
                                }
                            }
                        }
                    }
                }

                if last_update.elapsed() >= TARGET_TIME_PER_UPDATE && !paused {
                    last_update = Instant::now();
                    sandbox.update();
                }

                window.request_redraw();
            }

            Event::RedrawRequested(_) => {
                sandbox.render(pixels.get_frame());
                pixels.render().unwrap();
            }
            _ => {}
        }
    });
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    assert!(min <= max);
    let mut x = value;
    if x < min {
        x = min;
    }
    if x > max {
        x = max;
    }
    x
}
