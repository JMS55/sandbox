use crate::cell_grid::Chunk;
use crate::particle::{Particle, ParticleType};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

pub fn move_solid(chunk: &mut Chunk, x: usize, y: usize) -> (usize, usize) {
    // Move 1 down if able
    if chunk.is_empty(x, y + 1) {
        chunk.swap_cells(x, y, x, y + 1);
        return (x, y + 1);
    }
    (x, y)
}

pub fn move_powder(chunk: &mut Chunk, x: usize, y: usize) -> (usize, usize) {
    // Move 1 down if able
    if chunk.is_empty(x, y + 1) {
        chunk.swap_cells(x, y, x, y + 1);
        return (x, y + 1);
    }
    // Else move 1 down and left if able
    if chunk.is_empty(x - 1, y + 1) && chunk.is_empty(x - 1, y) {
        chunk.swap_cells(x, y, x - 1, y + 1);
        return (x - 1, y + 1);
    }
    // Else move 1 down and right if able
    if chunk.is_empty(x + 1, y + 1) && chunk.is_empty(x + 1, y) {
        chunk.swap_cells(x, y, x + 1, y + 1);
        return (x + 1, y + 1);
    }
    (x, y)
}

pub fn move_liquid(chunk: &mut Chunk, x: usize, y: usize) -> (usize, usize) {
    // Move 1 down if able
    if chunk.is_empty(x, y + 1) {
        chunk.swap_cells(x, y, x, y + 1);
        return (x, y + 1);
    }
    // Else move 1 down and left if able
    if chunk.is_empty(x - 1, y + 1) && chunk.is_empty(x - 1, y) {
        chunk.swap_cells(x, y, x - 1, y + 1);
        return (x - 1, y + 1);
    }
    // Else move 1 down and right if able
    if chunk.is_empty(x + 1, y + 1) && chunk.is_empty(x + 1, y) {
        chunk.swap_cells(x, y, x + 1, y + 1);
        return (x + 1, y + 1);
    }
    // Else move left if able
    if chunk.is_empty(x - 1, y) {
        chunk.swap_cells(x, y, x - 1, y);
        return (x - 1, y);
    }
    // Else move right if able
    if chunk.is_empty(x + 1, y) {
        chunk.swap_cells(x, y, x + 1, y);
        return (x + 1, y);
    }
    (x, y)
}

pub fn move_gas(chunk: &mut Chunk, x: usize, y: usize) -> (usize, usize) {
    if thread_rng().gen_bool(0.5) {
        // Move 1 up if able
        if chunk.is_empty(x, y - 1) {
            chunk.swap_cells(x, y, x, y - 1);
            return (x, y - 1);
        }
        // Else move 1 up and left if able
        if chunk.is_empty(x - 1, y - 1) && chunk.is_empty(x - 1, y) {
            chunk.swap_cells(x, y, x - 1, y - 1);
            return (x - 1, y - 1);
        }
        // Else move 1 up and right if able
        if chunk.is_empty(x + 1, y - 1) && chunk.is_empty(x + 1, y) {
            chunk.swap_cells(x, y, x + 1, y - 1);
            return (x + 1, y - 1);
        }
    } else {
        // Else move left if able
        if chunk.is_empty(x - 1, y) {
            chunk.swap_cells(x, y, x - 1, y);
            return (x - 1, y);
        }
        // Else move right if able
        if chunk.is_empty(x + 1, y) {
            chunk.swap_cells(x, y, x + 1, y);
            return (x + 1, y);
        }
    }
    (x, y)
}

