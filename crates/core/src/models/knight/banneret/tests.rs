//! The banneret's costume tests (CA-7, M6): his silhouette in every pose,
//! a cape that never passes through his legs, no plate face in another
//! colour's plane, the triangle cap, and his colours.

use super::super::ladder_poses::{axes, extent, fights, nearest_highlight, poses, TRIANGLE_CAP};
use super::super::model as knight_model;
use super::*;

const KIND: Kind = Kind::Banneret;

/// He reads at duel distance by the plume over a round helm and a cape
/// to the knee: in every pose the plume rises 12 cm or more over the
/// helm's top; standing, the cape's hem hangs no higher than the knees.
#[test]
fn the_banneret_keeps_his_plume_and_cape_in_every_pose() {
    let all = poses(KIND);
    assert!(all.len() > 90, "{} poses", all.len());
    for (n, p) in all.iter().enumerate() {
        let k = knight_model(KIND, p, 0);
        let (_, up) = axes(p);
        let (_, plume) = extent(&k, Role::Plume, up);
        let (_, helm) = extent(&k, Role::Helm, up);
        assert!(
            plume - helm >= ONE * 12 / 100,
            "pose {n}: plume {plume}, helm {helm}"
        );
    }
    let p = rest_pose(KIND, V3::ZERO, 0);
    let k = knight_model(KIND, &p, 0);
    let (hem, _) = extent(&k, Role::Cape, V3::UP);
    let knee = p.knee[LEFT].y.max(p.knee[RIGHT].y);
    assert!(hem <= knee, "hem {hem}, knee {knee}");
}

/// Standing, with one leg swung back from the hip by `deg` degrees and the
/// shin bent 30° further back: a long stride's or a lunge's back leg.
fn stride(s: usize, deg: i32) -> Pose {
    let mut p = rest_pose(KIND, V3::ZERO, 0);
    let back = facing(p.facing, 0).neg();
    let down = V3::UP.neg();
    let way = |d: i32| {
        down.scale(fx::cos(fx::deg(d)))
            .add(back.scale(fx::sin(fx::deg(d))))
    };
    let (thigh, shin) = (p.hip[s].dist(p.knee[s]), p.knee[s].dist(p.foot[s]));
    p.knee[s] = p.hip[s].add(way(deg).scale(thigh));
    p.foot[s] = p.knee[s].add(way(deg + 30).scale(shin));
    p
}

/// No point of either leg's surface, from the hip to the ankle, lies inside
/// a flat panel of the cape, in any pose the duel gives him or with either
/// leg swung back 15° to 60°: where a leg point is within a panel's length
/// and breadth, it is clear of the panel's face by the leg's radius
/// (R7.204).
#[test]
fn the_banneret_s_cape_never_passes_through_his_legs() {
    let b = crate::pose::build(KIND);
    let sc = |v: i32| (v as i64 * ONE as i64 * b.height as i64 / (1700 * ONE as i64)) as Fx;
    let wd = |v: i32| sc(v) * b.girth_pct / 100 * crate::pose::BODY_GIRTH_PCT / 100;
    let mut panels_met = 0;
    let mut all = poses(KIND);
    for s in [LEFT, RIGHT] {
        for deg in [15, 30, 45, 60] {
            all.push(stride(s, deg));
        }
    }
    for (n, p) in all.iter().enumerate() {
        let k = knight_model(KIND, p, 0);
        for s in [LEFT, RIGHT] {
            let ankle = p.foot[s].add(V3::UP.scale(sc(70)));
            let mut legs = Vec::new();
            for j in 0..=8 {
                legs.push((p.hip[s].lerp(p.knee[s], ONE * j / 8), wd(76)));
                legs.push((p.knee[s].lerp(ankle, ONE * j / 8), wd(63)));
            }
            for t in k
                .parts
                .iter()
                .filter(|t| t.role == Role::Cape && t.part.mesh == Mesh::Cube)
            {
                let a = t.part.at;
                let (wide, long, thick) = (a.c0.len(), a.c1.len(), a.c2.len());
                let (ru, uu, nu) = (a.c0.norm(), a.c1.norm(), a.c2.norm());
                panels_met += 1;
                for (q, r) in &legs {
                    let d = q.sub(a.t);
                    if d.dot(uu).abs() * 2 > long || d.dot(ru).abs() * 2 > wide {
                        continue;
                    }
                    let off = d.dot(nu).abs();
                    assert!(
                        off >= thick / 2 + r,
                        "pose {n}: the {} leg is {off} from a cape panel, inside its {} + {r}",
                        if s == LEFT { "left" } else { "right" },
                        thick / 2
                    );
                }
            }
        }
    }
    assert!(panels_met > 300, "{panels_met}");
}

