//! A probe, not a check: what a tick of the ring's duel costs (CA-6, under
//! 2 ms for two fighters). The duel is Practice's: the player in Knight A
//! and the computer's knight in Knight B on its pilot (M3; the bandit
//! before), the player cutting every second; the probe times 3 600 ticks
//! (a minute of play) and prints the mean and the worst. Numbers for
//! MEASUREMENTS.md, from a release build:
//!
//! ```bash
//! cargo test --release -p tool_core --test tick_probe -- --ignored --nocapture
//! ```
//!
//! The clock is the test's, not the core's: only `crates/core/src` is held
//! to the core's rules.

use std::time::Instant;

use tool_core::input::{self, Input};
use tool_core::world::World;

fn record(t: u32) -> Input {
    let k = t % 60;
    Input {
        keys: if t == 0 { input::DRAW } else { 0 }
            | if (t / 120).is_multiple_of(2) {
                input::FORWARD
            } else {
                input::LEFT
            },
        buttons: if (20..50).contains(&k) { input::SWING } else { 0 },
        dx: if k == 20 {
            [-6000, 6000, 0, 0][(t / 60 % 4) as usize]
        } else {
            40
        },
        dy: if k == 20 {
            [0, 0, 6000, -6000][(t / 60 % 4) as usize]
        } else {
            0
        },
        wheel: 0,
    }
}

#[test]
#[ignore]
fn tick_cost_of_a_two_fighter_duel() {
    let mut w = World::new(5);
    // M1: the game's own control, from the helmet
    w.settings.control = tool_core::world::Control::FirstPerson;
    w.settings.chivalry = true; // M4: as Practice plays it (R6.1)
                                // M3: Practice's two knights in plate, the computer's on its own pilot
    w.act("duel.me", "knight_a").unwrap();
    w.act("duel.spawn", "knight").unwrap();
    let mut worst = 0u128;
    let start = Instant::now();
    let n = 3600;
    for t in 0..n {
        let at = Instant::now();
        w.step(&record(t));
        worst = worst.max(at.elapsed().as_nanos());
        // a downed fighter is stood up again, so every tick is a live duel
        for f in std::iter::once(&mut w.me).chain(w.foes.iter_mut().map(|f| &mut f.fighter)) {
            if f.health.down() {
                f.health = tool_core::health::Health::new(f.health.most);
            }
        }
    }
    let mean = start.elapsed().as_nanos() / n as u128;
    println!(
        "ring duel, 2 fighters, {n} ticks: mean {} µs a tick, worst {} µs",
        mean / 1000,
        worst / 1000
    );
}
