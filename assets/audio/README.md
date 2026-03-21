# Audio assets

Place OGG Vorbis (`.ogg`) files here for in-game sound. Paths are relative to the `assets/` directory.

## SFX (one-shots)

In `assets/audio/sfx/`:

| File | When played |
|------|-------------|
| `hit.ogg` | Melee/ranged hit on enemy |
| `gather.ogg` | Harvesting resources (throttled) |
| `build.ogg` | Placing a structure |
| `craft.ogg` | Recipe completed |
| `pickup.ogg` | Picking up dropped item |
| `menu_open.ogg` | Opening menu / dungeon |
| `death.ogg` | Player or boss death |
| `boss_roar.ogg` | Boss aggro / attack |
| `place_invalid.ogg` | Invalid building placement click |
| `trade.ogg` | Trade completed with trader |
| `discovery.ogg` | First time entering a biome |
| `lore_complete.ogg` | All 20 journal lore entries collected |

Short clips (0.1–0.5 s) work well for SFX. If a file is missing, that event is simply not heard (no crash).

## Optional: ambient loops

Ambient events (`AmbientDay`, `AmbientNight`, `AmbientRain`, `AmbientStorm`) are sent by the game but not yet wired to looping tracks. Future: add looping assets and play them with `PlaybackSettings::LOOP`.
