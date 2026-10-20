//! Minecraft block textures used to draw the map.
//! The PNG files are not part of the project : copy them in `textures_dir()`, any missing one
//! is replaced by a plain color.

use std::path::PathBuf;

use iced::widget::image::Handle;
use tracing::warn;

use super::game::Tile;

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

#[derive(Debug)]
pub struct Textures {
    loaded: Vec<(Tile, Handle)>,
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
        Self { loaded }
    }
}

impl Textures {
    /// The texture of `tile`, `None` if its file is missing
    pub fn get(&self, tile: Tile) -> Option<&Handle> {
        self.loaded.iter().find(|(t, _)| *t == tile).map(|(_, handle)| handle)
    }
}
