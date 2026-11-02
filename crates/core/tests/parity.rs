//! CA-2's proof that the port kept CHUMWORLD's combat: a scripted duel
//! recorded in a scratch copy of CHUMWORLD at d9cb5b5 (on the level ground
//! of its test duel's spot, the bandit driven by its own pilot from the
//! world's seed, its rocks taken from it as the ring's bandit has none,
//! R2.3) replays in the ring to the same numbers, tick for tick:
//! both fighters' feet, facings and six arm points (elbows, hands, pommel,
//! tip), their health, the hit-stop and the player's aim.
//!
//! The recorder is `crates/core/tests/parity_trace.rs` in that copy, never
//! committed there; its text is kept beside the trace
//! (`tests/data/chumworld_duel_recorder.rs.txt`). The script below is its
//! script, word for word. x and z are taken from the player's feet after
//! the spawn, since CHUMWORLD's duel stood 2 km from its origin and the
//! ring's stands at its middle (SECOND-ORDER.md R2.7).

use tool_core::input::{self, Input};
use tool_core::player;
use tool_core::world::World;

const SEED: u64 = 11;
const TICKS: u32 = 600;

/// The player's record for tick `t`: draw, step in, then cuts. (Neither
/// fighter goes down in it: down, CHUMWORLD's story takes the player's keys
/// away, `story::filter`, which the ring does not.)
fn script(t: u32) -> Input {
    let mut i = Input::default();
    if t == 0 {
        i.keys = input::DRAW;
    }
    if (40..70).contains(&t) || (450..470).contains(&t) {
        i.keys |= input::FORWARD;
    }
    if (500..520).contains(&t) {
        i.keys |= input::LEFT;
    }
    // the hand wanders a little between cuts
    if (30..60).contains(&t) {
        i.dx = 300;
        i.dy = -150;
    }
    // (start, dx, dy, wheel): level, chop, backhand, thrust, rising, level, chop
    let cuts = [
        (80, -6000, 0, 0),
        (140, 0, 6000, 0),
        (200, 6000, 0, 0),
        (260, 0, 0, -100),
        (320, 0, -6000, 0),
        (380, -6000, 0, 0),
        (540, 0, 6000, 0),
    ];
    for (start, dx, dy, wheel) in cuts {
        if t == start {
            i.dx = dx;
            i.dy = dy;
            i.wheel = wheel;
        }
        if (start..start + 40).contains(&t) {
            i.buttons |= input::SWING;
        }
    }
    i
}

/// The duel in the ring, a row a tick, as the recorder writes them.
fn replay() -> Vec<Vec<i64>> {
    let mut w = World::new(SEED);
    w.act("duel.spawn", "bandit").unwrap();
    let origin = w.player.pos;
    let mut rows = Vec::new();
    for t in 0..TICKS {
        w.step(&script(t));
        let mut row = vec![
            w.tick as i64,
            w.feel.stop as i64,
            w.aim.yaw as i64,
            w.aim.pitch as i64,
        ];
        for id in 0..player::count(&w) {
            let b = player::body(&w, id);
            let f = player::fighter(&w, id);
            row.push(f.health.hp as i64);
            row.push(b.facing as i64);
            for p in [b.pos].iter().chain(f.arm.pts.iter().map(|q| &q.p)) {
                row.push((p.x - origin.x) as i64);
                row.push(p.y as i64);
                row.push((p.z - origin.z) as i64);
            }
        }
        rows.push(row);
    }
    rows
}

fn recorded() -> Vec<Vec<i64>> {
    let v: serde_json::Value = serde_json::from_str(include_str!("data/chumworld_duel.json")).unwrap();
    assert_eq!(v["seed"], SEED);
    assert_eq!(v["ticks"], TICKS);
    serde_json::from_value(v["rows"].clone()).unwrap()
}

#[test]
fn port_keeps_chumworld_duel() {
    let (want, got) = (recorded(), replay());
    assert_eq!(want.len(), TICKS as usize);
    for (t, (a, b)) in want.iter().zip(&got).enumerate() {
        assert_eq!(a, b, "the ring parts from CHUMWORLD at tick {}", t + 1);
    }
}

/// The trace is a duel and not a still: both are hurt, by blows that stop
/// the world, and neither goes down.
#[test]
fn the_recorded_duel_lands_blows_both_ways() {
    let rows = recorded();
    let hp = |r: &Vec<i64>| (r[4], r[4 + 2 + 21]);
    assert_eq!(hp(&rows[0]), (100, 90));
    // the player's 100 taken to 12 and the bandit's 90 to 55 by tick 600
    assert_eq!(hp(&rows[599]), (12, 55));
    let stops = rows.iter().filter(|r| r[1] > 0).count();
    assert!(stops > 20, "{stops} ticks of hit-stop");
}
