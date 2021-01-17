use crate::behavior::*;
use crate::sandbox::Sandbox;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use rand_pcg::Pcg64;

#[derive(Copy, Clone)]
pub struct Particle {
    pub ptype: ParticleType,
    pub temperature: i16,
    pub extra_data1: i8,
    pub extra_data2: i8,
    pub color_offset: i8,
    pub last_update: u8,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ParticleType {
    Sand,
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
    SuperLife,
    Blood,
    Smoke,
    Fire,
    Mirror,
    Steam,
    Glitch,
}

impl Distribution<ParticleType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ParticleType {
        match rng.gen_range(0..18) {
            0 => ParticleType::Sand,
            1 => ParticleType::Water,
            2 => ParticleType::Acid,
            3 => ParticleType::Iridium,
            4 => ParticleType::Replicator,
            5 => ParticleType::Plant,
            6 => ParticleType::Cryotheum,
            7 => ParticleType::Unstable,
            8 => ParticleType::Electricity,
            9 => ParticleType::Glass,
            10 => ParticleType::Life,
            11 => ParticleType::SuperLife,
            12 => ParticleType::Blood,
            13 => ParticleType::Smoke,
            14 => ParticleType::Fire,
            15 => ParticleType::Mirror,
            16 => ParticleType::Steam,
            17 => ParticleType::Glitch,
            _ => unreachable!(),
        }
    }
}

impl Particle {
    pub fn new(ptype: ParticleType, rng: &mut Pcg64) -> Self {
        Self {
            ptype,
            temperature: match ptype {
                ParticleType::Sand => 0,
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
                ParticleType::SuperLife => 0,
                ParticleType::Blood => 0,
                ParticleType::Fire => 130,
                ParticleType::Smoke => 0,
                ParticleType::Mirror => 0,
                ParticleType::Steam => 100,
                ParticleType::Glitch => 0,
            },
            extra_data1: match ptype {
                ParticleType::Sand => 0,
                ParticleType::Water => 0,
                ParticleType::Acid => 0,
                ParticleType::Iridium => 0,
                ParticleType::Replicator => 0,
                ParticleType::Plant => rng.gen_range(1..18),
                ParticleType::Cryotheum => 0,
                ParticleType::Unstable => 0,
                ParticleType::Electricity => 0,
                ParticleType::Glass => 0,
                ParticleType::Life => 0,
                ParticleType::SuperLife => 0,
                ParticleType::Blood => 0,
                ParticleType::Smoke => 90 + rng.gen_range(-20..20),
                ParticleType::Fire => rng.gen_range(0..60),
                ParticleType::Mirror => 0,
                ParticleType::Steam => 0,
                ParticleType::Glitch => 0,
            },
            extra_data2: match ptype {
                ParticleType::Sand => 0,
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
                ParticleType::SuperLife => 0,
                ParticleType::Blood => 0,
                ParticleType::Smoke => 90,
                ParticleType::Fire => 0,
                ParticleType::Mirror => 0,
                ParticleType::Steam => 0,
                ParticleType::Glitch => 0,
            },
            color_offset: rng.gen_range(-10..11),
            last_update: 0,
        }
    }

    pub fn move_particle(&self, sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
        let mut new_position = (x, y);
        match self.ptype {
            ParticleType::Sand => {
                if self.extra_data1 == 0 {
                    new_position = move_powder(sandbox, x, y);
                } else if self.extra_data1 == 1 {
                    new_position = move_solid(sandbox, x, y);
                }
            }
            ParticleType::Water => {
                if self.temperature > -80 {
                    new_position = move_liquid(sandbox, x, y);
                } else {
                    new_position = move_solid(sandbox, x, y);
                }
            }
            ParticleType::Acid => new_position = move_liquid(sandbox, x, y),
            ParticleType::Iridium => {}
            ParticleType::Replicator => {}
            ParticleType::Plant => {
                if self.extra_data2 == 0 {
                    new_position = move_powder(sandbox, x, y);
                }
            }
            ParticleType::Cryotheum => new_position = move_solid(sandbox, x, y),
            ParticleType::Unstable => {}
            ParticleType::Electricity => new_position = move_electricity(sandbox, x, y),
            ParticleType::Glass => {
                if self.temperature >= 30 {
                    new_position = move_liquid(sandbox, x, y);
                } else {
                    new_position = move_solid(sandbox, x, y);
                }
            }
            ParticleType::Life => new_position = move_life(sandbox, x, y),
            ParticleType::SuperLife => new_position = move_super_life(sandbox, x, y),
            ParticleType::Blood => new_position = move_liquid(sandbox, x, y),
            ParticleType::Smoke => new_position = move_gas(sandbox, x, y),
            ParticleType::Fire => new_position = move_fire(sandbox, x, y),
            ParticleType::Mirror => {}
            ParticleType::Steam => new_position = move_gas(sandbox, x, y),
            ParticleType::Glitch => new_position = move_liquid(sandbox, x, y),
        }
        new_position
    }

