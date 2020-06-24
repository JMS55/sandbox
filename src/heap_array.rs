use crate::sandbox::{Particle, SIMULATION_HEIGHT, SIMULATION_WIDTH};
use std::mem::ManuallyDrop;

// These functions create a heap allocated array like Box::new([T; N]), but unlike Box::new() the no data touches the stack
// 1. Allocate a Vec of data. The Vec's buffer has the same memory representation as an equivalent array.
// 2. Use unsafe magic to reinterpret the Vec's buffer as an array, and create a Box pointing to it. The Box now owns the memory.
// *  Note: The Vec has to be wrapped with ManuallyDrop, else at the end of the function the Vec will be dropped, and the memory will cleaned up while the Box still points to it.

pub fn create_cells_array(
    initial_value: Option<Particle>,
) -> Box<[[Option<Particle>; SIMULATION_HEIGHT]; SIMULATION_WIDTH]> {
    let mut array = ManuallyDrop::new(vec![initial_value; SIMULATION_HEIGHT * SIMULATION_WIDTH]);
    unsafe {
        Box::from_raw(
            array.as_mut_ptr() as *mut [[Option<Particle>; SIMULATION_HEIGHT]; SIMULATION_WIDTH]
        )
    }
}

pub fn create_background_array(
    initial_value: u8,
) -> Box<[u8; SIMULATION_HEIGHT * SIMULATION_WIDTH * 4]> {
    let mut array = ManuallyDrop::new(vec![
        initial_value;
        SIMULATION_HEIGHT * SIMULATION_WIDTH * 4
    ]);
    unsafe {
        Box::from_raw(array.as_mut_ptr() as *mut [u8; SIMULATION_HEIGHT * SIMULATION_WIDTH * 4])
    }
}