pub fn move_electricity(chunk: &mut Chunk, x: usize, y: usize) -> (usize, usize) {
    // Try switching with an adjacent water particle in the last direction moved
    if chunk.get_particle(x, y).extra_data2 != 0 {
        chunk.get_mut_particle(x, y).extra_data2 -= 1;
        let offset = match chunk.get_particle(x, y).extra_data1 {
            0 => (1, 0),
            1 => (-1, 0),
            2 => (0, 1),
            3 => (0, -1),
            _ => unreachable!(),
        };
        let (x2, y2) = (x as isize + offset.0, y as isize + offset.1);
        if (0..60).contains(&x2) && (0..60).contains(&y2) {
            let x2 = x2 as usize;
            let y2 = y2 as usize;
            if chunk.is_not_empty(x2, y2) {
                if chunk.get_particle(x2, y2).ptype == ParticleType::Water {
                    chunk.swap_cells(x, y, x2, y2);
                    return (x2, y2);
                }
            }
        }
        return (x, y);
    }

    // Else try switching with an adjacent water particle in a random direction
    let rng = &mut thread_rng();
    let mut offsets = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    offsets.shuffle(rng);
    for offset in &offsets {
        let (x2, y2) = (x as isize + offset.0, y as isize + offset.1);
        if (0..60).contains(&x2) && (0..60).contains(&y2) {
            let x2 = x2 as usize;
            let y2 = y2 as usize;
            if chunk.is_not_empty(x2, y2) {
                if chunk.get_particle(x2, y2).ptype == ParticleType::Water {
                    let particle = chunk.get_mut_particle(x, y);
                    particle.extra_data1 = match offset {
                        (1, 0) => 0,
                        (-1, 0) => 1,
                        (0, 1) => 2,
                        (0, -1) => 3,
                        _ => unreachable!(),
                    };
                    particle.extra_data2 = 100;
                    chunk.swap_cells(x, y, x2, y2);
                    return (x2, y2);
                }
            }
        }
    }

    // Else move 1 down if able
    if chunk.is_empty(x, y + 1) {
        chunk.swap_cells(x, y, x, y + 1);
        return (x, y + 1);
    }
    // Else mark for deletion if not above a Replicator
    if chunk.is_not_empty(x, y + 1) {
        if chunk.get_particle(x, y + 1).ptype != ParticleType::Replicator {
            chunk.get_mut_particle(x, y).extra_data2 -= 1;
        }
    }
    // Else mark for deletion if in the last row
    if !chunk.is_empty(x, y + 1) && !chunk.is_not_empty(x, y + 1) {
        chunk.get_mut_particle(x, y).extra_data2 = -1;
    }

    (x, y)
}

pub fn move_life(chunk: &mut Chunk, x: usize, y: usize) -> (usize, usize) {
    // Move 1 down if able
    if chunk.is_empty(x, y + 1) {
        // and increase the falling counter by 1
        chunk.get_mut_particle(x, y).extra_data1 =
            chunk.get_mut_particle(x, y).extra_data1.saturating_add(1);
        chunk.swap_cells(x, y, x, y + 1);
        return (x, y + 1);
    }

    // Kill the particle if the falling counter > 60, else reset it
    if chunk.get_particle(x, y).extra_data1 > 60 {
        chunk.get_mut_particle(x, y).extra_data2 = 1;
    } else {
        chunk.get_mut_particle(x, y).extra_data1 = 0;
    }

    // And if still living
    if chunk.get_particle(x, y).extra_data2 == 0 {
        let drop_is_short_enough = |x: usize, y: usize| -> bool {
            let mut y2 = y + 1;
            let mut drop_size = 0;
            while y2 < 60 {
                if chunk.is_empty(x, y2) {
                    drop_size += 1;
                } else {
                    break;
                }
                y2 += 1;
            }
            drop_size < 31
        };

        // Move left if able and the drop is short enough
        if chunk.is_empty(x - 1, y) && drop_is_short_enough(x - 1, y) {
            chunk.swap_cells(x, y, x - 1, y);
            return (x - 1, y);
        }
        // Else move right if able and the drop is short enough
        if chunk.is_empty(x + 1, y) && drop_is_short_enough(x + 1, y) {
            chunk.swap_cells(x, y, x + 1, y);
            return (x + 1, y);
        }
    }

    (x, y)
}

