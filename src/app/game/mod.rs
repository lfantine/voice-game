use std::fmt;
use crate::voice::VoiceInput;

mod map;
mod math;
mod player;

pub use map::{LEVEL_COUNT, Map, Tile};
pub use math::Direction;
use math::Position;
use player::Player;

/// The game logic, knows nothing about the window
#[derive(Debug)]
pub struct Game {
    /// Index of the current level, from 0
    level: usize,
    map: Map,
    player: Player,
}

impl Default for Game {
    fn default() -> Self {
        Self::at_level(0)
    }
}

impl Game {
    fn at_level(level: usize) -> Self {
        let map = Map::level(level);
        let player = Player::new(map.start());
        Self { level, map, player }
    }

    pub fn get_player(&self) -> &Player {
        &self.player
    }

    pub fn get_map(&self) -> &Map {
        &self.map
    }

    /// Number of the current level, from 1
    pub fn level_number(&self) -> usize {
        self.level + 1
    }

    /// Turn the player towards the input and walk one tile if it is free.
    /// Taking the diamond starts the next level, the first one comes back after the last.
    pub fn handle_input(&mut self, input: VoiceInput) {
        tracing::debug!("input reçu : {input:?}");
        let direction = match input {
            VoiceInput::Up => Direction::Up,
            VoiceInput::Down => Direction::Down,
            VoiceInput::Left => Direction::Left,
            VoiceInput::Right => Direction::Right,
        };
        self.player.face(direction);

        let (dx, dy) = direction.delta();
        let Position { x, y } = self.player.get_position();
        let target = Position { x: x + dx, y: y + dy };
        if !self.map.is_walkable(target.x, target.y) {
            return;
        }
        self.player.move_to(target);

        if self.map.tile(target.x, target.y) == Tile::Diamond {
            *self = Self::at_level((self.level + 1) % LEVEL_COUNT);
        }
    }
}

impl fmt::Display for Game {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
