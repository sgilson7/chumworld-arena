//! A probe, not a check: what a frame of an online duel costs at rollback's
//! worst (CA-6, under 2 ms for two fighters with rollback room). Numbers for
//! MEASUREMENTS.md, from a release build:
//!
//! ```bash
//! cargo test --release -p tool_core --test rollback_probe -- --ignored --nocapture
//! ```
//!
//! The host's session (`rollback.rs`) is kept at the edge of its window
//! the whole duel: each frame the guest's true record arrives for the
//! oldest guessed tick, and it is never the guess (the guest's record flips
//! every tick), so every frame puts back the copy from 8 ticks ago, runs
//! those 8 ticks again and then the new one: 9 ticks, 9 copies of the world
//! and the copy put back. That is the most one frame can be asked to do.
//! The host cuts every second and both walk, as in `tick_probe.rs`.
//!
//! The clock is the test's, not the core's: only `crates/core/src` is held
//! to the core's rules.

use std::time::Instant;

use tool_core::input::{self, Input};
use tool_core::online::{encode, Msg, Sim, DELAY};
use tool_core::rollback::{Rollback, WINDOW};

fn host_record(t: u32) -> Input {
    let k = t % 60;
    Input {
        keys: if (t / 120).is_multiple_of(2) {
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

/// The guest's record: never what the last one was, so every guess is wrong.
fn guest_record(t: u32) -> Input {
    Input {
        keys: if t.is_multiple_of(2) {
            input::FORWARD
        } else {
            input::FORWARD | input::RIGHT
        },
        dx: if t.is_multiple_of(2) { 300 } else { -300 },
        ..Input::default()
    }
}

#[test]
#[ignore]
fn frame_cost_with_a_full_rollback_every_frame() {
    // both knights too tough to fall, so every tick is a live duel (a
    // health a rollback would put back if it were set mid-duel)
    let mut start = tool_core::seat::duel();
    for f in std::iter::once(&mut start.me).chain(start.foes.iter_mut().map(|f| &mut f.fighter)) {
        let plate = f.health.limbs.plate;
        f.health = tool_core::health::Health::new(100_000);
        f.health.limbs.plate = plate;
    }
    let mut host = Rollback::new(0, "probe", start.clone());
    let mut guest = Rollback::new(1, "probe", start);
    host.connected(2);
    guest.connected(1);
    for (_, text) in host.take_outbox() {
        guest.message(1, &text);
    }
    for (_, text) in guest.take_outbox() {
        host.message(2, &text);
    }
    // fill the window: the host runs WINDOW ticks on guesses
    while !host.held_back() {
        let t = host.tick;
        host.local(host_record(t + DELAY));
        host.step();
    }
    let n = 3600u32;
    let (mut worst, mut total) = (0u128, 0u128);
    let mut each = Vec::with_capacity(n as usize);
    for _ in 0..n {
        // the guest's true record for the oldest guessed tick
        let t = host.frontier;
        host.message(
            2,
            &encode(&Msg::Input {
                tick: t,
                input: guest_record(t),
            }),
        );
        host.local(host_record(host.tick + DELAY));
        host.take_outbox();
        let at = Instant::now();
        assert!(host.step(), "the window moved on by one");
        let e = at.elapsed().as_nanos();
        worst = worst.max(e);
        total += e;
        each.push(e);
    }
    each.sort_unstable();
    let pct = |p: usize| each[(each.len() * p / 1000).min(each.len() - 1)] / 1000;
    let over = each.iter().filter(|&&e| e > 2_000_000).count();
    let r = host.world.rounds.as_ref().unwrap();
    assert_eq!(r.round, 1, "no one fell: every frame was a live duel");
    assert_eq!(host.stats.most, WINDOW);
    let w = host.world.clone();
    let at = Instant::now();
    let reps = 1000;
    for _ in 0..reps {
        std::hint::black_box(w.clone());
    }
    let clone_ns = at.elapsed().as_nanos() / reps;
    let at = Instant::now();
    let sum = w.sum();
    let sum_ns = at.elapsed().as_nanos();
    println!(
        "online duel, 2 knights, {n} frames each rolling back {WINDOW} ticks and running 1 more: mean {} µs a frame, median {} µs, 99th percentile {} µs, 99.9th {} µs, worst {} µs, {over} frames over 2 ms; {} rollbacks, {} ticks run again; a copy of the world {} µs; a checksum {} µs ({sum:x})",
        total / n as u128 / 1000,
        pct(500),
        pct(990),
        pct(999),
        worst / 1000,
        host.stats.rollbacks,
        host.stats.resimulated,
        clone_ns / 1000,
        sum_ns / 1000,
    );
}
