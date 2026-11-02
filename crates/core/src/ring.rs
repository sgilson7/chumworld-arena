//! The stone ring, drawn: what never moves (M2, CA-Q11: A; R2.14, R4.1).
//!
//! From the middle out: a floor of flagstones under a dusting of sand out
//! to `world::RADIUS` (20 m, 40 m across), each stone its own slightly
//! tilted facet a few millimetres over a darker bed of sand, so the light
//! picks the stones out one by one (the faceted look of Protection of the
//! Spheres and CHUMWORLD); a course of greyer stones marks the inner ring,
//! `rounds::SAFE_RADIUS` (6 m), sudden death's safe ground. At the floor's
//! edge a kerb `KERB_HIGH` (1.4 m) tall, then a walk, then the wall
//! `WALL_HIGH` (4 m) tall with pilasters, banners hung between them, and
//! behind it the stands rising in dark tiers with a crowd in silhouette.
//! Two gate towers stand over the wall at the ends of the opening line.
//! The sky is the far layer's dome (`Shapes::sky`).
//!
//! Each surface is one mesh with a colour per vertex (`MeshTable::add`),
//! so the floor, kerb, wall and stands are four instances; the banners and
//! towers are placed primitives: `INSTANCES` in all. Colours and the
//! presentation's sizes are `data/ring.json`'s; the floor's radius is the
//! rule's (`world::RADIUS`).

use std::sync::OnceLock;

use serde::Deserialize;

use crate::fx::{self, Fx, ONE};
use crate::geom::{v3, Affine, V3};
use crate::look::{rgb, Rgb};
use crate::prim::{push_coloured, Mesh, MeshTable};
use crate::render::Group;
use crate::world::RADIUS;

/// `data/ring.json`: the ring's colours (0–255) and sizes (centimetres).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Spec {
    pub sand: Vec<[i32; 3]>,
    pub bed: [i32; 3],
    pub inner: [i32; 3],
    pub kerb: [i32; 3],
    pub kerb_cap: [i32; 3],
    pub wall: [i32; 3],
    pub wall_cap: [i32; 3],
    pub pilaster: [i32; 3],
    pub stands: [i32; 3],
    pub crowd: Vec<[i32; 3]>,
    pub banner: Vec<[i32; 3]>,
    pub pole: [i32; 3],
    pub tower: [i32; 3],
    pub stone_cm: i32,
    pub joint_cm: i32,
    pub lift_mm: [i32; 2],
    pub kerb_cm: [i32; 2],
    pub walk_cm: i32,
    pub wall_cm: [i32; 2],
    pub banners: i32,
    pub banner_cm: [i32; 2],
    pub tiers: i32,
    pub tier_cm: [i32; 2],
    pub crowd_pct: i32,
    pub tower_cm: [i32; 2],
}

impl Spec {
    pub fn parse(json: &str) -> Result<Spec, String> {
        let mut v: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if let Some(o) = v.as_object_mut() {
            o.retain(|k, _| !k.starts_with('_'));
        }
        serde_json::from_value(v).map_err(|e| e.to_string())
    }
}

/// The ring's presentation, read once.
pub fn spec() -> &'static Spec {
    static SPEC: OnceLock<Spec> = OnceLock::new();
    SPEC.get_or_init(|| Spec::parse(include_str!("../../../data/ring.json")).expect("data/ring.json"))
}

fn cm(c: i32) -> Fx {
    c * ONE / 100
}

/// The kerb's height and the wall's, for the tests and the doc.
pub const KERB_HIGH: Fx = ONE * 14 / 10;
pub const WALL_HIGH: Fx = ONE * 4;

/// How many instances the ring is drawn with: floor, kerb, wall, stands,
/// a pole and a cloth for each banner, two towers each with a cap.
pub fn instances() -> usize {
    4 + 2 * spec().banners as usize + 4
}

/// A small fixed hash for the stones' and the crowd's variety: the same
/// numbers give the same ring on every machine.
fn hash(a: i32, b: i32, c: i32) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for v in [a, b, c] {
        for byte in v.to_le_bytes() {
            h ^= byte as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
    }
    h
}

fn polar(r: Fx, a: i32, y: Fx) -> V3 {
    v3(fx::mul(r, fx::cos(a)), y, fx::mul(r, fx::sin(a)))
}

