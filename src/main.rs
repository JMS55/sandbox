mod behavior;
mod sandbox;

use pixels::{wgpu::Surface, Pixels, SurfaceTexture};
use sandbox::{Particle, ParticleType, Sandbox};
use std::time::{Duration, Instant};
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const TARGET_TIME_PER_UPDATE: Duration = Duration::from_nanos(16666670);

fn main() {
    let mut sandbox = Sandbox::new(600, 400);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Sandbox")
        .with_inner_size(LogicalSize::new(
            sandbox.width as f64,
            sandbox.height as f64,
        ))
        .build(&event_loop)
        .unwrap();

    let surface_size = window.inner_size();
    let surface = Surface::create(&window);
    let surface_texture = SurfaceTexture::new(surface_size.width, surface_size.height, surface);
    let mut pixels =
        Pixels::new(sandbox.width as u32, sandbox.height as u32, surface_texture).unwrap();

    let mut last_update = Instant::now() - TARGET_TIME_PER_UPDATE;
    let mut paused = false;

    let mut selected_particle = None;
    let mut brush_size = 3;
    let mut should_place_particles = false;
    let mut x_axis_locked = None;
    let mut y_axis_locked = None;

    let mut dpi_factor = window.scale_factor();
    let mut prev_cursor_position = LogicalPosition::<f64>::new(0.0, 0.0);
    let mut curr_cursor_position = LogicalPosition::<f64>::new(0.0, 0.0);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                // Window events
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => {
                    let surface = Surface::create(&window);
                    let surface_texture =
                        SurfaceTexture::new(new_size.width, new_size.height, surface);
                    let new_size = new_size.to_logical(dpi_factor);
                    pixels = Pixels::new(new_size.width, new_size.height, surface_texture).unwrap();
                    sandbox.resize(new_size.width as usize, new_size.height as usize);
                }
                WindowEvent::ScaleFactorChanged {
                    scale_factor,
                    new_inner_size,
                } => {
                    dpi_factor = scale_factor;
                    pixels.resize(new_inner_size.width, new_inner_size.height);
                }

                // Mouse events
                WindowEvent::CursorMoved { position, .. } => {
                    prev_cursor_position = curr_cursor_position;
                    curr_cursor_position = position.to_logical(dpi_factor);

                    if should_place_particles {
                        // Convert prev_cursor_position and curr_cursor_position to sandbox coordinates
                        let p1 = prev_cursor_position;
                        let mut p2 = curr_cursor_position;
                        if let Some(x) = x_axis_locked {
                            p2.x = x;
                        }
                        if let Some(y) = y_axis_locked {
                            p2.y = y;
                        }
                        let p1x = clamp(p1.x, 0.0, sandbox.width as f64);
                        let p1y = clamp(p1.y, 0.0, sandbox.height as f64);
                        let p2x = clamp(p2.x, 0.0, sandbox.width as f64);
                        let p2y = clamp(p2.y, 0.0, sandbox.height as f64);

                        // Place particles in a straight line from prev_cursor_position to curr_cursor_position
                        let n = (p1x - p2y).abs().max((p1y - p2y).abs()) as usize;
                        for step in 0..(n + 1) {
                            let t = if n == 0 { 0.0 } else { step as f64 / n as f64 };
                            let x = (p1x + t * (p2x - p1x)).round() as usize;
                            let y = (p1y + t * (p2y - p1y)).round() as usize;
                            for x in x..(x + brush_size) {
                                for y in y..(y + brush_size) {
                                    if x < sandbox.width && y < sandbox.height {
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
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if button == MouseButton::Left {
                        should_place_particles = state == ElementState::Pressed;
                    }
                }

                // Keyboard events
                WindowEvent::ModifiersChanged(modifiers) => {
                    x_axis_locked = if modifiers.shift() {
                        Some(curr_cursor_position.x)
                    } else {
                        None
                    };
                    y_axis_locked = if modifiers.ctrl() {
                        Some(curr_cursor_position.y)
                    } else {
                        None
                    };
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed {
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::Escape) => *control_flow = ControlFlow::Exit,
                            Some(VirtualKeyCode::Back) => {
                                sandbox = Sandbox::new(sandbox.width, sandbox.height);
                            }
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
                            // Some(VirtualKeyCode::C) => {
                            //     selected_particle = Some(ParticleType::Cryotheum);
                            // }
                            Some(VirtualKeyCode::U) => {
                                selected_particle = Some(ParticleType::Unstable);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },

            Event::MainEventsCleared => {
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
