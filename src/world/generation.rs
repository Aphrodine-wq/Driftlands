use noise::{NoiseFn, Perlin};
use super::chunk::{Chunk, CHUNK_SIZE};
use super::tile::TileType;

pub struct WorldGenerator {
    elevation: Perlin,
    moisture: Perlin,
    detail: Perlin,
}

impl WorldGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            elevation: Perlin::new(seed),
            moisture: Perlin::new(seed.wrapping_add(1)),
            detail: Perlin::new(seed.wrapping_add(2)),
        }
    }

    pub fn generate_chunk(&self, chunk_pos: bevy::prelude::IVec2) -> Chunk {
        let mut chunk = Chunk::new(chunk_pos);

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_x = (chunk_pos.x * CHUNK_SIZE as i32 + x as i32) as f64;
                let world_y = (chunk_pos.y * CHUNK_SIZE as i32 + y as i32) as f64;

                let scale = 0.02;
                let detail_scale = 0.1;

                let elevation = self.elevation.get([world_x * scale, world_y * scale]);
                let moisture = self.moisture.get([world_x * scale * 0.8, world_y * scale * 0.8]);
                let detail = self.detail.get([world_x * detail_scale, world_y * detail_scale]);

                let tile = self.determine_tile(elevation, moisture, detail);
                chunk.set_tile(x, y, tile);
            }
        }

        chunk
    }

    fn determine_tile(&self, elevation: f64, moisture: f64, detail: f64) -> TileType {
        if elevation < -0.3 {
            return TileType::DeepWater;
        }
        if elevation < -0.15 {
            return TileType::Water;
        }
        if elevation < -0.05 {
            return TileType::Sand;
        }
        if elevation > 0.6 {
            return TileType::Stone;
        }
        if moisture < -0.3 && detail > 0.2 {
            return TileType::Dirt;
        }
        if moisture > 0.2 {
            return TileType::DarkGrass;
        }
        TileType::Grass
    }

    /// Deterministic hash for object placement — same position always gives same result
    pub fn position_hash(x: i32, y: i32, seed: u32) -> u32 {
        let mut h = seed;
        h = h.wrapping_mul(374761393);
        h = h.wrapping_add(x as u32).wrapping_mul(668265263);
        h = h.wrapping_add(y as u32).wrapping_mul(2654435761);
        h ^= h >> 13;
        h = h.wrapping_mul(1274126177);
        h ^= h >> 16;
        h
    }

    /// Returns true if a tree should spawn at this world tile position
    pub fn should_spawn_tree(&self, world_x: i32, world_y: i32, seed: u32) -> bool {
        let hash = Self::position_hash(world_x, world_y, seed);
        (hash % 100) < 6 // ~6% tree density
    }

    /// Returns true if a rock should spawn at this world tile position
    pub fn should_spawn_rock(&self, world_x: i32, world_y: i32, seed: u32) -> bool {
        let hash = Self::position_hash(world_x, world_y, seed.wrapping_add(100));
        (hash % 100) < 3 // ~3% rock density
    }

    /// Returns true if a bush should spawn at this world tile position
    pub fn should_spawn_bush(&self, world_x: i32, world_y: i32, seed: u32) -> bool {
        let hash = Self::position_hash(world_x, world_y, seed.wrapping_add(200));
        (hash % 100) < 4 // ~4% bush density
    }
}