/// Standing, the cape's lower panels hang at their rest: within 10° of the
/// world's down.
#[test]
fn the_banneret_s_cape_hangs_straight_when_he_stands() {
    let p = rest_pose(KIND, V3::ZERO, 0);
    let k = knight_model(KIND, &p, 0);
    let panels: Vec<&Tagged> = k
        .parts
        .iter()
        .filter(|t| t.role == Role::Cape && t.part.mesh == Mesh::Cube)
        .collect();
    assert_eq!(panels.len(), 2);
    for t in &panels {
        let along = t.part.at.c1.norm();
        // cos 10° = 0.985
        assert!(along.dot(V3::UP) >= ONE * 985 / 1000, "{}", along.dot(V3::UP));
    }
}

#[test]
fn no_face_of_the_banneret_shares_a_plane_with_another_colour() {
    let (bad, scanned) = fights(KIND);
    assert!(scanned > 25, "{scanned}");
    assert!(bad.is_empty(), "{} fights:\n{}", bad.len(), bad.join("\n"));
}

#[test]
fn the_banneret_is_under_the_triangle_cap() {
    let p = rest_pose(KIND, V3::ZERO, 0);
    let n = knight_model(KIND, &p, 0).model().triangles();
    assert!(n <= TRIANGLE_CAP, "{n} triangles");
    assert!(n > 1_000, "{n} triangles is no costume");
}

/// Every part wears plate; a part whose plate is gone shows mail and no
/// plate, and the plume goes with the helm.
#[test]
fn every_part_of_the_banneret_is_plated_and_shows_mail_when_bare() {
    let sc = scheme(KIND);
    let p = rest_pose(KIND, V3::ZERO, 0);
    let whole = knight_model(KIND, &p, 0);
    for s in 0..crate::blade::PARTS {
        assert!(
            whole
                .parts
                .iter()
                .any(|t| t.seg as usize == s && t.layer == Layer::Plate),
            "part {s}"
        );
        let k = knight_model(KIND, &p, bit(s));
        let on: Vec<&Tagged> = k.parts.iter().filter(|t| t.seg as usize == s).collect();
        assert!(
            on.iter().all(|t| t.layer != Layer::Plate),
            "part {s} still plated"
        );
        assert!(
            on.iter().any(|t| t.layer == Layer::Mail && t.part.rgb == sc.mail),
            "part {s}"
        );
    }
    assert!(!knight_model(KIND, &p, bit(HEAD))
        .parts
        .iter()
        .any(|t| t.role == Role::Plume));
}

#[test]
fn the_banneret_s_slit_is_where_the_visor_gap_is() {
    let p = rest_pose(KIND, V3::ZERO, 0);
    let k = knight_model(KIND, &p, 0);
    let (_, up) = axes(&p);
    let slit = k.parts.iter().find(|t| t.role == Role::Slit).unwrap();
    let y = slit.part.at.t.sub(p.head).dot(up);
    let (lo, hi) = (
        crate::plate::scaled(KIND, VISOR_MM.0),
        crate::plate::scaled(KIND, VISOR_MM.1),
    );
    assert!(lo <= y && y <= hi, "slit at {y}, gap {lo}..{hi}");
    assert!(slit.part.at.t.sub(p.head).dot(facing(p.facing, 0)) > ONE / 10);
}

/// Every slot of his scheme stands 100 or more from the duel's highlights
/// (R5.8; the steel, the gilt and the plume moved to stand clear, R7.205).
#[test]
fn the_banneret_s_colours_stand_clear_of_the_highlights() {
    let (d2, what) = nearest_highlight("banneret");
    assert!(d2 >= 100 * 100, "sqrt({d2}): {what}");
}
