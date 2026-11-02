//! The plate rules worked by hand (M3, CA-Q8: A).
//!
//! Every blow here lands at 17.5 m/s, 16 m/s over CHUMWORLD's 1.5 m/s
//! floor, so its damage is whole: 16 × 2.5 = 40 on the torso, × 1.5 = 60 on
//! the head, × 0.6 = 24 on an arm, × 0.7 = 28 on a leg. Through plate a cut
//! keeps 30% (12 on the torso, 7.2 → 7 on an arm, 18 on the head, 8.4 → 8
//! on a leg) and a thrust 80% (32 on the torso).

use super::*;
use crate::blade::Part;
use crate::fx::int;
use crate::geom::v3;
use crate::wounds::bit;

/// 17.5 m/s.
const FAST: Fx = ONE * 35 / 2;

/// The two knights in the ring, the computer's standing still: the player
/// (0) Knight A, the foe (1) Knight B.
fn duel() -> World {
    let mut w = World::new(3);
    w.act("duel.me", "knight_a").unwrap();
    w.act("duel.spawn", "knight_b").unwrap();
    w.act("duel.pilot", "still").unwrap();
    w
}

fn part_of(seg: usize) -> Part {
    match seg {
        HEAD => Part::Head,
        TORSO => Part::Torso,
        2..=5 => Part::Arm(if seg <= 3 { 0 } else { 1 }),
        _ => Part::Leg(if seg <= 7 { 0 } else { 1 }),
    }
}

/// The player's blade's line, pommel to tip.
fn axis(w: &World) -> V3 {
    let a = &w.me.arm;
    a.pts[arm::TIP].p.sub(a.pts[arm::POMMEL].p).norm()
}

/// A blow of the player's on the foe's part `seg`, `pct` of the way down
/// its spine (or `up` above the head's middle for the head), the blade
/// coming in from the way `dir`: a cut travels into the body square to the
/// blade's line, a thrust along it.
fn blow(w: &World, seg: usize, pct: i32, dir: V3, thrust: bool) -> Hit {
    let caps = crate::blade::body(Kind::KnightB, &player::pose_of(w, 1));
    let c = &caps[seg];
    let at = c.a.lerp(c.b, pct * ONE / 100);
    let ax = axis(w);
    let into = dir.neg();
    let vel = if thrust {
        ax.scale(FAST / 60)
    } else {
        into.sub(ax.scale(into.dot(ax))).norm().scale(FAST / 60)
    };
    Hit {
        by: 0,
        on: 1,
        part: part_of(seg),
        seg: seg as u8,
        vel,
        speed: FAST,
        at,
        from: at.add(dir.scale(ONE * 3 / 10)),
        through_pct: 100,
        plated: false,
    }
}

fn foe(w: &World) -> &crate::player::Fighter {
    &w.foes[0].fighter
}

fn ways(w: &World) -> (V3, V3, V3) {
    let p = player::pose_of(w, 1);
    let fwd = facing(p.facing, 0);
    (fwd, fwd.neg(), crate::geom::right_of(p.facing))
}

#[test]
fn a_knight_comes_in_whole_plate_and_anyone_else_in_none() {
    let w = duel();
    assert_eq!(w.me.health.limbs.plate, [66, 45, 27, 27, 27, 27, 30, 30, 30, 30]);
    assert_eq!(foe(&w).health.limbs.plate, INTEGRITY);
    assert_eq!(full(Kind::Bandit), [0; PARTS]);
    assert_eq!(full(Kind::Player), [0; PARTS]);
    assert_eq!(bare(&full(Kind::KnightB)), 0);
    assert_eq!(bare(&[0; PARTS]), 0b11_1111_1111);
}

/// The blades' own blows go the same way: a hit the blades found meets
/// plate in `health::step` before health is taken.
#[test]
fn a_blow_the_blades_find_meets_plate_before_health_is_taken() {
    let mut w = duel();
    let (_, _, right) = ways(&w);
    let h = blow(&w, TORSO, 50, right, false);
    w.blades.hits.push(h);
    crate::health::step(&mut w);
    assert_eq!(foe(&w).health.hp, 100 - 12);
    assert_eq!(foe(&w).health.limbs.plate[TORSO], 45 - 28);
    assert!(w.blades.hits[0].plated);
    assert_eq!(w.blades.hits[0].through_pct, 30);
}

