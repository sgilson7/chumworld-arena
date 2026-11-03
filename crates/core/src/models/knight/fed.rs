//! M6 (CA-7, R7.101): rungs F, E and D of the ladder, the squire, the
//! sellsword and the hedge knight, as the concept sheet drew them
//! (`chumworld-arena-journal/tools/ladder.py`), each in its own file and
//! built on the knights' builder (`K`, `body`'s shapes, `PROUD_MM`).
//!
//! What differs from the knights: each wears plate only on the parts
//! `plate::covered` names. A part with no plate by design is drawn in its
//! garment (padding, cloth, mail treated as cloth: R7.102) on the
//! `Layer::Cloth`, which never breaks off, and the bit `bare` holds for it
//! is ignored; a covered part whose plate has broken off shows the
//! scheme's mail, as a knight's does. Colours are the opponent's scheme in
//! `data/models.json`.

use super::*;

/// The model of a ladder opponent of `kind`, or `None` for any other kind.
pub(super) fn model(kind: Kind, pose: &Pose, bare: u16) -> Option<Knight> {
    match kind {
        Kind::Squire => Some(super::squire::model(pose, bare)),
        Kind::Sellsword => Some(super::sellsword::model(pose, bare)),
        Kind::Hedge => Some(super::hedge::model(pose, bare)),
        // T11 (R18): the campaign's new opponents
        Kind::Woodsman | Kind::Friar | Kind::Warden | Kind::Reiver | Kind::Pikeman | Kind::Headsman => {
            Some(super::campaign::model(kind, pose, bare))
        }
        _ => None,
    }
}

