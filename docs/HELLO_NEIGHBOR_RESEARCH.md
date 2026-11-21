# Hello Neighbor reference research

Research date: 2026-11-21. This document separates the original game from our adaptation.

## Core reference

The original **Hello Neighbor** was developed by Dynamic Pixels and published by tinyBuild in December 2017. Its core loop is stealth, environmental experimentation, and discovering the neighbor’s basement secret. The official store description emphasizes a neighbor who counters repeated approaches, including traps, cameras and shortcuts. Our prototype currently implements sight, noise investigation, pursuit, search and a remembered capture location—not the full advertised adaptation system.

Primary source: [Official Steam listing](https://store.steampowered.com/app/521890/Hello_Neighbor/).

Developer context: [Game Developer interview: Designing a domestic hunter-killer thriller](https://www.gamedeveloper.com/design/designing-a-domestic-hunter-killer-thriller-the-i-hello-neighbor-i-way). Useful for sound-driven investigation and reacting to changes in the environment; not evidence for a particular machine-learning implementation.

## Which house?

| Version | Reference value | Guardrail |
| --- | --- | --- |
| Release Act 1 | Smaller, more recognizable suburban home; establishes the basement mystery | Do not present its compact floor plan as the giant house |
| Release Act 2 | Expanded house, tall perimeter fence, pipes and flooded-room escape puzzle | Its flooded room includes the Sharkotron |
| Release Act 3 | Surreal, vertically expanded house; train, strange machinery, roof vegetation, fear spaces | Main reference for this rebuild |
| Alpha 4 | Earlier giant-house silhouette and exterior mechanisms | A development build, not a fourth release act; puzzles/layout differ |

Community references: [House development history](https://helloneighbor.wiki.gg/wiki/910_Friendly_Court/Development), [Rooms](https://helloneighbor.wiki.gg/wiki/Rooms), [Act 3](https://helloneighbor.wiki.gg/wiki/Act_3_(Hello_Neighbor)).

The community development pages mix released and unused content. Exact placements and proportions need screenshot-based comparison during later art passes. Our current room arrangement is authored for clear navigation and is not a measured reconstruction.

## Act 3 landmark findings

The release walkthrough confirms routes involving the elevated trolley and its control area, exposed pipes, roof trees, a hanging structure, a painting-operated route, and linked object/switch puzzles. Its basement access requires both removing boards with a crowbar and dealing with an electronic keycard lock. The crowbar’s original retrieval/cooling sequence is more elaborate than simply picking up a tool; the fear challenges also participate in progression.

Source: [GameSkinny’s December 2017 Act 3 walkthrough](https://www.gameskinny.com/tips/hello-neighbor-act-3-complete-walkthrough/).

The three fear spaces are school, supermarket and darkness/closet themes. They contextualize the surreal house and should eventually be distinct spaces, not recolored copies of ordinary rooms.

Sources: [Act 3](https://helloneighbor.wiki.gg/wiki/Act_3_(Hello_Neighbor)), [house development history](https://helloneighbor.wiki.gg/wiki/910_Friendly_Court/Development).

**Shark-room correction:** Act 2’s flooded room has the hostile Sharkotron; Act 3’s flooded room is a different version. Do not transfer an Alpha/Act 2 hazard into Act 3 and call it an exact reconstruction. Source: [Flooded Room](https://helloneighbor.wiki.gg/wiki/Flooded_Room).

## Peterson visual reference

The recognizable silhouette includes a large chin, reddish-brown hair, mustache, dark eyebrows, rolled shirt sleeves, blue patterned sweater vest, brown trousers and dark shoes/gloves. Our original primitive character uses these broad cues and a simple walking animation; it is not a final character model.

Community visual reference: [Theodore Peterson](https://helloneighbor.wiki.gg/wiki/Theodore_Peterson).

## Engine verification

The local toolchain is Rust 1.99.1. Bevy 0.20.1 is available in the local registry and verified against the package registry metadata. Feature flags, material/light fields, screenshot APIs and font APIs were checked against the actual 0.20.1 engine source and examples, rather than assuming older Bevy tutorials still compile.

Primary engine sources: [Bevy package metadata](https://crates.io/api/v1/crates/bevy/0.20.1), [Bevy documentation](https://docs.rs/bevy/0.20.1/bevy/), and the matching cached `bevy-0.20.1/examples`.

## Deliberate adaptation

The blue archive → green greenhouse → red tower lock chain and listening/dynamo gallery are our first playable house design. They make the house’s dependencies readable but are **not** claims about the original game’s exact key placement. No original game models, textures, sounds or maps were extracted or redistributed.

## User-provided interface reference

[Interface In Game — Hello Neighbor](https://interfaceingame.com/games/hello-neighbor/) was supplied during the polish pass. Its pickup screenshot was inspected for the compact inventory, small reticle and unobstructed first-person composition. The collection also distinguishes main menu, act selection, settings, controls, pause and keyhole views. Those screens inform future work; viewing the reference is not a claim that those mechanics or pages are implemented. The game now defaults entirely to English.
