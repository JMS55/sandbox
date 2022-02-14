use crate::glow_post_process::GlowPostProcess;
use crate::particle::{Particle, ParticleType};
use crate::sandbox::{Sandbox, SANDBOX_HEIGHT, SANDBOX_WIDTH};
use pixels::Pixels;
use std::time::{Duration, Instant};
use winit::dpi::PhysicalPosition;
use winit::window::Window;

const TARGET_TIME_PER_UPDATE: Duration = Duration::from_nanos(16666670);

pub struct Game {
    pub sandbox: Sandbox,

    // Update timing info
    pub frame_time: Duration,
    pub is_paused: bool,
    pub should_update_once: bool,

    // Particle placement info
    pub selected_particle: Option<ParticleType>,
    pub brush_size: u8,
    pub x_axis_locked: Option<f64>,
    pub y_axis_locked: Option<f64>,
    pub should_place_particles: bool,
    pub cursor_position: PhysicalPosition<f64>,
    pub previous_cursor_position: PhysicalPosition<f64>,
    pub particle_placement_queue: Vec<(PhysicalPosition<f64>, PhysicalPosition<f64>)>,

    pub last_window_resize: Option<Instant>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            sandbox: Sandbox::new(),

            frame_time: Duration::from_secs(0),
            is_paused: false,
            should_update_once: false,

            selected_particle: Some(ParticleType::Sand),
            brush_size: 3,
            x_axis_locked: None,
            y_axis_locked: None,
            should_place_particles: false,
            cursor_position: PhysicalPosition::new(0.0, 0.0),
            previous_cursor_position: PhysicalPosition::new(0.0, 0.0),
            particle_placement_queue: Vec::new(),

            last_window_resize: None,
        }
    }

    pub fn update(&mut self) {
        while self.frame_time >= TARGET_TIME_PER_UPDATE {
            if !self.is_paused || self.should_update_once {
                self.should_update_once = false;
                self.sandbox.update();
            }
            self.frame_time -= TARGET_TIME_PER_UPDATE;
        }
    }

    pub fn handle_cursor_move(&mut self, new_cursor_position: PhysicalPosition<f64>) {
        self.previous_cursor_position = self.cursor_position;
        self.cursor_position = new_cursor_position;

        if self.should_place_particles {
            self.particle_placement_queue
                .push((self.previous_cursor_position, self.cursor_position));
        }
    }

    /// Place particles in a straight line from previous_cursor_position to cursor_position
    /// In addition, use data cached from WindowEvent::CursorMoved to ensure all gestures are properly captured
    pub fn place_queued_particles(&mut self, pixels: &Pixels) {
        // Queue current position if should_place_particles
        if self.should_place_particles {
            self.particle_placement_queue
                .push((self.previous_cursor_position, self.cursor_position));
        }
        // Place each particle
        for (p1, mut p2) in self.particle_placement_queue.drain(..) {
            // Adjust coordinates
            if let Some(locked_x) = self.x_axis_locked {
                if self.selected_particle != Some(ParticleType::Electricity) {
                    p2.x = locked_x;
                }
            }
            if let Some(locked_y) = self.y_axis_locked {
                if self.selected_particle != Some(ParticleType::Electricity) {
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
            let brush_size_x = self.brush_size as usize;
            let brush_size_y = if self.selected_particle == Some(ParticleType::Electricity) {
                1
            } else {
                self.brush_size as usize
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
                            match self.selected_particle {
                                Some(selected_particle) => {
                                    if self.sandbox[x][y].is_none() {
                                        self.sandbox[x][y] = Some(Particle::new(
                                            selected_particle,
                                            &mut self.sandbox.rng,
                                        ));
                                    }
                                }
                                None => self.sandbox[x][y] = None,
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
    }

    pub fn handle_window_resize(
        &mut self,
        window: &Window,
        pixels: &mut Pixels,
        glow_post_process: &mut GlowPostProcess,
    ) {
        // If a window resize is scheduled
        if let Some(last_window_resize) = self.last_window_resize {
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
                    surface_size.height = height_ratio.round() as u32 * SANDBOX_HEIGHT as u32;

                    window.set_inner_size(surface_size);

                    pixels.resize_surface(surface_size.width, surface_size.height);
                    glow_post_process.resize(
                        pixels.device(),
                        surface_size.width,
                        surface_size.height,
                    );
                }

                self.last_window_resize = None;
            }
        }
    }
}
