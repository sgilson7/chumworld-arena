# Handoff

Stage 9, written by the agent. `C5 Consolidate in your own words`. A fresh session starts from this file, not from memory.

## What this is

M0 to M2 of a first-person sword duel for the browser: CHUMWORLD's combat (d9cb5b5) in one stone ring, seen from inside the helmet (M1: the mouse turns view and body, the guard rides the view, the right button frees the hand, the left reads a cut; 105° across, aim help off by default). M2: the real ring (`ring.rs`, `data/ring.json`), Practice as a duel of best of three rounds against the computer's knight (`rounds.rs`, pilot `knight`), sudden death after 90 s, the title, menu, settings, pause and result screens as view models, and one browser save plus a JSON file (`save.rs`, `web/save.js`). `port_keeps_chumworld_duel` still holds the port to CHUMWORLD's numbers on the world's default `Control::Shoulder` path (R3.1). The knights (M3) are being built in another worktree; the knight is drawn on the bandit's body until then.

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
- R2.14 (open, for the M0 journal entry): the ring's plain stone colours (`data/models.json`, `ring_*`) and the darker page frame are the agent's; the look is the person's to judge from the M0 shots (`../chumworld-arena-journal/img/build/m0/`).

## Mistakes so far

By system, from `SECOND-ORDER.md`, with the rule each one produced.

## What passed

2026-11-02, M0: `./scripts/check.sh` ends `BROWSER CHECK PASSED in chromium, firefox, webkit` and `BUILDER CHECK PASSED`; `./scripts/count-tests.sh` prints `237 passed, 0 failed, 1 ignored`.

2026-11-02, M1 and M2: `./scripts/check.sh` ends `BROWSER CHECK PASSED in chromium, firefox, webkit` and `BUILDER CHECK PASSED`; `./scripts/count-tests.sh` prints `263 passed, 0 failed, 1 ignored`; `port_keeps_chumworld_duel` passes unchanged.

## The next action

M3: merge the knights' worktree (models/, health.rs, wounds.rs, data/wounds.json, data/models.json; drop the unread `ring_*` colours, R4.15), then give the `knight` pilot its figure (R4.4).
