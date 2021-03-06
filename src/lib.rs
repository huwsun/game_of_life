#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;

    #[wasm_bindgen(js_namespace = console)]
    fn log(msg: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn time(name: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn timeEnd(name: &str);

    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
}

macro_rules! log {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

type Cells = Vec<u8>;

trait BitOper<T> {
    fn get_bit(&self, idx: T) -> bool;
    fn set_bit(&mut self, idx: T, val: bool);
    fn toggle(&mut self, idx: T);
}

impl BitOper<u8> for u8 {
    fn set_bit(&mut self, idx: u8, val: bool) {
        if val {
            *self = *self | 1 << idx;
        } else {
            *self = *self & !(1 << idx);
        }
    }

    fn get_bit(&self, idx: u8) -> bool {
        (self & 1 << idx) != 0
    }

    fn toggle(&mut self, idx: u8) {
        *self = *self ^ 1 << idx;
    }
}

impl BitOper<usize> for Cells {
    fn get_bit(&self, idx: usize) -> bool {
        self[idx / 8].get_bit((idx % 8) as u8)
    }

    fn set_bit(&mut self, idx: usize, val: bool) {
        self[idx / 8].set_bit( (idx % 8) as u8, val);
    }

    fn toggle(&mut self, idx: usize) {
        self[idx / 8].toggle( (idx % 8) as u8);
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Cells,
    pres: Cells,
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        let north = if row == 0 { self.height - 1 } else { row - 1 };

        let south = if row == self.height - 1 { 0 } else { row + 1 };

        let west = if column == 0 {
            self.width - 1
        } else {
            column - 1
        };

        let east = if column == self.width - 1 {
            0
        } else {
            column + 1
        };

        let cells = &self.pres;
        let idx = self.get_index(north, west);
        count += cells.get_bit(idx) as u8;

        let idx = self.get_index(north, column);
        count += cells.get_bit(idx) as u8;

        let idx = self.get_index(north, east);
        count += cells.get_bit(idx) as u8;

        let idx = self.get_index(row, west);
        count += cells.get_bit(idx) as u8;

        let idx = self.get_index(row, east);
        count += cells.get_bit(idx) as u8;

        let idx = self.get_index(south, west);
        count += cells.get_bit(idx) as u8;

        let idx = self.get_index(south, column);
        count += cells.get_bit(idx) as u8;

        let idx = self.get_index(south, east);
        count += cells.get_bit(idx) as u8;
        count
    }
    // ...
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn new(width: u32, height: u32) -> Universe {
        let len = (width * height) as usize;
        let vlen = if len % 8 == 0 { len / 8 } else { len / 8 + 1 };
        let mut cells = vec![0u8; vlen];

        for i in 0..len {
            if i % 2 == 0 || i % 7 == 0 {
                cells.set_bit(i, true);
            } else {
                cells.set_bit(i, false);
            }
        }
        //log!("new cells:{:?}",cells);
        Universe {
            width,
            height,
            cells,
            pres: vec![0u8; vlen],
        }
    }
    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");

        for i in 0..((self.width * self.height) as usize) {
            self.pres.set_bit(i, self.cells.get_bit(i));
        }

        //let _timer = Timer::new("new generation");

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.pres.get_bit(idx);
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

                self.cells.set_bit(idx, next_cell);
            }
        }
    }

    pub fn rand_gen(&mut self) {
        for i in 0..((self.width * self.height) as usize) {
            self.cells.set_bit(i, random() > 0.4995);
        }
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells.toggle(idx);
    }

    pub fn clear(&mut self) {
        self.cells.iter_mut().for_each(|x| *x = 0);
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

    pub fn render(&self) -> String {
        self.to_string()
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

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        time(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        timeEnd(self.name);
    }
}
