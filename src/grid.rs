const WORD_BYTES: usize = std::mem::size_of::<usize>();
const WORD_BITS: usize = WORD_BYTES * 8;
const DEFAULT_DIMS: (usize, usize) = (16, 16);

pub const GRID_SCALE: f32 = 1.3;
pub const SQUARE_GAP: f32 = 0.005;

use crate::generators::{Direction, Neighborhood};
use crate::renderer::Vertex;
use crate::State;

pub struct Dimensions {
    pub rows: usize,
    pub columns: usize,
}

//use bit_graph::hash::HashGraph;
//use bit_graph::baseline::AdjGraph;
use bit_graph::search::a_star::AStarMH;
use bit_graph::search::bfs::BFS;
use bit_graph::search::dfs::DFS;
use bit_graph::search::Pathfinder;
use bit_graph::BitGraph;
use bit_graph::Graph;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GridKind {
    Empty,
    Wall,
    Start,
    Goal,
    Path,
    Explored,
    Cursor,
}

impl From<GridKind> for [f32; 4] {
    fn from(kind: GridKind) -> Self {
        match kind {
            GridKind::Empty => [1.0, 1.0, 1.0, 1.0],
            GridKind::Wall => [0.0, 0.0, 0.0, 1.0],
            GridKind::Start => [1.0, 0.0, 0.0, 1.0],
            GridKind::Goal => [1.0, 1.0, 0.0, 1.0],
            GridKind::Explored => [0.2, 0.2, 0.6, 1.0],
            GridKind::Path => [0.1, 0.5, 0.1, 1.0],
            GridKind::Cursor => [0.0, 0.5, 0.3, 1.0],
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum SolverKind {
    DFS,
    BFS,
    AStar,
}

pub struct Grid {
    pub dims: Dimensions,

    /// uses each bit of vector to represent a square in the grid as being toggled or not
    /// this will require a refactor later but c'est la vie
    pub squares: Vec<GridKind>,

    pub start: Option<(usize, usize)>,
    pub goal: Option<(usize, usize)>,
    pub cursor: Option<(usize, usize)>,

    pub graph: Option<Box<dyn Graph<u64, bool>>>,
    pub solver: Option<Box<dyn Pathfinder<u64, bool>>>,
    pub solver_kind: SolverKind,
}

impl Grid {
    pub fn new() -> Self {
        Self::with_dims(DEFAULT_DIMS.0, DEFAULT_DIMS.1)
    }

    pub fn with_dims(rows: usize, columns: usize) -> Self {
        Self {
            squares: vec![GridKind::Empty; rows * columns],
            dims: Dimensions { rows, columns },
            start: None,
            goal: None,
            cursor: None,
            graph: None,
            solver: None,
            solver_kind: SolverKind::BFS,
        }
    }

    pub fn is_set(&self, row: usize, column: usize) -> bool {
        let word_row = self.dims.columns * row;
        let word_col = column;

        let kind = self.squares[word_row + word_col];

        kind != GridKind::Empty
    }

    pub fn get_square(&self, row: usize, column: usize) -> GridKind {
        let word_row = self.dims.columns * row;
        let word_col = column;

        self.squares[word_row + word_col]
    }

    // returns coords of neighbor
    pub fn set_neighbor_of(
        &mut self,
        coords: (usize, usize),
        direction: Direction,
        kind: GridKind,
    ) -> (usize, usize) {
        let row = coords.0;
        let column = coords.1;

        let (n_row, n_col) = match direction {
            Direction::North => (row + 1, column),
            Direction::South => (row - 1, column),
            Direction::East => (row, column + 1),
            Direction::West => (row, column - 1),
            Direction::Sentinel => panic!("no"),
        };

        self.set_square(n_row, n_col, kind);

        (n_row, n_col)
    }

    pub fn get_neighborhood_of(&self, row: usize, column: usize) -> Neighborhood {
        let mut neighbors = Neighborhood::new();
        let word_row = self.dims.columns * row;
        let word_col = column;
        let index = word_row + word_col;

        neighbors.north = if let Some(kind) = self.squares.get(index + self.dims.columns) {
            Some((*kind, (row + 1, column)))
        } else {
            None
        };

        neighbors.south = if index >= self.dims.columns {
            Some((self.squares[index - self.dims.columns], (row - 1, column)))
        } else {
            None
        };

        neighbors.east = if column < self.dims.columns - 1 {
            Some((self.squares[index + 1], (row, column + 1)))
        } else {
            None
        };

        neighbors.west = if column > 0 {
            Some((self.squares[index - 1], (row, column - 1)))
        } else {
            None
        };

        neighbors
    }

    pub fn set_square(&mut self, row: usize, column: usize, kind: GridKind) -> GridKind {
        let word_row = self.dims.columns * row;
        let word_col = column;
        let prev_kind = self.squares[word_row + word_col];
        self.squares[word_row + word_col] = kind;

        prev_kind
    }

    pub fn unset_square(&mut self, row: usize, column: usize) -> GridKind {
        self.set_square(row, column, GridKind::Empty)
    }

    pub fn clear(&mut self) {
        self.squares = vec![GridKind::Empty; self.squares.len()];
        self.start = None;
        self.goal = None;
        self.cursor = None;
    }

    pub fn fill(&mut self) {
        self.squares = vec![GridKind::Wall; self.squares.len()];
        self.start = None;
        self.goal = None;
        self.cursor = None;
    }

    pub fn toggle_square(&mut self, row: usize, column: usize, kind: GridKind) -> GridKind {
        let word_row = self.dims.columns * row;
        let word_col = column;
        let prev_kind = self.squares[word_row + word_col];

        if prev_kind == GridKind::Empty {
            self.squares[word_row + word_col] = kind;
        } else if prev_kind != kind {
            self.squares[word_row + word_col] = kind;
        } else {
            self.squares[word_row + word_col] = GridKind::Empty;
        }

        if kind == GridKind::Start {
            if let Some(start) = self.start {
                self.unset_square(start.0, start.1);
            }
            if self.squares[word_row + word_col] == GridKind::Start {
                self.start = Some((row, column));
            }
        }

        if kind == GridKind::Goal {
            if let Some(goal) = self.goal {
                self.unset_square(goal.0, goal.1);
            }
            if self.squares[word_row + word_col] == GridKind::Goal {
                self.goal = Some((row, column));
            }
        }

        prev_kind
    }

    pub fn handle_click(
        &mut self,
        pos: (f32, f32),
        size: winit::dpi::PhysicalSize<u32>,
        kind: GridKind,
    ) {
        let x = (2.0 * pos.0) - 1.0;
        let y = (2.0 * pos.1) - 1.0;
        let y = -y;

        let (sq_width, sq_height, bottom_left_x, bottom_left_y) = self.get_ndc_params(size);

        let (row, column) = {
            let x = x - bottom_left_x;
            let y = y - bottom_left_y;

            if x < 0. || y < 0. {
                return;
            }

            (
                (y / (sq_height + SQUARE_GAP)) as usize,
                (x / (sq_width + SQUARE_GAP)) as usize,
            )
        };

        if row < self.dims.rows && column < self.dims.columns {
            self.toggle_square(row, column, kind);
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

                let color: [f32; 4] = self.get_square(row, col).into();

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

    pub fn make_graph(&mut self) {
        let mut graph = BitGraph::with_capacity(self.dims.rows * self.dims.columns);

        for _ in &self.squares {
            graph.push_node(1);
        }

        for i in 0..self.squares.len() {
            let square = &self.squares[i];
            if square == &GridKind::Wall {
                continue;
            }
            let row = i / self.dims.columns;
            let col = i % self.dims.columns;

            if square == &GridKind::Path || square == &GridKind::Explored {
                self.set_square(row, col, GridKind::Empty);
            }

            // get directions
            if row > 0 && self.squares[i - self.dims.columns] != GridKind::Wall {
                graph.add_edge(i, i - self.dims.columns);
            }

            if row < self.dims.rows - 1 && self.squares[i + self.dims.columns] != GridKind::Wall {
                graph.add_edge(i, i + self.dims.columns);
            }

            if col > 0 && self.squares[i - 1] != GridKind::Wall {
                graph.add_edge(i, i - 1);
            }

            if col < self.dims.columns - 1 && self.squares[i + 1] != GridKind::Wall {
                graph.add_edge(i, i + 1);
            }
        }

        self.graph = Some(Box::new(graph));
        let graph = &**self.graph.as_ref().unwrap();
        let root = self.start.unwrap();
        let index = (self.dims.columns * root.0) + root.1;
        let goal = match self.goal {
            Some(inner) => inner,
            None => {
                if self.solver_kind == SolverKind::AStar {
                    panic!("Astar requires a goal")
                } else {
                    (0, 0)
                }
            }
        };
        let goal_idx = (self.dims.columns * goal.0) + goal.1;
        self.solver = Some(match self.solver_kind {
            SolverKind::BFS => Box::new(BFS::new(graph, index)),
            SolverKind::DFS => Box::new(DFS::new(graph, index)),
            SolverKind::AStar => Box::new(AStarMH::new(graph, index, goal_idx, self.dims.columns)),
        });
    }

    pub fn step_solve_path(&mut self) -> bool {
        if self.start.is_none() || self.goal.is_none() {
            return false;
        }

        let start = self.start.unwrap();
        let goal = self.goal.unwrap();
        self.set_square(start.0, start.1, GridKind::Start);
        self.set_square(goal.0, goal.1, GridKind::Goal);

        let mut index = 0;
        loop {
            if self.squares[index] == GridKind::Cursor {
                self.squares[index] = GridKind::Explored;
                break;
            }
            index += 1;

            if index >= self.squares.len() {
                break;
            }
        }

        if self.graph.is_none() {
            self.make_graph();
        }

        let solver = self.solver.as_mut().unwrap();
        let graph = &**self.graph.as_ref().unwrap();

        let (row, col, kind) = if solver.is_solved() {
            let cursor = self.cursor.unwrap();
            let idx = (cursor.0 * self.dims.columns) + cursor.1;
            let from = solver.from_index_of(idx);
            let row = from / self.dims.columns;
            let col = from % self.dims.columns;

            if row == start.0 && col == start.1 {
                return false;
            }

            self.cursor = Some((row, col));

            (row, col, GridKind::Path)
        } else {
            let (row, col, kind) = if let Some((idx, _from)) = solver.next(graph) {
                let row = idx / self.dims.columns;
                let col = idx % self.dims.columns;

                if row == goal.0 && col == goal.1 {
                    solver.set_solved();
                    self.cursor = Some((row, col));
                }

                (row, col, GridKind::Cursor)
            } else {
                return false;
            };

            (row, col, kind)
        };
        drop(solver);

        self.set_square(row, col, kind);
        true
    }

    pub fn solve_path(&mut self) {
        if self.start.is_none() || self.goal.is_none() {
            return;
        }

        self.make_graph();

        let start = self.start.unwrap();
        let goal = self.goal.unwrap();
        let root_idx = (start.0 * self.dims.columns) + start.1;
        let goal_idx = (goal.0 * self.dims.columns) + goal.1;
        println!("start: {}, goal: {}", root_idx, goal_idx);
        let graph = &**self.graph.as_ref().unwrap();
        let mut solver = self.solver.take().unwrap();

        if let Some(path) = solver.path_to(graph, goal_idx) {
            // pop off root
            println!("Path found: {:?}", path);

            for i in 1..path.len() - 1 {
                let row = path[i] / self.dims.columns;
                let col = path[i] % self.dims.columns;

                self.set_square(row, col, GridKind::Path);
            }
        } else {
            println!("path not found");
        }

        // clear graph for reasons
        self.graph = None;
    }
}

#[cfg(test)]
mod test_grid {
    use super::*;

    #[test]
    fn it_works() {
        let mut grid = Grid::with_dims(200, 400);

        grid.set_square(1, 2, GridKind::Wall);
        grid.set_square(0, 0, GridKind::Wall);
        grid.set_square(4, 4, GridKind::Wall);
        grid.set_square(3, 2, GridKind::Wall);
        grid.set_square(1, 1, GridKind::Wall);

        assert!(grid.is_set(0, 0));
        assert!(!grid.is_set(0, 1));

        assert!(grid.unset_square(0, 0) == GridKind::Wall);
        assert!(!grid.is_set(0, 0));

        assert!(!(grid.toggle_square(14, 1, GridKind::Wall) == GridKind::Wall));
        assert!(grid.is_set(14, 1));

        assert!(!(grid.toggle_square(100, 300, GridKind::Wall) == GridKind::Wall));
        assert!(grid.is_set(100, 300));
    }
}
