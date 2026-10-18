use super::math::Position;

#[derive(Debug)]
pub struct Player {
    position: Position,
}

impl Player {
    pub fn get_position(&self) -> &Position {
        return &self.position;
    }
    
    pub fn move_up(&mut self) {
        self.position.y -= 1;
    }

    pub fn move_down(&mut self) {
        self.position.y += 1;
    }

    pub fn move_left(&mut self) {
        self.position.x -= 1;
    }

    pub fn move_right(&mut self) {
        self.position.x += 1;
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            position: Position::default(),
        }
    }
}
