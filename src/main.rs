mod behavior;
mod glow_post_process;
mod heap_array;
mod particle;
mod sandbox;
mod ui;
mod wayland_csd;

use crate::glow_post_process::GlowPostProcess;
use crate::particle::{Particle, ParticleType};
use crate::sandbox::{Sandbox, SANDBOX_HEIGHT, SANDBOX_WIDTH};
use crate::ui::UI;
use crate::wayland_csd::WaylandCSDTheme;
use pixels::wgpu::*;
use pixels::{PixelsBuilder, SurfaceTexture};
use puffin::profile_scope;
use std::time::{Duration, Instant};
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::unix::WindowExtUnix;
use winit::window::{Fullscreen, WindowBuilder};

const TARGET_TIME_PER_UPDATE: Duration = Duration::from_nanos(16666670);

fn main() {
    // Simulation state
    let mut sandbox = Sandbox::new();
    let mut last_update = Instant::now();
    let mut frame_time = Duration::from_secs(0);
    let mut paused = false;
    let mut update_once = false;

    // Brush state
    let mut selected_particle = Some(ParticleType::Sand);
    let mut brush_size: u8 = 3;
    let mut x_axis_locked = None;
    let mut y_axis_locked = None;

    // Particle placement state
    let mut should_place_particles = false;
    let mut particle_placement_queue = Vec::new();

    // Window state
    let mut last_resize = None;
    let mut prev_cursor_position = PhysicalPosition::<f64>::new(0.0, 0.0);
    let mut curr_cursor_position = PhysicalPosition::<f64>::new(0.0, 0.0);

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
    window.set_wayland_theme(WaylandCSDTheme { selected_particle });

    // Setup rendering
    let surface_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(surface_size.width, surface_size.height, &window);
    let mut pixels =
        PixelsBuilder::new(SANDBOX_WIDTH as u32, SANDBOX_HEIGHT as u32, surface_texture)
            .request_adapter_options(RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
            })
            .build()
            .expect("Failed to setup rendering");
    let mut glow_post_process =
        GlowPostProcess::new(pixels.device(), surface_size.width, surface_size.height);
    let mut ui = UI::new(&window, pixels.device(), pixels.queue());

    event_loop.run(move |event, _, control_flow| {
        match &event {
            Event::NewEvents(_) => ui.start_of_frame(),

            Event::WindowEvent { event, .. } => match event {
                // Window events
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => {
                    pixels.resize(new_size.width, new_size.height);
                    glow_post_process.resize(pixels.device(), new_size.width, new_size.height);
                    last_resize = Some(Instant::now());
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    ui.set_scale_factor(*scale_factor, pixels.device(), pixels.queue());
                }

                // Mouse events
                WindowEvent::CursorMoved { position, .. } => {
                    prev_cursor_position = curr_cursor_position;
                    curr_cursor_position = *position;

                    if should_place_particles {
                        particle_placement_queue.push((prev_cursor_position, curr_cursor_position));
                    }
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if *button == MouseButton::Left
                        && (!ui.ui_wants_mouse_input()
                            || x_axis_locked.is_some()
                            || y_axis_locked.is_some())
                    {
                        should_place_particles = *state == ElementState::Pressed;
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
                            Some(VirtualKeyCode::Minus) => {
                                if brush_size > 1 {
                                    brush_size -= 1
                                }
                            }
                            Some(VirtualKeyCode::Key1) => ui.toggle_display_ui(),
                            Some(VirtualKeyCode::Key2) => ui.toggle_display_fps(),
                            Some(VirtualKeyCode::Key3) => ui.toggle_display_profiler(),

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

                        window.set_wayland_theme(WaylandCSDTheme { selected_particle });
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
                        if selected_particle != Some(ParticleType::Electricity) {
                            p2.x = locked_x;
                        }
                    }
                    if let Some(locked_y) = y_axis_locked {
                        if selected_particle != Some(ParticleType::Electricity) {
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
                    let brush_size_x = brush_size as usize;
                    let brush_size_y = if selected_particle == Some(ParticleType::Electricity) {
                        1
                    } else {
                        brush_size as usize
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
                                                sandbox[x][y] = Some(Particle::new(
                                                    selected_particle,
                                                    &mut sandbox.rng,
                                                ));
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
                frame_time += last_update.elapsed();
                last_update = Instant::now();
                let mut updates = 0;
                let max_updates = if cfg!(debug_assertions) { 1 } else { 5 };
                while frame_time >= TARGET_TIME_PER_UPDATE && updates != max_updates {
                    if !paused || update_once {
                        update_once = false;
                        updates += 1;
                        sandbox.update();
                    }
                    frame_time -= TARGET_TIME_PER_UPDATE;
                }

                ui.prepare_render(&window);
                window.request_redraw();
            }

            // Render
            Event::RedrawRequested(_) => {
                profile_scope!("render");
                let has_glow = sandbox.render(pixels.get_frame());

                profile_scope!("render_gpu");
                let _ = pixels.render_with(|encoder, render_texture, context| {
                    let scaling_renderer = &context.scaling_renderer;
                    if has_glow {
                        scaling_renderer.render(encoder, &glow_post_process.texture1);
                        glow_post_process.render(encoder, render_texture);
                    } else {
                        scaling_renderer.render(encoder, render_texture);
                    }

                    ui.render(
                        &mut sandbox,
                        &mut selected_particle,
                        &mut brush_size,
                        &mut paused,
                        &window,
                        &context.device,
                        &context.queue,
                        encoder,
                        render_texture,
                    );
                });
            }

            _ => {}
        }

        ui.handle_event(&window, &event);
    });
}
