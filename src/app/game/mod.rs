use std::fmt;
use crate::voice::VoiceInput;

mod math;
mod player;

use player::Player;
use tracing::warn;


/// The game logic, knows nothing about the window
#[derive(Debug, Default)]
pub struct Game {
    player : Player,
}

impl Game {
    
    pub fn get_player(&self) -> &Player {
        return &self.player;
    }
    
    pub fn handle_input(&mut self, input: VoiceInput) {
        tracing::debug!("input reçu : {input:?}");
        match input {
            VoiceInput::Up => {
                self.player.move_up();
            },
            VoiceInput::Down => {
                self.player.move_down();
            },
            VoiceInput::Left => {
                self.player.move_left();
            },
            VoiceInput::Right => {
                self.player.move_right();
            },
            _ => {
                warn!("Input not handled");
            }
        }
    }
}

impl fmt::Display for Game {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
