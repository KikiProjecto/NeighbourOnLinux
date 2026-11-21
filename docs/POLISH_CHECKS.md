# Integrated polish checks — November 21, 2026

The game defaults entirely to English: item and room names, interaction prompts, clues, objectives, awareness labels, controls, pause/capture/completion screens, and CLI help. No language setting is required. The compact HUD takes visual direction from the user-provided [Interface In Game reference](https://interfaceingame.com/games/hello-neighbor/), particularly its small inventory and unobstructed play view. Reference images are not bundled as game assets.

## Working household interactions

There are **37 authored interaction targets across 14 types**, in addition to keys, boxes, locked doors, machinery switches and the tower ladder.

| Type | Visible response | Gameplay / audio connection |
| --- | --- | --- |
| Television | Animated scan lines; screen switches on/off | Hiss loop; attracts the neighbor while on |
| Telephone | Receiver assembly vibrates | Six-second ringing distraction; retriggerable |
| Lamps | Bulb material and actual point-light intensity change | Switch sound; restored on retry |
| Kitchen / dresser drawers | Tray, front and handle slide together | Visible note and progression clue |
| Cupboard | Two leaves rotate about edge hinges | Reveals a note; no hiding mechanic |
| Books | Cover opens about its spine; page lines visible | Floor-specific clue and reading cue |
| Apples | Stem/leaf detail and damped wobble | Small interaction cue |
| Paintings | Frame, canvas and image rotate together | Clue connects colored keys to the power chain |
| Sink | Water under the faucet | Running-water loop and repeated AI noise |
| Shower | Valve and pooled falling droplets | Running-water loop and repeated AI noise |
| Kettle | Spout, handle and rising steam | Steam/hiss loop follows the control |
| Mailbox | Flag lowers/raises | Letter clue and interaction cue |
| Doorbell | Button depresses briefly | Bell and immediate neighbor hearing |
| Tower dial | Needle cycles through three positions | Windmill/generator/freezer clues and control cue |

Storage bodies retain static collision. Opening drawers/cupboards does not create a traversable hiding space. Household animation uses unscaled parent pivots and eased amounts; it freezes on pause and resets on retry. Sources use a finite twelve-particle pool for water/steam.

## Character, machines and environment

- Original procedural character: rounded torso/head, ears, nose, mustache, hair, eyes/eyelids, patterned vest, collar, belt/buckle, rolled sleeves, gloves, trouser cuffs, shoes/soles. Shoulder/hip joints carry their attached hands and footwear. Gait follows actual movement; facing blends; idle breathing, blinking and alert posture pause correctly.
- Locked door leaves swing around hinges. Basement boards fall away when removed. Collision remains governed by puzzle state.
- Windmill and trolley accelerate/coast down. The trolley follows a closed 113.2 m rectangular route without the former corner coordinate mismatch. Presentation resets on retry.
- Shared small procedural surface maps, material-specific roughness, lamp bloom, slight foliage sway, sink rim/faucet detail, and shaped listening horns.
- Hard landings and thrown-box impacts emit short-lived dust and impact sound. Dust is capped at 128 concurrent entities and expires after half a second.
- Wind, night insects, house hum, generator, windmill, trolley and appliance loops follow clock/location/progression/control state. Distance and wall attenuation are tested. Managed one-shots and loops pause with the game. These are mono sources, not a room-acoustics or HRTF implementation.
- A separate original stereo score now has thirteen situation themes with two arrangements each. It switches for outdoor time of day, indoor exploration, active machinery, disabled power, underground exploration, suspicion, chase, search, capture and completion. Crossfades, stable-state timing, cue ducking, bounded music players, pause/retry and M to mute are integrated. See [MUSIC.md](MUSIC.md) for the exact behavior and independent category volumes.
- Four compact equipment indicators, separate colored-key flags, a responsive reticle, and H to reveal the objective and full controls. The indicators do not implement a selectable inventory.

## Underground, controls and visual pass

- A deterministic 48.4 × 48.4 m labyrinth with twelve furnished chambers, 120 maze cells and a reserved staircase cell. Real floor/soil/ceiling openings and 18 descending steps connect to the existing basement gate.
- Three restored circuits release the service-key case. The service exit completes the run. Breaker levers, indicators, chamber lights, key-case lid and ventilation fan respond to progression.
- English room signs, continuous colored floor cables, readable records and an entrance maintenance board. Reading the board unlocks a Tab map with the player position and objectives.
- The actual player controller walks every underground navigation edge in both directions. Tests check descent/ascent, standing control approaches, actual E/Tab behavior with authored collision, pause/retry and actual neighbor traversal to a distant chamber.
- Neighbor direct routes account for body width. Periodic replanning keeps forward progress between widely spaced maze nodes. The shared controller can step onto stacked soil/floor thresholds.
- Camera and neighbor rendering interpolate fixed positions; long teleports snap. FXAA replaces 4× MSAA, shadows use two cascades at 75 m, unchanged prop/UI values avoid repeated writes, and small sky images are precomputed at startup.
- The clock is absent from the HUD. A full default day takes 120 seconds. H opens explicit running/throwing/control instructions; carrying a box adds G/Q hints.
- Exterior planting, layered distant ridges, cloud-painted sky and restrained color grading enrich the outdoor view. Underground lighting/fog and music/ambience distinguish the new chambers. Irrelevant underground meshes/lights are hidden for rendering; collision stays active.

## Verification

Native checks:

```sh
cargo fmt --manifest-path app/Cargo.toml --check
cargo clippy --manifest-path app/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path app/Cargo.toml --locked
cargo build --manifest-path app/Cargo.toml --locked
./app/target/debug/hello-neighbor --self-test
```

46 tests cover the existing progression, stair movement, collision and AI, plus every household target's clear approach and actual E handler, TV hearing, pause/retry, bounded effect lifecycle, character pause behavior, loop PCM integrity, sound distance/occlusion, and trolley route continuity. Score checks exercise situation priorities, remote-distraction/safe-street behavior, doorway stability, urgent crossfades, three simulated minutes of musical variation, duck/mute/retry, real ECS deck cleanup and Bevy decoding of every generated stereo arrangement. CLI checks cover independent category volumes. The expanded checks also cover the two-minute clock, motion interpolation, retry collision snapshots and the underground features listed above. The self-test reports 3,019 authored pieces, 778 colliders and 34 named room volumes before the runtime interaction hierarchies are expanded. The house register includes areas beyond these room volumes.

Native Vulkan rendering was exercised on the local Intel Iris Xe. A 180-frame run with audio enabled exited successfully with no audio initialization errors. The subsequent score-enabled native run completed 9,000 frames over 56.5 seconds, exited 0 and logged afternoon arrangement 1 → 2 → 1 with no reported audio errors. A live PipeWire output stream was observed during that run. Sound content is also checked as PCM data and decoded through Bevy. Subjective audio balance has not been established by listening tests. Flatpak packaging was pending during this native polish pass and was subsequently built and verified; see [PERFORMANCE.md](PERFORMANCE.md).

Inspection views are saved under [screenshots](screenshots/): morning, afternoon, evening, night, workshop, archive, tower, neighbor, active kitchen/bathroom/gallery/living-room scenes, and basement/laboratory/evidence-vault/controls-and-map views. The complete 16-image suite was refreshed from the final native binary after quiet mode was changed to skip audio-device initialization. The captures are 1280×720. `--showcase` enables household props for these render inspections and requires `--view` plus `--screenshot`. It does not bypass normal puzzle progression.

See [PERFORMANCE.md](PERFORMANCE.md) for native encounter measurements and their limits; a passing build does not establish that every machine will hold a particular frame rate.

## Remaining scope

This is still a procedural fan prototype. Fear challenges, a rideable trolley, wardrobe hiding, selectable inventory, keyhole peeking, full menu/settings flow, the original hot-crowbar/magnet/cooling puzzle and final authored assets remain unfinished. The three-position dial is a clue control, not a room-listening minigame. See [HOUSE_DESIGN.md](HOUSE_DESIGN.md) for the broader house plan and implementation boundaries.