    pub fn update(&self, sandbox: &mut Sandbox, x: usize, y: usize) {
        match self.ptype {
            ParticleType::Sand => update_sand(sandbox, x, y),
            ParticleType::Water => update_water(sandbox, x, y),
            ParticleType::Acid => update_acid(sandbox, x, y),
            ParticleType::Iridium => {}
            ParticleType::Replicator => update_replicator(sandbox, x, y),
            ParticleType::Plant => update_plant(sandbox, x, y),
            ParticleType::Cryotheum => update_cryotheum(sandbox, x, y),
            ParticleType::Unstable => update_unstable(sandbox, x, y),
            ParticleType::Electricity => update_electricity(sandbox, x, y),
            ParticleType::Glass => {}
            ParticleType::Life => update_life(sandbox, x, y),
            ParticleType::SuperLife => update_life(sandbox, x, y),
            ParticleType::Blood => update_blood(sandbox, x, y),
            ParticleType::Smoke => update_smoke(sandbox, x, y),
            ParticleType::Fire => update_fire(sandbox, x, y),
            ParticleType::Mirror => update_mirror(sandbox, x, y),
            ParticleType::Steam => update_steam(sandbox, x, y),
            ParticleType::Glitch => update_glitch(sandbox, x, y),
        }
    }

    pub fn thermal_conductivity(&self) -> i16 {
        // Higher thermal conductivity = Slower temperature transfer
        let tc = match self.ptype {
            ParticleType::Sand => 3,
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
            ParticleType::SuperLife => 3,
            ParticleType::Blood => 2,
            ParticleType::Smoke => 6,
            ParticleType::Fire => 2,
            ParticleType::Mirror => 7,
            ParticleType::Steam => 6,
            ParticleType::Glitch => 2,
        };
        assert!(tc > 1);
        tc
    }

    pub fn base_color(&self, rng: &mut Pcg64) -> (u8, u8, u8) {
        match self.ptype {
            ParticleType::Sand => {
                if self.extra_data1 == 0 {
                    (196, 192, 135)
                } else {
                    (166, 162, 105)
                }
            }
            ParticleType::Water => (26, 91, 175),
            ParticleType::Acid => (138, 209, 0),
            ParticleType::Iridium => (100, 100, 100),
            ParticleType::Replicator => {
                if self.extra_data1 == 0 {
                    (68, 11, 67)
                } else {
                    (88, 31, 107)
                }
            }
            ParticleType::Plant => {
                if self.extra_data1 < 2 {
                    (6, 89, 9)
                } else {
                    (20, 61, 21)
                }
            }
            ParticleType::Cryotheum => (12, 191, 201),
            ParticleType::Unstable => (84, 68, 45),
            ParticleType::Electricity => (247, 244, 49),
            ParticleType::Glass => (159, 198, 197),
            ParticleType::Life => {
                if self.extra_data2 == 0 {
                    (135, 12, 211)
                } else {
                    (90, 84, 84)
                }
            }
            ParticleType::SuperLife => {
                if self.extra_data2 == 0 {
                    (188, 20, 183)
                } else {
                    (90, 84, 84)
                }
            }
            ParticleType::Blood => (112, 4, 17),
            ParticleType::Smoke => (5, 5, 5),
            ParticleType::Fire => (237, 86, 4),
            ParticleType::Mirror => {
                // Lerp green-pink-green
                let mut t = self.extra_data1 as f64 / 59.0;
                let c1 = (78.0 / 255.0, 216.0 / 255.0, 131.0 / 255.0);
                let c2 = (216.0 / 255.0, 78.0 / 255.0, 163.0 / 255.0);
                let ((r1, g1, b1), (r2, g2, b2)) = {
                    if self.extra_data1 < 60 {
                        (c1, c2)
                    } else {
                        t = (self.extra_data1 - 60) as f64 / 59.0;
                        (c2, c1)
                    }
                };
                let r = (1.0 - t) * r1 + t * r2;
                let b = (1.0 - t) * b1 + t * b2;
                let g = (1.0 - t) * g1 + t * g2;
                ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
            }
            ParticleType::Steam => (40, 140, 140),
            ParticleType::Glitch => {
                if rng.gen_bool(0.95) {
                    (81, 80, 66)
                } else {
                    (218, 101, 126)
                }
            }
        }
    }

