mod behavior;
mod glow_post_process;
mod heap_array;
mod particle;
mod sandbox;

use glow_post_process::GlowPostProcess;
use imgui::*;
use imgui_wgpu::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use particle::{Particle, ParticleType};
use pixels::wgpu::*;
use pixels::{PixelsBuilder, SurfaceTexture};
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
    let surface_size = window.inner_size();
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
    let mut texture_descriptor = TextureDescriptor {
        label: None,
        size: Extent3d {
            width: surface_size.width,
            height: surface_size.height,
            depth: 1,
        },
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Bgra8UnormSrgb,
        usage: TextureUsage::SAMPLED | TextureUsage::OUTPUT_ATTACHMENT,
    };
    let mut scaling_renderer_texture = pixels
        .device()
        .create_texture(&texture_descriptor)
        .create_default_view();
    let mut glow_post_process = GlowPostProcess::new(
        pixels.device(),
        &scaling_renderer_texture,
        surface_size.width,
        surface_size.height,
    );

    // Setup UI
    let mut imgui = Context::create();
    let mut imgui_platform = WinitPlatform::init(&mut imgui);
    imgui_platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);
    imgui.io_mut().font_global_scale = (1.0 / imgui_platform.hidpi_factor()) as f32;
    imgui.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../Inter-Medium.otf"),
        size_pixels: (16.0 * imgui_platform.hidpi_factor()) as f32,
        config: None,
    }]);
    imgui.set_ini_filename(None);
    let mut imgui_renderer = Renderer::new(
        &mut imgui,
        pixels.device(),
        pixels.queue(),
        TextureFormat::Bgra8UnormSrgb,
        None,
    );
    let mut recent_frames = [Instant::now(); 10];
    let mut was_paused = false;
    let mut should_display_ui = true;
    let mut should_display_fps = cfg!(debug_assertions);

    // Simulation state
    let mut sandbox = Sandbox::new();
    let mut last_update = Instant::now();
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

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match &event {
            Event::NewEvents(_) => {
                let now = imgui.io_mut().update_delta_time(recent_frames[9]);
                recent_frames = [
                    recent_frames[1],
                    recent_frames[2],
                    recent_frames[3],
                    recent_frames[4],
                    recent_frames[5],
                    recent_frames[6],
                    recent_frames[7],
                    recent_frames[8],
                    recent_frames[9],
                    now,
                ];
            }

            Event::WindowEvent { event, .. } => match event {
                // Window events
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => {
                    last_resize = Some(Instant::now());
                    pixels.resize(new_size.width, new_size.height);
                    texture_descriptor.size = Extent3d {
                        width: new_size.width,
                        height: new_size.height,
                        depth: 1,
                    };
                    scaling_renderer_texture = pixels
                        .device()
                        .create_texture(&texture_descriptor)
                        .create_default_view();
                    glow_post_process.resize(
                        pixels.device(),
                        &scaling_renderer_texture,
                        new_size.width,
                        new_size.height,
                    );
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    imgui.io_mut().font_global_scale = (1.0 / scale_factor) as f32;
                    imgui.fonts().clear_input_data();
                    imgui.fonts().add_font(&[FontSource::TtfData {
                        data: include_bytes!("../Inter-Medium.otf"),
                        size_pixels: (16.0 * scale_factor) as f32,
                        config: None,
                    }]);
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
                        && (!imgui.io().want_capture_mouse
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
                            Some(VirtualKeyCode::Minus) | Some(VirtualKeyCode::Subtract) => {
                                if brush_size > 1 {
                                    brush_size -= 1
                                }
                            }
                            Some(VirtualKeyCode::Key1) => should_display_ui = !should_display_ui,
                            Some(VirtualKeyCode::Key2) => should_display_fps = !should_display_fps,

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

                imgui_platform
                    .prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare UI frame");
                window.request_redraw();
            }

            // Render
            Event::RedrawRequested(_) => {
                let has_glow = sandbox.render(pixels.get_frame());

                let _ = pixels.render_with(|encoder, render_texture, context| {
                    let scaling_renderer = &context.scaling_renderer;
                    if has_glow {
                        scaling_renderer.render(encoder, &scaling_renderer_texture);
                        glow_post_process.render(encoder, render_texture);
                    } else {
                        scaling_renderer.render(encoder, render_texture);
                    }

                    let ui = imgui.frame();
                    let foreground_color1 = [
                        (230.0 / 255.0f32).powf(2.2),
                        (230.0 / 255.0f32).powf(2.2),
                        (230.0 / 255.0f32).powf(2.2),
                        0.95,
                    ];
                    let background_color1 = [
                        (92.0 / 255.0f32).powf(2.2),
                        (64.0 / 255.0f32).powf(2.2),
                        (38.0 / 255.0f32).powf(2.2),
                        0.95,
                    ];
                    let foreground_color2 = [
                        (80.0 / 255.0f32).powf(2.2),
                        (80.0 / 255.0f32).powf(2.2),
                        (80.0 / 255.0f32).powf(2.2),
                        0.95,
                    ];
                    let background_color2 = [
                        (60.0 / 255.0f32).powf(2.2),
                        (60.0 / 255.0f32).powf(2.2),
                        (60.0 / 255.0f32).powf(2.2),
                        0.95,
                    ];
                    let style1 = ui.push_style_colors(&[
                        (StyleColor::Button, background_color1),
                        (StyleColor::ButtonActive, background_color1),
                        (StyleColor::ButtonHovered, background_color1),
                        (StyleColor::CheckMark, foreground_color1),
                        (StyleColor::FrameBg, background_color1),
                        (StyleColor::FrameBgActive, background_color1),
                        (StyleColor::FrameBgHovered, background_color1),
                        (StyleColor::ScrollbarBg, background_color2),
                        (StyleColor::ScrollbarGrab, foreground_color2),
                        (StyleColor::ScrollbarGrabActive, foreground_color2),
                        (StyleColor::ScrollbarGrabHovered, foreground_color2),
                        (StyleColor::SliderGrab, foreground_color1),
                        (StyleColor::SliderGrabActive, foreground_color1),
                    ]);
                    let mut style2 = Some(ui.push_style_vars(&[
                        StyleVar::FrameRounding(4.0),
                        StyleVar::WindowPadding([0.0, 0.0]),
                        StyleVar::WindowMinSize([1.0, 1.0]),
                    ]));

                    Window::new(im_str!("toggle_ui_window"))
                        .position([10.0, 27.0], Condition::Always)
                        .title_bar(false)
                        .draw_background(false)
                        .movable(false)
                        .resizable(false)
                        .build(&ui, || {
                            ui.checkbox(im_str!("Toggle UI"), &mut should_display_ui);
                        });

                    if should_display_ui {
                        let mut widget_x = 0.0;
                        let mut particle_selector_button =
                            |text: &ImStr,
                             ptype: Option<ParticleType>,
                             color: [f32; 3],
                             white_text: bool| {
                                ui.set_cursor_pos([
                                    widget_x,
                                    if ptype == selected_particle { 0.0 } else { 8.0 },
                                ]);
                                widget_x += if ptype == selected_particle {
                                    107.0
                                } else {
                                    92.0
                                };

                                let button_color = [
                                    color[0].powf(2.2),
                                    color[1].powf(2.2),
                                    color[2].powf(2.2),
                                    0.95,
                                ];
                                let style1 = ui.push_style_colors(&[
                                    (StyleColor::Button, button_color),
                                    (StyleColor::ButtonHovered, button_color),
                                    (StyleColor::ButtonActive, button_color),
                                    (
                                        StyleColor::Text,
                                        if white_text {
                                            [0.8, 0.8, 0.8, 1.0]
                                        } else {
                                            [0.0, 0.0, 0.0, 1.0]
                                        },
                                    ),
                                ]);
                                let style2 = ui.push_style_var(StyleVar::FrameRounding(6.0));
                                let size = if ptype == selected_particle {
                                    [100.0, 55.0]
                                } else {
                                    [85.0, 40.0]
                                };
                                if ui.button(text, size) {
                                    selected_particle = ptype;
                                }
                                style1.pop(&ui);
                                style2.pop(&ui);
                            };

                        let window_width =
                            window.inner_size().width as f32 / window.scale_factor() as f32;
                        Window::new(im_str!("particle_selection_window"))
                            .position([107.0, 10.0], Condition::Always)
                            .size([window_width - 170.0, 67.0], Condition::Always)
                            .title_bar(false)
                            .draw_background(false)
                            .movable(false)
                            .resizable(false)
                            .horizontal_scrollbar(true)
                            .build(&ui, || {
                                particle_selector_button(
                                    im_str!("Delete Tool"),
                                    None,
                                    [0.1, 0.1, 0.1],
                                    true,
                                );
                                particle_selector_button(
                                    im_str!("Sand"),
                                    Some(ParticleType::Sand),
                                    [196.0 / 255.0, 192.0 / 255.0, 135.0 / 255.0],
                                    false,
                                );
                                particle_selector_button(
                                    im_str!("Water"),
                                    Some(ParticleType::Water),
                                    [26.0 / 255.0, 91.0 / 255.0, 165.0 / 255.0],
                                    true,
                                );
                                particle_selector_button(
                                    im_str!("Acid"),
                                    Some(ParticleType::Acid),
                                    [148.0 / 255.0, 219.0 / 255.0, 10.0 / 255.0],
                                    false,
                                );
                                particle_selector_button(
                                    im_str!("Iridium"),
                                    Some(ParticleType::Iridium),
                                    [100.0 / 255.0, 100.0 / 255.0, 100.0 / 255.0],
                                    true,
                                );
                                particle_selector_button(
                                    im_str!("Replicator"),
                                    Some(ParticleType::Replicator),
                                    [78.0 / 255.0, 21.0 / 255.0, 77.0 / 255.0],
                                    true,
                                );
                                particle_selector_button(
                                    im_str!("Plant"),
                                    Some(ParticleType::Plant),
                                    [6.0 / 255.0, 89.0 / 255.0, 9.0 / 255.0],
                                    true,
                                );
                                particle_selector_button(
                                    im_str!("Cryotheum"),
                                    Some(ParticleType::Cryotheum),
                                    [12.0 / 255.0, 193.0 / 255.0, 255.0 / 255.0],
                                    false,
                                );
                                particle_selector_button(
                                    im_str!("Unstable"),
                                    Some(ParticleType::Unstable),
                                    [94.0 / 255.0, 78.0 / 255.0, 55.0 / 255.0],
                                    true,
                                );
                                particle_selector_button(
                                    im_str!("Electricity"),
                                    Some(ParticleType::Electricity),
                                    [255.0 / 255.0, 244.0 / 255.0, 49.0 / 255.0],
                                    false,
                                );
                                particle_selector_button(
                                    im_str!("Life"),
                                    Some(ParticleType::Life),
                                    [135.0 / 255.0, 12.0 / 255.0, 211.0 / 255.0],
                                    true,
                                );
                                particle_selector_button(
                                    im_str!("Fire"),
                                    Some(ParticleType::Fire),
                                    [255.0 / 255.0, 151.0 / 255.0, 20.0 / 255.0],
                                    false,
                                );
                                particle_selector_button(
                                    im_str!("Mirror"),
                                    Some(ParticleType::Mirror),
                                    [78.0 / 255.0, 216.0 / 255.0, 131.0 / 255.0],
                                    false,
                                );
                                particle_selector_button(
                                    im_str!("Glitch"),
                                    Some(ParticleType::Glitch),
                                    [120.0 / 255.0, 119.0 / 255.0, 100.0 / 255.0],
                                    false,
                                );
                            });

                        Window::new(im_str!("second_row_window"))
                            .always_auto_resize(true)
                            .position([10.0, 88.0], Condition::Always)
                            .title_bar(false)
                            .draw_background(false)
                            .movable(false)
                            .resizable(false)
                            .build(&ui, || {
                                ui.set_cursor_pos([0.0, 3.0]);
                                ui.checkbox(im_str!("Paused"), &mut paused);
                                ui.set_cursor_pos([97.0, 0.0]);
                                if ui.button(im_str!("Empty Sandbox"), [125.0, 27.0]) {
                                    was_paused = paused;
                                    paused = true;
                                    ui.open_popup(im_str!("empty_sandbox_popup"));
                                }
                                style2.take().unwrap().pop(&ui);
                                ui.popup_modal(im_str!("empty_sandbox_popup"))
                                    .title_bar(false)
                                    .movable(false)
                                    .resizable(false)
                                    .build(|| {
                                        ui.text("Empty Sandbox?");
                                        if ui.button(im_str!("Yes"), [60.0, 30.0]) {
                                            sandbox.empty_out();
                                            ui.close_current_popup();
                                            paused = was_paused;
                                        }
                                        ui.same_line(0.0);
                                        if ui.button(im_str!("No"), [60.0, 30.0]) {
                                            ui.close_current_popup();
                                            paused = was_paused;
                                        }
                                    });
                                style2 = Some(ui.push_style_vars(&[
                                    StyleVar::FrameRounding(4.0),
                                    StyleVar::WindowPadding([0.0, 0.0]),
                                    StyleVar::WindowMinSize([1.0, 1.0]),
                                ]));
                                ui.set_cursor_pos([232.0, 3.0]);
                                Slider::new(im_str!("Brush Size"), 1..=10)
                                    .build(&ui, &mut brush_size);
                            });
                    }

                    if should_display_fps {
                        let height =
                            window.inner_size().height as f32 / window.scale_factor() as f32;
                        let y = height - 26.0;
                        let fps =
                            recent_frames.len() as f64 / recent_frames[0].elapsed().as_secs_f64();
                        Window::new(im_str!("fps_window"))
                            .always_auto_resize(true)
                            .position([10.0, y], Condition::Always)
                            .title_bar(false)
                            .draw_background(false)
                            .movable(false)
                            .resizable(false)
                            .no_inputs()
                            .build(&ui, || ui.text(format!("FPS: {:.0}", fps)));
                    }

                    style1.pop(&ui);
                    style2.unwrap().pop(&ui);

                    imgui_platform.prepare_render(&ui, &window);
                    imgui_renderer
                        .render(ui.render(), &context.device, encoder, render_texture)
                        .expect("Failed to render UI");
                });
            }

            _ => {}
        }

        imgui_platform.handle_event(imgui.io_mut(), &window, &event);
    });
}
