use crate::sandbox::{Particle, ParticleType};
use crate::{SIMULATION_HEIGHT, SIMULATION_WIDTH};

pub fn move_solid(cells: &mut Vec<Vec<Option<Particle>>>, x: usize, y: usize) -> (usize, usize) {
    // Move 1 down if able
    if y != SIMULATION_HEIGHT - 1 {
        if cells[x][y + 1].is_none() {
            cells[x][y + 1] = cells[x][y].take();
            return (x, y + 1);
        }
    }
    (x, y)
}

pub fn move_powder(cells: &mut Vec<Vec<Option<Particle>>>, x: usize, y: usize) -> (usize, usize) {
    if y != SIMULATION_HEIGHT - 1 {
        // Move 1 down if able
        if cells[x][y + 1].is_none() {
            cells[x][y + 1] = cells[x][y].take();
            return (x, y + 1);
        }
        // Else move 1 down and left if able
        if x != 0 {
            if cells[x - 1][y + 1].is_none() && cells[x - 1][y].is_none() {
                cells[x - 1][y + 1] = cells[x][y].take();
                return (x - 1, y + 1);
            }
        }
        // Else move 1 down and right if able
        if x != SIMULATION_WIDTH - 1 {
            if cells[x + 1][y + 1].is_none() && cells[x + 1][y].is_none() {
                cells[x + 1][y + 1] = cells[x][y].take();
                return (x + 1, y + 1);
            }
        }
    }
    (x, y)
}

pub fn move_liquid(cells: &mut Vec<Vec<Option<Particle>>>, x: usize, y: usize) -> (usize, usize) {
    if y != SIMULATION_HEIGHT - 1 {
        // Move 1 down if able
        if cells[x][y + 1].is_none() {
            cells[x][y + 1] = cells[x][y].take();
            return (x, y + 1);
        }
        // Else move 1 down and left if able
        if x != 0 {
            if cells[x - 1][y + 1].is_none() && cells[x - 1][y].is_none() {
                cells[x - 1][y + 1] = cells[x][y].take();
                return (x - 1, y + 1);
            }
        }
        // Else move 1 down and right if able
        if x != SIMULATION_WIDTH - 1 {
            if cells[x + 1][y + 1].is_none() && cells[x + 1][y].is_none() {
                cells[x + 1][y + 1] = cells[x][y].take();
                return (x + 1, y + 1);
            }
        }
    }
    // Else move left if able
    if x != 0 {
        if cells[x - 1][y].is_none() {
            cells[x - 1][y] = cells[x][y].take();
            return (x - 1, y);
        }
    }
    // Else move right if able
    if x != SIMULATION_WIDTH - 1 {
        if cells[x + 1][y].is_none() {
            cells[x + 1][y] = cells[x][y].take();
            return (x + 1, y);
        }
    }
    (x, y)
}

