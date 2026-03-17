use noise::{NoiseFn, Perlin};
use serde::{Serialize, Deserialize};
use super::chunk::{Chunk, CHUNK_SIZE};
use super::tile::TileType;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Biome {
    Forest,
    Coastal,
    Swamp,
    Desert,
    Tundra,
    Volcanic,
    Fungal,
    CrystalCave,
    Mountain,
}

pub struct WorldGenerator {
    elevation: Perlin,
    moisture: Perlin,
    detail: Perlin,
    temperature: Perlin,
}

impl WorldGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            elevation: Perlin::new(seed),
            moisture: Perlin::new(seed.wrapping_add(1)),
            detail: Perlin::new(seed.wrapping_add(2)),
            temperature: Perlin::new(seed.wrapping_add(3)),
        }
    }

    /// Determine biome from temperature/moisture Whittaker diagram
    pub fn biome_at(&self, world_x: f64, world_y: f64) -> Biome {
        let biome_scale = 0.0016; // Larger-scale biome regions (approx 3x larger than 0.005)
        let temp = self.temperature.get([world_x * biome_scale, world_y * biome_scale]);
        let moist = self.moisture.get([world_x * biome_scale * 0.8, world_y * biome_scale * 0.8]);
        let elev = self.elevation.get([world_x * 0.02, world_y * 0.02]);

        // High elevation overrides
        if elev > 0.55 {
            return Biome::Mountain;
        }

        // Underground / deep areas
        if elev < -0.4 {
            return Biome::CrystalCave;
        }

        // Coastal: near water level
        if elev < -0.1 {
            return Biome::Coastal;
        }

        // Temperature/moisture Whittaker diagram
        match (temp, moist) {
            (t, _) if t > 0.4 && moist < -0.2 => Biome::Desert,
            (t, _) if t > 0.4 && moist > 0.2 => Biome::Volcanic,
            (t, _) if t < -0.35 => Biome::Tundra,
            (_, m) if m > 0.35 && temp < 0.1 => Biome::Swamp,
            (_, m) if m > 0.25 && temp > 0.1 => Biome::Fungal,
            _ => Biome::Forest,
        }
    }

    pub fn generate_chunk(&self, chunk_pos: bevy::prelude::IVec2) -> Chunk {
        // Determine biome from chunk center
        let center_x = (chunk_pos.x * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 / 2) as f64;
        let center_y = (chunk_pos.y * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 / 2) as f64;
        let biome = self.biome_at(center_x, center_y);

        let mut chunk = Chunk::new(chunk_pos);
        chunk.biome = biome;

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_x = (chunk_pos.x * CHUNK_SIZE as i32 + x as i32) as f64;
                let world_y = (chunk_pos.y * CHUNK_SIZE as i32 + y as i32) as f64;

                let scale = 0.02;
                let detail_scale = 0.1;

                let elevation = self.elevation.get([world_x * scale, world_y * scale]);
                let moisture = self.moisture.get([world_x * scale * 0.8, world_y * scale * 0.8]);
                let detail = self.detail.get([world_x * detail_scale, world_y * detail_scale]);

                let tile = self.determine_tile(elevation, moisture, detail, biome);
                chunk.set_tile(x, y, tile);
            }
        }

        chunk
    }

    fn determine_tile(&self, elevation: f64, moisture: f64, detail: f64, biome: Biome) -> TileType {
        // Universal water
        if elevation < -0.3 {
            return TileType::DeepWater;
        }
        if elevation < -0.15 {
            return TileType::Water;
        }

        match biome {
            Biome::Forest => {
                if elevation < -0.05 { return TileType::Sand; }
                if elevation > 0.6 { return TileType::Stone; }
                if moisture < -0.3 && detail > 0.2 { return TileType::Dirt; }
                if moisture > 0.2 { return TileType::DarkGrass; }
                TileType::Grass
            }
            Biome::Coastal => {
                if elevation < 0.0 { return TileType::Sand; }
                if detail > 0.3 { return TileType::Dirt; }
                TileType::Sand
            }
            Biome::Swamp => {
                if elevation < -0.05 { return TileType::Water; }
                if detail > 0.2 { return TileType::Mud; }
                TileType::DarkGrass
            }
            Biome::Desert => {
                if elevation > 0.5 { return TileType::Stone; }
                if detail > 0.4 { return TileType::Dirt; }
                TileType::Sand
            }
            Biome::Tundra => {
                if detail > 0.3 { return TileType::Ice; }
                TileType::Snow
            }
            Biome::Volcanic => {
                if detail > 0.4 { return TileType::Lava; }
                if detail > 0.1 { return TileType::Obsidian; }
                TileType::Stone
            }
            Biome::Fungal => {
                if detail > 0.3 { return TileType::MushroomGround; }
                TileType::DarkGrass
            }
            Biome::CrystalCave => {
                if detail > 0.3 { return TileType::CrystalFloor; }
                TileType::Stone
            }
            Biome::Mountain => {
                if detail > 0.3 { return TileType::MountainStone; }
                if elevation > 0.7 { return TileType::Snow; }
                TileType::Stone
            }
        }
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
    #[allow(dead_code)]
    pub fn should_spawn_tree(&self, world_x: i32, world_y: i32, seed: u32) -> bool {
        let hash = Self::position_hash(world_x, world_y, seed);
        (hash % 100) < 6 // ~6% tree density
    }

    /// Returns true if a rock should spawn at this world tile position
    #[allow(dead_code)]
    pub fn should_spawn_rock(&self, world_x: i32, world_y: i32, seed: u32) -> bool {
        let hash = Self::position_hash(world_x, world_y, seed.wrapping_add(100));
        (hash % 100) < 3 // ~3% rock density
    }

    /// Returns true if a bush should spawn at this world tile position
    #[allow(dead_code)]
    pub fn should_spawn_bush(&self, world_x: i32, world_y: i32, seed: u32) -> bool {
        let hash = Self::position_hash(world_x, world_y, seed.wrapping_add(200));
        (hash % 100) < 4 // ~4% bush density
    }
}
