# Handoff

Stage 9, written by the agent. `C5 Consolidate in your own words`. A fresh session starts from this file, not from memory.

## What this is

M0 to M2 of a first-person sword duel for the browser: CHUMWORLD's combat (d9cb5b5) in one stone ring, seen from inside the helmet (M1: the mouse turns view and body, the guard rides the view, the right button frees the hand, the left reads a cut; 105° across, aim help off by default). M2: the real ring (`ring.rs`, `data/ring.json`), Practice as a duel of best of three rounds against the computer's knight (`rounds.rs`, pilot `knight`), sudden death after 90 s, the title, menu, settings, pause and result screens as view models, and one browser save plus a JSON file (`save.rs`, `web/save.js`). `port_keeps_chumworld_duel` still holds the port to CHUMWORLD's numbers on the world's default `Control::Shoulder` path (R3.1). M3, merged from `m3-knights`: two knights in plate (`models/knight.rs`; A "The black helm", B "The ivory crown"), plate as a rule (`plate.rs`: 70% off a cut, 20% off a thrust, gaps whole, battered plate breaks off, plated limbs never sever) and one two-handed longsword. In Practice the player wears Knight A and the computer's knight wears Knight B while thinking with M2's `knight` pilot (`Brain::key`); the save's look B swaps them (R5.19). Every round stands both up in whole plate (R5.17). M5, branch `m5-online`: online 1v1 by room code or pasted codes (Builder's online kit, `web/net/`), rollback with an 8-tick window and a 2-tick input delay (`rollback.rs`) on the kit's lockstep session kept as its reference (`online.rs`); the guest's knight is a second seat seen from its own helmet (`seat.rs`); the host is Knight A, the guest Knight B; `versus.rs` runs hosting, joining, rematch, leaving, silence and desync.

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

- M5 (branch `m5-online`; shots in the coordinator's scratchpad `m5/`):
  - R8.7: the menu's fifth button, Pasted codes, and where the pasted path is offered.
  - R8.14: the room-code path between two networks, and two strict NATs (which cannot meet without TURN), by hand before a release; one machine met over the relays on 2026-11-02.
  - R8.15: the host always wears Knight A and the guest Knight B, whatever the save's look.
  - R8.18: the online strings and the glossary's Desync, Pasted codes, Relay and Room code entries.

- R3.10 (person; journal `#2026-11-02-m1`; default: cull the forearm near the eye): a chop's wind-up lifts the sword forearm across the eye for a few frames (`../chumworld-arena-journal/img/build/m1/chop-wind-up-arm.jpg`): accept it, cull the forearm near the eye, or move the eye back.
- R1.2: the game's name and the repository name. The agent chose `sgilson7/chumworld-arena` and the display name "Chumworld Arena" because CA-Q2's note named neither.
- R2.14 (open, for the M0 journal entry): the ring's look (M2's `data/ring.json`; M0's `ring_*` colours in `data/models.json` were dropped in the M3 merge, R4.15) and the darker page frame are the agent's; the look is the person's to judge from the shots (`../chumworld-arena-journal/img/build/m0/`, `m2/`).
- M3 (person; journal `#2026-11-02-m3`; shots in `../chumworld-arena-journal/img/build/m3/`):
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

2026-11-02, M5 (branch `m5-online`, in the worktree `wt-m5`): steps 1 to 7 and 9 of `./scripts/check.sh` run one by one (step 8 looks for `../wt-m5-journal`; `python3 scripts/journal_check.py /Users/samgilson/Documents/ChumworldArena/chumworld-arena` run in its place); results in the commit's report. `tests/browser/online.py chromium firefox webkit`: ONLINE CHECK PASSED in all three. `port_keeps_chumworld_duel` passes with its data unedited. `python3 scripts/break_tests.py`: 358 of 360, the two misses repaired and caught alone (R8.19).

## The next action

M4: the Chivalry layer (PLAN.md): stamina, feints, combos, ripostes and the kick.
