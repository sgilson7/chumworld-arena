//! T11 (CA-7, R18): the campaign's six new opponents, as the concept sheet
//! draws them (`chumworld-arena-journal/tools/opponents.py`), each carrying
//! its new weapon and wearing plate only where `plate::covered` says:
//!
//! - the woodsman (axe): a leather jerkin, a fur mantle over the shoulders
//!   wider than they are, a hood; plate on the forearms (bracers);
//! - the friar (quarterstaff): a robe to the feet, a rope belt, a bare
//!   tonsured head; no plate;
//! - the warden (mace): plate above the waist, heavy pauldrons, a blue
//!   tabard and a close barbute with a slot face; cloth legs;
//! - the reiver (broadsword): a jack sewn with plates, an open steel bonnet
//!   whose brim stands well past its crown, riding boots;
//! - the pikeman (spear): an open morion with a high comb, a breastplate,
//!   tassets over the thighs and a sash across;
//! - the headsman (great axe): a black hood rising to a point, bare arms
//!   and chest, a leather apron; no plate.
//!
//! Built on the knights' builder through `fed.rs`'s helpers, each garment
//! stood off the body's shapes so no face lies on another's.

use super::fed::{arm_plate, body_in, coat, column, cuirass, start, Garb, Wear};
use super::*;
use crate::plate::OPEN_HELM_MM;

/// Each opponent's slots in `data/models.json`.
#[cfg(test)]
pub(super) fn slots(kind: Kind) -> &'static [&'static str] {
    const MOST: &[&str] = &[
        "main", "second", "frame", "cloth", "shade", "accent", "mail", "hose", "skin",
    ];
    match kind {
        Kind::Warden => &MOST[..8],
        Kind::Friar => &[
            "main", "second", "frame", "cloth", "shade", "accent", "mail", "hose", "skin", "hair",
        ],
        _ => MOST,
    }
}

/// The scheme's name for each.
pub(super) fn key(kind: Kind) -> &'static str {
    match kind {
        Kind::Woodsman => "woodsman",
        Kind::Friar => "friar",
        Kind::Warden => "warden",
        Kind::Reiver => "reiver",
        Kind::Pikeman => "pikeman",
        _ => "headsman",
    }
}

pub(super) fn model(kind: Kind, pose: &Pose, bare: u16) -> Knight {
    let g = Garb(key(kind));
    let mut k = start(kind, pose, bare, &g);
    let (cloth, hose, frame) = (g.c("cloth"), g.c("hose"), g.c("frame"));
    let skin = if kind == Kind::Warden {
        g.c("frame")
    } else {
        g.c("skin")
    };
    let (head, chest, arm) = match kind {
        // the hood's cloth, the robe, the bare chest and arms
        Kind::Woodsman => (g.c("accent"), cloth, cloth),
        Kind::Friar => (skin, cloth, cloth),
        Kind::Headsman => (g.c("main"), skin, skin),
        _ => (skin, cloth, cloth),
    };
    let wear = Wear {
        parts: [head, chest, arm, arm, arm, arm, hose, hose, hose, hose],
        boot: frame,
        glove: frame,
    };
    body_in(&mut k, kind, &wear);
    match kind {
        Kind::Woodsman => woodsman(&mut k, &g),
        Kind::Friar => friar(&mut k, &g),
        Kind::Warden => warden(&mut k, &g),
        Kind::Reiver => reiver(&mut k, &g),
        Kind::Pikeman => pikeman(&mut k, &g),
        _ => headsman(&mut k, &g),
    }
    k.out
}

/// A face standing out of the head's front: an oval and two eyes.
fn face(k: &mut K, g: &Garb, skin: Rgb) {
    let p = k.pose;
    let pr = k.proud();
    let hr = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
    let at = p
        .head
        .add(k.fwd.scale(hr.z + pr - k.s(40)))
        .sub(k.up.scale(k.s(40)));
    let fr = v3(k.s(100), k.s(72), k.s(62));
    k.ball(HEAD, Layer::Cloth, Role::Body, at, fr, skin);
    for side in [-1, 1] {
        let (pt, n) = on_ellipsoid(fr, toward(side * fx::deg(26), fx::deg(14)));
        let local = pt.add(n.scale(k.s(4)));
        let e_at = at
            .add(k.right.scale(local.x))
            .add(k.up.scale(local.y))
            .add(k.back.scale(local.z));
        let e = k.s(26);
        let a = put(&k.at(e_at), V3::ZERO, v3(e, e, e));
        k.push(HEAD, Layer::Cloth, Role::Trim, Mesh::Dot, a, g.c("frame"));
    }
}

