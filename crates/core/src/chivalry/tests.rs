//! M4's rules, one test each, the expected values worked by hand from
//! `data/chivalry.json` (stamina 100; a chop 12, a level cut 10, a thrust
//! 8; a parry 10; 15 a second back, after 30 ticks; winded at none until
//! 25, at 55%; a feint 15 in the first 10 ticks; follow-ups in 12 ticks,
//! three, wound up in 8; a riposte in 18 ticks, wound up in 6, at 125%,
//! pressing 6 ticks; a kick 20, landing on its 8th tick, 175 cm, 50°,
//! staggering 14 ticks, 40° aside).

use super::*;
use crate::blade::Clash;
use crate::input::{DRAW, HOLD_LOOK, KICK, SWING};
use crate::pilot::{self, Move};
use crate::swing;
use crate::world::{Control, World};

fn press(keys: u32) -> Input {
    Input {
        keys,
        ..Input::default()
    }
}

/// The player in Knight A, in first person, sword drawn and settled, with
/// the layer on (or off) and a still knight `gap` cm ahead.
fn duel(on: bool, gap: i32) -> World {
    let mut w = World::new(5);
    w.settings.control = Control::FirstPerson;
    w.settings.chivalry = on;
    w.act("duel.me", "knight_a").unwrap();
    w.act("duel.pilot", "still").unwrap();
    w.act("duel.spawn", "knight").unwrap();
    w.act("duel.gap", &gap.to_string()).unwrap();
    w.step(&press(DRAW));
    for _ in 0..player::DRAW_TICKS + 30 {
        w.step(&Input::default());
    }
    assert_eq!(w.me.sheath, Sheath::Drawn);
    w
}

/// The left button held and the mouse moved `dx`, `dy` on the first tick.
fn cut(dx: i32, dy: i32, wheel: i32, t: usize) -> Input {
    Input {
        buttons: SWING,
        dx: if t == 0 { dx } else { 0 },
        dy: if t == 0 { dy } else { 0 },
        wheel: if t == 0 { wheel } else { 0 },
        ..Input::default()
    }
}

/// Steps a chop's gesture until the cut begins; the ticks it took.
fn begin_chop(w: &mut World) -> usize {
    for t in 0..5 {
        w.step(&cut(0, 6_000, 0, t));
        if w.me.arm.swing.cut.is_some() {
            return t;
        }
    }
    panic!("no cut began");
}

#[test]
fn each_cut_costs_stamina_by_its_way_when_it_is_chosen() {
    // a chop 12 of 100, a level cut 10, a thrust 8: 88, 90, 92 points
    for ((dx, dy, wheel), left) in [
        ((0, 6_000, 0), 8_800),
        ((-6_000, 0, 0), 9_000),
        ((0, 0, -100), 9_200),
    ] {
        let mut w = duel(true, 600);
        assert_eq!(w.me.chiv.stamina, 10_000);
        w.step(&cut(dx, dy, wheel, 0));
        assert!(w.me.arm.swing.cut.is_some());
        assert_eq!(w.me.chiv.stamina, left, "{dx} {dy} {wheel}");
    }
}

#[test]
fn stamina_comes_back_fifteen_a_second_after_half_a_second_and_not_while_cutting() {
    let mut w = duel(true, 600);
    w.act("chivalry.my_stamina", "50").unwrap();
    // ticks 1 to 29 rest; from the 30th, 25 hundredths a tick (15 a second)
    for _ in 0..29 {
        w.step(&Input::default());
    }
    assert_eq!(w.me.chiv.stamina, 5_000);
    w.step(&Input::default());
    assert_eq!(w.me.chiv.stamina, 5_025);
    for _ in 0..60 {
        w.step(&Input::default());
    }
    // 61 ticks of 25
    assert_eq!(w.me.chiv.stamina, 6_525);
    // a chop: 12 points, and nothing back while it is under way
    begin_chop(&mut w);
    assert_eq!(w.me.chiv.stamina, 5_325);
    for t in 1..20 {
        // (rested as long as it likes: the cut under way still holds it)
        w.me.chiv.rest = 100;
        w.step(&cut(0, 0, 0, t));
        assert!(w.me.arm.swing.cut.is_some(), "tick {t}");
        assert_eq!(w.me.chiv.stamina, 5_325, "tick {t}");
    }
}

