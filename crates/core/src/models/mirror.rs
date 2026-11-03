//! M6 (CA-7): the ladder's rung S, the mirror, built as the concept sheet
//! drew it (`chumworld-arena-journal/tools/ladder.py`, `mirror`) on the
//! knights' figure and their costume helpers (`knight.rs`).
//!
//! Blackened plate with mirror-bright edges: a bright rim at each end of
//! every plate and a bright stripe down the outside of each leg, a bright
//! collar and hem, close round pauldrons with a bright crescent under each,
//! and a smooth egg-shaped helm with no slit and one bright highlight on
//! its left flank (off the front, so it never reads as an eye). Its
//! plate is nearly as dark as Knight A's, so what names it at duel distance
//! is the egg (taller than it is wide, with no flat top), the narrow
//! shoulders (no cloak, no wide lames) and the bright edges, which show
//! from every side.
//!
//! Plate on all ten parts (`plate::armoured`), as a knight; a part whose
//! plate has broken off shows the scheme's mail (`K::under`). The helm
//! draws no slit, but the visor's gap (`plate::gap`) is still there (R7.304).

use super::knight::{
    belly, yoke, Layer, Role, COS18, ELBOW_R, FOREARM_R, HAND_R, HEAD_R, K, KNEE_R, PROUD_MM, SHIN_R,
    THIGH_R, UPPER_ARM_R,
};
use super::*;
use crate::pose::{LEFT, RIGHT};
use crate::wounds::{FOREARM, HEAD, SHIN, THIGH, TORSO, UPPER_ARM};

/// How tall a bright rim is, millimetres on the player.
const RIM_MM: i32 = 40;

/// The mirror's plates over the body `knight::body` has drawn.
pub(super) fn dress(k: &mut K) {
    legs(k);
    torso(k);
    arms(k);
    helm(k);
}

/// A plate round the limb `a`–`b` (radius `r`) from share `t0` to `t1`, a
/// bright rim round each end of it standing proud of the plate.
#[allow(clippy::too_many_arguments)]
fn rimmed(k: &mut K, seg: usize, a: V3, b: V3, t0: Fx, t1: Fx, r: Fx, rims: (bool, bool)) {
    let sc = k.sc;
    k.sleeve(seg, a, b, t0, t1, r, sc.main);
    let len = a.dist(b).max(1);
    let h = fx::div(k.s(RIM_MM), len);
    // the rim's ends sit off the plate's ends, so no face is shared
    let nudge = fx::div(k.s(6), len);
    let rr = r + k.proud() + k.s(PROUD_MM);
    for (on, t) in [(rims.0, t0 - nudge), (rims.1, t1 + nudge - h)] {
        if on {
            let (p, q) = (a.lerp(b, t), a.lerp(b, t + h));
            let at = bar(p, q, (2 * rr, 2 * rr), k.right);
            k.push(seg, Layer::Plate, Role::Trim, Mesh::Cyl, at, sc.second);
        }
    }
}

/// A bright stripe down the outside (`side`) of the limb `a`–`b`, standing
/// off a plate of radius `r` (`rimmed`'s), from share `t0` to `t1`.
#[allow(clippy::too_many_arguments)]
fn stripe(k: &mut K, seg: usize, a: V3, b: V3, t0: Fx, t1: Fx, r: Fx, side: i32) {
    let u = b.sub(a).norm();
    let r0 = k.right.scale(side);
    let out0 = r0.sub(u.scale(r0.dot(u))).norm();
    let out = if out0 == V3::ZERO { r0 } else { out0 };
    let off = out.scale(r + k.proud() + k.s(8));
    let (p, q) = (a.lerp(b, t0).add(off), a.lerp(b, t1).add(off));
    let sc = k.sc;
    // a long smooth glint (`Mesh::Dot`, never sharing a face)
    let at = bar(p, q, (k.s(34), k.s(34)), out);
    k.push(seg, Layer::Plate, Role::Trim, Mesh::Dot, at, sc.second);
}

fn legs(k: &mut K) {
    let p = k.pose;
    let sc = k.sc;
    for (s, side) in [(LEFT, -1), (RIGHT, 1)] {
        let ankle = p.foot[s].add(V3::UP.scale(k.s(70)));
        if k.plated(THIGH[s]) {
            let r = k.w(THIGH_R);
            let (t0, t1) = (ONE / 20, ONE * 80 / 100);
            rimmed(k, THIGH[s], p.hip[s], p.knee[s], t0, t1, r, (false, true));
            stripe(
                k,
                THIGH[s],
                p.hip[s],
                p.knee[s],
                ONE * 20 / 100,
                ONE * 74 / 100,
                r,
                side,
            );
            let kr = k.w(KNEE_R + 10) + k.proud();
            k.ball(
                THIGH[s],
                Layer::Plate,
                Role::Body,
                p.knee[s],
                v3(kr, kr, kr),
                sc.main,
            );
        }
        if k.plated(SHIN[s]) {
            let r = k.w(SHIN_R);
            let (t0, t1) = (ONE * 15 / 100, ONE * 94 / 100);
            rimmed(k, SHIN[s], p.knee[s], ankle, t0, t1, r, (true, false));
            stripe(
                k,
                SHIN[s],
                p.knee[s],
                ankle,
                ONE * 24 / 100,
                ONE * 85 / 100,
                r,
                side,
            );
            let shoe = p.foot[s].add(V3::UP.scale(k.s(44))).add(k.fwd.scale(k.s(34)));
            let pr = k.proud();
            let fr = v3(k.w(70) + pr, k.s(52) + pr, k.s(120) + pr);
            k.ball(SHIN[s], Layer::Plate, Role::Body, shoe, fr, sc.main);
        }
    }
}

