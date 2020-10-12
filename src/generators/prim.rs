use crate::grids::block_grid::{BlockGrid, GridKind};
use crate::generators::Generator;
use rand::prelude::*;

pub struct RandPrims {
    grid: BlockGrid,
    walls: Vec<(usize, usize)>,
    rng: ThreadRng,
    last_passage: (usize, usize),
    pub done: bool
}

impl RandPrims {
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut grid = BlockGrid::with_dims(rows, cols);
        grid.fill();
        let mut rng = rand::thread_rng();
        // make it odd
        let row = (rng.gen::<f32>() * (rows - 1) as f32) as usize | 1;
        let col = (rng.gen::<f32>() * (cols - 1) as f32) as usize | 1;
        let walls = vec![(row, col)];
        let last_passage = (row, col);
        Self {
            grid,
            walls,
            last_passage,
            rng,
            done: false,
        }
    }
}

impl Generator for RandPrims {
    fn step_generation(&mut self) {
        // loop until wall is found
        loop {
            if self.walls.len() == 0 {
                self.done = true;
                self.grid.set_square(self.last_passage.0, self.last_passage.1, GridKind::Empty);
                break;
            }
            let rand_wall_idx = (self.walls.len() as f32 * self.rng.gen::<f32>()) as usize;
            let rand_wall = self.walls.remove(rand_wall_idx);
            let neighbors = self.grid.get_neighborhood_of(rand_wall.0, rand_wall.1);


            let mut count = 0;
            let mut unwalled_dir = None;
            for (neighbor, dir) in neighbors {
                if neighbor.0 != GridKind::Wall {
                    count += 1;
                    unwalled_dir = Some(dir);
                }
            }

            if count < 2 {
                self.grid.set_square(self.last_passage.0, self.last_passage.1, GridKind::Empty);
                self.grid
                    .set_square(rand_wall.0, rand_wall.1, GridKind::Empty);

                if let Some(dir) = unwalled_dir {
                    self.last_passage = self.grid.set_neighbor_of(rand_wall, -dir, GridKind::Cursor);
                }
                let mut walls_to_add = Vec::new();
                for (neighbor, _) in self.grid.get_neighborhood_of(self.last_passage.0, self.last_passage.1) {

                    if neighbor.0 == GridKind::Wall {
                        if neighbor.1.0 == 0 || neighbor.1.0 == self.grid.dims.rows - 1 || neighbor.1.1 == 0 || neighbor.1.1 == self.grid.dims.columns - 1 {
                            continue;
                        }
                        walls_to_add.push(neighbor.1)
                    }
                }

                self.walls.append(&mut walls_to_add);
                break;
            }
        }
    }

    fn next_step(&mut self) -> Vec<GridKind> {
        self.step_generation();
        self.grid.squares.clone()
    }

    fn generate_maze(&mut self) -> Vec<GridKind> {
        loop {
            self.step_generation();
            if self.done {
                break;
            }
        }
        self.grid.set_square(self.last_passage.0, self.last_passage.1, GridKind::Empty);

        self.grid.squares.clone()
    }

    fn is_done(&self) -> bool {
        self.done
    }
}
