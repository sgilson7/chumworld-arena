//! The knights' costume tests (CA-7, Ape Race's costume flow): each keeps
//! its silhouette in every pose the figure takes in a duel, no plate face
//! lies in another colour's plane, each stays under the triangle cap,
//! every part is dressed, and the schemes stand clear of the duel's
//! highlight colours.

use super::*;
use crate::blade::PARTS as PARTS_COUNT;
use crate::flicker::{self, Inst, Seen};
use crate::fx::int;
use crate::input::{self, Input};
use crate::pose::Kind;
use crate::world::World;

const KNIGHTS: [Kind; 2] = [Kind::KnightA, Kind::KnightB];

/// Ape Race's cap on one costume.
const TRIANGLE_CAP: usize = 6_000;

/// The poses a knight takes: standing at its guard every eighth of a
/// turn, then a duel between the two knights (the player's script draws,
/// steps in and makes every cut while the computer's knight fights on its
/// tree), every sixth tick of both, and each fallen at the end.
fn poses(kind: Kind) -> Vec<Pose> {
    let mut out: Vec<Pose> = (0..8)
        .map(|k| rest_pose(kind, v3(int(3), 0, int(-2)), fx::TURN * k / 8))
        .collect();
    let mut w = World::new(7);
    w.act("duel.me", "knight_a").unwrap();
    w.act("duel.spawn", "knight_b").unwrap();
    let id = if kind == Kind::KnightA { 0 } else { 1 };
    let cuts = [
        (-6000, 0, 0),
        (0, 6000, 0),
        (6000, 0, 0),
        (0, 0, -100),
        (0, -6000, 0),
    ];
    for t in 0..480u32 {
        let mut i = Input::default();
        if t == 0 {
            i.keys = input::DRAW;
        }
        if (30..60).contains(&t) || (300..330).contains(&t) {
            i.keys |= input::FORWARD;
        }
        if (360..380).contains(&t) {
            i.keys |= input::LEFT;
        }
        let k = (t / 60) as usize;
        if (1..=5).contains(&k) {
            let (dx, dy, wheel) = cuts[k - 1];
            if t % 60 == 0 {
                i.dx = dx;
                i.dy = dy;
                i.wheel = wheel;
            }
            if t % 60 < 40 {
                i.buttons |= input::SWING;
            }
        }
        w.step(&i);
        if t % 6 == 0 {
            out.push(crate::player::pose_of(&w, id));
        }
    }
    // down: both fall over
    w.me.health.hp = 0;
    w.foes[0].fighter.health.hp = 0;
    for _ in 0..40 {
        w.step(&Input::default());
    }
    out.push(crate::player::pose_of(&w, id));
    out
}

/// The body's right and up on a pose, as the model takes them.
fn axes(p: &Pose) -> (V3, V3) {
    let fwd = facing(p.facing, 0);
    let up0 = p.chest.sub(p.pelvis).norm();
    let up = if up0 == V3::ZERO { V3::UP } else { up0 };
    let back = fwd.neg().sub(up.scale(fwd.neg().dot(up))).norm();
    (up.cross(back).norm(), up)
}

/// How far a part reaches along the unit `d`: its middle's place, less and
/// plus its half-extent (the unit mesh's box, placed).
fn reach(p: &Part, d: V3) -> (Fx, Fx) {
    let a = p.at;
    let e = (a.c0.dot(d).abs() + a.c1.dot(d).abs() + a.c2.dot(d).abs()) / 2;
    let m = a.t.dot(d);
    (m - e, m + e)
}

/// The span of the parts in `role` along `d`.
fn span(k: &Knight, role: Role, d: V3) -> Fx {
    let (mut lo, mut hi) = (Fx::MAX, Fx::MIN);
    for t in k.parts.iter().filter(|t| t.role == role) {
        let (a, b) = reach(&t.part, d);
        lo = lo.min(a);
        hi = hi.max(b);
    }
    assert!(hi > lo, "no {role:?}");
    hi - lo
}

fn top(k: &Knight, role: Role, up: V3) -> Fx {
    k.parts
        .iter()
        .filter(|t| t.role == role)
        .map(|t| reach(&t.part, up).1)
        .max()
        .expect("a part in the role")
}

