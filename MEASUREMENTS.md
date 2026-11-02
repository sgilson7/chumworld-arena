# Measurements

Stage 4, recorded by the agent before building and again at handoff. `B2 Require step-by-step justification`.

The state measured rather than recalled, so later changes can be compared with it. A probe prints numbers and asserts loose bounds at most; record its output here with the command that produced it.

| What | Command | Result | When |
| --- | --- | --- | --- |
| Toolchain | `rustc --version; wasm-bindgen --version` | rustc 1.99.0, wasm-bindgen 0.2.127 | 2026-11-02, Apple M2 Max |
| Tests | `./scripts/count-tests.sh` | 237 passed, 0 failed, 1 ignored (M0) | 2026-11-02, Apple M2 Max |
| Full check | `./scripts/check.sh` | | |
| WebAssembly size | printed by `./scripts/build.sh` | 872 KB at `opt-level = "s"` with LTO (M0); over PLAN.md's 600 KB budget, R2.10 | 2026-11-02, Apple M2 Max |
| Breaks caught | `python3 scripts/break_tests.py` | 245 of 249 on the first run; the four misses dealt with in R2.15, leaving 247 rows | 2026-11-02, Apple M2 Max |
| Core tick, ring duel, 2 fighters (CA-6, under 2 ms) | `cargo test --release -p tool_core --test tick_probe -- --ignored --nocapture` | mean 81 µs, worst 348 µs over 3 600 ticks (native, release) | 2026-11-02, Apple M2 Max |
| Parity with CHUMWORLD (CA-2) | `cargo test -p tool_core --test parity` | 600 of 600 ticks identical | 2026-11-02, Apple M2 Max |

## Not measured

Devices, browsers and conditions no measurement here covers.
