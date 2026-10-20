use iced::advanced::image::Image;
use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke};
use iced::widget::image::FilterMethod;
use iced::{Color, Point, Rectangle, Renderer, Size, Theme, Vector, border, mouse};

use super::game::{Game, Tile};
use super::textures::Textures;
use super::window::AppMessage;

/// Space kept between the map and the border of the canvas
const MARGIN: f32 = 20.0;

const SKIN: Color = Color::from_rgb(0.96, 0.78, 0.62);
const HAIR: Color = Color::from_rgb(0.28, 0.17, 0.10);
const SHIRT: Color = Color::from_rgb(0.20, 0.45, 0.85);

/// Draw the game in the main area of the game page
pub struct GameCanvas<'a> {
    pub game: &'a Game,
    pub textures: &'a Textures,
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
        let mut frame = Frame::new(renderer, bounds.size());
        let palette = theme.extended_palette();

        // background, the map is drawn on top of it
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), palette.background.weak.color);

        // the map is centered in the canvas, its cells are squares
        let map = self.game.get_map();
        let cell = ((bounds.width - 2.0 * MARGIN) / map.width() as f32)
            .min((bounds.height - 2.0 * MARGIN) / map.height() as f32)
            .max(0.0);
        let size = Size::new(cell * map.width() as f32, cell * map.height() as f32);
        let origin = Point::new((bounds.width - size.width) / 2.0, (bounds.height - size.height) / 2.0);

        for y in 0..map.height() {
            for x in 0..map.width() {
                let top_left = Point::new(origin.x + x as f32 * cell, origin.y + y as f32 * cell);
                let area = Rectangle::new(top_left, Size::new(cell, cell));
                match map.tile(x, y) {
                    Tile::Floor => self.draw_tile(&mut frame, area, Tile::Floor),
                    Tile::Wall => self.draw_tile(&mut frame, area, Tile::Wall),
                    Tile::Diamond => {
                        // the diamond lies on the floor
                        self.draw_tile(&mut frame, area, Tile::Floor);
                        self.draw_tile(&mut frame, area, Tile::Diamond);
                    }
                }
            }
        }

        // the player, drawn in the middle of its cell
        let player = self.game.get_player();
        let position = player.get_position();
        let center = Point::new(
            origin.x + (position.x as f32 + 0.5) * cell,
            origin.y + (position.y as f32 + 0.5) * cell,
        );
        draw_character(&mut frame, center, cell, player.get_facing().angle());

        vec![frame.into_geometry()]
    }
}

impl GameCanvas<'_> {
    /// Fill `area` with the texture of `tile`, or with a plain color when the texture file is missing
    fn draw_tile(&self, frame: &mut Frame, area: Rectangle, tile: Tile) {
        if let Some(handle) = self.textures.get(tile) {
            // the textures are tiny pixel arts, no smoothing
            frame.draw_image(area, Image::new(handle.clone()).filter_method(FilterMethod::Nearest));
            return;
        }

        let color = match tile {
            Tile::Floor => Color::from_rgb(0.36, 0.55, 0.24),
            Tile::Wall => Color::from_rgb(0.50, 0.50, 0.50),
            Tile::Diamond => Color::from_rgb(0.36, 0.93, 0.89),
        };
        match tile {
            // without texture the diamond is a small square on top of the floor
            Tile::Diamond => {
                let inset = area.width * 0.25;
                frame.fill_rectangle(
                    Point::new(area.x + inset, area.y + inset),
                    Size::new(area.width - 2.0 * inset, area.height - 2.0 * inset),
                    color,
                );
            }
            _ => {
                frame.fill_rectangle(area.position(), area.size(), color);
                frame.stroke_rectangle(
                    area.position(),
                    area.size(),
                    Stroke::default().with_width(1.0).with_color(Color { a: 0.25, ..Color::BLACK }),
                );
            }
        }
    }
}

/// A character seen from above, `cell` is the size of its cell and `angle` the direction it looks at
/// (0 = right). It is drawn looking right then rotated.
fn draw_character(frame: &mut Frame, center: Point, cell: f32, angle: f32) {
    frame.with_save(|frame| {
        frame.translate(Vector::new(center.x, center.y));
        frame.rotate(angle);

        // shoulders, across the looking direction
        let shoulders = Path::rounded_rectangle(
            Point::new(-0.15 * cell, -0.38 * cell),
            Size::new(0.30 * cell, 0.76 * cell),
            border::Radius::from(0.15 * cell),
        );
        frame.fill(&shoulders, SHIRT);

        // hands
        for side in [-1.0, 1.0] {
            frame.fill(&Path::circle(Point::new(0.10 * cell, side * 0.38 * cell), 0.08 * cell), SKIN);
        }

        // head, hair on the back and nose in front
        frame.fill(&Path::circle(Point::new(0.02 * cell, 0.0), 0.21 * cell), SKIN);
        frame.fill(&Path::circle(Point::new(-0.03 * cell, 0.0), 0.19 * cell), HAIR);
        frame.fill(&Path::circle(Point::new(0.22 * cell, 0.0), 0.05 * cell), SKIN);
    });
}
