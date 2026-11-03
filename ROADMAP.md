# ROADMAP — MVP

## Milestone 1 — Window & configuration

- [x] `window_conf` 1280x720, title "Hello Neighbor", resizable
- [x] `LinuxBackend::WaylandWithX11Fallback` backend
- [x] Parse `--screenshot`, `--self-test`, `--frames` flags

## Milestone 2 — Player

- [x] WASD, mouse look with cursor grab
- [x] Space jump, Shift sprint, Ctrl crouch, E interact
- [x] Gravity + AABB vs world colliders
- [x] Stair step-up and anti-tunneling substeps

## Milestone 3 — World

- [x] Two-story house built from box blocks
- [x] Static collider grid
- [x] Openable exit door
- [x] Key on the first floor
- [x] Stairs to the second floor

## Milestone 4 — Neighbor AI

- [x] Waypoint patrol
- [x] Distance + line-of-sight detection
- [x] Patrol / Suspicious / Chase states
- [x] Chase and capture = game over

## Milestone 5 — Items

- [x] Key and locked door
- [x] One carryable/droppable object
- [x] Win by opening the door with the key

## Milestone 6 — HUD

- [x] Crosshair
- [x] Interaction prompt "E: ..."
- [x] Detection status (SUSPICIOUS / DETECTED)
- [x] Win and lose screens

## Milestone 7 — Capture & documentation

- [x] `--screenshot` produces a 1280x720 PNG (> 5 KB)
- [x] `--self-test` passes, exit 0
- [x] `--frames N` auto-exit
- [x] CLAUDE.md, ROADMAP.md, STATE.md, README.md
