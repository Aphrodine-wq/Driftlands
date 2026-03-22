# Driftlands

**Survival crafting game built with Bevy and Rust** -- Procedural world generation, combat, farming, building, and exploration.

Driftlands is a 2D top-down survival crafting game where players explore procedurally generated worlds, gather resources, farm, craft tools and equipment, build structures, fight enemies, and delve into dungeons. Built from scratch in Rust using the Bevy game engine.

---

## Features

- **Procedural world generation** -- Noise-based terrain with biomes, chunk streaming, and tile-based maps
- **Combat** -- Real-time combat with a 5-state enemy AI (Idle/Patrol/Alert/Chase/Attack), damage numbers, screen shake, and knockback
- **Status effects** -- Poison, Burn, Freeze, Bleed, Stun, Regen, and WellFed applied via weapons and environment
- **Enchanting** -- Enchanted weapons (FlameBlade, FrostBlade, VenomBlade, LifestealBlade) that apply on-hit effects
- **Crafting** -- Recipe-based crafting system gated by tech tier and proximity to crafting stations
- **Tech tree** -- Progression system unlocking new recipes and abilities via research points
- **Skills** -- Six leveled skills (Gathering, Combat, Fishing, Farming, Crafting, Building) with XP gain
- **Farming** -- Plant crops with growth cycles tied to seasons and weather; cooked food gives stat buffs
- **Fishing** -- Cast at water tiles; six fish types with biome variation and cook/eat mechanics
- **Pets** -- Tame enemies into companions (Wolf, Cat, Hawk, Bear) each with unique passive effects
- **Quests** -- Quest system with reward items and research points; given by NPCs or triggered by actions
- **Building** -- Place and construct structures with placement validation (red preview on invalid spots)
- **Automation** -- Auto-smelter, crop sprinkler, and alarm bell placeable structures
- **World structures** -- Procedural points of interest: Abandoned Villages, Mine Shafts, Trader Outposts, Watchtowers, Fishing Docks, Wolf Dens, Spider Nests, Scorpion Burrows
- **Dungeons** -- Procedurally generated dungeon instances with rooms, corridors, enemies, and loot
- **NPCs** -- Trader NPCs, hermit NPCs, dialogue, and quest givers
- **Inventory system** -- 36-slot grid inventory, 9-slot hotbar, equipment slots, and chest storage (18 slots)
- **Day/night cycle** -- Dynamic ClearColor lerp through phases; atmosphere changes
- **Lighting** -- Normal-mapped 2D lighting via custom WGSL shaders (lit_chunk and lit_sprite materials); point lights for campfires and torches
- **Weather system** -- Rain, storms, and gameplay effects tied to weather state
- **Seasons** -- Four seasons with visual palette shifts affecting farming, weather, and world objects; seasonal tree sprites
- **Gathering** -- Harvest resources from world objects with progress bar and pickup animation
- **Particles** -- Particle effects for combat, gathering, and environment
- **Audio** -- Sound event system wired to combat, gathering, building, and crafting
- **Save/load** -- Full game state serialization with serde and bincode; multiple named save slots; backwards-compatible via `#[serde(default)]`
- **HUD** -- Health/hunger bars, hotbar, minimap with toggle and fullscreen mode, floating damage text, pause menu
- **Tutorial** -- First-play contextual hint system for gathering, combat, and exploration
- **Controls overlay** -- In-game keyboard shortcut reference panel (toggled via keybind)
- **Settings** -- Persistent settings menu: resolution, fullscreen (F11 toggle), audio volume, gamepad support
- **Performance tools** -- Debug overlay tracking chunk management, spatial grid update, and animation atlas build timings
- **Main menu** -- Title screen with New Game, Continue (save slot browser), and Quit
- **Lore** -- Discoverable world lore and story elements
- **Experiment system** -- In-game item combination/discovery workbench

---

## Architecture