pub fn move_super_life(chunk: &mut Chunk, mut x: usize, mut y: usize) -> (usize, usize) {
    // Switch with an adjacent Life particle
    let mut swapped = false;
    if !swapped && chunk.is_not_empty(x, y + 1) {
        if chunk.get_particle(x, y + 1).ptype == ParticleType::Life {
            chunk.swap_cells(x, y, x, y + 1);
            y += 1;
            swapped = true;
        }
    }
    if !swapped && chunk.is_not_empty(x + 1, y) {
        if chunk.get_particle(x + 1, y).ptype == ParticleType::Life {
            chunk.swap_cells(x, y, x + 1, y);
            x += 1;
            swapped = true;
        }
    }
    if !swapped && chunk.is_not_empty(x, y - 1) {
        if chunk.get_particle(x, y - 1).ptype == ParticleType::Life {
            chunk.swap_cells(x, y, x, y - 1);
            y -= 1;
            swapped = true;
        }
    }
    if !swapped && chunk.is_not_empty(x - 1, y) {
        if chunk.get_particle(x - 1, y).ptype == ParticleType::Life {
            chunk.swap_cells(x, y, x - 1, y);
            x -= 1;
        }
    }

    // Then move like normal Life twice
    for _ in 0..2 {
        let (x2, y2) = move_life(chunk, x, y);
        x = x2;
        y = y2;
    }

    (x, y)
}

pub fn move_fire(chunk: &mut Chunk, x: usize, y: usize) -> (usize, usize) {
    let new_position = move_gas(chunk, x, y);
    let extra_data2 = &mut chunk
        .get_mut_particle(new_position.0, new_position.1)
        .extra_data2;
    if new_position == (x, y) {
        *extra_data2 += 1;
    } else {
        *extra_data2 = extra_data2.saturating_sub(1);
    }
    new_position
}

pub fn update_sand(chunk: &mut Chunk, x: usize, y: usize) {
    // When wet and temperature >= 30, dry out
    if chunk.get_particle(x, y).extra_data1 == 1 && chunk.get_particle(x, y).temperature >= 30 {
        chunk.get_mut_particle(x, y).extra_data1 = 0;
    }

    // When temperature >= 120, turn into Glass
    if chunk.get_particle(x, y).temperature >= 120 {
        chunk.get_mut_particle(x, y).ptype = ParticleType::Glass;
    }
}

pub fn update_water(chunk: &mut Chunk, x: usize, y: usize) {
    if chunk.get_particle(x, y).temperature >= 100 {
        let t = (chunk.get_particle(x, y).temperature as f64 / 150.0)
            .min(1.0)
            .max(0.0);
        let chance = (1.0 - t) * 0.3 + t * 0.7;
        if thread_rng().gen_bool(chance) {
            chunk.get_mut_particle(x, y).ptype = ParticleType::Steam;
            return;
        }
    }

    // Find the first dry Sand below, delete this particle, and turn the Sand wet.
    let mut y2 = y + 1;
    while y2 < 60 {
        let particle = if chunk.is_not_empty(x, y2) {
            chunk.get_cell(x, y2)
        } else {
            &None
        };
        match particle {
            Some(particle) if particle.ptype == ParticleType::Sand => {
                if particle.extra_data1 == 0 {
                    *chunk.get_mut_cell(x, y) = None;
                    chunk.get_mut_particle(x, y2).extra_data1 = 1;
                    return;
                }
            }
            _ => return,
        }
        y2 += 1;
    }
}

