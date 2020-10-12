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