```
driftlands/
├── src/
│   ├── main.rs             # App entry point, plugin registration
│   ├── assets.rs           # Asset loading and handles
│   ├── player.rs           # Player controller, stats, movement, facing
│   ├── camera.rs           # Camera follow, dead zone, lerp, screen shake
│   ├── combat.rs           # Damage, enemy AI, knockback, research points
│   ├── death.rs            # Death screen, respawn, stats tracking
│   ├── status_effects.rs   # Poison, Burn, Freeze, Bleed, Stun, Regen, WellFed
│   ├── enchanting.rs       # Enchanted weapon on-hit effect helpers
│   ├── inventory.rs        # Item management, equipment, item types
│   ├── crafting.rs         # Recipe system, crafting station proximity
│   ├── techtree.rs         # Tech unlock progression
│   ├── skills.rs           # Gathering/Combat/Fishing/Farming/Crafting/Building XP
│   ├── gathering.rs        # Resource harvesting, dropped item pickup
│   ├── fishing.rs          # Fishing minigame, fish types, biome variation
│   ├── farming.rs          # Crop planting, growth, harvest, food buffs
│   ├── pets.rs             # Pet taming and companion behavior
│   ├── quests.rs           # Quest definitions, progress events, rewards
│   ├── building.rs         # Structure placement, validation, chest UI
│   ├── structures.rs       # Procedural world POIs and lootable structures
│   ├── automation.rs       # Auto-smelter, crop sprinkler, alarm bell
│   ├── dungeon.rs          # Dungeon generation and gameplay
│   ├── npc.rs              # NPC behavior, trader, hermit, dialogue
│   ├── daynight.rs         # Day/night cycle, phases
│   ├── lighting.rs         # Normal-mapped 2D lighting uniforms
│   ├── lit_materials.rs    # Custom WGSL materials for chunks and sprites
│   ├── theme.rs            # EtherealTheme color palette resource
│   ├── weather.rs          # Weather state and gameplay effects
│   ├── season.rs           # Seasonal transitions and visual palette shifts
│   ├── particles.rs        # Particle effects
│   ├── audio.rs            # SoundEvent system, GameAudio resource
│   ├── animation.rs        # Frame-based sprite animation (atlas-backed)
│   ├── spatial.rs          # Spatial hash grid for entity lookups (128px cells)
│   ├── hud.rs              # In-game UI, pause system, floating text, hotbar
│   ├── minimap.rs          # Minimap rendering, toggle, fullscreen map
│   ├── saveload.rs         # Save/load serialization, save slot management
│   ├── saveslots.rs        # Save slot browser UI (main menu and pause menu)
│   ├── mainmenu.rs         # Title screen
│   ├── tutorial.rs         # First-play hint system
│   ├── controls.rs         # Keyboard shortcut overlay
│   ├── settings.rs         # Settings menu, resolution, fullscreen, audio
│   ├── debug_perf.rs       # Performance timing overlay
│   ├── lore.rs             # World lore system
│   └── experiment.rs       # Item combination workbench
│   └── world/
│       ├── mod.rs          # World plugin, WorldState, ChunkObject
│       ├── generation.rs   # Procedural world gen (noise-based), biomes
│       ├── chunk.rs        # Chunk loading/unloading, tile data arrays
│       └── tile.rs         # Tile types, biome color palettes
├── assets/
│   ├── shaders/            # WGSL shaders (lit_chunk.wgsl, lit_sprite.wgsl)
│   ├── ui/                 # UI icon sprites
│   ├── ui_extra/           # HUD background sprites
│   ├── buildings/          # Building sprites
│   ├── tools/              # Tool and weapon sprites
│   ├── pets/               # Pet sprites
│   ├── crops_raw/          # Raw crop item sprites
│   ├── food_cooked/        # Cooked food item sprites
│   ├── fish_items/         # Fish item sprites
│   ├── trees_seasonal/     # Seasonal tree sprites (oak/pine x 4 seasons)
│   ├── elite_enemies/      # Elite/boss enemy sprites
│   ├── enchant_effects/    # Enchanted weapon aura sprites
│   └── audio/              # Audio files (placeholder)
├── ralph/                  # Ralph agent workspace (PRD, progress log)
├── Cargo.toml              # Rust dependencies
└── README.md
```

---

## Requirements

- Rust 1.75+ (2021 edition)
- Cargo

---

## Build & Run

```bash
# Run in development mode with dynamic linking (fastest iteration)
cargo run --features dev

# Run in development mode without dynamic linking
cargo run

# Build optimized release binary
cargo build --release

# Run the release binary
./target/release/driftlands
```

The `dev` feature enables `bevy/dynamic_linking`, which significantly reduces incremental compile times during development. Release builds use LTO, single codegen unit, and binary stripping for distribution.

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| bevy | 0.15 | Game engine (ECS, rendering, input, audio) |
| noise | 0.9 | Procedural noise for world generation |
| rand | 0.8 | Random number generation |
| serde | 1 | Serialization derive macros for save/load |
| serde_json | 1 | JSON serialization (settings, PRD tooling) |
| bincode | 1 | Binary encoding for save files |

---

## Build Profiles

**dev** -- `opt-level = 1` for game code, `opt-level = 3` for dependencies. Balances compile time with runtime performance during iteration.

**release** -- `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`, `strip = true`. Optimized for distribution.

---

## License

Proprietary. All rights reserved.