#[test]
fn winded_at_none_the_arms_work_at_part_strength_until_stamina_is_back_to_twenty_five() {
    let mut w = duel(true, 600);
    let strength = |w: &World| player::goals(w, 0, &Input::default()).2.strength;
    assert_eq!(strength(&w), [100, 100]);
    w.act("chivalry.my_stamina", "0").unwrap();
    assert!(w.me.chiv.winded);
    assert_eq!(strength(&w), [55, 55]);
    // back to 25 points: 30 ticks' wait, then 100 ticks of 25 hundredths;
    // the wait's 30th tick is the first to add, so the 129th tick is the
    // 100th to add
    for _ in 0..128 {
        w.step(&Input::default());
    }
    assert_eq!(w.me.chiv.stamina, 2_475);
    assert!(w.me.chiv.winded);
    w.step(&Input::default());
    assert_eq!(w.me.chiv.stamina, 2_500);
    assert!(!w.me.chiv.winded);
    assert_eq!(strength(&w), [100, 100]);
}

#[test]
fn a_winded_chop_comes_slower_than_a_fresh_one() {
    let top = |stamina: &str| {
        let mut w = duel(true, 600);
        w.act("chivalry.my_stamina", stamina).unwrap();
        let mut top = 0;
        for t in 0..60 {
            w.step(&cut(0, 6_000, 0, t));
            top = top.max(player::mps(w.me.arm.pts[crate::arm::TIP].vel().len()));
        }
        top
    };
    let (fresh, winded) = (top("100"), top("0"));
    assert!(winded * 10 < fresh * 9, "fresh {fresh} winded {winded}");
}

#[test]
fn a_blade_met_while_not_striking_is_a_parry_that_costs_the_parrier_and_opens_a_riposte() {
    let mut w = duel(true, 200);
    // the foe's cut is striking, the player's blade meets it at guard
    let mut c = swing::begin(Way::Chop, -deg(90), 0);
    c.phase = Phase::Strike;
    w.foes[0].fighter.arm.swing.cut = Some(c);
    w.blades.clashes = vec![Clash {
        a: 0,
        b: 1,
        at: V3::ZERO,
        speed: fx::int(6),
    }];
    step(&mut w, &Input::default(), &[Input::default()]);
    assert_eq!(w.me.chiv.stamina, 9_000, "100 less the parry's 10");
    // opened at 18 and this tick's closing taken from it
    assert_eq!(w.me.chiv.riposte, 17);
    assert_eq!(w.foes[0].fighter.chiv.stamina, 10_000, "the striker pays nothing");
    assert_eq!(w.foes[0].fighter.chiv.riposte, 0);
}

/// T12 (CA-Q22: A): the parried blade is beaten aside: its cut breaks off,
/// its hand's target goes 80° across to the side it was on (the still
/// knight's guard is 12° to its left, so to its left) and 45° down; it is
/// stunned 24 ticks and passes through blades 45, each less this tick's
/// count (23, 44). The parrier is told its parry beat the blade.
#[test]
fn a_parry_beats_the_striking_blade_aside_and_holds_it_out_of_line() {
    let mut w = duel(true, 200);
    let mut c = swing::begin(Way::Chop, -deg(90), 0);
    c.phase = Phase::Strike;
    w.foes[0].fighter.arm.swing.cut = Some(c);
    assert_eq!(w.foes[0].fighter.arm.shell.side, -deg(12));
    w.blades.clashes = vec![Clash {
        a: 0,
        b: 1,
        at: V3::ZERO,
        speed: fx::int(6),
    }];
    step(&mut w, &Input::default(), &[Input::default()]);
    let f = &w.foes[0].fighter;
    assert_eq!(f.arm.swing.cut, None, "its cut broken off");
    assert_eq!(
        (f.arm.shell.side, f.arm.shell.pitch),
        (crate::arm::SIDE_MIN, -deg(45)),
        "80° held to the shell's 75° on the left"
    );
    assert_eq!((f.chiv.stun, f.chiv.beaten), (23, 44));
    assert_eq!(f.chiv.event, Some((Event::Beaten, 1)));
    assert_eq!(w.me.chiv.event, Some((Event::Beats, 1)));
    assert!(crate::chivalry::passes(&w, 1) && !crate::chivalry::passes(&w, 0));
    // a blade already beaten is not beaten again by the next meeting
    w.foes[0].fighter.arm.swing.cut = Some(c);
    w.foes[0].fighter.chiv.stun = 5;
    step(&mut w, &Input::default(), &[Input::default()]);
    assert_eq!(w.foes[0].fighter.chiv.stun, 4, "not set back to 24");
}

