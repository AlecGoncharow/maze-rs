pub struct WallGrid {
    /// Each cell is a bit mask of sorts
    ///
    /// the high 4 bits pertain to the kind of cell, exlored, start, goal, etc...
    /// the lower 4 bits represented the existance of a wall in the WESN directions respectivley
    cells: Vec<u8>,
}
