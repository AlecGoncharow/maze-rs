use crate::grids::block_grid::BlockGrid;
use crate::grids::wall_grid::WallGrid;
use crate::generators::Generator;
use rand::prelude::*;
use crate::grids::{Direction, CellKind, Grid, GridKind};

pub struct AldousBroder {
    grid: Box<dyn Grid>,
    grid_kind: GridKind,
    visited: Vec<bool>,
    rng: ThreadRng,
    current_cell: (usize, usize),
    current_cell_kind: CellKind,
    pub done: bool,
}

impl AldousBroder {
    pub fn new(rows: usize, cols: usize, kind: GridKind) -> Self {
        let mut grid: Box<dyn Grid> = match kind {
            GridKind::Block => Box::new(BlockGrid::with_dims(rows, cols)),
            GridKind::Wall => Box::new(WallGrid::with_dims(rows, cols)),
        };
        grid.fill();

        let visited = match kind {
            GridKind::Block => (0..rows * cols).map(|index| {
            if index % cols == 0 
                || index % cols == cols - 1
                    || index / cols == 0
                    || index / cols == rows - 1 {
                        true 
                    } else {
                        false
            }
        }).collect(),
            GridKind::Wall => vec![false; rows * cols],
        };

        let current_cell = (1, 1);
        Self {
            grid,
            grid_kind: kind,
            visited,
            rng: rand::thread_rng(),
            current_cell,
            current_cell_kind: CellKind::Empty,
            done: false,
        }
    }
}


impl Generator for AldousBroder {
    fn step_generation(&mut self) {
            if !self.visited.contains(&false) {
                self.done = true;
                self.grid
                    .set_cell(self.current_cell.0, self.current_cell.1, self.current_cell_kind);
                return;
            }
            let neighbor_idx = (self.current_cell.0 * self.grid.dims().columns) + self.current_cell.1;
            self.visited[neighbor_idx] = true;
            let rand_wall = self.current_cell;
            let neighbors = self.grid.get_neighborhood_of(rand_wall.0, rand_wall.1);

            let rand_neighbor = loop {
                let neighbor = loop {
                    let choose = (self.rng.gen::<f32>() * 4.0) as usize;
                    match crate::grids::Direction::from(choose) {
                        Direction::North => {
                            if let Some(north) = neighbors.north {
                                break north;
                            }
                        }
                        Direction::South => {
                            if let Some(south) = neighbors.south {
                                break south;
                            }
                        }
                        Direction::East => {
                            if let Some(east) = neighbors.east {
                                break east;
                            }
                        }
                        Direction::West => {
                            if let Some(west) = neighbors.west {
                                break west;
                            }
                        }
                        Direction::Sentinel => {continue}
                    }};
                if self.grid_kind == GridKind::Block 
                    && (neighbor.1.0 == 0 || neighbor.1.0 == self.grid.dims().rows - 1 || neighbor.1.1 == 0 || neighbor.1.1 == self.grid.dims().columns - 1) {
                    continue;
                } else {
                    break neighbor;
                }
            };


            /*
                for (neighbor, _dir) in neighbors {

            */

            match self.grid_kind {
                GridKind::Block => {
                    let neighborhood_of_neighor = self.grid.get_neighborhood_of(rand_neighbor.1.0, rand_neighbor.1.1);

                    let mut count = 0;
                    for neighbor in neighborhood_of_neighor {
                        if (neighbor.0.0 != CellKind::Wall && neighbor.0.0 != CellKind::Cursor) || (self.current_cell_kind != CellKind::Wall && neighbor.0.0 == CellKind::Cursor) {
                            count += 1;
                        }
                    }

                    self.grid.set_cell(self.current_cell.0, self.current_cell.1, self.current_cell_kind);

                    if count == 1 {
                        self.current_cell_kind = CellKind::Empty;
                    } else {
                        self.current_cell_kind = rand_neighbor.0;
                    }
                }
                GridKind::Wall => {
                    self.grid.set_cell(self.current_cell.0, self.current_cell.1, self.current_cell_kind);
                    let rand_neighbor_idx = (rand_neighbor.1.0 * self.grid.dims().columns) + rand_neighbor.1.1;
                    if !self.visited[rand_neighbor_idx] {
                        self.grid.clear_wall_between(self.current_cell, rand_neighbor.1);
                    }
                }
            }
            self.current_cell = rand_neighbor.1; 
            let rand_neighbor_idx = (rand_neighbor.1.0 * self.grid.dims().columns) + rand_neighbor.1.1;
            self.visited[rand_neighbor_idx] = true;
            self.grid
                    .set_cell(rand_neighbor.1.0, rand_neighbor.1.1, CellKind::Cursor);
    }

    fn next_step(&mut self) -> &dyn Grid {
        self.step_generation();
        self.grid.as_ref()
    }

    fn generate_maze(&mut self) -> &dyn Grid {
        loop {
            self.step_generation();
            if self.done {
                break;
            }
        }
        self.grid
            .set_cell(self.current_cell.0, self.current_cell.1, self.current_cell_kind);

        self.grid.as_ref()
    }


    fn is_done(&self) -> bool {
        self.done
    }
}
