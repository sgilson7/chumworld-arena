//! M6 (CA-7): the ladder's rung C, the pit veteran (working label), as the
//! concept sheet draws him (`chumworld-arena-journal/tools/ladder.py`,
//! `veteran`): a barbute with a T-shaped face, one huge pauldron on the left
//! shoulder, bare upper arms, steel vambraces, a leather coat with a dark red
//! sash over a mail skirt, cloth hose and boots.
//!
//! He wears plate on four parts (`plate::covered`): the helm, the left upper
//! arm under the pauldron, and both forearms. Everything else is skin,
//! leather, cloth or mail, drawn in its own colour and never breaking off.
//! A plated part whose plate has broken off shows mail, as a knight's does.
//!
//! Built with `knight.rs`'s drawing state (`K`) and its sizes: millimetres
//! on the player's 1.70 m, scaled to the veteran's build; colours are the
//! `veteran` scheme in `data/models.json`, with two slots of its own
//! (`skin`, `hose`).

use super::*;

/// The colour in the `veteran` scheme's `slot`.
fn c(slot: &str) -> Rgb {
    table().slot("veteran", slot)
}

/// The veteran on `pose`, the plate off the parts in `bare`.
pub(super) fn model(pose: &Pose, bare: u16) -> Knight {
    let mut k = start(Kind::Veteran, pose, bare);
    body(&mut k);
    torso(&mut k);
    arms(&mut k);
    helm(&mut k);
    k.out
}

/// Whether part `s` wears plate now: covered and not broken off.
fn plated(k: &K, s: usize) -> bool {
    crate::plate::covered(Kind::Veteran) & bit(s) != 0 && k.bare & bit(s) == 0
}

/// A body shape on part `s` in `own` colour, or in mail if `s` wore plate
/// that has broken off.
fn skin_or_mail(k: &K, s: usize, own: Rgb) -> (Layer, Rgb) {
    if crate::plate::covered(Kind::Veteran) & bit(s) != 0 && !plated(k, s) {
        (Layer::Mail, k.sc.mail)
    } else {
        (Layer::Frame, own)
    }
}

/// A limb from `a` to `b`, left out when the two meet (an arm folded onto
/// itself in a pose draws as its joints' balls).
#[allow(clippy::too_many_arguments)]
fn limb(k: &mut K, seg: usize, layer: Layer, a: V3, b: V3, r: Fx, rgb: Rgb) {
    if a.dist(b) >= k.s(20) {
        k.limb(seg, layer, Role::Body, a, b, r, rgb);
    }
}

fn body(k: &mut K) {
    let p = k.pose;
    let (hose, boot, skin, leather) = (c("hose"), k.sc.frame, c("skin"), k.sc.cloth);
    for s in [LEFT, RIGHT] {
        let ankle = p.foot[s].add(V3::UP.scale(k.s(70)));
        let r = k.w(THIGH_R);
        k.limb(THIGH[s], Layer::Frame, Role::Body, p.hip[s], p.knee[s], r, hose);
        let kr = k.w(KNEE_R);
        k.ball(
            THIGH[s],
            Layer::Frame,
            Role::Body,
            p.knee[s],
            v3(kr, kr, kr),
            hose,
        );
        let r = k.w(SHIN_R);
        k.limb(SHIN[s], Layer::Frame, Role::Body, p.knee[s], ankle, r, hose);
        let shoe = p.foot[s].add(V3::UP.scale(k.s(44))).add(k.fwd.scale(k.s(34)));
        let fr = v3(k.w(70), k.s(52), k.s(120));
        k.ball(SHIN[s], Layer::Frame, Role::Body, shoe, fr, boot);
        // the boot's shaft, a short column over the ankle
        let rr = r + k.proud();
        let below = ankle.sub(V3::UP.scale(k.s(24)));
        let at = bar(
            p.knee[s].lerp(ankle, ONE * 72 / 100),
            below,
            (2 * rr, 2 * rr),
            k.right,
        );
        k.push(SHIN[s], Layer::Cloth, Role::Body, Mesh::Cyl, at, boot);
    }
    // the trunk in the leather coat, the neck bare
    let hips = k.off(p.pelvis, 0, 12, 0);
    let hr = v3(k.w(152), k.s(108), k.w(112));
    k.ball(TORSO, Layer::Frame, Role::Body, hips, hr, leather);
    let (bc, br) = belly(k);
    k.ball(TORSO, Layer::Frame, Role::Body, bc, br, leather);
    let (yc, yr) = yoke(k);
    k.ball(TORSO, Layer::Frame, Role::Body, yc, yr, leather);
    let chin = p.head.sub(k.up.scale(k.s(150)));
    let r = k.s(52);
    k.limb(TORSO, Layer::Frame, Role::Body, p.chest, chin, r, skin);
    for s in [LEFT, RIGHT] {
        let (lu, cu) = skin_or_mail(k, UPPER_ARM[s], skin);
        let (lf, cf) = skin_or_mail(k, FOREARM[s], k.sc.frame);
        let sr = k.w(SHOULDER_R);
        k.ball(UPPER_ARM[s], lu, Role::Body, p.shoulder[s], v3(sr, sr, sr), cu);
        let r = k.w(UPPER_ARM_R);
        limb(k, UPPER_ARM[s], lu, p.shoulder[s], p.elbow[s], r, cu);
        let er = k.w(ELBOW_R);
        k.ball(UPPER_ARM[s], lu, Role::Body, p.elbow[s], v3(er, er, er), cu);
        let r = k.w(FOREARM_R);
        limb(k, FOREARM[s], lf, p.elbow[s], p.hand[s], r, cf);
        let hr = k.w(HAND_R);
        k.ball(FOREARM[s], lf, Role::Body, p.hand[s], v3(hr, hr, hr), cf);
    }
    let (lh, ch) = skin_or_mail(k, HEAD, k.sc.frame);
    let r = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
    k.ball(HEAD, lh, Role::Body, p.head, r, ch);
}