/// T12: beaten aside, the player chooses no cut, holds no guard and makes
/// no kick until the stun is over; then it can again.
#[test]
fn beaten_aside_a_fighter_starts_no_cut_guard_or_kick_until_the_stun_ends() {
    let mut w = duel(true, 600);
    let (before, side) = (w.me.arm.shell.pitch, w.me.arm.shell.side);
    crate::chivalry::beat(&mut w, 0);
    let guard = Input {
        buttons: HOLD_LOOK,
        ..Input::default()
    };
    assert!(!crate::chivalry::guards(&w, 0, &guard), "no guard");
    w.step(&press(KICK));
    assert_eq!(w.me.chiv.kick, 0, "no kick");
    for t in 0..20 {
        w.step(&cut(-6_000, 0, 0, t % 2));
        assert_eq!(w.me.arm.swing.cut, None, "tick {t}: no cut");
    }
    // in first person the guard rides the view, but not while beaten: the
    // hand stays down, then the guard comes back to where it sat
    assert_eq!(w.me.arm.shell.pitch, -deg(45), "still thrown down");
    while w.me.chiv.stun > 0 {
        w.step(&Input::default());
    }
    w.step(&Input::default());
    assert_eq!(w.me.arm.shell.pitch, before, "the guard back where it sat");
    assert_eq!(w.me.arm.shell.side, side, "and back across");
    assert!(crate::chivalry::guards(&w, 0, &guard), "a guard again");
    w.step(&cut(-6_000, 0, 0, 0));
    assert!(w.me.arm.swing.cut.is_some(), "and a cut");
}

#[test]
fn the_right_button_in_a_cut_s_wind_up_feints_back_to_guard_for_fifteen() {
    let mut w = duel(true, 600);
    begin_chop(&mut w);
    assert_eq!(w.me.chiv.stamina, 8_800);
    // the hand's target off to the right while the cut holds it
    w.me.arm.shell.side = deg(40);
    w.step(&Input {
        buttons: SWING | HOLD_LOOK,
        ..Input::default()
    });
    assert_eq!(w.me.arm.swing.cut, None, "broken off");
    assert_eq!(w.me.chiv.stamina, 7_300, "88 less 15");
    assert_eq!(w.me.arm.shell.side, crate::arm::GUARD_SIDE);
    assert_eq!(w.me.chiv.event, Some((Event::Feint, 1)), "set, and a tick old");
    // with 14 points, too few: the cut goes on
    let mut w = duel(true, 600);
    w.act("chivalry.my_stamina", "26").unwrap();
    begin_chop(&mut w);
    assert_eq!(w.me.chiv.stamina, 1_400);
    w.step(&Input {
        buttons: SWING | HOLD_LOOK,
        ..Input::default()
    });
    assert!(w.me.arm.swing.cut.is_some());
    assert_eq!(w.me.chiv.stamina, 1_400);
    // past the first 10 ticks of the wind-up: the cut goes on
    let mut w = duel(true, 600);
    begin_chop(&mut w);
    for t in 1..=11 {
        w.step(&cut(0, 0, 0, t));
    }
    let c = w.me.arm.swing.cut.unwrap();
    assert_eq!((c.phase, c.t), (Phase::WindUp, 11));
    w.step(&Input {
        buttons: SWING | HOLD_LOOK,
        ..Input::default()
    });
    assert!(w.me.arm.swing.cut.is_some(), "too late to feint");
    assert_eq!(w.me.chiv.stamina, 8_800);
}

#[test]
fn a_cut_begun_in_the_follow_up_window_winds_up_in_eight_ticks_three_in_a_row() {
    let mut c = Chiv::new();
    let fresh = || swing::begin(Way::Chop, -deg(90), 0);
    // a plain cut, then the window: follow-ups 1, 2 and 3, then plain again
    let mut a = fresh();
    begin_cut(&mut c, &mut a);
    assert_eq!((a.mark, a.wind_ticks), (Mark::Plain, swing::WIND_TICKS));
    for n in 1..=3 {
        cut_ended(&mut c, false);
        assert_eq!(c.follow, 12);
        let mut b = fresh();
        begin_cut(&mut c, &mut b);
        assert_eq!(
            (b.mark, b.wind_ticks, b.strength_pct),
            (Mark::FollowUp(n), 8, 100)
        );
    }
    cut_ended(&mut c, false);
    let mut d = fresh();
    begin_cut(&mut c, &mut d);
    assert_eq!(
        (d.mark, d.wind_ticks, c.run),
        (Mark::Plain, 16, 0),
        "a run is three"
    );
    // a cut a blade stopped opens no window
    cut_ended(&mut c, true);
    assert_eq!(c.follow, 0);
    // in the world: a chop into the air, and a chop begun as it ends
    let mut w = duel(true, 600);
    begin_chop(&mut w);
    let mut t = 1;
    while w.me.arm.swing.cut.is_some() {
        w.step(&cut(0, 0, 0, t));
        t += 1;
        assert!(t < 90);
    }
    assert_eq!(w.me.chiv.follow, 11, "opened at 12, a tick gone");
    begin_chop(&mut w);
    assert_eq!(w.me.arm.swing.cut.unwrap().mark, Mark::FollowUp(1));
    // and its wind-up is over within 8 ticks (a plain chop's took 15 here)
    let mut wind = 0;
    while w.me.arm.swing.cut.is_some_and(|c| c.phase == Phase::WindUp) {
        w.step(&cut(0, 0, 0, 1));
        wind += 1;
    }
    assert!(wind <= 8, "{wind}");
}

