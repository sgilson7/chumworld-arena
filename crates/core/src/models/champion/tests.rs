//! The champion's costume tests (CA-7, Ape Race's costume flow), and the
//! helpers the mirror's share: the poses a duel between the two puts them
//! in, and the outline measured along an axis. Each keeps its silhouette
//! in every pose, no plate face lies in another colour's plane, it stays
//! under the triangle cap, every part is dressed, and the schemes stand
//! clear of the duel's highlight colours.

use super::super::knight::{model, plate, scheme, Knight, Tagged, SLOTS};
use super::*;
use crate::blade::PARTS;
use crate::flicker::{self, Inst, Seen};
use crate::fx::int;
use crate::geom::facing;
use crate::input::{self, Input};
use crate::pose::Kind;
use crate::world::World;
use crate::wounds::bit;

/// Ape Race's cap on one costume.
pub(in crate::models) const TRIANGLE_CAP: usize = 6_000;

/// The poses `kind` (the champion or the mirror) takes: standing at its
/// guard every eighth of a turn, then a duel between the two (the champion
/// the player, its script drawing, stepping in and making every cut; the
/// mirror on Knight B's tree), every sixth tick, and fallen at the end.
pub(in crate::models) fn poses(kind: Kind) -> Vec<Pose> {
    let mut out: Vec<Pose> = (0..8)
        .map(|k| rest_pose(kind, v3(int(3), 0, int(-2)), fx::TURN * k / 8))
        .collect();
    let mut w = World::new(7);
    w.act("duel.me", "champion").unwrap();
    w.act("duel.spawn", "mirror").unwrap();
    assert_eq!(w.me.kind, Kind::Champion);
    assert_eq!(w.foes[0].fighter.kind, Kind::Mirror);
    let id = if kind == Kind::Champion { 0 } else { 1 };
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
    w.me.health.hp = 0;
    w.foes[0].fighter.health.hp = 0;
    for _ in 0..40 {
        w.step(&Input::default());
    }
    out.push(crate::player::pose_of(&w, id));
    out
}

/// The body's right and up on a pose, as the model takes them.
pub(in crate::models) fn axes(p: &Pose) -> (V3, V3) {
    let fwd = facing(p.facing, 0);
    let up0 = p.chest.sub(p.pelvis).norm();
    let up = if up0 == V3::ZERO { V3::UP } else { up0 };
    let back = fwd.neg().sub(up.scale(fwd.neg().dot(up))).norm();
    (up.cross(back).norm(), up)
}

/// How far a part reaches along the unit `d`: its middle less and plus its
/// half-extent (the unit mesh's box, placed).
fn reach(p: &Part, d: V3) -> (Fx, Fx) {
    let a = p.at;
    let e = (a.c0.dot(d).abs() + a.c1.dot(d).abs() + a.c2.dot(d).abs()) / 2;
    let m = a.t.dot(d);
    (m - e, m + e)
}

/// The lowest and highest the parts in `role` reach along `d`.
pub(in crate::models) fn extent(k: &Knight, role: Role, d: V3) -> (Fx, Fx) {
    let (mut lo, mut hi) = (Fx::MAX, Fx::MIN);
    for t in k.parts.iter().filter(|t| t.role == role) {
        let (a, b) = reach(&t.part, d);
        lo = lo.min(a);
        hi = hi.max(b);
    }
    assert!(hi > lo, "no {role:?}");
    (lo, hi)
}

pub(in crate::models) fn span(k: &Knight, role: Role, d: V3) -> Fx {
    let (lo, hi) = extent(k, role, d);
    hi - lo
}

