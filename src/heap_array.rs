use crate::sandbox::{Particle, SIMULATION_HEIGHT, SIMULATION_WIDTH};
use std::mem::ManuallyDrop;

pub fn create_cells_array(
    initial_value: Option<Particle>,
) -> Box<[[Option<Particle>; SIMULATION_HEIGHT]; SIMULATION_WIDTH]> {
    // Create a Vec of the same capacity as the resulting Box<Array>
    // We will never drop() this vec, as that would free the memory
    let mut array: ManuallyDrop<Vec<Option<Particle>>> =
        ManuallyDrop::new(Vec::with_capacity(SIMULATION_HEIGHT * SIMULATION_WIDTH));

    // Fill the memory with the initial data
    let ptr = array.as_mut_ptr();
    for i in 0..(SIMULATION_HEIGHT * SIMULATION_WIDTH) {
        unsafe { std::ptr::write::<Option<Particle>>(ptr.offset(i as isize), initial_value) };
    }

    // Convert the Vec to a Box<Array>
    // The box now owns the memory and is in charge of freeing it
    unsafe { Box::from_raw(ptr as *mut [[Option<Particle>; SIMULATION_HEIGHT]; SIMULATION_WIDTH]) }
}

pub fn create_background_array(
    initial_value: u8,
) -> Box<[u8; SIMULATION_HEIGHT * SIMULATION_WIDTH * 4]> {
    // Create a Vec of the same capacity as the resulting Box<Array>
    // We will never drop() this vec, as that would free the memory
    let mut array: ManuallyDrop<Vec<u8>> =
        ManuallyDrop::new(Vec::with_capacity(SIMULATION_HEIGHT * SIMULATION_WIDTH * 4));

    // Fill the memory with the initial data
    let ptr = array.as_mut_ptr();
    for i in 0..(SIMULATION_HEIGHT * SIMULATION_WIDTH * 4) {
        unsafe { std::ptr::write::<u8>(ptr.offset(i as isize), initial_value) };
    }

    // Convert the Vec to a Box<Array>
    // The box now owns the memory and is in charge of freeing it
    unsafe { Box::from_raw(ptr as *mut [u8; SIMULATION_HEIGHT * SIMULATION_WIDTH * 4]) }
}