// pub fn update_acid(chunk: &mut Chunk, x: usize, y: usize) {
//     if y != SANDBOX_HEIGHT - 1 {
//         if let Some(particle) = &chunk[x][y + 1] {
//             if particle.dissolved_by_acid() {
//                 // Heat nearby particles when dissolving water
//                 if particle.ptype == ParticleType::Water {
//                     for x_offset in -3..=3 {
//                         for y_offset in -3..=3 {
//                             let x = x as isize + x_offset;
//                             let y = y as isize + y_offset;
//                             if !(x_offset == 0 && y_offset == 0)
//                                 && (0..(SANDBOX_WIDTH as isize)).contains(&x)
//                                 && (0..(SANDBOX_HEIGHT as isize)).contains(&y)
//                             {
//                                 if let Some(particle) = chunk[x as usize][y as usize].as_mut() {
//                                     particle.temperature += 2;
//                                 }
//                             }
//                         }
//                     }
//                 }
//                 // Delete both this particle and the adjacent one
//                 chunk[x][y] = None;
//                 chunk[x][y + 1] = None;
//                 return;
//             }
//         }
//     }
//     if x != SANDBOX_WIDTH - 1 {
//         if let Some(particle) = &chunk[x + 1][y] {
//             if particle.dissolved_by_acid() {
//                 // Heat nearby particles when dissolving water
//                 if particle.ptype == ParticleType::Water {
//                     for x_offset in -3..=3 {
//                         for y_offset in -3..=3 {
//                             let x = x as isize + x_offset;
//                             let y = y as isize + y_offset;
//                             if !(x_offset == 0 && y_offset == 0)
//                                 && (0..(SANDBOX_WIDTH as isize)).contains(&x)
//                                 && (0..(SANDBOX_HEIGHT as isize)).contains(&y)
//                             {
//                                 if let Some(particle) = chunk[x as usize][y as usize].as_mut() {
//                                     particle.temperature += 2;
//                                 }
//                             }
//                         }
//                     }
//                 }
//                 // Delete both this particle and the adjacent one
//                 chunk[x][y] = None;
//                 chunk[x + 1][y] = None;
//                 return;
//             }
//         }
//     }
//     if y != 0 {
//         if let Some(particle) = &chunk[x][y - 1] {
//             if particle.dissolved_by_acid() {
//                 // Heat nearby particles when dissolving water
//                 if particle.ptype == ParticleType::Water {
//                     for x_offset in -3..=3 {
//                         for y_offset in -3..=3 {
//                             let x = x as isize + x_offset;
//                             let y = y as isize + y_offset;
//                             if !(x_offset == 0 && y_offset == 0)
//                                 && (0..(SANDBOX_WIDTH as isize)).contains(&x)
//                                 && (0..(SANDBOX_HEIGHT as isize)).contains(&y)
//                             {
//                                 if let Some(particle) = chunk[x as usize][y as usize].as_mut() {
//                                     particle.temperature += 2;
//                                 }
//                             }
//                         }
//                     }
//                 }
//                 // Delete both this particle and the adjacent one
//                 chunk[x][y] = None;
//                 chunk[x][y - 1] = None;
//                 return;
//             }
//         }
//     }
//     if x != 0 {
//         if let Some(particle) = &chunk[x - 1][y] {
//             if particle.dissolved_by_acid() {
//                 // Heat nearby particles when dissolving water
//                 if particle.ptype == ParticleType::Water {
//                     for x_offset in -3..=3 {
//                         for y_offset in -3..=3 {
//                             let x = x as isize + x_offset;
//                             let y = y as isize + y_offset;
//                             if !(x_offset == 0 && y_offset == 0)
//                                 && (0..(SANDBOX_WIDTH as isize)).contains(&x)
//                                 && (0..(SANDBOX_HEIGHT as isize)).contains(&y)
//                             {
//                                 if let Some(particle) = chunk[x as usize][y as usize].as_mut() {
//                                     particle.temperature += 2;
//                                 }
//                             }
//                         }
//                     }
//                 }
//                 // Delete both this particle and the adjacent one
//                 chunk[x][y] = None;
//                 chunk[x - 1][y] = None;
//                 return;
//             }
//         }
//     }
// }