/// The circles the courses of stones lie between, out to `RADIUS`: courses
/// about `stone_cm` deep, and a narrow course across `SAFE_RADIUS` for the
/// inner ring's marker. Returns (inner, outer, marker) for each course.
pub fn courses() -> Vec<(Fx, Fx, bool)> {
    let s = spec();
    let safe = crate::rounds::SAFE_RADIUS;
    let half_mark = cm(s.stone_cm) / 8;
    let mut out = Vec::new();
    let fill = |from: Fx, to: Fx, out: &mut Vec<(Fx, Fx, bool)>| {
        let n = ((to - from + cm(s.stone_cm) / 2) / cm(s.stone_cm)).max(1);
        for k in 0..n {
            out.push((
                from + (to - from) * k / n,
                from + (to - from) * (k + 1) / n,
                false,
            ));
        }
    };
    fill(0, safe - half_mark, &mut out);
    out.push((safe - half_mark, safe + half_mark, true));
    fill(safe + half_mark, RADIUS, &mut out);
    out
}

/// The floor: a bed of sand at 0, and on it each flagstone a facet `lift`
/// millimetres up, its corners lifted unevenly, inset by half a joint.
pub fn floor(s: &Spec) -> Vec<i32> {
    let mut out = Vec::new();
    let bed = rgb(s.bed);
    // the bed: a fan of 64 under everything
    let n = 64;
    for k in 0..n {
        let (a, b) = (fx::TURN / n * k, fx::TURN / n * (k + 1));
        push_coloured(&mut out, V3::ZERO, polar(RADIUS, b, 0), polar(RADIUS, a, 0), bed);
    }
    let joint = cm(s.joint_cm) / 2;
    let mm = |v: i32| v * ONE / 1000;
    for (c, (r0, r1, mark)) in courses().into_iter().enumerate() {
        let c = c as i32;
        let mid = (r0 + r1) / 2;
        // stones about as long round as the course is deep (the middle
        // stone's course is cut in six)
        let round = fx::mul(mid, 411_775); // 2πr
        let width = if mark { cm(s.stone_cm) / 2 } else { cm(s.stone_cm) };
        let count = (round / width.max(1)).max(6);
        let turn0 = hash(c, 0, 7) as i32 & 0xfff;
        for k in 0..count {
            let a0 = turn0 + fx::TURN / count * k;
            let a1 = turn0 + fx::TURN / count * (k + 1);
            // the joint as an angle at this radius
            let ja =
                |r: Fx| (joint as i64 * fx::TURN as i64 / (fx::mul(r.max(ONE / 4), 411_775)) as i64) as i32;
            let (ri, ro) = (r0 + if r0 == 0 { 0 } else { joint }, r1 - joint);
            let h = |corner: i32| {
                let v = hash(c, k, corner) % 1000;
                mm(s.lift_mm[0]) + (mm(s.lift_mm[1] - s.lift_mm[0]) as i64 * v as i64 / 1000) as Fx
            };
            let tone = if mark {
                rgb(s.inner)
            } else {
                let pick = hash(c, k, 9) as usize % s.sand.len();
                rgb(s.sand[pick])
            };
            if ri == 0 {
                // the middle: a wedge to the centre
                let p = v3(0, h(4), 0);
                let q = polar(ro, a0 + ja(ro), h(0));
                let t = polar(ro, a1 - ja(ro), h(1));
                push_coloured(&mut out, p, t, q, tone);
                continue;
            }
            let p = polar(ri, a0 + ja(ri), h(0));
            let q = polar(ri, a1 - ja(ri), h(1));
            let t = polar(ro, a1 - ja(ro), h(2));
            let u = polar(ro, a0 + ja(ro), h(3));
            push_coloured(&mut out, p, q, t, tone);
            push_coloured(&mut out, p, t, u, tone);
        }
    }
    out
}

/// A band of `SEGMENTS` faces round the ring from radius `r0` to `r1`,
/// `y0` to `y1` up: its inner face, top and outer face.
const SEGMENTS: i32 = 72;

