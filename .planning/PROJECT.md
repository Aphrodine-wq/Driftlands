# Driftlands

## What This Is

A top-down open-world sandbox game built as a solo project in Bevy (Rust). Players explore an infinite procedurally generated world with 9 biomes, build modular structures, master deep crafting systems, engage in light survival mechanics, and discover environmental lore — all through a top-down 3/4 perspective in the style of Stardew Valley and Delverium.

## Core Value

**Build Anything, Anywhere** — modular snap-together construction on a rectangular grid gives players full creative control over structures. The building system is the heart of the game.

## Requirements

### Validated

<!-- Shipped and confirmed working — Phase 1 Foundation complete -->

- ✓ Bevy project with custom top-down 3/4 view rendering pipeline — Phase 1
- ✓ Custom chunk-based tilemap renderer (tiles as data arrays, chunks as entities) — Phase 1
- ✓ Single biome (Forest) with Perlin noise procedural generation — Phase 1
- ✓ Player movement (WASD rectangular grid, 150 px/s, normalized diagonal) — Phase 1
- ✓ Camera follow with smooth lerp and mouse-wheel zoom (0.5x–3.0x) — Phase 1
- ✓ Dynamic chunk loading/unloading (render distance 5, 11x11 grid) — Phase 1
- ✓ World objects (oak/pine trees, rocks, bushes) with deterministic placement — Phase 1
- ✓ Click-to-gather system with object health and resource drops — Phase 1
- ✓ 36-slot inventory with 9-slot hotbar, stacking, and display toggle — Phase 1
- ✓ 10 hand-crafting recipes (sticks, planks, rope, campfire, tools, workbench, floor) — Phase 1
- ✓ Basic building placement (wood floor, grid-snapped, build mode toggle) — Phase 1
- ✓ Day/night cycle (600s, 4 phases, ambient darkness overlay) — Phase 1
- ✓ Full HUD (day counter, time, hotbar, inventory, crafting menu, keybind help) — Phase 1

### Active

<!-- Current scope — Phase 2 through Phase 5 -->

- [ ] Health and hunger survival systems with regeneration and debuffs
- [ ] Full modular building (walls, doors, roofs, stairs) in multiple material tiers
- [ ] Workbench crafting station with Tier 2 recipes (20+ total)
- [ ] Basic combat with night enemy spawning (shadow crawlers)
- [ ] Enemy AI (patrol, aggro, chase states)
- [ ] Death/respawn with corpse-run inventory drop
- [ ] Save/load system (serde + bincode world serialization)
- [ ] HUD completion (health bar, hunger bar, minimap, tool durability)
- [ ] All 9 biomes with unique resources, terrain, and creatures
- [ ] Dungeon generation with cave enemies and boss encounters
- [ ] Forge + Anvil crafting (Tier 3, 50+ recipes)
- [ ] Light farming system (till, plant, harvest, cook)
- [ ] 4 seasons with visual and gameplay effects
- [ ] Weather system (rain, snow, storms)
- [ ] Advanced crafting tiers 4-5 (100+ recipes), tech tree, blueprint discovery
- [ ] Boss monsters (one per biome with exclusive drops)
- [ ] Wandering traders and hermit NPCs
- [ ] Environmental lore (journals, ruins, ancient machinery)
- [ ] Full art pass on all biomes and assets
- [ ] Performance optimization for Apple Silicon

### Out of Scope

- Multiplayer / co-op — single-player only, architectural complexity too high for solo dev
- Mobile platform — macOS primary, Windows/Linux via cross-compilation later
- Complex farming (soil quality, fertilizer, irrigation) — keeping farming light and optional
- Quest system / quest markers — atmosphere over narrative, environmental storytelling only
- Character levels — progression is entirely gear and tool driven

## Context

- **Engine:** Bevy 0.15 with dynamic linking for fast dev builds
- **Rendering:** Custom chunk-based pipeline — each chunk is 1 ECS entity with 1 mesh, tiles stored as data arrays (NOT individual entities). This is the critical architecture decision for tilemap performance.
- **Tile size:** 16px, chunks are 32x32 tiles (512px world size per chunk)
- **Current state:** Phase 1 complete with all systems functional. Using colored rectangles as placeholder sprites. No art assets yet.
- **Dependencies:** bevy 0.15, noise 0.9, rand 0.8
- **Visual target:** Delverium / Stardew Valley HD pixel art aesthetic (48-64px tile resolution)
- **Art is the bottleneck:** Code can be AI-accelerated; pixel art cannot. Placeholder art for now.

## Constraints

- **Tech stack**: Bevy + Rust (100%) — committed, entire codebase
- **Platform**: macOS Apple Silicon primary — 60 FPS minimum target
- **Performance**: Zero tile entities, <500 active game entities, <2GB RAM
- **Solo dev**: AI-assisted development, hobby pace with accelerated output
- **Bevy version**: Pinned at 0.15 per phase to avoid API churn

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Top-down 3/4 view (NOT isometric) | Simple Y-sort depth, rectangular grids, axis-aligned collision, easier sprites | ✓ Good |
| Tiles as data, not entities | Avoids Bevy's known performance cliff at 20k+ entities | ✓ Good |
| Custom tilemap renderer | No mature Bevy tilemap crate matches our architecture needs | — Pending |
| Placeholder colored rectangles | Unblocks all gameplay development while art pipeline is TBD | ✓ Good |
| Soft survival (no harsh punishment) | Matches Stardew/Valheim feel, keeps game chill with moments of urgency | — Pending |
| No character levels (gear-based progression) | Simplifies systems, focuses on crafting/building as core loop | — Pending |

---
*Last updated: 2026-03-02 after initialization*
