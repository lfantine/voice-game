
use crate::voice;

/// The possible page for the window
#[derive(Debug, Default)]
pub enum AppPage {
    #[default]
    Menu,
    Game,
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