#[test]
fn a_cut_on_plate_does_thirty_percent_and_wears_it() {
    let mut w = duel();
    let (_, _, right) = ways(&w);
    let h = blow(&w, TORSO, 50, right, false);
    wounds::stage(&mut w, h);
    // 40 × 30% = 12 through; the plate takes 28 of its 45
    assert_eq!(foe(&w).health.hp, 100 - 12);
    assert_eq!(foe(&w).health.limbs.dmg[TORSO], 12);
    assert_eq!(foe(&w).health.limbs.plate[TORSO], 45 - 28);
}

#[test]
fn a_thrust_on_plate_does_eighty_percent() {
    let mut w = duel();
    let (fwd, _, _) = ways(&w);
    let h = blow(&w, TORSO, 50, fwd, true);
    wounds::stage(&mut w, h);
    // 40 × 80% = 32 through; the plate takes 8
    assert_eq!(foe(&w).health.hp, 100 - 32);
    assert_eq!(foe(&w).health.limbs.plate[TORSO], 45 - 8);
    // at 10 m/s the share is taken before rounding: 8.5 × 2.5 × 80% = 17.0
    // through, where 80% of the rounded 21 would be 16.8 → 16
    let mut w = duel();
    let mut h = blow(&w, TORSO, 50, fwd, true);
    h.speed = int(10);
    wounds::stage(&mut w, h);
    assert_eq!(foe(&w).health.hp, 100 - 17);
    assert_eq!(foe(&w).health.limbs.plate[TORSO], 45 - (21 - 17));
}

#[test]
fn a_cut_into_the_armpit_does_all_its_damage_and_the_same_cut_outside_does_not() {
    let mut w = duel();
    let p = player::pose_of(&w, 1);
    let caps = crate::blade::body(Kind::KnightB, &p);
    let s = UPPER_ARM[RIGHT];
    let u = caps[s].b.sub(caps[s].a).norm();
    let inward = square_to(p.chest.sub(p.shoulder[RIGHT]), u);
    let h = blow(&w, s, 10, inward, false);
    wounds::stage(&mut w, h);
    // 24 whole through the gap, and the plate untouched
    assert_eq!(foe(&w).health.hp, 100 - 24);
    assert_eq!(foe(&w).health.limbs.plate[s], 27);
    // the same place from outside the arm: 7 through, 17 taken
    let h = blow(&w, s, 10, inward.neg(), false);
    wounds::stage(&mut w, h);
    assert_eq!(foe(&w).health.hp, 100 - 24 - 7);
    assert_eq!(foe(&w).health.limbs.plate[s], 27 - 17);
    // halfway down the arm, from the inside: plate (outside the first quarter)
    let h = blow(&w, s, 50, inward, false);
    assert_eq!(gap(Kind::KnightB, &p, s, h.at, h.from), None);
}

#[test]
fn the_visor_s_slit_and_the_back_of_the_knee_are_gaps_from_their_own_side_only() {
    let mut w = duel();
    let (fwd, back, _) = ways(&w);
    let p = player::pose_of(&w, 1);
    // the slit, 20 mm over the head's middle (8 to 32 on the player's 1.70
    // m, scaled to 1.85 m: 8.7 to 34.8 mm), from in front: 60 whole
    let caps = crate::blade::body(Kind::KnightB, &p);
    let head_up = caps[HEAD].b.sub(caps[HEAD].a);
    let share = scaled(Kind::KnightB, 20) as i64 * 100 / head_up.len() as i64;
    let h = blow(&w, HEAD, share as i32, fwd, false);
    assert_eq!(gap(Kind::KnightB, &p, HEAD, h.at, h.from), Some(Gap::Visor));
    wounds::stage(&mut w, h);
    assert_eq!(foe(&w).health.hp, 100 - 60);
    // from behind at the same height: plate, 18 through
    let h = blow(&w, HEAD, share as i32, back, false);
    wounds::stage(&mut w, h);
    assert_eq!(foe(&w).health.hp, 100 - 60 - 18);
    // the back of the right knee, 90% down the thigh: 28 whole
    let mut w = duel();
    let s = THIGH[RIGHT];
    let h = blow(&w, s, 90, back, false);
    assert_eq!(gap(Kind::KnightB, &p, s, h.at, h.from), Some(Gap::Knee));
    wounds::stage(&mut w, h);
    assert_eq!(foe(&w).health.hp, 100 - 28);
    // its front, the knee cop: 8 through
    let h = blow(&w, s, 90, fwd, false);
    wounds::stage(&mut w, h);
    assert_eq!(foe(&w).health.hp, 100 - 28 - 8);
    // the shin's top from behind is the same gap
    let h = blow(&w, SHIN[RIGHT], 10, back, false);
    assert_eq!(gap(Kind::KnightB, &p, SHIN[RIGHT], h.at, h.from), Some(Gap::Knee));
}

