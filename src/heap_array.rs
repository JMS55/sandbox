use crate::particle::Particle;
use crate::sandbox::{SANDBOX_HEIGHT, SANDBOX_WIDTH};
use std::mem::ManuallyDrop;

// These functions create a heap allocated array like Box::new([T; N]), but unlike Box::new() no data touches the stack.

// This file should be replaced when placement new is added to Rust.

// 1. Allocate a Vec of data. The Vec's buffer has the same memory representation as an equivalent array.
// 2. Use unsafe magic to reinterpret the Vec's buffer as an array, and create a Box pointing to it. The Box now owns the memory.
// *  Note: The Vec has to be wrapped with ManuallyDrop, else at the end of the function the Vec will be dropped, and the memory will cleaned up while the Box still points to it.

pub fn create_cells_array(
    initial_value: Option<Particle>,
) -> Box<[[Option<Particle>; SANDBOX_HEIGHT]; SANDBOX_WIDTH]> {
    let mut data = ManuallyDrop::new(vec![initial_value; SANDBOX_HEIGHT * SANDBOX_WIDTH]);
    unsafe {
        Box::from_raw(data.as_mut_ptr() as *mut [[Option<Particle>; SANDBOX_HEIGHT]; SANDBOX_WIDTH])
    }
}

pub fn create_background_array(initial_value: u8) -> Box<[u8; SANDBOX_HEIGHT * SANDBOX_WIDTH * 3]> {
    let mut data = ManuallyDrop::new(vec![initial_value; SANDBOX_HEIGHT * SANDBOX_WIDTH * 3]);
    unsafe { Box::from_raw(data.as_mut_ptr() as *mut [u8; SANDBOX_HEIGHT * SANDBOX_WIDTH * 3]) }
}
