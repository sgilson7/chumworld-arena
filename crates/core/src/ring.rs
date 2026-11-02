//! The stone ring, drawn: what never moves (M0, plain shapes for now; M2
//! builds the real ring, CA-Q11).
//!
//! The floor is a flat disc of flagstones out to `world::RADIUS`, in rings
//! of `COURSES` and sectors of `SECTORS`, alternating two stone colours; the
//! wall stands on its edge, `WALL_HIGH` tall and `WALL_THICK` thick, with a
//! paler cap. Each is one mesh with a colour per vertex (`MeshTable::add`),
//! so the whole ring is three instances. Colours are `data/models.json`'s.

use crate::fx::{self, Fx, ONE};
use crate::geom::{v3, Affine, V3};
use crate::look::Rgb;
use crate::prim::{push_coloured, MeshTable};
use crate::render::Group;
use crate::world::RADIUS;

/// Sectors round the ring, and the radii its rings of flagstones end at.
pub const SECTORS: i32 = 48;
pub const COURSES: [Fx; 4] = [ONE * 3, ONE * 8, ONE * 14, RADIUS];
/// The wall: how tall and how thick.
pub const WALL_HIGH: Fx = ONE * 12 / 10;
pub const WALL_THICK: Fx = ONE * 6 / 10;

fn at(r: Fx, k: i32) -> V3 {
    let a = fx::TURN / SECTORS * k;
    v3(fx::mul(r, fx::cos(a)), 0, fx::mul(r, fx::sin(a)))
}

/// The floor's triangles, wound to face up.
pub fn floor(stone: [Rgb; 2]) -> Vec<i32> {
    let mut out = Vec::new();
    let mut inner = 0;
    for (c, &outer) in COURSES.iter().enumerate() {
        for k in 0..SECTORS {
            let rgb = stone[(c + k as usize) % 2];
            let (a, b) = (at(outer, k), at(outer, k + 1));
            if inner == 0 {
                push_coloured(&mut out, V3::ZERO, b, a, rgb);
            } else {
                let (p, q) = (at(inner, k), at(inner, k + 1));
                push_coloured(&mut out, p, q, b, rgb);
                push_coloured(&mut out, p, b, a, rgb);
            }
        }
        inner = outer;
    }
    out
}

/// The wall's triangles: its inner face toward the middle, its top, its
/// outer face.
pub fn wall(face: Rgb, cap: Rgb) -> Vec<i32> {
    let mut out = Vec::new();
    let up = v3(0, WALL_HIGH, 0);
    let (r0, r1) = (RADIUS, RADIUS + WALL_THICK);
    for k in 0..SECTORS {
        let (a0, b0) = (at(r0, k), at(r0, k + 1));
        let (a1, b1) = (at(r1, k), at(r1, k + 1));
        // inner face, seen from the middle
        push_coloured(&mut out, a0, b0, b0.add(up), face);
        push_coloured(&mut out, a0, b0.add(up), a0.add(up), face);
        // top, seen from above
        push_coloured(&mut out, a0.add(up), b0.add(up), b1.add(up), cap);
        push_coloured(&mut out, a0.add(up), b1.add(up), a1.add(up), cap);
        // outer face, seen from outside
        push_coloured(&mut out, a1, a1.add(up), b1.add(up), face);
        push_coloured(&mut out, a1, b1.add(up), b1, face);
    }
    out
}

/// The ring's group for the static array, its meshes added to `meshes`.
pub fn scene(meshes: &mut MeshTable) -> Vec<Group> {
    let t = crate::models::table();
    let floor = meshes.add(floor([t.c("ring_stone"), t.c("ring_stone_dark")]));
    let wall = meshes.add(wall(t.c("ring_wall"), t.c("ring_cap")));
    let mut g = Group::new("ring", V3::ZERO, RADIUS + WALL_THICK + ONE);
    g.shapes.solid(floor, Affine::IDENTITY, [ONE; 3], 0);
    g.shapes.solid(wall, Affine::IDENTITY, [ONE; 3], 0);
    vec![g]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prim::VERTEX_VALUES;

    fn normals(v: &[i32]) -> Vec<(V3, V3)> {
        v.chunks(3 * VERTEX_VALUES)
            .map(|t| {
                let p = |i: usize| v3(t[i], t[i + 1], t[i + 2]);
                let c = (p(0).add(p(VERTEX_VALUES)).add(p(2 * VERTEX_VALUES))).scale(ONE / 3);
                (c, p(3))
            })
            .collect()
    }

    #[test]
    fn every_floor_stone_faces_up_and_lies_on_the_ground() {
        let t = crate::models::table();
        let v = floor([t.c("ring_stone"), t.c("ring_stone_dark")]);
        // one fan of 48 at the middle, then two triangles a sector for each
        // of the three rings out from it
        assert_eq!(v.len() / (3 * VERTEX_VALUES), 48 + 3 * 2 * 48);
        for (c, n) in normals(&v) {
            assert!(n.y > ONE * 99 / 100, "{n:?}");
            assert_eq!(c.y, 0);
        }
    }

    /// The wall's faces wind toward where they are seen from: the inner face
    /// toward the middle, the top up, the outer face away.
    #[test]
    fn the_wall_s_faces_wind_toward_where_they_are_seen_from() {
        let v = wall([ONE; 3], [ONE; 3]);
        let (mut inward, mut up, mut outward) = (0, 0, 0);
        for (c, n) in normals(&v) {
            let out = v3(c.x, 0, c.z).norm();
            if n.y > ONE / 2 {
                up += 1;
                assert!((c.y - WALL_HIGH).abs() <= 2, "{}", c.y);
            } else if n.dot(out) < 0 {
                inward += 1;
            } else {
                outward += 1;
            }
        }
        assert_eq!((inward, up, outward), (96, 96, 96));
    }

    #[test]
    fn the_scene_is_one_group_of_two_meshes_after_the_primitives() {
        let mut m = MeshTable::default();
        let g = scene(&mut m);
        assert_eq!(g.len(), 1);
        assert_eq!(m.world.len(), 2);
        assert_eq!(g[0].shapes.list.len(), 2);
    }
}
