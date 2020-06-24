use crate::sandbox::{Particle, SIMULATION_HEIGHT, SIMULATION_WIDTH};
use std::mem::ManuallyDrop;

pub fn create_cells_array(
    initial_value: Option<Particle>,
) -> Box<[[Option<Particle>; SIMULATION_HEIGHT]; SIMULATION_WIDTH]> {
    // Create a Vec of the same capacity as the resulting Box<Array>
    // We will never drop() this vec, as that would free the memory
    let mut array = ManuallyDrop::new(vec![initial_value; SIMULATION_HEIGHT * SIMULATION_WIDTH]);
    // Convert the Vec to a Box<Array>
    // The box now owns the memory and is in charge of freeing it
    let ptr = array.as_mut_ptr();
    unsafe { Box::from_raw(ptr as *mut [[Option<Particle>; SIMULATION_HEIGHT]; SIMULATION_WIDTH]) }
}

pub fn create_background_array(
    initial_value: u8,
) -> Box<[u8; SIMULATION_HEIGHT * SIMULATION_WIDTH * 4]> {
    // Create a Vec of the same capacity as the resulting Box<Array>
    // We will never drop() this vec, as that would free the memory
    let mut array = ManuallyDrop::new(vec![
        initial_value;
        SIMULATION_HEIGHT * SIMULATION_WIDTH * 4
    ]);
    // Convert the Vec to a Box<Array>
    // The box now owns the memory and is in charge of freeing it
    let ptr = array.as_mut_ptr();
    unsafe { Box::from_raw(ptr as *mut [u8; SIMULATION_HEIGHT * SIMULATION_WIDTH * 4]) }
}
