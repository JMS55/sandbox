mod behavior;
mod heap_array;
mod particle;
mod render;
mod sandbox;
#[cfg(feature = "video-recording")]
mod video_recorder;

use particle::{Particle, ParticleType};
use pixels::wgpu::*;
use pixels::{PixelsBuilder, SurfaceTexture};
use render::Render;
use sandbox::{Sandbox, SANDBOX_HEIGHT, SANDBOX_WIDTH};
use std::time::{Duration, Instant};
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, WindowBuilder};

const TARGET_TIME_PER_UPDATE: Duration = Duration::from_nanos(16666670);

fn main() {
    #[cfg(target_os = "linux")]
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    // Setup windowing
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Sandbox")
        .with_inner_size(LogicalSize::new(
            (SANDBOX_WIDTH * 3) as f64,
            (SANDBOX_HEIGHT * 3) as f64,
        ))
        .build(&event_loop)
        .expect("Failed to create a window");

    // Setup rendering
    #[allow(unused_assignments)]
    let mut surface_size = window.inner_size();
    let surface = Surface::create(&window);
    let surface_texture = SurfaceTexture::new(surface_size.width, surface_size.height, surface);
    let mut pixels =
        PixelsBuilder::new(SANDBOX_WIDTH as u32, SANDBOX_HEIGHT as u32, surface_texture)
            .request_adapter_options(RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
            })
            .build()
            .expect("Failed to setup rendering");
    let mut render = Render::new(&pixels.device(), surface_size.width, surface_size.height);

    // Setup the video recorder
    #[cfg(feature = "video-recording")]
    let mut video_recorder = video_recorder::VideoRecorder::new(pixels.device())
        .map_err(|error| eprintln!("Warning: Video recording disabled: {}", error))
        .ok();

    // Simulation state
    let mut sandbox = Sandbox::new();
    let mut last_update = Instant::now();
    let mut paused = false;
    let mut update_once = false;

    // Brush state
    let mut selected_particle = Some(ParticleType::Sand);
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
                    surface_size = new_size;
                    pixels.resize(surface_size.width, surface_size.height);
                    render.resize(&pixels.device(), surface_size.width, surface_size.height);
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
                            Some(VirtualKeyCode::Back) => sandbox.empty_out(),
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
                                if let Some(video_recorder) = video_recorder.as_mut() {
                                    if video_recorder.is_recording {
                                        video_recorder.stop_recording();
                                    } else {
                                        video_recorder.start_recording();
                                    }
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
                            Some(VirtualKeyCode::F) => {
                                selected_particle = Some(ParticleType::Fire);
                            }
                            Some(VirtualKeyCode::M) => {
                                selected_particle = Some(ParticleType::Mirror);
                            }
                            Some(VirtualKeyCode::G) => {
                                selected_particle = Some(ParticleType::Glitch);
                            }
                            _ => {}
                        }
                    }
                }

                _ => {}
            },

            Event::MainEventsCleared => {
                if let Some(lr) = last_resize {
                    // Prevent the window from becoming smaller than SIMULATION_SIZE
                    if lr.elapsed() >= Duration::from_millis(10) {
                        let mut surface_size = window.inner_size();
                        surface_size.width = surface_size.width.max(SANDBOX_WIDTH as u32);
                        surface_size.height = surface_size.height.max(SANDBOX_HEIGHT as u32);
                    }
                    // Snap the window size to multiples of SIMULATION_SIZE when less than 20% away
                    if lr.elapsed() >= Duration::from_millis(50) {
                        let mut surface_size = window.inner_size();
                        surface_size.width = surface_size.width.max(SANDBOX_WIDTH as u32);
                        surface_size.height = surface_size.height.max(SANDBOX_HEIGHT as u32);
                        let width_ratio = surface_size.width as f64 / SANDBOX_WIDTH as f64;
                        let height_ratio = surface_size.height as f64 / SANDBOX_HEIGHT as f64;
                        if (width_ratio.fract() < 0.20 || width_ratio.fract() > 0.80)
                            && (height_ratio.fract() < 0.20 || height_ratio.fract() > 0.80)
                        {
                            surface_size.width = width_ratio.round() as u32 * SANDBOX_WIDTH as u32;
                            surface_size.height =
                                height_ratio.round() as u32 * SANDBOX_HEIGHT as u32;
                            window.set_inner_size(surface_size);
                            pixels.resize(surface_size.width, surface_size.height);
                        }
                        last_resize = None;
                    }
                }

                // Place particles in a straight line from prev_cursor_position to curr_cursor_position
                // In addition, use data cached from WindowEvent::CursorMoved to ensure all gestures are properly captured
                if should_place_particles {
                    particle_placement_queue.push((prev_cursor_position, curr_cursor_position));
                }
                for (p1, mut p2) in particle_placement_queue.drain(..) {
                    // Adjust coordinates
                    if let Some(locked_x) = x_axis_locked {
                        if selected_particle != Some(ParticleType::Electricity)
                            && selected_particle != Some(ParticleType::Fire)
                        {
                            p2.x = locked_x;
                        }
                    }
                    if let Some(locked_y) = y_axis_locked {
                        if selected_particle != Some(ParticleType::Electricity)
                            && selected_particle != Some(ParticleType::Fire)
                        {
                            p2.y = locked_y;
                        }
                    }
                    let (p1x, p1y) = pixels
                        .window_pos_to_pixel(p1.into())
                        .unwrap_or_else(|p| pixels.clamp_pixel_pos(p));
                    let (p2x, p2y) = pixels
                        .window_pos_to_pixel(p2.into())
                        .unwrap_or_else(|p| pixels.clamp_pixel_pos(p));

                    // Don't place multiple Electricity vertically
                    let brush_size_x = brush_size;
                    let brush_size_y = if selected_particle == Some(ParticleType::Electricity) {
                        1
                    } else {
                        brush_size
                    };

                    // Place particles (Bresenham's line algorithm)
                    let (mut p1x, mut p1y) = (p1x as isize, p1y as isize);
                    let (p2x, p2y) = (p2x as isize, p2y as isize);
                    let dx = (p2x - p1x).abs();
                    let sx = if p1x < p2x { 1 } else { -1 };
                    let dy = -(p2y - p1y).abs();
                    let sy = if p1y < p2y { 1 } else { -1 };
                    let mut err = dx + dy;
                    loop {
                        let (x, y) = (p1x as usize, p1y as usize);
                        for x in x..(x + brush_size_x) {
                            for y in y..(y + brush_size_y) {
                                if x < SANDBOX_WIDTH && y < SANDBOX_HEIGHT {
                                    match selected_particle {
                                        Some(selected_particle) => {
                                            if sandbox[x][y].is_none() {
                                                sandbox[x][y] =
                                                    Some(Particle::new(selected_particle));
                                            }
                                        }
                                        None => sandbox[x][y] = None,
                                    }
                                }
                            }
                        }

                        if p1x == p2x && p1y == p2y {
                            break;
                        }
                        let e2 = 2 * err;
                        if e2 >= dy {
                            err += dy;
                            p1x += sx;
                        }
                        if e2 <= dx {
                            err += dx;
                            p1y += sy;
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
                sandbox.render(pixels.get_frame());

                #[cfg(feature = "video-recording")]
                let buffer = video_recorder
                    .as_ref()
                    .filter(|video_recorder| video_recorder.is_recording)
                    .map(|_| {
                        pixels.device().create_buffer(&BufferDescriptor {
                            size: (surface_size.width * surface_size.height * 4) as u64,
                            usage: BufferUsage::COPY_DST | BufferUsage::MAP_READ,
                            label: None,
                        })
                    });

                let _ = pixels.render_with(|encoder, render_texture, scaling_renderer| {
                    scaling_renderer.render(encoder, &render.screen_sized_texture_view);
                    render.glow_post_process(encoder);
                    render.copy_screen_texture_to_swapchain(encoder, render_texture);

                    #[cfg(feature = "video-recording")]
                    if let Some(buffer) = &buffer {
                        encoder.copy_texture_to_buffer(
                            TextureCopyView {
                                texture: &render.screen_sized_texture,
                                mip_level: 0,
                                array_layer: 0,
                                origin: Origin3d::ZERO,
                            },
                            BufferCopyView {
                                buffer: &buffer,
                                offset: 0,
                                bytes_per_row: (surface_size.width * 4 + 255) & !255,
                                rows_per_image: 0,
                            },
                            Extent3d {
                                width: surface_size.width,
                                height: surface_size.height,
                                depth: 1,
                            },
                        );
                    }
                });

                #[cfg(feature = "video-recording")]
                if let Some(buffer) = buffer {
                    video_recorder.as_mut().unwrap().upload_buffer(
                        buffer,
                        surface_size.width,
                        surface_size.height,
                    );
                }
            }

            _ => {}
        }
    });
}
