//! The models' tests: that a figure sits on its pose's joints and a held
//! item on its grip and tip, that every model draws in the file's colours,
//! that no two parts of any model share a plane and every mark stands off
//! its surface (CW-18). (CHUMWORLD's tests of its props, pickups, donkey
//! and lineup are left behind with them, R2.6.)

use super::figure::model as figure_model;
use super::held::model as held_model;
use super::*;
use crate::flicker::{self, Inst, Seen};
use crate::fx::int;
use crate::pose::{build, LEFT, RIGHT};

/// A unit mesh's two ends along its up axis, placed.
fn ends(p: &Part) -> (V3, V3) {
    let h = ONE / 2;
    (p.at.apply(v3(0, -h, 0)), p.at.apply(v3(0, h, 0)))
}

fn near(a: V3, b: V3, tol: Fx) -> bool {
    a.dist(b) <= tol
}

/// Every figure the lineup shows, and the player wearing every piece.
fn every_figure() -> Vec<(Kind, Dress, Model)> {
    let mut out = Vec::new();
    // every kind as CHUMWORLD's lineup cast it, each villager's look too
    let mut list: Vec<(Kind, Dress)> = KINDS
        .iter()
        .filter(|k| **k != Kind::Villager)
        .map(|k| (*k, Dress::default()))
        .collect();
    for v in 0..VILLAGER_VARIANTS {
        list.push((Kind::Villager, Dress { worn: 0, variant: v }));
    }
    list.push((
        Kind::Player,
        Dress {
            worn: SPEEDSTER_ALL,
            variant: 0,
        },
    ));
    for (kind, dress) in list {
        let pose = rest_pose(kind, v3(int(10), 0, int(20)), fx::deg(30));
        let mut m = figure_model(kind, &pose, dress);
        let held = held_model(kind_held(kind), pose.grip, pose.tip);
        m.parts.extend(held.parts);
        out.push((kind, dress, m));
    }
    out
}

fn every_other_model() -> Vec<(String, Model)> {
    let mut out: Vec<(String, Model)> = Vec::new();
    for item in [
        Held::ShortSword,
        Held::Sword,
        Held::Cleaver,
        Held::Boomerang,
        Held::Rock,
        Held::Knife,
    ] {
        out.push((
            format!("{item:?}"),
            held_model(
                item,
                v3(0, ONE, 0),
                v3(0, ONE, 0).add(v3(grip_to_tip(item), 0, 0)),
            ),
        ));
    }
    out
}

#[test]
fn every_model_draws_in_the_file_s_colours() {
    for (kind, dress, m) in every_figure() {
        for (i, p) in m.parts.iter().enumerate() {
            assert_ne!(
                p.rgb, MISSING,
                "{kind:?} {dress:?}: part {i} names a colour data/models.json lacks"
            );
        }
    }
    for (name, m) in every_other_model() {
        for (i, p) in m.parts.iter().enumerate() {
            assert_ne!(
                p.rgb, MISSING,
                "{name}: part {i} names a colour data/models.json lacks"
            );
        }
    }
    // the file's widths: 9 mm on a figure, 16 mm on a prop
    assert_eq!(table().ink_figure, 9 * ONE / 1000);
    assert_eq!(table().ink_prop, 16 * ONE / 1000);
}

/// The figure's limbs run exactly between the pose's joints: on the rest
/// pose, and on a pose with the right arm thrown up over the head, as a
/// swing would put it.
#[test]
fn limbs_run_between_the_pose_s_joints() {
    let mut pose = rest_pose(Kind::Player, V3::ZERO, 0);
    pose.elbow[RIGHT] = pose.shoulder[RIGHT].add(v3(0, mm(310), 0));
    pose.hand[RIGHT] = pose.elbow[RIGHT].add(v3(-mm(100), mm(270), -mm(40)));
    for p in [rest_pose(Kind::Player, V3::ZERO, 0), pose] {
        let m = figure_model(Kind::Player, &p, Dress::default());
        let has = |a: V3, b: V3| {
            m.parts.iter().filter(|q| q.mesh == Mesh::Cyl).any(|q| {
                let (x, y) = ends(q);
                near(x, a, 4) && near(y, b, 4)
            })
        };
        for s in [LEFT, RIGHT] {
            assert!(has(p.shoulder[s], p.elbow[s]), "upper arm {s}");
            assert!(has(p.elbow[s], p.hand[s]), "forearm {s}");
            assert!(has(p.hip[s], p.knee[s]), "thigh {s}");
            // the shin to the ankle, 7 cm over the foot
            assert!(has(p.knee[s], p.foot[s].add(v3(0, mm(70), 0))), "shin {s}");
        }
        // the hand balls sit on the hands
        for s in [LEFT, RIGHT] {
            assert!(
                m.parts
                    .iter()
                    .any(|q| q.mesh == Mesh::Facet && q.at.t == p.hand[s]),
                "hand {s}"
            );
        }
    }
}

