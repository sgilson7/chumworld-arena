# Handoff

Stage 9, written by the agent. `C5 Consolidate in your own words`. A fresh session starts from this file, not from memory.

## What this is

M0 to M2 of a first-person sword duel for the browser: CHUMWORLD's combat (d9cb5b5) in one stone ring, seen from inside the helmet (M1: the mouse turns view and body, the guard rides the view, the right button frees the hand, the left reads a cut; 105° across, aim help off by default). M2: the real ring (`ring.rs`, `data/ring.json`), Practice as a duel of best of three rounds against the computer's knight (`rounds.rs`, pilot `knight`), sudden death after 90 s, the title, menu, settings, pause and result screens as view models, and one browser save plus a JSON file (`save.rs`, `web/save.js`). `port_keeps_chumworld_duel` still holds the port to CHUMWORLD's numbers on the world's default `Control::Shoulder` path (R3.1). M3, merged from `m3-knights`: two knights in plate (`models/knight.rs`; A "The black helm", B "The ivory crown"), plate as a rule (`plate.rs`: 70% off a cut, 20% off a thrust, gaps whole, battered plate breaks off, plated limbs never sever) and one two-handed longsword. In Practice the player wears Knight A and the computer's knight wears Knight B while thinking with M2's `knight` pilot (`Brain::key`); the save's look B swaps them (R5.19). Every round stands both up in whole plate (R5.17). M6's costumes, merged from `ladder-costumes-1` to `-3`: seven ladder opponents (Squire, Sellsword, Hedge, Veteran, Banneret, Champion, Mirror; `models/knight/` and `models/champion.rs`, `models/mirror.rs`), each spawnable with `duel.spawn:<kind>` on placeholder pilots copied from `knight_b`; plate coverage is one table, `plate::covered` (R5.25). The ladder itself is not built.
M0 to M2 of a first-person sword duel for the browser: CHUMWORLD's combat (d9cb5b5) in one stone ring, seen from inside the helmet (M1: the mouse turns view and body, the guard rides the view, the right button frees the hand, the left reads a cut; 105° across, aim help off by default). M2: the real ring (`ring.rs`, `data/ring.json`), Practice as a duel of best of three rounds against the computer's knight (`rounds.rs`, pilot `knight`), sudden death after 90 s, the title, menu, settings, pause and result screens as view models, and one browser save plus a JSON file (`save.rs`, `web/save.js`). `port_keeps_chumworld_duel` still holds the port to CHUMWORLD's numbers on the world's default `Control::Shoulder` path (R3.1). M3, merged from `m3-knights`: two knights in plate (`models/knight.rs`; A "The black helm", B "The ivory crown"), plate as a rule (`plate.rs`: 70% off a cut, 20% off a thrust, gaps whole, battered plate breaks off, plated limbs never sever) and one two-handed longsword. In Practice the player wears Knight A and the computer's knight wears Knight B while thinking with M2's `knight` pilot (`Brain::key`); the save's look B swaps them (R5.19). Every round stands both up in whole plate (R5.17). M5, branch `m5-online`: online 1v1 by room code or pasted codes (Builder's online kit, `web/net/`), rollback with an 8-tick window and a 2-tick input delay (`rollback.rs`) on the kit's lockstep session kept as its reference (`online.rs`); the guest's knight is a second seat seen from its own helmet (`seat.rs`); the host is Knight A, the guest Knight B; `versus.rs` runs hosting, joining, rematch, leaving, silence and desync.

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

