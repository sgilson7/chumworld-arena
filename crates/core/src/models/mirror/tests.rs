//! The mirror's costume tests (CA-7, Ape Race's costume flow), on the
//! champion's helpers (`champion/tests.rs`: the duel between the two).

use super::super::champion::tests::{
    axes, dressed_in_every_pose, extent, faces_fight, poses, scheme_clear_of_highlights, span, TRIANGLE_CAP,
};
use super::super::knight::model;
use super::*;
use crate::pose::Kind;

/// Its plate is nearly as dark as Knight A's, so the shape and the edges
/// carry it. In every pose: its helm is an egg, at least 1.3 times as tall
/// as it is wide (A's bucket and B's round helm are under 1.2) and it
/// draws no slit; its pauldrons span at most 85% of Knight A's on the same
/// pose; it wears no cloth (A's cloak); and each of its ten parts carries
/// a bright edge in the scheme's `second`, which stands at least 150 over
/// `main` in brightness (the mean of the three channels).
#[test]
fn the_mirror_keeps_its_silhouette_in_every_pose() {
    let sc = super::super::knight::scheme(Kind::Mirror);
    let lum = |c: Rgb| (c[0] as i64 + c[1] as i64 + c[2] as i64) * 255 / (3 * ONE as i64);
    assert!(
        lum(sc.second) - lum(sc.main) >= 150,
        "{} {}",
        lum(sc.second),
        lum(sc.main)
    );
    let all = poses(Kind::Mirror);
    assert!(all.len() > 80, "{} poses", all.len());
    for (n, p) in all.iter().enumerate() {
        let k = model(Kind::Mirror, p, 0);
        let a = model(Kind::KnightA, p, 0);
        let (right, up) = axes(p);
        let (tall, wide) = (span(&k, Role::Helm, up), span(&k, Role::Helm, right));
        assert!(tall * 10 >= wide * 13, "pose {n}: helm {tall} tall, {wide} wide");
        assert!(k.parts.iter().all(|t| t.role != Role::Slit), "pose {n}: a slit");
        let (mine, a_s) = (span(&k, Role::Pauldron, right), span(&a, Role::Pauldron, right));
        assert!(
            mine * 100 <= a_s * 85,
            "pose {n}: pauldrons {mine} against A's {a_s}"
        );
        assert!(k.parts.iter().all(|t| t.layer != Layer::Cloth), "pose {n}: cloth");
        for s in 0..crate::blade::PARTS {
            assert!(
                k.parts
                    .iter()
                    .any(|t| t.seg as usize == s && t.layer == Layer::Plate && t.part.rgb == sc.second),
                "pose {n}: part {s} has no bright edge"
            );
        }
    }
    // the egg's top stands over A's flat bucket at rest
    let p = &all[0];
    let (_, up) = axes(p);
    let a = model(Kind::KnightA, p, 0);
    let k = model(Kind::Mirror, p, 0);
    assert!(extent(&k, Role::Helm, up).1 > extent(&a, Role::Helm, up).1);
}

#[test]
fn no_face_of_the_mirror_shares_a_plane_with_another_colour() {
    let (scanned, bad) = faces_fight(Kind::Mirror);
    assert!(scanned > 25, "{scanned}");
    assert!(bad.is_empty(), "{} fights:\n{}", bad.len(), bad.join("\n"));
}

#[test]
fn the_mirror_is_under_the_triangle_cap() {
    let p = rest_pose(Kind::Mirror, V3::ZERO, 0);
    let n = model(Kind::Mirror, &p, 0).model().triangles();
    assert!(n <= TRIANGLE_CAP, "{n} triangles");
    assert!(n > 1_000, "{n} triangles is no costume");
}

#[test]
fn the_mirror_is_dressed_in_every_pose() {
    dressed_in_every_pose(Kind::Mirror);
}

#[test]
fn the_mirror_s_scheme_stands_clear_of_the_highlight_colours() {
    scheme_clear_of_highlights("mirror");
}
