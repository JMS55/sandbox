use crate::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};

pub struct Sandbox {
    pub cells: Vec<Vec<Option<Particle>>>,
    rng: ThreadRng,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            cells: vec![vec![None; SIMULATION_HEIGHT]; SIMULATION_WIDTH],
            rng: thread_rng(),
        }
    }

    pub fn update(&mut self) {
        for x in 0..SIMULATION_WIDTH {
            for y in 0..SIMULATION_HEIGHT {
                if let Some(particle) = &mut self.cells[x][y] {
                    particle.moved = false;
                }
            }
        }

        for x in (0..SIMULATION_WIDTH).rev() {
            for y in (0..SIMULATION_HEIGHT).rev() {
                if let Some(particle) = &self.cells[x][y] {
                    if !particle.moved {
                        let mut new_particle_position = (x, y);
                        match particle.ptype {
                            ParticleType::Sand => {
                                new_particle_position = self.move_sand(x, y);
                            }
                            ParticleType::Water | ParticleType::Acid => {
                                new_particle_position = self.move_liquid(x, y);
                            }
                            ParticleType::Iridium => {}
                        }
                        self.cells[new_particle_position.0][new_particle_position.1]
                            .as_mut()
                            .unwrap()
                            .moved = true;
                    }
                }
            }
        }

        for x in (0..SIMULATION_WIDTH).rev() {
            for y in (0..SIMULATION_HEIGHT).rev() {
                self.update_acid(x, y);
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

    fn move_sand(&mut self, x: usize, y: usize) -> (usize, usize) {
        if y != SIMULATION_HEIGHT - 1 {
            // Move 1 down if able
            if self.cells[x][y + 1].is_none() {
                self.cells[x][y + 1] = self.cells[x][y].take();
                return (x, y + 1);
            }
            // Else move 1 down and left if able
            if x != 0 {
                if self.cells[x - 1][y + 1].is_none() && self.cells[x - 1][y].is_none() {
                    self.cells[x - 1][y + 1] = self.cells[x][y].take();
                    return (x - 1, y + 1);
                }
            }
            // Else move 1 down and right if able
            if x != SIMULATION_WIDTH - 1 {
                if self.cells[x + 1][y + 1].is_none() && self.cells[x + 1][y].is_none() {
                    self.cells[x + 1][y + 1] = self.cells[x][y].take();
                    return (x + 1, y + 1);
                }
            }
        }
        (x, y)
    }

    fn move_liquid(&mut self, x: usize, y: usize) -> (usize, usize) {
        if y != SIMULATION_HEIGHT - 1 {
            // Move 1 down if able
            if self.cells[x][y + 1].is_none() {
                self.cells[x][y + 1] = self.cells[x][y].take();
                return (x, y + 1);
            }
            // Else move 1 down and left if able
            if x != 0 {
                if self.cells[x - 1][y + 1].is_none() && self.cells[x - 1][y].is_none() {
                    self.cells[x - 1][y + 1] = self.cells[x][y].take();
                    return (x - 1, y + 1);
                }
            }
            // Else move 1 down and right if able
            if x != SIMULATION_WIDTH - 1 {
                if self.cells[x + 1][y + 1].is_none() && self.cells[x + 1][y].is_none() {
                    self.cells[x + 1][y + 1] = self.cells[x][y].take();
                    return (x + 1, y + 1);
                }
            }
        }
        // Else move left if able
        if x != 0 {
            if self.cells[x - 1][y].is_none() {
                self.cells[x - 1][y] = self.cells[x][y].take();
                return (x - 1, y);
            }
        }
        // Else move right if able
        if x != SIMULATION_WIDTH - 1 {
            if self.cells[x + 1][y].is_none() {
                self.cells[x + 1][y] = self.cells[x][y].take();
                return (x + 1, y);
            }
        }
        (x, y)
    }

    fn update_acid(&mut self, x: usize, y: usize) {
        if let Some(particle1) = &self.cells[x][y] {
            if particle1.ptype == ParticleType::Acid {
                if y != SIMULATION_HEIGHT - 1 {
                    if let Some(particle2) = &self.cells[x][y + 1] {
                        if particle2.ptype != ParticleType::Acid
                            && particle2.ptype != ParticleType::Iridium
                        {
                            self.cells[x][y] = None;
                            self.cells[x][y + 1] = None;
                            return;
                        }
                    }
                }
                if x != SIMULATION_WIDTH - 1 {
                    if let Some(particle2) = &self.cells[x + 1][y] {
                        if particle2.ptype != ParticleType::Acid
                            && particle2.ptype != ParticleType::Iridium
                        {
                            self.cells[x][y] = None;
                            self.cells[x + 1][y] = None;
                            return;
                        }
                    }
                }
                if y != 0 {
                    if let Some(particle2) = &self.cells[x][y - 1] {
                        if particle2.ptype != ParticleType::Acid
                            && particle2.ptype != ParticleType::Iridium
                        {
                            self.cells[x][y] = None;
                            self.cells[x][y - 1] = None;
                            return;
                        }
                    }
                }
                if x != 0 {
                    if let Some(particle2) = &self.cells[x - 1][y] {
                        if particle2.ptype != ParticleType::Acid
                            && particle2.ptype != ParticleType::Iridium
                        {
                            self.cells[x][y] = None;
                            self.cells[x - 1][y] = None;
                            return;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct Particle {
    pub ptype: ParticleType,
    pub moved: bool,
}

impl Particle {
    pub fn new(ptype: ParticleType) -> Self {
        Self {
            ptype,
            moved: false,
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
