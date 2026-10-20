use iced::widget::{button, canvas, center, column, container, row, scrollable, text};
use iced::{Center, Element, Fill, Subscription, Task};

mod game;
mod game_canvas;
#[cfg(feature = "train")]
mod record;
mod textures;
mod window;

use game::{Game, LEVEL_COUNT};
use game_canvas::GameCanvas;
use textures::Textures;
use window::{AppMessage, AppPage, WindowState};

use crate::voice;
#[cfg(feature = "train")]
use crate::train;

/// Width of the left column on the game page
const SIDEBAR_WIDTH: f32 = 200.0;

/// Time between the click on Record and the start of the recording
#[cfg(feature = "train")]
const RECORD_DELAY: std::time::Duration = std::time::Duration::from_millis(500);

// The main struct of the app, owned by iced
#[derive(Debug, Default)]
pub struct App {
    game: Game,
    textures: Textures,
    window: WindowState,
    #[cfg(feature = "train")]
    record: record::RecordState,
}

impl App {
    /// Open the window, blocks until it is closed
    pub fn launch() -> iced::Result {
        let window = iced::window::Settings {
            size: iced::Size::new(800.0, 600.0),
            resizable: false,
            position: iced::window::Position::Centered,
            // min_size: Some(iced::Size::new(600.0, 400.0)), // par exemple
            ..Default::default()
        };

        iced::application(App::default, App::update, App::view)
            .subscription(App::subscription)
            .window(window)
            .run()
    }

    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::Play => {
                self.game = Game::default();
                self.window.page = AppPage::Game;
                self.window.playing = true;
                self.window.inputs.clear();
                Task::none()
            }
            AppMessage::Start => {
                self.window.playing = true;
                Task::none()
            }
            AppMessage::Pause => {
                self.window.playing = false;
                Task::none()
            }
            AppMessage::Quit => {
                self.window.page = AppPage::Menu;
                self.window.playing = false;
                Task::none()
            }
            AppMessage::Voice(input) => {
                self.window.inputs.push(input);
                self.game.handle_input(input);
                Task::none()
            }
            #[cfg(feature = "train")]
            AppMessage::OpenRecord => {
                self.record.refresh();
                self.window.page = AppPage::Record;
                Task::none()
            }
            #[cfg(feature = "train")]
            AppMessage::SelectLabel(label) => {
                self.record.label = label;
                Task::none()
            }
            #[cfg(feature = "train")]
            AppMessage::StartRecording => {
                if self.record.recording {
                    return Task::none();
                }
                self.record.start();
                // the microphone only opens after this delay, so the sound of the click is not recorded
                Task::perform(
                    tokio::time::sleep(RECORD_DELAY),
                    |_| AppMessage::BeginRecording,
                )
            }
            #[cfg(feature = "train")]
            AppMessage::BeginRecording => {
                self.record.listening();
                let label = self.record.label;
                // the microphone is opened and closed on the blocking thread
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            train::record_sample(label)
                                .map(|path| path.display().to_string())
                                .map_err(|e| e.to_string())
                        })
                        .await
                        .unwrap_or_else(|e| Err(e.to_string()))
                    },
                    AppMessage::Recorded,
                )
            }
            #[cfg(feature = "train")]
            AppMessage::Recorded(result) => {
                self.record.finish(result);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, AppMessage> {
        match self.window.page {
            AppPage::Menu => self.view_menu(),
            AppPage::Game => self.view_game(),
            #[cfg(feature = "train")]
            AppPage::Record => self.record.view(),
        }
    }

    /// Home page : title, play button and record button
    fn view_menu(&self) -> Element<'_, AppMessage> {
        let menu = column![
            text("Voice Recognition Game").size(40),
            button(text("Play").size(20))
                .padding([10, 40])
                .on_press(AppMessage::Play),
        ];
        #[cfg(feature = "train")]
        let menu = menu.push(
            button(text("Record examples").size(20))
                .padding([10, 40])
                .on_press(AppMessage::OpenRecord),
        );

        center(menu.spacing(30).align_x(Center)).into()
    }

    /// Game page : controls and inputs on the left, game canvas on the right
    fn view_game(&self) -> Element<'_, AppMessage> {
        let playing = self.window.playing;

        let controls = row![
            control_button("Play", (!playing).then_some(AppMessage::Start)),
            control_button("Pause", playing.then_some(AppMessage::Pause)),
            control_button("Quit", Some(AppMessage::Quit)),
        ]
        .spacing(5);

        // newest inputs at the bottom, the list stays anchored to the bottom
        let inputs = scrollable(
            column(
                self.window
                    .inputs
                    .iter()
                    .map(|input| text(format!("{input:?}")).into()),
            )
            .spacing(4)
            .width(Fill),
        )
        .anchor_bottom()
        .height(Fill);

        let level = text(format!("Level {}/{}", self.game.level_number(), LEVEL_COUNT)).size(20);

        let sidebar = container(column![level, controls, inputs].spacing(10))
            .width(SIDEBAR_WIDTH)
            .height(Fill)
            .padding(10)
            .style(container::bordered_box);

        let game = canvas(GameCanvas { game: &self.game, textures: &self.textures })
            .width(Fill)
            .height(Fill);

        row![sidebar, game].into()
    }

    fn subscription(&self) -> Subscription<AppMessage> {
        match self.window.page {
            AppPage::Game if self.window.playing => {
                Subscription::run(voice::listen).map(AppMessage::Voice)
            }
            _ => Subscription::none(),
        }
    }
}

/// A button of the game controls, disabled when `message` is `None`
fn control_button(label: &str, message: Option<AppMessage>) -> Element<'_, AppMessage> {
    button(text(label).width(Fill).align_x(Center))
        .width(Fill)
        .on_press_maybe(message)
        .into()
}