#[test]
fn a_cut_begun_in_the_riposte_window_winds_up_in_six_ticks_and_drives_at_125() {
    let mut c = Chiv::new();
    c.riposte = 5;
    c.follow = 5;
    let mut r = swing::begin(Way::Level(true), fx::HALF_TURN, 0);
    begin_cut(&mut c, &mut r);
    assert_eq!(
        (r.mark, r.wind_ticks, r.strength_pct, r.press),
        (Mark::Riposte, 6, 125, 6)
    );
    assert_eq!((c.riposte, c.follow), (0, 0), "the windows spent");
    // its wind-up at 80% of 125, its strike at 125
    assert_eq!(swing::strength_pct(&r), 100);
    r.phase = Phase::Strike;
    assert_eq!(swing::strength_pct(&r), 125);
}

#[test]
fn a_riposte_presses_on_against_a_blade_for_six_ticks_and_a_plain_cut_is_spent_on_it() {
    let striking = |w: &mut World, mark_riposte: bool| {
        let mut c = swing::begin(Way::Level(true), fx::HALF_TURN, 0);
        c.phase = Phase::Strike;
        if mark_riposte {
            c.mark = Mark::Riposte;
            c.press = 6;
        }
        w.me.arm.swing.cut = Some(c);
        w.blades.clashing = vec![(0, 1)];
    };
    let mut w = duel(true, 200);
    striking(&mut w, false);
    swing::step(&mut w, 0, &cut(0, 0, 0, 1));
    assert_eq!(w.me.arm.swing.cut, None, "a plain cut is spent on the blade");
    let mut w = duel(true, 200);
    striking(&mut w, true);
    for k in (0..6).rev() {
        swing::step(&mut w, 0, &cut(0, 0, 0, 1));
        assert_eq!(w.me.arm.swing.cut.map(|c| c.press), Some(k));
    }
    swing::step(&mut w, 0, &cut(0, 0, 0, 1));
    assert_eq!(w.me.arm.swing.cut, None, "after six ticks it is spent");
}

#[test]
fn a_kick_in_reach_staggers_the_foe_breaks_its_cut_and_pushes_its_guard_aside() {
    let mut w = duel(true, 100);
    // the guards met and pushed the bodies apart as they settled: stood
    // 1.6 m apart again, straight ahead
    w.act("duel.gap", "160").unwrap();
    let side = w.foes[0].fighter.arm.shell.side;
    assert_eq!(side, -deg(12), "the still knight's guard, 12° to its left");
    w.step(&press(KICK));
    assert_eq!(w.me.chiv.stamina, 8_000, "100 less the kick's 20");
    assert_eq!(w.me.chiv.kick, 1);
    // (a hard blow's hit-stop holds the kick's count still too)
    while w.me.chiv.kick < 7 {
        w.step(&Input::default());
    }
    // the 7th tick of the kick: not yet; a cut put on the foe now
    assert_eq!(w.foes[0].fighter.chiv.event, None);
    assert_eq!(w.me.chiv.event, None);
    w.foes[0].fighter.arm.swing.cut = Some(swing::begin(Way::Chop, -deg(90), 0));
    while w.me.chiv.kick == 7 {
        w.step(&Input::default());
    }
    assert_eq!(w.me.chiv.kick, 8);
    let f = &w.foes[0].fighter;
    assert_eq!(f.gait.stagger, 14);
    assert_eq!(f.arm.swing.cut, None);
    assert_eq!(f.arm.shell.side, -deg(12) - deg(40), "40° further to its left");
    assert_eq!(f.chiv.event, Some((Event::Kicked, 1)));
    assert_eq!(w.me.chiv.event, Some((Event::KickLands, 1)));
    // shoved away from the player
    let away = w.foes[0].body.pos.sub(w.player.pos);
    assert!(w.foes[0].body.vel.dot(away) > 0);
    // the kick is over on its 30th tick
    while w.me.chiv.kick < 29 {
        w.step(&Input::default());
    }
    while w.me.chiv.kick == 29 {
        w.step(&Input::default());
    }
    assert_eq!(w.me.chiv.kick, 0);
}