// pub fn update_replicator(chunk: &mut Chunk, x: usize, y: usize) {
//     chunk.get_mut_particle(x, y).extra_data1 = 0;
//     if y < SANDBOX_HEIGHT - 2 {
//         if let Some(particle) = chunk[x][y + 1] {
//             if particle.ptype != ParticleType::Replicator {
//                 chunk.get_mut_particle(x, y).extra_data1 = 1;
//                 if chunk[x][y + 2].is_none() {
//                     let mut particle = particle.clone();
//                     particle.color_offset = chunk.rng.gen_range(-10..11);
//                     chunk[x][y + 2] = Some(particle);
//                 }
//             }
//         }
//     }
//     if x < SANDBOX_WIDTH - 2 {
//         if let Some(particle) = chunk[x + 1][y] {
//             if particle.ptype != ParticleType::Replicator {
//                 chunk.get_mut_particle(x, y).extra_data1 = 1;
//                 if chunk[x + 2][y].is_none() {
//                     let mut particle = particle.clone();
//                     particle.color_offset = chunk.rng.gen_range(-10..11);
//                     chunk[x + 2][y] = Some(particle);
//                 }
//             }
//         }
//     }
//     if y > 1 {
//         if let Some(particle) = chunk[x][y - 1] {
//             if particle.ptype != ParticleType::Replicator {
//                 chunk.get_mut_particle(x, y).extra_data1 = 1;
//                 if chunk[x][y - 2].is_none() {
//                     let mut particle = particle.clone();
//                     particle.color_offset = chunk.rng.gen_range(-10..11);
//                     chunk[x][y - 2] = Some(particle);
//                 }
//             }
//         }
//     }
//     if x > 1 {
//         if let Some(particle) = chunk[x - 1][y] {
//             if particle.ptype != ParticleType::Replicator {
//                 chunk.get_mut_particle(x, y).extra_data1 = 1;
//                 if chunk[x - 2][y].is_none() {
//                     let mut particle = particle.clone();
//                     particle.color_offset = chunk.rng.gen_range(-10..11);
//                     chunk[x - 2][y] = Some(particle);
//                 }
//             }
//         }
//     }
// }

// pub fn update_plant(chunk: &mut Chunk, x: usize, y: usize) {
//     // If temperature > 100, turn into Fire
//     if chunk.get_particle(x, y).temperature > 100 {
//         chunk.get_mut_particle(x, y).ptype = ParticleType::Fire;
//         return;
//     }

//     // If above wet Sand or another Plant that's growable, mark as growable (extra_data2 = 1)
//     if y != SANDBOX_HEIGHT - 1 {
//         if let Some(particle) = chunk[x][y + 1] {
//             if particle.ptype == ParticleType::Sand && particle.extra_data1 == 1 {
//                 chunk.get_mut_particle(x, y).extra_data2 = 1;
//             }
//             if particle.ptype == ParticleType::Plant && particle.extra_data2 == 1 {
//                 chunk.get_mut_particle(x, y).extra_data2 = 1;
//             }
//         }
//     }

//     // If growable and growing_time_left (extra_data_2) > 0, create another Plant nearby with 1 less growing_time_left
//     if y != 0 {
//         if chunk.get_particle(x, y).extra_data2 == 1 {
//             let extra_data1 = chunk.get_particle(x, y).extra_data1;
//             if extra_data1 > 0 {
//                 let x_offset = chunk.rng.gen_range(-1..2);
//                 let y_offset = chunk.rng.gen_range(-2..3);
//                 let x = x as isize + x_offset;
//                 let y = y as isize + y_offset;
//                 if (0..(SANDBOX_WIDTH as isize)).contains(&x)
//                     && (0..(SANDBOX_HEIGHT as isize)).contains(&y)
//                 {
//                     if chunk[x as usize][y as usize].is_none() {
//                         let mut particle = Particle::new(ParticleType::Plant, &mut chunk.rng);
//                         particle.extra_data1 = extra_data1 - 1;
//                         particle.extra_data2 = 1;
//                         chunk[x as usize][y as usize] = Some(particle);
//                     }
//                 }
//             }
//         }
//     }
// }

// // Check if temperature >= 0, and if so, wait 1/3rd of a second, and then delete itself and freeze around it
// pub fn update_cryotheum(chunk: &mut Chunk, x: usize, y: usize) {
//     if chunk.get_particle(x, y).temperature >= 0 && chunk.get_particle(x, y).extra_data1 == 0 {
//         chunk.get_mut_particle(x, y).extra_data1 = 21;
//         return;
//     }

