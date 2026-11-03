//! The costume tests for rungs F, E and D (CA-7, Ape Race's costume flow,
//! as `knight/tests.rs` holds them for the knights): each keeps its
//! silhouette trait in every pose of a duel, no face lies in another
//! colour's plane, each stays under the triangle cap, each wears plate
//! exactly where the sheet draws steel, and the colours stand clear of the
//! duel's highlights.

use super::*;
use crate::blade::PARTS as PARTS_COUNT;
use crate::flicker::{self, Inst, Seen};
use crate::fx::int;
use crate::input::{self, Input};
use crate::plate::{covered, full, OPEN_HELM_MM};
use crate::world::World;

const FED: [Kind; 3] = [Kind::Squire, Kind::Sellsword, Kind::Hedge];
const TRIANGLE_CAP: usize = 6_000;

fn key(kind: Kind) -> &'static str {
    match kind {
        Kind::Squire => "squire",
        Kind::Sellsword => "sellsword",
        _ => "hedge",
    }
}

/// The poses an opponent takes: at its guard every eighth of a turn, then
/// a duel against Knight A (the player's script draws, steps in and makes
/// every cut; the opponent fights on its tree), every sixth tick, and
/// fallen at the end. The knights' own list (`knight/tests.rs`).
fn poses(kind: Kind) -> Vec<Pose> {
    let mut out: Vec<Pose> = (0..8)
        .map(|k| rest_pose(kind, v3(int(3), 0, int(-2)), fx::TURN * k / 8))
        .collect();
    let mut w = World::new(7);
    w.act("duel.me", "knight_a").unwrap();
    w.act("duel.spawn", key(kind)).unwrap();
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
            out.push(crate::player::pose_of(&w, 1));
        }
    }
    w.me.health.hp = 0;
    w.foes[0].fighter.health.hp = 0;
    for _ in 0..40 {
        w.step(&Input::default());
    }
    out.push(crate::player::pose_of(&w, 1));
    out
}

/// The model as the duel draws it: the parts with no plate by design are
/// bare in the plates the body comes in with.
fn worn(kind: Kind, p: &Pose) -> Knight {
    super::super::model(kind, p, crate::plate::bare(&full(kind)))
}

fn axes(p: &Pose) -> (V3, V3) {
    let fwd = facing(p.facing, 0);
    let up0 = p.chest.sub(p.pelvis).norm();
    let up = if up0 == V3::ZERO { V3::UP } else { up0 };
    let back = fwd.neg().sub(up.scale(fwd.neg().dot(up))).norm();
    (up.cross(back).norm(), up)
}

/// How far a part reaches along the unit `d` (its placed unit box).
fn reach(p: &Part, d: V3) -> (Fx, Fx) {
    let a = p.at;
    let e = (a.c0.dot(d).abs() + a.c1.dot(d).abs() + a.c2.dot(d).abs()) / 2;
    let m = a.t.dot(d);
    (m - e, m + e)
}

fn in_role(k: &Knight, role: Role) -> Vec<&Tagged> {
    let v: Vec<&Tagged> = k.parts.iter().filter(|t| t.role == role).collect();
    assert!(!v.is_empty(), "no {role:?}");
    v
}

fn span(k: &Knight, role: Role, d: V3) -> Fx {
    let (mut lo, mut hi) = (Fx::MAX, Fx::MIN);
    for t in in_role(k, role) {
        let (a, b) = reach(&t.part, d);
        lo = lo.min(a);
        hi = hi.max(b);
    }
    hi - lo
}

fn top(k: &Knight, role: Role, up: V3) -> Fx {
    in_role(k, role)
        .iter()
        .map(|t| reach(&t.part, up).1)
        .max()
        .unwrap()
}