fn torso(k: &mut K) {
    let p = k.pose;
    let sc = k.sc;
    let pr = k.proud();
    let (bc, br) = belly(k);
    // the mail skirt from the coat's hem to the middle of the thigh
    let (mx, mz) = (
        fx::div(k.w(152) + pr, COS18),
        fx::div(k.w(112).max(br.z * 9 / 10) + pr, COS18),
    );
    let at = bar(
        k.off(p.pelvis, 0, -200, 0),
        k.off(p.pelvis, 0, 40, 0),
        (2 * mx, 2 * mz),
        k.right,
    );
    k.push(TORSO, Layer::Cloth, Role::Hip, Mesh::Cyl, at, sc.mail);
    // the coat's hem over the skirt's top, so the leather laps the mail
    let at = bar(
        k.off(p.pelvis, 0, 10, 0),
        k.off(p.pelvis, 0, 90, 0),
        (2 * (mx + pr), 2 * (mz + pr)),
        k.right,
    );
    k.push(TORSO, Layer::Cloth, Role::Waist, Mesh::Cyl, at, sc.cloth);
    // the dark red sash round the belly, its tail down the left hip
    let y = 230;
    let dy = k.off(p.pelvis, 0, y, 0).sub(bc).dot(k.up);
    let sx = fx::div(across_at(br.x, br.y, dy) + pr, COS18);
    let sz = fx::div(across_at(br.z, br.y, dy) + pr, COS18);
    let at = bar(
        k.off(p.pelvis, 0, y - 30, 0),
        k.off(p.pelvis, 0, y + 30, 0),
        (2 * sx, 2 * sz),
        k.right,
    );
    k.push(TORSO, Layer::Cloth, Role::Trim, Mesh::Cyl, at, sc.accent);
    let tail_x = -(sx * 80 / 100);
    let side = fx::div(k.w(152), COS18) + 2 * pr + k.s(30);
    let at = k
        .off(p.pelvis, 0, 40, 0)
        .add(k.right.scale(tail_x.min(-side)))
        .add(k.fwd.scale(k.s(40)));
    let none = Affine::IDENTITY;
    k.boxed(
        TORSO,
        Layer::Cloth,
        Role::Trim,
        Mesh::Cube,
        at,
        &none,
        v3(k.s(18), k.s(300), k.s(70)),
        sc.accent,
    );
}