/// The scheme's name in `data/models.json` and its slots (for the tests).
#[cfg(test)]
pub(super) fn scheme_of(kind: Kind) -> Option<(&'static str, &'static [&'static str])> {
    match kind {
        Kind::Squire => Some(("squire", super::squire::SLOTS)),
        Kind::Sellsword => Some(("sellsword", super::sellsword::SLOTS)),
        Kind::Hedge => Some(("hedge", super::hedge::SLOTS)),
        Kind::Woodsman | Kind::Friar | Kind::Warden | Kind::Reiver | Kind::Pikeman | Kind::Headsman => {
            Some((super::campaign::key(kind), super::campaign::slots(kind)))
        }
        _ => None,
    }
}

/// An opponent's colours, by slot.
pub(super) struct Garb(pub &'static str);

impl Garb {
    pub(super) fn c(&self, slot: &str) -> Rgb {
        table().slot(self.0, slot)
    }
}

/// The builder for `kind` on `pose`, its plate off the parts in `bare` and
/// never on a part `plate::covered` leaves out.
pub(super) fn start(kind: Kind, pose: &Pose, bare: u16, g: &Garb) -> K {
    let t = table();
    let b = build(kind);
    let up0 = pose.chest.sub(pose.pelvis).norm();
    let up = if up0 == V3::ZERO { V3::UP } else { up0 };
    let back = super::body_back(pose.facing, up);
    let right = up.cross(back).norm();
    let sc = Scheme {
        main: g.c("main"),
        second: g.c("second"),
        frame: g.c("frame"),
        cloth: g.c("cloth"),
        accent: g.c("accent"),
        glow: g.c("accent"),
        mail: g.c("mail"),
        scorch: g.c("frame"),
    };
    K {
        out: Knight::default(),
        b,
        ink: t.ink_figure,
        right,
        up,
        back,
        fwd: back.neg(),
        pose: *pose,
        sc,
        bare: bare | !crate::plate::covered(kind),
    }
}

/// What a body wears where it has no plate by design: each part's
/// garment (`blade::body`'s index), the boots and the gloves.
pub(super) struct Wear {
    pub parts: [Rgb; crate::blade::PARTS],
    pub boot: Rgb,
    pub glove: Rgb,
}

/// The colour and layer of part `seg`'s body: the frame under plate, mail
/// where the plate broke off, the garment where there was never plate.
fn dress(k: &K, cover: u16, seg: usize, garment: Rgb) -> (Layer, Rgb) {
    if cover & bit(seg) != 0 {
        k.under(seg)
    } else {
        (Layer::Cloth, garment)
    }
}

/// The figure's own shapes (`body`'s), each in what it wears.
pub(super) fn body_in(k: &mut K, kind: Kind, wear: &Wear) {
    let cover = crate::plate::covered(kind);
    let p = k.pose;
    for s in [LEFT, RIGHT] {
        let (lt, ct) = dress(k, cover, THIGH[s], wear.parts[THIGH[s]]);
        let (ls, cs) = dress(k, cover, SHIN[s], wear.parts[SHIN[s]]);
        let (lb, cb) = dress(k, cover, SHIN[s], wear.boot);
        let ankle = p.foot[s].add(V3::UP.scale(k.s(70)));
        let r = k.w(THIGH_R);
        k.limb(THIGH[s], lt, Role::Body, p.hip[s], p.knee[s], r, ct);
        let kr = v3(k.w(KNEE_R), k.w(KNEE_R), k.w(KNEE_R));
        k.ball(THIGH[s], lt, Role::Body, p.knee[s], kr, ct);
        let r = k.w(SHIN_R);
        k.limb(SHIN[s], ls, Role::Body, p.knee[s], ankle, r, cs);
        let shoe = p.foot[s].add(V3::UP.scale(k.s(44))).add(k.fwd.scale(k.s(34)));
        let fr = v3(k.w(70), k.s(52), k.s(120));
        k.ball(SHIN[s], lb, Role::Body, shoe, fr, cb);
    }
    let (l, c) = dress(k, cover, TORSO, wear.parts[TORSO]);
    let hips = k.off(p.pelvis, 0, 12, 0);
    let hr = v3(k.w(152), k.s(108), k.w(112));
    k.ball(TORSO, l, Role::Body, hips, hr, c);
    let (bc, br) = belly(k);
    k.ball(TORSO, l, Role::Body, bc, br, c);
    let (yc, yr) = yoke(k);
    k.ball(TORSO, l, Role::Body, yc, yr, c);
    let chin = p.head.sub(k.up.scale(k.s(150)));
    let r = k.s(52);
    k.limb(TORSO, l, Role::Body, p.chest, chin, r, c);
    for s in [LEFT, RIGHT] {
        let (lu, cu) = dress(k, cover, UPPER_ARM[s], wear.parts[UPPER_ARM[s]]);
        let (lf, cf) = dress(k, cover, FOREARM[s], wear.parts[FOREARM[s]]);
        let (lg, cg) = dress(k, cover, FOREARM[s], wear.glove);
        let sr = k.w(SHOULDER_R);
        k.ball(UPPER_ARM[s], lu, Role::Body, p.shoulder[s], v3(sr, sr, sr), cu);
        let r = k.w(UPPER_ARM_R);
        k.limb(UPPER_ARM[s], lu, Role::Body, p.shoulder[s], p.elbow[s], r, cu);
        let er = k.w(ELBOW_R);
        k.ball(UPPER_ARM[s], lu, Role::Body, p.elbow[s], v3(er, er, er), cu);
        let r = k.w(FOREARM_R);
        k.limb(FOREARM[s], lf, Role::Body, p.elbow[s], p.hand[s], r, cf);
        let hr = k.w(HAND_R);
        k.ball(FOREARM[s], lg, Role::Body, p.hand[s], v3(hr, hr, hr), cg);
    }
    let (lh, ch) = dress(k, cover, HEAD, wear.parts[HEAD]);
    let r = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
    k.ball(HEAD, lh, Role::Body, p.head, r, ch);
}

/// A ten-sided column round the torso's line from `y0` to `y1` millimetres
/// over the pelvis, radii `rx` across and `rz` front to back.
#[allow(clippy::too_many_arguments)]
pub(super) fn column(k: &mut K, layer: Layer, role: Role, y0: i32, y1: i32, rx: Fx, rz: Fx, rgb: Rgb) {
    let p = k.pose;
    let a = k.off(p.pelvis, 0, y0, 0);
    let b = k.off(p.pelvis, 0, y1, 0);
    let at = bar(a, b, (2 * rx, 2 * rz), k.right);
    k.push(TORSO, layer, role, Mesh::Cyl, at, rgb);
}

/// The radii of a ten-sided column whose faces stand `gap` off the wider of
/// the yoke and the belly, plus `out` on top of whatever is laid on them.
pub(super) fn coat(k: &K, gap: Fx) -> (Fx, Fx) {
    let (_, br) = belly(k);
    let (_, yr) = yoke(k);
    (
        fx::div(br.x.max(yr.x) + gap, COS18),
        fx::div(br.z.max(yr.z) + gap, COS18),
    )
}

/// The cuirass the knights wear (a breastplate over the yoke, a plackart
/// over the belly), `PROUD_MM` off the body, in `rgb`.
pub(super) fn cuirass(k: &mut K, rgb: Rgb) {
    let pr = k.proud();
    let (bc, br) = belly(k);
    let (yc, yr) = yoke(k);
    k.ball(
        TORSO,
        Layer::Plate,
        Role::Body,
        yc,
        v3(yr.x + pr, yr.y + pr, yr.z + pr),
        rgb,
    );
    k.ball(
        TORSO,
        Layer::Plate,
        Role::Body,
        bc,
        v3(br.x + pr, br.y, br.z + pr),
        rgb,
    );
}

/// A plated arm part as the knights wear it: a sleeve and the cop at its
/// lower joint (the elbow) or the gauntlet (the hand).
pub(super) fn arm_plate(k: &mut K, seg: usize, s: usize, main: Rgb, second: Rgb) {
    let p = k.pose;
    if seg == UPPER_ARM[s] {
        let r = k.w(UPPER_ARM_R);
        k.sleeve(
            seg,
            p.shoulder[s],
            p.elbow[s],
            ONE * 30 / 100,
            ONE * 88 / 100,
            r,
            main,
        );
        let er = k.w(ELBOW_R + 8) + k.proud();
        k.ball(seg, Layer::Plate, Role::Body, p.elbow[s], v3(er, er, er), second);
    } else {
        let r = k.w(FOREARM_R);
        k.sleeve(
            seg,
            p.elbow[s],
            p.hand[s],
            ONE * 14 / 100,
            ONE * 76 / 100,
            r,
            main,
        );
        let hr = k.w(HAND_R) + k.proud();
        k.ball(seg, Layer::Plate, Role::Body, p.hand[s], v3(hr, hr, hr), second);
    }
}

#[cfg(test)]
mod tests;
