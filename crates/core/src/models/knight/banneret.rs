//! M6 (CA-7): the ladder's rung B, the banneret (working label), as the
//! concept sheet draws him (`chumworld-arena-journal/tools/ladder.py`,
//! `banneret`): fluted plate from head to foot, a rounded armet with a
//! comb and a plume, a blue tabard with a gilt band, gilt knee cops, and a
//! dark blue cape behind.
//!
//! He wears plate on all ten parts (`plate::covered`). The body under it is
//! `knight.rs`'s own (`body`), in the frame colour, mail where a plate has
//! broken off.
//!
//! **What moves.** A model is a function of the pose alone, so cloth cannot
//! carry speed from one tick to the next. The cape and the plume instead
//! hang by the pose:
//!
//! - the cape's top half lies on the back, in the torso's frame; its lower
//!   half is two panels, one behind each leg, hanging by the world's down
//!   from the waist and swung back just far enough to pass behind that
//!   leg's hip, knee and shin (`CAPE_CLEAR_MM` clear of them), so a stride
//!   or a lunge lifts the cape behind the leg rather than passing the thigh
//!   through it (R7.204);
//! - the plume rises from the comb in the helm's frame, then its two outer
//!   lengths bend toward the world's down, so it droops whichever way the
//!   head leans. It is part of the helm's plate and goes with it.

use super::*;

/// How far each lower panel of the cape keeps behind its leg's surface,
/// millimetres on the player.
pub const CAPE_CLEAR_MM: i32 = 30;
/// The cape's lower panels: how long, from the waist; how wide each.
pub const CAPE_DROP_MM: i32 = 560;
const CAPE_HALF_MM: i32 = 300;
/// The least the lower panels swing back, as a tangent (about 6°).
const CAPE_REST_TAN: Fx = ONE / 10;
/// The most (about 72°): past it a leg is kicked up behind and the cape
/// lies on it.
const CAPE_MOST_TAN: Fx = ONE * 3;

/// The colour in the `banneret` scheme's `slot`.
fn c(slot: &str) -> Rgb {
    table().slot("banneret", slot)
}

/// The banneret on `pose`, the plate off the parts in `bare`.
pub(super) fn model(pose: &Pose, bare: u16) -> Knight {
    let mut k = start(Kind::Banneret, pose, bare);
    body(&mut k);
    legs(&mut k);
    torso(&mut k);
    cape(&mut k);
    arms(&mut k);
    helm(&mut k);
    k.out
}

/// `v` with its share along the unit `u` taken out, made unit; `or` when
/// nothing is left.
fn square(v: V3, u: V3, or: V3) -> V3 {
    let s = v.sub(u.scale(v.dot(u))).norm();
    if s == V3::ZERO {
        or
    } else {
        s
    }
}

/// Three flutes down the front of the plate sleeve on `a`–`b` (radius `r`
/// under the plate) from share `t0` to `t1`: thin ribs standing out of the
/// sleeve's faces, in the second tone.
#[allow(clippy::too_many_arguments)]
fn flutes(k: &mut K, seg: usize, a: V3, b: V3, t0: Fx, t1: Fx, r: Fx) {
    let u = b.sub(a).norm();
    if u == V3::ZERO {
        return;
    }
    let front = square(k.fwd, u, square(k.up, u, k.right));
    let side = u.cross(front).norm();
    let out = fx::mul(r + k.proud(), COS18) + k.s(4);
    let t = k.s(16);
    let rgb = k.sc.second;
    for deg in [-40, 0, 40] {
        let d = front
            .scale(fx::cos(fx::deg(deg)))
            .add(side.scale(fx::sin(fx::deg(deg))));
        let (p, q) = (a.lerp(b, t0).add(d.scale(out)), a.lerp(b, t1).add(d.scale(out)));
        let at = bar(p, q, (t, t), side);
        k.push(seg, Layer::Plate, Role::Trim, Mesh::Cube, at, rgb);
    }
}

