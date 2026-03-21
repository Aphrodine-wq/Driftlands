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
            TileType::Grass => [55, 130, 45, 255],
            TileType::DarkGrass => [40, 100, 35, 255],
            TileType::Dirt => [120, 90, 60, 255],
            TileType::Water => [45, 100, 170, 255],
            TileType::DeepWater => [25, 65, 130, 255],
            TileType::Sand => [190, 175, 130, 255],
            TileType::Stone => [120, 120, 130, 255],
            TileType::Mud => [85, 65, 45, 255],
            TileType::Ice => [170, 200, 225, 255],
            TileType::Snow => [220, 230, 240, 255],
            TileType::Lava => [210, 80, 25, 255],
            TileType::Obsidian => [35, 25, 45, 255],
            TileType::MushroomGround => [80, 50, 100, 255],
            TileType::CrystalFloor => [110, 95, 155, 255],
            TileType::MountainStone => [95, 95, 105, 255],
        }
    }

    /// Biome-aware tile color — each biome gets a distinct palette for common tiles.
    pub fn biome_color(&self, biome: Biome) -> [u8; 4] {
        match biome {
            Biome::Forest => match self {
                // Rich forest greens
                TileType::Grass => [45, 120, 35, 255],
                TileType::DarkGrass => [30, 90, 25, 255],
                TileType::Dirt => [110, 80, 50, 255],
                TileType::Sand => [160, 145, 105, 255],
                TileType::Water => [40, 95, 160, 255],
                TileType::DeepWater => [20, 60, 120, 255],
                _ => self.color(),
            },
            Biome::Coastal => match self {
                // Warm sand and bright water
                TileType::Grass => [90, 140, 65, 255],
                TileType::DarkGrass => [70, 115, 50, 255],
                TileType::Dirt => [145, 125, 90, 255],
                TileType::Sand => [190, 175, 130, 255],
                TileType::Water => [50, 130, 185, 255],
                TileType::DeepWater => [30, 90, 155, 255],
                _ => self.color(),
            },
            Biome::Swamp => match self {
                // Murky but readable green-brown
                TileType::Grass => [55, 85, 40, 255],
                TileType::DarkGrass => [40, 65, 30, 255],
                TileType::Dirt => [80, 60, 40, 255],
                TileType::Sand => [105, 90, 65, 255],
                TileType::Mud => [70, 55, 35, 255],
                TileType::Water => [45, 75, 55, 255],
                TileType::DeepWater => [30, 50, 40, 255],
                _ => self.color(),
            },
            Biome::Desert => match self {
                // Golden sand
                TileType::Grass => [150, 135, 80, 255],
                TileType::DarkGrass => [130, 115, 65, 255],
                TileType::Dirt => [170, 140, 85, 255],
                TileType::Sand => [210, 185, 120, 255],
                TileType::Stone => [155, 140, 115, 255],
                TileType::Water => [55, 115, 160, 255],
                TileType::DeepWater => [35, 80, 130, 255],
                _ => self.color(),
            },
            Biome::Tundra => match self {
                // Bright white snow and ice blue
                TileType::Grass => [130, 155, 145, 255],
                TileType::DarkGrass => [105, 130, 120, 255],
                TileType::Dirt => [120, 115, 110, 255],
                TileType::Sand => [155, 150, 145, 255],
                TileType::Snow => [220, 230, 240, 255],
                TileType::Ice => [170, 200, 225, 255],
                TileType::Stone => [135, 135, 145, 255],
                TileType::Water => [60, 110, 165, 255],
                TileType::DeepWater => [40, 80, 140, 255],
                _ => self.color(),
            },
            Biome::Volcanic => match self {
                // Dark ash with visible red/orange contrast
                TileType::Grass => [75, 50, 35, 255],
                TileType::DarkGrass => [60, 40, 28, 255],
                TileType::Dirt => [90, 60, 40, 255],
                TileType::Sand => [115, 85, 55, 255],
                TileType::Stone => [80, 65, 55, 255],
                TileType::Lava => [210, 80, 25, 255],
                TileType::Obsidian => [35, 25, 45, 255],
                TileType::Water => [70, 45, 50, 255],
                TileType::DeepWater => [45, 30, 40, 255],
                _ => self.color(),
            },
            Biome::Fungal => match self {
                // Purple/teal mystical
                TileType::Grass => [70, 95, 80, 255],
                TileType::DarkGrass => [55, 75, 85, 255],
                TileType::Dirt => [90, 70, 105, 255],
                TileType::Sand => [115, 100, 125, 255],
                TileType::MushroomGround => [95, 55, 120, 255],
                TileType::Water => [65, 80, 130, 255],
                TileType::DeepWater => [40, 55, 100, 255],
                _ => self.color(),
            },
            Biome::CrystalCave => match self {
                // Blue/purple sparkle
                TileType::Grass => [90, 105, 140, 255],
                TileType::DarkGrass => [75, 90, 125, 255],
                TileType::Dirt => [100, 85, 120, 255],
                TileType::Sand => [130, 120, 150, 255],
                TileType::Stone => [105, 110, 130, 255],
                TileType::CrystalFloor => [120, 105, 170, 255],
                TileType::Water => [60, 100, 155, 255],
                TileType::DeepWater => [40, 70, 125, 255],
                _ => self.color(),
            },
            Biome::Mountain => match self {
                // Grey stone with visible texture contrast
                TileType::Grass => [80, 105, 65, 255],
                TileType::DarkGrass => [60, 85, 50, 255],
                TileType::Dirt => [110, 95, 75, 255],
                TileType::Sand => [140, 130, 110, 255],
                TileType::Stone => [130, 125, 115, 255],
                TileType::MountainStone => [110, 110, 120, 255],
                TileType::Snow => [215, 225, 235, 255],
                TileType::Water => [50, 100, 150, 255],
                TileType::DeepWater => [30, 70, 120, 255],
                _ => self.color(),
            },
        }
    }

    pub fn is_walkable(&self) -> bool {
        // Traversal collision: treat water/deep water/lava as impassable.
        // (Swimming/tinting is still implemented visually, but traversal will prevent entering these tiles.)
        !matches!(self, TileType::Water | TileType::DeepWater | TileType::Lava)
    }

    /// Returns true if this tile is a water/swim tile (slows movement).
    pub fn is_water(&self) -> bool {
        matches!(self, TileType::Water | TileType::DeepWater)
    }
}
