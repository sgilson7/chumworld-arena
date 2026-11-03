# Chumworld Arena

A first-person sword duel for a desktop browser. You steer a simulated sword arm with the mouse and your feet with WASD, from inside a knight's helmet, in a stone ring. Play it at https://sgilson7.github.io/chumworld-arena/.

- **Practice:** a best-of-three duel against the computer's knight, which moves through the same input a player gives.
- **Host and Join:** the same duel against another browser, met by a room code or by pasted codes. Nothing loads from another site until you press one.
- **Controls:** hold the left button and swipe to cut; hold the right button to guard, or press it as a cut draws back to feint; F kicks; Esc pauses. Cuts, parries, feints and kicks cost stamina.

The combat is CHUMWORLD's (same author), copied at commit d9cb5b5 and changed as `SECOND-ORDER.md` records. The game is built under Builder: the rules are in `BUILDER-SPEC.md`, a coding agent's instructions in `CLAUDE.md` and `AGENTS.md`, each term in `GLOSSARY.md`, and the rules for every string in `TONE.md`.

## Build, test and serve

```bash
./scripts/setup.sh              # once: Python venv with Playwright and three browsers
cargo test --workspace          # the core, boundary, copy and storage tests, in seconds
./scripts/build.sh              # release WebAssembly build into dist/
python3 scripts/serve.py        # http://127.0.0.1:8000/
./scripts/check.sh              # everything, before publishing
python3 scripts/break_tests.py  # each row of tests/breaks.json must make its test fail
./scripts/count-tests.sh        # the test count to quote in a document
./scripts/publish.sh            # stage 8: push dist/ to gh-pages, from a clean tree
python3 tools/journal/new_journal.py  # once: start the design journal beside this folder
```

Needs Rust (pinned in `rust-toolchain.toml`), `wasm-bindgen-cli` 0.2.127, and Python 3. Rendering art needs `pdflatex` and `pdftocairo`; the check does not.

## Layout

| Path | Holds |
| --- | --- |
| `crates/core` | every application rule; no input or output; `api.rs` is the door: `start`, and `act(name, value)` |
| `crates/shim` | the `wasm-bindgen` functions the page calls; each is one call into the core |
| `web/` | the static page: a generic renderer for the core's view model |
| `data/` | reader-visible text, presentation numbers, shaders |
| `GLOSSARY.md`, `TONE.md` | what each term means; how every string is written |
| `tools/art/`, `styles/default/` | the art pipeline and its fallback style pack |
| `tools/journal/` | the design journal's page, its entry kinds, and the script that starts it in `../<this folder>-journal/` |
| `tests/browser/` | the browser check and this tool's interactions |
| `tests/breaks.json` | one deliberate break per rule, for `scripts/break_tests.py` |
| `scripts/` | build, check, serve, publish, import audit, mutation audit, test count |