/// The squire's brim spans at least half as wide again as his hat's
/// crown; the sellsword's pauldron stands at least 16 cm farther out from
/// the chest's middle than his bare left shoulder does, and he has none on
/// the left; the hedge knight's point stands at least 10 cm over his helm's
/// dome. In every pose.
#[test]
fn each_ladder_opponent_keeps_its_silhouette_in_every_pose() {
    for kind in FED {
        let all = poses(kind);
        assert!(all.len() > 80, "{} poses", all.len());
        for (n, p) in all.iter().enumerate() {
            let k = worn(kind, p);
            let (right, up) = axes(p);
            match kind {
                Kind::Squire => {
                    let (brim, crown) = (span(&k, Role::Brim, right), span(&k, Role::Helm, right));
                    assert!(brim * 100 >= crown * 150, "pose {n}: brim {brim}, crown {crown}");
                }
                Kind::Sellsword => {
                    let mid = p.chest.dot(right);
                    let out = in_role(&k, Role::Pauldron)
                        .iter()
                        .map(|t| {
                            assert_eq!(
                                t.seg as usize, UPPER_ARM[RIGHT],
                                "pose {n}: a pauldron on the left"
                            );
                            reach(&t.part, right).1
                        })
                        .max()
                        .unwrap()
                        - mid;
                    // the left shoulder's ball: its joint and the figure's
                    // 62 mm radius (scaled to 1.80 m and girth 105)
                    let left = mid - p.shoulder[LEFT].dot(right) + k_w(Kind::Sellsword, SHOULDER_R);
                    assert!(
                        out >= left + ONE * 16 / 100,
                        "pose {n}: pauldron {out}, left shoulder {left}"
                    );
                }
                _ => {
                    let (point, dome) = (top(&k, Role::Point, up), top(&k, Role::Helm, up));
                    assert!(point - dome >= ONE / 10, "pose {n}: point {point}, dome {dome}");
                }
            }
        }
    }
}

