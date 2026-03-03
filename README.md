# Driftlands

**Survival crafting game built with Bevy and Rust** -- Procedural world generation, combat, farming, building, and exploration.

Driftlands is a 2D survival crafting game where players explore procedurally generated worlds, gather resources, farm, craft tools and equipment, build structures, fight enemies, and delve into dungeons. Built from scratch in Rust using the Bevy game engine.

---

## Features

- **Procedural world generation** -- Noise-based terrain with biomes, chunk streaming, and tile-based maps
- **Combat** -- Real-time combat system with weapons, enemies, and a death/respawn loop
- **Crafting** -- Recipe-based crafting system for tools, weapons, and materials
- **Farming** -- Plant crops, manage growth cycles tied to seasons and weather
- **Building** -- Place and construct structures in the world
- **Day/night cycle** -- Dynamic lighting and time-of-day effects
- **Weather system** -- Rain, storms, and seasonal weather patterns
- **Seasons** -- World changes with the seasons, affecting farming, weather, and NPC behavior
- **Dungeons** -- Procedurally generated dungeon instances with enemies and loot
- **NPCs** -- Non-player characters with dialogue and interactions
- **Inventory system** -- Grid-based inventory with item stacking and equipment slots
- **Save/load** -- Full game state serialization with serde and bincode
- **HUD & minimap** -- In-game UI with health, inventory quickbar, and minimap
- **Main menu** -- Title screen with save slot management
- **Audio** -- Sound effects and ambient audio
- **Particles** -- Visual effects for combat, weather, and environment
- **Gathering** -- Harvest resources from the world (trees, rocks, plants)
- **Tech tree** -- Progression system unlocking new recipes and abilities
- **Lore** -- Discoverable world lore and story elements

## Architecture

```
driftlands/
├── src/
│   ├── main.rs            # App entry point, plugin registration
│   ├── player.rs          # Player controller, stats, movement
│   ├── combat.rs          # Combat system, damage, enemies
│   ├── crafting.rs        # Crafting recipes and logic
│   ├── farming.rs         # Crop planting, growth, harvest
│   ├── building.rs        # Structure placement and construction
│   ├── inventory.rs       # Item management, equipment
│   ├── gathering.rs       # Resource harvesting
│   ├── dungeon.rs         # Dungeon generation and gameplay
│   ├── npc.rs             # NPC behavior and dialogue
│   ├── daynight.rs        # Day/night cycle
│   ├── weather.rs         # Weather system
│   ├── season.rs          # Seasonal changes
│   ├── saveload.rs        # Save/load with serde + bincode
│   ├── camera.rs          # Camera follow and controls
│   ├── hud.rs             # In-game UI
│   ├── minimap.rs         # Minimap rendering
│   ├── mainmenu.rs        # Title screen
│   ├── audio.rs           # Sound management
│   ├── particles.rs       # Particle effects
│   ├── death.rs           # Death and respawn
│   ├── techtree.rs        # Progression unlocks
│   ├── lore.rs            # World lore system
│   ├── experiment.rs      # Experimental features
│   └── world/
│       ├── mod.rs          # World plugin
│       ├── generation.rs   # Procedural world gen (noise-based)
│       ├── chunk.rs        # Chunk loading/unloading
│       └── tile.rs         # Tile types and properties
├── assets/                 # Sprites, audio, data files
├── Cargo.toml              # Rust dependencies
└── README.md
```

## Requirements

- Rust 1.75+ (2021 edition)
- Cargo

## Build & Run

```bash
# Run in development mode (with dynamic linking for fast iteration)
cargo run

# Build release
cargo build --release

# Run release build
./target/release/driftlands
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| bevy 0.15 | Game engine (ECS, rendering, input, audio) |
| noise 0.9 | Procedural noise for world generation |
| rand 0.8 | Random number generation |
| serde 1 | Serialization for save/load |
| bincode 1 | Binary encoding for save files |

## Dev Profile

Development builds use `opt-level = 1` for the game code and `opt-level = 3` for dependencies, balancing compile time with runtime performance during iteration.

## License

Proprietary. All rights reserved.
