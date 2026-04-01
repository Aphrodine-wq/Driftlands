# Driftlands

## What This Is

A top-down open-world sandbox game built as a solo project in Bevy (Rust). Players explore an infinite procedurally generated world with 9 biomes, build modular structures, master deep crafting systems, engage in light survival mechanics, and discover environmental lore -- all through a top-down 3/4 perspective in the style of Stardew Valley and Delverium.

## Core Value

**Build Anything, Anywhere** -- modular snap-together construction on a rectangular grid gives players full creative control over structures. The building system is the heart of the game.

## Requirements

### Validated

<!-- Shipped and confirmed working -- all gameplay systems implemented across 47 source files -->

- [x] Bevy project with custom top-down 3/4 view rendering pipeline
- [x] Custom chunk-based tilemap renderer (tiles as data arrays, chunks as entities)
- [x] Procedural world generation with noise-based terrain and biomes
- [x] Player movement (WASD rectangular grid, normalized diagonal) with facing direction
- [x] Camera follow with smooth lerp, dead zone, mouse-wheel zoom, and screen shake
- [x] Dynamic chunk loading/unloading with spatial grid (128px cells)
- [x] World objects (trees, rocks, bushes) with deterministic placement
- [x] Click-to-gather system with object health, progress bar, and resource drops
- [x] 36-slot inventory with 9-slot hotbar, equipment slots, stacking, and display toggle
- [x] Recipe-based crafting system gated by tech tier and crafting station proximity
- [x] Tech tree progression system unlocking recipes via research points
- [x] Building placement with grid snapping and validation (red preview on invalid spots)
- [x] Chest storage (18-slot) and crafting station proximity checks (64px)
- [x] Day/night cycle with dynamic ClearColor lerp through phases
- [x] Normal-mapped 2D lighting via custom WGSL shaders (lit_chunk and lit_sprite materials)
- [x] Full HUD: health/hunger bars, hotbar, minimap with toggle and fullscreen, floating text, pause menu
- [x] Real-time combat with 5-state enemy AI (Idle/Patrol/Alert/Chase/Attack)
- [x] Damage numbers, screen shake, knockback feedback
- [x] Status effects: Poison, Burn, Freeze, Bleed, Stun, Regen, WellFed
- [x] Enchanted weapons (FlameBlade, FrostBlade, VenomBlade, LifestealBlade) with on-hit effects
- [x] Death screen with respawn and stats tracking
- [x] Save/load with full world serialization (serde + bincode), multiple named save slots, backwards-compatible
- [x] Save slot browser UI in main menu and pause menu
- [x] Six leveled skills (Gathering, Combat, Fishing, Farming, Crafting, Building) with XP
- [x] Farming with crop growth cycles tied to seasons and weather; cooked food gives stat buffs
- [x] Fishing with six fish types, biome variation, and cook/eat mechanics
- [x] Pet taming (Wolf, Cat, Hawk, Bear) with unique passive effects
- [x] Quest system with reward items and research points
- [x] NPC system: traders, hermits, dialogue, and quest givers
- [x] Automation structures: auto-smelter, crop sprinkler, alarm bell
- [x] Procedural world structures: Abandoned Villages, Mine Shafts, Trader Outposts, Watchtowers, Fishing Docks, Wolf Dens, Spider Nests, Scorpion Burrows
- [x] Procedural dungeon instances with rooms, corridors, enemies, and loot
- [x] Four seasons with visual palette shifts affecting farming, weather, and world objects
- [x] Weather system with rain, storms, and gameplay effects
- [x] Minimap with toggle and fullscreen mode
- [x] Particle effects for combat, gathering, and environment
- [x] Sound event system wired to combat, gathering, building, and crafting
- [x] Main menu with New Game, Continue (save slot browser), and Quit
- [x] Tutorial hint system for first-play contextual guidance
- [x] Controls overlay (in-game keyboard shortcut reference)
- [x] Persistent settings: resolution, fullscreen (F11), audio volume, gamepad support
- [x] Lore discovery system
- [x] Experiment system (item combination/discovery workbench)
- [x] Sprite animation system (atlas-backed frame cycling)
- [x] Debug performance overlay (chunk management, spatial grid, animation timing)
- [x] Custom WGSL shaders for lit_chunk and lit_sprite materials
- [x] Pixel art sprite assets across 37 asset directories

