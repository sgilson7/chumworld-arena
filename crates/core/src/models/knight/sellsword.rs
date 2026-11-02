//! M6 (CA-7, R7.101): rung E, the sellsword, as `ladder.py`'s `sellsword`
//! draws him: a dark leather brigandine studded with brass rivets, cloth
//! sleeves and hose, steel vambraces, a sallet with a tail behind and a
//! bevor at the chin, and one big pauldron on the right shoulder only: the
//! odd shoulder names him at duel distance. Plate (`plate::covered`): the
//! head, the torso (the brigandine's plates), both forearms and the right
//! upper arm under its pauldron.

use super::fed::{arm_plate, body_in, column, cuirass, start, Garb, Wear};
use super::*;

/// The sellsword's slots in `data/models.json`.
#[cfg(test)]
pub(super) const SLOTS: &[&str] = &[
    "main", "second", "frame", "cloth", "accent", "mail", "hose", "leather",
];

/// The rivets' rows (millimetres over the pelvis) and columns (across).
const RIVET_ROWS: [i32; 4] = [90, 210, 330, 450];
const RIVET_COLS: [i32; 6] = [-200, -130, -60, 60, 130, 200];

pub(super) fn model(pose: &Pose, bare: u16) -> Knight {
    let g = Garb("sellsword");
    let mut k = start(Kind::Sellsword, pose, bare, &g);
    let (cloth, hose) = (g.c("cloth"), g.c("hose"));
    let wear = Wear {
        parts: [cloth, cloth, cloth, cloth, cloth, cloth, hose, hose, hose, hose],
        boot: g.c("frame"),
        glove: g.c("frame"),
    };
    body_in(&mut k, Kind::Sellsword, &wear);
    brigandine(&mut k, &g);
    arms(&mut k, &g);
    sallet(&mut k, &g);
    k.out
}

fn brigandine(k: &mut K, g: &Garb) {
    if !k.plated(TORSO) {
        return;
    }
    let p = k.pose;
    let pr = k.proud();
    let leather = g.c("leather");
    cuirass(k, leather);
    // its skirt over the hips
    let (bc, br) = belly(k);
    let hx = fx::div(k.w(160).max(k.w(152)) + pr, COS18);
    let hz = fx::div(k.w(122).max(k.w(112)) + pr, COS18);
    column(k, Layer::Plate, Role::Hip, -150, 60, hx, hz, leather);
    // the rivets: smooth studs on the front, standing on the plackart
    let (yc, yr) = yoke(k);
    let r = k.s(16);
    for y in RIVET_ROWS {
        let at_y = k.off(p.pelvis, 0, y, 0);
        for x in RIVET_COLS {
            let wx = k.w(x);
            // the deeper of the two balls' fronts at this height and across
            let front = |c: V3, rr: V3| {
                let dy = fx::div(at_y.sub(c).dot(k.up), rr.y.max(1));
                let dx = fx::div(wx, rr.x);
                let q = ONE - fx::mul(dx, dx) - fx::mul(dy, dy);
                if q <= 0 {
                    0
                } else {
                    fx::mul(rr.z, fx::sqrt(q))
                }
            };
            let z =
                front(bc, v3(br.x + pr, br.y, br.z + pr)).max(front(yc, v3(yr.x + pr, yr.y + pr, yr.z + pr)));
            if z == 0 {
                continue;
            }
            let at = at_y.add(k.right.scale(wx)).add(k.fwd.scale(z + r / 2));
            let a = put(&k.at(at), V3::ZERO, v3(2 * r, 2 * r, 2 * r));
            k.push(TORSO, Layer::Plate, Role::Trim, Mesh::Dot, a, g.c("accent"));
        }
    }
}

fn arms(k: &mut K, g: &Garb) {
    let p = k.pose;
    let (main, second) = (g.c("main"), g.c("second"));
    for s in [LEFT, RIGHT] {
        if k.plated(FOREARM[s]) {
            arm_plate(k, FOREARM[s], s, main, second);
        }
    }
    if !k.plated(UPPER_ARM[RIGHT]) {
        return;
    }
    let ua = UPPER_ARM[RIGHT];
    arm_plate(k, ua, RIGHT, main, second);
    // the odd pauldron: two broad lames off the right shoulder only
    for i in 0..2 {
        let at = k.off(p.shoulder[RIGHT], 70 + 26 * i, 70 - 86 * i, 0);
        let tilt = Affine::rot_z(-fx::deg(18 + 12 * i));
        let size = v3(k.w(320 - 30 * i), k.s(124 - 12 * i), k.w(270 - 30 * i));
        let colour = if i == 0 { main } else { second };
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

/// The sallet over the whole head, its tail swept back, the slit at the
/// visor gap, and the bevor over the chin.
fn sallet(k: &mut K, g: &Garb) {
    if !k.plated(HEAD) {
        return;
    }
    let p = k.pose;
    let pr = k.proud();
    let (main, dark) = (g.c("main"), g.c("frame"));
    let none = Affine::IDENTITY;
    let hr = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
    let r = v3(hr.x + pr + k.s(6), hr.y + pr + k.s(6), hr.z + pr + k.s(6));
    k.ball(HEAD, Layer::Plate, Role::Helm, p.head, r, main);
    // the tail: a flat four-sided point swept back and a little down
    let at = p.head.add(k.back.scale(r.z + k.s(50))).sub(k.up.scale(k.s(100)));
    let sweep = Affine::rot_x(fx::deg(108));
    k.boxed(
        HEAD,
        Layer::Plate,
        Role::Helm,
        Mesh::Pyramid,
        at,
        &sweep,
        v3(k.w(340), k.s(280), k.s(60)),
        main,
    );
    // the slit across the front, where `plate::gap` finds the visor
    let slit_y = k.s((VISOR_MM.0 + VISOR_MM.1) / 2);
    let slit_h = k.s(VISOR_MM.1 - VISOR_MM.0);
    let z = across_at(r.z, r.y, slit_y + k.s(8));
    let at = p.head.add(k.up.scale(slit_y)).add(k.fwd.scale(z));
    k.boxed(
        HEAD,
        Layer::Plate,
        Role::Slit,
        Mesh::Cube,
        at,
        &none,
        v3(k.s(250), slit_h, k.s(30)),
        dark,
    );
    // the bevor, a rounded plate standing out over the chin and throat
    let c = p.head.sub(k.up.scale(k.s(120))).add(k.fwd.scale(k.s(36)));
    k.ball(
        HEAD,
        Layer::Plate,
        Role::Body,
        c,
        v3(k.s(178), k.s(122), k.s(170)),
        g.c("second"),
    );
}
