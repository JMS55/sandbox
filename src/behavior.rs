use crate::sandbox::{Particle, ParticleType, Sandbox, SIMULATION_HEIGHT, SIMULATION_WIDTH};
use rand::Rng;

pub fn move_solid(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    // Move 1 down if able
    if y != SIMULATION_HEIGHT - 1 {
        if sandbox.cells[x][y + 1].is_none() {
            sandbox.cells[x][y + 1] = sandbox.cells[x][y].take();
            return (x, y + 1);
        }
    }
    (x, y)
}

pub fn move_powder(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    if y != SIMULATION_HEIGHT - 1 {
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
        if x != SIMULATION_WIDTH - 1 {
            if sandbox.cells[x + 1][y + 1].is_none() && sandbox.cells[x + 1][y].is_none() {
                sandbox.cells[x + 1][y + 1] = sandbox.cells[x][y].take();
                return (x + 1, y + 1);
            }
        }
    }
    (x, y)
}

pub fn move_liquid(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    if y != SIMULATION_HEIGHT - 1 {
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
        if x != SIMULATION_WIDTH - 1 {
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
    if x != SIMULATION_WIDTH - 1 {
        if sandbox.cells[x + 1][y].is_none() {
            sandbox.cells[x + 1][y] = sandbox.cells[x][y].take();
            return (x + 1, y);
        }
    }
    (x, y)
}

pub fn move_gas(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    if y != 0 && sandbox.rng.gen_bool(0.5) {
        // Move 1 up if able
        if sandbox.cells[x][y - 1].is_none() {
            sandbox.cells[x][y - 1] = sandbox.cells[x][y].take();
            return (x, y - 1);
        }
        // Else move 1 up and left if able
        if x != 0 {
            if sandbox.cells[x - 1][y - 1].is_none() && sandbox.cells[x - 1][y].is_none() {
                sandbox.cells[x - 1][y - 1] = sandbox.cells[x][y].take();
                return (x - 1, y - 1);
            }
        }
        // Else move 1 up and right if able
        if x != SIMULATION_WIDTH - 1 {
            if sandbox.cells[x + 1][y - 1].is_none() && sandbox.cells[x + 1][y].is_none() {
                sandbox.cells[x + 1][y - 1] = sandbox.cells[x][y].take();
                return (x + 1, y - 1);
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
    if x != SIMULATION_WIDTH - 1 {
        if sandbox.cells[x + 1][y].is_none() {
            sandbox.cells[x + 1][y] = sandbox.cells[x][y].take();
            return (x + 1, y);
        }
    }
    (x, y)
}

pub fn move_electricity(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    // Try switching with an adjacent water particle in the last direction moved
    if sandbox.cells[x][y].unwrap().extra_data2 != 0 {
        sandbox.cells[x][y].as_mut().unwrap().extra_data2 -= 1;
        let offset = match sandbox.cells[x][y].unwrap().extra_data1 {
            0 => (1, 0),
            1 => (-1, 0),
            2 => (0, 1),
            3 => (0, -1),
            _ => unreachable!(),
        };
        let (x2, y2) = (x as isize + offset.0, y as isize + offset.1);
        if (0..(SIMULATION_WIDTH as isize)).contains(&x2)
            && (0..(SIMULATION_HEIGHT as isize)).contains(&y2)
        {
            let x2 = x2 as usize;
            let y2 = y2 as usize;
            if let Some(particle) = sandbox.cells[x2][y2] {
                if particle.ptype == ParticleType::Water {
                    let temp = sandbox.cells[x][y];
                    sandbox.cells[x][y] = sandbox.cells[x2][y2];
                    sandbox.cells[x2][y2] = temp;
                    return (x2, y2);
                }
            }
        }
        return (x, y);
    }

    // Else try switching with an adjacent water particle in a random direction
    let mut offsets = vec![(1, 0), (-1, 0), (0, 1), (0, -1)];
    while !offsets.is_empty() {
        let offset = offsets.remove(sandbox.rng.gen_range(0, offsets.len()));
        let (x2, y2) = (x as isize + offset.0, y as isize + offset.1);
        if (0..(SIMULATION_WIDTH as isize)).contains(&x2)
            && (0..(SIMULATION_HEIGHT as isize)).contains(&y2)
        {
            let x2 = x2 as usize;
            let y2 = y2 as usize;
            if let Some(particle) = sandbox.cells[x2][y2] {
                if particle.ptype == ParticleType::Water {
                    let temp = sandbox.cells[x][y];
                    temp.unwrap().extra_data1 = match offset {
                        (1, 0) => 0,
                        (-1, 0) => 1,
                        (0, 1) => 2,
                        (0, -1) => 3,
                        _ => unreachable!(),
                    };
                    temp.unwrap().extra_data2 = 100;
                    sandbox.cells[x][y] = sandbox.cells[x2][y2];
                    sandbox.cells[x2][y2] = temp;
                    return (x2, y2);
                }
            }
        }
    }

    if y != SIMULATION_HEIGHT - 1 {
        match sandbox.cells[x][y + 1] {
            None => {
                // Else move 1 down if able
                sandbox.cells[x][y + 1] = sandbox.cells[x][y].take();
                return (x, y + 1);
            }
            Some(particle) => {
                // Else mark for deletion if not above a Replicator
                if particle.ptype != ParticleType::Replicator {
                    sandbox.cells[x][y].as_mut().unwrap().extra_data2 = -1;
                }
            }
        }
    } else {
        // Else mark for deletion if in the last row
        sandbox.cells[x][y].as_mut().unwrap().extra_data2 = -1;
    }

    (x, y)
}

pub fn move_life(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    // Fall down if able
    if y != SIMULATION_HEIGHT - 1 {
        if sandbox.cells[x][y + 1].is_none() {
            // And increase the falling counter by 1
            sandbox.cells[x][y].as_mut().unwrap().extra_data1 = sandbox.cells[x][y]
                .as_mut()
                .unwrap()
                .extra_data1
                .saturating_add(1);
            sandbox.cells[x][y + 1] = sandbox.cells[x][y].take();
            return (x, y + 1);
        }
    }

    // Kill the particle if the falling counter > 60, else reset it
    if sandbox.cells[x][y].unwrap().extra_data1 > 60 {
        sandbox.cells[x][y].as_mut().unwrap().extra_data2 = 1;
    } else {
        sandbox.cells[x][y].as_mut().unwrap().extra_data1 = 0;
    }

    // And if still living
    if sandbox.cells[x][y].unwrap().extra_data2 == 0 {
        let drop_is_short_enough = |x: usize, y: usize| -> bool {
            let mut y2 = y + 1;
            let mut drop_size = 0;
            while y2 < SIMULATION_HEIGHT {
                if sandbox.cells[x][y2].is_none() {
                    drop_size += 1;
                } else {
                    break;
                }
                y2 += 1;
            }
            drop_size < 31
        };

        // Move left if able and the drop is short enough
        if x != 0 {
            if drop_is_short_enough(x - 1, y) {
                if sandbox.cells[x - 1][y].is_none() {
                    sandbox.cells[x - 1][y] = sandbox.cells[x][y].take();
                    return (x - 1, y);
                }
            }
        }
        // Else move right if able and the drop is short enough
        if x != SIMULATION_WIDTH - 1 {
            if drop_is_short_enough(x + 1, y) {
                if sandbox.cells[x + 1][y].is_none() {
                    sandbox.cells[x + 1][y] = sandbox.cells[x][y].take();
                    return (x + 1, y);
                }
            }
        }
    }

    (x, y)
}

pub fn update_sand(sandbox: &mut Sandbox, x: usize, y: usize) {
    if sandbox.cells[x][y].unwrap().tempature >= 120 {
        sandbox.cells[x][y].as_mut().unwrap().ptype = ParticleType::Glass;
    }
}

pub fn update_water(sandbox: &mut Sandbox, x: usize, y: usize) {
    if sandbox.cells[x][y].unwrap().tempature >= 100 {
        sandbox.cells[x][y] = None;
        return;
    }

    let mut y2 = y + 1;
    while y2 < SIMULATION_HEIGHT {
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
    fn dissolved_by_acid(ptype: ParticleType) -> bool {
        match ptype {
            ParticleType::Sand => true,
            ParticleType::WetSand => true,
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
            ParticleType::Blood => true,
            ParticleType::Smoke => false,
        }
    }

    if y != SIMULATION_HEIGHT - 1 {
        if let Some(particle) = &sandbox.cells[x][y + 1] {
            if dissolved_by_acid(particle.ptype) {
                sandbox.cells[x][y] = None;
                sandbox.cells[x][y + 1] = None;
                return;
            }
        }
    }
    if x != SIMULATION_WIDTH - 1 {
        if let Some(particle) = &sandbox.cells[x + 1][y] {
            if dissolved_by_acid(particle.ptype) {
                sandbox.cells[x][y] = None;
                sandbox.cells[x + 1][y] = None;
                return;
            }
        }
    }
    if y != 0 {
        if let Some(particle) = &sandbox.cells[x][y - 1] {
            if dissolved_by_acid(particle.ptype) {
                sandbox.cells[x][y] = None;
                sandbox.cells[x][y - 1] = None;
                return;
            }
        }
    }
    if x != 0 {
        if let Some(particle) = &sandbox.cells[x - 1][y] {
            if dissolved_by_acid(particle.ptype) {
                sandbox.cells[x][y] = None;
                sandbox.cells[x - 1][y] = None;
                return;
            }
        }
    }
}

pub fn update_replicator(sandbox: &mut Sandbox, x: usize, y: usize) {
    if y < SIMULATION_HEIGHT - 2 {
        if let Some(particle) = sandbox.cells[x][y + 1] {
            if particle.ptype != ParticleType::Replicator {
                if sandbox.cells[x][y + 2].is_none() {
                    let mut particle = particle.clone();
                    particle.color_offset = sandbox.rng.gen_range(-10, 11);
                    sandbox.cells[x][y + 2] = Some(particle);
                }
            }
        }
    }
    if x < SIMULATION_WIDTH - 2 {
        if let Some(particle) = sandbox.cells[x + 1][y] {
            if particle.ptype != ParticleType::Replicator {
                if sandbox.cells[x + 2][y].is_none() {
                    let mut particle = particle.clone();
                    particle.color_offset = sandbox.rng.gen_range(-10, 11);
                    sandbox.cells[x + 2][y] = Some(particle);
                }
            }
        }
    }
    if y > 1 {
        if let Some(particle) = sandbox.cells[x][y - 1] {
            if particle.ptype != ParticleType::Replicator {
                if sandbox.cells[x][y - 2].is_none() {
                    let mut particle = particle.clone();
                    particle.color_offset = sandbox.rng.gen_range(-10, 11);
                    sandbox.cells[x][y - 2] = Some(particle);
                }
            }
        }
    }
    if x > 1 {
        if let Some(particle) = sandbox.cells[x - 1][y] {
            if particle.ptype != ParticleType::Replicator {
                if sandbox.cells[x - 2][y].is_none() {
                    let mut particle = particle.clone();
                    particle.color_offset = sandbox.rng.gen_range(-10, 11);
                    sandbox.cells[x - 2][y] = Some(particle);
                }
            }
        }
    }
}

pub fn update_plant(sandbox: &mut Sandbox, x: usize, y: usize) {
    if y != SIMULATION_HEIGHT - 1 {
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
                if y % 2 == 0 && x != SIMULATION_WIDTH - 1 {
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
                        let new_x = x as isize + x_offset;
                        let new_y = y as isize + y_offset;
                        if new_x > -1
                            && new_x < SIMULATION_WIDTH as isize
                            && new_y > -1
                            && new_y < SIMULATION_HEIGHT as isize
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

// Check if tempature >= 0, and if so, wait 1/5th of a second, and then delete itself and freeze around it
pub fn update_cryotheum(sandbox: &mut Sandbox, x: usize, y: usize) {
    fn affected_by_cryotheum_coldsnap(ptype: ParticleType) -> bool {
        match ptype {
            ParticleType::Sand => true,
            ParticleType::WetSand => true,
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
            ParticleType::Blood => true,
            ParticleType::Smoke => false,
        }
    }

    if sandbox.cells[x][y].unwrap().tempature >= 0 {
        sandbox.cells[x][y].as_mut().unwrap().extra_data1 = 13;
        return;
    }

    if sandbox.cells[x][y].unwrap().extra_data1 > 1 {
        sandbox.cells[x][y].as_mut().unwrap().extra_data1 -= 1;
    }

    if sandbox.cells[x][y].unwrap().extra_data1 == 1 {
        sandbox.cells[x][y] = None;

        for x_offset in (-15..=15).skip(0) {
            for y_offset in (-15..=15).skip(0) {
                if x_offset * x_offset + y_offset * y_offset > 15 * 15 {
                    continue;
                }
                let x = x as isize + x_offset;
                let y = y as isize + y_offset;
                if (0..(SIMULATION_WIDTH as isize)).contains(&x)
                    && (0..(SIMULATION_HEIGHT as isize)).contains(&y)
                {
                    if let Some(particle) = sandbox.cells[x as usize][y as usize].as_mut() {
                        if affected_by_cryotheum_coldsnap(particle.ptype) {
                            particle.tempature -= 100;
                        }
                    }
                }
            }
        }
    }
}

pub fn update_unstable(sandbox: &mut Sandbox, x: usize, y: usize) {
    fn vaporized_by_unstable(ptype: ParticleType) -> bool {
        match ptype {
            ParticleType::Sand => true,
            ParticleType::WetSand => true,
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
            ParticleType::Blood => true,
            ParticleType::Smoke => false,
        }
    }

    // Increase tempature by 10 every half a second
    let mut particle = sandbox.cells[x][y].as_mut().unwrap();
    if particle.extra_data1 == 30 {
        particle.extra_data1 = 0;
        particle.tempature += 10;
    } else {
        particle.extra_data1 += 1;
    }

    // When tempature >= 200 (10 seconds of existing), replace the surrounding area with Smoke
    if particle.tempature >= 200 {
        for x_offset in -30..=30 {
            for y_offset in -30..=30 {
                let x = x as isize + x_offset;
                let y = y as isize + y_offset;
                if (0..(SIMULATION_WIDTH as isize)).contains(&x)
                    && (0..(SIMULATION_HEIGHT as isize)).contains(&y)
                {
                    if let Some(particle) = sandbox.cells[x as usize][y as usize] {
                        if vaporized_by_unstable(particle.ptype) {
                            sandbox.cells[x as usize][y as usize] =
                                Some(Particle::new(ParticleType::Smoke));
                        }
                    }
                }
            }
        }
    }
}

pub fn update_electricity(sandbox: &mut Sandbox, x: usize, y: usize) {
    // If this particle was unable able to move, delete it
    if sandbox.cells[x][y].unwrap().extra_data2 == -1 {
        sandbox.cells[x][y] = None;
    }
}

pub fn update_life(sandbox: &mut Sandbox, x: usize, y: usize) {
    // When tempature less than -50, or greater than 50, this particle dies
    let mut particle = sandbox.cells[x][y].as_mut().unwrap();
    if particle.tempature < -50 || particle.tempature > 50 {
        particle.extra_data2 = 1;
    }

    // When alive and touching a Plant particle, chance to turn it into a new Life particle
    if particle.extra_data2 == 0 {
        if y != SIMULATION_HEIGHT - 1 {
            if let Some(particle) = &sandbox.cells[x][y + 1] {
                if particle.ptype == ParticleType::Plant && sandbox.rng.gen_bool(0.4) {
                    sandbox.cells[x][y + 1] = Some(Particle::new(ParticleType::Life));
                    return;
                }
            }
        }
        if x != SIMULATION_WIDTH - 1 {
            if let Some(particle) = &sandbox.cells[x + 1][y] {
                if particle.ptype == ParticleType::Plant && sandbox.rng.gen_bool(0.4) {
                    sandbox.cells[x + 1][y] = Some(Particle::new(ParticleType::Life));
                    return;
                }
            }
        }
        if y != 0 {
            if let Some(particle) = &sandbox.cells[x][y - 1] {
                if particle.ptype == ParticleType::Plant && sandbox.rng.gen_bool(0.4) {
                    sandbox.cells[x][y - 1] = Some(Particle::new(ParticleType::Life));
                    return;
                }
            }
        }
        if x != 0 {
            if let Some(particle) = &sandbox.cells[x - 1][y] {
                if particle.ptype == ParticleType::Plant && sandbox.rng.gen_bool(0.4) {
                    sandbox.cells[x - 1][y] = Some(Particle::new(ParticleType::Life));
                    return;
                }
            }
        }
    } else {
        // Else if dead, if enough particles stacked above, chance to turn into blood
        let mut offset = 0;
        let mut particles_above = 0;
        while y > 0 {
            offset += 1;
            if sandbox.cells[x][y - offset].is_some() {
                particles_above += 1;
            } else {
                break;
            }
        }
        if particles_above > 30 && sandbox.rng.gen_bool(0.1) {
            sandbox.cells[x][y] = Some(Particle::new(ParticleType::Blood));
        }
    }
}

pub fn update_blood(sandbox: &mut Sandbox, x: usize, y: usize) {
    // Evaporate above a certain tempature
    if sandbox.cells[x][y].unwrap().tempature >= 137 {
        sandbox.cells[x][y] = None;
    }
}

pub fn update_smoke(sandbox: &mut Sandbox, x: usize, y: usize) {
    let mut particle = sandbox.cells[x][y].as_mut().unwrap();
    if particle.extra_data1 == 0 {
        particle.extra_data2 -= 1;
        if particle.extra_data2 == 0 {
            sandbox.cells[x][y] = None;
        }
    } else {
        particle.extra_data1 -= 1;
    }
}
