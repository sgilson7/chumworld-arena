//! M6 (CA-7, R7.101): rung D, the hedge knight, as `ladder.py`'s `hedge`
//! draws him: half plate (cuirass, arms, cuisses and knee cops, no greaves)
//! under a long green surcoat torn at the hem with a painted device on the
//! chest, cloth hose below the knee, a mail aventail from the shoulders to
//! the helm, and a pointed bascinet whose point names him at duel distance.
//! Plate (`plate::covered`): every part but the shins.

use super::fed::{arm_plate, body_in, coat, column, cuirass, start, Garb, Wear};
use super::*;

/// The hedge knight's slots in `data/models.json`.
#[cfg(test)]
pub(super) const SLOTS: &[&str] = &[
    "main", "second", "frame", "cloth", "shade", "accent", "mail", "hose",
];

/// The surcoat, millimetres over the pelvis on the player: its body to
/// the chest, its skirt from the hem up past the belt, wider.
const SURCOAT: (i32, i32) = (30, 470);
const SKIRT: (i32, i32) = (-230, 60);
/// How far the bascinet's point stands over the head's middle.
const POINT_MM: i32 = 380;
/// cos 22.5°: an eight-sided column's faces lie at this share of its radius.
const COS22: Fx = ONE * 924 / 1000;

pub(super) fn model(pose: &Pose, bare: u16) -> Knight {
    let g = Garb("hedge");
    let mut k = start(Kind::Hedge, pose, bare, &g);
    let (cloth, hose) = (g.c("cloth"), g.c("hose"));
    let wear = Wear {
        parts: [cloth, cloth, cloth, cloth, cloth, cloth, hose, hose, hose, hose],
        boot: g.c("frame"),
        glove: g.c("frame"),
    };
    body_in(&mut k, Kind::Hedge, &wear);
    legs(&mut k, &g);
    torso(&mut k, &g);
    arms(&mut k, &g);
    helm(&mut k, &g);
    k.out
}

/// Cuisses and knee cops; the shins are bare.
fn legs(k: &mut K, g: &Garb) {
    let p = k.pose;
    for s in [LEFT, RIGHT] {
        if !k.plated(THIGH[s]) {
            continue;
        }
        let r = k.w(THIGH_R);
        k.sleeve(
            THIGH[s],
            p.hip[s],
            p.knee[s],
            ONE / 20,
            ONE * 80 / 100,
            r,
            g.c("main"),
        );
        let kr = k.w(KNEE_R + 16) + k.proud();
        k.ball(
            THIGH[s],
            Layer::Plate,
            Role::Body,
            p.knee[s],
            v3(kr, kr, kr),
            g.c("second"),
        );
    }
}

/// The cuirass under a surcoat that hangs to mid-thigh, torn at the hem,
/// a painted device on its chest.
fn torso(k: &mut K, g: &Garb) {
    let pr = k.proud();
    let none = Affine::IDENTITY;
    if k.plated(TORSO) {
        cuirass(k, g.c("main"));
    }
    let (cloth, shade) = (g.c("cloth"), g.c("shade"));
    // over the cuirass whether it is on or not (cloth never breaks off)
    let (rx, rz) = coat(k, 2 * pr + k.s(10));
    column(k, Layer::Cloth, Role::Body, SURCOAT.0, SURCOAT.1, rx, rz, cloth);
    let flare = k.s(40);
    column(
        k,
        Layer::Cloth,
        Role::Hip,
        SKIRT.0,
        SKIRT.1,
        rx + flare,
        rz + flare,
        cloth,
    );
    // the tear: three ragged tongues hanging off the hem's front right
    let face = fx::mul(rz + flare, COS18);
    let p = k.pose;
    for (j, (x, len)) in [(40, 120), (110, 80), (180, 150)].into_iter().enumerate() {
        let tall = k.s(len);
        let at = k
            .off(p.pelvis, x, SKIRT.0, 0)
            .add(k.fwd.scale(face + k.s(20 + 2 * j as i32)))
            .sub(k.up.scale(tall / 2 - k.s(30)));
        let flip = Affine::rot_x(fx::HALF_TURN);
        k.boxed(
            TORSO,
            Layer::Cloth,
            Role::Trim,
            Mesh::Pyramid,
            at,
            &flip,
            v3(k.w(70), tall, k.s(14)),
            shade,
        );
    }
    // the painted device: a shield, its point down, on the chest
    let face = fx::mul(rz, COS18);
    let at = k.off(p.pelvis, 0, 360, 0).add(k.fwd.scale(face + k.s(12)));
    k.boxed(
        TORSO,
        Layer::Cloth,
        Role::Trim,
        Mesh::Cube,
        at,
        &none,
        v3(k.w(120), k.s(130), k.s(12)),
        g.c("accent"),
    );
    let at = k.off(p.pelvis, 0, 255, 0).add(k.fwd.scale(face + k.s(12)));
    let flip = Affine::rot_x(fx::HALF_TURN);
    k.boxed(
        TORSO,
        Layer::Cloth,
        Role::Trim,
        Mesh::Pyramid,
        at,
        &flip,
        v3(k.w(120), k.s(80), k.s(12)),
        g.c("accent"),
    );
}

