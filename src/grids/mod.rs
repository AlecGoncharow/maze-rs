#[allow(dead_code)]
pub mod block_grid;

#[allow(dead_code)]
pub mod wall_grid;

pub enum RealGridKind {
    Block,
    Wall,
}

pub struct Dimensions {
    pub rows: usize,
    pub columns: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    North = 0,
    South = 1,
    East = 2,
    West = 3,

    Sentinel = 255,
}

impl std::ops::Neg for Direction {
    type Output = Direction;

    fn neg(self) -> Self::Output {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            Direction::Sentinel => Direction::Sentinel,
        }
    }
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

pub type Neighbor = (GridKind, (usize, usize));

#[derive(Debug, Clone, Copy)]
pub struct Neighborhood {
    pub north: Option<Neighbor>,
    pub south: Option<Neighbor>,
    pub east: Option<Neighbor>,
    pub west: Option<Neighbor>,

    counter: Direction,
}

impl Neighborhood {
    pub fn new() -> Self {
        Self {
            north: None,
            south: None,
            east: None,
            west: None,
            counter: Direction::North,
        }
    }
}

impl Iterator for Neighborhood {
    type Item = (Neighbor, Direction);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.counter {
                Direction::North => {
                    self.counter = Direction::South;
                    if let Some(north) = self.north {
                        return Some((north, Direction::North));
                    }
                }
                Direction::South => {
                    self.counter = Direction::East;
                    if let Some(south) = self.south {
                        return Some((south, Direction::South));
                    }
                }
                Direction::East => {
                    self.counter = Direction::West;
                    if let Some(east) = self.east {
                        return Some((east, Direction::East));
                    }
                }
                Direction::West => {
                    // reset counter
                    self.counter = Direction::Sentinel;

                    return if let Some(west) = self.west {
                        Some((west, Direction::West))
                    } else {
                        None
                    };
                }
                Direction::Sentinel => {
                    self.counter = Direction::North;

                    return None;
                }
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
// @TODO @FIXME should be renamed to CellKind
pub enum GridKind {
    Empty = 0,
    Wall = 1,
    Start = 2,
    Goal = 3,
    Path = 4,
    Explored = 5,
    Cursor = 6,
}

impl From<u8> for GridKind {
    fn from(code: u8) -> Self {
        match code {
            0 => GridKind::Empty,
            1 => GridKind::Wall,
            2 => GridKind::Start,
            3 => GridKind::Goal,
            4 => GridKind::Path,
            5 => GridKind::Explored,
            6 => GridKind::Cursor,
            _ => unreachable!(),
        }
    }
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

pub trait Grid {
    fn render(&self, state: &crate::State) -> Vec<crate::renderer::Vertex>;
    fn cells(&self) -> &Vec<GridKind>;
    fn set_cells(&mut self, cells: Vec<GridKind>);
    fn set_solver_kind(&mut self, kind: SolverKind);
    fn solve_path(&mut self);
    fn step_solve_path(&mut self);
    fn clear(&mut self);
    fn fill(&mut self);
    fn handle_click(
        &mut self,
        pos: (f32, f32),
        size: winit::dpi::PhysicalSize<u32>,
        kind: GridKind,
    );
    fn get_neighborhood_of(&self, row: usize, column: usize) -> Neighborhood;
    fn set_neighbor_of(
        &mut self,
        coords: (usize, usize),
        direction: Direction,
        kind: GridKind,
    ) -> (usize, usize);
    fn set_cell(&mut self, row: usize, column: usize, kind: GridKind) -> GridKind;
}
