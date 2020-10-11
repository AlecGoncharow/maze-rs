pub mod aldous_broder;
pub mod division;
pub mod prim;
use crate::GridKind;

pub trait Generator {
    fn step_generation(&mut self);
    fn next_step(&mut self) -> Vec<GridKind>;
    fn generate_maze(&mut self) -> Vec<GridKind>;
    fn is_done(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GeneratorKind {
    AldousBroder,
    RandPrims,
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

                    if let Some(west) = self.west {
                        return Some((west, Direction::West));
                    } else {
                        return None;
                    }
                }
                Direction::Sentinel => {
                    self.counter = Direction::North;

                    return None;
                }
            }
        }
    }
}