fn band(out: &mut Vec<i32>, r0: Fx, r1: Fx, y0: Fx, y1: Fx, face: Rgb, cap: Rgb) {
    for k in 0..SEGMENTS {
        let (a, b) = (fx::TURN / SEGMENTS * k, fx::TURN / SEGMENTS * (k + 1));
        let (a0, b0, a1, b1) = (polar(r0, a, 0), polar(r0, b, 0), polar(r1, a, 0), polar(r1, b, 0));
        let lo = v3(0, y0, 0);
        let up = v3(0, y1, 0);
        // inner face, seen from the middle
        push_coloured(out, a0.add(lo), b0.add(lo), b0.add(up), face);
        push_coloured(out, a0.add(lo), b0.add(up), a0.add(up), face);
        // top, seen from above
        push_coloured(out, a0.add(up), b0.add(up), b1.add(up), cap);
        push_coloured(out, a0.add(up), b1.add(up), a1.add(up), cap);
        // outer face, seen from outside
        push_coloured(out, a1.add(lo), a1.add(up), b1.add(up), face);
        push_coloured(out, a1.add(lo), b1.add(up), b1.add(lo), face);
    }
}

/// The kerb: from the floor's edge, `kerb_cm[1]` thick and `KERB_HIGH` up,
/// with a capping course a little proud of it.
pub fn kerb(s: &Spec) -> Vec<i32> {
    let mut out = Vec::new();
    let (r0, r1) = (RADIUS, RADIUS + cm(s.kerb_cm[1]));
    band(
        &mut out,
        r0,
        r1,
        0,
        KERB_HIGH - ONE / 10,
        rgb(s.kerb),
        rgb(s.kerb_cap),
    );
    band(
        &mut out,
        r0 - ONE / 25,
        r1 + ONE / 25,
        KERB_HIGH - ONE / 10,
        KERB_HIGH,
        rgb(s.kerb_cap),
        rgb(s.kerb_cap),
    );
    out
}

/// Where the wall's inner face stands.
pub fn wall_radius(s: &Spec) -> Fx {
    RADIUS + cm(s.kerb_cm[1]) + cm(s.walk_cm)
}

/// The wall `WALL_HIGH` up, with a pilaster between each pair of banners.
pub fn wall(s: &Spec) -> Vec<i32> {
    let mut out = Vec::new();
    let r0 = wall_radius(s);
    let r1 = r0 + cm(s.wall_cm[1]);
    band(
        &mut out,
        r0,
        r1,
        0,
        WALL_HIGH - ONE / 5,
        rgb(s.wall),
        rgb(s.wall_cap),
    );
    band(
        &mut out,
        r0 - ONE / 20,
        r1 + ONE / 20,
        WALL_HIGH - ONE / 5,
        WALL_HIGH,
        rgb(s.wall_cap),
        rgb(s.wall_cap),
    );
    // the pilasters: a block standing proud of the wall between banners
    let n = s.banners;
    for k in 0..n {
        let a = fx::TURN / n * k + fx::TURN / n / 2;
        let at = polar(r0 - ONE / 10, a, 0);
        block(
            &mut out,
            at,
            a,
            ONE * 35 / 100,
            WALL_HIGH + ONE / 5,
            rgb(s.pilaster),
        );
    }
    out
}

/// The stands: `tiers` steps rising behind the wall, each `tier_cm` deep
/// and high, dark, with a crowd in silhouette on them (a block a seat,
/// `crowd_pct` of the seats taken).
pub fn stands(s: &Spec) -> Vec<i32> {
    let mut out = Vec::new();
    // (a hand's width behind the wall, so no face of the stands lies on
    // the wall's back)
    let back = wall_radius(s) + cm(s.wall_cm[1]) + ONE / 10;
    let (deep, high) = (cm(s.tier_cm[0]), cm(s.tier_cm[1]));
    let dark = rgb(s.stands);
    for t in 0..s.tiers {
        let r0 = back + deep * t;
        let y = WALL_HIGH + high * t;
        // the riser (seen from the ring) and the tread
        band(&mut out, r0, r0 + deep, y - high, y, dark, dark);
        // the crowd on the tread: shoulders and a head, a seat apart
        let round = fx::mul(r0 + deep / 2, 411_775);
        let seats = round / (ONE * 7 / 10);
        for k in 0..seats {
            if (hash(t, k, 3) % 100) as i32 >= s.crowd_pct {
                continue;
            }
            let a = fx::TURN / seats * k;
            let tone = rgb(s.crowd[hash(t, k, 5) as usize % s.crowd.len()]);
            let at = polar(r0 + deep * 2 / 5, a, y);
            let tall = ONE * 55 / 100 + (hash(t, k, 6) % 15) as i32 * ONE / 100;
            block(&mut out, at, a, ONE * 22 / 100, tall, tone);
            let head = at.add(v3(0, tall, 0));
            block(&mut out, head, a, ONE / 10, ONE / 5, tone);
        }
    }
    out
}

