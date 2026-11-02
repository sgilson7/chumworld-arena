//! M2's probe: a whole duel in rounds runs to a result. The player is
//! scripted from the world as a reader would play (turn toward the knight,
//! step in, cut every 3/4 s, a level cut and a chop in turn); the knight is
//! its pilot, through the same input record (CA-3). Best of three, sudden
//! death after 90 s: the duel must end, and the probe prints how many ticks
//! it took, for MEASUREMENTS.md:
//!
//! ```bash
//! cargo test --release -p tool_core --test duel_probe -- --nocapture
//! ```

use tool_core::fx;
use tool_core::geom::yaw_of;
use tool_core::input::{self, Input};
use tool_core::rounds::Phase;
use tool_core::world::{Control, World};

/// The player's record this tick, read off the world.
fn script(w: &World, t: u32) -> Input {
    let me = w.player.pos;
    let Some(foe) = w.foes.first() else {
        return Input::default();
    };
    let gap = fx::wrap(yaw_of(foe.body.pos.sub(me)) - w.aim.yaw);
    // 100 px turns 2 184 angle units: the turn that faces the knight
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

#[test]
fn a_scripted_player_against_the_knight_runs_to_the_duel_s_result() {
    let mut w = World::new(9);
    w.settings.control = Control::FirstPerson;
    w.act("rounds.begin", "knight").unwrap();
    let most = 60 * 60 * 10; // ten minutes: far past three rounds of sudden death
    let mut t = 0;
    let mut rounds = Vec::new();
    while t < most {
        let i = script(&w, t);
        let fighting = w.rounds.as_ref().unwrap().phase == Phase::Fight;
        w.step(&i);
        t += 1;
        let r = w.rounds.as_ref().unwrap();
        // a fall: the fight gives way to the beat (counted once, though a
        // hard blow's hit-stop holds the beat still for a few ticks)
        if fighting && matches!(r.phase, Phase::Beat(_)) {
            rounds.push((r.round, t, r.fall));
        }
        if r.phase == Phase::Over {
            break;
        }
    }
    let r = w.rounds.as_ref().unwrap();
    println!(
        "duel over in {t} ticks ({} s); rounds won {:?}; winner {:?}; falls {rounds:?}",
        t / 60,
        r.wins,
        r.winner()
    );
    assert_eq!(r.phase, Phase::Over, "the duel ended within ten minutes");
    assert!(r.winner().is_some());
    assert!((2..=5).contains(&rounds.len()), "{} rounds", rounds.len());
}