fn torso(k: &mut K) {
    if !k.plated(TORSO) {
        return;
    }
    let p = k.pose;
    let sc = k.sc;
    let pr = k.proud();
    let (bc, br) = belly(k);
    let (yc, yr) = yoke(k);
    // the cuirass: breastplate and plackart, smooth and dark
    k.ball(
        TORSO,
        Layer::Plate,
        Role::Body,
        yc,
        v3(yr.x + pr, yr.y + pr, yr.z + pr),
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
    // the gorget, dark, and a bright collar round its foot
    let neck_top = p.head.sub(k.up.scale(k.s(120)));
    let gr = k.s(108);
    k.limb(
        TORSO,
        Layer::Plate,
        Role::Body,
        p.chest.sub(k.up.scale(k.s(20))),
        neck_top,
        gr,
        sc.main,
    );
    let cr = fx::div(gr + k.s(PROUD_MM + 30), COS18);
    let (c0, c1) = (k.off(p.chest, 0, 46, 0), k.off(p.chest, 0, 46 + RIM_MM, 0));
    let at = bar(c0, c1, (2 * cr, 2 * cr * 90 / 100), k.right);
    k.push(TORSO, Layer::Plate, Role::Trim, Mesh::Cyl, at, sc.second);
    // the fauld over the hips, dark, its hem bright
    let fx_ = fx::div(br.x.max(k.w(152)) + pr, COS18);
    let fz = fx::div(br.z.max(k.w(112)) + pr, COS18);
    let a = k.off(p.pelvis, 0, -130, 0);
    let b2 = k.off(p.pelvis, 0, 70, 0);
    let at = bar(a, b2, (2 * fx_, 2 * fz), k.right);
    k.push(TORSO, Layer::Plate, Role::Hip, Mesh::Cyl, at, sc.main);
    let band = k.s(PROUD_MM + 2);
    // its foot 2 cm under the fauld's, far enough for the depth buffer
    let lo = k.off(p.pelvis, 0, -150, 0);
    let hi = k.off(p.pelvis, 0, -150 + RIM_MM, 0);
    let at = bar(lo, hi, (2 * (fx_ + band), 2 * (fz + band)), k.right);
    k.push(TORSO, Layer::Plate, Role::Trim, Mesh::Cyl, at, sc.second);
}

fn arms(k: &mut K) {
    let p = k.pose;
    let sc = k.sc;
    for (s, side) in [(LEFT, -1), (RIGHT, 1)] {
        let ua = UPPER_ARM[s];
        if k.plated(ua) {
            let r = k.w(UPPER_ARM_R);
            rimmed(
                k,
                ua,
                p.shoulder[s],
                p.elbow[s],
                ONE * 30 / 100,
                ONE * 88 / 100,
                r,
                (false, true),
            );
            let er = k.w(ELBOW_R + 6) + k.proud();
            k.ball(ua, Layer::Plate, Role::Body, p.elbow[s], v3(er, er, er), sc.main);
            // a close round pauldron, a bright crescent showing under it
            let at = k.off(p.shoulder[s], side * 14, 30, 0);
            let r = v3(k.w(108), k.s(84), k.w(108));
            k.ball(ua, Layer::Plate, Role::Pauldron, at, r, sc.main);
            let at = k.off(p.shoulder[s], side * 20, -4, 0);
            let r = v3(k.w(114), k.s(24), k.w(112));
            k.ball(ua, Layer::Plate, Role::Pauldron, at, r, sc.second);
        }
        let fa = FOREARM[s];
        if k.plated(fa) {
            let r = k.w(FOREARM_R);
            rimmed(
                k,
                fa,
                p.elbow[s],
                p.hand[s],
                ONE * 14 / 100,
                ONE * 76 / 100,
                r,
                (true, true),
            );
            let hr = k.w(HAND_R) + k.proud();
            k.ball(fa, Layer::Plate, Role::Body, p.hand[s], v3(hr, hr, hr), sc.main);
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
    let hr = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
    // the egg: a round lower half over the head, and a taller, narrower
    // upper half, smooth and with no slit
    let low = p.head.add(k.up.scale(k.s(4)));
    let lr = v3(hr.x + pr + k.s(4), hr.y + pr + k.s(10), hr.z + pr + k.s(4));
    k.ball(HEAD, Layer::Plate, Role::Helm, low, lr, sc.main);
    let top = p.head.add(k.up.scale(k.s(110)));
    let tr = v3(hr.x * 84 / 100 + pr, k.s(290), hr.z * 84 / 100 + pr);
    k.ball(HEAD, Layer::Plate, Role::Helm, top, tr, sc.main);
    // one highlight, high on the front, a little to its left
    let (q, _) = on_ellipsoid(tr, v3(-ONE * 85 / 100, ONE * 50 / 100, -ONE * 15 / 100));
    let at = top
        .add(k.right.scale(q.x))
        .add(k.up.scale(q.y))
        .add(k.back.scale(q.z))
        .add(k.fwd.scale(k.s(6)));
    let a = put(&k.at(at), V3::ZERO, v3(k.s(36), k.s(120), k.s(26)));
    k.push(HEAD, Layer::Plate, Role::Trim, Mesh::Dot, a, sc.second);
}

#[cfg(test)]
mod tests;
