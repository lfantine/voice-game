
use crate::voice;

/// The possible page for the window
#[derive(Debug, Default)]
pub enum AppPage {
    #[default]
    Menu,
    Game,
    /// Record examples for the training
    #[cfg(feature = "train")]
    Record,
}

/// The Message the app can get ( for interaction )
#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Go from the menu to the game page
    Play,
    /// Start / resume the game
    Start,
    /// Pause the game
    Pause,
    /// Leave the game page, back to the menu
    Quit,

    Voice(voice::VoiceInput),

    /// Go from the menu to the record page
    #[cfg(feature = "train")]
    OpenRecord,
    /// Choose the label to record
    #[cfg(feature = "train")]
    SelectLabel(&'static str),
    /// Click on Record : wait a bit so the click is not recorded, then `BeginRecording`
    #[cfg(feature = "train")]
    StartRecording,
    /// The delay after the click is over, open the microphone and record one example
    #[cfg(feature = "train")]
    BeginRecording,
    /// A recording ended, with the saved file name or the error
    #[cfg(feature = "train")]
    Recorded(Result<String, String>),
}

/// The state of the window
#[derive(Debug)]
pub struct WindowState {
    pub page: AppPage,
    pub playing: bool,
    pub inputs: Vec<voice::VoiceInput>,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            page: AppPage::default(),
            playing: true,
            inputs: Vec::new()
        }
    }
}