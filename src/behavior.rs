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
                // Else mark for deletion if not above a replicator
                if particle.ptype != ParticleType::Replicator {
                    sandbox.cells[x][y].as_mut().unwrap().extra_data2 = -1;
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
                        let mut particle = Particle::new(ParticleType::Plant, &mut sandbox.rng);
                        particle.extra_data1 = sandbox.cells[x][y].unwrap().extra_data1 - 1;
                        particle.extra_data2 = 1;
                        sandbox.cells[x + 1][y - 1] = Some(particle);
                        sandbox.cells[x][y].as_mut().unwrap().extra_data1 = -1;
                    }
                } else if x != 0 {
                    if sandbox.cells[x - 1][y - 1].is_none() {
                        let mut particle = Particle::new(ParticleType::Plant, &mut sandbox.rng);
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
                                let mut particle =
                                    Particle::new(ParticleType::Plant, &mut sandbox.rng);
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

        for x_offset in (-10..=10).skip(0) {
            for y_offset in (-10..=10).skip(0) {
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
            ParticleType::Replicator => true,
            ParticleType::Plant => true,
            ParticleType::Cryotheum => true,
            ParticleType::Unstable => true,
            ParticleType::Electricity => true,
            ParticleType::Glass => true,
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

    // When tempature >= 200 (10 seconds of existing), vaporize the surrounding area
    if particle.tempature >= 200 {
        for x_offset in -20..=20 {
            for y_offset in -20..=20 {
                let x = x as isize + x_offset;
                let y = y as isize + y_offset;
                if (0..(SIMULATION_WIDTH as isize)).contains(&x)
                    && (0..(SIMULATION_HEIGHT as isize)).contains(&y)
                {
                    if let Some(particle) = sandbox.cells[x as usize][y as usize] {
                        if vaporized_by_unstable(particle.ptype) {
                            sandbox.cells[x as usize][y as usize] = None;
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
