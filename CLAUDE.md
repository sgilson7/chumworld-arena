# Instructions for a coding agent in a Builder project

`AGENTS.md` holds the same rules for agents that read that file instead.

## Where code goes

1. Read `BRIEF.md`, `BUILDER-SPEC.md` and `GLOSSARY.md` before changing code.
2. The core (`crates/core`) owns every application decision and performs no input or output. Scripts and tools that need a rule read the core's output; they do not recompute it.
3. The shim (`crates/shim`) translates values and decides no application behaviour.
4. The page (`web/`) draws what the core returns and does not reproduce a core rule. The core describes the screen (`ui`) and takes settings, screens and flags through one function (`act`). Presentation numbers go in `data/*.json`; shaders go in `data/shaders/` and take rule values as uniforms.

## How to work

5. Run `cargo test` after each meaningful core change.
6. For each new behaviour test, add a row to `tests/breaks.json` and run `python3 scripts/break_tests.py`: the test must fail with its behaviour broken. Work expected values out by hand; never compare a value with the function under test. When a test needs a workaround to pass, the workaround is the finding: record it.
7. Keep `SECOND-ORDER.md` while you work: one row per assumption, surprise, divergence from the plan, or item the person must do, written when you notice it. It is how the person sees what you decided alone. A row that produced a rule names where the rule now lives; a row with status *person* is named in `HANDOFF.md`.
8. A scripted edit that replaces text asserts that it matched. A count in a document comes from `./scripts/count-tests.sh`.
9. Do not deploy unless `BRIEF.md`'s Autonomy paragraph says you may. Deploying is otherwise the person's decision (stage 8).
10. Before handoff, run `./scripts/check.sh` and record the result in `HANDOFF.md`.

## Words

11. Use each term in the sense `GLOSSARY.md` gives it, and never a word from its `Not:` line. A new term gets an entry before it appears in a string.
12. Reader-visible text follows `TONE.md`. A string that looks wrong is reported in `SECOND-ORDER.md`, not repaired. A string you write yourself is listed at the foot of `SECOND-ORDER.md` for the person to read. A list you generate that a reader will see (names, labels, word lists) is a worklist row for the person.

## The design journal

13. Keep the design journal unless `BRIEF.md`'s Journal section says `none`. Start it with `python3 tools/journal/new_journal.py`; it lives beside the project (by default `../<this folder>-journal/`) as its own repository and is never pushed. `tools/journal/ENTRIES.md` says what each kind of entry holds.
14. Before writing `PLAN.md`, write the journal's opening entry: read Builder and the brief back to the person, with numbered rules and what holds each, the question prefix, where you stop, and your questions. When the person answers, each rule that applies to every later session goes under "Rules agreed for this project" below, and each rule the core can enforce becomes a test.
15. Every decision put to the person, visible change, divergence from the plan and deploy gets a journal entry when it happens, citing its rows (T4, R2.3, XX-Q5). Never remove an entry. `scripts/journal_check.py` checks the form; commit in the journal's repository after each change.

## Rules agreed for this project

Agreed 2026-11-02 in the journal (Builder page, `#opening`, and the Game page, `#2026-11-02-for-you`; answers in the published journal's store).

- CA-1. Every combat rule lives in the core, in Q16.16 integers; the page draws instances and reads input.
- CA-2. CHUMWORLD's combat is copied in at commit d9cb5b5 before it is changed (CA-Q1: A). Each change to one of its numbers is a `SECOND-ORDER.md` row naming the old value.
- CA-3. The computer's knight moves only through the same input record a player makes.
- CA-4. Nothing off the origin loads until Host or Join; every relay is in `web/net/config.js` and the CSP and nowhere else (`online_fence.rs`).
- CA-5. Online play is rollback (CA-Q10: A), with the kit's lockstep tests kept as the reference rollback must agree with; a checksum mismatch is reported.
- CA-6. A duel's core tick stays under 2 ms for two fighters with rollback room; probe readings go in `MEASUREMENTS.md`.
- CA-7. Each knight and each ladder opponent's costume is built from primitives in code through Ape Race's costume flow (concept sheet, model, tests for silhouette, z-fighting and a triangle cap, shots, review entry). Knights A and B are approved as drawn (CA-Q13: A). Costume models go to background agents in their own worktrees once the plate contract is fixed (CA-Q3: A).
- CA-8. No name, likeness, asset or line from Dark Souls, Chivalry or Armored Core in the game (`GLOSSARY.md` Not: lines, `copy.rs`).
- CA-9. Every visible change, divergence and deploy gets a journal entry with pictures of the running build.
- CA-10. Feel is the person's judgement from play. The person asked for no stops until the MVP is deployed, so the play stops after M1 and M4 are replaced by journal entries with shots for later review (R1.3).
- CA-11. Publishing is granted in `BRIEF.md`'s Autonomy paragraph for the MVP and the ladder.
- Design answers: the mouse turns view and body, the guard rides with the view, right button frees the hand, a held left button reads a gesture (CA-Q6: A); stamina, feints, combos, ripostes and a kick (CA-Q7: B); plate cuts 70% of a cut and 20% of a thrust, gaps take full damage, battered plate falls off, limbs sever only bare (CA-Q8: A); unlocks change looks only (CA-Q9: A); a real stone ring (CA-Q11: A); one two-handed longsword for both knights (CA-Q12: A); best of three, sudden death after 90 s inside the inner 12 m (CA-Q14: A); 100–110° field of view and no visor edge (CA-Q15: B); no aim help by default, with a soft pull as a setting (CA-Q16: C and the person's note); one browser save for settings and look from the MVP, the ladder from M6, plus a JSON file (CA-Q4: C).

## Stopping

Write the journal's opening entry, then `PLAN.md` from the brief, and stop until the person has read it, unless `BRIEF.md`'s Autonomy paragraph approves the plan in advance. When a rule in `BUILDER-SPEC.md` seems to block the work, stop and say so in a worklist row; do not loosen a check to make it pass. When a request is refused on safety grounds, leave the feature out and say so; do not reword the request to get past the refusal.
