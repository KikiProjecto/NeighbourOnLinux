# Bevy implementation guide

## Ownership and schedules

| State | Owner | Writers |
| --- | --- | --- |
| Player position, velocity, look, crouch | `Player` component | Input sampling, fixed movement, explicit tower climb / restart |
| Neighbor perception, route, patrol, memory | `Neighbor` component | Fixed neighbor simulation, noise interactions, restart |
| Geometry, room volumes, navigation graph | `HousePlan` resource | Startup authoring in `house.rs` |
| Static + dynamic collision snapshot | `CollisionWorld` resource | Fixed collision rebuild |
| Held / collected objects | `Pickup` component + `Inventory` resource | Interaction and fixed object motion |
| Door / machine dependencies | `PuzzleProgress` resource | Interaction, restart |
| Run result, pause, notice, attempts | `Session` resource | Controls, gameplay, presentation notice timer |
| Day, hour, day length | `WorldClock` resource | Atmosphere plugin |
| Render transforms, materials, visibility | Render entities | Presentation systems |
| Household controls, animation amount, timers | `InteractiveProp` and `PropPart` components | Props interaction / presentation, attempt reset |
| Character pose | `Pose` resource and `Joint` components | Character presentation; gait follows actual displacement |
| Dust and foliage | `Dust` / `LeafSway` components | Effects presentation; impact cues / bounded lifetime |
| Short audio | `SoundCue` messages → Bevy `AudioPlayer` entities | Gameplay emits; audio plugin consumes |
| Score selection, crossfades, arrangement transport, mute | `MusicDirector` resource and at most two `MusicDeck` entities | Music plugin after presentation; consumes independent cue reader for ducking |

Input is sampled after Bevy’s input update in `PreUpdate`; mouse deltas are applied once per rendered frame. Jump is latched until a fixed tick consumes it. The 60 Hz simulation explicitly chains motion history → collision snapshot → object motion → player → neighbor. Presentation follows interactions; HUD and audio follow presentation. Daylight advances on elapsed game time, never on frame count.

## Files

- `main.rs`: validated CLI, selected Bevy features, plugins and window.
- `house.rs` and `house/underground.rs`: authored house/maze geometry, furnishing, silhouettes, room volumes, real stair openings and graph links.
- `basement.rs` and `basement/`: deterministic maze data, three-circuit/service-key/exit progression, practical lighting, lever/fan/case animation, English room signs and a native Tab maintenance map.
- `motion.rs`: fixed position history and interpolated camera/neighbor rendering; teleports snap.
- `visuals.rs`: shared color grading and precomputed small sky cubemaps; gameplay only switches handles.
- `performance.rs`: opt-in fixed-resolution native encounter replay and CPU/GPU CSV timings.
- `world.rs`: shared AABB/ray math, procedural piece data, collision resource, BFS path graph.
- `player.rs`: bounded-substep kinematic controller, step-up and crouch clearance.
- `ai.rs`: patrol / investigate / chase / search, cone + occluded sight, hearing, route choice.
- `items.rs`: item types, colored locks and explicit puzzle prerequisites.
- `game.rs`: entity setup, interactions, gameplay schedules and procedural animated models.
- `atmosphere.rs`: continuous clock and interpolated sun/moon, fog and sky parameters.
- `ui.rs`: native Bevy UI, English messages, carrying hints and an H controls guide. The clock is hidden.
- `props.rs`: 14 household interaction types, target selection, eased hierarchies, finite water/steam pools, clue feedback and repeat-attempt reset.
- `character.rs`: procedural articulated character, movement-driven gait, blended facing, breathing and blinking.
- `materials.rs`: shared 32×32 procedural surface maps and material-specific roughness.
- `effects.rs`: impact dust with a 128-entity cap and timed cleanup; leaf sway.
- `audio.rs`: synthesized WAV assets, appliance/environment loops, distance/occlusion gain, and pause management for both loops and effects.
- `music.rs`: original stereo themes/arrangements, state priorities, stable situation changes, bounded crossfade decks, cue ducking, music mute and retry/pause handling. See [MUSIC.md](MUSIC.md).
- `capture.rs`: asynchronous native Bevy screenshot capture and windowless self-test.

## Physics boundary

The existing bounded-substep AABB controller has been retained as engine-independent Rust and moved to Bevy components/fixed systems. This is a deliberate small kinematic implementation, not a claim that Bevy has built-in rigid-body physics. It supports the authored axis-aligned architecture, stairs and simple thrown boxes. It does not provide general rigid-body stacking, arbitrary rotated collider hulls or a rideable train. Adopt a single verified physics integration later when those requirements become central; avoid maintaining two authorities over character transforms.