/// T12: a kick into a guard held up (the knight's `guard_up`) beats the
/// blade aside as a parry does; into an open guard it does not (the kick
/// test above: pushed 40°, no stun).
#[test]
fn a_kick_into_a_held_guard_beats_the_blade_aside() {
    let mut w = duel(true, 160);
    w.step(&press(KICK));
    while w.me.chiv.kick < 4 {
        w.step(&Input::default());
    }
    w.act("chivalry.foe_move", "guard_up").unwrap();
    while w.me.chiv.kick < 8 {
        w.step(&Input::default());
    }
    let f = &w.foes[0].fighter;
    assert!(
        f.chiv.stun > 0 && f.chiv.beaten > 0,
        "stun {} beaten {}",
        f.chiv.stun,
        f.chiv.beaten
    );
    assert_eq!(f.chiv.guard, None, "the guard let go");
    assert_eq!(w.me.chiv.event.map(|e| e.0), Some(Event::Beats));
    let mut w = duel(true, 160);
    w.step(&press(KICK));
    while w.me.chiv.kick < 8 {
        w.step(&Input::default());
    }
    let f = &w.foes[0].fighter;
    assert_eq!(
        (f.chiv.stun, f.chiv.beaten),
        (0, 0),
        "an open guard is only kicked"
    );
}

#[test]
fn a_kick_out_of_reach_misses_and_one_without_twenty_stamina_is_not_made() {
    let mut w = duel(true, 300);
    w.step(&press(KICK));
    for _ in 0..7 {
        w.step(&Input::default());
    }
    assert_eq!(w.foes[0].fighter.gait.stagger, 0);
    assert_eq!(w.me.chiv.event, Some((Event::KickMisses, 1)));
    let mut w = duel(true, 100);
    w.act("chivalry.my_stamina", "19").unwrap();
    w.step(&press(KICK));
    assert_eq!(w.me.chiv.kick, 0);
    assert_eq!(w.me.chiv.stamina, 1_900);
}

#[test]
fn the_layer_is_off_on_the_port_s_path() {
    let mut w = duel(false, 100);
    assert!(!World::new(1).settings.chivalry, "World::new keeps it off");
    begin_chop(&mut w);
    assert_eq!(w.me.chiv.stamina, 10_000, "a cut costs nothing");
    let mut w = duel(false, 100);
    w.step(&press(KICK));
    assert_eq!(w.me.chiv.kick, 0, "F does nothing");
    w.act("chivalry.my_stamina", "0").unwrap();
    assert_eq!(player::goals(&w, 0, &Input::default()).2.strength, [100, 100]);
}

// ---- the computer's knight, through its input record (CA-3) ----

/// The knight's first decision at a check tick (the tick the tree is read),
/// over seeds, with the world staged by `stage`.
fn decisions(gap: i32, stage: impl Fn(&mut World)) -> Vec<Option<Move>> {
    (0..12)
        .map(|seed| {
            let mut w = duel(true, gap);
            w.foes[0].brain.control = pilot::Control::Tree;
            w.foes[0].brain.rng = crate::rng::Rng::new(seed);
            // stood `gap` apart again, the blades still (the guards pushed
            // the bodies apart as they settled)
            w.act("duel.gap", &gap.to_string()).unwrap();
            for id in 0..2 {
                for p in &mut player::parts_mut(&mut w, id).1.arm.pts {
                    p.q = p.p;
                }
            }
            // and the pilot sees the duel as it is now
            w.pilots.memory = Default::default();
            stage(&mut w);
            // (tick + id) a multiple of 6: the tree is read
            w.tick = 5;
            let (_, b) = pilot::decide(&w, 1);
            b.current.map(|(m, _)| m)
        })
        .collect()
}

fn attack(m: Option<Move>) -> bool {
    matches!(
        m,
        Some(Move::Overhead | Move::Sweep | Move::Backhand | Move::Rising | Move::Thrust)
    )
}

#[test]
fn the_knight_ripostes_after_its_blade_parries() {
    // in reach, the player's blade at rest: with the window open, every seed
    // attacks; with it shut, the chance-led attack only some
    let open = decisions(200, |w| w.foes[0].fighter.chiv.riposte = 10);
    assert!(open.iter().all(|m| attack(*m)), "{open:?}");
    let shut = decisions(200, |_| {});
    assert!(!shut.iter().all(|m| attack(*m)), "{shut:?}");
}

#[test]
fn the_knight_rests_when_its_stamina_runs_low() {
    let low = decisions(200, |w| w.act("chivalry.foe_stamina", "20").unwrap());
    assert!(low.iter().all(|m| *m == Some(Move::BackOff)), "{low:?}");
}

