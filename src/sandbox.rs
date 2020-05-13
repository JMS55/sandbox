use crate::behavior::*;
use crate::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};

pub type Cells = Box<[[Option<Particle>; SIMULATION_HEIGHT]; SIMULATION_WIDTH]>;

pub struct Sandbox {
    pub cells: Cells,
    rng: ThreadRng,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            cells: Box::new([[None; SIMULATION_HEIGHT]; SIMULATION_WIDTH]),
            rng: thread_rng(),
        }
    }

    pub fn update(&mut self) {
        // Mark all particles as should_update
        for x in 0..SIMULATION_WIDTH {
            for y in 0..SIMULATION_HEIGHT {
                if let Some(particle) = &mut self.cells[x][y] {
                    particle.should_update = true;
                }
            }
        }
        // Move particles
        for x in 0..SIMULATION_WIDTH {
            for y in 0..SIMULATION_HEIGHT {
                if let Some(particle) = &self.cells[x][y] {
                    if particle.should_update {
                        let mut new_particle_position = (x, y);
                        match particle.ptype {
                            ParticleType::Sand => {
                                new_particle_position = move_powder(&mut self.cells, x, y);
                            }
                            ParticleType::WetSand => {
                                new_particle_position = move_solid(&mut self.cells, x, y);
                            }
                            ParticleType::Water => {
                                new_particle_position = move_liquid(&mut self.cells, x, y);
                            }
                            ParticleType::Acid => {
                                new_particle_position = move_liquid(&mut self.cells, x, y);
                            }
                            ParticleType::Iridium => {}
                            ParticleType::Replicator => {}
                            ParticleType::Plant => {
                                if particle.extra_data2 == 0 {
                                    new_particle_position = move_powder(&mut self.cells, x, y);
                                }
                            }
                            ParticleType::Cryotheum => {}
                            ParticleType::Unstable => {}
                        }
                        self.cells[new_particle_position.0][new_particle_position.1]
                            .as_mut()
                            .unwrap()
                            .should_update = false;
                    }
                }
            }
        }

        // Transfer tempature between adjacent particles
        let cells_copy = self.cells.clone();
        for x in 0..SIMULATION_WIDTH {
            for y in 0..SIMULATION_HEIGHT {
                if self.cells[x][y].is_some() {
                    if y != SIMULATION_HEIGHT - 1 {
                        if self.cells[x][y + 1].is_some() {
                            let t = cells_copy[x][y].unwrap().tempature / 5;
                            self.cells[x][y].as_mut().unwrap().tempature -= t;
                            self.cells[x][y + 1].as_mut().unwrap().tempature += t;
                        }
                    }
                    if x != SIMULATION_WIDTH - 1 {
                        if self.cells[x + 1][y].is_some() {
                            let t = cells_copy[x][y].unwrap().tempature / 5;
                            self.cells[x][y].as_mut().unwrap().tempature -= t;
                            self.cells[x + 1][y].as_mut().unwrap().tempature += t;
                        }
                    }
                    if y != 0 {
                        if self.cells[x][y - 1].is_some() {
                            let t = cells_copy[x][y].unwrap().tempature / 5;
                            self.cells[x][y].as_mut().unwrap().tempature -= t;
                            self.cells[x][y - 1].as_mut().unwrap().tempature += t;
                        }
                    }
                    if x != 0 {
                        if self.cells[x - 1][y].is_some() {
                            let t = cells_copy[x][y].unwrap().tempature / 5;
                            self.cells[x][y].as_mut().unwrap().tempature -= t;
                            self.cells[x - 1][y].as_mut().unwrap().tempature += t;
                        }
                    }
                }
            }
        }

        // Mark all particles as should_update
        for x in 0..SIMULATION_WIDTH {
            for y in 0..SIMULATION_HEIGHT {
                if let Some(particle) = &mut self.cells[x][y] {
                    particle.should_update = true;
                }
            }
        }
        // Perform particle interactions and state updates
        for x in 0..SIMULATION_WIDTH {
            for y in 0..SIMULATION_HEIGHT {
                if let Some(particle) = &self.cells[x][y] {
                    if particle.should_update {
                        match particle.ptype {
                            ParticleType::Sand => {}
                            ParticleType::WetSand => {}
                            ParticleType::Water => update_water(&mut self.cells, x, y),
                            ParticleType::Acid => update_acid(&mut self.cells, x, y),
                            ParticleType::Iridium => {}
                            ParticleType::Replicator => update_replicator(&mut self.cells, x, y),
                            ParticleType::Plant => update_plant(&mut self.cells, x, y),
                            ParticleType::Cryotheum => {}
                            ParticleType::Unstable => update_unstable(&mut self.cells, x, y),
                        }
                    }
                }
            }
        }
    }

    pub fn render(&mut self, frame: &mut [u8]) {
        let mut i = 0;
        for y in 0..SIMULATION_HEIGHT {
            for x in 0..SIMULATION_WIDTH {
                let mut color = (20, 20, 20);
                if let Some(particle) = &self.cells[x][y] {
                    // Base color
                    color = match particle.ptype {
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
                    };

                    // Darken/Lighten randomly
                    let noise = match particle.ptype {
                        ParticleType::Sand => 0,
                        ParticleType::WetSand => 0,
                        ParticleType::Water => 30,
                        ParticleType::Acid => 50,
                        ParticleType::Iridium => 0,
                        ParticleType::Replicator => 10,
                        ParticleType::Plant => {
                            if particle.extra_data1 < 2 {
                                5
                            } else {
                                0
                            }
                        }
                        ParticleType::Cryotheum => 0,
                        ParticleType::Unstable => {
                            if particle.tempature > 0 {
                                (10.0 * (particle.tempature as f64 / 200.0)) as i16
                            } else {
                                0
                            }
                        }
                    };
                    if noise != 0 {
                        let m = self.rng.gen_range(-noise, noise + 1);
                        color.0 = clamp(color.0 as i16 + m, 0, 255) as u8;
                        color.1 = clamp(color.1 as i16 + m, 0, 255) as u8;
                        color.2 = clamp(color.2 as i16 + m, 0, 255) as u8;
                    }

                    // Tint blue/red based on tempature
                    if particle.tempature < 0 {
                        let tempature = clamp(particle.tempature.abs(), 0, 255);
                        color.2 = color.2.saturating_add(tempature as u8);
                    } else {
                        let tempature = clamp(particle.tempature, 0, 255);
                        color.0 = color.0.saturating_add(tempature as u8);
                    }
                }

                frame[i] = color.0;
                frame[i + 1] = color.1;
                frame[i + 2] = color.2;
                frame[i + 3] = 255;

                i += 4;
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
    pub should_update: bool,
}

impl Particle {
    pub fn new(ptype: ParticleType) -> Self {
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
            },
            should_update: false,
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