pub fn update_water(cells: &mut Vec<Vec<Option<Particle>>>, x: usize, y: usize) {
    if cells[x][y].unwrap().tempature <= -60 {
        cells[x][y].as_mut().unwrap().ptype = ParticleType::Cryotheum;
        return;
    }

    let mut y2 = y + 1;
    while y2 < SIMULATION_HEIGHT {
        match &cells[x][y2] {
            Some(particle) => match particle.ptype {
                ParticleType::Sand => {
                    cells[x][y] = None;
                    cells[x][y2].as_mut().unwrap().ptype = ParticleType::WetSand;
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

pub fn update_acid(cells: &mut Vec<Vec<Option<Particle>>>, x: usize, y: usize) {
    if y != SIMULATION_HEIGHT - 1 {
        if let Some(particle) = &cells[x][y + 1] {
            if particle.ptype != ParticleType::Acid
                && particle.ptype != ParticleType::Iridium
                && particle.ptype != ParticleType::Replicator
            {
                cells[x][y] = None;
                cells[x][y + 1] = None;
                return;
            }
        }
    }
    if x != SIMULATION_WIDTH - 1 {
        if let Some(particle) = &cells[x + 1][y] {
            if particle.ptype != ParticleType::Acid
                && particle.ptype != ParticleType::Iridium
                && particle.ptype != ParticleType::Replicator
            {
                cells[x][y] = None;
                cells[x + 1][y] = None;
                return;
            }
        }
    }
    if y != 0 {
        if let Some(particle) = &cells[x][y - 1] {
            if particle.ptype != ParticleType::Acid
                && particle.ptype != ParticleType::Iridium
                && particle.ptype != ParticleType::Replicator
            {
                cells[x][y] = None;
                cells[x][y - 1] = None;
                return;
            }
        }
    }
    if x != 0 {
        if let Some(particle) = &cells[x - 1][y] {
            if particle.ptype != ParticleType::Acid
                && particle.ptype != ParticleType::Iridium
                && particle.ptype != ParticleType::Replicator
            {
                cells[x][y] = None;
                cells[x - 1][y] = None;
                return;
            }
        }
    }
}

pub fn update_replicator(cells: &mut Vec<Vec<Option<Particle>>>, x: usize, y: usize) {
    if y < SIMULATION_HEIGHT - 2 {
        if let Some(particle) = &cells[x][y + 1] {
            if particle.ptype != ParticleType::Replicator {
                if cells[x][y + 2].is_none() {
                    cells[x][y + 2] = Some(Particle::new(particle.ptype));
                }
            }
        }
    }
    if x < SIMULATION_WIDTH - 2 {
        if let Some(particle) = &cells[x + 1][y] {
            if particle.ptype != ParticleType::Replicator {
                if cells[x + 2][y].is_none() {
                    cells[x + 2][y] = Some(Particle::new(particle.ptype));
                }
            }
        }
    }
    if y > 1 {
        if let Some(particle) = &cells[x][y - 1] {
            if particle.ptype != ParticleType::Replicator {
                if cells[x][y - 2].is_none() {
                    cells[x][y - 2] = Some(Particle::new(particle.ptype));
                }
            }
        }
    }
    if x > 1 {
        if let Some(particle) = &cells[x - 1][y] {
            if particle.ptype != ParticleType::Replicator {
                if cells[x - 2][y].is_none() {
                    cells[x - 2][y] = Some(Particle::new(particle.ptype));
                }
            }
        }
    }
}

pub fn update_plant(cells: &mut Vec<Vec<Option<Particle>>>, x: usize, y: usize) {
    if y != SIMULATION_HEIGHT - 1 {
        if let Some(particle) = cells[x][y + 1] {
            if particle.ptype == ParticleType::WetSand {
                cells[x][y].as_mut().unwrap().extra_data2 = 1;
            }
            if particle.ptype == ParticleType::Plant && particle.extra_data2 == 1 {
                cells[x][y].as_mut().unwrap().extra_data2 = 1;
            }
        }
    }

    if y != 0 {
        if cells[x][y].unwrap().extra_data2 == 1 {
            if cells[x][y].unwrap().extra_data1 > 0 {
                if y % 2 == 0 && x != SIMULATION_WIDTH - 1 {
                    if cells[x + 1][y - 1].is_none() {
                        let mut particle = Particle::new(ParticleType::Plant);
                        particle.extra_data1 = cells[x][y].unwrap().extra_data1 - 1;
                        particle.extra_data2 = 1;
                        cells[x + 1][y - 1] = Some(particle);
                        cells[x][y].as_mut().unwrap().extra_data1 = -1;
                    }
                } else if x != 0 {
                    if cells[x - 1][y - 1].is_none() {
                        let mut particle = Particle::new(ParticleType::Plant);
                        particle.extra_data1 = cells[x][y].unwrap().extra_data1 - 1;
                        particle.extra_data2 = 1;
                        cells[x - 1][y - 1] = Some(particle);
                        cells[x][y].as_mut().unwrap().extra_data1 = -1;
                    }
                }
            }

            if cells[x][y].unwrap().extra_data1 == 0 {
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
                            if cells[new_x as usize][new_y as usize].is_none() {
                                let mut particle = Particle::new(ParticleType::Plant);
                                particle.extra_data1 = -1;
                                particle.extra_data2 = 1;
                                cells[new_x as usize][new_y as usize] = Some(particle);
                                cells[x][y].as_mut().unwrap().extra_data1 = -1;
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn update_unstable(cells: &mut Vec<Vec<Option<Particle>>>, x: usize, y: usize) {
    // Increase tempature by 10 every half a second
    if cells[x][y].unwrap().extra_data1 == 30 {
        cells[x][y].as_mut().unwrap().extra_data1 = 0;
        cells[x][y].as_mut().unwrap().tempature += 10;
    } else {
        cells[x][y].as_mut().unwrap().extra_data1 += 1;
    }

    // When tempature >= 200 (10 seconds of existing), vaporize the surrounding area
    if cells[x][y].unwrap().tempature >= 200 {
        for x_offset in -20..=20 {
            for y_offset in -20..=20 {
                let x = x as i16 + x_offset;
                let y = y as i16 + y_offset;
                if (0..(SIMULATION_WIDTH as i16)).contains(&x)
                    && (0..(SIMULATION_HEIGHT as i16)).contains(&y)
                {
                    cells[x as usize][y as usize] = None;
                }
            }
        }
    }
}
