pub mod aldous_broder;
pub mod division;
pub mod prim;
use crate::grids::Grid;

pub trait Generator {
    fn step_generation(&mut self);
    fn next_step(&mut self) -> &dyn Grid;
    fn generate_maze(&mut self) -> &dyn Grid;
    fn is_done(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GeneratorKind {
    AldousBroder,
    RandPrims,
}
