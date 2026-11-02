# Licences

Every file in this repository that is not code written for it: its source, its author, its terms. The code is under the repository's licence.

## Assets

| Path | Source | Terms |
| --- | --- | --- |
| `web/art/` | drawn by `tools/art/art.py` from TikZ in this repository | this repository's licence |

## Vendored code

| Path | Project and version | Author | Licence | Checksum |
| --- | --- | --- | --- | --- |
| `web/net/vendor/trystero-nostr-0.25.4.js` | Trystero 0.25.4, Nostr strategy (`@trystero-p2p/nostr`), bundled by esm.sh with noble-secp256k1; copied from Sam's Level Twenty with the same sum (M5) | Dan Motzenbecker (Trystero); Paul Miller (noble-secp256k1) | MIT (both) | sha256 `6bfce15d72a64384cc66c2693917994e1b900f4f98c9b7a2d54e4e86f5202906`, `web/net/vendor/SHA256SUMS` |

## Code taken from other repositories

| From | What | Licence |
| --- | --- | --- |
| CHUMWORLD (`sgilson7`, same author, commit d9cb5b5; `/Users/samgilson/Documents/CHUMWORLD/chumworld`) | The combat and its renderer, copied whole then cut (`SECOND-ORDER.md` R2.1 to R2.6): `crates/core/src/` `arm`, `blade`, `camera`, `feel`, `flicker`, `fx`, `geom`, `glossary`, `health`, `input`, `look`, `mesh`, `models` (with `figure`, `held`, `tests`), `motion`, `page`, `pilot`, `player`, `pose`, `prim`, `render`, `rng`, `sky`, `swing`, `ticks`, `wounds` (with `tests`); parts of `world.rs`, `game.rs`, `api.rs` and the shim; `data/` `pilots`, `wounds`, `models`, `look`, `controls`, `display` and `sky` `.json` and `shaders/scene.*`; `web/input.js`, `gl.js`, `app.js` and `style.css` in part; M2: `web/save.js` (less its wipe), `crates/core/tests/one_save.rs`, and the save's checksum and refusal pattern in `crates/core/src/save.rs`; strings and glossary entries named in `SECOND-ORDER.md`. CHUMWORLD's own `fx.rs`, `rng.rs`, `ticks.rs` and `input.js` came from Builder's realtime kit (MIT, Sam Gilson) | MIT, Sam Gilson (CHUMWORLD's `LICENSE`) |

| Builder's online kit (`sgilson7/builder-setting`, `kits/online/`) | M5: `crates/core/src/online.rs` (changed: the record on the wire, `Sim`, `Link`), `crates/core/tests/online_fence.rs`, `web/net/config.js` (filled in), `tests/browser/online.py` (filled in), the `boundary.rs` patch, the vendor README's form. `web/net/rtc.js` and `web/online.js` were written for this project to the kit's design, which is Peter's (ptonkle, Mandible and Ape Race; no code of his copied); Sam's Level Twenty transport (MIT, Sam Gilson) was read for reference, its SDP shrink and grow carried over in form | MIT, Sam Gilson |

A file with no licence, a likeness, a recording or any third-party picture is a worklist row in `SECOND-ORDER.md` for the person to decide, and stays out of git until they do.
