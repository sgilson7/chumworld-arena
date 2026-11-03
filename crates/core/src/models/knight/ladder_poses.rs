//! M6: what the ladder's costume tests share: the poses an opponent takes in
//! a duel, how far a shape reaches, and the flicker scan's input. The same
//! measures as `knight/tests.rs`, for kinds the knights' tests do not list.

use super::*;
use crate::flicker::Inst;
use crate::fx::int;
use crate::input::{self, Input};
use crate::world::World;

/// Ape Race's cap on one costume.
pub const TRIANGLE_CAP: usize = 6_000;

/// The poses `kind` takes: standing at its guard every eighth of a turn,
/// then a duel as the player against Knight B on its tree (the script
/// draws, steps in, makes every cut, steps aside, then lunges forward
/// hard), every sixth tick, and fallen at the end.
pub fn poses(kind: Kind) -> Vec<Pose> {
    let mut out: Vec<Pose> = (0..8)
        .map(|k| rest_pose(kind, v3(int(3), 0, int(-2)), fx::TURN * k / 8))
        .collect();
    let mut w = World::new(7);
    w.act("duel.me", crate::pilot::kind_name(kind)).unwrap();
    w.act("duel.spawn", "knight_b").unwrap();
    assert_eq!(w.me.kind, kind);
    let cuts = [
        (-6000, 0, 0),
        (0, 6000, 0),
        (6000, 0, 0),
        (0, 0, -100),
        (0, -6000, 0),
    ];
    for t in 0..540u32 {
        let mut i = Input::default();
        if t == 0 {
            i.keys = input::DRAW;
        }
        if (30..60).contains(&t) || (300..330).contains(&t) || (480..540).contains(&t) {
            i.keys |= input::FORWARD;
        }
        if (360..380).contains(&t) {
            i.keys |= input::LEFT;
        }
        if (420..440).contains(&t) {
            i.keys |= input::BACK;
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
            out.push(crate::player::pose_of(&w, 0));
        }
    }
    w.me.health.hp = 0;
    for _ in 0..40 {
        w.step(&Input::default());
    }
    out.push(crate::player::pose_of(&w, 0));
    out
}

/// The body's right and up on a pose, as the model takes them.
pub fn axes(p: &Pose) -> (V3, V3) {
    let fwd = facing(p.facing, 0);
    let up0 = p.chest.sub(p.pelvis).norm();
    let up = if up0 == V3::ZERO { V3::UP } else { up0 };
    let back = fwd.neg().sub(up.scale(fwd.neg().dot(up))).norm();
    (up.cross(back).norm(), up)
}

/// How far a part reaches along the unit `d`: least and most.
pub fn reach(p: &Part, d: V3) -> (Fx, Fx) {
    let a = p.at;
    let e = (a.c0.dot(d).abs() + a.c1.dot(d).abs() + a.c2.dot(d).abs()) / 2;
    let m = a.t.dot(d);
    (m - e, m + e)
}

/// The least and most of the parts in `role` along `d`.
pub fn extent(k: &Knight, role: Role, d: V3) -> (Fx, Fx) {
    let (mut lo, mut hi) = (Fx::MAX, Fx::MIN);
    for t in k.parts.iter().filter(|t| t.role == role) {
        let (a, b) = reach(&t.part, d);
        lo = lo.min(a);
        hi = hi.max(b);
    }
    assert!(hi > lo, "no {role:?}");
    (lo, hi)
}

pub fn insts(k: &Knight) -> Vec<Inst> {
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

/// The flicker scan of `knight/tests.rs` on `kind`: every fourth pose whole,
/// and at rest with each part bare and all bare. Returns the fights found
/// and how many cases were scanned.
pub fn fights(kind: Kind) -> (Vec<String>, usize) {
    use crate::flicker::{self, Seen};
    let table = crate::prim::MeshTable::default();
    let square_cm = (ONE as i64) * (ONE as i64) / 10_000;
    let n = crate::blade::PARTS;
    let all = poses(kind);
    let mut cases: Vec<(Pose, u16)> = all.iter().step_by(4).map(|p| (*p, 0)).collect();
    for s in 0..n {
        cases.push((all[0], bit(s)));
    }
    cases.push((all[0], (1 << n) - 1));
    let mut bad = Vec::new();
    for (p, bare) in &cases {
        let i = insts(&model(kind, p, *bare));
        for f in flicker::fights_over(&i, &table, &Seen::near(), square_cm) {
            bad.push(format!("{kind:?} bare {bare:#x}: {}", flicker::describe(&i, &f)));
        }
    }
    (bad, cases.len())
}

/// The distance (0–255 RGB) from every slot of scheme `name` to each
/// highlight the duel draws on a body (`knight/tests.rs`, R5.8): the
/// nearest, and the slot and highlight it was.
pub fn nearest_highlight(name: &str) -> (i64, String) {
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
    let mut best = (i64::MAX, String::new());
    let slots = models["schemes"][name].as_object().expect("the scheme");
    for (slot, v) in slots {
        let c = rgb(v);
        for (m, h) in &marks {
            let d2: i64 = (0..3).map(|i| (c[i] - h[i]) * (c[i] - h[i])).sum();
            if d2 < best.0 {
                best = (d2, format!("{name}.{slot} {c:?} to {m} {h:?}"));
            }
        }
    }
    best
}
