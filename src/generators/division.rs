use crate::grids::block_grid::BlockGrid;
use crate::grids::{Direction, GridKind};
use rand::prelude::*;

pub struct RecursiveDivider {
    grid: BlockGrid,
    rng: ThreadRng,
}

impl RecursiveDivider {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            grid: BlockGrid::with_dims(rows, cols),
            rng: rand::thread_rng(),
        }
    }

    pub fn generate_maze(&mut self) -> Vec<GridKind> {
        self.subdivide(
            (1, 1),
            (self.grid.dims.rows - 1, self.grid.dims.columns - 1),
            &mut vec![],
        );

        self.grid.cells.clone()
    }

    pub fn subdivide(
        &mut self,
        bottom_left: (usize, usize),
        top_right: (usize, usize),
        dont_wall: &Vec<(usize, usize)>,
    ) {
        let mut do_x_divide = true;
        let mut do_y_divide = true;
        if bottom_left.0 >= top_right.0 && bottom_left.1 >= top_right.1 {
            return;
        } else if bottom_left.0 >= top_right.0 {
            do_x_divide = false;
        } else if bottom_left.1 >= top_right.1 {
            do_y_divide = false;
        }

        let bottom_x = if bottom_left.0 >= self.grid.dims.columns {
            self.grid.dims.columns - 1
        } else {
            bottom_left.0
        };
        let bottom_y = if bottom_left.1 >= self.grid.dims.rows {
            self.grid.dims.rows - 1
        } else {
            bottom_left.1
        };

        let bottom_left = (bottom_x, bottom_y);

        let diff_x = if do_x_divide {
            top_right.0 - bottom_left.0
        } else {
            0
        };
        let divide_x: usize = bottom_left.0 + (self.rng.gen::<f64>() * diff_x as f64) as usize;

        let diff_y = if do_y_divide {
            top_right.1 - bottom_left.1
        } else {
            0
        };
        let divide_y: usize = bottom_left.1 + (self.rng.gen::<f64>() * diff_y as f64) as usize;
        println!(
            "bot_left: {}, {}, top_right: {}, {} | divide_x {}, divide_y {}",
            bottom_left.0, bottom_left.1, top_right.0, top_right.1, divide_x, divide_y
        );
        // divide grid
        for i in bottom_left.0 - 1..top_right.0 + 1 {
            self.grid.set_cell(divide_y, i, GridKind::Wall);
        }
        for i in bottom_left.1 - 1..top_right.1 + 1 {
            self.grid.set_cell(i, divide_x, GridKind::Wall);
        }

        for (i, j) in dont_wall.iter() {
            self.grid.set_cell(*i, *j, GridKind::Empty);
        }

        // break walls
        let wall_to_leave_out: usize = (self.rng.gen::<f64>() * 4.0) as usize;

        let mut dont_wall = Vec::new();
        for i in 0..4 {
            if i == wall_to_leave_out {
                continue;
            }

            if !do_y_divide && i < 2 {
                continue;
            }

            if !do_x_divide && i > 1 {
                continue;
            }

            let rng: f64 = self.rng.gen();

            match Direction::from(i) {
                Direction::North => {
                    let break_point =
                        divide_y + 1 + ((top_right.1 - divide_y) as f64 * rng) as usize;
                    self.grid.set_cell(break_point, divide_x, GridKind::Empty);
                    if divide_x + 1 < self.grid.dims.columns {
                        dont_wall.push((break_point, divide_x + 1));
                    }
                    if divide_x > 0 {
                        dont_wall.push((break_point, divide_x - 1));
                    }
                }
                Direction::South => {
                    let break_point =
                        divide_y - 1 - ((divide_y - bottom_left.1) as f64 * rng) as usize;
                    self.grid.set_cell(break_point, divide_x, GridKind::Empty);
                    if divide_x + 1 < self.grid.dims.columns {
                        dont_wall.push((break_point, divide_x + 1));
                    }
                    if divide_x > 0 {
                        dont_wall.push((break_point, divide_x - 1));
                    }
                }
                Direction::East => {
                    let break_point =
                        divide_x + 1 + ((top_right.0 - divide_x) as f64 * rng) as usize;
                    self.grid.set_cell(divide_y, break_point, GridKind::Empty);
                    if divide_y + 1 < self.grid.dims.rows {
                        dont_wall.push((divide_y + 1, break_point));
                    }
                    if divide_y > 0 {
                        dont_wall.push((divide_y - 1, break_point));
                    }
                }
                Direction::West => {
                    let break_point =
                        divide_x - 1 - ((divide_x - bottom_left.0) as f64 * rng) as usize;
                    self.grid.set_cell(divide_y, break_point, GridKind::Empty);
                    if divide_y + 1 < self.grid.dims.rows {
                        dont_wall.push((divide_y + 1, break_point));
                    }
                    if divide_y > 0 {
                        dont_wall.push((divide_y - 1, break_point));
                    }
                }

                Direction::Sentinel => (),
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
