mod behavior;
mod sandbox;
#[cfg(feature = "video-recording")]
mod video_recorder;

use pixels::wgpu::{PowerPreference, RequestAdapterOptions, Surface};
use pixels::{PixelsBuilder, SurfaceTexture};
use sandbox::{Particle, ParticleType, Sandbox, SIMULATION_HEIGHT, SIMULATION_WIDTH};
use std::time::{Duration, Instant};
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, WindowBuilder};

const TARGET_TIME_PER_UPDATE: Duration = Duration::from_nanos(16666670);

fn main() {
    // Setup winit
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Sandbox")
        .with_inner_size(LogicalSize::new(
            (SIMULATION_WIDTH * 2) as f64,
            (SIMULATION_HEIGHT * 2) as f64,
        ))
        .build(&event_loop)
        .unwrap();

    // Setup pixels
    let surface_size = window.inner_size();
    let surface = Surface::create(&window);
    let surface_texture = SurfaceTexture::new(surface_size.width, surface_size.height, surface);
    let mut pixels = PixelsBuilder::new(
        SIMULATION_WIDTH as u32,
        SIMULATION_HEIGHT as u32,
        surface_texture,
    )
    .request_adapter_options(RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        compatible_surface: None,
    })
    .build()
    .unwrap();

    // Setup the video recorder
    #[cfg(feature = "video-recording")]
    let mut video_recorder = video_recorder::VideoRecorder::new();

    // Simulation state
    let mut sandbox = Sandbox::new();
    let start_time = Instant::now();
    let mut last_update = Instant::now();
    let mut paused = false;
    let mut update_once = false;

    // Brush state
    let mut selected_particle = None;
    let mut brush_size = 3;
    let mut x_axis_locked = None;
    let mut y_axis_locked = None;

    // Particle placement state
    let mut should_place_particles = false;
    let mut particle_placement_queue = Vec::new();

    // Window state
    let mut last_resize = None;
    let mut prev_cursor_position = PhysicalPosition::<f64>::new(0.0, 0.0);
    let mut curr_cursor_position = PhysicalPosition::<f64>::new(0.0, 0.0);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                // Window events
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => {
                    last_resize = Some(Instant::now());
                    pixels.resize(new_size.width, new_size.height);
                }

                // Mouse events
                WindowEvent::CursorMoved { position, .. } => {
                    prev_cursor_position = curr_cursor_position;
                    curr_cursor_position = position;

                    if should_place_particles {
                        particle_placement_queue.push((prev_cursor_position, curr_cursor_position));
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
                            // Misc controls
                            Some(VirtualKeyCode::Escape) => *control_flow = ControlFlow::Exit,
                            Some(VirtualKeyCode::Return) => {
                                let fullscreen = match window.fullscreen() {
                                    Some(_) => None,
                                    None => Some(Fullscreen::Borderless(window.current_monitor())),
                                };
                                window.set_fullscreen(fullscreen);
                            }
                            Some(VirtualKeyCode::Back) => sandbox = Sandbox::new(),
                            Some(VirtualKeyCode::Space) => paused = !paused,
                            Some(VirtualKeyCode::Period) if paused => update_once = true,
                            Some(VirtualKeyCode::Equals) => {
                                if brush_size < 10 {
                                    brush_size += 1
                                }
                            }
                            Some(VirtualKeyCode::Minus) | Some(VirtualKeyCode::Subtract) => {
                                if brush_size > 1 {
                                    brush_size -= 1
                                }
                            }
                            #[cfg(feature = "video-recording")]
                            Some(VirtualKeyCode::Key1) => {
                                if video_recorder.is_recording {
                                    video_recorder.stop_recording();
                                } else {
                                    video_recorder.start_recording();
                                }
                            }

                            // Particle selection controls
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
                            Some(VirtualKeyCode::C) => {
                                selected_particle = Some(ParticleType::Cryotheum);
                            }
                            Some(VirtualKeyCode::U) => {
                                selected_particle = Some(ParticleType::Unstable);
                            }
                            Some(VirtualKeyCode::E) => {
                                selected_particle = Some(ParticleType::Electricity);
                            }
                            Some(VirtualKeyCode::L) => {
                                selected_particle = Some(ParticleType::Life);
                            }

                            _ => {}
                        }
                    }

                    #[cfg(feature = "video-recording")]
                    if video_recorder.is_recording && *control_flow == ControlFlow::Exit {
                        video_recorder.stop_recording();
                    }
                }

                _ => {}
            },

            Event::MainEventsCleared => {
                if let Some(lr) = last_resize {
                    // Prevent the window from becoming smaller than SIMULATION_SIZE
                    if lr.elapsed() >= Duration::from_millis(10) {
                        let mut surface_size = window.inner_size();
                        surface_size.width = surface_size.width.max(SIMULATION_WIDTH as u32);
                        surface_size.height = surface_size.height.max(SIMULATION_HEIGHT as u32);
                    }
                    // Snap the window size to multiples of SIMULATION_SIZE when less than 20% away
                    if lr.elapsed() >= Duration::from_millis(50) {
                        let mut surface_size = window.inner_size();
                        surface_size.width = surface_size.width.max(SIMULATION_WIDTH as u32);
                        surface_size.height = surface_size.height.max(SIMULATION_HEIGHT as u32);
                        let width_ratio = surface_size.width as f64 / SIMULATION_WIDTH as f64;
                        let height_ratio = surface_size.height as f64 / SIMULATION_HEIGHT as f64;
                        if (width_ratio.fract() < 0.20 || width_ratio.fract() > 0.80)
                            && (height_ratio.fract() < 0.20 || height_ratio.fract() > 0.80)
                        {
                            surface_size.width =
                                width_ratio.round() as u32 * SIMULATION_WIDTH as u32;
                            surface_size.height =
                                height_ratio.round() as u32 * SIMULATION_HEIGHT as u32;
                            window.set_inner_size(surface_size);
                            pixels.resize(surface_size.width, surface_size.height);
                        }
                        last_resize = None;
                    }
                }

                // Place particles in a straight line from prev_cursor_position to curr_cursor_position
                // In addition, uses data cached from WindowEvent::CursorMoved to ensure all gestures are properly captured
                if should_place_particles {
                    particle_placement_queue.push((prev_cursor_position, curr_cursor_position));
                }
                for (p1, mut p2) in particle_placement_queue.drain(..) {
                    // Adjust coordinates
                    if let Some(x) = x_axis_locked {
                        p2.x = x;
                    }
                    if let Some(y) = y_axis_locked {
                        p2.y = y;
                    }
                    let p1 = pixels
                        .window_pos_to_pixel(p1.into())
                        .unwrap_or_else(|p| pixels.clamp_pixel_pos(p));
                    let p2 = pixels
                        .window_pos_to_pixel(p2.into())
                        .unwrap_or_else(|p| pixels.clamp_pixel_pos(p));
                    let (p1x, p1y) = (p1.0 as f64, p1.1 as f64);
                    let (p2x, p2y) = (p2.0 as f64, p2.1 as f64);

                    // Place particles
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

                // Update the simulation
                if last_update.elapsed() >= TARGET_TIME_PER_UPDATE && (!paused || update_once) {
                    update_once = false;
                    last_update = Instant::now();
                    sandbox.update();
                }

                window.request_redraw();
            }

            Event::RedrawRequested(_) => {
                // Generate frame
                let frame = pixels.get_frame();
                sandbox.render(frame, start_time.elapsed().as_secs_f32() * 20.0);

                // Record frame to video
                #[cfg(feature = "video-recording")]
                video_recorder.upload_frame(frame);

                // Render frame to window
                let _ = pixels.render();
            }

            _ => {}
        }
    });
}
