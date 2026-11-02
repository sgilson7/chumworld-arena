//! The pit veteran's costume tests (CA-7, M6): his silhouette in every
//! pose, no plate face in another colour's plane, the triangle cap, where
//! he wears plate and what that does to a blow, and his colours.

use super::super::ladder_poses::{axes, extent, fights, nearest_highlight, poses, TRIANGLE_CAP};
use super::super::model as knight_model;
use super::*;
use crate::blade::{Hit, Part as BodyPart};
use crate::fx::int;
use crate::world::World;

const KIND: Kind = Kind::Veteran;

/// The veteran reads at duel distance by one huge pauldron on the left
/// and none on the right: in every pose the pauldron is at least half as
/// wide again as his helm, it reaches 12 cm or more out past his left
/// shoulder's joint, and every piece of it sits on the left upper arm, left
/// of the chest.
#[test]
fn the_veteran_keeps_one_huge_left_pauldron_in_every_pose() {
    let all = poses(KIND);
    assert!(all.len() > 90, "{} poses", all.len());
    for (n, p) in all.iter().enumerate() {
        let k = knight_model(KIND, p, 0);
        let (right, _) = axes(p);
        let (plo, phi) = extent(&k, Role::Pauldron, right);
        let (hlo, hhi) = extent(&k, Role::Helm, right);
        assert!(
            (phi - plo) * 2 >= (hhi - hlo) * 3,
            "pose {n}: pauldron {}, helm {}",
            phi - plo,
            hhi - hlo
        );
        let shoulder = p.shoulder[LEFT].dot(right);
        assert!(
            shoulder - plo >= ONE * 12 / 100,
            "pose {n}: {} past the shoulder",
            shoulder - plo
        );
        let chest = p.chest.dot(right);
        for t in k.parts.iter().filter(|t| t.role == Role::Pauldron) {
            assert_eq!(t.seg as usize, UPPER_ARM[LEFT], "pose {n}");
            assert!(
                t.part.at.t.dot(right) < chest,
                "pose {n}: a pauldron right of the chest"
            );
        }
    }
}

/// His face is a T: the eyes' bar at least three times as wide as the bar
/// down from it, which hangs below it, in front of the head.
#[test]
fn the_veteran_s_helm_has_a_t_shaped_face() {
    let p = rest_pose(KIND, V3::ZERO, 0);
    let k = knight_model(KIND, &p, 0);
    let (right, up) = axes(&p);
    let slit = k
        .parts
        .iter()
        .find(|t| t.role == Role::Slit)
        .expect("the eyes' bar");
    let stem: Vec<&Tagged> = k
        .parts
        .iter()
        .filter(|t| t.seg as usize == HEAD && t.role == Role::Trim && t.part.rgb == scheme(KIND).frame)
        .collect();
    let (sl, sh) = super::super::ladder_poses::reach(&slit.part, right);
    let below: Vec<&&Tagged> = stem
        .iter()
        .filter(|t| t.part.at.t.dot(up) < slit.part.at.t.dot(up))
        .collect();
    assert!(!below.is_empty(), "no bar below the eyes");
    for t in below {
        let (a, b) = super::super::ladder_poses::reach(&t.part, right);
        assert!((sh - sl) >= (b - a) * 3, "eyes {} stem {}", sh - sl, b - a);
        assert!(t.part.at.t.sub(p.head).dot(facing(p.facing, 0)) > ONE / 10);
    }
}

/// His upper arms are bare: the right wears nothing over the skin, and the
/// left only the pauldron; neither is drawn in the frame's dark.
#[test]
fn the_veteran_s_upper_arms_are_bare_skin() {
    let skin = table().slot("veteran", "skin");
    for p in poses(KIND).iter().step_by(10) {
        let k = knight_model(KIND, p, 0);
        for t in k.parts.iter().filter(|t| t.seg as usize == UPPER_ARM[RIGHT]) {
            assert_eq!(t.layer, Layer::Frame);
            assert_eq!(t.part.rgb, skin);
        }
        for t in k.parts.iter().filter(|t| t.seg as usize == UPPER_ARM[LEFT]) {
            assert!(t.role == Role::Pauldron || t.part.rgb == skin, "{:?}", t.role);
        }
    }
}

#[test]
fn no_face_of_the_veteran_shares_a_plane_with_another_colour() {
    let (bad, scanned) = fights(KIND);
    assert!(scanned > 25, "{scanned}");
    assert!(bad.is_empty(), "{} fights:\n{}", bad.len(), bad.join("\n"));
}