#[test]
fn the_knight_kicks_a_guard_held_back_at_close_range() {
    let held = decisions(110, |w| w.me.chiv.idle = 90);
    assert!(held.contains(&Some(Move::Kick)), "{held:?}");
    let busy = decisions(110, |w| w.me.chiv.idle = 89);
    assert!(!busy.contains(&Some(Move::Kick)), "{busy:?}");
    // the kick goes out as F in the knight's record
    let mut w = duel(true, 110);
    w.foes[0].brain.control = pilot::Control::Tree;
    w.foes[0].brain.current = Some((Move::Kick, 0));
    w.tick = 0;
    let (i, _) = pilot::decide(&w, 1);
    assert_eq!(i.keys & KICK, KICK);
}

#[test]
fn the_knight_feints_through_its_record_and_the_feint_holds_in_the_world() {
    // the feint's records, as the pilot sends them
    let mut w = duel(true, 600);
    w.foes[0].brain.control = pilot::Control::Tree;
    w.foes[0].brain.current = Some((Move::FeintCut, 0));
    w.tick = 0; // (0 + 1) is no multiple of 6: no check, the move plays
    let mut records = Vec::new();
    for _ in 0..20 {
        let (i, b) = pilot::decide(&w, 1);
        w.foes[0].brain = b;
        records.push(i);
    }
    // the chop's gesture at the 8th tick (110 px down), the right button
    // pressed 3 ticks on, then the button let go
    assert_eq!((records[8].buttons, records[8].dy), (SWING, 11_000));
    assert_eq!(records[11].buttons, SWING | HOLD_LOOK);
    assert_eq!(records[12].buttons & SWING, 0);
    // played back through the world's own rules
    let mut w = duel(true, 600);
    w.foes[0].brain.control = pilot::Control::Script(records);
    w.foes[0].brain.ticks = 0;
    let mut feinted = false;
    for _ in 0..20 {
        w.step(&Input::default());
        feinted |= w.foes[0].fighter.chiv.event.is_some_and(|e| e.0 == Event::Feint);
    }
    assert!(feinted);
    assert_eq!(w.foes[0].fighter.arm.swing.cut, None);
}

// ---- R3.10's default (M4): the sword forearm near the eye ----

/// The shapes drawn within `FOREARM_R` of the player's sword forearm, with
/// the frame drawn from the eye or not.
fn along_forearm(w: &World, eye_view: bool) -> usize {
    let mut w = w.clone();
    w.eye_view = eye_view;
    let mut out = crate::render::Shapes::default();
    player::draw(&w, ONE, &crate::look::tests::look(), &mut out);
    let (e, h) = (w.me.pose.elbow[1], w.me.pose.hand[1]);
    out.list
        .iter()
        .filter(|s| crate::arm::nearest_on(e, h, s.at.t).dist(s.at.t) < player::FOREARM_R)
        .count()
}

#[test]
fn the_sword_forearm_is_not_drawn_across_the_eye_in_a_chop_s_wind_up() {
    let mut w = duel(true, 600);
    // at the guard the forearm is far from the eye and drawn whole
    let eye = |w: &World| player::eye(w, ONE, w.aim);
    let far = crate::arm::nearest_on(w.me.pose.elbow[1], w.me.pose.hand[1], eye(&w)).dist(eye(&w));
    assert!(far >= player::FOREARM_NEAR, "{far}");
    assert_eq!(along_forearm(&w, true), along_forearm(&w, false));
    // a chop winds up: some tick lifts the forearm within 32 cm of the eye,
    // and on each such tick fewer shapes lie along it from the eye
    let mut near_ticks = 0;
    for t in 0..40 {
        w.step(&cut(0, 6_000, 0, t));
        let Some(c) = w.me.arm.swing.cut else { continue };
        if c.phase != Phase::WindUp {
            break;
        }
        let d = crate::arm::nearest_on(w.me.pose.elbow[1], w.me.pose.hand[1], eye(&w)).dist(eye(&w));
        if d < player::FOREARM_NEAR {
            near_ticks += 1;
            let (from_eye, behind) = (along_forearm(&w, true), along_forearm(&w, false));
            assert!(from_eye < behind, "tick {t}: {from_eye} of {behind}");
        }
    }
    assert!(
        near_ticks > 0,
        "the wind-up never lifted the forearm near the eye"
    );
}

// ---- the page: stamina in shape and words, the controls ----

#[test]
fn practice_turns_the_layer_on_and_each_bar_shows_stamina_in_words_and_winded_by_name() {
    let mut g = crate::game::tests::game();
    crate::game::tests::practice(&mut g);
    assert!(g.world.settings.chivalry, "Practice plays with the layer on");
    g.world.act("chivalry.my_stamina", "62").unwrap();
    g.world.act("chivalry.foe_stamina", "0").unwrap();
    let bars = crate::api::bars(&g);
    let mine = bars[0].stamina.as_ref().unwrap();
    assert_eq!((mine.label.as_str(), mine.text.as_str()), ("Stamina", "62 / 100"));
    assert_eq!((mine.share, mine.winded), (6_200, false));
    let theirs = bars[1].stamina.as_ref().unwrap();
    assert_eq!(theirs.text, "0 / 100 · Winded");
    assert_eq!((theirs.share, theirs.winded), (0, true));
    // off, no meter
    g.world.settings.chivalry = false;
    assert!(crate::api::bars(&g)[0].stamina.is_none());
}