The primitive builder adds colliders only for actual solids. Non-axis-aligned decorative beams and roofs do not silently get inaccurate axis-aligned collision hulls. Door collision derives from the same progression state as visible door leaves. Held boxes are omitted from collision snapshots.

## Relevant skills applied

Bevy / Bevy Master, porting, Cargo features, ECS components/queries/systems, cameras, PBR materials, rendering, testing and audio; Bevy world-building, navigation and stealth references; idiomatic Rust. Engine APIs were checked against the installed 0.20.1 source. The skills guide the implementation; they do not establish that unfinished mechanics are complete.

## Verification

```sh
cd app
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
./target/debug/hello-neighbor --self-test
./target/debug/hello-neighbor --frames 120 --quiet
./target/debug/hello-neighbor --overview --time 9 --screenshot shot-morning.png --quiet
./target/debug/hello-neighbor --overview --time 14 --screenshot shot-afternoon.png --quiet
./target/debug/hello-neighbor --overview --time 17.7 --screenshot shot-evening.png --quiet
./target/debug/hello-neighbor --overview --time 23 --screenshot shot-night.png --quiet
```

`--screenshot` renders to a 1280×720 image target and exits only after the asynchronous capture has been saved. `--frames` is a separate smoke-run limit; it never interrupts an outstanding requested screenshot. Screenshot timeout/failure exits with an error. Time controls validate finite values and reject a zero day length.

See the session’s final verification notes / README for checks actually run; a command listed above is an acceptance target, not evidence that it passed.

## Presentation boundaries

Household targets are selected from authored pieces by name in one central mapping. Mesh children move about unscaled pivots. The four HUD cells indicate crowbar, keycard, umbrella and carried box; keys are separate flags. These cells do not implement item selection or swapping. H exposes objective/control help. The underground entrance board unlocks a Tab map with the player position, circuits, key vault and exit. The map hides while paused or outside the underground area.

Cupboards and drawers retain their furniture body's collision. Their opening is a storage/clue interaction, not a hiding mechanic. Door gate collision still derives from puzzle state, while visible leaves ease open. Windmill/trolley presentation coasts down; progression state changes at the switch press. The trolley route uses a consistent 113.2 m perimeter and smooth orientation around its rectangular corners.

Water/steam use twelve persistent particles per source and hide while off. Impact dust emits six short-lived particles per cue and caps concurrent particles at 128. Leaf and character motion stop during pause. Household controls, door poses, train position/speed and rotor pose reset on a new attempt. No camera shake or forced camera bob is added.

World sounds use mono synthesized sources with application-controlled distance and wall attenuation. They do not simulate room acoustics, HRTF or stereo source positioning. Environmental mix responds to the continuous clock and approximate indoor location. Generator/windmill/train loops respond to puzzle state; appliances respond to their controls. One-shot entities despawn after playback; all managed sinks pause with the game. --quiet suppresses playback.

Music is a separate non-spatial stereo category, with thirteen original themes and two precomputed arrangements per theme. Two decks crossfade; outgoing entities despawn after the transition, and retry replaces the previous attempt's decks. Routine state selection has a dwell and minimum hold; urgent states shorten an ongoing fade before starting their own transition. The director's transport freezes with pause. All volume options are validated independently; live volume changes use sinks, not PlaybackSettings. M changes only score mute and keeps that choice across retries.

--showcase is only accepted with --view and --screenshot and enables household props for inspection. It does not bypass the puzzle in normal play.

## Underground and encounter boundaries

The maze is a deterministic 11 × 11 graph. One cell is reserved for the entrance staircase; twelve two-by-two chambers and additional loops break up the corridors. Authored static geometry and matching navigation share the same graph. Every underground graph edge is walked in both directions by the real player controller in tests. Neighbor route replanning preserves forward progress in long corridors and requires body clearance before a direct shortcut.

Opening the house basement gate leaves the game exploring. Only the underground service-exit interaction completes the run. Basement progress and map knowledge reset with the attempt. Presentation freezes on pause/result screens. Distant underground geometry and practical lights are hidden when irrelevant to the camera, while their collision remains present. These lamps use approximate unshadowed light pools, rather than room-acoustic or light-portal simulation.

Collision snapshot reset is atomic: retry rebuilds solids, the static boundary and loose-box entries together. This prevents a render-frame interaction from slicing a stale boundary before the next fixed tick. The regression test retries repeatedly without a physics system installed.

Simulation catch-up is capped at 100 ms of virtual time per render frame (at most six 60 Hz ticks), preventing a long rendering stall from triggering a large burst of physics work. Real-time replay measurement still uses `Instant`. Detailed GPU render diagnostics require `NEIGHBOR_PROFILE_GPU=1`; ordinary replay captures main-schedule/frame intervals without timestamp query/readback overhead.
