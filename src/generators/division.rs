use crate::grid::{Grid, GridKind};
use rand::prelude::*;

pub struct RecursiveDivider {
    grid: Grid,
    rng: ThreadRng,
}

enum Direction {
    North = 0,
    South = 1,
    East = 2,
    West = 3,
}

impl From<usize> for Direction {
    fn from(dir: usize) -> Self {
        match dir {
            0 => Direction::North,
            1 => Direction::South,
            2 => Direction::East,
            3 => Direction::West,
            _ => unreachable!(),
        }
    }
}

impl RecursiveDivider {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            grid: Grid::with_dims(rows, cols),
            rng: rand::thread_rng(),
        }
    }

    pub fn generate_maze(&mut self) -> Vec<GridKind> {
        self.subdivide(
            (1, 1),
            (self.grid.dims.rows - 1, self.grid.dims.columns - 1),
            &vec![],
        );

        self.grid.squares.clone()
    }

    pub fn subdivide(
        &mut self,
        bottom_left: (usize, usize),
        top_right: (usize, usize),
        dont_wall: &Vec<(usize, usize)>,
    ) {
        if bottom_left.0 >= top_right.0 || bottom_left.1 >= top_right.1 {
            return;
        }

        let diff_x = top_right.0 - bottom_left.0;
        if diff_x <= 1 {
            return;
        }
        let divide_x: usize = bottom_left.0 + (self.rng.gen::<f64>() * diff_x as f64) as usize;

        let diff_y = top_right.1 - bottom_left.1;
        if diff_y <= 1 {
            return;
        }
        let divide_y: usize = bottom_left.1 + (self.rng.gen::<f64>() * diff_y as f64) as usize;

        // divide grid
        for i in bottom_left.0 - 1..top_right.0 + 1 {
            self.grid.set_square(divide_y, i, GridKind::Wall);
        }
        for i in bottom_left.1 - 1..top_right.1 + 1 {
            self.grid.set_square(i, divide_x, GridKind::Wall);
        }

        for (i, j) in dont_wall {
            self.grid.set_square(*i, *j, GridKind::Empty);
        }

        // break walls
        let wall_to_leave_out: usize = (self.rng.gen::<f64>() * 4.0) as usize;
        println!(
            "bot_left: {}, {}, top_right: {}, {} | divide_x {}, divide_y {}",
            bottom_left.0, bottom_left.1, top_right.0, top_right.1, divide_x, divide_y
        );
        let mut dont_wall = Vec::new();
        for i in 0..4 {
            if i == wall_to_leave_out {
                continue;
            }

            let rng: f64 = self.rng.gen();

            match Direction::from(i) {
                Direction::North => {
                    let break_point =
                        divide_y + 1 + ((top_right.1 - divide_y) as f64 * rng) as usize;
                    self.grid.set_square(break_point, divide_x, GridKind::Empty);
                    dont_wall.push((break_point, divide_x + 1));
                    dont_wall.push((break_point, divide_x - 1));
                }
                Direction::South => {
                    let break_point =
                        divide_y - 1 - ((divide_y - bottom_left.1) as f64 * rng) as usize;
                    self.grid.set_square(break_point, divide_x, GridKind::Empty);
                    dont_wall.push((break_point, divide_x + 1));
                    dont_wall.push((break_point, divide_x - 1));
                }
                Direction::East => {
                    let break_point =
                        divide_x + 1 + ((top_right.0 - divide_x) as f64 * rng) as usize;
                    self.grid.set_square(divide_y, break_point, GridKind::Empty);
                    dont_wall.push((divide_y + 1, break_point));
                    dont_wall.push((divide_y - 1, break_point));
                }
                Direction::West => {
                    let break_point =
                        divide_x - 1 - ((divide_x - bottom_left.0) as f64 * rng) as usize;
                    self.grid.set_square(divide_y, break_point, GridKind::Empty);
                    dont_wall.push((divide_y + 1, break_point));
                    dont_wall.push((divide_y - 1, break_point));
                }
            }
        }

        // recurse

        // TOP LEFT
        self.subdivide(
            (bottom_left.0, divide_y + 2),
            (divide_x - 1, top_right.1),
            &dont_wall,
        );
        // TOP RIGHT
        self.subdivide(
            (divide_x + 2, divide_y + 2),
            (top_right.0, top_right.1),
            &dont_wall,
        );
        // BOT LEFT
        self.subdivide(
            (bottom_left.0, bottom_left.1),
            (divide_x - 1, divide_y - 1),
            &dont_wall,
        );
        // BOT RIGHT
        self.subdivide(
            (divide_x + 2, bottom_left.1),
            (top_right.0, divide_y - 1),
            &dont_wall,
        );
    }
}
