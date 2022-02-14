mod behavior;
mod game;
mod glow_post_process;
mod heap_array;
mod particle;
mod sandbox;
mod ui;

use crate::glow_post_process::GlowPostProcess;
use crate::particle::ParticleType;
use crate::sandbox::{SANDBOX_HEIGHT, SANDBOX_WIDTH};
use crate::ui::UI;
use game::Game;
use pixels::wgpu::BlendState;
use pixels::{PixelsBuilder, SurfaceTexture};
use puffin::profile_scope;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, Window, WindowBuilder};

fn main() {
    // Setup game
    let mut game = Game::new();
    let mut last_update = Instant::now();

    // Setup windowing
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Sandbox")
        .with_inner_size(LogicalSize::new(
            (SANDBOX_WIDTH * 3) as f64,
            (SANDBOX_HEIGHT * 3) as f64,
        ))
        .with_min_inner_size(LogicalSize::new(
            SANDBOX_WIDTH as f64,
            SANDBOX_HEIGHT as f64,
        ))
        .build(&event_loop)
        .expect("Failed to create a window");

    // Setup rendering
    let surface_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(surface_size.width, surface_size.height, &window);
    let mut pixels =
        PixelsBuilder::new(SANDBOX_WIDTH as u32, SANDBOX_HEIGHT as u32, surface_texture)
            .blend_state(BlendState::REPLACE)
            .build()
            .expect("Failed to setup rendering");
    let mut glow_post_process =
        GlowPostProcess::new(pixels.device(), surface_size.width, surface_size.height);
    let mut ui = UI::new(&window, pixels.device(), pixels.queue());

    // Handle events
    event_loop.run(move |event, _, control_flow| {
        match &event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                let time_since_last_frame = last_update.elapsed();
                game.frame_time += time_since_last_frame;
                ui.start_of_frame(time_since_last_frame);
                last_update = now;
            }

            Event::WindowEvent { event, .. } => match event {
                // Window events
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => {
                    pixels.resize_surface(new_size.width, new_size.height);
                    glow_post_process.resize(pixels.device(), new_size.width, new_size.height);
                    game.last_window_resize = Some(Instant::now());
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    ui.set_scale_factor(*scale_factor, pixels.device(), pixels.queue());
                }

                // Mouse events
                WindowEvent::CursorMoved { position, .. } => game.handle_cursor_move(*position),
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
                        handle_key_press(
                            &input.virtual_keycode,
                            &window,
                            control_flow,
                            &mut game,
                            &mut ui,
                        );
                    }
                }

                _ => {}
            },

            Event::MainEventsCleared => {
                // Update game state
                game.handle_window_resize(&window, &mut pixels, &mut glow_post_process);
                game.place_queued_particles(&pixels);
                game.update();

                // Prepare render
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

                    Ok(())
                });
            }

            _ => {}
        }

        ui.handle_event(&window, &event);
    });
}

fn handle_key_press(
    keycode: &Option<VirtualKeyCode>,
    window: &Window,
    control_flow: &mut ControlFlow,
    game: &mut Game,
    ui: &mut UI,
) {
    match keycode {
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
}