//     if chunk.get_particle(x, y).extra_data1 > 1 {
//         chunk.get_mut_particle(x, y).extra_data1 -= 1;
//     }

//     if chunk.get_particle(x, y).extra_data1 == 1 {
//         chunk[x][y] = None;

//         for x_offset in (-15..=15).skip(0) {
//             for y_offset in (-15..=15).skip(0) {
//                 if x_offset * x_offset + y_offset * y_offset > 15 * 15 {
//                     continue;
//                 }
//                 let x = x as isize + x_offset;
//                 let y = y as isize + y_offset;
//                 if (0..(SANDBOX_WIDTH as isize)).contains(&x)
//                     && (0..(SANDBOX_HEIGHT as isize)).contains(&y)
//                 {
//                     if let Some(particle) = chunk[x as usize][y as usize].as_mut() {
//                         if particle.affected_by_cryotheum_coldsnap() {
//                             particle.temperature -= 100;
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

// pub fn update_unstable(chunk: &mut Chunk, x: usize, y: usize) {
//     // Increase temperature by 10 every half a second
//     let mut particle = chunk[x][y].as_mut().unwrap();
//     if particle.extra_data1 == 30 {
//         particle.extra_data1 = 0;
//         particle.temperature += 10;
//     } else {
//         particle.extra_data1 += 1;
//     }

//     // When temperature >= 200 (10 seconds of existing), replace the surrounding area with Smoke
//     if particle.temperature >= 200 {
//         for x_offset in -30..=30 {
//             for y_offset in -30..=30 {
//                 let x = x as isize + x_offset;
//                 let y = y as isize + y_offset;
//                 if (0..(SANDBOX_WIDTH as isize)).contains(&x)
//                     && (0..(SANDBOX_HEIGHT as isize)).contains(&y)
//                 {
//                     if let Some(particle) = chunk[x as usize][y as usize] {
//                         if particle.can_be_vaporized_by_unstable() {
//                             chunk[x as usize][y as usize] =
//                                 Some(Particle::new(ParticleType::Smoke, &mut chunk.rng));
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

// pub fn update_electricity(chunk: &mut Chunk, x: usize, y: usize) {
//     // If this particle was unable able to move, delete it
//     if chunk.get_particle(x, y).extra_data2 == -1 {
//         chunk[x][y] = None;
//     }
// }

// pub fn update_life(chunk: &mut Chunk, x: usize, y: usize) {
//     // When temperature less than -50, or greater than 50, this particle dies
//     let mut particle = chunk[x][y].as_mut().unwrap();
//     if particle.temperature < -50 || particle.temperature > 50 {
//         particle.extra_data2 = 1;
//     }