#[test]
fn the_veteran_is_under_the_triangle_cap() {
    let p = rest_pose(KIND, V3::ZERO, 0);
    let n = knight_model(KIND, &p, 0).model().triangles();
    assert!(n <= TRIANGLE_CAP, "{n} triangles");
    assert!(n > 1_000, "{n} triangles is no costume");
}

/// The eyes' bar is where `plate::gap` finds the visor.
#[test]
fn the_veteran_s_eyes_are_where_the_visor_gap_is() {
    let p = rest_pose(KIND, V3::ZERO, 0);
    let k = knight_model(KIND, &p, 0);
    let (_, up) = axes(&p);
    let slit = k.parts.iter().find(|t| t.role == Role::Slit).unwrap();
    let y = slit.part.at.t.sub(p.head).dot(up);
    let (lo, hi) = (
        crate::plate::scaled(KIND, VISOR_MM.0),
        crate::plate::scaled(KIND, VISOR_MM.1),
    );
    assert!(lo <= y && y <= hi, "eyes at {y}, gap {lo}..{hi}");
}

/// Every slot of his scheme, his own two included, stands 100 or more
/// from the duel's highlights (R5.8; the sash darkened, R7.205).
#[test]
fn the_veteran_s_colours_stand_clear_of_the_highlights() {
    let (d2, what) = nearest_highlight("veteran");
    assert!(d2 >= 100 * 100, "sqrt({d2}): {what}");
}

/// Where he wears plate, worked by hand from `plate::INTEGRITY` (head 66,
/// each arm part 27): the helm, the left upper arm and both forearms; the
/// torso, the right upper arm and the legs none (R7.202).
#[test]
fn the_veteran_wears_plate_on_his_helm_left_arm_and_forearms_only() {
    assert_eq!(crate::plate::full(KIND), [66, 0, 27, 27, 0, 27, 0, 0, 0, 0]);
    assert_eq!(
        crate::plate::full(Kind::Banneret),
        [66, 45, 27, 27, 27, 27, 30, 30, 30, 30]
    );
}

/// The veteran as the still foe (1), the player Knight A.
fn duel() -> World {
    let mut w = World::new(3);
    w.act("duel.me", "knight_a").unwrap();
    w.act("duel.spawn", "veteran").unwrap();
    w.act("duel.pilot", "still").unwrap();
    assert_eq!(w.foes[0].fighter.kind, KIND);
    w
}

/// A 40 m/s cut on the foe's part `seg`, halfway down it, from its right.
fn cut(w: &World, seg: usize) -> Hit {
    let p = crate::player::pose_of(w, 1);
    let caps = crate::blade::body(KIND, &p);
    let c = &caps[seg];
    let at = c.a.lerp(c.b, ONE / 2);
    let dir = crate::geom::right_of(p.facing);
    let part = match seg {
        HEAD => BodyPart::Head,
        TORSO => BodyPart::Torso,
        2..=5 => BodyPart::Arm(if seg <= 3 { 0 } else { 1 }),
        _ => BodyPart::Leg(if seg <= 7 { 0 } else { 1 }),
    };
    Hit {
        by: 0,
        on: 1,
        part,
        seg: seg as u8,
        // square to the blade's line, which points up the player's guard:
        // a cut, never a thrust
        vel: dir.neg().scale(int(40) / 60),
        speed: int(40),
        at,
        from: at.add(dir.scale(ONE * 3 / 10)),
        through_pct: 100,
        plated: false,
    }
}

/// His bare right upper arm comes off at one 40 m/s cut (CHUMWORLD's
/// one-cut speed for an arm is 12.5 m/s), where the same cut on the left,
/// under the pauldron, breaks the plate (40 m/s on an arm is 57, of which
/// 17 goes through and 40 is taken, past its 27) and leaves the arm on. A
/// cut on his leather torso goes through whole.
#[test]
fn the_veteran_s_bare_arm_is_cut_off_and_his_pauldroned_arm_is_not() {
    let mut w = duel();
    let s = UPPER_ARM[RIGHT];
    let h = cut(&w, s);
    crate::wounds::stage(&mut w, h);
    assert_eq!(w.foes[0].fighter.health.limbs.lost & bit(s), bit(s), "bare: off");

    let mut w = duel();
    let s = UPPER_ARM[LEFT];
    let h = cut(&w, s);
    crate::wounds::stage(&mut w, h);
    assert_eq!(w.foes[0].fighter.health.limbs.lost, 0, "under the pauldron: kept");
    assert_eq!(
        w.foes[0].fighter.health.limbs.plate[s], 0,
        "the pauldron broken off"
    );

    let mut w = duel();
    let mut h = cut(&w, TORSO);
    crate::plate::land(&mut w, &mut h);
    assert!(!h.plated);
    assert_eq!(h.through_pct, 100);
}
