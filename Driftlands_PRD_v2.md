# DRIFTLANDS

**A Top-Down Open-World Sandbox**

*Product Requirements Document — Version 2.0 | March 2026*
*Solo Developer | Bevy + Rust | macOS*

---

| Property | Value |
|---|---|
| Working Title | Driftlands |
| Genre | Top-Down Open-World Sandbox / Survival-Craft |
| Engine | Bevy (Rust) with custom chunk-based tilemap renderer |
| Camera | Top-down ¾ view (Stardew / Zelda style) — NOT isometric |
| Platform | macOS (primary), Windows/Linux planned |
| Multiplayer | Single-player only |
| Team | Solo developer (AI-assisted development) |
| Timeline | Accelerated hobby-pace, AI-augmented workflow |
| Monetization | Early Access (itch.io / Steam) leading to full release |
| Visual Target | Delverium / Stardew Valley HD pixel art aesthetic |

---

## 1. Executive Summary

Driftlands is a top-down, open-world sandbox game built as a solo project using Bevy and a full Rust technology stack. The game combines procedural world generation, modular base-building, deep crafting systems, light farming, and atmospheric exploration into a cohesive experience. Players take on the role of a lone wanderer shaping an infinite, procedurally generated world one block, structure, and discovery at a time.

The game draws inspiration from Minecraft, Stardew Valley, Valheim, Factorio, and Delverium while carving its own identity through HD pixel art visuals, a fixed top-down ¾ perspective, environmental lore-driven storytelling, and a gear-based progression system with no character levels.

Development leverages AI-assisted coding workflows to dramatically accelerate the solo development timeline. AI tools are used for code generation, asset pipeline automation, system prototyping, and iterative debugging, allowing a single developer to achieve output comparable to a small team.

---

## 2. Game Vision & Design Pillars

### 2.1 Player Fantasy

The player is a Creative Architect — someone who shapes the world around them. Every tree felled, every wall placed, every crafting chain mastered, and every ruin explored feeds back into the core loop of transforming a wild, procedurally generated landscape into something uniquely theirs.

### 2.2 Core Design Pillars

**Build Anything, Anywhere:** Modular construction with snap-together pieces (walls, floors, roofs, stairs) gives players full creative control over structures. The building system is the heart of the game.

**Discover Through Exploration:** An infinite procedurally generated world with 9 distinct biomes rewards curiosity. Ruins hold blueprints, caves hide rare ores, and every direction offers something new.

**Master Deep Crafting:** 100+ recipes organized across workbenches, tech trees, blueprint discovery, and experimentation. Crafting chains grow in complexity as players progress through gear tiers.

**Earn Your Survival:** Soft survival mechanics (food buffs, tool durability, corpse runs on death) create tension without punishment. The world is dangerous at night and underground, but never unfair.

**Atmosphere Over Narrative:** Environmental storytelling through ruins, journals, and ancient artifacts. No quest markers, no cutscenes — just a world that whispers its history to those who look closely.

### 2.3 Pacing

Moderate pacing — predominantly chill with moments of urgency. Daytime is peaceful (gather, build, farm, explore). Night brings danger. Dungeons and boss encounters provide adrenaline spikes. The player controls their own pace by choosing when to venture into dangerous territory.

---

## 3. Camera & Perspective

> **CRITICAL CORRECTION FROM v1.0:** Driftlands uses a top-down ¾ view perspective, NOT isometric. This matches the visual style of Delverium, Stardew Valley, classic Zelda, and Graveyard Keeper. The camera looks almost straight down with a slight vertical offset that shows the front faces of objects and buildings.

### 3.1 Why Top-Down ¾ View (Not Isometric)

The v1.0 PRD incorrectly specified a fixed isometric perspective. Isometric projection uses a diamond-shaped tile grid with complex coordinate transforms, making collision detection, depth sorting, and sprite creation significantly harder. The Delverium visual target uses a rectangular grid top-down view, which provides major development advantages: simple Y-sort depth ordering, straightforward rectangular tile grids, axis-aligned collision detection, and easier sprite art creation without isometric angle distortion.

### 3.2 Camera Specifications

Fixed top-down ¾ perspective with no rotation. Zoom in/out supported (3 zoom levels minimum). The camera follows the player with smooth interpolation and configurable dead zone. Transparent roof rendering when the player is inside buildings. Underground and cave areas use a separate rendering layer with limited light radius.

---

## 4. World Generation

### 4.1 World Structure

