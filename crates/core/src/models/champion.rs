//! M6 (CA-7): the ladder's rung A, the champion, built as the concept sheet
//! drew it (`chumworld-arena-journal/tools/ladder.py`, `champion`) on the
//! knights' figure and their costume helpers (`knight.rs`).
//!
//! Broad tourney plate in pale steel with gilt: a flat-topped great helm
//! with a gilt cross on its face and two gilt wings swept up and out over
//! it, huge domed pauldrons with gilt lames under them, a breastplate with
//! a gilt ridge, a gilt belt over a red cloth skirt, gilt knee and elbow
//! cops and gilt gauntlets. The silhouette that names it at duel distance:
//! the widest shoulders on the ladder under a helm with wings.
//!
//! Plate on all ten parts (`plate::armoured`), as a knight; a part whose
//! plate has broken off shows the scheme's mail (`K::under`).

use super::knight::{
    across_at, belly, yoke, Layer, Role, COS18, ELBOW_R, FOREARM_R, HAND_R, HEAD_R, K, KNEE_R, PROUD_MM,
    SHIN_R, THIGH_R, UPPER_ARM_R,
};
use super::*;
use crate::plate::VISOR_MM;
use crate::pose::{LEFT, RIGHT};
use crate::wounds::{FOREARM, HEAD, SHIN, THIGH, TORSO, UPPER_ARM};

/// The champion's plates over the body `knight::body` has drawn.
pub(super) fn dress(k: &mut K) {
    legs(k);
    torso(k);
    arms(k);
    helm(k);
}

fn legs(k: &mut K) {
    let p = k.pose;
    let sc = k.sc;
    for s in [LEFT, RIGHT] {
        let ankle = p.foot[s].add(V3::UP.scale(k.s(70)));
        if k.plated(THIGH[s]) {
            let r = k.w(THIGH_R);
            k.sleeve(
                THIGH[s],
                p.hip[s],
                p.knee[s],
                ONE / 20,
                ONE * 80 / 100,
                r,
                sc.main,
            );
            // a gilt knee cop, broad
            let kr = k.w(KNEE_R + 22) + k.proud();
            k.ball(
                THIGH[s],
                Layer::Plate,
                Role::Body,
                p.knee[s],
                v3(kr, kr, kr),
                sc.second,
            );
        }
        if k.plated(SHIN[s]) {
            let r = k.w(SHIN_R);
            k.sleeve(
                SHIN[s],
                p.knee[s],
                ankle,
                ONE * 15 / 100,
                ONE * 94 / 100,
                r,
                sc.main,
            );
            let shoe = p.foot[s].add(V3::UP.scale(k.s(44))).add(k.fwd.scale(k.s(34)));
            let pr = k.proud();
            let fr = v3(k.w(70) + pr, k.s(52) + pr, k.s(120) + pr);
            k.ball(SHIN[s], Layer::Plate, Role::Body, shoe, fr, sc.main);
        }
    }
}

