mod behavior;
mod game;
mod glow_post_process;
mod heap_array;
mod particle;
mod sandbox;
mod ui;
#[cfg(target_os = "linux")]
mod wayland_csd;

use crate::glow_post_process::GlowPostProcess;
use crate::particle::ParticleType;
use crate::sandbox::{SANDBOX_HEIGHT, SANDBOX_WIDTH};
use crate::ui::UI;
#[cfg(target_os = "linux")]
use crate::wayland_csd::WaylandCSDTheme;
use game::Game;
use pixels::wgpu::*;
use pixels::{PixelsBuilder, SurfaceTexture};
use puffin::profile_scope;
use std::time::{Duration, Instant};
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
#[cfg(target_os = "linux")]
use winit::platform::unix::WindowExtUnix;
use winit::window::{Fullscreen, WindowBuilder};

fn main() {
    // Setup game
    let mut game = Game::new();

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
    #[cfg(target_os = "linux")]
    window.set_wayland_theme(WaylandCSDTheme::new(game.selected_particle));

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
            Event::NewEvents(_) => {
                game.start_of_frame();
                ui.start_of_frame();
            }

            Event::WindowEvent { event, .. } => match event {
                // Window events
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => {
                    pixels.resize(new_size.width, new_size.height);
                    glow_post_process.resize(pixels.device(), new_size.width, new_size.height);
                    game.last_window_resize = Some(Instant::now());
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    ui.set_scale_factor(*scale_factor, pixels.device(), pixels.queue());
                }

                // Mouse events
                WindowEvent::CursorMoved { position, .. } => game.cursor_moved(*position),
                WindowEvent::MouseInput { button, state, .. } => {
                    if *button == MouseButton::Left
                        && (!ui.ui_wants_mouse_input()
                            || game.x_axis_locked.is_some()
                            || game.y_axis_locked.is_some())
                    {
                        game.should_place_particles = *state == ElementState::Pressed;
                    }
                }

                // Keyboard events
                WindowEvent::ModifiersChanged(modifiers) => {
                    game.x_axis_locked = if modifiers.shift() {
                        Some(game.cursor_position.x)
                    } else {
                        None
                    };
                    game.y_axis_locked = if modifiers.ctrl() {
                        Some(game.cursor_position.y)
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
                            Some(VirtualKeyCode::Back) => game.sandbox.empty_out(),
                            Some(VirtualKeyCode::Space) => game.is_paused = !game.is_paused,
                            Some(VirtualKeyCode::Period) if game.is_paused => {
                                game.should_update_once = true;
                            }
                            Some(VirtualKeyCode::Equals) => {
                                if game.brush_size < 10 {
                                    game.brush_size += 1
                                }
                            }
                            Some(VirtualKeyCode::Minus) => {
                                if game.brush_size > 1 {
                                    game.brush_size -= 1
                                }
                            }
                            Some(VirtualKeyCode::Key1) => ui.toggle_display_ui(),
                            Some(VirtualKeyCode::Key2) => ui.toggle_display_fps(),
                            Some(VirtualKeyCode::Key3) => ui.toggle_display_profiler(),

                            // Particle selection controls
                            Some(VirtualKeyCode::D) => {
                                game.selected_particle = None;
                            }
                            Some(VirtualKeyCode::S) => {
                                game.selected_particle = Some(ParticleType::Sand);
                            }
                            Some(VirtualKeyCode::W) => {
                                game.selected_particle = Some(ParticleType::Water);
                            }
                            Some(VirtualKeyCode::A) => {
                                game.selected_particle = Some(ParticleType::Acid);
                            }
                            Some(VirtualKeyCode::I) => {
                                game.selected_particle = Some(ParticleType::Iridium);
                            }
                            Some(VirtualKeyCode::R) => {
                                game.selected_particle = Some(ParticleType::Replicator);
                            }
                            Some(VirtualKeyCode::P) => {
                                game.selected_particle = Some(ParticleType::Plant);
                            }
                            Some(VirtualKeyCode::C) => {
                                game.selected_particle = Some(ParticleType::Cryotheum);
                            }
                            Some(VirtualKeyCode::U) => {
                                game.selected_particle = Some(ParticleType::Unstable);
                            }
                            Some(VirtualKeyCode::E) => {
                                game.selected_particle = Some(ParticleType::Electricity);
                            }
                            Some(VirtualKeyCode::L) => {
                                game.selected_particle = Some(ParticleType::Life);
                            }
                            Some(VirtualKeyCode::F) => {
                                game.selected_particle = Some(ParticleType::Fire);
                            }
                            Some(VirtualKeyCode::M) => {
                                game.selected_particle = Some(ParticleType::Mirror);
                            }
                            Some(VirtualKeyCode::G) => {
                                game.selected_particle = Some(ParticleType::Glitch);
                            }
                            _ => {}
                        }

                        #[cfg(target_os = "linux")]
                        window.set_wayland_theme(WaylandCSDTheme::new(game.selected_particle));
                    }
                }

                _ => {}
            },

            Event::MainEventsCleared => {
                // Resize window if scheduled
                if let Some(last_window_resize) = game.last_window_resize {
                    // Prevent the window from becoming smaller than SIMULATION_SIZE
                    if last_window_resize.elapsed() >= Duration::from_millis(10) {
                        let mut surface_size = window.inner_size();
                        surface_size.width = surface_size.width.max(SANDBOX_WIDTH as u32);
                        surface_size.height = surface_size.height.max(SANDBOX_HEIGHT as u32);
                    }
                    // Snap the window size to multiples of SIMULATION_SIZE when less than 20% away
                    if last_window_resize.elapsed() >= Duration::from_millis(50) {
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
                        game.last_window_resize = None;
                    }
                }

                // Place particles
                game.place_particles(&pixels);

                // Update the simulation
                game.update();

                // Request render
                ui.prepare_render(&window);
                window.request_redraw();
            }

            // Render
            Event::RedrawRequested(_) => {
                profile_scope!("render");
                let has_glow = game.sandbox.render(pixels.get_frame());

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
                        &mut game.sandbox,
                        &mut game.selected_particle,
                        &mut game.brush_size,
                        &mut game.is_paused,
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