/// `K::w` for a kind, outside a builder.
fn k_w(kind: Kind, v: i32) -> Fx {
    let b = build(kind);
    let s = (v as i64 * ONE as i64 * b.height as i64 / (PLAYER_HEIGHT * ONE as i64)) as Fx;
    (s as i64 * b.girth_pct as i64 * crate::pose::BODY_GIRTH_PCT as i64 / 10_000) as Fx
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

/// No face lies within 2 mm (`flicker::PLANE_TOL`) of another face of
/// another colour facing the same way and overlapping it by more than a
/// square centimetre: in every fourth pose as worn, and at rest with each
/// plated part's plate broken off and with all of them off.
#[test]
fn no_face_of_a_ladder_opponent_shares_a_plane_with_another_colour() {
    let table = crate::prim::MeshTable::default();
    let square_cm = (ONE as i64) * (ONE as i64) / 10_000;
    let mut bad = Vec::new();
    let mut scanned = 0;
    for kind in FED {
        let all = poses(kind);
        let worn_bare = crate::plate::bare(&full(kind));
        let mut cases: Vec<(Pose, u16)> = all.iter().step_by(4).map(|p| (*p, worn_bare)).collect();
        for s in 0..PARTS_COUNT {
            if covered(kind) & bit(s) != 0 {
                cases.push((all[0], worn_bare | bit(s)));
            }
        }
        cases.push((all[0], (1 << PARTS_COUNT) - 1));
        for (p, bare) in cases {
            let i = insts(&super::super::model(kind, &p, bare));
            for f in flicker::fights_over(&i, &table, &Seen::near(), square_cm) {
                bad.push(format!("{kind:?} bare {bare:#x}: {}", flicker::describe(&i, &f)));
            }
            scanned += 1;
        }
    }
    assert!(scanned > 50, "{scanned}");
    assert!(bad.is_empty(), "{} fights:\n{}", bad.len(), bad.join("\n"));
}

/// Each is drawn in at most 6 000 triangles (solids).
#[test]
fn each_ladder_opponent_is_under_the_triangle_cap() {
    for kind in FED {
        let p = rest_pose(kind, V3::ZERO, 0);
        let n = worn(kind, &p).model().triangles();
        assert!(n <= TRIANGLE_CAP, "{kind:?}: {n} triangles");
        assert!(n > 1_000, "{kind:?}: {n} triangles is no costume");
    }
}

/// The plates each comes into the ring with, worked by hand from the
/// sheet and `plate::INTEGRITY` (head 66, torso 45, arm parts 27, leg
/// parts 30), by `blade::body`'s index (head, torso, left upper arm, left
/// forearm, right upper arm, right forearm, left thigh, left shin, right
/// thigh, right shin).
#[test]
fn each_ladder_opponent_wears_plate_where_the_sheet_draws_steel() {
    assert_eq!(full(Kind::Squire), [66, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(full(Kind::Sellsword), [66, 45, 0, 27, 27, 27, 0, 0, 0, 0]);
    assert_eq!(full(Kind::Hedge), [66, 45, 27, 27, 27, 27, 30, 0, 30, 0]);
    // the knights are untouched: plate everywhere
    assert_eq!(full(Kind::KnightA), crate::plate::INTEGRITY);
}

/// The model agrees with the rule: a covered part shows plate in every
/// pose and an uncovered one never does (its shapes are cloth, whatever
/// `bare` says of it); a covered part whose plate broke off shows mail and
/// no plate, and `plate` gives the pieces that fell.
#[test]
fn each_ladder_opponent_is_drawn_in_plate_only_where_it_wears_it() {
    for kind in FED {
        let c = covered(kind);
        let sc_mail = Garb(key(kind)).c("mail");
        for (n, p) in poses(kind).iter().enumerate() {
            for bare in [0, crate::plate::bare(&full(kind))] {
                let k = super::super::model(kind, p, bare);
                for s in 0..PARTS_COUNT {
                    let plated = k
                        .parts
                        .iter()
                        .any(|t| t.seg as usize == s && t.layer == Layer::Plate);
                    assert_eq!(
                        plated,
                        c & bit(s) != 0,
                        "{kind:?} pose {n} bare {bare:#x}: part {s}"
                    );
                    if c & bit(s) == 0 {
                        assert!(
                            k.parts
                                .iter()
                                .filter(|t| t.seg as usize == s)
                                .all(|t| t.layer == Layer::Cloth),
                            "{kind:?} pose {n}: part {s} not in its garment"
                        );
                    }
                }
            }
        }
        let p = rest_pose(kind, V3::ZERO, 0);
        let worn_bare = crate::plate::bare(&full(kind));
        for s in (0..PARTS_COUNT).filter(|&s| c & bit(s) != 0) {
            let k = super::super::model(kind, &p, worn_bare | bit(s));
            let on: Vec<&Tagged> = k.parts.iter().filter(|t| t.seg as usize == s).collect();
            assert!(
                on.iter().all(|t| t.layer != Layer::Plate),
                "{kind:?}: part {s} still plated"
            );
            assert!(
                on.iter().any(|t| t.layer == Layer::Mail && t.part.rgb == sc_mail),
                "{kind:?}: part {s} shows no mail"
            );
            let fell = super::super::plate(kind, &p, worn_bare | bit(s), s).len();
            let had = worn(kind, &p)
                .parts
                .iter()
                .filter(|t| t.seg as usize == s && t.layer == Layer::Plate)
                .count();
            assert!(
                fell > 0 && fell == had,
                "{kind:?}: part {s} sheds {fell} of {had}"
            );
        }
    }
}

/// In the ring, a 30 m/s cut on the right forearm takes the squire's off
/// (padding is bare) and leaves the sellsword's on (a vambrace), its plate
/// worn instead; on the shin the hedge knight's comes off (hose) and his
/// thigh stays (a cuisse). CHUMWORLD's take-off speed for a foe's limb is
/// far under 30 m/s.
#[test]
fn a_part_without_plate_is_cut_off_and_a_plated_one_is_not() {
    let lost = |kind: Kind, part: &str, s: usize| {
        let mut w = World::new(3);
        w.act("duel.spawn", key(kind)).unwrap();
        w.act("duel.pilot", "still").unwrap();
        w.act("wounds.cut", &format!("{part},30")).unwrap();
        let f = &w.foes[0].fighter;
        (f.health.limbs.lost & bit(s) != 0, f.health.limbs.plate[s])
    };
    assert_eq!(lost(Kind::Squire, "forearm_r", FOREARM[RIGHT]), (true, 0));
    let (gone, left) = lost(Kind::Sellsword, "forearm_r", FOREARM[RIGHT]);
    assert!(
        !gone && left < 27,
        "sellsword's forearm: gone {gone}, plate {left}"
    );
    assert_eq!(lost(Kind::Hedge, "shin_r", SHIN[RIGHT]), (true, 0));
    let (gone, _) = lost(Kind::Hedge, "thigh_r", THIGH[RIGHT]);
    assert!(!gone, "the hedge knight's cuisse");
}

/// The squire's hat covers his head only above its brim: a level blow
/// from in front at the head's middle comes in whole (`Gap::Open`), the
/// same blow on the hedge knight meets plate, and one from above on the
/// squire meets the hat. The brim's lowest edge is drawn where the open
/// helm ends, within 5 mm.
#[test]
fn the_squire_s_hat_is_plate_only_above_its_brim() {
    use crate::plate::{gap, scaled, Gap};
    for kind in [Kind::Squire, Kind::Hedge] {
        let p = rest_pose(kind, V3::ZERO, 0);
        let caps = crate::blade::body(kind, &p);
        let fwd = facing(p.facing, 0);
        let mid = caps[HEAD].a;
        let want = if kind == Kind::Squire {
            Some(Gap::Open)
        } else {
            None
        };
        assert_eq!(
            gap(kind, &p, HEAD, mid, mid.add(fwd.scale(ONE / 4))),
            want,
            "{kind:?}"
        );
        let crown = caps[HEAD].b;
        assert_eq!(
            gap(kind, &p, HEAD, crown, crown.add(V3::UP.scale(ONE / 4))),
            None,
            "{kind:?}"
        );
    }
    let p = rest_pose(Kind::Squire, V3::ZERO, 0);
    let k = worn(Kind::Squire, &p);
    let (_, up) = axes(&p);
    let low = in_role(&k, Role::Brim)
        .iter()
        .map(|t| reach(&t.part, up).0)
        .min()
        .unwrap();
    let want = p.head.dot(up) + scaled(Kind::Squire, OPEN_HELM_MM);
    assert!(
        (low - want).abs() <= ONE / 200,
        "brim at {low}, open helm ends at {want}"
    );
}

/// The sellsword's and the hedge knight's slits are where `plate::gap`
/// finds the visor (the knights' test).
#[test]
fn the_drawn_slit_is_where_the_visor_gap_is_on_the_closed_helms() {
    for kind in [Kind::Sellsword, Kind::Hedge] {
        let p = rest_pose(kind, V3::ZERO, 0);
        let k = worn(kind, &p);
        let (_, up) = axes(&p);
        let slit = in_role(&k, Role::Slit)[0];
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

/// Every slot of each scheme stands 100 or more (RGB distance, 0–255) from
/// the hit flash, blood, the stump and the lamp colour (R5.8's rule).
#[test]
fn the_ladder_schemes_stand_clear_of_the_highlight_colours() {
    let models: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../../data/models.json")).unwrap();
    let wounds: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../../data/wounds.json")).unwrap();
    let look: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../../data/look.json")).unwrap();
    let rgb = |v: &serde_json::Value| -> [i64; 3] { [0, 1, 2].map(|i| v[i].as_i64().unwrap()) };
    let mut marks = Vec::new();
    for n in ["flash", "blood", "stump"] {
        marks.push((n, rgb(&wounds["colours"][n])));
    }
    marks.push(("lamp", rgb(&look["palette"]["lamp"])));
    let mut checked = 0;
    for kind in FED {
        let (name, slots) = scheme_of(kind).unwrap();
        for slot in slots {
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
    // 9 squire slots, 8 sellsword, 8 hedge, each against 4 marks
    assert_eq!(checked, (9 + 8 + 8) * 4);
}
