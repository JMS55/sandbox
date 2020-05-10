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
                    particle.should_update = true;
                }
            }
        }

        for x in 0..SIMULATION_WIDTH {
            for y in 0..SIMULATION_HEIGHT {
                if let Some(particle) = &self.cells[x][y] {
                    if particle.should_update {
                        let mut new_particle_position = (x, y);
                        match particle.ptype {
                            ParticleType::Sand => {
                                new_particle_position = self.move_solid(x, y);
                            }
                            ParticleType::WetSand => {}
                            ParticleType::Water | ParticleType::Acid => {
                                new_particle_position = self.move_liquid(x, y);
                            }
                            ParticleType::Iridium => {}
                            ParticleType::Replicator => {}
                            ParticleType::Plant => {
                                if particle.extra_data2 == 0 {
                                    new_particle_position = self.move_solid(x, y);
                                }
                            }
                        }
                        self.cells[new_particle_position.0][new_particle_position.1]
                            .as_mut()
                            .unwrap()
                            .should_update = false;
                    }
                }
            }
        }

        for x in 0..SIMULATION_WIDTH {
            for y in 0..SIMULATION_HEIGHT {
                if let Some(particle) = &mut self.cells[x][y] {
                    particle.should_update = true;
                }
            }
        }

        for x in 0..SIMULATION_WIDTH {
            for y in 0..SIMULATION_HEIGHT {
                if let Some(particle) = &self.cells[x][y] {
                    if particle.should_update {
                        match particle.ptype {
                            ParticleType::Sand => {}
                            ParticleType::WetSand => {}
                            ParticleType::Water => self.update_water(x, y),
                            ParticleType::Acid => self.update_acid(x, y),
                            ParticleType::Iridium => {}
                            ParticleType::Replicator => self.update_replicator(x, y),
                            ParticleType::Plant => self.update_plant(x, y),
                        }
                    }
                }
                if let Some(particle) = self.cells[x][y].as_mut() {
                    particle.should_update = false;
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
                    };
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

    fn move_solid(&mut self, x: usize, y: usize) -> (usize, usize) {
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

    fn update_water(&mut self, x: usize, y: usize) {
        let mut y2 = y + 1;
        while y2 < SIMULATION_HEIGHT {
            match &self.cells[x][y2] {
                Some(particle) => match particle.ptype {
                    ParticleType::Sand => {
                        self.cells[x][y] = None;
                        self.cells[x][y2].as_mut().unwrap().ptype = ParticleType::WetSand;
                        return;
                    }
                    ParticleType::WetSand => {}
                    _ => return,
                },
                None => return,
            }
            y2 += 1;
        }
    }

    fn update_acid(&mut self, x: usize, y: usize) {
        if y != SIMULATION_HEIGHT - 1 {
            if let Some(particle) = &self.cells[x][y + 1] {
                if particle.ptype != ParticleType::Acid
                    && particle.ptype != ParticleType::Iridium
                    && particle.ptype != ParticleType::Replicator
                {
                    self.cells[x][y] = None;
                    self.cells[x][y + 1] = None;
                    return;
                }
            }
        }
        if x != SIMULATION_WIDTH - 1 {
            if let Some(particle) = &self.cells[x + 1][y] {
                if particle.ptype != ParticleType::Acid
                    && particle.ptype != ParticleType::Iridium
                    && particle.ptype != ParticleType::Replicator
                {
                    self.cells[x][y] = None;
                    self.cells[x + 1][y] = None;
                    return;
                }
            }
        }
        if y != 0 {
            if let Some(particle) = &self.cells[x][y - 1] {
                if particle.ptype != ParticleType::Acid
                    && particle.ptype != ParticleType::Iridium
                    && particle.ptype != ParticleType::Replicator
                {
                    self.cells[x][y] = None;
                    self.cells[x][y - 1] = None;
                    return;
                }
            }
        }
        if x != 0 {
            if let Some(particle) = &self.cells[x - 1][y] {
                if particle.ptype != ParticleType::Acid
                    && particle.ptype != ParticleType::Iridium
                    && particle.ptype != ParticleType::Replicator
                {
                    self.cells[x][y] = None;
                    self.cells[x - 1][y] = None;
                    return;
                }
            }
        }
    }

    fn update_replicator(&mut self, x: usize, y: usize) {
        if y < SIMULATION_HEIGHT - 2 {
            if let Some(particle) = &self.cells[x][y + 1] {
                if particle.ptype != ParticleType::Replicator {
                    if self.cells[x][y + 2].is_none() {
                        self.cells[x][y + 2] = Some(Particle::new(particle.ptype));
                        return;
                    }
                }
            }
        }
        if x < SIMULATION_WIDTH - 2 {
            if let Some(particle) = &self.cells[x + 1][y] {
                if particle.ptype != ParticleType::Replicator {
                    if self.cells[x + 2][y].is_none() {
                        self.cells[x + 2][y] = Some(Particle::new(particle.ptype));
                        return;
                    }
                }
            }
        }
        if y > 1 {
            if let Some(particle) = &self.cells[x][y - 1] {
                if particle.ptype != ParticleType::Replicator {
                    if self.cells[x][y - 2].is_none() {
                        self.cells[x][y - 2] = Some(Particle::new(particle.ptype));
                        return;
                    }
                }
            }
        }
        if x > 1 {
            if let Some(particle) = &self.cells[x - 1][y] {
                if particle.ptype != ParticleType::Replicator {
                    if self.cells[x - 2][y].is_none() {
                        self.cells[x - 2][y] = Some(Particle::new(particle.ptype));
                        return;
                    }
                }
            }
        }
    }

    fn update_plant(&mut self, x: usize, y: usize) {
        if y != SIMULATION_HEIGHT - 1 {
            if let Some(particle) = self.cells[x][y + 1] {
                if particle.ptype == ParticleType::WetSand {
                    self.cells[x][y].as_mut().unwrap().extra_data2 = 1;
                }
                if particle.ptype == ParticleType::Plant && particle.extra_data2 == 1 {
                    self.cells[x][y].as_mut().unwrap().extra_data2 = 1;
                }
            }
        }

        if y != 0 {
            if self.cells[x][y].unwrap().extra_data2 == 1 {
                if self.cells[x][y].unwrap().extra_data1 > 0 {
                    if y % 2 == 0 && x != SIMULATION_WIDTH - 1 {
                        if self.cells[x + 1][y - 1].is_none() {
                            let mut particle = Particle::new(ParticleType::Plant);
                            particle.extra_data1 = self.cells[x][y].unwrap().extra_data1 - 1;
                            particle.extra_data2 = 1;
                            self.cells[x + 1][y - 1] = Some(particle);
                            self.cells[x][y].unwrap().extra_data1 = -1;
                        }
                    } else if x != 0 {
                        if self.cells[x - 1][y - 1].is_none() {
                            let mut particle = Particle::new(ParticleType::Plant);
                            particle.extra_data1 = self.cells[x][y].unwrap().extra_data1 - 1;
                            particle.extra_data2 = 1;
                            self.cells[x - 1][y - 1] = Some(particle);
                            self.cells[x][y].unwrap().extra_data1 = -1;
                        }
                    }
                }

                if self.cells[x][y].unwrap().extra_data1 == 0 {
                    for y_offset in 0..=10 {
                        let x_range = if y_offset < 4 {
                            4
                        } else if y_offset < 9 {
                            8
                        } else {
                            6
                        };
                        for x_offset in -x_range..=x_range {
                            let new_x = x as i32 + x_offset;
                            let new_y = y as i32 + y_offset;
                            if new_x > -1
                                && new_x < SIMULATION_WIDTH as i32
                                && new_y > -1
                                && new_y < SIMULATION_HEIGHT as i32
                            {
                                if self.cells[new_x as usize][new_y as usize].is_none() {
                                    let mut particle = Particle::new(ParticleType::Plant);
                                    particle.extra_data1 = -1;
                                    particle.extra_data2 = 1;
                                    self.cells[new_x as usize][new_y as usize] = Some(particle);
                                    self.cells[x][y].unwrap().extra_data1 = -1;
                                }
                            }
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
    pub extra_data1: i8,
    pub extra_data2: i8,
    pub should_update: bool,
}

impl Particle {
    pub fn new(ptype: ParticleType) -> Self {
        Self {
            ptype,
            extra_data1: match ptype {
                ParticleType::Sand => 0,
                ParticleType::WetSand => 0,
                ParticleType::Water => 0,
                ParticleType::Acid => 0,
                ParticleType::Iridium => 0,
                ParticleType::Replicator => 0,
                ParticleType::Plant => thread_rng().gen_range(5, 21),
            },
            extra_data2: match ptype {
                ParticleType::Sand => 0,
                ParticleType::WetSand => 0,
                ParticleType::Water => 0,
                ParticleType::Acid => 0,
                ParticleType::Iridium => 0,
                ParticleType::Replicator => 0,
                ParticleType::Plant => 0,
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
