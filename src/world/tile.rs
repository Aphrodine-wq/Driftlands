use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Grass,
    DarkGrass,
    Dirt,
    Water,
    DeepWater,
    Sand,
    Stone,
}

impl TileType {
    pub fn color(&self) -> [u8; 4] {
        match self {
            TileType::Grass => [86, 166, 75, 255],
            TileType::DarkGrass => [62, 137, 52, 255],
            TileType::Dirt => [139, 105, 69, 255],
            TileType::Water => [64, 128, 200, 255],
            TileType::DeepWater => [40, 80, 160, 255],
            TileType::Sand => [210, 190, 140, 255],
            TileType::Stone => [140, 140, 140, 255],
        }
    }

    pub fn is_walkable(&self) -> bool {
        !matches!(self, TileType::Water | TileType::DeepWater)
    }
}