- M5 (branch `m5-online`; shots in the coordinator's scratchpad `m5/`):
  - R8.7: the menu's fifth button, Pasted codes, and where the pasted path is offered.
  - R8.14: the room-code path between two networks, and two strict NATs (which cannot meet without TURN), by hand before a release; one machine met over the relays on 2026-11-02.
  - R8.15: the host always wears Knight A and the guest Knight B, whatever the save's look.
  - R8.18: the online strings and the glossary's Desync, Pasted codes, Relay and Room code entries.

- M6, the ladder (branch `m6-ladder`; shots in the coordinator's scratchpad `m6/`):
  - R9.1: how the ladder is climbed: F to S, the next rung opened by a win, one mark for a win and a second for a win without losing a round.
  - R9.4: every number in the seven opponents' pilots is the agent's guess; judge the climb from play.
  - R9.5: the scripted probe finds rung D harder than C, and Practice's knight harder than B and A.
  - R9.8: the ladder's strings, the dossiers, and the glossary's Dossier, Look, Mark, Rank, Rank card and Rung.
  - R9.10: wearing the champion's look, its gilt ridge shows when the player looks down; keep it or cull it.
  - R9.13: the ladder page's look (the cards, their borders, the marks as diamonds).

- R3.10 (person; journal `#2026-11-02-m1`; default: cull the forearm near the eye): a chop's wind-up lifts the sword forearm across the eye for a few frames (`../chumworld-arena-journal/img/build/m1/chop-wind-up-arm.jpg`): accept it, cull the forearm near the eye, or move the eye back.
- R3.10 (still *person*): M4 applied the default (cull the forearm near the eye, R6.14); shot `scratchpad/m4/chop-windup-r310.jpg`, to be copied into the journal by the coordinator.
- R13.6 (person): watch mode (T11, CA-Q17: A) is the agent's layout and words: the controls at the left, the shown tree at the right, each check and move named in plain words in `data/strings.json`, the camera and the starting pair.
- R12.5 (person): the beat aside (T12, CA-Q22: A): a parry, or a kick into a held guard, throws the other blade aside, stuns it 24 ticks and lets it pass through blades for 45; the numbers are in `data/chivalry.json` for the person to judge in play.
- R11.9 (person): the animation pass (T10) is the agent's first guess from stills of gameplay video: the eye's tilt with a cut, a blow, a kick and a block; the red edge on a blow taken; the walking bob; the stippled trail band; blood kept on a blade. Each number is in `feel.rs`, `player.rs` and `swing.rs` (R11.3–R11.8) for the person to judge in play. Not done: two hands in view, motion blur, the opponent's wind-up.
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

- Ladder costumes (person; journal `#2026-11-02-ladder-costumes`):
  - R7.107, R7.208, R7.309: the seven opponents' working labels on screen.
  - R7.304: the mirror's hidden visor gap (kept by default).

## Mistakes so far

By system, from `SECOND-ORDER.md`, with the rule each one produced.

## What passed

2026-11-02, M0: `./scripts/check.sh` ends `BROWSER CHECK PASSED in chromium, firefox, webkit` and `BUILDER CHECK PASSED`; `./scripts/count-tests.sh` prints `237 passed, 0 failed, 1 ignored`.

2026-11-02, M1 and M2: `./scripts/check.sh` ends `BROWSER CHECK PASSED in chromium, firefox, webkit` and `BUILDER CHECK PASSED`; `./scripts/count-tests.sh` prints `263 passed, 0 failed, 1 ignored`; `port_keeps_chumworld_duel` passes unchanged.

2026-11-02, M3 (branch `m3-knights`): `./scripts/check.sh` ends `BROWSER CHECK PASSED in chromium, firefox, webkit` and `BUILDER CHECK PASSED`; `./scripts/count-tests.sh` prints `257 passed, 0 failed, 1 ignored`; `python3 scripts/break_tests.py M3` catches all 40 rows whose names hold M3.

2026-11-02, M3 merged into main (M1, M2 and M3 together): `./scripts/check.sh` ends `BROWSER CHECK PASSED in chromium, firefox, webkit` and `BUILDER CHECK PASSED`; `./scripts/count-tests.sh` prints `287 passed, 0 failed, 1 ignored`; `python3 scripts/break_tests.py` catches 326 of 326 (325 on the first full run; the rotted M1 cull row repointed, R5.23); `port_keeps_chumworld_duel` passes with its test data unedited. The M3 branch's own line above held before its last commit's `println!` (R5.21).

2026-11-02, M6 rungs F, E and D (branch `ladder-costumes-1`, R7.101): `./scripts/check.sh` passes steps 1 to 7 (format, clippy, `cargo test`, the release build, the import audit, art, the notebook); step 8 fails only because a worktree's folder is `wt-costumes-1`, so the journal it looks for is `../wt-costumes-1-journal/`: `scripts/journal_check.py ../chumworld-arena` on the real journal passes; step 9, `tests/browser/check.py`, ends `BROWSER CHECK PASSED in chromium, firefox, webkit`. `./scripts/count-tests.sh` prints `266 passed, 0 failed, 1 ignored`; `python3 scripts/break_tests.py "M6 FED"` catches all 13 rows.

2026-11-02, M6 costumes C and B (branch `ladder-costumes-2`): `./scripts/check.sh` passes steps 1 to 7 and stops at step 8 only because this worktree's folder name points the journal check at `../wt-costumes-2-journal`; `python3 scripts/journal_check.py ../chumworld-arena` passes, and `tests/browser/check.py` run alone ends `BROWSER CHECK PASSED in chromium, firefox, webkit`. `./scripts/count-tests.sh` prints `274 passed, 0 failed, 1 ignored`; `python3 scripts/break_tests.py "M6 "` catches 17 of 17. Rows R7.201 to R7.212.
2026-11-02, the three ladder-costume branches merged into main (55d29bd): `./scripts/check.sh` ends `BROWSER CHECK PASSED in chromium, firefox, webkit` and `BUILDER CHECK PASSED`; `./scripts/count-tests.sh` prints `326 passed, 0 failed, 1 ignored`; then, run alone, `python3 scripts/break_tests.py` catches 372 of 372 (every branch's rows kept, one rotted M3 row repointed, R5.26, one added, R5.27). The tick probe's worst tick passed 2 ms once under another agent's build (R5.28).
2026-11-02, M5 (branch `m5-online`, in the worktree `wt-m5`): steps 1 to 7 and 9 of `./scripts/check.sh` run one by one (step 8 looks for `../wt-m5-journal`; `python3 scripts/journal_check.py /Users/samgilson/Documents/ChumworldArena/chumworld-arena` run in its place); results in the commit's report. `tests/browser/online.py chromium firefox webkit`: ONLINE CHECK PASSED in all three. `port_keeps_chumworld_duel` passes with its data unedited. `python3 scripts/break_tests.py`: 358 of 360, the two misses repaired and caught alone (R8.19).
2026-11-02, M4 (branch `m4-chivalry`, in its worktree, before T7 and T8): `cargo fmt --check`, `cargo clippy -D warnings` and `cargo test` pass; `./scripts/count-tests.sh` prints `310 passed, 0 failed, 1 ignored`; `scripts/build.sh` (1 071 KB), `wasm_imports.py`, `art_check.py` and `notebook_check.py` pass; `journal_check.py` run on the main checkout passes (check.sh's step 8 looks for `../wt-m4-journal` in a worktree); `tests/browser/check.py` ends `BROWSER CHECK PASSED in chromium, firefox, webkit`; `python3 scripts/break_tests.py` catches 370 of 370; `port_keeps_chumworld_duel` passes with its data unedited.

2026-11-02, M4 with T7 and T8 merged with main (M5 and the ladder costumes) on `m4-chivalry`: steps 1 to 7 and 9 of `./scripts/check.sh` run one by one in the worktree (fmt, clippy, `cargo test`, build 1 369 KB, import audit, art, notebook; `BROWSER CHECK PASSED in chromium, firefox, webkit`), step 8 as `python3 scripts/journal_check.py` on the main checkout; `python3 tests/browser/online.py chromium firefox webkit` ends `ONLINE CHECK PASSED in chromium, firefox, webkit`; `./scripts/count-tests.sh` prints `385 passed, 0 failed, 2 ignored`; `python3 scripts/break_tests.py M4 T7 T8 M5 "held left button"` catches 92 of 92 (the full list, 376 of 376, was run on the branch before the merge); `port_keeps_chumworld_duel` passes with its data unedited.
2026-11-03, M6 the ladder (branch `m6-ladder`, in the worktree `wt-m6`): `./scripts/check.sh` with only step 8 pointed at the main checkout (`journal_check.py /Users/samgilson/Documents/ChumworldArena/chumworld-arena`) ends `BROWSER CHECK PASSED in chromium, firefox, webkit` and `BUILDER CHECK PASSED`; `tests/browser/online.py chromium firefox webkit` ends `ONLINE CHECK PASSED`; `./scripts/count-tests.sh` prints `399 passed, 0 failed, 3 ignored`; `python3 scripts/break_tests.py` on the 29 new M6 rows and the four repointed catches 33 of 33 (the full list was not run); build 1 415 KB.

## The next action

Merge `m6-ladder` into main, run the full break list once, then publish under T1's grant so T6's "deploy that too" is met; the M6 journal entry with the shots in `scratchpad/m6/`.
