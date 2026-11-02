# Handoff

Stage 9, written by the agent. `C5 Consolidate in your own words`. A fresh session starts from this file, not from memory.

## What this is

M0 to M2 of a first-person sword duel for the browser: CHUMWORLD's combat (d9cb5b5) in one stone ring, seen from inside the helmet (M1: the mouse turns view and body, the guard rides the view, the right button frees the hand, the left reads a cut; 105° across, aim help off by default). M2: the real ring (`ring.rs`, `data/ring.json`), Practice as a duel of best of three rounds against the computer's knight (`rounds.rs`, pilot `knight`), sudden death after 90 s, the title, menu, settings, pause and result screens as view models, and one browser save plus a JSON file (`save.rs`, `web/save.js`). `port_keeps_chumworld_duel` still holds the port to CHUMWORLD's numbers on the world's default `Control::Shoulder` path (R3.1). M3, merged from `m3-knights`: two knights in plate (`models/knight.rs`; A "The black helm", B "The ivory crown"), plate as a rule (`plate.rs`: 70% off a cut, 20% off a thrust, gaps whole, battered plate breaks off, plated limbs never sever) and one two-handed longsword. In Practice the player wears Knight A and the computer's knight wears Knight B while thinking with M2's `knight` pilot (`Brain::key`); the save's look B swaps them (R5.19). Every round stands both up in whole plate (R5.17).

## Rules that hold it up

The rules that break silently if a later change ignores them, and the test that guards each.

## Shape of the code

Where each kind of thing lives, in a short table.

## Commands

Build, test, check, publish, and the probes, with what each prints when it passes.

## What will bite within the hour

What a new session gets wrong first.

## The journal

Where the design journal is, its link if it is published, and its newest entry.

## Open for the person

- R3.10 (open until the coordinator's M1 journal entry): a chop's wind-up lifts the sword forearm across the eye for a few frames (`../chumworld-arena-journal/img/build/m1/chop-wind-up-arm.jpg`): accept it, cull the forearm near the eye, or move the eye back.
- R1.2: the game's name and the repository name. The agent chose `sgilson7/chumworld-arena` and the display name "Chumworld Arena" because CA-Q2's note named neither.
- R2.14 (open, for the M0 journal entry): the ring's look (M2's `data/ring.json`; M0's `ring_*` colours in `data/models.json` were dropped in the M3 merge, R4.15) and the darker page frame are the agent's; the look is the person's to judge from the shots (`../chumworld-arena-journal/img/build/m0/`, `m2/`).
- M3 (open, for the coordinator's M3 journal entry, then *person*; shots in `../chumworld-arena-journal/img/build/m3/`):
  - R5.4: how long plate lasts (each plate breaks on the third 10 m/s cut in one place).
  - R5.5: the gaps' size (armpit, visor slit, back of the knee).
  - R5.8: Knight B's ivory darkened and its tabard turned from red to oxblood, to stand clear of the spark and blood colours.
  - R5.9: the knights' working labels on screen, "The black helm" and "The ivory crown" ("tower" is ruled out by the Ladder entry).
  - R5.10: B's skirt steps out lame by lame instead of flaring.
  - R5.13: the player in plate no longer gets CHUMWORLD's player toughness against severing.

## Mistakes so far

By system, from `SECOND-ORDER.md`, with the rule each one produced.

## What passed

2026-11-02, M0: `./scripts/check.sh` ends `BROWSER CHECK PASSED in chromium, firefox, webkit` and `BUILDER CHECK PASSED`; `./scripts/count-tests.sh` prints `237 passed, 0 failed, 1 ignored`.

2026-11-02, M1 and M2: `./scripts/check.sh` ends `BROWSER CHECK PASSED in chromium, firefox, webkit` and `BUILDER CHECK PASSED`; `./scripts/count-tests.sh` prints `263 passed, 0 failed, 1 ignored`; `port_keeps_chumworld_duel` passes unchanged.

2026-11-02, M3 (branch `m3-knights`): `./scripts/check.sh` ends `BROWSER CHECK PASSED in chromium, firefox, webkit` and `BUILDER CHECK PASSED`; `./scripts/count-tests.sh` prints `257 passed, 0 failed, 1 ignored`; `python3 scripts/break_tests.py M3` catches all 40 rows whose names hold M3.

2026-11-02, M3 merged into main (M1, M2 and M3 together): `./scripts/check.sh` ends `BROWSER CHECK PASSED in chromium, firefox, webkit` and `BUILDER CHECK PASSED`; `./scripts/count-tests.sh` prints `287 passed, 0 failed, 1 ignored`; `python3 scripts/break_tests.py` catches 326 of 326 (325 on the first full run; the rotted M1 cull row repointed, R5.23); `port_keeps_chumworld_duel` passes with its test data unedited. The M3 branch's own line above held before its last commit's `println!` (R5.21).

## The next action

M4: the Chivalry layer (PLAN.md): stamina, feints, combos, ripostes and the kick. The coordinator's M3 journal entry puts R5.4, R5.5, R5.8, R5.9, R5.10, R5.13 and R3.10 to the person and flips them to *person*.
