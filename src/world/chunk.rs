use bevy::prelude::*;
use super::tile::TileType;

pub const CHUNK_SIZE: usize = 32;

#[derive(Component)]
pub struct Chunk {
    pub position: IVec2,
    pub tiles: [[TileType; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
    pub fn new(position: IVec2) -> Self {
        Self {
            position,
            tiles: [[TileType::Grass; CHUNK_SIZE]; CHUNK_SIZE],
        }
    }

    pub fn get_tile(&self, x: usize, y: usize) -> TileType {
        self.tiles[y][x]
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: TileType) {
        self.tiles[y][x] = tile;
    }
}
