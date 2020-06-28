use crate::particle::{Particle, ParticleType};
use crate::sandbox::{Sandbox, SIMULATION_HEIGHT, SIMULATION_WIDTH};
use rand::Rng;

pub fn move_solid(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    // Move 1 down if able
    if y != SIMULATION_HEIGHT - 1 {
        if sandbox[x][y + 1].is_none() {
            sandbox[x][y + 1] = sandbox[x][y].take();
            return (x, y + 1);
        }
    }
    (x, y)
}

pub fn move_powder(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    if y != SIMULATION_HEIGHT - 1 {
        // Move 1 down if able
        if sandbox[x][y + 1].is_none() {
            sandbox[x][y + 1] = sandbox[x][y].take();
            return (x, y + 1);
        }
        // Else move 1 down and left if able
        if x != 0 {
            if sandbox[x - 1][y + 1].is_none() && sandbox[x - 1][y].is_none() {
                sandbox[x - 1][y + 1] = sandbox[x][y].take();
                return (x - 1, y + 1);
            }
        }
        // Else move 1 down and right if able
        if x != SIMULATION_WIDTH - 1 {
            if sandbox[x + 1][y + 1].is_none() && sandbox[x + 1][y].is_none() {
                sandbox[x + 1][y + 1] = sandbox[x][y].take();
                return (x + 1, y + 1);
            }
        }
    }
    (x, y)
}

pub fn move_liquid(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    if y != SIMULATION_HEIGHT - 1 {
        // Move 1 down if able
        if sandbox[x][y + 1].is_none() {
            sandbox[x][y + 1] = sandbox[x][y].take();
            return (x, y + 1);
        }
        // Else move 1 down and left if able
        if x != 0 {
            if sandbox[x - 1][y + 1].is_none() && sandbox[x - 1][y].is_none() {
                sandbox[x - 1][y + 1] = sandbox[x][y].take();
                return (x - 1, y + 1);
            }
        }
        // Else move 1 down and right if able
        if x != SIMULATION_WIDTH - 1 {
            if sandbox[x + 1][y + 1].is_none() && sandbox[x + 1][y].is_none() {
                sandbox[x + 1][y + 1] = sandbox[x][y].take();
                return (x + 1, y + 1);
            }
        }
    }
    // Else move left if able
    if x != 0 {
        if sandbox[x - 1][y].is_none() {
            sandbox[x - 1][y] = sandbox[x][y].take();
            return (x - 1, y);
        }
    }
    // Else move right if able
    if x != SIMULATION_WIDTH - 1 {
        if sandbox[x + 1][y].is_none() {
            sandbox[x + 1][y] = sandbox[x][y].take();
            return (x + 1, y);
        }
    }
    (x, y)
}

pub fn move_gas(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    if y != 0 && sandbox.rng.gen_bool(0.5) {
        // Move 1 up if able
        if sandbox[x][y - 1].is_none() {
            sandbox[x][y - 1] = sandbox[x][y].take();
            return (x, y - 1);
        }
        // Else move 1 up and left if able
        if x != 0 {
            if sandbox[x - 1][y - 1].is_none() && sandbox[x - 1][y].is_none() {
                sandbox[x - 1][y - 1] = sandbox[x][y].take();
                return (x - 1, y - 1);
            }
        }
        // Else move 1 up and right if able
        if x != SIMULATION_WIDTH - 1 {
            if sandbox[x + 1][y - 1].is_none() && sandbox[x + 1][y].is_none() {
                sandbox[x + 1][y - 1] = sandbox[x][y].take();
                return (x + 1, y - 1);
            }
        }
    }
    // Else move left if able
    if x != 0 {
        if sandbox[x - 1][y].is_none() {
            sandbox[x - 1][y] = sandbox[x][y].take();
            return (x - 1, y);
        }
    }
    // Else move right if able
    if x != SIMULATION_WIDTH - 1 {
        if sandbox[x + 1][y].is_none() {
            sandbox[x + 1][y] = sandbox[x][y].take();
            return (x + 1, y);
        }
    }
    (x, y)
}