The world is infinite and chunk-based, generating new terrain as the player explores outward. Each new game produces a unique world from a random seed (or player-provided seed for shareable worlds). The world should feel vast — requiring multiple real-world days of play to see a meaningful portion of the generated content.

### 4.2 Chunk System

| Parameter | Specification |
|---|---|
| Chunk Size | 32×32 tiles per chunk (tunable during development) |
| Load Radius | Active simulation within 3–5 chunks of the player |
| Render Radius | Visual rendering within 7–9 chunks |
| Generation | Async chunk generation on background threads (non-blocking) |
| Persistence | Modified chunks saved to disk; unmodified chunks regenerated from seed |
| Tile Storage | Tiles stored as data arrays inside chunk entities, NOT as individual ECS entities |

> **CRITICAL ARCHITECTURE NOTE:** Each chunk is a single Bevy ECS entity containing a data array of tile information and a single mesh for rendering. Individual tiles are NEVER ECS entities. This is the key architectural decision that ensures Bevy performs well for large tilemap games. The naive approach of making each tile an entity causes severe performance degradation at 20,000+ entities.

### 4.3 Biomes

Nine biomes distributed via a temperature/moisture Whittaker diagram overlaid on Perlin noise maps. Each biome features unique terrain, vegetation, resources, creatures, ambient audio, and weather patterns.

| Biome | Key Resources | Unique Features |
|---|---|---|
| Forest | Oak, birch, pine, mushrooms, berries | Dense canopy, woodland creatures, fallen logs |
| Coastal/Beach | Sand, shells, driftwood, salt, seaweed | Tidal mechanics, shipwrecks, coral deposits |
| Swamp | Peat, reeds, mud clay, toxic herbs | Fog, slow movement, poisonous gas pockets |
| Desert | Sandstone, cactus fiber, quartz, fossils | Heat shimmer, oasis pools, buried ruins |
| Tundra | Ice crystals, frozen ore, arctic herbs | Blizzards, frozen lakes, ice caves |
| Volcanic | Obsidian, sulfur, magma shards, rare metals | Lava flows, eruption events, heat damage zones |
| Mushroom/Fungal | Giant mushroom wood, spores, biolum. gel | Glowing environment, unique crafting reagents |
| Crystal Caves | Crystal shards, gemstones, echo stone | Underground only, light refraction puzzles |
| Mountain/Alpine | Granite, high-grade ore, alpine flowers | Elevation, cliff faces, thin air (slower stamina) |

### 4.4 Verticality

The world features multiple vertical layers: underground caverns and dungeon systems, surface terrain, and elevated terrain (mountain peaks, raised plateaus). Mining and digging allow access to underground resources. Building upward is supported through multi-story modular construction. Vertical layers are rendered as separate tilemap layers with occlusion handled by the rendering system.

### 4.5 Procedural Structures

**Ruins:** Procedurally generated using Wave Function Collapse or template stitching. Contain lore journals, blueprint fragments, and treasure chests.

**Dungeons:** Multi-room underground structures with enemies, traps, and a boss encounter. Difficulty scales with distance from world origin.

**Points of Interest:** Abandoned campsites, hermit huts, ancient machinery, natural formations (waterfalls, hot springs, geysers).

---

## 5. Core Gameplay Systems

### 5.1 Resource Gathering

Gathering is the foundational activity. Players interact with the world to collect raw materials that feed into the crafting system. Tree chopping yields different wood types per biome, with trees falling using directional physics and breaking into log segments. Loose stones can be picked up by hand while rock formations require a pickaxe. Berries, mushrooms, herbs, and other biome-specific plants are gathered by hand. Underground mining reveals ore veins, crystal deposits, and fossil layers with deeper areas yielding rarer materials.

### 5.2 Building System

The building system uses modular snap-together pieces placed on the rectangular tile grid. Structures are composed of individual elements that connect at predefined snap points. Because we use a top-down ¾ view with a rectangular grid, building placement is straightforward grid-snapping without the coordinate complexity of isometric systems.

**Foundations & Floors:** Base layer, required before walls. Multiple material variants (wood, stone, brick, metal).

**Walls:** Full walls, half walls, walls with windows, doorframes. Snap to floor edges. Drawn with front-face visible in the ¾ perspective.

**Roofs:** Flat, sloped, peaked. Snap to wall tops. Required for weather protection. Become transparent when player is inside.

**Stairs & Ladders:** Vertical traversal between floors.

