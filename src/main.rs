mod behavior;
mod sandbox;

#[cfg(feature = "video-recording")]
use {
    gstreamer::glib::object::{Cast, ObjectExt},
    gstreamer::{
        Buffer, ElementExt, ElementExtManual, Event as GStreamerEvent, Format, GstBinExt,
        MessageView, Pipeline, State, CLOCK_TIME_NONE,
    },
    gstreamer_app::AppSrc,
    gstreamer_video::{VideoFormat, VideoInfo},
    std::fs,
    std::time::SystemTime,
};

use pixels::wgpu::{PowerPreference, RequestAdapterOptions, Surface};
use pixels::{PixelsBuilder, SurfaceTexture};
use sandbox::{Particle, ParticleType, Sandbox, SIMULATION_HEIGHT, SIMULATION_WIDTH};
use std::time::{Duration, Instant};
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, Event as WinitEvent, MouseButton, VirtualKeyCode, WindowEvent};
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

    // Setup the video recording pipeline
    #[cfg(feature = "video-recording")]
    let (video_recording_pipeline, video_recording_src) = {
        gstreamer::init().unwrap();

        let video_recording_pipeline = gstreamer::parse_launch(
            "appsrc name=src is-live=true do-timestamp=true ! videoconvert ! x264enc ! mp4mux ! filesink name=filesink",
        )
        .unwrap()
        .downcast::<Pipeline>()
        .unwrap();

        let video_recording_src = video_recording_pipeline
            .get_by_name("src")
            .unwrap()
            .dynamic_cast::<AppSrc>()
            .unwrap();
        let video_info = VideoInfo::new(
            VideoFormat::Rgba,
            SIMULATION_WIDTH as u32,
            SIMULATION_HEIGHT as u32,
        )
        .build()
        .unwrap();
        video_recording_src.set_caps(Some(&video_info.to_caps().unwrap()));
        video_recording_src.set_property_format(Format::Time);

        (video_recording_pipeline, video_recording_src)
    };

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

    // Video recording state
    #[cfg(feature = "video-recording")]
    let (mut is_recording, filesink, mut video_file_location) = {
        let filesink = video_recording_pipeline.get_by_name("filesink").unwrap();

        let mut video_file_location = dirs_next::video_dir().unwrap();
        video_file_location.push("sandbox");
        let _ = fs::create_dir(&video_file_location);
        video_file_location.push("placeholder.mp4");

        (false, filesink, video_file_location)
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            WinitEvent::WindowEvent { event, .. } => match event {
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
                                is_recording = !is_recording;
                                if is_recording {
                                    video_file_location.set_file_name(&format!(
                                        "sandbox-{}.mp4",
                                        humantime::format_rfc3339_seconds(SystemTime::now())
                                    ));
                                    filesink
                                        .set_property(
                                            "location",
                                            &video_file_location.to_str().unwrap(),
                                        )
                                        .unwrap();
                                    video_recording_pipeline.set_state(State::Playing).unwrap();
                                } else {
                                    video_recording_pipeline
                                        .send_event(GStreamerEvent::new_eos().build());
                                    let bus = video_recording_pipeline.get_bus().unwrap();
                                    for message in bus.iter_timed(CLOCK_TIME_NONE) {
                                        match message.view() {
                                            MessageView::Eos(..) => break,
                                            _ => {}
                                        }
                                    }
                                    video_recording_pipeline.set_state(State::Null).unwrap();
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
                    if is_recording && *control_flow == ControlFlow::Exit {
                        is_recording = false;
                        video_recording_pipeline.send_event(GStreamerEvent::new_eos().build());
                        let bus = video_recording_pipeline.get_bus().unwrap();
                        for message in bus.iter_timed(CLOCK_TIME_NONE) {
                            match message.view() {
                                MessageView::Eos(..) => break,
                                _ => {}
                            }
                        }
                        video_recording_pipeline.set_state(State::Null).unwrap();
                    }
                }

                _ => {}
            },

            WinitEvent::MainEventsCleared => {
                // Snap the window size to multiples of SIMULATION_SIZE when less than 20% away
                if let Some(lr) = last_resize {
                    if lr.elapsed() >= Duration::from_millis(50) {
                        let mut surface_size = window.inner_size();
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

            WinitEvent::RedrawRequested(_) => {
                // Generate frame
                let frame = pixels.get_frame();
                sandbox.render(frame, start_time.elapsed().as_secs_f32() * 20.0);

                // Record frame to video
                #[cfg(feature = "video-recording")]
                if is_recording {
                    video_recording_src
                        .push_buffer(Buffer::from_slice(frame.to_vec()))
                        .unwrap();
                }

                // Render frame to window
                let _ = pixels.render();
            }

            _ => {}
        }
    });
}
