const WORD_BYTES: usize = std::mem::size_of::<usize>();
const WORD_BITS: usize = WORD_BYTES * 8;
const DEFAULT_DIMS: (usize, usize) = (16, 16);

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
