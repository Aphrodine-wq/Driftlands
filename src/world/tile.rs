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
            TileType::Grass => [86, 166, 75, 255],
            TileType::DarkGrass => [62, 137, 52, 255],
            TileType::Dirt => [139, 105, 69, 255],
            TileType::Water => [64, 128, 200, 255],
            TileType::DeepWater => [40, 80, 160, 255],
            TileType::Sand => [210, 190, 140, 255],
            TileType::Stone => [140, 140, 140, 255],
            TileType::Mud => [90, 70, 50, 255],
            TileType::Ice => [180, 210, 240, 255],
            TileType::Snow => [230, 235, 240, 255],
            TileType::Lava => [200, 60, 20, 255],
            TileType::Obsidian => [30, 20, 35, 255],
            TileType::MushroomGround => [80, 60, 100, 255],
            TileType::CrystalFloor => [160, 140, 200, 255],
            TileType::MountainStone => [120, 115, 110, 255],
        }
    }

    /// Biome-aware tile color — each biome gets a distinct palette for common tiles.
    pub fn biome_color(&self, biome: Biome) -> [u8; 4] {
        match biome {
            Biome::Forest => match self {
                // Rich green palette
                TileType::Grass => [51, 140, 51, 255],
                TileType::DarkGrass => [38, 115, 38, 255],
                TileType::Dirt => [120, 90, 55, 255],
                TileType::Sand => [170, 155, 110, 255],
                TileType::Water => [64, 128, 200, 255],
                TileType::DeepWater => [40, 80, 160, 255],
                _ => self.color(),
            },
            Biome::Coastal => match self {
                // Sandy yellow palette
                TileType::Grass => [140, 165, 90, 255],
                TileType::DarkGrass => [110, 140, 70, 255],
                TileType::Dirt => [160, 140, 100, 255],
                TileType::Sand => [194, 179, 128, 255],
                // Tropical blue water
                TileType::Water => [60, 160, 210, 255],
                TileType::DeepWater => [30, 120, 180, 255],
                _ => self.color(),
            },
            Biome::Swamp => match self {
                // Murky olive palette
                TileType::Grass => [70, 95, 45, 255],
                TileType::DarkGrass => [55, 80, 35, 255],
                TileType::Dirt => [80, 65, 40, 255],
                TileType::Sand => [110, 100, 65, 255],
                TileType::Mud => [70, 55, 35, 255],
                // Dark murky water
                TileType::Water => [45, 75, 60, 255],
                TileType::DeepWater => [30, 55, 45, 255],
                _ => self.color(),
            },
            Biome::Desert => match self {
                // Warm tan palette
                TileType::Grass => [170, 155, 100, 255],
                TileType::DarkGrass => [145, 130, 80, 255],
                TileType::Dirt => [175, 145, 95, 255],
                TileType::Sand => [209, 184, 140, 255],
                TileType::Stone => [170, 150, 120, 255],
                TileType::Water => [70, 130, 180, 255],
                TileType::DeepWater => [50, 100, 150, 255],
                _ => self.color(),
            },
            Biome::Tundra => match self {
                // Icy white-blue palette
                TileType::Grass => [170, 190, 180, 255],
                TileType::DarkGrass => [150, 175, 165, 255],
                TileType::Dirt => [160, 155, 145, 255],
                TileType::Sand => [190, 185, 175, 255],
                TileType::Snow => [230, 235, 240, 255],
                TileType::Ice => [180, 210, 240, 255],
                TileType::Stone => [155, 160, 165, 255],
                // Icy water
                TileType::Water => [100, 150, 200, 255],
                TileType::DeepWater => [70, 110, 170, 255],
                _ => self.color(),
            },
            Biome::Volcanic => match self {
                // Dark red-brown palette
                TileType::Grass => [90, 55, 35, 255],
                TileType::DarkGrass => [75, 45, 30, 255],
                TileType::Dirt => [100, 60, 40, 255],
                TileType::Sand => [130, 90, 60, 255],
                TileType::Stone => [80, 65, 55, 255],
                TileType::Lava => [200, 60, 20, 255],
                TileType::Obsidian => [30, 20, 35, 255],
                TileType::Water => [80, 60, 70, 255],
                TileType::DeepWater => [55, 40, 50, 255],
                _ => self.color(),
            },
            Biome::Fungal => match self {
                // Purple-teal palette
                TileType::Grass => [80, 90, 70, 255],
                TileType::DarkGrass => [65, 75, 85, 255],
                TileType::Dirt => [90, 70, 100, 255],
                TileType::Sand => [120, 100, 130, 255],
                TileType::MushroomGround => [80, 60, 100, 255],
                TileType::Water => [70, 90, 130, 255],
                TileType::DeepWater => [50, 65, 100, 255],
                _ => self.color(),
            },
            Biome::CrystalCave => match self {
                // Pale blue-white palette
                TileType::Grass => [140, 150, 170, 255],
                TileType::DarkGrass => [120, 130, 155, 255],
                TileType::Dirt => [130, 125, 150, 255],
                TileType::Sand => [160, 155, 175, 255],
                TileType::Stone => [135, 140, 160, 255],
                TileType::CrystalFloor => [160, 140, 200, 255],
                TileType::Water => [90, 130, 190, 255],
                TileType::DeepWater => [65, 100, 160, 255],
                _ => self.color(),
            },
            Biome::Mountain => match self {
                // Gray-brown palette
                TileType::Grass => [100, 110, 85, 255],
                TileType::DarkGrass => [80, 92, 70, 255],
                TileType::Dirt => [115, 105, 85, 255],
                TileType::Sand => [145, 135, 115, 255],
                TileType::Stone => [128, 122, 107, 255],
                TileType::MountainStone => [120, 115, 110, 255],
                TileType::Snow => [225, 230, 235, 255],
                TileType::Water => [70, 115, 170, 255],
                TileType::DeepWater => [45, 85, 140, 255],
                _ => self.color(),
            },
        }
    }

    pub fn is_walkable(&self) -> bool {
        !matches!(self, TileType::Water | TileType::DeepWater | TileType::Lava)
    }
}
