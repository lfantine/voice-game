//! The levels : text maps turned into a grid of tiles

use super::math::Position;

/// Number of levels of the game
pub const LEVEL_COUNT: usize = 3;

/// The levels, one row of text per row of cells :
/// `#` wall, `.` floor, `P` where the player starts, `D` the diamond
const LEVELS: [&str; LEVEL_COUNT] = [
    "
############
#P.........#
#..........#
#..####....#
#..........#
#......#...#
#......#.D.#
#..........#
#..........#
############
",
    "
############
#P...#.....#
#.##.#.###.#
#.#..#...#.#
#.#.####.#.#
#.#......#.#
#.######.#.#
#......#.#.#
#.####...#D#
############
",
    "
############
#P.#.....#D#
#..#.###.#.#
#.##.#...#.#
#....#.###.#
#.####.....#
#......###.#
#.####.#...#
#..........#
############
",
];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Tile {
    Floor,
    Wall,
    /// Reaching it ends the level
    Diamond,
}

#[derive(Debug)]
pub struct Map {
    tiles: Vec<Tile>,
    width: i32,
    height: i32,
    start: Position,
}

impl Map {
    /// The map of the level `index` (0-based)
    pub fn level(index: usize) -> Self {
        Self::parse(LEVELS[index])
    }

    /// Build a map from its text, the levels are constants so a bad one is a bug
    fn parse(text: &str) -> Self {
        let rows: Vec<&str> = text.lines().filter(|row| !row.is_empty()).collect();
        let height = rows.len() as i32;
        let width = rows[0].len() as i32;
        let mut tiles = Vec::new();
        let mut start = None;

        for (y, row) in rows.iter().enumerate() {
            assert_eq!(row.len() as i32, width, "row {y} of a level has a different width");
            for (x, symbol) in row.chars().enumerate() {
                tiles.push(match symbol {
                    '#' => Tile::Wall,
                    '.' => Tile::Floor,
                    'D' => Tile::Diamond,
                    'P' => {
                        start = Some(Position { x: x as i32, y: y as i32 });
                        Tile::Floor
                    }
                    other => panic!("unknown symbol '{other}' in a level"),
                });
            }
        }

        Self {
            tiles,
            width,
            height,
            start: start.expect("a level needs a start 'P'"),
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn start(&self) -> Position {
        self.start
    }

    /// The tile at (`x`, `y`), outside of the map it's a wall
    pub fn tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return Tile::Wall;
        }
        self.tiles[(y * self.width + x) as usize]
    }

    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        self.tile(x, y) != Tile::Wall
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Number of cells of the map that can be reached from the start, and if the diamond is one of them
    fn explore(map: &Map) -> bool {
        let mut seen = vec![false; (map.width * map.height) as usize];
        let mut stack = vec![(map.start.x, map.start.y)];
        while let Some((x, y)) = stack.pop() {
            if !map.is_walkable(x, y) || seen[(y * map.width + x) as usize] {
                continue;
            }
            seen[(y * map.width + x) as usize] = true;
            if map.tile(x, y) == Tile::Diamond {
                return true;
            }
            stack.extend([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]);
        }
        false
    }

    #[test]
    fn every_level_can_be_finished() {
        for index in 0..LEVEL_COUNT {
            assert!(explore(&Map::level(index)), "the diamond of level {} can't be reached", index + 1);
        }
    }

    #[test]
    fn every_level_has_one_diamond() {
        for index in 0..LEVEL_COUNT {
            let map = Map::level(index);
            let diamonds = (0..map.height)
                .flat_map(|y| (0..map.width).map(move |x| (x, y)))
                .filter(|&(x, y)| map.tile(x, y) == Tile::Diamond)
                .count();
            assert_eq!(diamonds, 1, "level {} needs exactly one diamond", index + 1);
        }
    }
}