/// Knight A is an inverted triangle: its pauldrons span at least a fifth
/// wider than its fauld, in every pose. Knight B is an hourglass that ends
/// in points: its hem at least a third wider than its belt, and its crown's
/// spikes at least 5 cm over its helm's top, in every pose.
#[test]
fn each_knight_keeps_its_silhouette_in_every_pose() {
    for kind in KNIGHTS {
        let all = poses(kind);
        assert!(all.len() > 80, "{} poses", all.len());
        for (n, p) in all.iter().enumerate() {
            let k = model(kind, p, 0);
            let (right, up) = axes(p);
            if kind == Kind::KnightA {
                let (shoulders, hips) = (span(&k, Role::Pauldron, right), span(&k, Role::Hip, right));
                assert!(
                    shoulders * 100 >= hips * 120,
                    "pose {n}: pauldrons {shoulders}, fauld {hips}"
                );
            } else {
                let (hem, waist) = (span(&k, Role::Hem, right), span(&k, Role::Waist, right));
                assert!(hem * 100 >= waist * 133, "pose {n}: hem {hem}, belt {waist}");
                let (crown, helm) = (top(&k, Role::Crown, up), top(&k, Role::Helm, up));
                assert!(crown - helm >= ONE / 20, "pose {n}: crown {crown}, helm {helm}");
            }
        }
    }
}

fn insts(k: &Knight) -> Vec<Inst> {
    k.parts
        .iter()
        .map(|t| Inst {
            mesh: t.part.mesh.into(),
            at: t.part.at,
            rgb: t.part.rgb,
            value: t.part.glow,
            pass: render::Pass::Solid,
        })
        .collect()
}

/// No face of a knight lies within 2 mm (`flicker::PLANE_TOL`) of another
/// face of another colour facing the same way and overlapping it by more
/// than a square centimetre (Ape Race's `no_plate_shares_a_face_with_
/// another_colour`; the figure scan lets be anything under 100 cm²): in
/// every fourth pose whole, and at rest with each part's plate broken off
/// and with all of them off.
#[test]
fn no_plate_shares_a_face_with_another_colour() {
    let table = crate::prim::MeshTable::default();
    let square_cm = (ONE as i64) * (ONE as i64) / 10_000;
    let mut bad = Vec::new();
    let mut scanned = 0;
    for kind in KNIGHTS {
        let all = poses(kind);
        let mut cases: Vec<(Pose, u16)> = all.iter().step_by(4).map(|p| (*p, 0)).collect();
        for s in 0..PARTS_COUNT {
            cases.push((all[0], bit(s)));
        }
        cases.push((all[0], (1 << PARTS_COUNT) - 1));
        for (p, bare) in cases {
            let i = insts(&model(kind, &p, bare));
            for f in flicker::fights_over(&i, &table, &Seen::near(), square_cm) {
                bad.push(format!("{kind:?} bare {bare:#x}: {}", flicker::describe(&i, &f)));
            }
            scanned += 1;
        }
    }
    assert!(scanned > 50, "{scanned}");
    assert!(bad.is_empty(), "{} fights:\n{}", bad.len(), bad.join("\n"));
}

/// Each knight is drawn in at most 6 000 triangles, counted from the
/// meshes as Ape Race counts them (solids; an outline draws as many again).
#[test]
fn each_knight_is_under_the_triangle_cap() {
    for kind in KNIGHTS {
        let p = rest_pose(kind, V3::ZERO, 0);
        let n = model(kind, &p, 0).model().triangles();
        println!("{kind:?}: {n} triangles");
        assert!(n <= TRIANGLE_CAP, "{kind:?}: {n} triangles");
        assert!(n > 1_000, "{kind:?}: {n} triangles is no costume");
    }
}

