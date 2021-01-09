use crate::particle::Particle;
use crate::sandbox::{SANDBOX_HEIGHT, SANDBOX_WIDTH};
use rand::seq::SliceRandom;
use rand_pcg::Pcg64;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr;

pub struct CellGrid {
    cells: Box<[[Option<Particle>; SANDBOX_HEIGHT]; SANDBOX_WIDTH]>,
}

/// These functions create a heap allocated array like Box::new([T; N]), but unlike Box::new() no data touches the stack.
///
/// 1. Allocate a Vec of data. The Vec's buffer has the same memory representation as an equivalent array.
/// 2. Use unsafe magic to reinterpret the Vec's buffer as an array, and create a Box pointing to it. The Box now owns the memory.
/// *  Note: The Vec has to be wrapped with ManuallyDrop, else at the end of the function the Vec will be dropped, and the memory will cleaned up while the Box still points to it.
impl<'a> CellGrid {
    pub fn new() -> Self {
        let mut data = ManuallyDrop::new(vec![None::<Particle>; SANDBOX_HEIGHT * SANDBOX_WIDTH]);
        let cells = unsafe {
            Box::from_raw(
                data.as_mut_ptr() as *mut [[Option<Particle>; SANDBOX_HEIGHT]; SANDBOX_WIDTH]
            )
        };
        Self { cells }
    }

    pub fn create_background_array(
        initial_value: u8,
    ) -> Box<[u8; SANDBOX_HEIGHT * SANDBOX_WIDTH * 4]> {
        let mut data = ManuallyDrop::new(vec![initial_value; SANDBOX_HEIGHT * SANDBOX_WIDTH * 4]);
        unsafe { Box::from_raw(data.as_mut_ptr() as *mut [u8; SANDBOX_HEIGHT * SANDBOX_WIDTH * 4]) }
    }

    pub fn update_in_parallel<F>(
        &mut self,
        chunks: &mut Vec<Chunk>,
        rng: &mut Pcg64,
        update_function: F,
    ) where
        F: Fn(&mut Chunk, usize, usize) + Sync + Send,
    {
        chunks.clear();

        let mut offsets = [(false, false), (true, false), (false, true), (true, true)];
        offsets.shuffle(rng);

        for (x_offset_by_1, y_offset_by_1) in &offsets {
            self.chunks_mut(*x_offset_by_1, *y_offset_by_1, chunks);
            chunks.par_iter_mut().for_each(|chunk| {
                for x in 15..45 {
                    for y in 15..45 {
                        (update_function)(chunk, x, y);
                    }
                }
            });
        }
    }

    /// Split the grid into 30x30 chunks, and then return as many chunks as possible that don't touch, plus a 15-wide ring around each chunk.
    /// The arguments determine whether or not to start offset by 1 chunk from the top left.
    fn chunks_mut(&'a mut self, x_offset_by_1: bool, y_offset_by_1: bool, chunks: &mut Vec<Chunk>) {
        let x_start = if x_offset_by_1 { 15 } else { -15 };
        let y_start = if y_offset_by_1 { 15 } else { -15 };
        let x_end = SANDBOX_WIDTH as isize - 45;
        let y_end = SANDBOX_HEIGHT as isize - 45;

        for chunk_start_y in (y_start..=y_end).step_by(60) {
            for chunk_start_x in (x_start..=x_end).step_by(60) {
                const OUT_OF_BOUNDS: ChunkCell<'static> = ChunkCell::OutOfBounds;
                const OUT_OF_BOUNDS_60: [ChunkCell<'static>; 60] = [OUT_OF_BOUNDS; 60];
                let mut chunk = [OUT_OF_BOUNDS_60; 60];

                for y_offset in 0..60 {
                    for x_offset in 0..60 {
                        let x = chunk_start_x + x_offset as isize;
                        let y = chunk_start_y + y_offset as isize;
                        if (0..SANDBOX_WIDTH as isize).contains(&x)
                            && (0..SANDBOX_HEIGHT as isize).contains(&y)
                        {
                            let cell_pointer: *mut Option<Particle> =
                                &mut self.cells[x as usize][y as usize];
                            let cell_pointer = unsafe { cell_pointer.as_mut().unwrap() };
                            chunk[x_offset][y_offset] = ChunkCell::InBounds(cell_pointer);
                        }
                    }
                }

                chunks.push(Chunk { cells: chunk });
            }
        }
    }
}

impl Deref for CellGrid {
    type Target = Box<[[Option<Particle>; SANDBOX_HEIGHT]; SANDBOX_WIDTH]>;

    fn deref(&self) -> &Self::Target {
        &self.cells
    }
}

impl DerefMut for CellGrid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cells
    }
}

pub struct Chunk<'a> {
    cells: [[ChunkCell<'a>; 60]; 60],
}

impl<'a> Chunk<'a> {
    pub fn get_particle(&self, x: usize, y: usize) -> &Particle {
        match &self.cells[x][y] {
            ChunkCell::OutOfBounds => panic!("Not in bounds."),
            ChunkCell::InBounds(particle) => particle.as_ref().unwrap(),
        }
    }

    pub fn get_mut_particle(&mut self, x: usize, y: usize) -> &mut Particle {
        match &mut self.cells[x][y] {
            ChunkCell::OutOfBounds => panic!("Not in bounds."),
            ChunkCell::InBounds(particle) => particle.as_mut().unwrap(),
        }
    }

    pub fn is_empty(&self, x: usize, y: usize) -> bool {
        match &self.cells[x][y] {
            ChunkCell::OutOfBounds => false,
            ChunkCell::InBounds(particle) => particle.is_none(),
        }
    }

    pub fn is_not_empty(&self, x: usize, y: usize) -> bool {
        match &self.cells[x][y] {
            ChunkCell::OutOfBounds => false,
            ChunkCell::InBounds(particle) => particle.is_some(),
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> &Option<Particle> {
        match &self.cells[x][y] {
            ChunkCell::OutOfBounds => panic!("Not in bounds."),
            ChunkCell::InBounds(particle) => particle,
        }
    }

    pub fn get_mut_cell(&mut self, x: usize, y: usize) -> &mut Option<Particle> {
        match &mut self.cells[x][y] {
            ChunkCell::OutOfBounds => panic!("Not in bounds."),
            ChunkCell::InBounds(particle) => particle,
        }
    }

    pub fn swap_cells(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) {
        unsafe { ptr::swap(self.get_mut_cell(x1, y1), self.get_mut_cell(x2, y2)) }
    }
}

pub enum ChunkCell<'a> {
    OutOfBounds,
    InBounds(&'a mut Option<Particle>),
}
