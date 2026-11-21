# Native encounter performance — November 21, 2026

The encounter work fixes specific failure modes and reduces presentation cost. It does not establish a guaranteed frame rate on all machines.

## Changes

- Retry reconstructs the collision array, its static boundary and loose-box snapshot atomically. Repeated render-frame retries no longer use a stale slice boundary; a regression test reproduces the original scheduling condition without physics installed.
- Camera and articulated-neighbor rendering interpolate fixed positions. Mouse look remains current-frame; collision remains at 60 Hz. Ladder, retry and recovery teleports snap.
- FXAA replaces 4× MSAA. Directional shadows use two cascades to 75 m rather than four to 150 m.
- Unchanged UI/prop/material/light state avoids unnecessary writes. Out-of-range audio/hearing checks return early. Small sky images are precomputed at startup.
- Underground meshes and lights are hidden when irrelevant to the camera. Physics keeps the complete authored collision. Rooms use shared meshes/materials and practical unshadowed lamps.
- Direct neighbor movement checks body clearance, and route replanning preserves progress between long corridor nodes. Player stepping handles stacked soil/floor thresholds.
- Virtual-time catch-up is capped at 100 ms (six fixed ticks per render frame), reducing cascading physics work after a rendering stall.
- Sight and sound occlusion rays now broad-phase against the ray's bounds before testing individual colliders, avoiding segment tests for distant solids. The static collision snapshot is retained between puzzle changes; fixed ticks refresh only loose-box collisions until a lock/progression state changes.
- Optional `--low-graphics` mode uses a 960×540 window (43.75% fewer pixels than 1280×720), disables bloom and both directional/flashlight shadow maps, and shortens the directional shadow setup. It is opt-in and leaves gameplay and physics unchanged; screenshots stay at 1280×720.

## Reproduction

```sh
cargo build --manifest-path app/Cargo.toml --locked
./app/target/debug/hello-neighbor --profile-encounter /tmp/encounter.csv
./app/target/debug/hello-neighbor --profile-encounter /tmp/encounter-quiet.csv --quiet
NEIGHBOR_PROFILE_GPU=1 ./app/target/debug/hello-neighbor --profile-encounter /tmp/encounter-gpu.csv --quiet
```

The replay uses a 1280×720 window and uncapped presentation, waits for render pipelines, warms up for four seconds, then records until 28 seconds. It alternates the safe street and close neighbor encounters, including actual capture and R retry. CSV rows group frame interval and main-schedule duration by patrol, chase and caught state; pass-level GPU/CPU diagnostics are opt-in. This mode does no work during ordinary play.

Frames are measured with `Instant`; the main-schedule measurement excludes much of the separate render thread. The CSV is a diagnostic replay, not a minimum-FPS guarantee or a complete manual playthrough. The native build is the optimized development profile, not a release build.

## Instrumented captures and limits

The local machine uses Intel Iris Xe integrated graphics, Vulkan, Mesa 26.3.1, with a live desktop workload. Raw instrumented captures are in [performance](performance/), with `instrumented-` prefixes. Two quiet before/after pairs completed, followed by an audio-enabled candidate replay. None panicked. The audio-enabled instrumented replay reported one rodio buffer underrun.

Instrumented chase median frame intervals were 29.0 / 29.6 ms before and 26.6 / 31.4 ms after. However, long stalls pushed their 95th percentiles above one second, and the audio-enabled run differed sharply again. These numbers are too variable to justify an improvement percentage or a claim that all lag is gone. GPU timestamp/readback instrumentation and desktop/driver conditions are possible contributors; the captures do not isolate their causes.

Detailed render diagnostics therefore require `NEIGHBOR_PROFILE_GPU=1`. Ordinary replay records frame/main-schedule timings without the extra renderer instrumentation.

### Final quiet-mode validation and replay

On November 21, 2026, the final native development binary passed `cargo fmt --check`, all 46 tests, Clippy with warnings denied, `cargo build --locked`, `--self-test`, and a 120-frame `--quiet` windowed smoke run. `--quiet` disables Bevy's audio plugin while retaining the audio asset type required by gameplay data; the smoke run and replays completed without ALSA/rodio initialization errors.

Two ordinary quiet replays were captured from this binary. Each replay warms up for four seconds and samples the next 28 seconds. The table shows the range across those two runs, not a confidence interval:

| State | Samples (run 1 / 2) | Frame interval p50 (ms) | p95 (ms) | p99 (ms) | Largest observed (ms) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Patrol | 318 / 277 | 6.52–8.97 | 50.60–67.60 | 181.16–658.56 | 1,415.51–1,452.51 |
| Chase | 151 / 134 | 9.87–9.91 | 50.70–85.65 | 1,216.24–1,285.41 | 1,327.38–1,454.97 |
| Caught | 394 / 286 | 6.89–8.11 | 139.97–157.42 | 158.94–168.84 | 1,351.15–1,362.40 |