fn torso(k: &mut K) {
    let p = k.pose;
    let sc = k.sc;
    let pr = k.proud();
    let none = Affine::IDENTITY;
    let (bc, br) = belly(k);
    let (yc, yr) = yoke(k);
    // how wide the plackart is at a height `y` over the pelvis
    let plackart_at = |k: &K, y: i32| {
        let dy = k.off(p.pelvis, 0, y, 0).sub(bc).dot(k.up);
        (across_at(br.x + pr, br.y, dy), across_at(br.z + pr, br.y, dy))
    };
    // the red skirt, cloth, in two tiers, the lower wider: it never breaks off
    let (px, pz) = plackart_at(k, 40);
    let sx = fx::div(px.max(k.w(152)) + pr, COS18);
    let sz = fx::div(pz.max(k.w(112)) + pr, COS18);
    let tiers = [(-60, 40, 0), (-250, -50, 34)];
    for (bottom, top, grow) in tiers {
        let a = k.off(p.pelvis, 0, bottom, 0);
        let b2 = k.off(p.pelvis, 0, top, 0);
        let (rx, rz) = (sx + k.w(grow), sz + k.w(grow * 3 / 4));
        let at = bar(a, b2, (2 * rx, 2 * rz), k.right);
        k.push(TORSO, Layer::Cloth, Role::Hip, Mesh::Cyl, at, sc.cloth);
    }
    if !k.plated(TORSO) {
        return;
    }
    // the cuirass, broad: a breastplate over the yoke, a plackart over the belly
    let wide = k.w(24);
    k.ball(
        TORSO,
        Layer::Plate,
        Role::Body,
        yc,
        v3(yr.x + pr + wide, yr.y + pr, yr.z + pr),
        sc.main,
    );
    k.ball(
        TORSO,
        Layer::Plate,
        Role::Body,
        bc,
        v3(br.x + pr, br.y, br.z + pr),
        sc.main,
    );
    // the gilt ridge down the breastplate
    let front = yc.add(k.fwd.scale(yr.z + pr));
    k.boxed(
        TORSO,
        Layer::Plate,
        Role::Trim,
        Mesh::Cube,
        front,
        &none,
        v3(k.s(40), k.s(220), k.s(30)),
        sc.second,
    );
    // the gorget, gilt
    let neck_top = p.head.sub(k.up.scale(k.s(120)));
    let gr = k.s(112);
    k.limb(
        TORSO,
        Layer::Plate,
        Role::Body,
        p.chest.sub(k.up.scale(k.s(20))),
        neck_top,
        gr,
        sc.second,
    );
    // the gilt belt over the plackart and the skirt's top
    let (px, pz) = plackart_at(k, 70);
    let bx = fx::div(px.max(sx) + pr, COS18);
    let bz = fx::div(pz.max(sz) + pr, COS18);
    let a = k.off(p.pelvis, 0, 30, 0);
    let b2 = k.off(p.pelvis, 0, 74, 0);
    let at = bar(a, b2, (2 * bx, 2 * bz), k.right);
    k.push(TORSO, Layer::Plate, Role::Waist, Mesh::Cyl, at, sc.second);
}

fn arms(k: &mut K) {
    let p = k.pose;
    let sc = k.sc;
    for (s, side) in [(LEFT, -1), (RIGHT, 1)] {
        let ua = UPPER_ARM[s];
        if k.plated(ua) {
            let r = k.w(UPPER_ARM_R);
            k.sleeve(
                ua,
                p.shoulder[s],
                p.elbow[s],
                ONE * 30 / 100,
                ONE * 88 / 100,
                r,
                sc.main,
            );
            // the elbow cop turned a little, so its facets never lie in the
            // planes of the other elbow's ball when the hands meet
            let er = k.w(ELBOW_R + 10) + k.proud();
            let turn = Affine::rot_y(fx::deg(13));
            k.boxed(
                ua,
                Layer::Plate,
                Role::Body,
                Mesh::Facet,
                p.elbow[s],
                &turn,
                v3(2 * er, 2 * er, 2 * er),
                sc.second,
            );
            // the great pauldron: a dome over the shoulder, rising to the neck
            let at = k.off(p.shoulder[s], side * 80, 52, 0);
            let r = v3(k.w(205), k.s(124), k.w(160));
            k.ball(ua, Layer::Plate, Role::Pauldron, at, r, sc.main);
            // two lames under it, gilt then steel, each further out and down
            for i in 0..2 {
                let at = k.off(p.shoulder[s], side * (130 + 30 * i), -40 - 64 * i, 0);
                let tilt = Affine::rot_z(-side * fx::deg(26 + 10 * i));
                let size = v3(k.w(250 - 30 * i), k.s(84 - 8 * i), k.w(250 - 30 * i));
                let colour = if i == 0 { sc.second } else { sc.main };
                k.boxed(
                    ua,
                    Layer::Plate,
                    Role::Pauldron,
                    Mesh::HalfCyl,
                    at,
                    &tilt,
                    size,
                    colour,
                );
            }
        }
        let fa = FOREARM[s];
        if k.plated(fa) {
            let r = k.w(FOREARM_R);
            k.sleeve(
                fa,
                p.elbow[s],
                p.hand[s],
                ONE * 14 / 100,
                ONE * 76 / 100,
                r,
                sc.main,
            );
            let hr = k.w(HAND_R + 8) + k.proud();
            k.ball(fa, Layer::Plate, Role::Body, p.hand[s], v3(hr, hr, hr), sc.second);
        }
    }
}