**Furniture & Decor:** Functional and cosmetic items placed inside structures (chests, tables, lighting, crafting stations).

**Doors & Gates:** Interactive elements that block enemy pathfinding when closed.

Each building piece exists in multiple material tiers (wood, stone, brick, reinforced stone, metal), with increasing durability and visual quality. Higher tiers require more advanced crafting stations.

### 5.3 Crafting System

Deep crafting with 100+ recipes, organized into tiers and discovered through multiple methods: crafting stations reveal associated recipes, a tech tree unlocked by spending research points earned through exploration and crafting, blueprint discovery in ruins and dungeons, and experimentation by combining unexpected items with a chance to discover hidden recipes.

| Tier | Station | Example Outputs | Key Materials |
|---|---|---|---|
| 1 – Primitive | Hand crafting | Stick tools, campfire, thatch shelter | Sticks, stones, plant fiber |
| 2 – Basic | Workbench | Wooden tools, chest, basic walls | Wood planks, rope, flint |
| 3 – Intermediate | Forge + Anvil | Metal tools, stone buildings, armor | Iron ore, coal, stone blocks |
| 4 – Advanced | Adv. Forge + Lab | Steel gear, alchemy, machines | Steel alloy, crystals, rare herbs |
| 5 – Master | Ancient Workstation | Endgame tools, automation, ancient tech | Ancient cores, obsidian, gems |

### 5.4 Farming

Farming is a light, optional system. Players can till soil, plant seeds found through foraging, and harvest crops over time. Farming is not required for survival but provides the most efficient source of food buffs. Different crops thrive in different seasons; planting off-season results in no growth. No soil quality, no fertilizer, no irrigation — plant it, it grows, you eat it. Harvested crops can be cooked at a campfire or kitchen station for enhanced food buffs.

### 5.5 Survival Mechanics

Soft survival — the world encourages you to eat and prepare, but never punishes harshly for neglecting needs. A hunger meter depletes over time, causing slower movement and gathering when empty. Eating food restores the meter and provides temporary buffs. Health is damaged by enemies, environmental hazards, and falls, regenerating slowly when well-fed. Tools degrade with use and require repair or replacement. On death, the player respawns at their last bed/spawn point and drops all inventory at the death location, creating a corpse-run mechanic.

### 5.6 Seasons & Weather

| Season | Weather | Gameplay Effect | Visual Changes |
|---|---|---|---|
| Spring | Light rain, occasional storms | Fastest crop growth, new forage spawns | Flowers bloom, bright greens |
| Summer | Hot, dry, rare thunderstorms | Drought risk, desert biome expands | Saturated colors, heat shimmer |
| Autumn | Fog, cool rain, wind | Harvest season, mushroom boom | Orange/red foliage, falling leaves |
| Winter | Snow, blizzards, ice | No crops, frozen water, longer nights | Snow cover, bare trees, ice on water |

---

## 6. Combat System

### 6.1 Combat Style

Simple click-to-attack combat. The player faces the nearest enemy or cursor direction and attacks with their equipped weapon on click. Combat is deliberately simple to keep the focus on building and exploration, but is consequential due to the corpse-run death penalty. The top-down ¾ view provides clear visibility of enemy positions and attack patterns, similar to Delverium's combat feel.

### 6.2 Weapons & Armor

Melee weapons include swords, axes (dual-purpose with woodcutting), maces, and spears, each with different speed, damage, and range tradeoffs. Ranged weapons include bows and crossbows requiring crafted ammunition. Shields are off-hand items that reduce damage when held. Armor covers helmet, chest, legs, and boots, providing passive damage reduction scaled by material tier.

### 6.3 Enemy Encounters

| Context | Behavior | Examples |
|---|---|---|
| Night Surface | Spawn at dusk, despawn at dawn. Patrol and aggro. | Shadow crawlers, feral wolves, skeletons |
| Dungeons/Caves | Always present. Guard rooms and treasure. | Cave spiders, fungal zombies, stone golems |
| Biome-Specific | Unique creatures tied to biome type. | Lava elementals, ice wraiths, bog lurkers |
| Boss Monsters | Guard rare loot in dungeon depths or landmarks. | One unique boss per biome. Exclusive drops. |

---

## 7. NPCs & Narrative

### 7.1 Wandering Traders

Rare NPC encounters in the overworld. Wandering traders appear randomly and offer to buy/sell items. Each trader has a specialty (tools, rare seeds, exotic materials, blueprints). They stay in an area for 1–2 in-game days before moving on. They cannot be harmed by the player.

