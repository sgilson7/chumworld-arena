# Licences

Every file in this repository that is not code written for it: its source, its author, its terms. The code is under the repository's licence.

## Assets

| Path | Source | Terms |
| --- | --- | --- |
| `web/art/` | drawn by `tools/art/art.py` from TikZ in this repository | this repository's licence |

## Vendored code

| Path | Project and version | Author | Licence | Checksum |
| --- | --- | --- | --- | --- |

## Code taken from other repositories

| From | What | Licence |
| --- | --- | --- |
| CHUMWORLD (`sgilson7`, same author, commit d9cb5b5; `/Users/samgilson/Documents/CHUMWORLD/chumworld`) | The combat and its renderer, copied whole then cut (`SECOND-ORDER.md` R2.1 to R2.6): `crates/core/src/` `arm`, `blade`, `camera`, `feel`, `flicker`, `fx`, `geom`, `glossary`, `health`, `input`, `look`, `mesh`, `models` (with `figure`, `held`, `tests`), `motion`, `page`, `pilot`, `player`, `pose`, `prim`, `render`, `rng`, `sky`, `swing`, `ticks`, `wounds` (with `tests`); parts of `world.rs`, `game.rs`, `api.rs` and the shim; `data/` `pilots`, `wounds`, `models`, `look`, `controls`, `display` and `sky` `.json` and `shaders/scene.*`; `web/input.js`, `gl.js`, `app.js` and `style.css` in part; M2: `web/save.js` (less its wipe), `crates/core/tests/one_save.rs`, and the save's checksum and refusal pattern in `crates/core/src/save.rs`; strings and glossary entries named in `SECOND-ORDER.md`. CHUMWORLD's own `fx.rs`, `rng.rs`, `ticks.rs` and `input.js` came from Builder's realtime kit (MIT, Sam Gilson) | MIT, Sam Gilson (CHUMWORLD's `LICENSE`) |

A file with no licence, a likeness, a recording or any third-party picture is a worklist row in `SECOND-ORDER.md` for the person to decide, and stays out of git until they do.