For the main schedule, p50 was 3.10–4.57 ms (patrol), 4.79–4.88 ms (chase), and 3.23–3.98 ms (caught); p95 ranged from 16.57 to 35.38 ms across states and runs. Exact counts and all summary statistics are in [`final-quiet-1.csv`](performance/final-quiet-1.csv) and [`final-quiet-2.csv`](performance/final-quiet-2.csv).

One additional replay with `NEIGHBOR_PROFILE_GPU=1` completed and is saved as [`final-gpu-quiet.csv`](performance/final-gpu-quiet.csv). Its frame-interval p50 was about 16.7 ms in all three states, with p95 of 35.03 ms (caught), 51.14 ms (chase), and 41.40 ms (patrol). The main opaque pass GPU p50 was 5.68 ms (patrol), 8.26 ms (chase), and 9.11 ms (caught). The instrumented replay still observed maxima of 632.94 ms (patrol), 443.94 ms (chase), and 620.60 ms (caught). GPU pass timings are diagnostic and are not additive to the frame interval.

The ordinary quiet runs' very low medians coexist with chase p99 values above 1.2 seconds and maxima around 1.4 seconds in every state. This demonstrates substantial outliers and variation under the live desktop workload; it does **not** establish a stable frame rate, an improvement percentage, or performance on other systems. The optional GPU timestamp/readback path also changes the workload. Keep the raw CSVs with these caveats when comparing future captures.

The 16-image screenshot suite was refreshed from the same final binary after the quiet-mode fix. The PNGs are 1280×720 and cover morning, afternoon, evening, night, house rooms, the neighbor, underground rooms, and the maintenance map. They establish rendering/readability only, not motion performance. At the time of this initial capture pass, Flatpak packaging was still pending; it was subsequently built and verified below. Subjective audio balance still needs listening tests.

### Follow-up lag polish — November 21, 2026

The low-graphics option and collision/ray optimizations passed formatting, Clippy (`-D warnings`), the native build, and all 48 tests, including a deterministic comparison of the optimized sight-ray broad phase against full segment checks for 500 generated rays and 1,001 colliders. The existing interaction tests still verify that a newly opened keyed door is removed from collision on the next fixed tick.

Two interleaved quiet encounter captures per setting were made on the same machine after the change. `--low-graphics` reduced the typical frame-interval median in the first pair (about 5.9–8.0 ms versus 18.4–39.3 ms at default settings), but the second pair reversed that ordering: about 9.9–11.4 ms default versus 16.7–20.6 ms low-graphics, depending on state. The p95s remained high and varied widely in both modes (roughly 53–197 ms); captures also recorded isolated pauses up to 32.7 seconds. Full data is in the `polish-default-ab-*` and `polish-low-ab-*` CSVs.

The host was simultaneously running unrelated CPU- and memory-intensive tasks, with little free RAM and no swap; a profiling run itself took 54.9 seconds wall time for a nominal 28-second capture. Those outliers should not be attributed to the game. The runs are not controlled enough to quantify a guaranteed gain, but the opt-in mode provides a real render-cost fallback and the ray/static-snapshot changes remove avoidable CPU work. If severe stalls continue in a clean session, compare on the target machine with background workloads closed and collect a fresh capture.

Pass-level instrumented captures are [`polish-before-gpu.csv`](performance/polish-before-gpu.csv) and [`polish-low-gpu.csv`](performance/polish-low-gpu.csv). The low preset removes the bloom pass and reduces its shadow-pass median to approximately 0.011 ms. Opaque-pass GPU medians were mixed: default → low was 34.99 → 32.77 ms (caught), 21.65 → 26.50 ms (chase), and 38.00 → 27.05 ms (patrol). Since these are single, non-isolated runs under heavy host load, they verify the quality-setting change but do not support a general performance percentage.

### Flatpak package verification — November 21, 2026

The release-profile x86_64 Flatpak built successfully with Flatpak 1.18.4 and `flatpak-builder` 1.4.12 against Freedesktop 26.08. The resulting `org.helloneighbor.HelloNeighbor.flatpak` is **14,432,864 bytes (13.77 MiB)**; the installed app payload is 75,635,712 bytes (75.6 MB). The runtime is a separate dependency and is not included in the single-file application bundle. The bundle installed successfully, its packaged `--self-test` passed (3,019 authored pieces, 778 colliders, 34 named room volumes, six floors, day/night cycle, and 12-chamber underground labyrinth), and a 120-frame windowed `--quiet` run exited successfully on the local Intel Iris Xe/Vulkan setup. This validates packaging and startup on this machine, not compatibility or performance on every Linux system.