/// A coat round the torso from `y0` to `y1` over the pelvis, `gap` off the
/// body, with a belt at `belt` in `belt_c`.
fn coat_on(k: &mut K, y0: i32, y1: i32, gap: Fx, role: Role, rgb: Rgb, belt: i32, belt_c: Rgb) {
    let (rx, rz) = coat(k, gap);
    column(k, Layer::Cloth, role, y0, y1, rx, rz, rgb);
    let b = k.s(PROUD_MM + 6);
    column(
        k,
        Layer::Cloth,
        Role::Waist,
        belt - 20,
        belt + 20,
        rx + b,
        rz + b,
        belt_c,
    );
}

/// The woodsman: jerkin, fur mantle, a hood behind the head, bracers.
fn woodsman(k: &mut K, g: &Garb) {
    let p = k.pose;
    let pr = k.proud();
    coat_on(k, -200, 540, pr, Role::Hip, g.c("cloth"), 130, g.c("frame"));
    face(k, g, g.c("skin"));
    // the mantle: a short cone over the shoulders, wider than they are
    let a = k.off(p.chest, 0, -60, 0);
    let b = k.off(p.chest, 0, 70, 0);
    let (_, yr) = yoke(k);
    let r = yr.x + k.s(170);
    let rz = yr.z + k.s(80);
    let at = bar(a, b, (2 * r, 2 * rz), k.right);
    k.push(
        TORSO,
        Layer::Cloth,
        Role::Pauldron,
        Mesh::Taper,
        at,
        g.c("second"),
    );
    // the hood's back, a cone from the nape over the head
    let hr = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
    let c = p.head.add(k.back.scale(k.s(40))).add(k.up.scale(k.s(30)));
    k.ball(
        HEAD,
        Layer::Cloth,
        Role::Helm,
        c,
        v3(hr.x + pr, hr.y + pr, hr.z),
        g.c("accent"),
    );
    for s in [LEFT, RIGHT] {
        if k.plated(FOREARM[s]) {
            arm_plate(k, FOREARM[s], s, g.c("main"), g.c("second"));
        }
    }
}

/// The friar: a robe to the feet, a rope belt, a cowl down on the
/// shoulders, a tonsure.
fn friar(k: &mut K, g: &Garb) {
    let p = k.pose;
    let pr = k.proud();
    // to the ankles: the pelvis stands about 720 mm over the soles
    coat_on(
        k,
        -640,
        500,
        pr + k.s(30),
        Role::Hem,
        g.c("cloth"),
        120,
        g.c("accent"),
    );
    // the cowl, a ring at the neck
    let a = k.off(p.chest, 0, 40, 0);
    let b = k.off(p.chest, 0, 120, 0);
    let (_, yr) = yoke(k);
    let at = bar(a, b, (2 * (yr.x + k.s(30)), 2 * (yr.z + k.s(30))), k.right);
    k.push(TORSO, Layer::Cloth, Role::Trim, Mesh::Taper, at, g.c("shade"));
    face(k, g, g.c("skin"));
    // the tonsure: a ring of hair round the crown, above the ears
    let hr = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
    let c = p.head.add(k.up.scale(k.s(60))).add(k.back.scale(k.s(10)));
    k.ball(
        HEAD,
        Layer::Cloth,
        Role::Helm,
        c,
        v3(hr.x + 4 * pr, hr.y * 6 / 10, hr.z + 4 * pr),
        g.c("hair"),
    );
}