### 7.2 Hermits

Solitary NPCs found at fixed procedurally-placed huts in remote biomes. Each hermit has unique dialogue that hints at world lore and may offer a one-time trade for a rare item or blueprint. Finding all hermits across the world is an implicit completionist goal.

### 7.3 Environmental Storytelling

Journal pages found in ruins, dungeons, and hermit huts piece together the world's history. Ancient machinery scattered in ruins can be repaired with endgame materials. Environmental clues like battlefield remnants, overgrown roads, and collapsed structures provide visual storytelling embedded in world generation.

---

## 8. Progression & Endgame

### 8.1 Progression Model

No character levels. Progression is entirely gear and tool driven. The player grows stronger by crafting better equipment, discovering blueprints for advanced items, and unlocking higher-tier crafting stations. Knowledge (recipes, lore) persists; power is tied to what you carry and what you have built.

### 8.2 Gear Tiers

Five material tiers for tools, weapons, and armor: Primitive (sticks/stone), Basic (refined wood/flint), Intermediate (iron/steel), Advanced (alloys/crystals), Master (ancient materials). Each tier is a meaningful upgrade in efficiency and durability.

### 8.3 Endgame

No formal endgame. Driftlands is an infinite sandbox. Implicit long-term goals include discovering all biomes, defeating all bosses, collecting all journal pages, crafting all recipes, building elaborate structures, and repairing ancient machinery.

---

## 9. Art Direction & Audio

### 9.1 Visual Style

HD pixel art at 48–64px tile resolution with detailed sprites, matching the Delverium aesthetic. Rich color palettes that shift dramatically between biomes and seasons. Dynamic time-of-day lighting (sunrise, midday, sunset, night) with ambient particle effects (fireflies, falling leaves, snow, rain, dust). The visual target is Delverium's polished top-down pixel art style — warm, detailed, and inviting with clear readability at all zoom levels.

### 9.2 Audio Direction

Chill, ambient soundtrack in the style of Stardew Valley and Animal Crossing. Gentle acoustic and piano melodies during daytime, subdued atmospheric tones at night, and tension-building tracks for dungeons and boss encounters. Biome-specific environmental audio (birdsong in forests, waves on coasts, wind in mountains) with seamless crossfading. Satisfying, tactile sound effects for gathering, building, crafting, and combat.

---

## 10. UI & Controls

### 10.1 Controls

WASD movement on a rectangular grid (NOT isometric diamond-grid translation). Mouse click to interact, attack, and place building pieces. Number keys 1–9 for hotbar, Tab/I for inventory, B for build mode, M for map with fog of war. All controls designed around the top-down ¾ perspective where screen-space and world-space axes are aligned.

### 10.2 HUD Elements

Health bar (top-left), hunger meter (below health), hotbar (bottom-center, 9 slots), minimap (top-right), day/night and season indicator (near minimap), and tool durability (on active hotbar item). Clean, pixel-art styled UI that matches the game's visual identity.

---

## 11. Technical Architecture

### 11.1 Technology Stack

| Component | Technology | Notes |
|---|---|---|
| Language | Rust (100%) | Entire codebase |
| Game Engine | Bevy (ECS architecture) | Application framework, NOT tilemap renderer |
| Tilemap Renderer | Custom chunk-based pipeline | Each chunk = 1 entity, 1 mesh, tiles as data arrays |
| GPU | wgpu (via Bevy) | Metal on macOS, Vulkan/DX12 for future platforms |
| Noise Generation | noise-rs or bracket-noise | Perlin/Simplex for terrain generation |
| Structure Gen | Wave Function Collapse | Custom or wfc crate |
| Serialization | serde + bincode | World saves and configuration |
| Database | rusqlite | Recipe/item/lore databases |
| Audio | kira (via bevy_kira_audio) | Music, ambient, SFX |
| UI Framework | bevy_egui (dev tools) | Custom sprite-based UI for gameplay |
| Target OS | macOS (Apple Silicon native) | Windows/Linux via cross-compilation later |

### 11.2 Architecture Overview

The game uses Bevy's Entity Component System for game object management (player, enemies, NPCs, dropped items, particles) while implementing a custom chunk-based rendering system for the tilemap. This hybrid approach gets the benefits of Bevy's ECS for gameplay logic, input handling, audio, and asset management while avoiding the known performance pitfall of making tiles into entities.