fn legs(k: &mut K) {
    let p = k.pose;
    let sc = k.sc;
    for s in [LEFT, RIGHT] {
        let ankle = p.foot[s].add(V3::UP.scale(k.s(70)));
        if k.plated(THIGH[s]) {
            let r = k.w(THIGH_R);
            let (t0, t1) = (ONE / 20, ONE * 80 / 100);
            k.sleeve(THIGH[s], p.hip[s], p.knee[s], t0, t1, r, sc.main);
            flutes(k, THIGH[s], p.hip[s], p.knee[s], ONE / 10, ONE * 76 / 100, r);
            // the knee cop, gilt
            let kr = k.w(KNEE_R + 16) + k.proud();
            k.ball(
                THIGH[s],
                Layer::Plate,
                Role::Body,
                p.knee[s],
                v3(kr, kr, kr),
                sc.accent,
            );
        }
        if k.plated(SHIN[s]) {
            let r = k.w(SHIN_R);
            let (t0, t1) = (ONE * 15 / 100, ONE * 94 / 100);
            k.sleeve(SHIN[s], p.knee[s], ankle, t0, t1, r, sc.main);
            flutes(k, SHIN[s], p.knee[s], ankle, ONE * 22 / 100, ONE * 88 / 100, r);
            let shoe = p.foot[s].add(V3::UP.scale(k.s(44))).add(k.fwd.scale(k.s(34)));
            let pr = k.proud();
            let fr = v3(k.w(70) + pr, k.s(52) + pr, k.s(120) + pr);
            k.ball(SHIN[s], Layer::Plate, Role::Body, shoe, fr, sc.main);
        }
    }
}

/// How far the tabard and the cape stand in front of and behind the
/// cuirass's middle.
fn cloth_depth(k: &K) -> Fx {
    let (_, br) = belly(k);
    let (_, yr) = yoke(k);
    yr.z.max(br.z) + k.proud() + k.s(30)
}

fn torso(k: &mut K) {
    let p = k.pose;
    let sc = k.sc;
    let pr = k.proud();
    let none = Affine::IDENTITY;
    let (bc, br) = belly(k);
    let (yc, yr) = yoke(k);
    // the tabard in front, from the chest to below the belt, cloth: it
    // stays when the cuirass under it breaks
    let depth = cloth_depth(k);
    let top = k.off(p.chest, 0, 40, 0);
    let bottom = k.off(p.pelvis, 0, -90, 0);
    let mid = top.lerp(bottom, ONE / 2).add(k.fwd.scale(depth));
    let tall = top.dist(bottom);
    k.boxed(
        TORSO,
        Layer::Cloth,
        Role::Body,
        Mesh::Cube,
        mid,
        &none,
        v3(k.w(270), tall, k.s(16)),
        sc.cloth,
    );
    // the gilt band across it, the house's mark
    let band = k.off(p.pelvis, 0, 230, 0).add(k.fwd.scale(depth + k.s(10)));
    k.boxed(
        TORSO,
        Layer::Cloth,
        Role::Trim,
        Mesh::Cube,
        band,
        &none,
        v3(k.w(294), k.s(80), k.s(14)),
        sc.accent,
    );
    if !k.plated(TORSO) {
        return;
    }
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
    // the gorget
    let neck_top = p.head.sub(k.up.scale(k.s(120)));
    let gr = k.s(112);
    let a = p.chest.sub(k.up.scale(k.s(20)));
    k.limb(TORSO, Layer::Plate, Role::Body, a, neck_top, gr, sc.second);
    // the fauld below the tabard's hem, round the hips
    let fx_ = fx::div(k.w(152) + pr, COS18);
    let fz = fx::div(k.w(112) + pr, COS18);
    let at = bar(
        k.off(p.pelvis, 0, -110, 0),
        k.off(p.pelvis, 0, 70, 0),
        (2 * fx_, 2 * fz),
        k.right,
    );
    k.push(TORSO, Layer::Plate, Role::Hip, Mesh::Cyl, at, sc.second);
}

