/// A cell of the map, `y` grows towards the bottom of the screen
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Move of one cell in this direction
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    /// Angle in radians of this direction on screen (0 = right, clockwise)
    pub fn angle(self) -> f32 {
        use std::f32::consts::{FRAC_PI_2, PI};
        match self {
            Direction::Right => 0.0,
            Direction::Down => FRAC_PI_2,
            Direction::Left => PI,
            Direction::Up => -FRAC_PI_2,
        }
    }
}
