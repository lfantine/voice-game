use iced::widget::canvas::{self, Frame, Geometry, Path};
use iced::{Color, Point, Rectangle, Renderer, Theme, mouse};

use super::game::Game;
use super::window::AppMessage;

/// Draw the game in the main area of the game page
pub struct GameCanvas<'a> {
    pub game: &'a Game,
}

impl canvas::Program<AppMessage> for GameCanvas<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {

        // nouvelle frame en cours 
        let mut frame = Frame::new(renderer, bounds.size());
        
        // background, the game will be drawn on top of it
        let background = theme.extended_palette().background.weak.color;
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), background);

        // getting the player data
        let player_data = self.game.get_player();

        // joueur 
        let center = player_data.get_position();
        let center_point : Point<f32> = Point::new(center.x as f32, center.y as f32);
        let player = Path::circle(center_point, 2.0);

        frame.fill(&player, Color::BLACK);
        

        // apply de la nouvelle frame
        vec![frame.into_geometry()]
    }
}
