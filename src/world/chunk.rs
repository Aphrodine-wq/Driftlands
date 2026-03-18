use bevy::prelude::*;
use super::tile::TileType;
use super::generation::Biome;

pub const CHUNK_SIZE: usize = 32;

#[derive(Component)]
pub struct Chunk {
    pub position: IVec2,
    pub tiles: [[TileType; CHUNK_SIZE]; CHUNK_SIZE],
    pub biome: Biome,
}

impl Chunk {
    pub fn new(position: IVec2) -> Self {
        Self {
            position,
            tiles: [[TileType::Grass; CHUNK_SIZE]; CHUNK_SIZE],
            biome: Biome::Forest,
        }
    }

    /// Build a chunk from saved tile data (e.g. from LoadedChunkCache). `tiles` is row-major [y][x].
    pub fn from_tiles(position: IVec2, tiles: &[Vec<TileType>], biome: Biome) -> Self {
        let mut chunk = Self::new(position);
        chunk.biome = biome;
        for (y, row) in tiles.iter().take(CHUNK_SIZE).enumerate() {
            for (x, &tile) in row.iter().take(CHUNK_SIZE).enumerate() {
                chunk.tiles[y][x] = tile;
            }
        }
        chunk
    }

    pub fn get_tile(&self, x: usize, y: usize) -> TileType {
        self.tiles[y][x]
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: TileType) {
        self.tiles[y][x] = tile;
    }
}
