use crate::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};

pub struct Sandbox {
    pub cells: [[Option<Particle>; SIMULATION_HEIGHT]; SIMULATION_WIDTH],
    rng: ThreadRng,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            cells: [[None; SIMULATION_HEIGHT]; SIMULATION_WIDTH],
            rng: thread_rng(),
        }
    }

    pub fn update(&mut self) {
        for x in 0..SIMULATION_WIDTH {
            for y in 0..SIMULATION_HEIGHT {
                if let Some(particle) = &mut self.cells[x][y] {
                    particle.updated = false;
                }
            }
        }

        for x in (0..SIMULATION_WIDTH).rev() {
            for y in (0..SIMULATION_HEIGHT).rev() {
                if let Some(particle) = &self.cells[x][y] {
                    if !particle.updated {
                        let mut new_particle_position = Some((x, y));
                        loop {
                            match particle.ptype {
                                ParticleType::Sand => {
                                    if y != SIMULATION_HEIGHT - 1 {
                                        // Move 1 down if able
                                        if self.cells[x][y + 1].is_none() {
                                            self.cells[x][y + 1] = self.cells[x][y].take();
                                            new_particle_position = Some((x, y + 1));
                                            break;
                                        }
                                        // Else move 1 down and left if able
                                        if x != 0 {
                                            if self.cells[x - 1][y + 1].is_none()
                                                && self.cells[x - 1][y].is_none()
                                            {
                                                self.cells[x - 1][y + 1] = self.cells[x][y].take();
                                                new_particle_position = Some((x - 1, y + 1));
                                                break;
                                            }
                                        }
                                        // Else move 1 down and right if able
                                        if x != SIMULATION_WIDTH - 1 {
                                            if self.cells[x + 1][y + 1].is_none()
                                                && self.cells[x + 1][y].is_none()
                                            {
                                                self.cells[x + 1][y + 1] = self.cells[x][y].take();
                                                new_particle_position = Some((x + 1, y + 1));
                                                break;
                                            }
                                        }
                                    }
                                }
                                ParticleType::Water | ParticleType::Acid => {
                                    if y != SIMULATION_HEIGHT - 1 {
                                        // Move 1 down if able
                                        if self.cells[x][y + 1].is_none() {
                                            self.cells[x][y + 1] = self.cells[x][y].take();
                                            new_particle_position = Some((x, y + 1));
                                            break;
                                        }
                                        // Else move 1 down and left if able
                                        if x != 0 {
                                            if self.cells[x - 1][y + 1].is_none()
                                                && self.cells[x - 1][y].is_none()
                                            {
                                                self.cells[x - 1][y + 1] = self.cells[x][y].take();
                                                new_particle_position = Some((x - 1, y + 1));
                                                break;
                                            }
                                        }
                                        // Else move 1 down and right if able
                                        if x != SIMULATION_WIDTH - 1 {
                                            if self.cells[x + 1][y + 1].is_none()
                                                && self.cells[x + 1][y].is_none()
                                            {
                                                self.cells[x + 1][y + 1] = self.cells[x][y].take();
                                                new_particle_position = Some((x + 1, y + 1));
                                                break;
                                            }
                                        }
                                    }
                                    // Else move left if able
                                    if x != 0 {
                                        if self.cells[x - 1][y].is_none() {
                                            self.cells[x - 1][y] = self.cells[x][y].take();
                                            new_particle_position = Some((x - 1, y));
                                            break;
                                        }
                                    }
                                    // Else move right if able
                                    if x != SIMULATION_WIDTH - 1 {
                                        if self.cells[x + 1][y].is_none() {
                                            self.cells[x + 1][y] = self.cells[x][y].take();
                                            new_particle_position = Some((x + 1, y));
                                            break;
                                        }
                                    }
                                }
                                ParticleType::Iridium => {}
                            }
                            break;
                        }
                        if let Some((x, y)) = new_particle_position {
                            self.cells[x][y].as_mut().unwrap().updated = true;
                        }
                    }
                }
            }
        }

        for x in (0..SIMULATION_WIDTH).rev() {
            for y in (0..SIMULATION_HEIGHT).rev() {
                if let Some(particle1) = &self.cells[x][y] {
                    if particle1.ptype != ParticleType::Acid
                        && particle1.ptype != ParticleType::Iridium
                    {
                        loop {
                            if y != SIMULATION_HEIGHT - 1 {
                                if let Some(particle2) = self.cells[x][y + 1] {
                                    if particle2.ptype == ParticleType::Acid {
                                        self.cells[x][y] = None;
                                        self.cells[x][y + 1] = None;
                                        break;
                                    }
                                }
                            }
                            if x != SIMULATION_WIDTH - 1 {
                                if let Some(particle2) = self.cells[x + 1][y] {
                                    if particle2.ptype == ParticleType::Acid {
                                        self.cells[x][y] = None;
                                        self.cells[x + 1][y] = None;
                                        break;
                                    }
                                }
                            }
                            if y != 0 {
                                if let Some(particle2) = self.cells[x][y - 1] {
                                    if particle2.ptype == ParticleType::Acid {
                                        self.cells[x][y] = None;
                                        self.cells[x][y - 1] = None;
                                        break;
                                    }
                                }
                            }
                            if x != 0 {
                                if let Some(particle2) = self.cells[x - 1][y] {
                                    if particle2.ptype == ParticleType::Acid {
                                        self.cells[x][y] = None;
                                        self.cells[x - 1][y] = None;
                                        break;
                                    }
                                }
                            }
                            break;
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
                    color = match particle.ptype {
                        ParticleType::Sand => (196, 192, 135),
                        ParticleType::Water => (8, 130, 201),
                        ParticleType::Acid => (128, 209, 0),
                        ParticleType::Iridium => (205, 210, 211),
                    };
                    let noise = match particle.ptype {
                        ParticleType::Water => 30,
                        ParticleType::Acid => 50,
                        _ => 0,
                    };
                    if noise != 0 {
                        let m = self.rng.gen_range(-noise, noise + 1);
                        color.0 = clamp(color.0 as i16 + m, 0, 255) as u8;
                        color.1 = clamp(color.1 as i16 + m, 0, 255) as u8;
                        color.2 = clamp(color.2 as i16 + m, 0, 255) as u8;
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
    pub updated: bool,
}

impl Particle {
    pub fn new(ptype: ParticleType) -> Self {
        Self {
            ptype,
            updated: false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ParticleType {
    Sand,
    Water,
    Acid,
    Iridium,
}

// TODO: Replace with the std version once rust adds it
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
