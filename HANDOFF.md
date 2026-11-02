# Handoff

Stage 9, written by the agent. `C5 Consolidate in your own words`. A fresh session starts from this file, not from memory.

## What this is

M0 of a first-person sword duel for the browser: CHUMWORLD's combat (d9cb5b5) cut down to one flat stone ring and two fighters, the player and a bandit on CHUMWORLD's pilot, still in CHUMWORLD's third-person camera. A start screen with Practice, the duel with each fighter's health, and pause. `port_keeps_chumworld_duel` holds the port to CHUMWORLD's numbers.

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

- R1.2: the game's name and the repository name. The agent chose `sgilson7/chumworld-arena` and the display name "Chumworld Arena" because CA-Q2's note named neither.
- R2.14 (open, for the M0 journal entry): the ring's plain stone colours (`data/models.json`, `ring_*`) and the darker page frame are the agent's; the look is the person's to judge from the M0 shots (`../chumworld-arena-journal/img/build/m0/`).

## Mistakes so far

By system, from `SECOND-ORDER.md`, with the rule each one produced.

## What passed

2026-11-02, M0: `./scripts/check.sh` ends `BROWSER CHECK PASSED in chromium, firefox, webkit` and `BUILDER CHECK PASSED`; `./scripts/count-tests.sh` prints `237 passed, 0 failed, 1 ignored`.

## The next action

M1: first person (PLAN.md).