#[test]
fn the_controls_name_the_feint_and_the_kick_and_f_is_the_kick() {
    let mut g = crate::game::tests::game();
    g.act("screen.show", "menu").unwrap();
    let ui = serde_json::to_value(crate::api::ui(&g)).unwrap();
    let rows: Vec<(String, String)> = ui["panels"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|p| p["rows"].as_array().cloned().unwrap_or_default())
        .map(|r| {
            (
                r["key"].as_str().unwrap().into(),
                r["does"].as_str().unwrap().into(),
            )
        })
        .collect();
    assert!(rows.contains(&("F".into(), "Kick".into())), "{rows:?}");
    assert!(rows.contains(&("Right button, as a cut draws back".into(), "Feint".into())));
    let controls: serde_json::Value =
        serde_json::from_str(include_str!("../../../../data/controls.json")).unwrap();
    assert_eq!(controls["KeyF"], "kick");
    assert!(input::ACTIONS.contains(&("kick", input::KICK)));
}

#[test]
fn the_hud_names_the_player_s_last_move_for_three_quarters_of_a_second() {
    let mut g = crate::game::tests::game();
    crate::game::tests::practice(&mut g);
    g.world.me.chiv.event = Some((Event::FollowUp(2), 0));
    let ui = crate::api::ui(&g);
    let item = ui.hud.iter().find(|h| h.id == "move").unwrap();
    assert_eq!((item.text.as_str(), item.shown), ("Follow-up 2", true));
    // 45 ticks shown (aged 0 to 45), gone on the 46th
    let mut w = g.world.clone();
    w.rounds = None;
    for _ in 0..45 {
        step(&mut w, &Input::default(), &[Input::default()]);
    }
    assert_eq!(w.me.chiv.event, Some((Event::FollowUp(2), 45)));
    step(&mut w, &Input::default(), &[Input::default()]);
    assert_eq!(w.me.chiv.event, None);
}

#[test]
fn a_kick_raises_the_leg_ahead_out_in_front_at_its_hit_tick() {
    let mut w = duel(true, 600);
    let b = w.player;
    let flat = player::pose_of(&w, 0).foot[0];
    w.me.chiv.kick = 8;
    let foot = player::pose_of(&w, 0).foot[0];
    // 55 cm up and 90 cm ahead of the feet, at the hit tick
    assert!((foot.y - b.pos.y - ONE * 55 / 100).abs() <= 1, "{foot:?}");
    let ahead = foot.sub(b.pos).dot(facing(b.facing, 0));
    assert!((ahead - ONE * 90 / 100).abs() <= 2, "{ahead}");
    // none of it before the kick or after
    w.me.chiv.kick = 0;
    assert_eq!(player::pose_of(&w, 0).foot[0], flat);
}

#[test]
fn kicked_the_player_s_own_arms_are_not_drawn_across_the_eye() {
    // the knight kicks the player; through the stagger's lean the arms come
    // up past the eye (R6.17): none of the player's own shapes is drawn
    // within 25 cm ahead of the eye, tick by tick
    let mut g = crate::game::tests::game();
    g.act("practice.stage", "duel.pilot=still").unwrap();
    g.act("practice.stage", "duel.gap=160").unwrap();
    g.act("practice.stage", "chivalry.foe_move=kick").unwrap();
    crate::game::tests::practice(&mut g);
    let w = &mut g.world;
    let mut kicked = false;
    for t in 0..30 {
        w.step(&Input::default());
        // T10: the tick the kick lands, the eye is thrown 5° (910 units)
        // on top of whatever lean it had
        if !kicked && w.me.gait.stagger > 0 {
            assert!(
                (w.feel.tilt - w.feel.prev_tilt).abs() >= 900,
                "thrown by the kick: {} from {}",
                w.feel.tilt,
                w.feel.prev_tilt
            );
        }
        kicked |= w.me.gait.stagger > 0;
        let e = player::eye(w, ONE, w.aim);
        let mut v = w.clone();
        v.foes.clear();
        let mut out = crate::render::Shapes::default();
        player::draw(&v, ONE, &crate::look::tests::look(), &mut out);
        let ahead = facing(w.aim.yaw, w.aim.pitch);
        let near = out
            .list
            .iter()
            .filter(|s| s.at.t.dist(e) < ONE / 4 && s.at.t.sub(e).dot(ahead) > 0)
            .count();
        assert_eq!(near, 0, "tick {t}");
    }
    assert!(kicked, "the kick landed");
}