fn helm(k: &mut K) {
    if !k.plated(HEAD) {
        return;
    }
    let p = k.pose;
    let sc = k.sc;
    let pr = k.proud();
    let none = Affine::IDENTITY;
    let hr = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
    // the great helm: a flat-topped column, longer than the head, resting
    // low on the gorget
    let rx = fx::div(hr.x + pr + k.s(6), COS18);
    let rz = fx::div(hr.z + pr + k.s(6), COS18);
    let (lo, hi) = (k.s(-226), k.s(236));
    let at = bar(
        p.head.add(k.up.scale(lo)),
        p.head.add(k.up.scale(hi)),
        (2 * rx, 2 * rz),
        k.right,
    );
    k.push(HEAD, Layer::Plate, Role::Helm, Mesh::Cyl, at, sc.main);
    // a gilt band round its crown's edge
    let band = k.s(PROUD_MM + 2);
    let at = bar(
        p.head.add(k.up.scale(hi - k.s(46))),
        p.head.add(k.up.scale(hi - k.s(10))),
        (2 * (rx + band), 2 * (rz + band)),
        k.right,
    );
    k.push(HEAD, Layer::Plate, Role::Trim, Mesh::Cyl, at, sc.second);
    // the slit at `plate::VISOR_MM`, the gap the rule tests for, and a gilt
    // bar down the face under it: a cross
    let face = fx::mul(rz, COS18);
    let slit_y = k.s((VISOR_MM.0 + VISOR_MM.1) / 2);
    let slit_h = k.s(VISOR_MM.1 - VISOR_MM.0);
    let at = p.head.add(k.up.scale(slit_y)).add(k.fwd.scale(face));
    k.boxed(
        HEAD,
        Layer::Plate,
        Role::Slit,
        Mesh::Cube,
        at,
        &none,
        v3(k.s(150), slit_h, k.s(26)),
        sc.frame,
    );
    let (top, bottom) = (k.s(VISOR_MM.0) - k.s(4), k.s(-236));
    let at = p.head.add(k.up.scale((top + bottom) / 2)).add(k.fwd.scale(face));
    k.boxed(
        HEAD,
        Layer::Plate,
        Role::Trim,
        Mesh::Cube,
        at,
        &none,
        v3(k.s(36), top - bottom, k.s(30)),
        sc.second,
    );
    // the wings: on each side two gilt feathers swept up and out from the
    // helm's top corners, the long one steeper
    for side in [-1, 1] {
        for (len, deg, y, wide) in [(340, 34, 150, 170), (230, 60, 90, 140)] {
            let base = p
                .head
                .add(k.up.scale(k.s(y)))
                .add(k.right.scale(side * (rx - k.s(20))));
            // the feather's axis, out and up from its base
            let a = fx::deg(deg);
            let out = k.right.scale(side * fx::sin(a)).add(k.up.scale(fx::cos(a)));
            let at = base.add(out.scale(k.s(len) / 2));
            let turn = Affine::rot_z(-side * a);
            k.boxed(
                HEAD,
                Layer::Plate,
                Role::Wing,
                Mesh::Pyramid,
                at,
                &turn,
                v3(k.s(wide), k.s(len), k.s(26)),
                sc.second,
            );
        }
    }
}

#[cfg(test)]
pub(super) mod tests;
