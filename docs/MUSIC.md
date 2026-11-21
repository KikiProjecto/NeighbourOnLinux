# Background music

The score changes automatically with gameplay. It is separate from footsteps, appliance sounds and environmental ambience. These are original synthesized compositions for this prototype; the original game's recordings are not bundled.

| Situation | Musical response |
| --- | --- |
| Morning outdoors | Sparse bells and a lighter neighborhood motif |
| Afternoon outdoors | Plucked, slightly crooked melody |
| Evening outdoors | Lower felt keys and sustained harmony |
| Night outdoors | Widely spaced upper bells and a quieter bed |
| Exploring inside the house | An uneasy domestic motif |
| Near active generator, tower machinery or trolley controls | Mechanical ticking and a repeating plucked figure |
| Inside after the generator is disabled | Sparse low notes and more space |
| Nearby neighbor investigates or becomes suspicious | Tension notes and ticking |
| Neighbor chases the player | Faster subdivisions, bass and percussion |
| Nearby neighbor searches after losing the player | Restrained ticking and unresolved notes |
| Captured | A descending, subdued theme under the result screen |
| Underground labyrinth exploration | Low, sparse notes and a distinct enclosed-space motif |
| Escaped through the underground service exit | A brighter closing motif |

Capture/completion take priority over threat, and threat takes priority over location and time. A distant investigation does not trigger suspicion music, and crossing the safe street restores the outdoor theme after the transition delay. Machinery music requires a nearby active mechanism. Below-ground location overrides the surface house bounds; above-ground indoor detection remains approximate. This is not a complete acoustic room system.

Each of the thirteen themes has two 24-second stereo arrangements. Long stays automatically crossfade to the alternate arrangement before the current one ends. The shared 80 BPM pace and related pitch palette keep transitions coherent; chase uses faster subdivisions. These are short synthesized arrangements, not final mastered recordings.

## Transitions and controls

Normal situation changes must remain stable for 1.1 seconds, with at least 3.5 seconds on the current arrangement, to avoid switching repeatedly at a doorway. Exploration crossfades take 2.6 seconds; suspicion/search take 1.3 seconds. Chase enters with a 0.45-second fade and capture/completion with 0.85 seconds. If danger arrives during another transition, that transition finishes within 0.12 seconds before the danger fade starts. At most two music players exist at once.

Alert, capture, completion, pickup and reading cues briefly lower the score so their feedback remains clear. Pause freezes the music transport and sinks. Retry removes the old decks and begins the new attempt's score. Music mute persists across retries.

- Music starts enabled. Press **M** to mute/restore only the music; the HUD confirms the choice in English.
- `--music-volume 0.7` sets music gain; this is the default.
- `--ambience-volume 1` sets environmental/machinery/appliance gain.
- `--effects-volume 1` sets short feedback/footstep gain.
- Each volume accepts a finite number from 0 to 1. `--quiet` disables all playback and skips score synthesis.

For example:

```sh
./app/target/debug/hello-neighbor --music-volume 0.5 --ambience-volume 0.8 --effects-volume 1
```

There is no full audio settings menu yet. Playback and PCM checks establish functional behavior, not subjective listening quality; listening balance still needs play sessions.
