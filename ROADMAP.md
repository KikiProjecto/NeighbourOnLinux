# ROADMAP — MVP

## Milestone 1 — Window & konfigurasi

- [x] `window_conf` 1280x720, judul "Hello Neighbor", resizable
- [x] Backend `LinuxBackend::WaylandWithX11Fallback`
- [x] Parsing flag `--screenshot`, `--self-test`, `--frames`

## Milestone 2 — Player

- [x] WASD, mouse look dengan cursor grab
- [x] Space lompat, Shift lari, Ctrl jongkok, E interaksi
- [x] Gravitasi + AABB vs collider dunia
- [x] Step-up tangga dan substep anti-tunneling

## Milestone 3 — World

- [x] Rumah dua lantai dari box blocks
- [x] Grid collider statis
- [x] Pintu keluar yang bisa dibuka
- [x] Kunci di lantai 1
- [x] Tangga ke lantai 2

## Milestone 4 — Neighbor AI

- [x] Patroli waypoint
- [x] Deteksi jarak + line of sight
- [x] State Patrol / Suspicious / Chase
- [x] Kejar dan tangkap = game over

## Milestone 5 — Items

- [x] Kunci dan pintu terkunci
- [x] Satu objek bisa dibawa/dilepas
- [x] Menang saat membuka pintu dengan kunci

## Milestone 6 — HUD

- [x] Crosshair
- [x] Prompt interaksi "E: ..."
- [x] Status deteksi (MENCURIGAI / TERDETEKSI)
- [x] Layar menang dan kalah

## Milestone 7 — Capture & dokumentasi

- [x] `--screenshot` menghasilkan PNG 1280x720 (> 5 KB)
- [x] `--self-test` lolos, exit 0
- [x] `--frames N` auto-exit
- [x] CLAUDE.md, ROADMAP.md, STATE.md, README.md
