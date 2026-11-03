//! M6's probe: the scripted player of `duel_probe.rs` (turn toward the
//! foe, step in, cut every 3/4 s, a level cut and a chop in turn) against
//! each rung's pilot, in Knight A's plate with the Chivalry layer on, as a
//! ladder fight plays it. Each rung is fought from several seeds; the probe
//! prints the duels won, the rounds won each way and the mean length, for
//! MEASUREMENTS.md. It asserts only that every duel ends; whether the climb
//! feels right is the person's (CA-10).
//!
//! ```bash
//! cargo test --release -p tool_core --test ladder_probe -- --ignored --nocapture
//! ```

use tool_core::fx;
use tool_core::geom::yaw_of;
use tool_core::input::{self, Input};
use tool_core::ladder::RUNGS;
use tool_core::rounds::Phase;
use tool_core::world::{Control, World};

/// The player's record this tick, read off the world (`duel_probe.rs`).
fn script(w: &World, t: u32) -> Input {
    let me = w.player.pos;
    let Some(foe) = w.foes.first() else {
        return Input::default();
    };
    let gap = fx::wrap(yaw_of(foe.body.pos.sub(me)) - w.aim.yaw);
    let turn = (-(gap as i64) * 10_000 / 2_184).clamp(-6_000, 6_000) as i32;
    let near = me.dist(foe.body.pos) < fx::int(2);
    let k = t % 45;
    let mut i = Input {
        keys: if near { 0 } else { input::FORWARD },
        dx: turn,
        ..Input::default()
    };
    if k < 30 {
        i.buttons = input::SWING;
        i.dx = 0;
        if k == 0 {
            if (t / 45).is_multiple_of(2) {
                i.dx = -6_000;
            } else {
                i.dy = 6_000;
            }
        }
    }
    i
}

/// One duel against `pilot` from `seed`: the rounds won and its ticks.
fn duel(pilot: &str, seed: u64) -> ([u8; 2], u32) {
    let mut w = World::new(seed);
    w.settings.control = Control::FirstPerson;
    w.settings.chivalry = true;
    w.act("duel.me", "knight_a").unwrap();
    w.act("rounds.begin", pilot).unwrap();
    let most = 60 * 60 * 10;
    let mut t = 0;
    while t < most {
        let i = script(&w, t);
        w.step(&i);
        t += 1;
        if w.rounds.as_ref().unwrap().phase == Phase::Over {
            break;
        }
    }
    let r = w.rounds.as_ref().unwrap();
    assert_eq!(r.phase, Phase::Over, "{pilot} seed {seed}: the duel ended");
    (r.wins, t)
}

#[test]
#[ignore]
fn the_scripted_player_climbs_the_ladder() {
    let seeds: Vec<u64> = (1..=32).collect();
    for r in RUNGS {
        let mut won = 0;
        let mut rounds = [0u32; 2];
        let mut ticks = 0u64;
        for &s in &seeds {
            let (wins, t) = duel(r.key, s);
            won += (wins[0] > wins[1]) as u32;
            rounds[0] += wins[0] as u32;
            rounds[1] += wins[1] as u32;
            ticks += t as u64;
        }
        println!(
            "rung {} {}: the player won {won} of {} duels; rounds {}–{}; mean {} s",
            r.rank,
            r.key,
            seeds.len(),
            rounds[0],
            rounds[1],
            ticks / seeds.len() as u64 / 60
        );
    }
    // Practice's knight beside them, for comparison
    let mut won = 0;
    for &s in &seeds {
        let (wins, _) = duel("knight", s);
        won += (wins[0] > wins[1]) as u32;
    }
    println!("practice knight: the player won {won} of {} duels", seeds.len());
}