#[test]
fn plate_breaks_off_once_it_has_taken_its_integrity_and_then_shows_mail() {
    let mut w = duel();
    let (_, _, right) = ways(&w);
    let h = blow(&w, TORSO, 50, right, false);
    wounds::stage(&mut w, h);
    assert_eq!(foe(&w).health.limbs.plate[TORSO], 17, "45 − 28: still on");
    assert!(w.wounds.pieces.is_empty());
    wounds::stage(&mut w, h);
    // 17 − 28: broken off, falling as a piece of plate, no stump
    assert_eq!(foe(&w).health.limbs.plate[TORSO], 0);
    assert_eq!(bare(&foe(&w).health.limbs.plate), bit(TORSO));
    assert_eq!(w.wounds.pieces.len(), 1);
    assert!(w.wounds.pieces[0].plate);
    assert!(!w.wounds.pieces[0].parts.is_empty());
    assert_eq!(foe(&w).health.limbs.lost, 0);
    assert_eq!(foe(&w).health.hp, 100 - 12 - 12);
    // bare: the whole 40
    wounds::stage(&mut w, h);
    assert_eq!(foe(&w).health.hp, 100 - 12 - 12 - 40);
    // the knight is drawn with mail on its torso and its cuirass gone
    let k = crate::models::knight_model(
        Kind::KnightB,
        &player::pose_of(&w, 1),
        bare(&foe(&w).health.limbs.plate),
    );
    let sc = crate::models::knight_scheme(Kind::KnightB);
    assert!(k
        .parts
        .iter()
        .any(|t| t.seg as usize == TORSO && t.part.rgb == sc.mail));
    assert!(!k
        .parts
        .iter()
        .any(|t| t.seg as usize == TORSO && t.layer == crate::models::Layer::Plate));
}

#[test]
fn a_plated_limb_is_never_severed_whatever_the_speed() {
    let mut w = duel();
    let (_, _, right) = ways(&w);
    let s = UPPER_ARM[RIGHT];
    let mut h = blow(&w, s, 60, right, false);
    // 40 m/s: 38.5 × 2.5 × 0.6 = 57.75 → 57, past CHUMWORLD's one-cut 12.5
    // m/s; and its damage already past the arm's 30
    h.speed = int(40);
    w.foes[0].fighter.health.limbs.dmg[s] = 1_000;
    wounds::stage(&mut w, h);
    assert_eq!(foe(&w).health.limbs.lost, 0, "plated: kept");
    // it took 57 − 17 = 40, past its 27: the plate is off, the arm on
    assert_eq!(foe(&w).health.limbs.plate[s], 0);
    // bare now, the next such blow takes it off
    wounds::stage(&mut w, h);
    assert_eq!(foe(&w).health.limbs.lost & bit(s), bit(s));
}

#[test]
fn a_bare_limb_severs_at_chumworld_s_thresholds() {
    let mut w = duel();
    let (_, _, right) = ways(&w);
    let s = UPPER_ARM[RIGHT];
    w.foes[0].fighter.health.limbs.plate[s] = 0;
    let mut h = blow(&w, s, 60, right, false);
    // 12 m/s: 10.5 × 2.5 × 0.6 = 15.75 → 15, under the arm's 30 and under
    // the one-cut 12.5 m/s
    h.speed = int(12);
    wounds::stage(&mut w, h);
    assert_eq!(foe(&w).health.limbs.lost, 0);
    assert_eq!(foe(&w).health.limbs.dmg[s], 15);
    // 30: off
    wounds::stage(&mut w, h);
    assert_eq!(foe(&w).health.limbs.lost & bit(s), bit(s));
    // a bare forearm: one blow at 12.5 m/s takes it off at once
    let mut w = duel();
    let s = FOREARM[RIGHT];
    w.foes[0].fighter.health.limbs.plate[s] = 0;
    let mut h = blow(&w, s, 50, right, false);
    h.speed = ONE * 25 / 2;
    wounds::stage(&mut w, h);
    assert_eq!(foe(&w).health.limbs.lost & bit(s), bit(s));
}