Key architecture: World Generation System (async chunk generation from seed), Chunk Management System (load/unload based on player position, save modified chunks), Custom Tile Renderer (sprite batching with texture atlases, Y-sort depth ordering), Physics/Collision System (rectangular tile-based collision, AABB for combat), AI System (state-machine driven enemy AI), Crafting System (recipe registry from data files), Save System (chunk-based world saves via serde/bincode).

### 11.3 Performance Targets

| Metric | Target |
|---|---|
| Frame Rate | 60 FPS minimum on Apple Silicon Macs |
| Load Time | < 3 seconds from menu to gameplay |
| Chunk Generation | < 50ms per chunk (async, non-blocking) |
| Memory | < 2 GB RAM for standard play session |
| Save File Size | < 100 MB for heavily explored world |
| Active Tile Entities | ZERO — tiles are data, not entities |
| Active Game Entities | < 500 at any time (enemies, NPCs, items, particles) |

---

## 12. AI-Augmented Development Strategy

This section is new to v2.0 and reflects the reality that modern solo development with AI assistance fundamentally changes the development calculus. AI coding tools (Claude, Copilot, Cursor, etc.) are not just helpers — they are force multipliers that collapse the traditional timeline for a project of this scope.

### 12.1 What AI Accelerates

**Boilerplate & Systems Code:** ECS component definitions, system registration, resource loading, serialization, input mapping, UI layout — all of this can be generated rapidly with AI assistance. What might take days of manual coding can be produced in hours.

**Procedural Generation Algorithms:** Noise-based terrain generation, Whittaker diagram implementation, WFC structure generation, dungeon layout algorithms. AI can produce working implementations of well-documented algorithms quickly.

**Data-Driven Content:** Recipe definitions, item databases, biome configurations, enemy stat tables, loot tables. AI excels at generating structured data from specifications.

**Shader & Rendering Code:** Custom sprite batching, dynamic lighting, particle effects, seasonal palette swaps. WGSL/GLSL shaders can be iterated on rapidly.

**Debugging & Optimization:** AI can analyze performance profiles, suggest optimizations, identify memory leaks, and help with the specific kind of systems-level debugging that Rust projects require.

### 12.2 What AI Cannot Replace

**Game Feel & Tuning:** The "juice" that makes chopping a tree satisfying, the weight of combat, the pacing of exploration — this requires human playtesting and iteration. AI can generate code, but only you can feel whether it's fun.

**Art Direction:** Pixel art sprites, tilesets, character animations, UI design. AI image generation is not yet reliable for consistent, production-quality game pixel art at this fidelity. This remains the primary bottleneck.

**Audio & Music:** Original compositions, ambient soundscapes, satisfying SFX. While AI audio tools exist, curating and integrating audio still requires significant human judgment.

**Architecture Decisions:** High-level system design, ECS component architecture, chunk rendering strategy, save system design. AI can implement architectures but the developer must define them.

**Bevy Ecosystem Navigation:** Bevy's API changes frequently. AI training data may lag behind the latest Bevy version. The developer must verify generated code against current Bevy documentation.

### 12.3 Realistic AI-Augmented Timeline

With disciplined use of AI coding tools, the development timeline compresses significantly compared to traditional solo development. However, compression is uneven — code-heavy phases accelerate dramatically while art, audio, and tuning phases see modest improvement.

| Phase | Traditional Solo | AI-Augmented Solo | Bottleneck |
|---|---|---|---|
| Phase 1: Foundation | 2–4 months | 2–4 weeks | Bevy learning curve, rendering pipeline |
| Phase 2: Core Loop | 3–5 months | 4–8 weeks | Game feel tuning, combat balance |
| Phase 3: World Expansion | 4–6 months | 6–10 weeks | Art assets for 9 biomes |
| Phase 4: Depth & Polish | 4–6 months | 6–10 weeks | Content volume, boss design, audio |
| Phase 5: Early Access | 2–3 months | 4–6 weeks | Art polish, QA, platform testing |
| **TOTAL** | **15–24 months** | **6–10 months** | **Art pipeline is the limiting factor** |

The critical path is NOT code — it's art. A developer spending focused weekends and evenings with AI-assisted coding could realistically have a playable, feature-complete prototype within 4–6 months. Reaching Early Access quality (with polished art, audio, and content) depends almost entirely on how the art pipeline is solved: placeholder art with asset packs for EA, commissioned pixel art, or learning to create it personally.

