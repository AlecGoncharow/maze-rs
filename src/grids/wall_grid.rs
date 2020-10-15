const DEFAULT_DIMS: (usize, usize) = (15, 15);

use crate::grids::{Dimensions, Direction, GridKind};

pub struct WallGrid {
    pub dims: Dimensions,
    /// Each cell is a bit mask of sorts
    ///
    /// the high 4 bits pertain to the kind of cell, exlored, start, goal, etc...
    /// the lower 4 bits represented the existance of a wall in the WESN directions respectivley
    cells: Vec<u8>,
}

impl WallGrid {
    pub fn new() -> Self {
        Self::with_dims(DEFAULT_DIMS.0, DEFAULT_DIMS.1)
    }

    pub fn with_dims(rows: usize, columns: usize) -> Self {
        Self {
            cells: vec![0x00; rows * columns],
            dims: Dimensions { rows, columns },
        }
    }

    pub fn get_cell(&self, row: usize, column: usize) -> u8 {
        let byte_row = self.dims.columns * row;
        let byte_col = column;

        self.cells[byte_row + byte_col]
    }

    pub fn get_cell_kind(&self, row: usize, column: usize) -> GridKind {
        let byte = self.get_cell(row, column);

        let code = byte >> 4;

        GridKind::from(code)
    }

    pub fn walls_of(&self, row: usize, column: usize) -> Vec<Direction> {
        let byte = self.get_cell(row, column);
        let mut walls = Vec::new();
        for i in 0..4 {
            if byte >> i % 2 == 1 {
                walls.push(Direction::from(i));
            };
        }

        walls
    }
}