/// Every part of the body wears plate in every pose; a part whose plate is
/// gone wears none, and its body is drawn in the scheme's mail.
#[test]
fn every_part_is_dressed_in_every_pose() {
    for kind in KNIGHTS {
        let sc = scheme(kind);
        for (n, p) in poses(kind).iter().enumerate() {
            let k = model(kind, p, 0);
            for s in 0..PARTS_COUNT {
                assert!(
                    k.parts
                        .iter()
                        .any(|t| t.seg as usize == s && t.layer == Layer::Plate),
                    "{kind:?} pose {n}: part {s} bare"
                );
            }
        }
        let p = rest_pose(kind, V3::ZERO, 0);
        for s in 0..PARTS_COUNT {
            let k = model(kind, &p, bit(s));
            let on: Vec<&Tagged> = k.parts.iter().filter(|t| t.seg as usize == s).collect();
            assert!(
                on.iter().all(|t| t.layer != Layer::Plate),
                "{kind:?}: part {s} still plated"
            );
            assert!(
                on.iter().any(|t| t.layer == Layer::Mail && t.part.rgb == sc.mail),
                "{kind:?}: part {s} shows no mail"
            );
            assert_eq!(
                plate(kind, &p, bit(s), s).len(),
                model(kind, &p, 0)
                    .parts
                    .iter()
                    .filter(|t| t.seg as usize == s && t.layer == Layer::Plate)
                    .count()
            );
        }
    }
}

/// The slit each helm shows is where `plate::gap` finds the visor: its
/// middle between `VISOR_MM` over the head's middle, in front of it.
#[test]
fn the_drawn_slit_is_where_the_visor_gap_is() {
    for kind in KNIGHTS {
        let p = rest_pose(kind, V3::ZERO, 0);
        let k = model(kind, &p, 0);
        let (_, up) = axes(&p);
        let slit = k.parts.iter().find(|t| t.role == Role::Slit).expect("a slit");
        let rel = slit.part.at.t.sub(p.head);
        let y = rel.dot(up);
        let (lo, hi) = (
            crate::plate::scaled(kind, VISOR_MM.0),
            crate::plate::scaled(kind, VISOR_MM.1),
        );
        assert!(lo <= y && y <= hi, "{kind:?}: slit at {y}, gap {lo}..{hi}");
        assert!(
            rel.dot(facing(p.facing, 0)) > ONE / 10,
            "{kind:?}: the slit is in front"
        );
    }
}

/// Every slot of each knight's scheme stands 100 or more (RGB distance,
/// 0–255) from each highlight the duel draws on or near a body: the hit
/// flash, blood and stump (`data/wounds.json`), and the lamp colour the
/// sparks, the cut's cue and trail and the hand's target are drawn in
/// (`data/look.json`). R5.8 says which colours count and why.
#[test]
fn the_schemes_stand_clear_of_the_highlight_colours() {
    let models: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../data/models.json")).unwrap();
    let wounds: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../data/wounds.json")).unwrap();
    let look: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../data/look.json")).unwrap();
    let rgb = |v: &serde_json::Value| -> [i64; 3] { [0, 1, 2].map(|i| v[i].as_i64().unwrap()) };
    let mut marks = Vec::new();
    for n in ["flash", "blood", "stump"] {
        marks.push((n, rgb(&wounds["colours"][n])));
    }
    marks.push(("lamp", rgb(&look["palette"]["lamp"])));
    let mut checked = 0;
    for name in ["knight_a", "knight_b"] {
        for slot in SLOTS {
            let c = rgb(&models["schemes"][name][slot]);
            for (m, h) in &marks {
                let d2: i64 = (0..3).map(|i| (c[i] - h[i]) * (c[i] - h[i])).sum();
                assert!(
                    d2 >= 100 * 100,
                    "{name}.{slot} {c:?} is sqrt({d2}) from {m} {h:?}"
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 2 * 8 * 4);
}

/// The knights' one weapon (CA-Q12: A): a 95 cm blade on a 26 cm grip for
/// both hands, drawn 130 + 28 + 950 = 1 108 mm from the grip to the tip,
/// its heft 80 between the sword's 75 and the short sword's 100 (R5.2).
#[test]
fn the_longsword_is_two_handed_with_a_95_cm_blade() {
    use crate::pose::Held;
    assert_eq!(kind_held(Kind::KnightA), Held::Longsword);
    assert_eq!(kind_held(Kind::KnightB), Held::Longsword);
    assert_eq!(grip_to_tip(Held::Longsword), mm(1108));
    let w = crate::arm::weapon(Held::Longsword);
    assert!(w.two_handed());
    assert_eq!(w.hilt, 26 * ONE / 100);
    assert_eq!(w.heft_pct, 80);
    let (sword, short) = (
        crate::arm::weapon(Held::Sword),
        crate::arm::weapon(Held::ShortSword),
    );
    assert!(sword.heft_pct < w.heft_pct && w.heft_pct < short.heft_pct);
}
