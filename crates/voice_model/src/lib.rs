//! Voice command recognition : audio samples -> `Option<Command>`
//!
//! The only contract with the audio side : mono `f32` samples at `SAMPLE_RATE`.

mod features;
mod inference;
mod model;
#[cfg(feature = "train")]
mod train;

pub use features::FeatureExtractor;
pub use inference::{ModelError, Recognizer};
pub use model::{VoiceModel, VoiceModelConfig};
#[cfg(feature = "train")]
pub use train::{VoiceBatch, VoiceBatcher, VoiceItem};

/// Sample rate expected by the model
pub const SAMPLE_RATE: u32 = 16_000;
/// Number of samples per prediction (1 s of audio)
pub const WINDOW_LEN: usize = 16_000;

/// Classes predicted by the model, the index in this array is the class id.
/// It is also the name of the folder holding the examples of each class.
pub const LABELS: [&str; NUM_CLASSES] = ["up", "down", "left", "right", "noise"];
pub const NUM_CLASSES: usize = 5;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Command {
    Up,
    Down,
    Left,
    Right,
}

impl Command {
    /// The command matching a class id, `None` for the noise class
    pub fn from_class(class: usize) -> Option<Self> {
        match class {
            0 => Some(Command::Up),
            1 => Some(Command::Down),
            2 => Some(Command::Left),
            3 => Some(Command::Right),
            _ => None,
        }
    }
}

/// The class id of a label of `LABELS`
pub fn class_of(label: &str) -> Option<usize> {
    LABELS.iter().position(|&l| l == label)
}