#[test]
fn a_staged_move_starts_every_foe_on_it_and_a_bad_one_is_refused() {
    let mut w = duel(true, 600);
    w.act("chivalry.foe_move", "kick").unwrap();
    assert_eq!(w.foes[0].brain.current, Some((Move::Kick, 0)));
    assert_eq!(w.foes[0].brain.control, pilot::Control::Tree);
    assert_eq!(
        w.act("chivalry.foe_move", "dance"),
        Err("A staged move is guard_up, kick or feint_cut, not \"dance\".".into())
    );
}

// ---- T7: the right button's guard, for the computer's knight too ----

#[test]
fn the_knight_raises_the_guard_through_its_record_when_a_cut_comes() {
    // the player's tip coming at the knight at 6 m/s: some seeds raise the
    // guard (chance 25), the rest block
    let coming = decisions(200, |w| {
        let to = w.foes[0].body.pos.sub(w.player.pos).norm().scale(ONE / 10);
        let tip = &mut w.me.arm.pts[crate::arm::TIP];
        tip.q = tip.p.sub(to);
    });
    assert!(coming.contains(&Some(Move::GuardUp)), "{coming:?}");
    // the move goes out as the right button held, and the knight's hand
    // takes the guard in front through the world's own rules
    let mut w = duel(true, 600);
    w.act("chivalry.foe_move", "guard_up").unwrap();
    w.tick = 0;
    let (i, _) = pilot::decide(&w, 1);
    assert_eq!(i.buttons & HOLD_LOOK, HOLD_LOOK);
    for _ in 0..3 {
        w.step(&Input::default());
    }
    let f = &w.foes[0].fighter;
    assert!(f.chiv.guard.is_some());
    assert_eq!(f.arm.shell.pitch, deg(5) + f.chiv.guard.unwrap().shift.1);
}

#[test]
fn the_right_button_guards_only_in_first_person() {
    // the port's path: the right button looks, as CHUMWORLD's did
    let mut w = World::new(1);
    w.step(&press(DRAW));
    for _ in 0..player::DRAW_TICKS + 10 {
        w.step(&Input::default());
    }
    w.step(&Input {
        buttons: HOLD_LOOK,
        ..Input::default()
    });
    assert_eq!(w.me.chiv.guard, None);
    assert!(w.me.mouse_looks);
}

// ---- M5's guest seat: every M4 rule, and T7 and T8, from its own record ----

#[test]
fn the_guest_seat_s_knight_pays_feints_guards_kicks_and_holds_its_view_from_its_own_record() {
    use crate::online::Sim;
    let mut w = crate::seat::duel();
    assert!(w.settings.chivalry);
    let idle = [Input::default(), Input::default()];
    while w.rounds.as_ref().unwrap().phase != crate::rounds::Phase::Fight {
        w.tick(idle);
    }
    for _ in 0..30 {
        w.tick(idle);
    }
    let seat = |w: &World| w.seat.as_ref().unwrap().aim;
    let f = |w: &World| w.foes[0].fighter.clone();
    assert_eq!(f(&w).chiv.stamina, 10_000);
    // a chop's gesture (12), the view held through a hard swipe (T8)
    let aim = seat(&w);
    w.tick([Input::default(), cut(0, 6_000, 0, 0)]);
    assert!(f(&w).arm.swing.cut.is_some());
    assert_eq!(f(&w).chiv.stamina, 8_800);
    w.tick([
        Input::default(),
        Input {
            buttons: SWING,
            dx: 30_000,
            dy: 9_000,
            ..Input::default()
        },
    ]);
    assert_eq!(
        seat(&w),
        aim,
        "the seat's view holds while its left button is held"
    );
    // the right button in the wind-up: a feint (15), then the guard (T7)
    w.tick([
        Input::default(),
        Input {
            buttons: SWING | HOLD_LOOK,
            ..Input::default()
        },
    ]);
    assert_eq!(f(&w).arm.swing.cut, None);
    assert_eq!(f(&w).chiv.stamina, 7_300);
    w.tick([
        Input::default(),
        Input {
            buttons: HOLD_LOOK,
            dx: 5_000,
            ..Input::default()
        },
    ]);
    assert!(f(&w).chiv.guard.is_some());
    assert_eq!(f(&w).arm.shell.side, deg(-18) + 1365);
    assert_eq!(seat(&w), aim, "and while its right button is held");
    // let go, F: a kick (20)
    w.tick(idle);
    w.tick([Input::default(), press(KICK)]);
    assert_eq!(f(&w).chiv.kick, 1);
    assert_eq!(f(&w).chiv.stamina, 5_300);
}