//     // When alive and touching a Plant particle, chance to turn it into a new Life particle
//     // When alive and touching a Blood particle, turn into SuperLife
//     if particle.extra_data2 == 0 {
//         if y != SANDBOX_HEIGHT - 1 {
//             if let Some(particle) = chunk[x][y + 1] {
//                 if particle.ptype == ParticleType::Plant && chunk.rng.gen_bool(0.2) {
//                     chunk[x][y + 1] = Some(Particle::new(ParticleType::Life, &mut chunk.rng));
//                     return;
//                 }
//                 if particle.ptype == ParticleType::Blood
//                     && chunk.get_particle(x, y).ptype != ParticleType::SuperLife
//                 {
//                     chunk[x][y + 1] = None;
//                     chunk.get_mut_particle(x, y).ptype = ParticleType::SuperLife;
//                     return;
//                 }
//             }
//         }
//         if x != SANDBOX_WIDTH - 1 {
//             if let Some(particle) = chunk[x + 1][y] {
//                 if particle.ptype == ParticleType::Plant && chunk.rng.gen_bool(0.2) {
//                     chunk[x + 1][y] = Some(Particle::new(ParticleType::Life, &mut chunk.rng));
//                     return;
//                 }
//                 if particle.ptype == ParticleType::Blood
//                     && chunk.get_particle(x, y).ptype != ParticleType::SuperLife
//                 {
//                     chunk[x + 1][y] = None;
//                     chunk.get_mut_particle(x, y).ptype = ParticleType::SuperLife;
//                     return;
//                 }
//             }
//         }
//         if y != 0 {
//             if let Some(particle) = chunk[x][y - 1] {
//                 if particle.ptype == ParticleType::Plant && chunk.rng.gen_bool(0.2) {
//                     chunk[x][y - 1] = Some(Particle::new(ParticleType::Life, &mut chunk.rng));
//                     return;
//                 }
//                 if particle.ptype == ParticleType::Blood
//                     && chunk.get_particle(x, y).ptype != ParticleType::SuperLife
//                 {
//                     chunk[x][y - 1] = None;
//                     chunk.get_mut_particle(x, y).ptype = ParticleType::SuperLife;
//                     return;
//                 }
//             }
//         }
//         if x != 0 {
//             if let Some(particle) = chunk[x - 1][y] {
//                 if particle.ptype == ParticleType::Plant && chunk.rng.gen_bool(0.2) {
//                     chunk[x - 1][y] = Some(Particle::new(ParticleType::Life, &mut chunk.rng));
//                     return;
//                 }
//                 if particle.ptype == ParticleType::Blood
//                     && chunk.get_particle(x, y).ptype != ParticleType::SuperLife
//                 {
//                     chunk[x - 1][y] = None;
//                     chunk.get_mut_particle(x, y).ptype = ParticleType::SuperLife;
//                     return;
//                 }
//             }
//         }
//     } else {
//         // Else if dead, and if enough particles stacked above, chance to turn into blood
//         let mut count = 1;
//         while count <= y {
//             match chunk[x][y - count] {
//                 Some(_) => count += 1,
//                 None => break,
//             }
//         }
//         count -= 1;
//         if count > 30 && chunk.rng.gen_bool(0.1) {
//             chunk[x][y] = Some(Particle::new(ParticleType::Blood, &mut chunk.rng));
//         }
//     }
// }

// pub fn update_blood(chunk: &mut Chunk, x: usize, y: usize) {
//     // Evaporate above a certain temperature
//     if chunk.get_particle(x, y).temperature >= 137 {
//         chunk[x][y] = None;
//     }
// }

// pub fn update_smoke(chunk: &mut Chunk, x: usize, y: usize) {
//     let mut particle = chunk[x][y].as_mut().unwrap();
//     if particle.extra_data1 == 0 {
//         particle.extra_data2 -= 1;
//         if particle.extra_data2 == 0 {
//             chunk[x][y] = None;
//         }
//     } else {
//         particle.extra_data1 -= 1;
//     }
// }

// pub fn update_fire(chunk: &mut Chunk, x: usize, y: usize) {
//     // When this particle hasn't moved for more than 1/2 a second or temperature < 40, delete it
//     if chunk.get_particle(x, y).extra_data2 > 30 || chunk.get_particle(x, y).temperature < 40 {
//         chunk[x][y] = None;
//     }

//     // Replace adjacent flammable particles with fire
//     if y != SANDBOX_HEIGHT - 1 {
//         if let Some(particle) = &chunk[x][y + 1] {
//             if particle.is_flammable() && chunk.rng.gen_bool(0.35) {
//                 chunk[x][y] = None;
//                 chunk[x][y + 1] = Some(Particle::new(ParticleType::Fire, &mut chunk.rng));
//             }
//         }
//     }
//     if x != SANDBOX_WIDTH - 1 {
//         if let Some(particle) = &chunk[x + 1][y] {
//             if particle.is_flammable() && chunk.rng.gen_bool(0.35) {
//                 chunk[x][y] = None;
//                 chunk[x + 1][y] = Some(Particle::new(ParticleType::Fire, &mut chunk.rng));
//             }
//         }
//     }
//     if y != 0 {
//         if let Some(particle) = &chunk[x][y - 1] {
//             if particle.is_flammable() && chunk.rng.gen_bool(0.35) {
//                 chunk[x][y] = None;
//                 chunk[x][y - 1] = Some(Particle::new(ParticleType::Fire, &mut chunk.rng));
//             }
//         }
//     }
//     if x != 0 {
//         if let Some(particle) = &chunk[x - 1][y] {
//             if particle.is_flammable() && chunk.rng.gen_bool(0.35) {
//                 chunk[x][y] = None;
//                 chunk[x - 1][y] = Some(Particle::new(ParticleType::Fire, &mut chunk.rng));
//             }
//         }
//     }
// }