### 12.4 AI Workflow Recommendations

**Session-Based Development:** Define clear goals for each coding session. Provide AI with the PRD context, current codebase state, and specific implementation targets. AI works best when given well-scoped tasks with clear acceptance criteria.

**Iterative Architecture:** Use AI to generate initial system implementations, then refine through testing and profiling. Don't try to get perfect code on the first pass — iterate.

**Test Generation:** Have AI generate unit tests alongside implementation code. Rust's type system catches many bugs, but game logic (crafting recipes, biome generation, save/load) benefits from explicit testing.

**Documentation as Code:** Keep the PRD and technical design docs current. AI performs dramatically better when it has up-to-date context about the project's architecture and goals.

---

## 13. Development Roadmap

Organized into vertical slices. Each phase produces a playable build. AI-augmented timelines are estimates assuming focused development sessions with AI coding assistance.

### Phase 1: Foundation (Vertical Slice)

**Target: 2–4 weeks AI-augmented**

- Bevy project setup with custom top-down ¾ view rendering pipeline
- Custom chunk-based tilemap renderer (tiles as data, chunks as entities)
- Single biome (Forest) with chunk-based procedural generation
- Player movement (WASD rectangular grid), basic camera follow with zoom
- Tree chopping, stone pickup, basic inventory
- Hand-crafting (Tier 1) with 5–10 recipes
- Place and break a single building piece (wood floor)
- Day/night cycle with basic lighting

### Phase 2: Core Loop

**Target: 4–8 weeks AI-augmented**

- Workbench crafting station, Tier 2 recipes (20+ recipes)
- Full modular building system (walls, floors, roofs, doors)
- Basic combat (1–2 night enemy types)
- Death/respawn with corpse-run inventory drop
- Health and hunger systems
- Save/load system
- HUD (health, hunger, hotbar, minimap)

### Phase 3: World Expansion

**Target: 6–10 weeks AI-augmented**

- All 9 biomes with unique resources and terrain
- Biome-specific creatures and enemies
- Dungeon generation with cave enemies
- Forge + Anvil crafting (Tier 3, 50+ recipes total)
- Light farming system
- 4 seasons with visual and gameplay effects
- Weather system (rain, snow, storms)

### Phase 4: Depth & Polish

**Target: 6–10 weeks AI-augmented**

Advanced crafting tiers (Tier 4–5, 100+ recipes), tech tree and blueprint discovery, experimentation crafting, boss monsters (one per biome), wandering traders and hermit NPCs, environmental lore (journals, ruins, ancient machinery), full audio implementation.

### Phase 5: Early Access Release

**Target: 4–6 weeks AI-augmented**

Full art pass on all biomes and assets, UI/UX polish and accessibility options, performance optimization for Apple Silicon, bug fixing and balance tuning, Steam / itch.io store page and release.

---

## 14. Risks & Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Scope creep (9 biomes ambitious solo) | Delayed or unfinished project | Phase biomes in gradually. Forest first, add 1–2 per phase. |
| Bevy ecosystem immaturity | Missing features, breaking API changes | Pin Bevy version per phase. Use custom renderer for tilemap. |
| Art pipeline (solo dev creating HD pixel art) | Art bottleneck slows all progress | Use placeholder art early. Commission sprites or use asset packs for EA. |
| Infinite world performance | Memory/CPU issues at scale | Chunk architecture with tiles-as-data. Profile early and often. |
| AI-generated code quality | Subtle bugs, non-idiomatic patterns | Review all generated code. Test thoroughly. Profile performance. |
| Bevy API drift vs AI training data | Generated code may use outdated Bevy APIs | Always verify against current Bevy docs. Pin Bevy versions. |
| Hobby pace = motivation risk | Project abandonment | Keep phases small and playable. Share progress publicly. |
| macOS-only limits audience | Small player base for EA revenue | Rust + Bevy cross-compile easily. Add Windows/Linux later. |

---

## 15. Success Metrics

### 15.1 Pre-Release

Playable build after each phase that is fun to play for 30+ minutes. Consistent development cadence — at least one meaningful commit per week. Community interest through devlog audience (Twitter/Reddit/YouTube) before EA launch.

### 15.2 Early Access

Player retention with average play session exceeding 2 hours. Positive review ratio above 80% on Steam. Revenue sufficient to justify continued development (covers asset costs, tools, and time).

---

*End of Document — Driftlands PRD v2.0 — Corrected & AI-Augmented*
