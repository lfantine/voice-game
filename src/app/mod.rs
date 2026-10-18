use iced::widget::{button, canvas, center, column, container, row, scrollable, text};
use iced::{Center, Element, Fill, Subscription, Task};

mod game;
mod game_canvas;
mod window;

use game::Game;
use game_canvas::GameCanvas;
use window::{AppMessage, AppPage, WindowState};

use crate::voice;

/// Width of the left column on the game page
const SIDEBAR_WIDTH: f32 = 200.0;

// The main struct of the app, owned by iced
#[derive(Debug, Default)]
pub struct App {
    game: Game,
    window: WindowState,
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
        }
    }

    fn view(&self) -> Element<'_, AppMessage> {
        match self.window.page {
            AppPage::Menu => self.view_menu(),
            AppPage::Game => self.view_game(),
        }
    }

    /// Home page : title and play button
    fn view_menu(&self) -> Element<'_, AppMessage> {
        center(
            column![
                text("Voice Recognition Game").size(40),
                button(text("Play").size(20))
                    .padding([10, 40])
                    .on_press(AppMessage::Play),
            ]
            .spacing(30)
            .align_x(Center),
        )
        .into()
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

        let sidebar = container(column![controls, inputs].spacing(10))
            .width(SIDEBAR_WIDTH)
            .height(Fill)
            .padding(10)
            .style(container::bordered_box);

        let game = canvas(GameCanvas { game: &self.game })
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
