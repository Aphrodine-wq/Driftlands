use serde::{Serialize, Deserialize};
use super::generation::Biome;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Grass,
    DarkGrass,
    Dirt,
    Water,
    DeepWater,
    Sand,
    Stone,
    // Phase 3 biome tiles
    Mud,
    Ice,
    Snow,
    Lava,
    Obsidian,
    MushroomGround,
    CrystalFloor,
    MountainStone,
}

impl TileType {
    /// Default tile color (biome-agnostic fallback).
    pub fn color(&self) -> [u8; 4] {
        match self {
            TileType::Grass => [20, 40, 20, 255],
            TileType::DarkGrass => [15, 30, 15, 255],
            TileType::Dirt => [35, 25, 20, 255],
            TileType::Water => [10, 25, 45, 255],
            TileType::DeepWater => [5, 15, 30, 255],
            TileType::Sand => [50, 45, 35, 255],
            TileType::Stone => [30, 30, 35, 255],
            TileType::Mud => [25, 20, 15, 255],
            TileType::Ice => [45, 55, 65, 255],
            TileType::Snow => [60, 65, 70, 255],
            TileType::Lava => [80, 25, 10, 255],
            TileType::Obsidian => [10, 5, 15, 255],
            TileType::MushroomGround => [25, 15, 35, 255],
            TileType::CrystalFloor => [40, 35, 55, 255],
            TileType::MountainStone => [25, 25, 28, 255],
        }
    }

    /// Biome-aware tile color — each biome gets a distinct palette for common tiles.
    pub fn biome_color(&self, biome: Biome) -> [u8; 4] {
        match biome {
            Biome::Forest => match self {
                // Deep nocturnal green
                TileType::Grass => [15, 45, 15, 255],
                TileType::DarkGrass => [10, 35, 10, 255],
                TileType::Dirt => [30, 25, 15, 255],
                TileType::Sand => [45, 40, 30, 255],
                TileType::Water => [15, 35, 55, 255],
                TileType::DeepWater => [10, 25, 40, 255],
                _ => self.color(),
            },
            Biome::Coastal => match self {
                // Spectral slate sand
                TileType::Grass => [40, 50, 30, 255],
                TileType::DarkGrass => [30, 40, 20, 255],
                TileType::Dirt => [45, 40, 30, 255],
                TileType::Sand => [55, 50, 40, 255],
                TileType::Water => [20, 55, 75, 255],
                TileType::DeepWater => [15, 40, 60, 255],
                _ => self.color(),
            },
            Biome::Swamp => match self {
                // Murky obsidian palette
                TileType::Grass => [20, 30, 15, 255],
                TileType::DarkGrass => [15, 25, 10, 255],
                TileType::Dirt => [25, 20, 15, 255],
                TileType::Sand => [35, 30, 25, 255],
                TileType::Mud => [20, 15, 10, 255],
                TileType::Water => [15, 25, 20, 255],
                TileType::DeepWater => [10, 15, 12, 255],
                _ => self.color(),
            },
            Biome::Desert => match self {
                // Warm amber sand (darkened)
                TileType::Grass => [50, 45, 30, 255],
                TileType::DarkGrass => [40, 35, 20, 255],
                TileType::Dirt => [55, 45, 30, 255],
                TileType::Sand => [65, 55, 45, 255],
                TileType::Stone => [50, 45, 40, 255],
                TileType::Water => [25, 45, 60, 255],
                TileType::DeepWater => [15, 35, 50, 255],
                _ => self.color(),
            },
            Biome::Tundra => match self {
                // Cold spectral slate
                TileType::Grass => [45, 55, 55, 255],
                TileType::DarkGrass => [35, 45, 45, 255],
                TileType::Dirt => [40, 40, 45, 255],
                TileType::Sand => [50, 50, 55, 255],
                TileType::Snow => [70, 75, 80, 255],
                TileType::Ice => [55, 65, 75, 255],
                TileType::Stone => [45, 45, 50, 255],
                TileType::Water => [30, 45, 65, 255],
                TileType::DeepWater => [20, 35, 55, 255],
                _ => self.color(),
            },
            Biome::Volcanic => match self {
                // Ash and obsidian
                TileType::Grass => [25, 15, 10, 255],
                TileType::DarkGrass => [20, 10, 8, 255],
                TileType::Dirt => [30, 20, 15, 255],
                TileType::Sand => [40, 30, 20, 255],
                TileType::Stone => [25, 20, 18, 255],
                TileType::Lava => [80, 25, 10, 255],
                TileType::Obsidian => [10, 5, 15, 255],
                TileType::Water => [25, 15, 20, 255],
                TileType::DeepWater => [15, 10, 15, 255],
                _ => self.color(),
            },
            Biome::Fungal => match self {
                // Luminous purple (darkened)
                TileType::Grass => [25, 30, 25, 255],
                TileType::DarkGrass => [20, 25, 28, 255],
                TileType::Dirt => [30, 25, 35, 255],
                TileType::Sand => [40, 35, 45, 255],
                TileType::MushroomGround => [30, 20, 40, 255],
                TileType::Water => [25, 30, 45, 255],
                TileType::DeepWater => [15, 20, 35, 255],
                _ => self.color(),
            },
            Biome::CrystalCave => match self {
                // Spectral blue
                TileType::Grass => [40, 45, 55, 255],
                TileType::DarkGrass => [35, 40, 50, 255],
                TileType::Dirt => [40, 35, 45, 255],
                TileType::Sand => [50, 45, 55, 255],
                TileType::Stone => [40, 45, 50, 255],
                TileType::CrystalFloor => [50, 45, 65, 255],
                TileType::Water => [30, 45, 60, 255],
                TileType::DeepWater => [20, 35, 50, 255],
                _ => self.color(),
            },
            Biome::Mountain => match self {
                // Slate and granite
                TileType::Grass => [35, 40, 30, 255],
                TileType::DarkGrass => [25, 35, 20, 255],
                TileType::Dirt => [40, 35, 30, 255],
                TileType::Sand => [50, 45, 40, 255],
                TileType::Stone => [40, 38, 35, 255],
                TileType::MountainStone => [35, 35, 38, 255],
                TileType::Snow => [75, 80, 85, 255],
                TileType::Water => [25, 40, 55, 255],
                TileType::DeepWater => [15, 30, 45, 255],
                _ => self.color(),
            },
        }
    }

    pub fn is_walkable(&self) -> bool {
        !matches!(self, TileType::Water | TileType::DeepWater | TileType::Lava)
    }
}
