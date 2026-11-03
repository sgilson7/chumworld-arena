# Handoff

Stage 9, written by the agent. `C5 Consolidate in your own words`. A fresh session starts from this file, not from memory.

## What this is

M0 to M2 of a first-person sword duel for the browser: CHUMWORLD's combat (d9cb5b5) in one stone ring, seen from inside the helmet (M1: the mouse turns view and body, the guard rides the view, the right button frees the hand, the left reads a cut; 105° across, aim help off by default). M2: the real ring (`ring.rs`, `data/ring.json`), Practice as a duel of best of three rounds against the computer's knight (`rounds.rs`, pilot `knight`), sudden death after 90 s, the title, menu, settings, pause and result screens as view models, and one browser save plus a JSON file (`save.rs`, `web/save.js`). `port_keeps_chumworld_duel` still holds the port to CHUMWORLD's numbers on the world's default `Control::Shoulder` path (R3.1). M3, merged from `m3-knights`: two knights in plate (`models/knight.rs`; A "The black helm", B "The ivory crown"), plate as a rule (`plate.rs`: 70% off a cut, 20% off a thrust, gaps whole, battered plate breaks off, plated limbs never sever) and one two-handed longsword. In Practice the player wears Knight A and the computer's knight wears Knight B while thinking with M2's `knight` pilot (`Brain::key`); the save's look B swaps them (R5.19). Every round stands both up in whole plate (R5.17).

M4, on branch `m4-chivalry`: the Chivalry layer (`chivalry.rs`, numbers in `data/chivalry.json`, R6.1 to R6.21). Stamina (100; cuts 8 to 12, a parry 10 to the parrier, 15 a second back after half a second's rest; winded at none, arms and the cut's motor at 55% until 25), the feint (the right button in a cut's first 10 ticks of wind-up, 15), the follow-up (a cut begun within 12 ticks of a cut that met no blade winds up in 8, three in a row), the riposte (a cut within 18 ticks of a parry winds up in 6, drives at 125% and presses 6 ticks against a blade) and the kick (F, 20, lands on its 8th tick within 175 cm: stagger, shove, the foe's cut broken and its guard 40° aside). The block stays blade on blade. The layer is a world setting, off on the port's path and on in every duel the game makes (R6.1). The computer's knight has its own tree `knight` (riposte, rest, kick a held guard, feinted chop), all through its input record. R3.10's default applied: the player's own forearms and elbows are not drawn within 32 cm of the eye (R6.14, R6.17). T7: the right button now holds a guard in front (the blade raised across the body, shifted by the mouse up to 15°), replacing M1's free hand (R6.22); T8: the view holds still while either button is held, so a hard swipe no longer flings it (R6.24). Both work per fighter from its own input record (`chivalry::guards(w, id, input)`); the steady view is in `player::look_mouse`, the player's view.

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

- R3.10 (person; journal `#2026-11-02-m1`; default: cull the forearm near the eye): a chop's wind-up lifts the sword forearm across the eye for a few frames (`../chumworld-arena-journal/img/build/m1/chop-wind-up-arm.jpg`): accept it, cull the forearm near the eye, or move the eye back.
- R3.10 (still *person*): M4 applied the default (cull the forearm near the eye, R6.14); shot `scratchpad/m4/chop-windup-r310.jpg`, to be copied into the journal by the coordinator.
- R6.16 (person): every number of the Chivalry layer (`data/chivalry.json`) and the knight's tree (`data/pilots.json` `knight`) is the agent's guess at the pace; the person judges it from play (CA-10). R6.13: the scripted probe's duel now lasts 53 s (74 s before T7) and the knight wins it 2–1. T7 and T8 (TRIAGE.md) are answered: the right button holds a guard in front, and the view holds while either button is held; whether they feel right is the person's. R6.20: the feint's window may be too tight. R6.21: the settled guard shows the blade end-on.
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

2026-11-02, M4 (branch `m4-chivalry`, in its worktree, before T7 and T8): `cargo fmt --check`, `cargo clippy -D warnings` and `cargo test` pass; `./scripts/count-tests.sh` prints `310 passed, 0 failed, 1 ignored`; `scripts/build.sh` (1 071 KB), `wasm_imports.py`, `art_check.py` and `notebook_check.py` pass; `journal_check.py` run on the main checkout passes (check.sh's step 8 looks for `../wt-m4-journal` in a worktree); `tests/browser/check.py` ends `BROWSER CHECK PASSED in chromium, firefox, webkit`; `python3 scripts/break_tests.py` catches 370 of 370; `port_keeps_chumworld_duel` passes with its data unedited.

## The next action

M4: the Chivalry layer (PLAN.md): stamina, feints, combos, ripostes and the kick.