fn arms(k: &mut K) {
    let p = k.pose;
    let sc = k.sc;
    for s in [LEFT, RIGHT] {
        let fa = FOREARM[s];
        if plated(k, fa) {
            // the vambrace and a steel gauntlet
            let r = k.w(FOREARM_R);
            if p.elbow[s].dist(p.hand[s]) < k.s(20) {
                continue;
            }
            k.sleeve(
                fa,
                p.elbow[s],
                p.hand[s],
                ONE * 12 / 100,
                ONE * 78 / 100,
                r,
                sc.main,
            );
            let hr = k.w(HAND_R) + k.proud();
            k.ball(fa, Layer::Plate, Role::Body, p.hand[s], v3(hr, hr, hr), sc.second);
        }
    }
    // the one great pauldron, on the left shoulder only: a dome and three
    // lames stepping down and out past the elbow's line
    let ua = UPPER_ARM[LEFT];
    if !plated(k, ua) {
        return;
    }
    let sh = p.shoulder[LEFT];
    let dome = k.off(sh, -60, 20, 0);
    k.ball(
        ua,
        Layer::Plate,
        Role::Pauldron,
        dome,
        v3(k.w(220), k.s(140), k.w(190)),
        sc.main,
    );
    // a raised ridge over the dome, the sheet's dark stripe
    let ridge = k.off(sh, -70, 20 + 140, 0);
    let tilt = Affine::rot_z(fx::deg(20));
    k.boxed(
        ua,
        Layer::Plate,
        Role::Pauldron,
        Mesh::Cube,
        ridge,
        &tilt,
        v3(k.w(260), k.s(26), k.w(40)),
        sc.second,
    );
    for i in 0..3 {
        let at = k.off(sh, -(130 + 40 * i), -40 - 80 * i, 0);
        let tilt = Affine::rot_z(fx::deg(22 + 10 * i));
        let size = v3(k.w(440 - 20 * i), k.s(120 - 8 * i), k.w(350 - 30 * i));
        let colour = if i % 2 == 0 { sc.second } else { sc.main };
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

fn helm(k: &mut K) {
    if !plated(k, HEAD) {
        return;
    }
    let p = k.pose;
    let sc = k.sc;
    let pr = k.proud();
    let none = Affine::IDENTITY;
    let hr = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
    // the barbute: a tall round skull coming down round the jaw
    let lift = k.s(14);
    let c0 = p.head.add(k.up.scale(lift));
    let r = v3(hr.x + pr + k.s(6), hr.y + pr + k.s(30), hr.z + pr + k.s(6));
    k.ball(HEAD, Layer::Plate, Role::Helm, c0, r, sc.main);
    let (jx, jz) = (fx::div(r.x * 88 / 100, COS18), fx::div(r.z * 88 / 100, COS18));
    let (lo, hi) = (k.s(-225), k.s(-40));
    let at = bar(
        p.head.add(k.up.scale(lo)),
        p.head.add(k.up.scale(hi)),
        (2 * jx, 2 * jz),
        k.right,
    );
    k.push(HEAD, Layer::Plate, Role::Helm, Mesh::Cyl, at, sc.main);
    // how far forward the helm's face is at `y` over the head's middle
    let front = |y: Fx| across_at(r.z, r.y, y - lift).max(fx::mul(jz, COS18) * i32::from(y <= hi));
    // the T: the eyes' bar across at `plate::VISOR_MM`, then the bar down
    // over nose and mouth, dark
    let slit_y = k.s((VISOR_MM.0 + VISOR_MM.1) / 2);
    let at = p
        .head
        .add(k.up.scale(slit_y))
        .add(k.fwd.scale(front(slit_y) - k.s(4)));
    k.boxed(
        HEAD,
        Layer::Plate,
        Role::Slit,
        Mesh::Cube,
        at,
        &none,
        v3(k.s(210), k.s(44), k.s(30)),
        sc.frame,
    );
    for (y0, y1) in [(-10, -90), (-90, -170)] {
        let (a, b) = (k.s(y0), k.s(y1));
        let mid = (a + b) / 2;
        let z = front(a).max(front(b)) - k.s(6);
        let at = p.head.add(k.up.scale(mid)).add(k.fwd.scale(z));
        k.boxed(
            HEAD,
            Layer::Plate,
            Role::Trim,
            Mesh::Cube,
            at,
            &none,
            v3(k.s(56), a - b, k.s(30)),
            sc.frame,
        );
    }
}

#[cfg(test)]
mod tests;