/// The warden: cuirass, plated arms with heavy pauldrons, a blue tabard
/// over the cuirass, a close barbute with a slot face.
fn warden(k: &mut K, g: &Garb) {
    let p = k.pose;
    let pr = k.proud();
    if k.plated(TORSO) {
        cuirass(k, g.c("main"));
    }
    // the tabard stands off the cuirass
    coat_on(k, -320, 500, 3 * pr, Role::Hip, g.c("cloth"), 110, g.c("frame"));
    let device = k
        .off(p.chest, 0, -140, 0)
        .add(k.fwd.scale(coat(k, 3 * pr).1 + k.s(4)));
    k.boxed(
        TORSO,
        Layer::Cloth,
        Role::Trim,
        Mesh::Cube,
        device,
        &Affine::IDENTITY,
        v3(k.s(90), k.s(120), k.s(8)),
        g.c("accent"),
    );
    for s in [LEFT, RIGHT] {
        if k.plated(UPPER_ARM[s]) {
            arm_plate(k, UPPER_ARM[s], s, g.c("main"), g.c("second"));
            let pr2 = k.w(SHOULDER_R) + k.s(60);
            k.ball(
                UPPER_ARM[s],
                Layer::Plate,
                Role::Pauldron,
                k.off(p.shoulder[s], 0, 30, 0),
                v3(pr2, pr2 * 8 / 10, pr2),
                g.c("main"),
            );
        }
        if k.plated(FOREARM[s]) {
            arm_plate(k, FOREARM[s], s, g.c("main"), g.c("second"));
        }
    }
    if k.plated(HEAD) {
        let hr = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
        let c = p.head.add(k.up.scale(k.s(20)));
        let r = v3(hr.x + 3 * pr, hr.y + 3 * pr, hr.z + 3 * pr);
        k.ball(HEAD, Layer::Plate, Role::Helm, c, r, g.c("main"));
        let slot = c.add(k.fwd.scale(r.z - k.s(6)));
        k.boxed(
            HEAD,
            Layer::Plate,
            Role::Slit,
            Mesh::Cube,
            slot,
            &Affine::IDENTITY,
            v3(k.s(36), k.s(170), k.s(30)),
            g.c("frame"),
        );
    }
}

/// The reiver: a jack of plates, an open steel bonnet with a wide peaked
/// brim, tall riding boots.
fn reiver(k: &mut K, g: &Garb) {
    let p = k.pose;
    let pr = k.proud();
    if k.plated(TORSO) {
        cuirass(k, g.c("second"));
    }
    coat_on(k, -220, 520, 3 * pr, Role::Hip, g.c("cloth"), 130, g.c("frame"));
    // the rows of plates sewn in the jack
    let (rx, rz) = coat(k, 3 * pr);
    let band = k.s(PROUD_MM);
    for y in [-120, 10, 140, 270, 400] {
        column(
            k,
            Layer::Cloth,
            Role::Trim,
            y - 8,
            y + 8,
            rx + band,
            rz + band,
            g.c("accent"),
        );
    }
    face(k, g, g.c("skin"));
    for s in [LEFT, RIGHT] {
        let a = p.knee[s].lerp(p.foot[s], ONE * 25 / 100);
        let b = p.foot[s].add(V3::UP.scale(k.s(90)));
        let r = k.w(SHIN_R) + k.s(18);
        k.limb(SHIN[s], Layer::Cloth, Role::Trim, a, b, r, g.c("frame"));
    }
    if k.plated(HEAD) {
        let c = p.head.add(k.up.scale(k.s(90)));
        k.ball(
            HEAD,
            Layer::Plate,
            Role::Helm,
            c,
            v3(k.s(200), k.s(140), k.s(196)),
            g.c("main"),
        );
        let lo = OPEN_HELM_MM;
        let at = p.head.add(k.up.scale(k.s(lo + 24)));
        let r = k.s(340);
        k.boxed(
            HEAD,
            Layer::Plate,
            Role::Brim,
            Mesh::Taper,
            at,
            &Affine::IDENTITY,
            v3(2 * r, k.s(44), 2 * r * 8 / 10),
            g.c("second"),
        );
    }
}

