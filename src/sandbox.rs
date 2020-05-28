use crate::behavior::*;
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use simdnoise::NoiseBuilder;

pub const SIMULATION_WIDTH: usize = 600;
pub const SIMULATION_HEIGHT: usize = 400;

pub struct Sandbox {
    pub cells: Box<[[Option<Particle>; SIMULATION_HEIGHT]; SIMULATION_WIDTH]>,
    pub rng: ThreadRng,
    update_counter: u8,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            cells: Box::new([[None; SIMULATION_HEIGHT]; SIMULATION_WIDTH]),
            rng: thread_rng(),
            update_counter: 1,
        }
    }

    pub fn update(&mut self) {
        // Move particles
        self.update_counter = self.update_counter.checked_add(1).unwrap_or(1);
        for x in 0..SIMULATION_WIDTH {
            for y in 0..SIMULATION_HEIGHT {
                if let Some(particle) = &self.cells[x][y] {
                    if particle.last_update != self.update_counter {
                        let mut new_particle_position = (x, y);
                        match particle.ptype {
                            ParticleType::Sand => new_particle_position = move_powder(self, x, y),
                            ParticleType::WetSand => new_particle_position = move_solid(self, x, y),
                            ParticleType::Water => new_particle_position = move_liquid(self, x, y),
                            ParticleType::Acid => new_particle_position = move_liquid(self, x, y),
                            ParticleType::Iridium => {}
                            ParticleType::Replicator => {}
                            ParticleType::Plant => {
                                if particle.extra_data2 == 0 {
                                    new_particle_position = move_powder(self, x, y);
                                }
                            }
                            ParticleType::Cryotheum => {
                                new_particle_position = move_solid(self, x, y);
                            }
                            ParticleType::Unstable => {}
                            ParticleType::Electricity => {
                                new_particle_position = move_electricity(self, x, y);
                            }
                            ParticleType::Glass => {
                                if particle.tempature >= 30 {
                                    new_particle_position = move_liquid(self, x, y);
                                } else {
                                    new_particle_position = move_solid(self, x, y);
                                }
                            }
                            ParticleType::Life => new_particle_position = move_life(self, x, y),
                            ParticleType::Blood => new_particle_position = move_liquid(self, x, y),
                        }
                        self.cells[new_particle_position.0][new_particle_position.1]
                            .as_mut()
                            .unwrap()
                            .last_update = self.update_counter
                    }
                }
            }
        }

        // Transfer tempature between adjacent particles
        // Higher thermal conductivity = Slower tempature transfer
        fn thermal_conductivity(ptype: ParticleType) -> i16 {
            let tc = match ptype {
                ParticleType::Sand => 3,
                ParticleType::WetSand => 4,
                ParticleType::Water => 5,
                ParticleType::Acid => 4,
                ParticleType::Iridium => 8,
                ParticleType::Replicator => 3,
                ParticleType::Plant => 3,
                ParticleType::Cryotheum => 2,
                ParticleType::Unstable => 2,
                ParticleType::Electricity => 2,
                ParticleType::Glass => 2,
                ParticleType::Life => 3,
                ParticleType::Blood => 2,
            };
            assert!(tc > 1);
            tc
        }
        let cells_copy = self.cells.clone();
        for x in 0..SIMULATION_WIDTH {
            for y in 0..SIMULATION_HEIGHT {
                if let Some(particle1) = &cells_copy[x][y] {
                    if y != SIMULATION_HEIGHT - 1 {
                        if let Some(particle2) = &self.cells[x][y + 1] {
                            let tc = thermal_conductivity(particle1.ptype)
                                + thermal_conductivity(particle2.ptype);
                            let t = particle1.tempature / tc;
                            self.cells[x][y].as_mut().unwrap().tempature -= t;
                            self.cells[x][y + 1].as_mut().unwrap().tempature += t;
                        }
                    }
                    if x != SIMULATION_WIDTH - 1 {
                        if let Some(particle2) = &self.cells[x + 1][y] {
                            let tc = thermal_conductivity(particle1.ptype)
                                + thermal_conductivity(particle2.ptype);
                            let t = particle1.tempature / tc;
                            self.cells[x][y].as_mut().unwrap().tempature -= t;
                            self.cells[x + 1][y].as_mut().unwrap().tempature += t;
                        }
                    }
                    if y != 0 {
                        if let Some(particle2) = &self.cells[x][y - 1] {
                            let tc = thermal_conductivity(particle1.ptype)
                                + thermal_conductivity(particle2.ptype);
                            let t = particle1.tempature / tc;
                            self.cells[x][y].as_mut().unwrap().tempature -= t;
                            self.cells[x][y - 1].as_mut().unwrap().tempature += t;
                        }
                    }
                    if x != 0 {
                        if let Some(particle2) = &self.cells[x - 1][y] {
                            let tc = thermal_conductivity(particle1.ptype)
                                + thermal_conductivity(particle2.ptype);
                            let t = particle1.tempature / tc;
                            self.cells[x][y].as_mut().unwrap().tempature -= t;
                            self.cells[x - 1][y].as_mut().unwrap().tempature += t;
                        }
                    }
                }
            }
        }

        // Perform particle interactions and state updates
        self.update_counter = self.update_counter.checked_add(1).unwrap_or(1);
        for x in 0..SIMULATION_WIDTH {
            for y in 0..SIMULATION_HEIGHT {
                if let Some(particle) = &self.cells[x][y] {
                    if particle.last_update != self.update_counter {
                        match particle.ptype {
                            ParticleType::Sand => update_sand(self, x, y),
                            ParticleType::WetSand => {}
                            ParticleType::Water => update_water(self, x, y),
                            ParticleType::Acid => update_acid(self, x, y),
                            ParticleType::Iridium => {}
                            ParticleType::Replicator => update_replicator(self, x, y),
                            ParticleType::Plant => update_plant(self, x, y),
                            ParticleType::Cryotheum => update_cryotheum(self, x, y),
                            ParticleType::Unstable => update_unstable(self, x, y),
                            ParticleType::Electricity => update_electricity(self, x, y),
                            ParticleType::Glass => {}
                            ParticleType::Life => update_life(self, x, y),
                            ParticleType::Blood => update_blood(self, x, y),
                        }
                    }
                }
            }
        }
    }

    pub fn render(&mut self, frame: &mut [u8], dt: f32) {
        let noise =
            NoiseBuilder::turbulence_2d_offset(dt, SIMULATION_WIDTH * 2, dt, SIMULATION_HEIGHT / 2)
                .generate_scaled(-1.0, 1.0);

        let mut frame_index = 0;
        let mut noise_index = 0;
        for y in 0..SIMULATION_HEIGHT {
            for x in 0..SIMULATION_WIDTH {
                let mut color: (u8, u8, u8) = (20, 20, 20);
                if let Some(particle) = &self.cells[x][y] {
                    // Base color
                    let base_color: (u8, u8, u8) = match particle.ptype {
                        ParticleType::Sand => (196, 192, 135),
                        ParticleType::WetSand => (166, 162, 105),
                        ParticleType::Water => (8, 130, 201),
                        ParticleType::Acid => (128, 209, 0),
                        ParticleType::Iridium => (205, 210, 211),
                        ParticleType::Replicator => (68, 11, 67),
                        ParticleType::Plant => {
                            if particle.extra_data1 < 2 {
                                (75, 209, 216)
                            } else {
                                (86, 216, 143)
                            }
                        }
                        ParticleType::Cryotheum => (12, 191, 201),
                        ParticleType::Unstable => (181, 158, 128),
                        ParticleType::Electricity => (247, 244, 49),
                        ParticleType::Glass => (159, 198, 197),
                        ParticleType::Life => {
                            if particle.extra_data2 == 0 {
                                (135, 12, 211)
                            } else {
                                (90, 84, 84)
                            }
                        }
                        ParticleType::Blood => (112, 4, 17),
                    };

                    // Tint blue/red based on tempature
                    let mut r = 0;
                    let mut b = 0;
                    if particle.ptype != ParticleType::Electricity {
                        if particle.tempature < 0 {
                            b = clamp(particle.tempature.abs(), 0, 255);
                        } else {
                            r = clamp(particle.tempature, 0, 255);
                        }
                    }

                    // Darken/Lighten based on noise
                    let mut m = 0;
                    let noise_intensity = match particle.ptype {
                        ParticleType::Sand => 10,
                        ParticleType::WetSand => 10,
                        ParticleType::Water => 30,
                        ParticleType::Acid => 50,
                        ParticleType::Iridium => 0,
                        ParticleType::Replicator => 10,
                        ParticleType::Plant => {
                            if particle.extra_data1 < 2 {
                                10
                            } else {
                                5
                            }
                        }
                        ParticleType::Cryotheum => 10,
                        ParticleType::Unstable => {
                            if particle.tempature > 0 {
                                (10.0 * (particle.tempature as f64 / 200.0)) as i16
                            } else {
                                0
                            }
                        }
                        ParticleType::Electricity => 200,
                        ParticleType::Glass => 50,
                        ParticleType::Life => 0,
                        ParticleType::Blood => 20,
                    };
                    if noise_intensity != 0 {
                        m = (noise[noise_index] * noise_intensity as f32) as i16;
                    }

                    // Combine everything together
                    let r = base_color.0 as i16 + r + m + particle.color_offset as i16;
                    let g = base_color.1 as i16 + m + particle.color_offset as i16;
                    let b = base_color.2 as i16 + b + m + particle.color_offset as i16;
                    color = (
                        clamp(r, 0, 255) as u8,
                        clamp(g, 0, 255) as u8,
                        clamp(b, 0, 255) as u8,
                    )
                }

                frame[frame_index] = color.0;
                frame[frame_index + 1] = color.1;
                frame[frame_index + 2] = color.2;
                frame[frame_index + 3] = 255;

                frame_index += 4;
                noise_index += 1;
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct Particle {
    pub ptype: ParticleType,
    pub tempature: i16,
    pub extra_data1: i8,
    pub extra_data2: i8,
    pub color_offset: i8,
    last_update: u8,
}

impl Particle {
    pub fn new(ptype: ParticleType, rng: &mut impl Rng) -> Self {
        Self {
            ptype,
            tempature: match ptype {
                ParticleType::Sand => 0,
                ParticleType::WetSand => -5,
                ParticleType::Water => -10,
                ParticleType::Acid => 0,
                ParticleType::Iridium => 0,
                ParticleType::Replicator => 0,
                ParticleType::Plant => 0,
                ParticleType::Cryotheum => -60,
                ParticleType::Unstable => 0,
                ParticleType::Electricity => 300,
                ParticleType::Glass => 0,
                ParticleType::Life => 0,
                ParticleType::Blood => 0,
            },
            extra_data1: match ptype {
                ParticleType::Sand => 0,
                ParticleType::WetSand => 0,
                ParticleType::Water => 0,
                ParticleType::Acid => 0,
                ParticleType::Iridium => 0,
                ParticleType::Replicator => 0,
                ParticleType::Plant => thread_rng().gen_range(5, 21),
                ParticleType::Cryotheum => 0,
                ParticleType::Unstable => 0,
                ParticleType::Electricity => 0,
                ParticleType::Glass => 0,
                ParticleType::Life => 0,
                ParticleType::Blood => 0,
            },
            extra_data2: match ptype {
                ParticleType::Sand => 0,
                ParticleType::WetSand => 0,
                ParticleType::Water => 0,
                ParticleType::Acid => 0,
                ParticleType::Iridium => 0,
                ParticleType::Replicator => 0,
                ParticleType::Plant => 0,
                ParticleType::Cryotheum => 0,
                ParticleType::Unstable => 0,
                ParticleType::Electricity => 0,
                ParticleType::Glass => 0,
                ParticleType::Life => 0,
                ParticleType::Blood => 0,
            },
            color_offset: rng.gen_range(-10, 11),
            last_update: 0,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ParticleType {
    Sand,
    WetSand,
    Water,
    Acid,
    Iridium,
    Replicator,
    Plant,
    Cryotheum,
    Unstable,
    Electricity,
    Glass,
    Life,
    Blood,
}

fn clamp(value: i16, min: i16, max: i16) -> i16 {
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