pub(in crate::models) fn insts(k: &Knight) -> Vec<Inst> {
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

/// No face of `kind` lies within 2 mm (`flicker::PLANE_TOL`) of another
/// face of another colour facing the same way and overlapping it by more
/// than a square centimetre: in every fourth pose whole, and at rest with
/// each part's plate broken off and with all of them off.
pub(in crate::models) fn faces_fight(kind: Kind) -> (usize, Vec<String>) {
    let table = crate::prim::MeshTable::default();
    let square_cm = (ONE as i64) * (ONE as i64) / 10_000;
    let mut bad = Vec::new();
    let all = poses(kind);
    let mut cases: Vec<(Pose, u16)> = all.iter().step_by(4).map(|p| (*p, 0)).collect();
    for s in 0..PARTS {
        cases.push((all[0], bit(s)));
    }
    cases.push((all[0], (1 << PARTS) - 1));
    for (p, bare) in &cases {
        let i = insts(&model(kind, p, *bare));
        for f in flicker::fights_over(&i, &table, &Seen::near(), square_cm) {
            bad.push(format!("{kind:?} bare {bare:#x}: {}", flicker::describe(&i, &f)));
        }
    }
    (cases.len(), bad)
}

/// Every part of `kind` wears plate in every pose; a part whose plate is
/// gone wears none and shows the scheme's mail, and what breaks off is
/// what was on it.
pub(in crate::models) fn dressed_in_every_pose(kind: Kind) {
    let sc = scheme(kind);
    for (n, p) in poses(kind).iter().enumerate() {
        let k = model(kind, p, 0);
        for s in 0..PARTS {
            assert!(
                k.parts
                    .iter()
                    .any(|t| t.seg as usize == s && t.layer == Layer::Plate),
                "{kind:?} pose {n}: part {s} bare"
            );
        }
    }
    let p = rest_pose(kind, V3::ZERO, 0);
    for s in 0..PARTS {
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
        let whole = model(kind, &p, 0);
        let plates = whole
            .parts
            .iter()
            .filter(|t| t.seg as usize == s && t.layer == Layer::Plate)
            .count();
        assert!(plates > 0);
        assert_eq!(plate(kind, &p, bit(s), s).len(), plates);
    }
}

/// Every slot of `name`'s scheme stands 100 or more (RGB distance, 0–255)
/// from the hit flash, blood, stump and lamp colours (R5.8's rule).
pub(in crate::models) fn scheme_clear_of_highlights(name: &str) {
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
    assert_eq!(checked, 8 * 4);
}

/// The champion has the broadest shoulders on the ladder's plate and a
/// winged helm: its pauldrons span at least a third wider than its skirt
/// and at least as wide as Knight A's on the same pose (1.0×), its wings
/// rise at least 12 cm over the great helm's flat top and reach at least
/// 8 cm past the helm on each side, and it wears no crown (Knight B's), in
/// every pose.
#[test]
fn the_champion_keeps_its_silhouette_in_every_pose() {
    let all = poses(Kind::Champion);
    assert!(all.len() > 80, "{} poses", all.len());
    for (n, p) in all.iter().enumerate() {
        let k = model(Kind::Champion, p, 0);
        let a = model(Kind::KnightA, p, 0);
        let (right, up) = axes(p);
        let (shoulders, hips) = (span(&k, Role::Pauldron, right), span(&k, Role::Hip, right));
        assert!(
            shoulders * 100 >= hips * 133,
            "pose {n}: pauldrons {shoulders}, skirt {hips}"
        );
        let a_shoulders = span(&a, Role::Pauldron, right);
        assert!(
            shoulders >= a_shoulders,
            "pose {n}: {shoulders} against A's {a_shoulders}"
        );
        let (wing, helm) = (extent(&k, Role::Wing, up).1, extent(&k, Role::Helm, up).1);
        assert!(
            wing - helm >= ONE * 12 / 100,
            "pose {n}: wings {wing}, helm {helm}"
        );
        let ((wl, wr), (hl, hr)) = (extent(&k, Role::Wing, right), extent(&k, Role::Helm, right));
        assert!(
            hl - wl >= ONE * 8 / 100 && wr - hr >= ONE * 8 / 100,
            "pose {n}: wings out"
        );
        assert!(k.parts.iter().all(|t| t.role != Role::Crown), "pose {n}: a crown");
    }
}

#[test]
fn no_face_of_the_champion_shares_a_plane_with_another_colour() {
    let (scanned, bad) = faces_fight(Kind::Champion);
    assert!(scanned > 25, "{scanned}");
    assert!(bad.is_empty(), "{} fights:\n{}", bad.len(), bad.join("\n"));
}

#[test]
fn the_champion_is_under_the_triangle_cap() {
    let p = rest_pose(Kind::Champion, V3::ZERO, 0);
    let n = model(Kind::Champion, &p, 0).model().triangles();
    assert!(n <= TRIANGLE_CAP, "{n} triangles");
    assert!(n > 1_000, "{n} triangles is no costume");
}

#[test]
fn the_champion_is_dressed_in_every_pose() {
    dressed_in_every_pose(Kind::Champion);
}

/// The champion's great helm draws its slit where `plate::gap` finds the
/// visor: between `VISOR_MM` over the head's middle, in front of it.
#[test]
fn the_champion_s_slit_is_where_the_visor_gap_is() {
    let p = rest_pose(Kind::Champion, V3::ZERO, 0);
    let k = model(Kind::Champion, &p, 0);
    let (_, up) = axes(&p);
    let slit = k.parts.iter().find(|t| t.role == Role::Slit).expect("a slit");
    let rel = slit.part.at.t.sub(p.head);
    let y = rel.dot(up);
    let (lo, hi) = (
        crate::plate::scaled(Kind::Champion, VISOR_MM.0),
        crate::plate::scaled(Kind::Champion, VISOR_MM.1),
    );
    assert!(lo <= y && y <= hi, "slit at {y}, gap {lo}..{hi}");
    assert!(rel.dot(facing(p.facing, 0)) > ONE / 10, "the slit is in front");
}

#[test]
fn the_champion_s_scheme_stands_clear_of_the_highlight_colours() {
    scheme_clear_of_highlights("champion");
}

/// Plate coverage as the sheet draws it: the champion and the mirror wear
/// plate on all ten parts, whole when they come into the ring as the
/// player or as a foe, with the knights' integrities (66 on the head, 45
/// the torso, 27 each arm part, 30 each leg part), and their longsword.
#[test]
fn the_champion_and_the_mirror_come_in_whole_plate_with_the_longsword() {
    let whole = [66, 45, 27, 27, 27, 27, 30, 30, 30, 30];
    for kind in [Kind::Champion, Kind::Mirror] {
        assert!(crate::plate::armoured(kind), "{kind:?}");
        assert_eq!(crate::plate::full(kind), whole, "{kind:?}");
        assert_eq!(crate::plate::bare(&crate::plate::full(kind)), 0);
        assert_eq!(kind_held(kind), crate::pose::Held::Longsword);
    }
    let mut w = World::new(3);
    w.act("duel.me", "mirror").unwrap();
    w.act("duel.spawn", "champion").unwrap();
    assert_eq!(w.me.health.limbs.plate, whole);
    assert_eq!(w.foes[0].fighter.health.limbs.plate, whole);
    assert_eq!(w.foes[0].fighter.kind, Kind::Champion);
}
