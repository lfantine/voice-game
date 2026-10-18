use iced::futures::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc::{Sender, error::TrySendError};

#[derive(Debug, Copy, Clone)]
pub enum VoiceInput {
    Up,
    Left,
    Right,
    Down
}

/// This is the function called by iced to start the listening subscription
pub fn listen() -> impl Stream<Item = VoiceInput> {
    // creting channel 
    let (tx, rx) = tokio::sync::mpsc::channel(10); // un unbounded_channel fonctionnerai mais je veux test avec des bounds
    let _ = std::thread::spawn(move || recognition(tx));
    ReceiverStream::new(rx)
}

fn recognition(tx: Sender<VoiceInput>) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        match tx.try_send(VoiceInput::Right) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {}       // channel plein → on jette, pas grave
            Err(TrySendError::Closed(_)) => break, // iced a arrêté la subscription → on sort
        }
    }
}