/// The head sits at `pose.head` and faces `pose.facing`: turned a quarter
/// left (facing west, −x), the eyes are west of the head's middle.
#[test]
fn the_head_sits_at_the_pose_s_head_and_faces_its_way() {
    for (yaw, west) in [(fx::QUARTER_TURN, true), (-fx::QUARTER_TURN, false)] {
        let pose = rest_pose(Kind::Bandit, v3(int(5), 0, int(5)), yaw);
        let m = figure_model(Kind::Bandit, &pose, Dress::default());
        let head = m
            .parts
            .iter()
            .find(|p| p.mesh == Mesh::Facet && p.at.t == pose.head);
        assert!(head.is_some(), "a ball at the pose's head");
        // the eyes: the decals laid on the head
        let k = m
            .parts
            .iter()
            .position(|p| p.mesh == Mesh::Facet && p.at.t == pose.head)
            .unwrap();
        let eyes: Vec<&Part> = m
            .decals
            .iter()
            .filter(|(_, b)| *b == k)
            .map(|(d, _)| &m.parts[*d])
            .collect();
        assert!(eyes.len() >= 2);
        for e in eyes {
            // 18 cm out from the middle of the head, to the west or east
            let dx = e.at.t.x - pose.head.x;
            assert!(if west { dx < -int(1) / 10 } else { dx > int(1) / 10 }, "{dx}");
        }
    }
}

/// `rest_pose` keeps the build's lengths, so M1's arms and the drawn arms
/// agree: upper arm 31 cm, forearm 29 cm, thigh 37 cm, shin 35 cm on the
/// player, and the head's middle 1.70 − 0.03 − 0.21 = 1.46 m up.
#[test]
fn the_rest_pose_keeps_the_build_s_lengths() {
    let p = rest_pose(Kind::Player, V3::ZERO, fx::deg(70));
    let close = |a: V3, b: V3, cm: i32| (a.dist(b) - cm * ONE / 100).abs() <= 8;
    for s in [LEFT, RIGHT] {
        assert!(close(p.shoulder[s], p.elbow[s], 31), "upper arm {s}");
        assert!(close(p.elbow[s], p.hand[s], 29), "forearm {s}");
        assert!(close(p.hip[s], p.knee[s], 37), "thigh {s}");
        assert!(close(p.knee[s], p.foot[s], 35), "shin {s}");
        assert_eq!(p.foot[s].y, 0, "feet on the ground");
    }
    assert!((p.head.y - ONE * 146 / 100).abs() <= 4, "{}", p.head.y);
    // the short sword: the tip 80.6 cm from the grip
    assert!((p.tip.dist(p.grip) - mm(806)).abs() <= 4);
    // an orc's build is its own: its forearm 29 × 200/170 = 34.1 cm
    let o = rest_pose(Kind::Orc, V3::ZERO, 0);
    assert!((o.elbow[RIGHT].dist(o.hand[RIGHT]) - build(Kind::Orc).forearm).abs() <= 8);
}

/// A held item runs from grip to tip with the hilt at the grip: the
/// sword's handle is centred on the grip and its point ends at the tip.
#[test]
fn a_held_item_runs_from_its_grip_to_its_tip() {
    assert_eq!(grip_to_tip(Held::ShortSword), mm(80 + 26 + 700));
    assert_eq!(grip_to_tip(Held::Sword), mm(110 + 28 + 1000));
    let grip = v3(int(3), int(1), int(4));
    for dir in [
        v3(ONE, 0, 0),
        v3(0, 0, -ONE),
        v3(ONE * 6 / 10, ONE * 8 / 10, 0),
        v3(0, ONE, 0),
    ] {
        let tip = grip.add(dir.scale(grip_to_tip(Held::Sword)));
        let m = held_model(Held::Sword, grip, tip);
        let hilt = &m.parts[0];
        assert_eq!(hilt.mesh, Mesh::Cyl);
        assert!(near(hilt.at.t, grip, 2), "the hilt's middle is the grip");
        let point = m.parts.iter().find(|p| p.mesh == Mesh::Wedge).unwrap();
        assert!(
            near(ends(point).1, tip, 6),
            "the point ends at the tip: {:?} {:?}",
            ends(point).1,
            tip
        );
    }
    assert!(held_model(Held::Nothing, grip, grip.add(v3(ONE, 0, 0)))
        .parts
        .is_empty());
}

