use crate::sandbox::{Particle, ParticleType, Sandbox};

pub fn move_solid(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    // Move 1 down if able
    if y != sandbox.height - 1 {
        if sandbox.cells[x][y + 1].is_none() {
            sandbox.cells[x][y + 1] = sandbox.cells[x][y].take();
            return (x, y + 1);
        }
    }
    (x, y)
}

pub fn move_powder(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    if y != sandbox.height - 1 {
        // Move 1 down if able
        if sandbox.cells[x][y + 1].is_none() {
            sandbox.cells[x][y + 1] = sandbox.cells[x][y].take();
            return (x, y + 1);
        }
        // Else move 1 down and left if able
        if x != 0 {
            if sandbox.cells[x - 1][y + 1].is_none() && sandbox.cells[x - 1][y].is_none() {
                sandbox.cells[x - 1][y + 1] = sandbox.cells[x][y].take();
                return (x - 1, y + 1);
            }
        }
        // Else move 1 down and right if able
        if x != sandbox.width - 1 {
            if sandbox.cells[x + 1][y + 1].is_none() && sandbox.cells[x + 1][y].is_none() {
                sandbox.cells[x + 1][y + 1] = sandbox.cells[x][y].take();
                return (x + 1, y + 1);
            }
        }
    }
    (x, y)
}

pub fn move_liquid(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    if y != sandbox.height - 1 {
        // Move 1 down if able
        if sandbox.cells[x][y + 1].is_none() {
            sandbox.cells[x][y + 1] = sandbox.cells[x][y].take();
            return (x, y + 1);
        }
        // Else move 1 down and left if able
        if x != 0 {
            if sandbox.cells[x - 1][y + 1].is_none() && sandbox.cells[x - 1][y].is_none() {
                sandbox.cells[x - 1][y + 1] = sandbox.cells[x][y].take();
                return (x - 1, y + 1);
            }
        }
        // Else move 1 down and right if able
        if x != sandbox.width - 1 {
            if sandbox.cells[x + 1][y + 1].is_none() && sandbox.cells[x + 1][y].is_none() {
                sandbox.cells[x + 1][y + 1] = sandbox.cells[x][y].take();
                return (x + 1, y + 1);
            }
        }
    }
    // Else move left if able
    if x != 0 {
        if sandbox.cells[x - 1][y].is_none() {
            sandbox.cells[x - 1][y] = sandbox.cells[x][y].take();
            return (x - 1, y);
        }
    }
    // Else move right if able
    if x != sandbox.width - 1 {
        if sandbox.cells[x + 1][y].is_none() {
            sandbox.cells[x + 1][y] = sandbox.cells[x][y].take();
            return (x + 1, y);
        }
    }
    (x, y)
}

