//! M6 (CA-7, R7.101): rung F, the squire, as `ladder.py`'s `squire` draws
//! him: a padded coat quilted in bands from the chest to mid-thigh, cloth
//! sleeves and hose, a mail hood round an open face, and a steel kettle hat
//! whose wide brim names him at duel distance. The hat is his only plate
//! (`plate::covered`), and it covers the head only above its brim: below
//! it the face and the hood take a blow whole (`plate::OPEN_HELM_MM`).

use super::fed::{body_in, coat, column, start, Garb, Wear};
use super::*;
use crate::plate::OPEN_HELM_MM;

/// The squire's slots in `data/models.json`.
#[cfg(test)]
pub(super) const SLOTS: &[&str] = &[
    "main", "second", "frame", "cloth", "shade", "accent", "mail", "hose", "skin",
];

/// Where the padded coat runs, millimetres over the pelvis on the player,
/// and the heights of its quilting.
const COAT: (i32, i32) = (-250, 560);
const QUILTS: [i32; 6] = [-170, -50, 70, 190, 310, 430];
/// How far the brim reaches from the hat's middle, millimetres on the
/// player: the trait the sheet names him by.
const BRIM_MM: i32 = 330;
/// The brim's depth: from `OPEN_HELM_MM` up this far.
const BRIM_DEEP_MM: i32 = 44;

pub(super) fn model(pose: &Pose, bare: u16) -> Knight {
    let g = Garb("squire");
    let mut k = start(Kind::Squire, pose, bare, &g);
    let (cloth, hose, glove) = (g.c("cloth"), g.c("hose"), g.c("frame"));
    let wear = Wear {
        parts: [
            g.c("skin"),
            cloth,
            cloth,
            cloth,
            cloth,
            cloth,
            hose,
            hose,
            hose,
            hose,
        ],
        boot: glove,
        glove,
    };
    body_in(&mut k, Kind::Squire, &wear);
    gambeson(&mut k, &g);
    hood(&mut k, &g);
    hat(&mut k, &g);
    k.out
}

/// The padded coat: a straight column over the torso, quilted in bands,
/// belted at the waist.
fn gambeson(k: &mut K, g: &Garb) {
    let pr = k.proud();
    let (rx, rz) = coat(k, pr);
    column(k, Layer::Cloth, Role::Hip, COAT.0, COAT.1, rx, rz, g.c("cloth"));
    let band = k.s(PROUD_MM);
    for y in QUILTS {
        column(
            k,
            Layer::Cloth,
            Role::Trim,
            y - 8,
            y + 8,
            rx + band,
            rz + band,
            g.c("shade"),
        );
    }
    // the belt, between two bands
    let belt = k.s(PROUD_MM + 6);
    column(
        k,
        Layer::Cloth,
        Role::Waist,
        110,
        150,
        rx + belt,
        rz + belt,
        g.c("frame"),
    );
}

/// The mail hood over the head, its skirt over the neck and the tops of
/// the shoulders, and the face showing through it.
fn hood(k: &mut K, g: &Garb) {
    let p = k.pose;
    let pr = k.proud();
    let mail = g.c("mail");
    let hr = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
    k.ball(
        HEAD,
        Layer::Cloth,
        Role::Body,
        p.head,
        v3(hr.x + pr, hr.y + pr, hr.z + pr),
        mail,
    );
    // the hood's skirt: an eight-sided cone from the shoulders to the jaw
    let a = k.off(p.chest, 0, -10, 0);
    let b = p.head.sub(k.up.scale(k.s(140)));
    let (_, yr) = yoke(k);
    let r = fx::div(yr.x + pr, ONE * 924 / 1000);
    let rz = fx::div(yr.z + pr, ONE * 924 / 1000);
    let at = bar(a, b, (2 * r, 2 * rz), k.right);
    k.push(TORSO, Layer::Cloth, Role::Body, Mesh::Taper, at, mail);
    // the face, an oval standing out of the hood's front, and two eyes
    let face = p
        .head
        .add(k.fwd.scale(hr.z + pr - k.s(40)))
        .sub(k.up.scale(k.s(40)));
    let fr = v3(k.s(100), k.s(72), k.s(62));
    k.ball(HEAD, Layer::Cloth, Role::Body, face, fr, g.c("skin"));
    for side in [-1, 1] {
        let (pt, n) = on_ellipsoid(fr, toward(side * fx::deg(26), fx::deg(14)));
        let local = pt.add(n.scale(k.s(4)));
        let at = face
            .add(k.right.scale(local.x))
            .add(k.up.scale(local.y))
            .add(k.back.scale(local.z));
        let e = k.s(26);
        let a = put(&k.at(at), V3::ZERO, v3(e, e, e));
        k.push(HEAD, Layer::Cloth, Role::Trim, Mesh::Dot, a, g.c("frame"));
    }
}

/// The kettle hat: a round crown and a brim sloping down and out, its
/// lower edge where the open helm's plate ends.
fn hat(k: &mut K, g: &Garb) {
    if !k.plated(HEAD) {
        return;
    }
    let p = k.pose;
    let steel = g.c("main");
    let none = Affine::IDENTITY;
    // the crown, clear of the hood above the brim
    let c = p.head.add(k.up.scale(k.s(70)));
    k.ball(
        HEAD,
        Layer::Plate,
        Role::Helm,
        c,
        v3(k.s(214), k.s(150), k.s(208)),
        steel,
    );
    // the brim: a short eight-sided cone, its top 60% of its foot
    let lo = OPEN_HELM_MM;
    let hi = OPEN_HELM_MM + BRIM_DEEP_MM;
    let at = p.head.add(k.up.scale(k.s((lo + hi) / 2)));
    let r = k.s(BRIM_MM);
    k.boxed(
        HEAD,
        Layer::Plate,
        Role::Brim,
        Mesh::Taper,
        at,
        &none,
        v3(2 * r, k.s(BRIM_DEEP_MM), 2 * r),
        g.c("second"),
    );
}
