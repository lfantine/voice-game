use super::math::{Direction, Position};

#[derive(Debug)]
pub struct Player {
    position: Position,
    facing: Direction,
}

impl Player {
    /// A player on `position`, looking down
    pub fn new(position: Position) -> Self {
        Self {
            position,
            facing: Direction::Down,
        }
    }

    pub fn get_position(&self) -> Position {
        self.position
    }

    pub fn get_facing(&self) -> Direction {
        self.facing
    }

    pub fn face(&mut self, direction: Direction) {
        self.facing = direction;
    }

    pub fn move_to(&mut self, position: Position) {
        self.position = position;
    }
}