    pub fn shimmer_intensity(&self) -> i16 {
        match self.ptype {
            ParticleType::Sand => 10,
            ParticleType::Water => 30,
            ParticleType::Acid => 50,
            ParticleType::Iridium => 0,
            ParticleType::Replicator => 10,
            ParticleType::Plant => {
                if self.extra_data1 < 2 {
                    10
                } else {
                    5
                }
            }
            ParticleType::Cryotheum => 10,
            ParticleType::Unstable => {
                if self.temperature > 0 {
                    (self.temperature as f64 / 5.0).round() as i16
                } else {
                    0
                }
            }
            ParticleType::Electricity => 200,
            ParticleType::Glass => 50,
            ParticleType::Life => 0,
            ParticleType::SuperLife => 15,
            ParticleType::Blood => 20,
            ParticleType::Smoke => 10,
            ParticleType::Fire => 50,
            ParticleType::Mirror => 20,
            ParticleType::Steam => 10,
            ParticleType::Glitch => 30,
        }
    }

    pub fn is_glowing(&self) -> bool {
        match self.ptype {
            ParticleType::Sand => false,
            ParticleType::Water => false,
            ParticleType::Acid => true,
            ParticleType::Iridium => false,
            ParticleType::Replicator => false,
            ParticleType::Plant => false,
            ParticleType::Cryotheum => false,
            ParticleType::Unstable => false,
            ParticleType::Electricity => true,
            ParticleType::Glass => false,
            ParticleType::Life => false,
            ParticleType::SuperLife => false,
            ParticleType::Blood => false,
            ParticleType::Smoke => true,
            ParticleType::Fire => true,
            ParticleType::Mirror => false,
            ParticleType::Steam => false,
            ParticleType::Glitch => true,
        }
    }

    pub fn dissolved_by_acid(&self) -> bool {
        match self.ptype {
            ParticleType::Sand => true,
            ParticleType::Water => true,
            ParticleType::Acid => false,
            ParticleType::Iridium => false,
            ParticleType::Replicator => false,
            ParticleType::Plant => true,
            ParticleType::Cryotheum => true,
            ParticleType::Unstable => true,
            ParticleType::Electricity => true,
            ParticleType::Glass => false,
            ParticleType::Life => true,
            ParticleType::SuperLife => true,
            ParticleType::Blood => true,
            ParticleType::Smoke => false,
            ParticleType::Fire => true,
            ParticleType::Mirror => false,
            ParticleType::Steam => true,
            ParticleType::Glitch => true,
        }
    }

    pub fn affected_by_cryotheum_coldsnap(&self) -> bool {
        match self.ptype {
            ParticleType::Sand => true,
            ParticleType::Water => true,
            ParticleType::Acid => true,
            ParticleType::Iridium => true,
            ParticleType::Replicator => true,
            ParticleType::Plant => true,
            ParticleType::Cryotheum => false,
            ParticleType::Unstable => true,
            ParticleType::Electricity => true,
            ParticleType::Glass => true,
            ParticleType::Life => true,
            ParticleType::SuperLife => true,
            ParticleType::Blood => true,
            ParticleType::Smoke => false,
            ParticleType::Fire => true,
            ParticleType::Mirror => true,
            ParticleType::Steam => true,
            ParticleType::Glitch => true,
        }
    }

    pub fn can_be_vaporized_by_unstable(&self) -> bool {
        match self.ptype {
            ParticleType::Sand => true,
            ParticleType::Water => true,
            ParticleType::Acid => true,
            ParticleType::Iridium => false,
            ParticleType::Replicator => false,
            ParticleType::Plant => true,
            ParticleType::Cryotheum => true,
            ParticleType::Unstable => true,
            ParticleType::Electricity => true,
            ParticleType::Glass => true,
            ParticleType::Life => true,
            ParticleType::SuperLife => true,
            ParticleType::Blood => true,
            ParticleType::Smoke => false,
            ParticleType::Fire => false,
            ParticleType::Mirror => true,
            ParticleType::Steam => true,
            ParticleType::Glitch => true,
        }
    }

    pub fn is_flammable(&self) -> bool {
        match self.ptype {
            ParticleType::Sand => false,
            ParticleType::Water => false,
            ParticleType::Acid => false,
            ParticleType::Iridium => false,
            ParticleType::Replicator => false,
            ParticleType::Plant => true,
            ParticleType::Cryotheum => true,
            ParticleType::Unstable => false,
            ParticleType::Electricity => false,
            ParticleType::Glass => false,
            ParticleType::Life => true,
            ParticleType::SuperLife => false,
            ParticleType::Blood => false,
            ParticleType::Smoke => false,
            ParticleType::Fire => false,
            ParticleType::Mirror => false,
            ParticleType::Steam => false,
            ParticleType::Glitch => true,
        }
    }
}