/// The pikeman: breastplate, tassets, a sash across, and an open morion
/// with a brim and a high comb.
fn pikeman(k: &mut K, g: &Garb) {
    let p = k.pose;
    let pr = k.proud();
    if k.plated(TORSO) {
        cuirass(k, g.c("main"));
    }
    // the sash, from the right hip over the left shoulder, off the cuirass
    let (_, yr) = yoke(k);
    let out = yr.z + 3 * pr;
    let a = k.off(p.pelvis, 140, 60, 0).add(k.fwd.scale(out));
    let b = k.off(p.chest, -150, 90, 0).add(k.fwd.scale(out));
    let at = bar(a, b, (k.s(80), k.s(14)), k.fwd);
    k.push(TORSO, Layer::Cloth, Role::Trim, Mesh::Cube, at, g.c("cloth"));
    face(k, g, g.c("skin"));
    for s in [LEFT, RIGHT] {
        if k.plated(THIGH[s]) {
            let r = k.w(THIGH_R);
            k.sleeve(
                THIGH[s],
                p.hip[s],
                p.knee[s],
                ONE / 20,
                ONE * 70 / 100,
                r,
                g.c("main"),
            );
        }
    }
    if k.plated(HEAD) {
        let c = p.head.add(k.up.scale(k.s(90)));
        k.ball(
            HEAD,
            Layer::Plate,
            Role::Helm,
            c,
            v3(k.s(196), k.s(150), k.s(200)),
            g.c("main"),
        );
        let at = p.head.add(k.up.scale(k.s(OPEN_HELM_MM + 22)));
        let r = k.s(290);
        k.boxed(
            HEAD,
            Layer::Plate,
            Role::Brim,
            Mesh::Taper,
            at,
            &Affine::IDENTITY,
            v3(2 * r, k.s(40), 2 * r * 7 / 10),
            g.c("second"),
        );
        // the comb, front to back over the crown
        let top = c.add(k.up.scale(k.s(150)));
        k.boxed(
            HEAD,
            Layer::Plate,
            Role::Point,
            Mesh::Cube,
            top,
            &Affine::IDENTITY,
            v3(k.s(26), k.s(150), k.s(330)),
            g.c("main"),
        );
    }
}

/// The headsman: a black hood rising to a point with eye holes, bare arms
/// and chest, a leather apron and its strap.
fn headsman(k: &mut K, g: &Garb) {
    let p = k.pose;
    let pr = k.proud();
    // the apron: a slab before the belly and thighs, off the body
    let (_, br) = belly(k);
    let front = br.z + 2 * pr + k.s(20);
    let c = k.off(p.pelvis, 0, 40, 0).add(k.fwd.scale(front));
    k.boxed(
        TORSO,
        Layer::Cloth,
        Role::Hip,
        Mesh::Cube,
        c,
        &Affine::IDENTITY,
        v3(k.s(340), k.s(700), k.s(16)),
        g.c("cloth"),
    );
    // its strap round the neck
    let a = k.off(p.pelvis, 0, 380, 0).add(k.fwd.scale(front));
    let b = p.head.sub(k.up.scale(k.s(160)));
    let at = bar(a, b, (k.s(30), k.s(14)), k.fwd);
    k.push(TORSO, Layer::Cloth, Role::Trim, Mesh::Cube, at, g.c("frame"));
    // the hood's point over the head
    let hr = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
    let base = p.head.add(k.up.scale(hr.y * 5 / 10));
    let tip = p.head.add(k.up.scale(hr.y + k.s(260)));
    let at = bar(
        base,
        tip,
        (2 * (hr.x + pr) * 9 / 10, 2 * (hr.z + pr) * 9 / 10),
        k.right,
    );
    k.push(HEAD, Layer::Cloth, Role::Point, Mesh::Taper, at, g.c("main"));
    // the hood's cape over the shoulders
    let a = k.off(p.chest, 0, 20, 0);
    let b = k.off(p.chest, 0, 110, 0);
    let (_, yr) = yoke(k);
    let at = bar(a, b, (2 * (yr.x + k.s(40)), 2 * (yr.z + k.s(40))), k.right);
    k.push(TORSO, Layer::Cloth, Role::Trim, Mesh::Taper, at, g.c("second"));
    // two eye holes in the hood's front
    for side in [-1, 1] {
        let (pt, n) = on_ellipsoid(hr, toward(side * fx::deg(20), fx::deg(8)));
        let local = pt.add(n.scale(k.s(4)));
        let at = p
            .head
            .add(k.right.scale(local.x))
            .add(k.up.scale(local.y))
            .add(k.back.scale(local.z));
        let a = put(&k.at(at), V3::ZERO, v3(k.s(50), k.s(24), k.s(16)));
        k.push(HEAD, Layer::Cloth, Role::Trim, Mesh::Dot, a, g.c("skin"));
    }
}