/// The cape: its top on the back in the torso's frame, its lower half two
/// panels hanging by the world's down, each swung back behind its leg.
fn cape(k: &mut K) {
    let p = k.pose;
    let rgb = c("cape");
    let depth = cloth_depth(k) + k.s(16);
    let top = k.off(p.chest, 0, 90, 0).add(k.back.scale(depth));
    let waist = k.off(p.pelvis, 0, 130, 0).add(k.back.scale(depth));
    let thick = k.s(18);
    // the top half narrows to the shoulders: a flattened taper
    let at = bar(waist, top, (k.w(2 * CAPE_HALF_MM), thick), k.right);
    k.push(TORSO, Layer::Cloth, Role::Cape, Mesh::Taper, at, rgb);
    // the collar where it hangs from the pauldrons
    let collar = k.off(p.chest, 0, 0, 0).add(k.back.scale(depth - k.s(20)));
    let r = k.s(36);
    let ends = (
        collar.sub(k.right.scale(k.w(CAPE_HALF_MM) * 65 / 100)),
        collar.add(k.right.scale(k.w(CAPE_HALF_MM) * 65 / 100)),
    );
    let at = bar(ends.0, ends.1, (2 * r, 2 * r), k.back);
    k.push(TORSO, Layer::Cloth, Role::Cape, Mesh::Cyl, at, rgb);
    let flat = v3(k.back.x, 0, k.back.z).norm();
    let bh = if flat == V3::ZERO { k.back } else { flat };
    for (s, side) in [(LEFT, -1), (RIGHT, 1)] {
        let (hem, a) = cape_panel(k, s, side, waist, bh);
        let at = bar(hem, a, (k.w(CAPE_HALF_MM), thick), k.right);
        k.push(TORSO, Layer::Cloth, Role::Cape, Mesh::Cube, at, rgb);
    }
}