// pub fn update_mirror(chunk: &mut Chunk, x: usize, y: usize) {
//     // Update the frame counter
//     let extra_data1 = &mut chunk.get_mut_particle(x, y).extra_data1;
//     *extra_data1 += 1;
//     if *extra_data1 > 120 {
//         *extra_data1 = 0;
//     }

//     // If a non-Mirror particle is above this particle, teleport it down to the last empty cell before another non-Mirror particle
//     if y != 0 {
//         if chunk[x][y - 1].is_some() {
//             if chunk[x][y - 1].unwrap().ptype != ParticleType::Mirror {
//                 let mut new_y = y + 1;
//                 while new_y != SANDBOX_HEIGHT {
//                     if chunk[x][new_y].is_none() {
//                         chunk[x][new_y] = chunk[x][y - 1].take();
//                         return;
//                     } else if chunk[x][new_y].unwrap().ptype != ParticleType::Mirror {
//                         return;
//                     }
//                     new_y += 1;
//                 }
//             }
//         }
//     }
// }

// pub fn update_steam(chunk: &mut Chunk, x: usize, y: usize) {
//     if chunk.get_particle(x, y).temperature < 100 {
//         chunk.get_mut_particle(x, y).ptype = ParticleType::Water;
//     }
// }

// pub fn update_glitch(chunk: &mut Chunk, x: usize, y: usize) {
//     // Convert adjacent other particle types to a random type
//     if y != SANDBOX_HEIGHT - 1 {
//         if chunk[x][y + 1].is_some() {
//             if chunk[x][y + 1].unwrap().ptype != ParticleType::Glitch
//                 && chunk[x][y + 1].unwrap().ptype != ParticleType::Replicator
//             {
//                 chunk[x][y] = None;
//                 chunk[x][y + 1] = Some(Particle::new(chunk.rng.gen(), &mut chunk.rng));
//             }
//         }
//     }
//     if x != SANDBOX_WIDTH - 1 {
//         if chunk[x + 1][y].is_some() {
//             if chunk[x + 1][y].unwrap().ptype != ParticleType::Glitch
//                 && chunk[x + 1][y].unwrap().ptype != ParticleType::Replicator
//             {
//                 chunk[x][y] = None;
//                 chunk[x + 1][y] = Some(Particle::new(chunk.rng.gen(), &mut chunk.rng));
//             }
//         }
//     }
//     if y != 0 {
//         if chunk[x][y - 1].is_some() {
//             if chunk[x][y - 1].unwrap().ptype != ParticleType::Glitch
//                 && chunk[x][y - 1].unwrap().ptype != ParticleType::Replicator
//             {
//                 chunk[x][y] = None;
//                 chunk[x][y - 1] = Some(Particle::new(chunk.rng.gen(), &mut chunk.rng));
//             }
//         }
//     }
//     if x != 0 {
//         if chunk[x - 1][y].is_some() {
//             if chunk[x - 1][y].unwrap().ptype != ParticleType::Glitch
//                 && chunk[x - 1][y].unwrap().ptype != ParticleType::Replicator
//             {
//                 chunk[x][y] = None;
//                 chunk[x - 1][y] = Some(Particle::new(chunk.rng.gen(), &mut chunk.rng));
//             }
//         }
//     }
// }
