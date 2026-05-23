use std::sync::Arc;

use korangar_interface::element::StateElement;
use rust_state::RustState;

use crate::graphics::Texture;

/// State of the active map's minimap. Populated when a map finishes loading
/// and cleared on map change.
#[derive(Default, RustState, StateElement)]
pub struct MinimapState {
    #[hidden_element]
    texture: Option<Arc<Texture>>,
    map_width_tiles: u16,
    map_height_tiles: u16,
}

impl MinimapState {
    pub fn set(&mut self, texture: Arc<Texture>, map_width_tiles: u16, map_height_tiles: u16) {
        self.texture = Some(texture);
        self.map_width_tiles = map_width_tiles;
        self.map_height_tiles = map_height_tiles;
    }

    pub fn clear(&mut self) {
        self.texture = None;
        self.map_width_tiles = 0;
        self.map_height_tiles = 0;
    }

    pub fn texture(&self) -> Option<&Arc<Texture>> {
        self.texture.as_ref()
    }

    pub fn map_width_tiles(&self) -> u16 {
        self.map_width_tiles
    }

    pub fn map_height_tiles(&self) -> u16 {
        self.map_height_tiles
    }
}