### Active

<!-- Remaining work for Early Access polish -->

- [ ] Full art pass polish on all biomes and assets (production quality)
- [ ] Audio file integration (SFX and ambient loops -- system is wired, files are placeholders)
- [ ] Performance optimization pass for Apple Silicon
- [ ] Bug fixing and balance tuning
- [ ] Steam / itch.io store page and release packaging

### Out of Scope

- Multiplayer / co-op -- single-player only, architectural complexity too high for solo dev
- Mobile platform -- macOS primary, Windows/Linux via cross-compilation later
- Complex farming (soil quality, fertilizer, irrigation) -- keeping farming light and optional
- Character levels -- progression is entirely gear and tool driven

## Context

- **Engine:** Bevy 0.15 with dynamic linking for fast dev builds
- **Rendering:** Custom chunk-based pipeline -- each chunk is 1 ECS entity with 1 mesh, tiles stored as data arrays (NOT individual entities). This is the critical architecture decision for tilemap performance.
- **Lighting:** Normal-mapped 2D lighting via custom WGSL shaders (lit_chunk.wgsl, lit_sprite.wgsl)
- **Tile size:** 16px, chunks are 32x32 tiles (512px world size per chunk)
- **Current state:** All core gameplay systems implemented. 47 source files, ~27K lines of Rust, 43 Bevy plugins. Pixel art sprite assets in 37 asset directories. Audio system wired but audio files are placeholders.
- **Dependencies:** bevy 0.15, noise 0.9, rand 0.8, serde 1, serde_json 1, bincode 1
- **Visual target:** Delverium / Stardew Valley HD pixel art aesthetic (48-64px tile resolution)

## Constraints

- **Tech stack**: Bevy + Rust (100%) -- committed, entire codebase
- **Platform**: macOS Apple Silicon primary -- 60 FPS minimum target
- **Performance**: Zero tile entities, <500 active game entities, <2GB RAM
- **Solo dev**: AI-assisted development, hobby pace with accelerated output
- **Bevy version**: Pinned at 0.15 per phase to avoid API churn

## Key Decisions

| Decision                                     | Rationale                                                                         | Outcome |
| -------------------------------------------- | --------------------------------------------------------------------------------- | ------- |
| Top-down 3/4 view (NOT isometric)            | Simple Y-sort depth, rectangular grids, axis-aligned collision, easier sprites    | Good    |
| Tiles as data, not entities                  | Avoids Bevy's known performance cliff at 20k+ entities                            | Good    |
| Custom tilemap renderer                      | No mature Bevy tilemap crate matches our architecture needs                       | Good    |
| Custom WGSL shaders for lighting             | Normal-mapped 2D lighting for point lights (campfires, torches)                   | Good    |
| Soft survival (no harsh punishment)          | Matches Stardew/Valheim feel, keeps game chill with moments of urgency            | Good    |
| No character levels (gear-based progression) | Simplifies systems, focuses on crafting/building as core loop                     | Good    |
| SpatialGrid for proximity queries            | 128px cells avoid raw ECS iteration for finding nearby entities                   | Good    |
| Max 15 plugins per add_plugins tuple         | Bevy tuple size limit; main.rs uses 3 tuples (12, 15, 11) plus 5 individual calls | Good    |
| serde(default) on all SaveData fields        | Backwards compatibility for save files as features are added                      | Good    |

---

_Last updated: 2026-04-01 -- full audit against 47 source files and 43 plugins_