/// The lift a mark needs, worked by hand: the near layer's depth step at
/// its 128 m far plane is 159 Q16 units (2.43 mm, `render.rs`), the far
/// layer's at 1 km 34 units (0.53 mm), so four steps of the worse: 636
/// units, 9.7 mm. `DECAL_LIFT` (1 cm, 655 units) clears it.
#[test]
fn the_lift_a_mark_needs_is_four_depth_steps_at_the_worst_distance() {
    assert_eq!(decal_lift_needed(FIGURE_FAR), 4 * 159);
    assert!(DECAL_LIFT >= decal_lift_needed(FIGURE_FAR));
    // to 3 km: the far layer's step there is 9 × 34.6 = 311 units, four of
    // them 1 245 (19 mm), under `PROP_GAP` (25 mm, 1 638)
    let at_3km = decal_lift_needed(PROP_FAR);
    assert!((at_3km - 4 * 311).abs() <= 4, "{at_3km}");
    assert!(PROP_GAP >= at_3km);
}

/// How far a point is outside the ellipsoid part `base` along `-dir`, to
/// the tenth of a millimetre: where its implicit function changes sign.
fn depth_into(base: &Part, p: V3, dir: V3) -> Fx {
    let inside = |q: V3| {
        let rel = q.sub(base.at.t);
        let mut sum: i128 = 0;
        for c in [base.at.c0, base.at.c1, base.at.c2] {
            // the share of the column's half length (the mesh's radius ½)
            let l2 = c.x as i128 * c.x as i128 + c.y as i128 * c.y as i128 + c.z as i128 * c.z as i128;
            let dot = c.x as i128 * rel.x as i128 + c.y as i128 * rel.y as i128 + c.z as i128 * rel.z as i128;
            let u = dot * 2 * ONE as i128 / l2;
            sum += u * u;
        }
        sum <= (ONE as i128) * (ONE as i128)
    };
    let (mut lo, mut hi) = (0, mm(60));
    if inside(p) {
        return 0;
    }
    while hi - lo > 6 {
        let mid = (lo + hi) / 2;
        if inside(p.sub(dir.scale(mid))) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    lo
}

/// Every mark (eyes, catch-lights, blush, rivets, buttons, stripes) stands
/// off the surface under it by at least the lift the depth buffer needs
/// out to `FIGURE_FAR`, measured from the mark's outermost point (its dot's
/// pole) back along its normal to where the surface under it starts.
#[test]
fn every_mark_stands_off_its_surface_by_the_lift_the_depth_buffer_needs() {
    let need = decal_lift_needed(FIGURE_FAR);
    let mut checked = 0;
    let mut all: Vec<(String, Model)> = every_figure()
        .into_iter()
        .map(|(k, d, m)| (format!("{k:?} {d:?}"), m))
        .collect();
    all.extend(every_other_model());
    for (name, m) in all {
        for &(d, b) in &m.decals {
            let mark = &m.parts[d];
            let n = mark.at.c1.norm();
            let pole = mark.at.apply(v3(0, ONE / 2, 0));
            let lift = depth_into(&m.parts[b], pole, n);
            assert!(
                lift >= need - 6,
                "{name}: mark {d} on part {b} stands {} µm off, under {} µm",
                lift * 1_000_000 / ONE,
                need * 1_000_000 / ONE
            );
            checked += 1;
        }
    }
    assert!(checked > 80, "{checked} marks measured");
}

/// A model's parts as the flicker scan reads them.
fn insts(m: &Model) -> Vec<Inst> {
    m.parts
        .iter()
        .map(|p| Inst {
            mesh: p.mesh.into(),
            at: p.at,
            rgb: p.rgb,
            value: p.glow,
            pass: render::Pass::Solid,
        })
        .collect()
}

/// Cameras all round a model at `d`, at eye height and from above.
fn eyes_round(centre: V3, d: Fx) -> Vec<V3> {
    let mut out = Vec::new();
    for k in 0..8 {
        let a = fx::TURN * k / 8;
        for up in [ONE * 2, d / 2] {
            out.push(centre.add(v3(fx::mul(fx::cos(a), d), up, fx::mul(fx::sin(a), d))));
        }
    }
    out
}

fn scan(name: &str, m: &Model, centre: V3, far: Fx) -> Vec<String> {
    let i = insts(m);
    let table = crate::prim::MeshTable::default();
    let mut bad = Vec::new();
    // two parts in one plane, from anywhere; then planes too near for the
    // depth buffer from `far - 8 m` round the model, the farthest it is drawn
    let seen_far = Seen {
        bottom: centre.y,
        eyes: eyes_round(centre, far - int(8)),
        far,
    };
    for seen in [
        Seen {
            bottom: centre.y,
            ..Seen::near()
        },
        seen_far,
    ] {
        for f in flicker::fights(&i, &table, &seen) {
            bad.push(format!("{name}: {}", flicker::describe(&i, &f)));
        }
    }
    bad
}

/// No two parts of any figure share a plane facing one way (CW-18): the
/// flicker scan over every figure at rest holding its item, seen from
/// near and from 120 m, where the near layer's depth steps are coarsest.
#[test]
fn no_two_parts_of_any_figure_share_a_plane() {
    let mut bad = Vec::new();
    for (kind, dress, m) in every_figure() {
        let name = format!("{kind:?} {dress:?}");
        bad.extend(scan(&name, &m, v3(int(10), 0, int(20)), render::NEAR_LAYER_FAR));
        bad.extend(scan(&name, &m, v3(int(10), 0, int(20)), FIGURE_FAR));
    }
    assert!(bad.is_empty(), "{} fights:\n{}", bad.len(), bad.join("\n"));
}

/// Each speedster piece worn shows on its own part of the body: gloves on
/// the hands, trousers on the knees, boots on the feet, the jacket on the
/// chest, the hood over the head; and none worn shows none.
#[test]
fn each_speedster_piece_shows_on_its_own_part_of_the_body() {
    let pose = rest_pose(Kind::Player, V3::ZERO, 0);
    let speed = table().c("speed_base");
    let colour_at = |worn: u8, at: V3| {
        let m = figure_model(Kind::Player, &pose, Dress { worn, variant: 0 });
        m.parts
            .iter()
            .find(|p| p.mesh == Mesh::Facet && p.at.t == at)
            .map(|p| p.rgb)
    };
    let hand = pose.hand[RIGHT];
    let knee = pose.knee[RIGHT];
    assert_eq!(colour_at(GLOVES, hand), Some(speed));
    assert_ne!(colour_at(0, hand), Some(speed));
    assert_ne!(
        colour_at(TROUSERS, hand),
        Some(speed),
        "trousers leave the hands bare"
    );
    assert_eq!(colour_at(TROUSERS, knee), Some(speed));
    assert_ne!(colour_at(BOOTS, knee), Some(speed));
    let count = |worn: u8| {
        figure_model(Kind::Player, &pose, Dress { worn, variant: 0 })
            .parts
            .iter()
            .filter(|p| p.rgb == speed)
            .count()
    };
    assert_eq!(count(0), 0);
    for bit in [HOOD, JACKET, GLOVES, TROUSERS, BOOTS] {
        assert!(count(bit) > 0, "piece {bit} shows");
    }
    assert_eq!(
        count(SPEEDSTER_ALL),
        [HOOD, JACKET, GLOVES, TROUSERS, BOOTS]
            .iter()
            .map(|b| count(*b))
            .sum::<usize>()
    );
}

/// M7 (R9.88): bodies are drawn 8% broader across (`pose::BODY_GIRTH_PCT`),
/// heights unchanged: the player's hips, 2 × 152 mm across, are now
/// 2 × 152 × 1.08 = 328.3 mm; its thigh 2 × 76 × 1.08 = 164.2 mm thick; its
/// head as before (CW-17).
#[test]
fn bodies_are_drawn_eight_percent_broader_and_no_taller() {
    let pose = rest_pose(Kind::Player, V3::ZERO, 0);
    let m = figure_model(Kind::Player, &pose, Dress::default());
    // the legs' eight parts come first (each side: thigh, knee, shin, shoe),
    // then the hips
    let hips = m.parts[8].at.c0.len();
    assert!((hips - mm(328)).abs() <= mm(1), "hips {hips}");
    let thigh = m.parts[0].at.c0.len();
    assert!((thigh - mm(164)).abs() <= mm(1), "thigh {thigh}");
    assert_eq!(build(Kind::Player).head * 100 / build(Kind::Player).height, 24);
}
