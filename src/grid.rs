const WORD_BYTES: usize = std::mem::size_of::<usize>();
const WORD_BITS: usize = WORD_BYTES * 8;
const DEFAULT_DIMS: (usize, usize) = (16, 16);

pub const GRID_SCALE: f32 = 1.5;
pub const SQUARE_GAP: f32 = 0.01;

use crate::renderer::Vertex;
use crate::State;

pub struct Dimensions {
    rows: usize,
    columns: usize,
}

pub struct Grid {
    dims: Dimensions,

    /// uses each bit of vector to represent a square in the grid as being toggled or not
    /// this will require a refactor later but c'est la vie
    pub squares: Vec<usize>,
}

impl Grid {
    pub fn new() -> Self {
        Self::with_dims(DEFAULT_DIMS.0, DEFAULT_DIMS.1)
    }

    pub fn with_dims(rows: usize, columns: usize) -> Self {
        Self {
            squares: vec![0; (rows * columns) / WORD_BITS + 1],
            dims: Dimensions { rows, columns },
        }
    }

    pub fn is_set(&self, row: usize, column: usize) -> bool {
        let word_row = (self.dims.columns * row) / WORD_BITS;
        let word_col = column / WORD_BITS;
        let offset = column % WORD_BITS + ((self.dims.columns * row) % WORD_BITS);

        let word = self.squares[word_row + word_col];

        get_bit(word, offset)
    }

    pub fn toggle_square(&mut self, row: usize, column: usize) -> bool {
        self._set_square(row, column, toggle_bit)
    }

    pub fn set_square(&mut self, row: usize, column: usize) -> bool {
        self._set_square(row, column, set_bit)
    }

    pub fn unset_square(&mut self, row: usize, column: usize) -> bool {
        self._set_square(row, column, unset_bit)
    }

    fn _set_square<F>(&mut self, row: usize, column: usize, fun: F) -> bool
    where
        F: FnOnce(usize, usize) -> usize,
    {
        let word_row = (self.dims.columns * row) / WORD_BITS;
        let word_col = column / WORD_BITS;
        let offset = column % WORD_BITS + ((self.dims.columns * row) % WORD_BITS);

        let prev_word = self.squares[word_row + word_col];

        self.squares[word_row + word_col] = fun(prev_word, offset);

        get_bit(prev_word, offset)
    }

    pub fn handle_click(&mut self, pos: (f32, f32), size: winit::dpi::PhysicalSize<u32>) {
        let x = (2.0 * pos.0) - 1.0;
        let y = (2.0 * pos.1) - 1.0;
        let y = -y;

        let (sq_width, sq_height, bottom_left_x, bottom_left_y) = self.get_ndc_params(size);

        let (row, column) = {
            let x = x - bottom_left_x;
            let y = y - bottom_left_y;

            (
                (y / (sq_height + SQUARE_GAP)) as usize,
                (x / (sq_width + SQUARE_GAP)) as usize,
            )
        };

        if row < self.dims.rows && column < self.dims.columns {
            self.toggle_square(row, column);
        }
    }

    fn get_ndc_params(&self, size: winit::dpi::PhysicalSize<u32>) -> (f32, f32, f32, f32) {
        let ratio = size.width as f32 / size.height as f32;
        let (sq_width, sq_height) = if ratio >= 1.0 {
            (
                GRID_SCALE / self.dims.columns as f32 / ratio,
                GRID_SCALE / self.dims.rows as f32,
            )
        } else {
            (
                GRID_SCALE / self.dims.columns as f32,
                GRID_SCALE / self.dims.rows as f32 * ratio,
            )
        };

        // centers the grid somehow, this will need some additional calculations to detect large
        // gaps, but good enough for now to buy space
        let bottom_left_x =
            (2.0 - (GRID_SCALE + (self.dims.columns as f32 * SQUARE_GAP))) / 2.0 - 1.0;
        let bottom_left_y = (2.0 - (GRID_SCALE + (self.dims.rows as f32 * SQUARE_GAP))) / 2.0 - 1.0;

        (sq_width, sq_height, bottom_left_x, bottom_left_y)
    }

    pub fn render(&self, state: &State) -> Vec<Vertex> {
        let mut grid = Vec::new();
        let (sq_width, sq_height, center_x, center_y) = self.get_ndc_params(state.gfx_ctx.size);

        let mut offset_x = center_x;
        let mut offset_y = center_y;

        //@TODO factor in GRID_SCALE somehow so that the grid has a margin from the border of the
        //screen, make boxes touch?
        for row in 0..self.dims.rows {
            for col in 0..self.dims.columns {
                let low_x = offset_x;
                let low_y = offset_y;

                let up_x = low_x + sq_width;
                let up_y = low_y + sq_height;

                let color = if self.is_set(row, col) {
                    [0.0, 0.0, 0.0, 1.0]
                } else {
                    [1.0, 1.0, 1.0, 1.0]
                };

                let verts: &[Vertex] = &[
                    // lower left triangle
                    Vertex {
                        position: [low_x, low_y],
                        color,
                    },
                    Vertex {
                        position: [up_x, low_y],
                        color,
                    },
                    Vertex {
                        position: [low_x, up_y],
                        color,
                    },
                    // upper right triangle
                    Vertex {
                        position: [low_x, up_y],
                        color,
                    },
                    Vertex {
                        position: [up_x, low_y],
                        color,
                    },
                    Vertex {
                        position: [up_x, up_y],
                        color,
                    },
                ];

                grid.append(&mut Vec::from(verts));

                offset_x += sq_width + SQUARE_GAP;
            }
            offset_y += sq_height + SQUARE_GAP;
            offset_x = center_x;
        }

        grid
    }
}

#[inline(always)]
fn get_bit(n: usize, k: usize) -> bool {
    if (n >> k) & 1 == 0 {
        false
    } else {
        true
    }
}

#[inline(always)]
pub fn set_bit(n: usize, k: usize) -> usize {
    n | (1 << k)
}

#[inline(always)]
pub fn unset_bit(n: usize, k: usize) -> usize {
    n & !(1 << k)
}

#[inline(always)]
pub fn toggle_bit(n: usize, k: usize) -> usize {
    n ^ (1 << k)
}

#[cfg(test)]
mod test_grid {
    use super::*;

    #[test]
    fn it_works() {
        let mut grid = Grid::with_dims(200, 400);

        grid.set_square(1, 2);
        grid.set_square(0, 0);
        grid.set_square(4, 4);
        grid.set_square(3, 2);
        grid.set_square(1, 1);

        assert!(grid.is_set(0, 0));
        assert!(!grid.is_set(0, 1));

        assert!(grid.unset_square(0, 0));
        assert!(!grid.is_set(0, 0));

        assert!(!grid.toggle_square(14, 1));
        assert!(grid.is_set(14, 1));

        assert!(!grid.toggle_square(100, 300));
        assert!(grid.is_set(100, 300));
    }
}