fn arms(k: &mut K, g: &Garb) {
    let p = k.pose;
    let (main, second) = (g.c("main"), g.c("second"));
    for s in [LEFT, RIGHT] {
        if k.plated(UPPER_ARM[s]) {
            arm_plate(k, UPPER_ARM[s], s, main, second);
            // a rounded spaulder over the shoulder
            let r = k.w(SHOULDER_R + 30) + k.proud();
            k.ball(
                UPPER_ARM[s],
                Layer::Plate,
                Role::Pauldron,
                p.shoulder[s],
                v3(r, r * 80 / 100, r),
                main,
            );
        }
        if k.plated(FOREARM[s]) {
            arm_plate(k, FOREARM[s], s, main, second);
        }
    }
}

/// The aventail (mail, cloth: it stays) and the pointed bascinet with its
/// slit at the visor gap and a dark breath below.
fn helm(k: &mut K, g: &Garb) {
    let p = k.pose;
    let pr = k.proud();
    let none = Affine::IDENTITY;
    let hr = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
    // the aventail: an eight-sided cone from the shoulders to the jaw
    let (_, yr) = yoke(k);
    let a = k.off(p.chest, 0, -10, 0);
    let b = p.head.sub(k.up.scale(k.s(110)));
    let r = fx::div(yr.x + 2 * pr, COS22);
    let rz = fx::div(yr.z + 2 * pr, COS22);
    let at = bar(a, b, (2 * r, 2 * rz), k.right);
    k.push(HEAD, Layer::Cloth, Role::Body, Mesh::Taper, at, g.c("mail"));
    if !k.plated(HEAD) {
        return;
    }
    let main = g.c("main");
    let c = p.head.add(k.up.scale(k.s(8)));
    let r = v3(hr.x + pr + k.s(4), hr.y + pr + k.s(10), hr.z + pr + k.s(4));
    k.ball(HEAD, Layer::Plate, Role::Helm, c, r, main);
    // the point: an eight-sided cone rising from inside the dome
    let foot = k.s(150);
    let tall = k.s(POINT_MM) - foot;
    let at = p.head.add(k.up.scale(foot + tall / 2));
    let rb = k.s(125);
    k.boxed(
        HEAD,
        Layer::Plate,
        Role::Point,
        Mesh::Cone,
        at,
        &none,
        v3(2 * rb, tall, 2 * rb),
        main,
    );
    // the slit and, below it, a dark breath pointing down
    let slit_y = k.s((VISOR_MM.0 + VISOR_MM.1) / 2);
    let slit_h = k.s(VISOR_MM.1 - VISOR_MM.0);
    let dark = g.c("frame");
    let z = across_at(r.z, r.y, slit_y - k.s(8));
    let at = p.head.add(k.up.scale(slit_y)).add(k.fwd.scale(z));
    k.boxed(
        HEAD,
        Layer::Plate,
        Role::Slit,
        Mesh::Cube,
        at,
        &none,
        v3(k.s(200), slit_h, k.s(30)),
        dark,
    );
    let z = across_at(r.z, r.y, -k.s(70));
    let at = p.head.sub(k.up.scale(k.s(62))).add(k.fwd.scale(z));
    let flip = Affine::rot_x(fx::HALF_TURN);
    k.boxed(
        HEAD,
        Layer::Plate,
        Role::Trim,
        Mesh::Pyramid,
        at,
        &flip,
        v3(k.s(120), k.s(110), k.s(26)),
        dark,
    );
}
