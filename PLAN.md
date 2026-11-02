# Plan

Stage 2, written by the agent from `BRIEF.md` and the person's answers to CA-Q1 to CA-Q16 (journal, 2026-11-02). `B2 Require step-by-step justification`. Approved in advance (BRIEF.md Autonomy).

Each milestone ends with `./scripts/check.sh` passing, a probe reading in `MEASUREMENTS.md`, shots of the running build, and a journal entry.

| # | Decision | Reason | Test that shows it holds |
| --- | --- | --- | --- |
| M0 | Port CHUMWORLD's combat (arm, blade, swing, health, wounds, feel, pilot, motion, pose, fx, ticks, rng, input, geom, prim, mesh, render, look, models/figure) from commit d9cb5b5 into a lean core whose world is one flat ring and two fighters. World-only systems (terrain, water, towns, items, magic, story, dungeons) are left behind; their hooks in the combat code are cut, each cut a notebook row. | CA-Q1: A; CA-2. A ring needs none of the world, and the download should shrink (budget: under 600 KB of WebAssembly). | `port_keeps_chumworld_duel`: a scripted duel recorded in a copy of CHUMWORLD (arm points, health, every tick) replays here to the same trace. |
| M1 | First person: the eye in the helmet at 100–110° (CA-Q15: B), no visor edge; the mouse turns view and body; the reach shell and guard ride the view; right button frees the hand; a held left button reads a CHUMWORLD gesture; own arms, sword and legs drawn; no aim help by default, a soft pull as a setting (CA-Q16: C). | CA-Q6: A. | Core tests: the guard follows yaw and pitch; the gesture still snaps to the four cuts; the soft pull is off by default and keeps the foe within the view when on. |
| M2 | The 40 m stone ring (CA-Q11: A), rounds (best of three, sudden death after 90 s inside the inner 12 m, CA-Q14: A), the computer's knight (one pilot), menus as view models (title, main menu, practice, settings, round and match results), and one browser save for settings and look plus a JSON file (CA-Q4: C, D17 change). | The MVP's single-player loop. | Round and match rules tested in the core; a whole AI duel probe runs to a result; save round trip and refusal of a bad file. |
| M3 | The two knights, A (the black tower) and B (the ivory crown), as approved (CA-Q13: A), built from primitives on CHUMWORLD's figure by background agents (CA-Q3: A); one two-handed longsword (CA-Q12: A). Plate rules (CA-Q8: A): plate takes 70% off a cut, 20% off a thrust; gaps take full damage; a part's plate breaks off after enough damage and shows mail; severing only on bare parts. | CA-7. | Silhouette, no shared faces within 2 mm in different colours, triangle cap 6,000 each; plate damage worked by hand in tests. |
| M4 | The Chivalry layer (CA-Q7: B): stamina, feints, combos, ripostes, and a kick that pushes a guard aside for 20 stamina; the block stays physical. | T3. | One test per rule, each broken once in `tests/breaks.json`. |
| M5 | Online 1v1 with Builder's online kit: room code and pasted code, loaded only on Host or Join; rollback with an 8-tick window (CA-Q10: A), the kit's lockstep tests kept; checksum reports desync. Then publish. | The MVP is online; T1. | The kit's eight tests, two rollback tests, `online_fence.rs`, `tests/browser/online.py` (two browsers on one machine). |
| M6 | The ladder: seven rungs F to S, each opponent a pilot with its own settings and its own costume built through the costume flow (concept sheet, model, tests, shots); rank cards and dossiers; rungs unlock looks only (CA-Q9: A); progress in the save. Then publish. | T6. | Each costume's silhouette, z-fight and triangle tests; ladder unlock rules; save round trip with progress. |

## Open questions for the person

None blocking. R1.2 (the game's name and the repository name) is the person's to change.

## Status

- [x] Approved in advance on: 2026-11-02 (BRIEF.md Autonomy)