pub fn move_electricity(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    // Try switching with an adjacent water particle in the last direction moved
    if sandbox[x][y].unwrap().extra_data2 != 0 {
        sandbox[x][y].as_mut().unwrap().extra_data2 -= 1;
        let offset = match sandbox[x][y].unwrap().extra_data1 {
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
            if let Some(particle) = sandbox[x2][y2] {
                if particle.ptype == ParticleType::Water {
                    let temp = sandbox[x][y];
                    sandbox[x][y] = sandbox[x2][y2];
                    sandbox[x2][y2] = temp;
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
            if let Some(particle) = sandbox[x2][y2] {
                if particle.ptype == ParticleType::Water {
                    let temp = sandbox[x][y];
                    temp.unwrap().extra_data1 = match offset {
                        (1, 0) => 0,
                        (-1, 0) => 1,
                        (0, 1) => 2,
                        (0, -1) => 3,
                        _ => unreachable!(),
                    };
                    temp.unwrap().extra_data2 = 100;
                    sandbox[x][y] = sandbox[x2][y2];
                    sandbox[x2][y2] = temp;
                    return (x2, y2);
                }
            }
        }
    }

    if y != SIMULATION_HEIGHT - 1 {
        match sandbox[x][y + 1] {
            None => {
                // Else move 1 down if able
                sandbox[x][y + 1] = sandbox[x][y].take();
                return (x, y + 1);
            }
            Some(particle) => {
                // Else mark for deletion if not above a Replicator
                if particle.ptype != ParticleType::Replicator {
                    sandbox[x][y].as_mut().unwrap().extra_data2 = -1;
                }
            }
        }
    } else {
        // Else mark for deletion if in the last row
        sandbox[x][y].as_mut().unwrap().extra_data2 = -1;
    }

    (x, y)
}

pub fn move_life(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    // Fall down if able
    if y != SIMULATION_HEIGHT - 1 {
        if sandbox[x][y + 1].is_none() {
            // And increase the falling counter by 1
            sandbox[x][y].as_mut().unwrap().extra_data1 = sandbox[x][y]
                .as_mut()
                .unwrap()
                .extra_data1
                .saturating_add(1);
            sandbox[x][y + 1] = sandbox[x][y].take();
            return (x, y + 1);
        }
    }

    // Kill the particle if the falling counter > 60, else reset it
    if sandbox[x][y].unwrap().extra_data1 > 60 {
        sandbox[x][y].as_mut().unwrap().extra_data2 = 1;
    } else {
        sandbox[x][y].as_mut().unwrap().extra_data1 = 0;
    }

    // And if still living
    if sandbox[x][y].unwrap().extra_data2 == 0 {
        let drop_is_short_enough = |x: usize, y: usize| -> bool {
            let mut y2 = y + 1;
            let mut drop_size = 0;
            while y2 < SIMULATION_HEIGHT {
                if sandbox[x][y2].is_none() {
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
                if sandbox[x - 1][y].is_none() {
                    sandbox[x - 1][y] = sandbox[x][y].take();
                    return (x - 1, y);
                }
            }
        }
        // Else move right if able and the drop is short enough
        if x != SIMULATION_WIDTH - 1 {
            if drop_is_short_enough(x + 1, y) {
                if sandbox[x + 1][y].is_none() {
                    sandbox[x + 1][y] = sandbox[x][y].take();
                    return (x + 1, y);
                }
            }
        }
    }

    (x, y)
}

pub fn move_fire(sandbox: &mut Sandbox, x: usize, y: usize) -> (usize, usize) {
    let new_position = move_gas(sandbox, x, y);
    let extra_data2 = &mut sandbox[new_position.0][new_position.1]
        .as_mut()
        .unwrap()
        .extra_data2;
    if new_position == (x, y) {
        *extra_data2 += 1;
    } else {
        *extra_data2 = extra_data2.saturating_sub(1);
    }
    new_position
}

pub fn update_sand(sandbox: &mut Sandbox, x: usize, y: usize) {
    // When wet and tempature >= 30, dry out
    if sandbox[x][y].unwrap().extra_data1 == 1 && sandbox[x][y].unwrap().tempature >= 30 {
        sandbox[x][y].as_mut().unwrap().extra_data1 = 0;
    }

    // When tempature >= 120, turn into Glass
    if sandbox[x][y].unwrap().tempature >= 120 {
        sandbox[x][y].as_mut().unwrap().ptype = ParticleType::Glass;
    }
}

pub fn update_water(sandbox: &mut Sandbox, x: usize, y: usize) {
    if sandbox[x][y].unwrap().tempature >= 100 {
        sandbox[x][y] = None;
        return;
    }

    let mut y2 = y + 1;
    while y2 < SIMULATION_HEIGHT {
        match &sandbox[x][y2] {
            Some(particle) => {
                if particle.ptype == ParticleType::Sand && particle.extra_data1 == 0 {
                    sandbox[x][y] = None;
                    sandbox[x][y2].as_mut().unwrap().extra_data1 = 1;
                    return;
                }
            }
            None => return,
        }
        y2 += 1;
    }
}

pub fn update_acid(sandbox: &mut Sandbox, x: usize, y: usize) {
    if y != SIMULATION_HEIGHT - 1 {
        if let Some(particle) = &sandbox[x][y + 1] {
            if particle.dissolved_by_acid() {
                sandbox[x][y] = None;
                sandbox[x][y + 1] = None;
                return;
            }
        }
    }
    if x != SIMULATION_WIDTH - 1 {
        if let Some(particle) = &sandbox[x + 1][y] {
            if particle.dissolved_by_acid() {
                sandbox[x][y] = None;
                sandbox[x + 1][y] = None;
                return;
            }
        }
    }
    if y != 0 {
        if let Some(particle) = &sandbox[x][y - 1] {
            if particle.dissolved_by_acid() {
                sandbox[x][y] = None;
                sandbox[x][y - 1] = None;
                return;
            }
        }
    }
    if x != 0 {
        if let Some(particle) = &sandbox[x - 1][y] {
            if particle.dissolved_by_acid() {
                sandbox[x][y] = None;
                sandbox[x - 1][y] = None;
                return;
            }
        }
    }
}

pub fn update_replicator(sandbox: &mut Sandbox, x: usize, y: usize) {
    if y < SIMULATION_HEIGHT - 2 {
        if let Some(particle) = sandbox[x][y + 1] {
            if particle.ptype != ParticleType::Replicator {
                if sandbox[x][y + 2].is_none() {
                    let mut particle = particle.clone();
                    particle.color_offset = sandbox.rng.gen_range(-10, 11);
                    sandbox[x][y + 2] = Some(particle);
                }
            }
        }
    }
    if x < SIMULATION_WIDTH - 2 {
        if let Some(particle) = sandbox[x + 1][y] {
            if particle.ptype != ParticleType::Replicator {
                if sandbox[x + 2][y].is_none() {
                    let mut particle = particle.clone();
                    particle.color_offset = sandbox.rng.gen_range(-10, 11);
                    sandbox[x + 2][y] = Some(particle);
                }
            }
        }
    }
    if y > 1 {
        if let Some(particle) = sandbox[x][y - 1] {
            if particle.ptype != ParticleType::Replicator {
                if sandbox[x][y - 2].is_none() {
                    let mut particle = particle.clone();
                    particle.color_offset = sandbox.rng.gen_range(-10, 11);
                    sandbox[x][y - 2] = Some(particle);
                }
            }
        }
    }
    if x > 1 {
        if let Some(particle) = sandbox[x - 1][y] {
            if particle.ptype != ParticleType::Replicator {
                if sandbox[x - 2][y].is_none() {
                    let mut particle = particle.clone();
                    particle.color_offset = sandbox.rng.gen_range(-10, 11);
                    sandbox[x - 2][y] = Some(particle);
                }
            }
        }
    }
}

pub fn update_plant(sandbox: &mut Sandbox, x: usize, y: usize) {
    // If tempature > 100, turn into Fire
    if sandbox[x][y].unwrap().tempature > 100 {
        sandbox[x][y].as_mut().unwrap().ptype = ParticleType::Fire;
        return;
    }

    // If above wet Sand or another Plant that's growable, mark as growable (extra_data2 = 1)
    if y != SIMULATION_HEIGHT - 1 {
        if let Some(particle) = sandbox[x][y + 1] {
            if particle.ptype == ParticleType::Sand && particle.extra_data1 == 1 {
                sandbox[x][y].as_mut().unwrap().extra_data2 = 1;
            }
            if particle.ptype == ParticleType::Plant && particle.extra_data2 == 1 {
                sandbox[x][y].as_mut().unwrap().extra_data2 = 1;
            }
        }
    }

    // If growable and growing_time_left (extra_data_2) > 0, create another Plant nearby with 1 less growing_time_left
    if y != 0 {
        if sandbox[x][y].unwrap().extra_data2 == 1 {
            let extra_data1 = sandbox[x][y].unwrap().extra_data1;
            if extra_data1 > 0 {
                let x_offset = sandbox.rng.gen_range(-1, 2);
                let y_offset = sandbox.rng.gen_range(-2, 3);
                let x = x as isize + x_offset;
                let y = y as isize + y_offset;
                if (0..(SIMULATION_WIDTH as isize)).contains(&x)
                    && (0..(SIMULATION_HEIGHT as isize)).contains(&y)
                {
                    if sandbox[x as usize][y as usize].is_none() {
                        let mut particle = Particle::new(ParticleType::Plant);
                        particle.extra_data1 = extra_data1 - 1;
                        particle.extra_data2 = 1;
                        sandbox[x as usize][y as usize] = Some(particle);
                    }
                }
            }
        }
    }
}

// Check if tempature >= 0, and if so, wait 1/5th of a second, and then delete itself and freeze around it
pub fn update_cryotheum(sandbox: &mut Sandbox, x: usize, y: usize) {
    if sandbox[x][y].unwrap().tempature >= 0 {
        sandbox[x][y].as_mut().unwrap().extra_data1 = 13;
        return;
    }

    if sandbox[x][y].unwrap().extra_data1 > 1 {
        sandbox[x][y].as_mut().unwrap().extra_data1 -= 1;
    }

    if sandbox[x][y].unwrap().extra_data1 == 1 {
        sandbox[x][y] = None;

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
                    if let Some(particle) = sandbox[x as usize][y as usize].as_mut() {
                        if particle.affected_by_cryotheum_coldsnap() {
                            particle.tempature -= 100;
                        }
                    }
                }
            }
        }
    }
}

