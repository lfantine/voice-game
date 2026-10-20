//! Minecraft block textures used to draw the map.
//! The PNG files are not part of the project : copy them in `textures_dir()`, any missing one
//! is replaced by a plain color.

use std::path::PathBuf;

use iced::widget::image::Handle;
use image::imageops::{self, overlay};
use image::RgbaImage;
use tracing::warn;

use super::game::{Direction, Tile};

/// Folder holding the textures, `assets/textures` by default
fn textures_dir() -> PathBuf {
    std::env::var("TEXTURES_DIR").unwrap_or_else(|_| "assets/textures".to_string()).into()
}

/// File of each tile, from `assets/minecraft/textures/block/` of the game
const FILES: [(Tile, &str); 3] = [
    (Tile::Floor, "moss_block.png"),
    (Tile::Wall, "cobblestone.png"),
    (Tile::Diamond, "diamond_ore.png"),
];

/// Skin of the player, a standard 64x64 Minecraft skin
const PLAYER_FILE: &str = "player.png";

#[derive(Debug)]
pub struct Textures {
    loaded: Vec<(Tile, Handle)>,
    /// The player seen from above, looking down, left, up and right
    player: Option<[Handle; 4]>,
}

impl Default for Textures {
    fn default() -> Self {
        let dir = textures_dir();
        let mut loaded = Vec::new();
        for (tile, file) in FILES {
            let path = dir.join(file);
            if path.exists() {
                loaded.push((tile, Handle::from_path(path)));
            } else {
                warn!("Texture {path:?} not found, {tile:?} is drawn with a plain color");
            }
        }

        let player_path = dir.join(PLAYER_FILE);
        let player = if player_path.exists() {
            match player_sprites(&player_path) {
                Ok(sprites) => Some(sprites),
                Err(e) => {
                    warn!("Can't use the skin {player_path:?} ({e}), the player is drawn with shapes");
                    None
                }
            }
        } else {
            warn!("Skin {player_path:?} not found, the player is drawn with shapes");
            None
        };
        Self { loaded, player }
    }
}

impl Textures {
    /// The texture of `tile`, `None` if its file is missing
    pub fn get(&self, tile: Tile) -> Option<&Handle> {
        self.loaded.iter().find(|(t, _)| *t == tile).map(|(_, handle)| handle)
    }

    /// The player seen from above looking towards `direction`, `None` if there is no skin
    pub fn player(&self, direction: Direction) -> Option<&Handle> {
        let index = match direction {
            Direction::Down => 0,
            Direction::Left => 1,
            Direction::Up => 2,
            Direction::Right => 3,
        };
        self.player.as_ref().map(|sprites| &sprites[index])
    }
}

/// Build the top view of a skin : the head over the shoulders and the arms, for each direction.
/// The sprite is 16x16 pixels and looks down (the front of the head is at the bottom of its top face).
fn player_sprites(path: &std::path::Path) -> Result<[Handle; 4], String> {
    let skin = image::open(path).map_err(|e| e.to_string())?.to_rgba8();
    if skin.width() != 64 || !matches!(skin.height(), 32 | 64) {
        return Err(format!("expected a 64x64 skin, got {}x{}", skin.width(), skin.height()));
    }

    let part = |x, y, w, h| imageops::crop_imm(&skin, x, y, w, h).to_image();
    let right_arm = part(44, 16, 4, 4);
    // old 64x32 skins have no left arm, it is the right one
    let left_arm = if skin.height() == 64 { part(36, 48, 4, 4) } else { right_arm.clone() };

    let mut down = RgbaImage::new(16, 16);
    overlay(&mut down, &right_arm, 0, 6);
    overlay(&mut down, &part(20, 16, 8, 4), 4, 6);
    overlay(&mut down, &left_arm, 12, 6);
    overlay(&mut down, &part(8, 0, 8, 8), 4, 4);
    // the hat layer, hair and helmets
    overlay(&mut down, &part(40, 0, 8, 8), 4, 4);

    // rotating clockwise turns "looking down" into "looking left", then up, then right
    let sprites = [
        down.clone(),
        imageops::rotate90(&down),
        imageops::rotate180(&down),
        imageops::rotate270(&down),
    ];
    Ok(sprites.map(|sprite| Handle::from_rgba(16, 16, sprite.into_raw())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_the_player_from_a_skin() {
        let dir = std::env::temp_dir().join("voice_game_skin_test");
        std::fs::create_dir_all(&dir).unwrap();

        for height in [64, 32] {
            let path = dir.join(format!("skin{height}.png"));
            RgbaImage::from_pixel(64, height, image::Rgba([200, 100, 50, 255])).save(&path).unwrap();
            assert!(player_sprites(&path).is_ok(), "a 64x{height} skin should be accepted");
        }

        let path = dir.join("bad.png");
        RgbaImage::new(30, 30).save(&path).unwrap();
        assert!(player_sprites(&path).is_err());
    }
}
