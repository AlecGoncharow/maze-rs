use crate::grids::block_grid::BlockGrid;
use crate::generators::Generator;
use rand::prelude::*;
use crate::grids::{Direction, GridKind};

pub struct AldousBroder {
    grid: BlockGrid,
    visited: Vec<bool>,
    rng: ThreadRng,
    current_cell: (usize, usize),
    current_cell_kind: GridKind,
    pub done: bool,
}

impl AldousBroder {
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut grid = BlockGrid::with_dims(rows, cols);
        grid.fill();
        let visited = (0..rows * cols).map(|index| {
            if index % cols == 0 
                || index % cols == cols - 1
                    || index / cols == 0
                    || index / cols == rows - 1 {
                        true 
                    } else {
                        false
            }
        }).collect();

        let current_cell = (1, 1);
        Self {
            grid,
            visited,
            rng: rand::thread_rng(),
            current_cell,
            current_cell_kind: GridKind::Empty,
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
            let neighbor_idx = (self.current_cell.0 * self.grid.dims.columns) + self.current_cell.1;
            self.visited[neighbor_idx] = true;
            let rand_wall = self.current_cell;
            let neighbors = self.grid.get_neighborhood_of(rand_wall.0, rand_wall.1);

            let rand_neighbor = loop {
                let choose = (self.rng.gen::<f32>() * 4.0) as usize;

                let neighbor = loop {
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
                if neighbor.1.0 == 0 || neighbor.1.0 == self.grid.dims.rows - 1 || neighbor.1.1 == 0 || neighbor.1.1 == self.grid.dims.columns - 1 {
                    continue;
                } else {
                    break neighbor;
                }
            };


            /*
                for (neighbor, _dir) in neighbors {

            */

                let neighborhood_of_neighor = self.grid.get_neighborhood_of(rand_neighbor.1.0, rand_neighbor.1.1);

            let mut count = 0;
            for neighbor in neighborhood_of_neighor {
                if (neighbor.0.0 != GridKind::Wall && neighbor.0.0 != GridKind::Cursor) || (self.current_cell_kind != GridKind::Wall && neighbor.0.0 == GridKind::Cursor) {
                    count += 1;
                }
            }
            self.grid
                    .set_cell(self.current_cell.0, self.current_cell.1, self.current_cell_kind);
        
            if count == 1 {
                self.current_cell_kind = GridKind::Empty;
            } else {
                self.current_cell_kind = rand_neighbor.0;
            }
            self.current_cell = rand_neighbor.1; 
            let rand_neighbor_idx = (rand_neighbor.1.0 * self.grid.dims.columns) + rand_neighbor.1.1;
            self.visited[rand_neighbor_idx] = true;
            self.grid
                    .set_cell(rand_neighbor.1.0, rand_neighbor.1.1, GridKind::Cursor);
    }

    fn next_step(&mut self) -> Vec<GridKind> {
        self.step_generation();
        self.grid.cells.clone()
    }

    fn generate_maze(&mut self) -> Vec<GridKind> {
        loop {
            self.step_generation();
            if self.done {
                break;
            }
        }
        self.grid
            .set_cell(self.current_cell.0, self.current_cell.1, self.current_cell_kind);

        self.grid.cells.clone()
    }


    fn is_done(&self) -> bool {
        self.done
    }
}
