//! T11 probe (ignored): the four styles and the knight, each pair fought
//! 16 times from both sides in watch mode's world (the left pilot flies the
//! player's fighter). Prints wins, the mean duel length in seconds, and how
//! often each side feinted, riposted, landed a kick and beat a blade aside,
//! counted per duel. Run with
//! `cargo test --release -p tool_core --test styles_probe -- --ignored --nocapture`.

use tool_core::chivalry::Event;
use tool_core::input::Input;
use tool_core::world::{Control, World};

const PILOTS: [&str; 5] = ["knight", "sentry", "press", "bully", "trickster"];
const SEEDS: u64 = 16;
const MOST_TICKS: u32 = 60 * 180;

#[derive(Default, Clone, Copy)]
struct Tally {
    wins: u32,
    feints: u32,
    ripostes: u32,
    kicks: u32,
    misses: u32,
    beats: u32,
}

fn duel(left: &str, right: &str, seed: u64) -> (Option<usize>, u32, [Tally; 2]) {
    let mut w = World::new(seed);
    w.settings.control = Control::Shoulder;
    w.settings.chivalry = true;
    let (kind, _) = tool_core::pilot::pilot_named(left).unwrap();
    tool_core::rounds::begin(&mut w, right, kind).unwrap();
    w.watch = tool_core::pilot::brain(left, seed ^ 0x5bd1_e995);
    let mut t = [Tally::default(); 2];
    let mut ticks = 0;
    let mut was = [None; 2];
    while ticks < MOST_TICKS {
        w.step(&Input::default());
        ticks += 1;
        for id in 0..2 {
            let f = tool_core::player::fighter(&w, id);
            // an event is new when it is not last tick's, a tick older (or
            // the same age, through a hard blow's stop)
            let new = match (f.chiv.event, was[id]) {
                (Some((e, a)), Some((p, b))) => !(e == p && (a == b + 1 || a == b)),
                (Some(_), None) => true,
                _ => false,
            };
            was[id] = f.chiv.event;
            if let (true, Some((e, _))) = (new, f.chiv.event) {
                match e {
                    Event::Feint => t[id].feints += 1,
                    Event::Riposte => t[id].ripostes += 1,
                    Event::KickLands => t[id].kicks += 1,
                    Event::KickMisses => t[id].misses += 1,
                    Event::Beats => t[id].beats += 1,
                    _ => {}
                }
            }
        }
        let r = w.rounds.as_ref().unwrap();
        if r.wins[0] >= 2 || r.wins[1] >= 2 {
            let won = if r.wins[0] >= 2 { 0 } else { 1 };
            t[won].wins += 1;
            return (Some(won), ticks, t);
        }
    }
    (None, ticks, t)
}

#[test]
#[ignore]
fn styles_probe() {
    println!("left vs right: left wins / right wins / undecided, mean s; per duel: feints, ripostes, kicks, beats (left | right)");
    for a in PILOTS {
        for b in PILOTS {
            if a == b {
                continue;
            }
            let mut sum = [Tally::default(); 2];
            let (mut undecided, mut ticks) = (0, 0u64);
            for s in 0..SEEDS {
                let (won, t, tally) = duel(a, b, 1000 + s);
                if won.is_none() {
                    undecided += 1;
                }
                ticks += t as u64;
                for k in 0..2 {
                    sum[k].wins += tally[k].wins;
                    sum[k].feints += tally[k].feints;
                    sum[k].ripostes += tally[k].ripostes;
                    sum[k].kicks += tally[k].kicks;
                    sum[k].misses += tally[k].misses;
                    sum[k].beats += tally[k].beats;
                }
            }
            let per = |x: u32| x as f64 / SEEDS as f64;
            println!(
                "{a:>9} vs {b:<9}: {:>2} / {:>2} / {:>2}, {:>5.1} s; f {:.1}|{:.1} r {:.1}|{:.1} k {:.1}|{:.1} miss {:.1}|{:.1} b {:.1}|{:.1}",
                sum[0].wins, sum[1].wins, undecided,
                ticks as f64 / SEEDS as f64 / 60.0,
                per(sum[0].feints), per(sum[1].feints),
                per(sum[0].ripostes), per(sum[1].ripostes),
                per(sum[0].kicks), per(sum[1].kicks),
                per(sum[0].misses), per(sum[1].misses),
                per(sum[0].beats), per(sum[1].beats),
            );
        }
    }
}