/// An upright block at `at` (its foot), turned to face the middle, `half`
/// wide each way, `tall` high: four sides and a top.
fn block(out: &mut Vec<i32>, at: V3, a: i32, half: Fx, tall: Fx, c: Rgb) {
    let side = polar(half, a + fx::QUARTER_TURN, 0);
    let fwd = polar(half * 3 / 4, a, 0);
    let corners = [
        at.sub(side).sub(fwd),
        at.add(side).sub(fwd),
        at.add(side).add(fwd),
        at.sub(side).add(fwd),
    ];
    let up = v3(0, tall, 0);
    for e in 0..4 {
        let (m, q) = (corners[e], corners[(e + 1) % 4]);
        push_coloured(out, m, q.add(up), m.add(up), c);
        push_coloured(out, m, q, q.add(up), c);
    }
    push_coloured(out, corners[0].add(up), corners[1].add(up), corners[2].add(up), c);
    push_coloured(out, corners[0].add(up), corners[2].add(up), corners[3].add(up), c);
}

/// The ring's group for the static array, its meshes added to `meshes`.
pub fn scene(meshes: &mut MeshTable) -> Vec<Group> {
    let s = spec();
    let floor = meshes.add(floor(s));
    let kerb = meshes.add(kerb(s));
    let wall = meshes.add(wall(s));
    let stands = meshes.add(stands(s));
    let outer = wall_radius(s) + cm(s.wall_cm[1]) + cm(s.tier_cm[0]) * s.tiers;
    let mut g = Group::new("ring", V3::ZERO, outer + ONE * 12);
    for m in [floor, kerb, wall, stands] {
        g.shapes.solid(m, Affine::IDENTITY, [ONE; 3], 0);
    }
    // the banners: a pole out from the wall's top, a cloth hanging from it
    let r = wall_radius(s);
    let (bw, bh) = (cm(s.banner_cm[0]), cm(s.banner_cm[1]));
    for k in 0..s.banners {
        let a = fx::TURN / s.banners * k;
        let yaw = fx::QUARTER_TURN - a;
        let turn = Affine::rot_y(yaw);
        let top = WALL_HIGH - ONE * 3 / 10;
        let pole_at = polar(r - ONE / 10, a, top);
        g.shapes.solid(
            Mesh::Cube,
            Affine::translate(pole_at)
                .then(&turn)
                .then(&Affine::scale(v3(bw + ONE / 5, ONE / 16, ONE / 16))),
            rgb(s.pole),
            0,
        );
        let cloth_at = polar(r - ONE * 14 / 100, a, top - bh / 2);
        g.shapes.solid(
            Mesh::Cube,
            Affine::translate(cloth_at)
                .then(&turn)
                .then(&Affine::scale(v3(bw, bh, ONE / 30))),
            rgb(s.banner[k as usize % s.banner.len()]),
            0,
        );
    }
    // the gate towers at the ends of the opening line, over the wall
    let (tw, th) = (cm(s.tower_cm[0]), cm(s.tower_cm[1]));
    for end in [0, fx::HALF_TURN] {
        let a = fx::QUARTER_TURN - crate::world::START_YAW + end;
        let at = polar(r + cm(s.wall_cm[1]) / 2, a, th / 2);
        let turn = Affine::rot_y(fx::QUARTER_TURN - a);
        g.shapes.solid(
            Mesh::Cube,
            Affine::translate(at)
                .then(&turn)
                .then(&Affine::scale(v3(tw, th, tw))),
            rgb(s.tower),
            0,
        );
        g.shapes.solid(
            Mesh::Pyramid,
            Affine::translate(at.add(v3(0, th / 2, 0)))
                .then(&turn)
                .then(&Affine::scale(v3(tw + ONE / 5, tw, tw + ONE / 5))),
            rgb(s.wall_cap),
            0,
        );
    }
    vec![g]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prim::VERTEX_VALUES;

    fn tris(v: &[i32]) -> Vec<(V3, V3)> {
        v.chunks(3 * VERTEX_VALUES)
            .map(|t| {
                let p = |i: usize| v3(t[i], t[i + 1], t[i + 2]);
                let c = (p(0).add(p(VERTEX_VALUES)).add(p(2 * VERTEX_VALUES))).scale(ONE / 3);
                (c, p(3))
            })
            .collect()
    }

    /// Every stone faces up, lies inside the floor's 20 m and sits on the
    /// bed between 2 and 9 mm up (data/ring.json's lift, 3 to 8 mm, with a
    /// millimetre either way for rounding), and the bed is at 0.
    #[test]
    fn every_floor_stone_faces_up_and_lies_a_few_millimetres_over_its_bed() {
        let v = floor(spec());
        let mm = ONE / 1000;
        for (c, n) in tris(&v) {
            assert!(n.y > ONE * 98 / 100, "{n:?}");
            assert!(fx::length(c.x, c.z) <= RADIUS, "{c:?}");
            assert!(c.y == 0 || (2 * mm..=9 * mm).contains(&c.y), "{}", c.y);
        }
    }

    /// The inner ring's course straddles 6 m, in its own colour.
    #[test]
    fn a_course_of_its_own_marks_the_inner_ring_at_six_metres() {
        let marks: Vec<_> = courses().into_iter().filter(|c| c.2).collect();
        assert_eq!(marks.len(), 1);
        let (a, b, _) = marks[0];
        assert!(a < fx::int(6) && b > fx::int(6) && b - a < ONE / 2, "{a} {b}");
        let inner = rgb(spec().inner);
        let v = floor(spec());
        let near_six = v
            .chunks(3 * VERTEX_VALUES)
            .filter(|t| t[6..9] == inner[..])
            .all(|t| (fx::length(t[0], t[2]) - fx::int(6)).abs() < ONE / 3);
        assert!(near_six, "the marker's stones lie about 6 m out");
        // the courses tile the floor from the middle to the wall, no gap
        let c = courses();
        assert_eq!(c[0].0, 0);
        assert_eq!(c.last().unwrap().1, RADIUS);
        for w in c.windows(2) {
            assert_eq!(w[0].1, w[1].0);
        }
    }

    /// The kerb is 1.4 m tall at the floor's edge and the wall 4 m tall
    /// behind it; their faces wind toward where they are seen from.
    #[test]
    fn the_kerb_and_wall_stand_at_the_edge_their_faces_toward_the_eye() {
        let s = spec();
        // by hand: the kerb's cap tops out at 1.4 m, the wall's pilasters at
        // 4 m and 20 cm more
        for (v, top) in [(kerb(s), ONE * 14 / 10), (wall(s), ONE * 42 / 10)] {
            let mut highest = 0;
            for (c, n) in tris(&v) {
                highest = highest.max(c.y);
                let r = fx::length(c.x, c.z);
                assert!(r >= RADIUS - ONE / 2, "nothing stands on the floor: {r}");
                let _ = n;
            }
            // the top faces lie flat at the top (a triangle's middle, taken
            // as a third of its corners, rounds by a few units)
            assert!((highest - top).abs() <= 8, "{highest} {top}");
        }
        // the kerb's inner face: normals toward the middle
        let inward = tris(&kerb(s))
            .iter()
            .filter(|(c, n)| n.y.abs() < ONE / 10 && fx::length(c.x, c.z) < RADIUS + ONE / 10)
            .all(|(c, n)| n.dot(v3(c.x, 0, c.z).norm()) < 0);
        assert!(inward);
    }

    /// A block's four sides and top each face away from its middle.
    #[test]
    fn a_block_s_faces_point_out() {
        let mut v = Vec::new();
        let at = v3(fx::int(3), 0, fx::int(4));
        block(&mut v, at, fx::deg(33), ONE / 2, ONE, [ONE; 3]);
        let mid = at.add(v3(0, ONE / 2, 0));
        for (c, n) in tris(&v) {
            assert!(n.dot(c.sub(mid)) > 0, "{c:?} {n:?}");
        }
    }

    /// The ring is drawn with a small number of instances (PLAN.md M2,
    /// about 30 to 60): four meshes, two shapes a banner and two a tower.
    #[test]
    fn the_ring_is_one_group_of_a_few_dozen_instances() {
        let mut m = MeshTable::default();
        let g = scene(&mut m);
        assert_eq!(g.len(), 1);
        assert_eq!(m.world.len(), 4);
        let n = g[0].shapes.list.len();
        assert_eq!(n, instances());
        assert!((30..=60).contains(&n), "{n}");
    }
}