#[test]
fn a_kind_without_plate_takes_chumworld_s_blow_unchanged() {
    let mut w = World::new(3);
    w.act("duel.spawn", "bandit").unwrap();
    w.act("duel.pilot", "still").unwrap();
    let caps = crate::blade::body(Kind::Bandit, &player::pose_of(&w, 1));
    let at = caps[TORSO].a.lerp(caps[TORSO].b, ONE / 2);
    let h = Hit {
        by: 0,
        on: 1,
        part: Part::Torso,
        seg: TORSO as u8,
        vel: v3(ONE / 4, 0, 0),
        speed: FAST,
        at,
        from: at.sub(v3(ONE / 4, 0, 0)),
        through_pct: 100,
        plated: false,
    };
    wounds::stage(&mut w, h);
    // the bandit's 90 less the whole 40
    assert_eq!(foe(&w).health.hp, 50);
    assert_eq!(foe(&w).health.limbs.plate, [0; PARTS]);
    // a 12.5 m/s cut takes its bare arm off, as in CHUMWORLD
    let s = UPPER_ARM[RIGHT];
    let at = caps[s].a.lerp(caps[s].b, ONE / 2);
    let h = Hit {
        part: Part::Arm(1),
        seg: s as u8,
        speed: ONE * 25 / 2,
        at,
        from: at.sub(v3(ONE / 4, 0, 0)),
        ..h
    };
    wounds::stage(&mut w, h);
    assert_eq!(foe(&w).health.limbs.lost & bit(s), bit(s));
}

#[test]
fn a_blow_is_a_thrust_within_forty_five_degrees_of_the_blade_s_line() {
    let ax = v3(ONE, 0, 0);
    assert!(is_thrust(v3(ONE, 0, 0), ax));
    // 40° off: cos 40° = 0.766 ≥ 0.707
    assert!(is_thrust(v3(ONE * 766 / 1000, ONE * 643 / 1000, 0), ax));
    // 50° off: cos 50° = 0.643
    assert!(!is_thrust(v3(ONE * 643 / 1000, ONE * 766 / 1000, 0), ax));
    assert!(!is_thrust(v3(-ONE, 0, 0), ax), "pommel first");
    assert!(!is_thrust(V3::ZERO, ax));
    assert_eq!(through_pct(true, false, false), 30);
    assert_eq!(through_pct(true, false, true), 80);
    assert_eq!(through_pct(true, true, false), 100);
    assert_eq!(through_pct(false, false, true), 100);
}

#[test]
fn plate_is_in_the_world_s_hash() {
    let a = duel();
    let mut b = duel();
    assert_eq!(a.hash(), b.hash());
    b.foes[0].fighter.health.limbs.plate[TORSO] -= 1;
    assert_ne!(a.hash(), b.hash());
}

/// In a duel the blades' own blows meet plate (`health::step` lands each
/// on it before health is taken): the computer's knight, on its tree,
/// against the player's draw-and-cut script, wears a plate somewhere: a
/// blow on plate goes through at a cut's or a thrust's share.
#[test]
fn the_blades_own_blows_meet_plate_in_a_duel() {
    use crate::input::{self, Input};
    let mut w = World::new(5);
    w.act("duel.me", "knight_a").unwrap();
    w.act("duel.spawn", "knight_b").unwrap();
    let mut worn = 0;
    for t in 0..900u32 {
        let mut i = Input::default();
        if t == 0 {
            i.keys = input::DRAW;
        }
        if (30..70).contains(&t) {
            i.keys |= input::FORWARD;
        }
        if t >= 90 && t % 50 == 0 {
            i.dx = if (t / 50) % 2 == 0 { -6000 } else { 6000 };
        }
        if t >= 90 && t % 50 < 35 {
            i.buttons |= input::SWING;
        }
        w.step(&i);
        for h in &w.blades.hits {
            assert!([30, 80, 100].contains(&h.through_pct), "{}", h.through_pct);
            if h.plated && h.through_pct < 100 {
                worn += 1;
            }
        }
    }
    let plates = [w.me.health.limbs.plate, w.foes[0].fighter.health.limbs.plate];
    assert!(worn > 0, "a blow met plate");
    assert!(
        plates
            .iter()
            .any(|p| p.iter().zip(INTEGRITY).any(|(a, b)| *a < b)),
        "{plates:?}"
    );
}