/// One lower panel of the cape on the `side` of leg `s`: where its hem's
/// middle is, and its top's.
pub(super) fn cape_panel(k: &K, s: usize, side: i32, waist: V3, bh: V3) -> (V3, V3) {
    let p = k.pose;
    let a = waist
        .add(k.right.scale(side * k.w(CAPE_HALF_MM) / 2))
        .add(V3::UP.scale(k.s(30)));
    let drop = k.s(CAPE_DROP_MM);
    let ankle = p.foot[s].add(V3::UP.scale(k.s(70)));
    // the leg's points the panel must pass behind, and their radii
    let mut points = vec![
        (p.hip[s], k.w(THIGH_R)),
        (p.hip[s].lerp(p.knee[s], ONE / 2), k.w(THIGH_R)),
        (p.knee[s], k.w(KNEE_R + 16)),
    ];
    for j in 1..=4 {
        points.push((p.knee[s].lerp(ankle, ONE * j / 4), k.w(SHIN_R)));
    }
    let clear = k.proud() + k.s(CAPE_CLEAR_MM) + k.s(9);
    let mut tan = CAPE_REST_TAN;
    for (q, r) in points {
        let below = a.sub(q).dot(V3::UP);
        if below <= k.s(40) || below > drop {
            continue;
        }
        // the panel must pass `r + clear` behind the point, measured square
        // to the panel: below·sin θ − ahead·cos θ ≥ m, that is tan θ ≥
        // (ahead + m·√(1 + tan²θ)) / below, found by four rounds from the
        // rest (each round only raises it)
        let (ahead, m) = (q.sub(a).dot(bh), r + clear);
        let mut need = tan;
        for _ in 0..4 {
            let sec = fx::sqrt(ONE + fx::mul(need, need).min(ONE * 16));
            need = need.max(fx::div(ahead + fx::mul(m, sec), below));
        }
        tan = tan.max(need);
    }
    let tan = tan.min(CAPE_MOST_TAN);
    let cos = fx::div(ONE, fx::sqrt(ONE + fx::mul(tan, tan)));
    let sin = fx::mul(tan, cos);
    let hem = a
        .sub(V3::UP.scale(fx::mul(drop, cos)))
        .add(bh.scale(fx::mul(drop, sin)));
    (hem, a)
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
            let er = k.w(ELBOW_R + 8) + k.proud();
            k.ball(
                ua,
                Layer::Plate,
                Role::Body,
                p.elbow[s],
                v3(er, er, er),
                sc.second,
            );
            // a round pauldron and one lame under it
            let at = k.off(p.shoulder[s], side * 30, 30, 0);
            let r = v3(k.w(120), k.s(80), k.w(116));
            k.ball(ua, Layer::Plate, Role::Pauldron, at, r, sc.main);
            let at = k.off(p.shoulder[s], side * 70, -50, 0);
            let tilt = Affine::rot_z(-side * fx::deg(24));
            let size = v3(k.w(210), k.s(90), k.w(220));
            k.boxed(
                ua,
                Layer::Plate,
                Role::Pauldron,
                Mesh::HalfCyl,
                at,
                &tilt,
                size,
                sc.second,
            );
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
            flutes(k, fa, p.elbow[s], p.hand[s], ONE * 20 / 100, ONE * 70 / 100, r);
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
    let none = Affine::IDENTITY;
    let hr = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
    // the armet: a round skull and a jaw closing under the chin
    let lift = k.s(8);
    let c0 = p.head.add(k.up.scale(lift));
    let r = v3(hr.x + pr + k.s(4), hr.y + pr + k.s(14), hr.z + pr + k.s(4));
    k.ball(HEAD, Layer::Plate, Role::Helm, c0, r, sc.main);
    let (jx, jz) = (fx::div(r.x * 86 / 100, COS18), fx::div(r.z * 92 / 100, COS18));
    let at = bar(
        p.head.add(k.up.scale(k.s(-150))),
        p.head.add(k.up.scale(k.s(-50))),
        (2 * jx, 2 * jz),
        k.right,
    );
    k.push(HEAD, Layer::Plate, Role::Helm, Mesh::Cyl, at, sc.main);
    // the eye slit at `plate::VISOR_MM`, dark across the front
    let slit_y = k.s((VISOR_MM.0 + VISOR_MM.1) / 2);
    let z = across_at(r.z, r.y, slit_y - lift);
    let at = p.head.add(k.up.scale(slit_y)).add(k.fwd.scale(z));
    let slit_h = k.s(VISOR_MM.1 - VISOR_MM.0);
    k.boxed(
        HEAD,
        Layer::Plate,
        Role::Slit,
        Mesh::Cube,
        at,
        &none,
        v3(k.s(200), slit_h, k.s(28)),
        sc.frame,
    );
    // the comb along the crown, front to back
    let crown = p.head.add(k.up.scale(lift + r.y));
    k.boxed(
        HEAD,
        Layer::Plate,
        Role::Trim,
        Mesh::Cube,
        crown,
        &none,
        v3(k.s(26), k.s(40), 2 * r.z * 70 / 100),
        sc.second,
    );
    // the plume: up from the comb's back in the helm's frame, then back and
    // down by the world's
    let down = V3::UP.neg();
    let p0 = crown.add(k.back.scale(r.z * 40 / 100));
    let p1 = p0.add(k.up.scale(k.s(170))).add(k.back.scale(k.s(80)));
    let p2 = p1.add(k.back.scale(k.s(170))).add(down.scale(k.s(10)));
    let p3 = p2.add(k.back.scale(k.s(70))).add(down.scale(k.s(190)));
    let plume = c("plume");
    for (a, b, t) in [(p0, p1, 120), (p1, p2, 150), (p2, p3, 120)] {
        let at = bar(
            a.sub(b.sub(a).scale(ONE / 6)),
            b.add(b.sub(a).scale(ONE / 6)),
            (k.w(t), k.s(t) * 8 / 10),
            k.right,
        );
        k.push(HEAD, Layer::Plate, Role::Plume, Mesh::Facet, at, plume);
    }
}

#[cfg(test)]
mod tests;
