const DEFAULT_DIMS: (usize, usize) = (15, 15);

use crate::grids::{CellKind, Dimensions, Direction, Grid, Neighborhood, SolverKind};
use bit_graph::{BitGraph, Graph};
pub const GRID_SCALE: f32 = 1.3;
pub const SQUARE_GAP: f32 = 0.005;

use crate::renderer::Vertex;
use crate::State;

type WalledCell = u8;

pub struct WallGrid {
    pub dims: Dimensions,

    pub cells: Vec<CellKind>,
    // graph edges represents existance of wall or not
    pub graph: Box<dyn Graph<u64, bool>>,

    pub start: Option<(usize, usize)>,
    pub goal: Option<(usize, usize)>,
    pub cursor: Option<(usize, usize)>,
    pub solver_kind: SolverKind,
}

impl WallGrid {
    pub fn new() -> Self {
        Self::with_dims(DEFAULT_DIMS.0, DEFAULT_DIMS.1)
    }

    pub fn with_dims(rows: usize, columns: usize) -> Self {
        Self {
            cells: vec![CellKind::Empty; rows * columns],
            dims: Dimensions { rows, columns },
            graph: Box::new(BitGraph::with_capacity(rows * columns)),
            start: None,
            goal: None,
            cursor: None,
            solver_kind: SolverKind::BFS,
        }
    }

    pub fn clear_wall_between(&mut self, one: (usize, usize), two: (usize, usize)) {
        let index_one = self.index_of(one.0, one.1);
        let index_two = self.index_of(two.0, two.1);

        // graph is directed and im lazy to make undirected
        self.graph.remove_edge(index_one, index_two);
        self.graph.remove_edge(index_two, index_one);
    }

    pub fn add_wall_between(&mut self, one: (usize, usize), two: (usize, usize)) {
        let index_one = self.index_of(one.0, one.1);
        let index_two = self.index_of(two.0, two.1);

        // graph is directed and im lazy to make undirected
        self.graph.add_edge(index_one, index_two);
        self.graph.add_edge(index_two, index_one);
    }

    // returns coords of neighbor
    fn get_neighbor_coords_of(
        &mut self,
        coords: (usize, usize),
        direction: Direction,
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

        (n_row, n_col)
    }

    pub fn toggle_cell(&mut self, row: usize, column: usize, kind: CellKind) -> CellKind {
        let index = self.index_of(row, column);
        let prev_kind = self.cells[index];

        if prev_kind == CellKind::Empty {
            self.cells[index] = kind;
        } else if prev_kind != kind {
            self.cells[index] = kind;
        } else {
            self.cells[index] = CellKind::Empty;
        }

        if kind == CellKind::Start {
            if let Some(start) = self.start {
                self.unset_cell(start.0, start.1);
            }
            if self.cells[index] == CellKind::Start {
                self.start = Some((row, column));
            }
        }

        if kind == CellKind::Goal {
            if let Some(goal) = self.goal {
                self.unset_cell(goal.0, goal.1);
            }
            if self.cells[index] == CellKind::Goal {
                self.goal = Some((row, column));
            }
        }

        prev_kind
    }

    #[inline]
    fn index_of(&self, row: usize, column: usize) -> usize {
        (self.dims.columns * row) + column
    }

    #[inline]
    pub fn unset_cell(&mut self, row: usize, column: usize) -> CellKind {
        self.set_cell(row, column, CellKind::Empty)
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
}

impl Grid for WallGrid {
    #[inline]
    fn get_cell(&self, row: usize, column: usize) -> CellKind {
        self.cells[self.index_of(row, column)]
    }

    #[inline]
    fn set_cell(&mut self, row: usize, column: usize, kind: CellKind) -> CellKind {
        let index = self.index_of(row, column);
        let prev_kind = self.cells[index];
        self.cells[index] = kind;
        prev_kind
    }

    fn handle_click(
        &mut self,
        pos: (f32, f32),
        size: winit::dpi::PhysicalSize<u32>,
        kind: CellKind,
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
            self.toggle_cell(row, column, kind);
        }
    }

    fn render(&self, state: &State) -> Vec<Vertex> {
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

                let color: [f32; 4] = self.get_cell(row, col).into();

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
                if col != self.dims.columns - 1 {
                    let index = (row * self.dims.columns) + col;
                    let color: [f32; 4] = if self.graph.has_edge(index, index + 1) {
                        [1.0, 1.0, 1.0, 1.0]
                    } else {
                        [0.0, 0.0, 0.0, 1.0]
                    };
                    let low_x = low_x + sq_width;
                    let up_x = low_x + SQUARE_GAP;
                    let up_y = up_y + SQUARE_GAP;

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
                }

                if row != self.dims.rows - 1 {
                    let index = (row * self.dims.columns) + col;

                    let color: [f32; 4] = if self.graph.has_edge(index, index + self.dims.columns) {
                        [1.0, 1.0, 1.0, 1.0]
                    } else {
                        [0.0, 0.0, 0.0, 1.0]
                    };
                    let up_x = low_x + SQUARE_GAP + sq_width;
                    let low_y = up_y;
                    let up_y = low_y + SQUARE_GAP;

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
                }
            }
            offset_y += sq_height + SQUARE_GAP;
            offset_x = center_x;
        }

        grid
    }

    fn cells(&self) -> &Vec<CellKind> {
        &self.cells
    }

    fn set_cells(&mut self, cells: Vec<CellKind>) {
        self.cells = cells;
    }

    fn set_solver_kind(&mut self, kind: super::SolverKind) {
        self.solver_kind = kind;
    }

    fn solve_path(&mut self) {
        todo!()
    }

    fn step_solve_path(&mut self) -> bool {
        todo!()
    }

    fn clear(&mut self) {
        self.cells = vec![CellKind::Empty; self.cells.len()];
        self.start = None;
        self.goal = None;
        self.cursor = None;
    }

    fn fill(&mut self) {
        todo!()
    }

    fn get_neighborhood_of(&self, row: usize, column: usize) -> super::Neighborhood {
        let mut neighbors = Neighborhood::new();
        let word_row = self.dims.columns * row;
        let word_col = column;
        let index = word_row + word_col;

        neighbors.north = if let Some(kind) = self.cells.get(index + self.dims.columns) {
            Some((*kind, (row + 1, column)))
        } else {
            None
        };

        neighbors.south = if index >= self.dims.columns {
            Some((self.cells[index - self.dims.columns], (row - 1, column)))
        } else {
            None
        };

        neighbors.east = if column < self.dims.columns - 1 {
            Some((self.cells[index + 1], (row, column + 1)))
        } else {
            None
        };

        neighbors.west = if column > 0 {
            Some((self.cells[index - 1], (row, column - 1)))
        } else {
            None
        };

        neighbors
    }

    fn set_neighbor_of(
        &mut self,
        coords: (usize, usize),
        direction: super::Direction,
        kind: CellKind,
    ) -> (usize, usize) {
        let (n_row, n_col) = self.get_neighbor_coords_of(coords, direction);

        self.set_cell(n_row, n_col, kind);

        (n_row, n_col)
    }

    fn dims(&self) -> Dimensions {
        self.dims
    }

    fn solver_kind(&self) -> SolverKind {
        self.solver_kind
    }

    fn reset_solver(&mut self) {
        todo!()
    }
}
