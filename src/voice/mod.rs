use std::path::PathBuf;

use burn::backend::NdArray;
use iced::futures::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc::{Sender, error::TrySendError};
use tracing::{debug, error};
use voice_audio::Microphone;
use voice_model::{Command, Recognizer};

/// Weights produced by `--train`, read by the game
pub fn weights_path() -> PathBuf {
    std::env::var("WEIGHTS_PATH")
        .unwrap_or_else(|_| "artifacts/model.mpk".to_string())
        .into()
}

#[derive(Debug, Copy, Clone)]
pub enum VoiceInput {
    Up,
    Left,
    Right,
    Down
}

impl From<Command> for VoiceInput {
    fn from(command: Command) -> Self {
        match command {
            Command::Up => VoiceInput::Up,
            Command::Down => VoiceInput::Down,
            Command::Left => VoiceInput::Left,
            Command::Right => VoiceInput::Right,
        }
    }
}

/// This is the function called by iced to start the listening subscription
pub fn listen() -> impl Stream<Item = VoiceInput> {
    // creting channel
    let (tx, rx) = tokio::sync::mpsc::channel(10); // un unbounded_channel fonctionnerai mais je veux test avec des bounds
    let _ = std::thread::spawn(move || recognition(tx));
    ReceiverStream::new(rx)
}

fn recognition(tx: Sender<VoiceInput>) {
    let recognizer = match Recognizer::<NdArray>::load(&weights_path(), Default::default()) {
        Ok(recognizer) => recognizer,
        Err(e) => {
            error!("Can't load the model from {:?} (run with --train first ?): {e}", weights_path());
            return;
        }
    };
    let mut mic = match Microphone::open(voice_model::SAMPLE_RATE) {
        Ok(mic) => mic,
        Err(e) => {
            error!("Can't open the microphone: {e}");
            return;
        }
    };

    loop {
        let chunk = match mic.next_chunk(voice_model::WINDOW_LEN) {
            Ok(chunk) => chunk,
            Err(e) => {
                error!("Microphone stopped: {e}");
                break;
            }
        };
        let Some(command) = recognizer.predict(&chunk) else {
            // rien reconnu, mais on vérifie quand même que iced écoute encore
            if tx.is_closed() {
                break;
            }
            continue;
        };
        debug!("Recognized {command:?}");

        match tx.try_send(command.into()) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {}       // channel plein → on jette, pas grave
            Err(TrySendError::Closed(_)) => break, // iced a arrêté la subscription → on sort
        }
    }
}
