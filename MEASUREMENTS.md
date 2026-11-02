# Measurements

Stage 4, recorded by the agent before building and again at handoff. `B2 Require step-by-step justification`.

The state measured rather than recalled, so later changes can be compared with it. A probe prints numbers and asserts loose bounds at most; record its output here with the command that produced it.

| What | Command | Result | When |
| --- | --- | --- | --- |
| Toolchain | `rustc --version; wasm-bindgen --version` | rustc 1.99.0, wasm-bindgen 0.2.127 | 2026-11-02, Apple M2 Max |
| Tests | `./scripts/count-tests.sh` | 237 passed, 0 failed, 1 ignored (M0); 263 passed, 0 failed, 1 ignored (M1 and M2); 287 passed, 0 failed, 1 ignored (M3 merged) | 2026-11-02, Apple M2 Max |
| Full check | `./scripts/check.sh` | BUILDER CHECK PASSED; browser check in chromium, firefox and webkit (M3 merged) | 2026-11-02, Apple M2 Max |
| WebAssembly size | printed by `./scripts/build.sh` | 872 KB at `opt-level = "s"` with LTO (M0); 956 KB (M1 and M2, R4.12); 961 KB on the M3 branch alone; 1 022 KB with M1, M2 and M3 merged (R5.16); over PLAN.md's 600 KB budget, R2.10 | 2026-11-02, Apple M2 Max |
| Breaks caught | `python3 scripts/break_tests.py` | 245 of 249 on the first run; the four misses dealt with in R2.15, leaving 247 rows. M1 and M2: 289 of 289 (45 rows added, three dropped and seven repointed, R4.1). M3 merged: 326 of 326 (both sides' rows kept, eight added, five repointed, R5.23) | 2026-11-02, Apple M2 Max |
| Core tick, ring duel, 2 fighters (CA-6, under 2 ms) | `cargo test --release -p tool_core --test tick_probe -- --ignored --nocapture` | mean 81 µs, worst 348 µs over 3 600 ticks (native, release), M0 on the shoulder path; M2, the same duel in first person (the probe now sets the game's control): mean 122 µs, worst 319 to 694 µs over three runs; M3 merged, the probe's duel now Practice's two knights in plate (Knight A against the `knight` pilot in Knight B): mean 121 and 122 µs, worst 269 and 556 µs over two runs | 2026-11-02, Apple M2 Max |
| Parity with CHUMWORLD (CA-2) | `cargo test -p tool_core --test parity` | 600 of 600 ticks identical (M0; still after M1 and M2, on `Control::Shoulder`, R3.1) | 2026-11-02, Apple M2 Max |
| A whole duel in rounds to its result (M2) | `cargo test --release -p tool_core --test duel_probe -- --nocapture` | a scripted player (turn, step in, a level cut and a chop each 3/4 s) against the knight's pilot, seed 9: over in 976 ticks (16 s), 2–0 to the player, both falls to a chop (R4.10). M3 merged, the player in Knight A against the knight in Knight B, both with the longsword: over in 1 648 ticks (27 s), 2–0 to the player, falls to a chop and a level cut | 2026-11-02, Apple M2 Max |
| Ring instances and triangles (M2) | `ring::instances`, the flicker scan's count | 44 instances; about 19 000 triangles, most of them the crowd (R4.1) | 2026-11-02, Apple M2 Max |
| M4: tests | `./scripts/count-tests.sh` | 310 passed, 0 failed, 1 ignored (branch `m4-chivalry`) | 2026-11-02, Apple M2 Max |
| M4: core tick with the Chivalry layer on (CA-6, under 2 ms) | `cargo test --release -p tool_core --test tick_probe -- --ignored --nocapture` | the probe now turns the layer on as Practice does: mean 127, 124 and 122 µs, worst 564, 479 and 437 µs over three runs of 3 600 ticks | 2026-11-02, Apple M2 Max |
| M4: the scripted duel with the layer on (R6.13) | `cargo test --release -p tool_core --test duel_probe -- --nocapture` | seed 9, the same scripted player: over in 4 444 ticks (74 s), 2–1 to the knight (round 1 to the player, rounds 2 and 3 to the knight's chop and thrust); the knight riposted 10 times and feinted twice, and did not kick (the script never holds its guard back); the player was winded for 3 006 ticks (50 s). M3 merged: 1 648 ticks (27 s), 2–0 to the player | 2026-11-02, Apple M2 Max |
| M4: WebAssembly size | printed by `./scripts/build.sh` | 1 071 KB (1 022 KB at M3 merged, R6.19) | 2026-11-02, Apple M2 Max |
| M4: parity with CHUMWORLD (CA-2) | `cargo test -p tool_core --test parity` | `port_keeps_chumworld_duel` passes with its data unedited; the layer is off on its path (R6.1) | 2026-11-02, Apple M2 Max |

## Not measured

Devices, browsers and conditions no measurement here covers.