pub fn update_unstable(sandbox: &mut Sandbox, x: usize, y: usize) {
    // Increase tempature by 10 every half a second
    let mut particle = sandbox[x][y].as_mut().unwrap();
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
                    if let Some(particle) = sandbox[x as usize][y as usize] {
                        if particle.can_be_vaporized_by_unstable() {
                            sandbox[x as usize][y as usize] =
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
    if sandbox[x][y].unwrap().extra_data2 == -1 {
        sandbox[x][y] = None;
    }
}

pub fn update_life(sandbox: &mut Sandbox, x: usize, y: usize) {
    // When tempature less than -50, or greater than 50, this particle dies
    let mut particle = sandbox[x][y].as_mut().unwrap();
    if particle.tempature < -50 || particle.tempature > 50 {
        particle.extra_data2 = 1;
    }

    // When alive and touching a Plant particle, chance to turn it into a new Life particle
    // When alive and touching a Blood particle, turn into SuperLife
    if particle.extra_data2 == 0 {
        if y != SIMULATION_HEIGHT - 1 {
            if let Some(particle) = sandbox[x][y + 1] {
                if particle.ptype == ParticleType::Plant && sandbox.rng.gen_bool(0.2) {
                    sandbox[x][y + 1] = Some(Particle::new(ParticleType::Life));
                    return;
                }
                if particle.ptype == ParticleType::Blood
                    && sandbox[x][y].unwrap().ptype != ParticleType::SuperLife
                {
                    sandbox[x][y + 1] = None;
                    sandbox[x][y].as_mut().unwrap().ptype = ParticleType::SuperLife;
                    return;
                }
            }
        }
        if x != SIMULATION_WIDTH - 1 {
            if let Some(particle) = sandbox[x + 1][y] {
                if particle.ptype == ParticleType::Plant && sandbox.rng.gen_bool(0.2) {
                    sandbox[x + 1][y] = Some(Particle::new(ParticleType::Life));
                    return;
                }
                if particle.ptype == ParticleType::Blood
                    && sandbox[x][y].unwrap().ptype != ParticleType::SuperLife
                {
                    sandbox[x + 1][y] = None;
                    sandbox[x][y].as_mut().unwrap().ptype = ParticleType::SuperLife;
                    return;
                }
            }
        }
        if y != 0 {
            if let Some(particle) = sandbox[x][y - 1] {
                if particle.ptype == ParticleType::Plant && sandbox.rng.gen_bool(0.2) {
                    sandbox[x][y - 1] = Some(Particle::new(ParticleType::Life));
                    return;
                }
                if particle.ptype == ParticleType::Blood
                    && sandbox[x][y].unwrap().ptype != ParticleType::SuperLife
                {
                    sandbox[x][y - 1] = None;
                    sandbox[x][y].as_mut().unwrap().ptype = ParticleType::SuperLife;
                    return;
                }
            }
        }
        if x != 0 {
            if let Some(particle) = sandbox[x - 1][y] {
                if particle.ptype == ParticleType::Plant && sandbox.rng.gen_bool(0.2) {
                    sandbox[x - 1][y] = Some(Particle::new(ParticleType::Life));
                    return;
                }
                if particle.ptype == ParticleType::Blood
                    && sandbox[x][y].unwrap().ptype != ParticleType::SuperLife
                {
                    sandbox[x - 1][y] = None;
                    sandbox[x][y].as_mut().unwrap().ptype = ParticleType::SuperLife;
                    return;
                }
            }
        }
    } else {
        // Else if dead, and if enough particles stacked above, chance to turn into blood
        let mut count = 1;
        while count <= y {
            match sandbox[x][y - count] {
                Some(_) => count += 1,
                None => break,
            }
        }
        count -= 1;
        if count > 30 && sandbox.rng.gen_bool(0.1) {
            sandbox[x][y] = Some(Particle::new(ParticleType::Blood));
        }
    }
}

pub fn update_blood(sandbox: &mut Sandbox, x: usize, y: usize) {
    // Evaporate above a certain tempature
    if sandbox[x][y].unwrap().tempature >= 137 {
        sandbox[x][y] = None;
    }
}

pub fn update_smoke(sandbox: &mut Sandbox, x: usize, y: usize) {
    let mut particle = sandbox[x][y].as_mut().unwrap();
    if particle.extra_data1 == 0 {
        particle.extra_data2 -= 1;
        if particle.extra_data2 == 0 {
            sandbox[x][y] = None;
        }
    } else {
        particle.extra_data1 -= 1;
    }
}

pub fn update_fire(sandbox: &mut Sandbox, x: usize, y: usize) {
    // When this particle hasn't moved for more than 1/2 a second or tempature < 40, delete it
    if sandbox[x][y].unwrap().extra_data2 > 30 || sandbox[x][y].unwrap().tempature < 40 {
        sandbox[x][y] = None;
    }

    // Replace adjacent flammable particles with fire
    if y != SIMULATION_HEIGHT - 1 && sandbox.rng.gen_bool(0.5) {
        if let Some(particle) = &sandbox[x][y + 1] {
            if particle.is_flammable() {
                sandbox[x][y] = None;
                if sandbox.rng.gen_bool(0.70) {
                    sandbox[x][y + 1] = Some(Particle::new(ParticleType::Fire));
                }
            }
        }
    }
    if x != SIMULATION_WIDTH - 1 {
        if let Some(particle) = &sandbox[x + 1][y] {
            if particle.is_flammable() {
                sandbox[x][y] = None;
                if sandbox.rng.gen_bool(0.70) {
                    sandbox[x + 1][y] = Some(Particle::new(ParticleType::Fire));
                }
            }
        }
    }
    if y != 0 && sandbox.rng.gen_bool(0.5) {
        if let Some(particle) = &sandbox[x][y - 1] {
            if particle.is_flammable() {
                sandbox[x][y] = None;
                if sandbox.rng.gen_bool(0.70) {
                    sandbox[x][y - 1] = Some(Particle::new(ParticleType::Fire));
                }
            }
        }
    }
    if x != 0 {
        if let Some(particle) = &sandbox[x - 1][y] {
            if particle.is_flammable() {
                sandbox[x][y] = None;
                if sandbox.rng.gen_bool(0.70) {
                    sandbox[x - 1][y] = Some(Particle::new(ParticleType::Fire));
                }
            }
        }
    }
}

pub fn update_mirror(sandbox: &mut Sandbox, x: usize, y: usize) {
    // Update the frame counter
    let extra_data1 = &mut sandbox[x][y].as_mut().unwrap().extra_data1;
    *extra_data1 += 1;
    if *extra_data1 > 120 {
        *extra_data1 = 0;
    }

    // If a non-Mirror particle is above this particle, teleport it down to the last empty cell before another non-Mirror particle
    if y != 0 {
        if sandbox[x][y - 1].is_some() {
            if sandbox[x][y - 1].unwrap().ptype != ParticleType::Mirror {
                let mut new_y = y + 1;
                while new_y != SIMULATION_HEIGHT {
                    if sandbox[x][new_y].is_none() {
                        sandbox[x][new_y] = sandbox[x][y - 1].take();
                        return;
                    } else {
                        if sandbox[x][new_y].unwrap().ptype != ParticleType::Mirror {
                            return;
                        }
                    }
                    new_y += 1;
                }
            }
        }
    }
}