pub fn update_water(sandbox: &mut Sandbox, x: usize, y: usize) {
    // if sandbox.cells[x][y].unwrap().tempature >= 100 {
    //     sandbox.cells[x][y] = None;
    //     return;
    // }

    // if sandbox.cells[x][y].unwrap().tempature <= -60 {
    //     sandbox.cells[x][y].as_mut().unwrap().ptype = ParticleType::Cryotheum;
    //     return;
    // }

    let mut y2 = y + 1;
    while y2 < sandbox.height {
        match &sandbox.cells[x][y2] {
            Some(particle) => match particle.ptype {
                ParticleType::Sand => {
                    sandbox.cells[x][y] = None;
                    sandbox.cells[x][y2].as_mut().unwrap().ptype = ParticleType::WetSand;
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

pub fn update_acid(sandbox: &mut Sandbox, x: usize, y: usize) {
    if y != sandbox.height - 1 {
        if let Some(particle) = &sandbox.cells[x][y + 1] {
            if particle.ptype != ParticleType::Acid
                && particle.ptype != ParticleType::Iridium
                && particle.ptype != ParticleType::Replicator
            {
                sandbox.cells[x][y] = None;
                sandbox.cells[x][y + 1] = None;
                return;
            }
        }
    }
    if x != sandbox.width - 1 {
        if let Some(particle) = &sandbox.cells[x + 1][y] {
            if particle.ptype != ParticleType::Acid
                && particle.ptype != ParticleType::Iridium
                && particle.ptype != ParticleType::Replicator
            {
                sandbox.cells[x][y] = None;
                sandbox.cells[x + 1][y] = None;
                return;
            }
        }
    }
    if y != 0 {
        if let Some(particle) = &sandbox.cells[x][y - 1] {
            if particle.ptype != ParticleType::Acid
                && particle.ptype != ParticleType::Iridium
                && particle.ptype != ParticleType::Replicator
            {
                sandbox.cells[x][y] = None;
                sandbox.cells[x][y - 1] = None;
                return;
            }
        }
    }
    if x != 0 {
        if let Some(particle) = &sandbox.cells[x - 1][y] {
            if particle.ptype != ParticleType::Acid
                && particle.ptype != ParticleType::Iridium
                && particle.ptype != ParticleType::Replicator
            {
                sandbox.cells[x][y] = None;
                sandbox.cells[x - 1][y] = None;
                return;
            }
        }
    }
}

pub fn update_replicator(sandbox: &mut Sandbox, x: usize, y: usize) {
    if y < sandbox.height - 2 {
        if let Some(particle) = &sandbox.cells[x][y + 1] {
            if particle.ptype != ParticleType::Replicator {
                if sandbox.cells[x][y + 2].is_none() {
                    sandbox.cells[x][y + 2] = Some(Particle::new(particle.ptype));
                }
            }
        }
    }
    if x < sandbox.width - 2 {
        if let Some(particle) = &sandbox.cells[x + 1][y] {
            if particle.ptype != ParticleType::Replicator {
                if sandbox.cells[x + 2][y].is_none() {
                    sandbox.cells[x + 2][y] = Some(Particle::new(particle.ptype));
                }
            }
        }
    }
    if y > 1 {
        if let Some(particle) = &sandbox.cells[x][y - 1] {
            if particle.ptype != ParticleType::Replicator {
                if sandbox.cells[x][y - 2].is_none() {
                    sandbox.cells[x][y - 2] = Some(Particle::new(particle.ptype));
                }
            }
        }
    }
    if x > 1 {
        if let Some(particle) = &sandbox.cells[x - 1][y] {
            if particle.ptype != ParticleType::Replicator {
                if sandbox.cells[x - 2][y].is_none() {
                    sandbox.cells[x - 2][y] = Some(Particle::new(particle.ptype));
                }
            }
        }
    }
}

pub fn update_plant(sandbox: &mut Sandbox, x: usize, y: usize) {
    if y != sandbox.height - 1 {
        if let Some(particle) = sandbox.cells[x][y + 1] {
            if particle.ptype == ParticleType::WetSand {
                sandbox.cells[x][y].as_mut().unwrap().extra_data2 = 1;
            }
            if particle.ptype == ParticleType::Plant && particle.extra_data2 == 1 {
                sandbox.cells[x][y].as_mut().unwrap().extra_data2 = 1;
            }
        }
    }

    if y != 0 {
        if sandbox.cells[x][y].unwrap().extra_data2 == 1 {
            if sandbox.cells[x][y].unwrap().extra_data1 > 0 {
                if y % 2 == 0 && x != sandbox.width - 1 {
                    if sandbox.cells[x + 1][y - 1].is_none() {
                        let mut particle = Particle::new(ParticleType::Plant);
                        particle.extra_data1 = sandbox.cells[x][y].unwrap().extra_data1 - 1;
                        particle.extra_data2 = 1;
                        sandbox.cells[x + 1][y - 1] = Some(particle);
                        sandbox.cells[x][y].as_mut().unwrap().extra_data1 = -1;
                    }
                } else if x != 0 {
                    if sandbox.cells[x - 1][y - 1].is_none() {
                        let mut particle = Particle::new(ParticleType::Plant);
                        particle.extra_data1 = sandbox.cells[x][y].unwrap().extra_data1 - 1;
                        particle.extra_data2 = 1;
                        sandbox.cells[x - 1][y - 1] = Some(particle);
                        sandbox.cells[x][y].as_mut().unwrap().extra_data1 = -1;
                    }
                }
            }

            if sandbox.cells[x][y].unwrap().extra_data1 == 0 {
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
                            && new_x < sandbox.width as i32
                            && new_y > -1
                            && new_y < sandbox.height as i32
                        {
                            if sandbox.cells[new_x as usize][new_y as usize].is_none() {
                                let mut particle = Particle::new(ParticleType::Plant);
                                particle.extra_data1 = -1;
                                particle.extra_data2 = 1;
                                sandbox.cells[new_x as usize][new_y as usize] = Some(particle);
                                sandbox.cells[x][y].as_mut().unwrap().extra_data1 = -1;
                            }
                        }
                    }
                }
            }
        }
    }
}

// pub fn update_cryotheum(sandbox: &mut Sandbox, x: usize, y: usize) {
//     if sandbox.cells[x][y].unwrap().tempature >= -10 {
//         sandbox.cells[x][y].as_mut().unwrap().ptype = ParticleType::Water;
//         return;
//     }
// }

pub fn update_unstable(sandbox: &mut Sandbox, x: usize, y: usize) {
    // Increase tempature by 10 every half a second
    if sandbox.cells[x][y].unwrap().extra_data1 == 30 {
        sandbox.cells[x][y].as_mut().unwrap().extra_data1 = 0;
        sandbox.cells[x][y].as_mut().unwrap().tempature += 10;
    } else {
        sandbox.cells[x][y].as_mut().unwrap().extra_data1 += 1;
    }

    // When tempature >= 200 (10 seconds of existing), vaporize the surrounding area
    if sandbox.cells[x][y].unwrap().tempature >= 200 {
        for x_offset in -20..=20 {
            for y_offset in -20..=20 {
                let x = x as i16 + x_offset;
                let y = y as i16 + y_offset;
                if (0..(sandbox.width as i16)).contains(&x)
                    && (0..(sandbox.height as i16)).contains(&y)
                {
                    sandbox.cells[x as usize][y as usize] = None;
                }
            }
        }
    }
}
