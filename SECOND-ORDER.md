# Second-order notebook

Stage 6, kept by the agent as the work happens, not afterwards. `B3 Build in self-questioning`.

The code and tests are the work; this file is the record about the work. It is where a decision the agent made alone, an assumption that turned out wrong, or something only the person can do becomes visible to the person. With the plan approved in advance, it is the main place those decisions surface before the person uses the tool. [`docs/procedure.md`](https://github.com/sgilson7/builder-setting/blob/main/docs/procedure.md#the-second-order-notebook) says why.

One row per assumption, surprise or open item. **Kind** is *divergence* (what was assumed, what turned out to be true, what changed), *finding*, or *worklist* (the person's to do). Number rows by round (R1.1, R1.2, then R2.1 after the first triage) so a later round does not renumber an earlier one. **Rule** is the rule the row produced for later sessions, and where it now lives (`CLAUDE.md`, a test, `tests/breaks.json`), or blank. **Status** is *done*, *open*, or *person*; every *person* row is named in `HANDOFF.md`. `scripts/notebook_check.py` checks the form of each row and that the notebook is not empty once `PLAN.md` holds a decision, not what the rows say.

| # | Kind | Assumed or found | What is true | What changed | Rule | Status |
| --- | --- | --- | --- | --- | --- | --- |
| R1.1 | finding | The person would answer each question in the journal's chips | All 16 answered in the published journal's store on 2026-11-02; five differ from the defaults (T1–T5) | Rules moved into CLAUDE.md and AGENTS.md under "Rules agreed" | CLAUDE.md, CA-1 to CA-11 | done |
| R1.2 | divergence | CA-Q2's note would name the repository | The note was empty | The agent chose `sgilson7/chumworld-arena` and the address `sgilson7.github.io/chumworld-arena/`; the game's display name stays "Chumworld Arena" until the person names it | BRIEF.md Autonomy | person |
| R1.3 | divergence | M1 and M4 stop for the person to play (CA-10 as proposed) | The person asked for no stop until the MVP is deployed | Each becomes a journal entry with shots, for review after the deploy | CLAUDE.md CA-10 | done |

## Strings written by the agent

Each reader-visible string the agent wrote, with its key, for the person to read.

| Key | String |
| --- | --- |
