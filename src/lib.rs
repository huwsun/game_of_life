#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate bit_field;
extern crate wasm_bindgen;

use bit_field::*;
use wasm_bindgen::prelude::*;

type Cells = Vec<u8>;

trait BitOper {
    fn get_bit(&self, idx: usize) -> bool;
    fn set_bit(&mut self, idx: usize, val: bool);
}

impl BitOper for Cells {
    fn get_bit(&self, idx: usize) -> bool {
        self.as_slice().get_bit(idx)
    }

    fn set_bit(&mut self, idx: usize, val: bool) {
        self.as_mut_slice().set_bit(idx, val);
    }
}
#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Cells,
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells.get_bit(idx) as u8;
            }
        }
        count
    }

    // ...
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        let width = 64u32;
        let height = 64u32;
        let len = (width * height) as usize;

        let mut cells = vec![0u8; if len % 8 == 0 { len / 8 } else { len / 8 + 1 }];

        for i in 0..len {
            if i % 2 == 0 || i % 7 == 0 {
                cells.set_bit(i, true);
            } else {
                cells.set_bit(i, false);
            }
        }
        //log!("new:{:?}",un.cells);
        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn tick(&mut self) {
        let mut cells = self.cells.clone();
        //log!("tick before:{:?}",self.cells);
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells.get_bit(idx);
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (true, x) if x < 2 => false,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (true, 2) | (true, 3) => true,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (true, x) if x > 3 => false,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (false, 3) => true,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };

                cells.set_bit(idx, next_cell);
            }
        }
        self.cells = cells;
        //log!("tick after:{:?}",self.cells);
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn bytes(&self) -> u32 {
        self.cells.len() as u32
    }

    pub fn cells(&self) -> *const u8 {
        //log!("cells:{:?}",self.cells);
        //log!("ptr:{:?}",self.cells.as_ptr());
        self.cells.as_ptr()
    }

    // ...
}

use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        println!("{:?}", self.cells);
        for i in 0..self.width * self.height {
            let symbol = if self.cells.get_bit(i as usize) {
                "1"
            } else {
                "0"
            };
            write!(f, "{}", symbol)?;
            if (i + 1) % self.width == 0 {
                write!(f, "\n")?;
            }
        }

        Ok(())
    }
}