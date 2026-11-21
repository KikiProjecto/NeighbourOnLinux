# Underground maintenance labyrinth

The basement now continues beyond the house's crowbar/keycard door. Real stairs descend 4.2 m into a 48.4 × 48.4 m labyrinth: an 11 × 11 layout, with one cell reserved for the staircase and twelve larger 8.8 × 8.8 m chambers. The remaining corridors branch, loop and reconnect. The layout is deterministic, so its signs, map and cable routes agree on every attempt.

## Play sequence

1. Complete the existing colored-key → tower windmill → generator → freezer keycard progression and remove the basement boards with the crowbar.
2. Open the basement door and walk down the stairs. The game continues exploring.
3. Aim at the maintenance board near the foot of the stairs and press **E**. **Tab** then opens/closes a map showing the corridors, your location, breaker positions, key vault and exit. North is at the top.
4. Follow **orange** cable to the boiler room, **teal** to the generator vault, and **purple** to ventilation. Press **E** at each breaker. Any order works. Restored circuits change their indicators, handle poses and chamber lights; ventilation starts the fan.
5. Find the evidence vault. All three circuits release its hinged key-case lid. Press **E** to collect the service key.
6. Take the key to the service exit and press **E** to escape. **R** starts a fresh attempt after escape or capture.

The map is optional. Room numbers and English signs identify landmarks; colored cables provide continuous routes from the entrance to the breakers. **H** shows the current objective and controls. Run with **Shift**, crouch with **Ctrl/C**, interact with **E**, throw a carried box with **G**, drop it with **Q**, and use **F** for the flashlight.

## Room register

| Number | Chamber | Details and purpose |
| --- | --- | --- |
| 01 | Old boiler room | Pressure vessels, gauges, pipework, maintenance log, orange circuit |
| 02 | Maintenance workshop | Workbench, tools and spare-part shelving |
| 03 | Pump station | Pressure tanks and a shallow decorative basin |
| 04 | Generator vault | Transformers, cooling fins, overhead cable and teal circuit |
| 05 | Abandoned cistern | Tanks and still-water dressing |
| 06 | Cold storage | Tall cabinets, latches and wall vent |
| 07 | Records archive | File shelves, desk and readable progression records |
| 08 | Observation room | Monitor, recording indicators and desk |
| 09 | Ventilation chamber | Ducts, purple circuit and responsive fan |
| 10 | Improvised laboratory | Flask bench, spare parts and three-color circuit diagram |
| 11 | Evidence vault | Lockers and powered service-key case |
| 12 | Service exit | Reinforced terminal escape door, handle and emergency lamp |

## Integrated behavior and boundaries

The player can walk both down and back up without jumping. Floors, soil and ceiling have a matching stair opening. Collision and navigation derive from the authored layout; the neighbor can follow the entrance and navigate underground after hearing or seeing the player. Existing carrying, throwing and flashlight mechanics continue to work.

Underground exploration has a separate adaptive score and a drips/ventilation ambience layer. Threat and result music retain priority. Exterior wind and insects attenuate below ground. Room lights respond to circuits; dark distance fog limits distant views. All sound, motion and time honor pause. Circuit/key/map state resets on retry.

The pressure tanks, pumps, stored water, observation monitor and flasks are environmental dressing. They are not fluid simulations, surveillance systems or additional puzzle controls. The final service-door interaction ends this prototype's run; there is no further playable district behind it. Geometry and props remain original procedural models.

## Inspection and validation

```sh
./app/target/debug/hello-neighbor --view basement --showcase --screenshot boiler.png --quiet
./app/target/debug/hello-neighbor --view basement-lab --showcase --screenshot lab.png --quiet
./app/target/debug/hello-neighbor --view basement-vault --showcase --screenshot vault.png --quiet
./app/target/debug/hello-neighbor --view basement --showcase --controls --screenshot map.png --quiet
```

Tests cover deterministic connectivity and reciprocal passages, every underground navigation edge in both directions using the actual player controller, standing approaches to every control, descent/ascent, the actual neighbor's route to a distant chamber, and E/Tab progression with real authored collision, pause and retry. The existing house progression test verifies that opening its basement gate no longer ends the run.
