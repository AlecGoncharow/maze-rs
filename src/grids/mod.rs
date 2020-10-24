#[allow(dead_code)]
pub mod block_grid;

#[allow(dead_code)]
pub mod wall_grid;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub enum GridKind {
    Block,
    Wall,
}

#[derive(Debug, Clone, Copy)]
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

pub type Neighbor = (CellKind, (usize, usize));

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
pub enum CellKind {
    Empty = 0,
    Wall = 1,
    Start = 2,
    Goal = 3,
    Path = 4,
    Explored = 5,
    Cursor = 6,
}

impl From<u8> for CellKind {
    fn from(code: u8) -> Self {
        match code {
            0 => CellKind::Empty,
            1 => CellKind::Wall,
            2 => CellKind::Start,
            3 => CellKind::Goal,
            4 => CellKind::Path,
            5 => CellKind::Explored,
            6 => CellKind::Cursor,
            _ => unreachable!(),
        }
    }
}

impl From<CellKind> for [f32; 4] {
    fn from(kind: CellKind) -> Self {
        match kind {
            CellKind::Empty => [1.0, 1.0, 1.0, 1.0],
            CellKind::Wall => [0.0, 0.0, 0.0, 1.0],
            CellKind::Start => [1.0, 0.0, 0.0, 1.0],
            CellKind::Goal => [1.0, 1.0, 0.0, 1.0],
            CellKind::Explored => [0.2, 0.2, 0.6, 1.0],
            CellKind::Path => [0.1, 0.5, 0.1, 1.0],
            CellKind::Cursor => [0.0, 0.5, 0.3, 1.0],
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
    fn dims(&self) -> Dimensions;
    fn cells(&self) -> &Vec<CellKind>;
    fn set_cells(&mut self, cells: Vec<CellKind>);
    fn solver_kind(&self) -> SolverKind;
    fn set_solver_kind(&mut self, kind: SolverKind);
    fn solve_path(&mut self);
    fn step_solve_path(&mut self) -> bool;
    fn reset_solver(&mut self);
    fn clear(&mut self);
    fn fill(&mut self);
    fn handle_click(
        &mut self,
        pos: (f32, f32),
        size: winit::dpi::PhysicalSize<u32>,
        kind: CellKind,
    );
    fn get_neighborhood_of(&self, row: usize, column: usize) -> Neighborhood;
    fn set_neighbor_of(
        &mut self,
        coords: (usize, usize),
        direction: Direction,
        kind: CellKind,
    ) -> (usize, usize);
    fn set_cell(&mut self, row: usize, column: usize, kind: CellKind) -> CellKind;
    fn get_cell(&self, row: usize, column: usize) -> CellKind;
    fn paths(&self) -> Vec<(usize, usize)>;
    fn set_paths(&mut self, paths: Vec<(usize, usize)>);
    fn add_wall_between(&mut self, one: (usize, usize), two: (usize, usize));
    fn clear_wall_between(&mut self, one: (usize, usize), two: (usize, usize));
}
