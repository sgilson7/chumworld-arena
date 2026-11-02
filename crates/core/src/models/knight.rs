//! M3 (CA-7, CA-Q13: A): the two knights, built as the concept sheet drew
//! them (`chumworld-arena-journal/tools/knights.py`), on CHUMWORLD's figure.
//!
//! Ape Race's costume flow: the body is the figure's own shapes (limbs
//! between the joints, a ball on each joint, the torso from pelvis to chest,
//! the head at `pose.head`), every one recoloured to the scheme's dark
//! `frame`; plates are laid over it, each standing at least `PROUD_MM`
//! proud of what is under it, so the frame shows only at the joints. A part
//! whose plate has broken off (`plate.rs`) draws its body in `mail` and its
//! plates not at all.
//!
//! - Knight A, the black tower: blackened plate, a flat-topped bucket helm
//!   with an eye slit, a breath slot and a small ridge, three-lame
//!   pauldrons wider than the hips, a gorget, a cuirass, a fauld, cuisses,
//!   knee cops, greaves and sabatons, a torn dark cloak behind and a tail of
//!   cloth in front. An inverted triangle.
//! - Knight B, the ivory crown: pale plate with dull gilt, a rounded helm
//!   under a crown of spikes, eye slit and breath slot, a fur mantle over
//!   round pauldrons, a cuirass with a gilt ridge, a gilt belt, a skirt of
//!   four lames widening to the knee, its lowest scorched and caught with
//!   embers, a tabard on the back. An hourglass that ends in points.
//!
//! Every shape is tagged with the part of the body it is on (`blade::body`'s
//! index, as `wounds.rs` counts parts), its layer, and its role in the
//! silhouette, so the wounds can tint, sever and shed by part and the tests
//! can measure the outline. Sizes are millimetres on the player's 1.70 m,
//! scaled to the knights' build (`pose::build`) as `figure.rs` does;
//! colours are the scheme's slots in `data/models.json`.

use super::*;
use crate::geom::facing;
use crate::plate::VISOR_MM;
use crate::pose::{build, Build, LEFT, RIGHT};
use crate::wounds::{bit, FOREARM, HEAD, SHIN, THIGH, TORSO, UPPER_ARM};

/// How far a plate stands off what is under it, millimetres on the player:
/// over Ape Race's 5 mm, and over the 9.7 mm the depth buffer needs to tell
/// two faces apart out to `FIGURE_FAR` (`decal_lift_needed`).
pub const PROUD_MM: i32 = 12;

/// What a shape is in the costume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    /// the body, in the dark frame colour, showing at the joints
    Frame,
    /// the body of a part whose plate is gone
    Mail,
    Plate,
    /// cloak, tabard, fur: never breaks off
    Cloth,
}

/// What a shape does for the silhouette (the tests measure by these).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Body,
    Pauldron,
    /// the widest thing at the hips: A's fauld
    Hip,
    Helm,
    Crown,
    Slit,
    /// B's lowest lame
    Hem,
    /// B's belt, where the skirt starts
    Waist,
    Trim,
}

/// One shape of a knight and what it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tagged {
    pub part: Part,
    /// the body part it is on (`blade::body`'s index)
    pub seg: u8,
    pub layer: Layer,
    pub role: Role,
}

/// A knight's shapes in draw order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Knight {
    pub parts: Vec<Tagged>,
}

impl Knight {
    pub fn model(&self) -> Model {
        Model {
            parts: self.parts.iter().map(|t| t.part).collect(),
            decals: Vec::new(),
        }
    }

    /// The body part each shape is on, in order (the wounds' labels).
    pub fn segs(&self) -> Vec<u8> {
        self.parts.iter().map(|t| t.seg).collect()
    }
}

/// A knight's colours (`data/models.json`, `schemes`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scheme {
    pub main: Rgb,
    pub second: Rgb,
    pub frame: Rgb,
    pub cloth: Rgb,
    pub accent: Rgb,
    pub glow: Rgb,
    pub mail: Rgb,
    pub scorch: Rgb,
}

/// The slots a scheme has, as `data/models.json` names them.
pub const SLOTS: [&str; 8] = [
    "main", "second", "frame", "cloth", "accent", "glow", "mail", "scorch",
];

/// The scheme's name in `data/models.json`.
pub fn scheme_name(kind: Kind) -> &'static str {
    if kind == Kind::KnightB {
        "knight_b"
    } else {
        "knight_a"
    }
}

pub fn scheme(kind: Kind) -> Scheme {
    let t = table();
    let n = scheme_name(kind);
    let c = |s: &str| t.slot(n, s);
    Scheme {
        main: c("main"),
        second: c("second"),
        frame: c("frame"),
        cloth: c("cloth"),
        accent: c("accent"),
        glow: c("glow"),
        mail: c("mail"),
        scorch: c("scorch"),
    }
}

/// Limb and joint radii, millimetres on the player (`figure.rs`'s).
const THIGH_R: i32 = 76;
const SHIN_R: i32 = 63;
const UPPER_ARM_R: i32 = 56;
const FOREARM_R: i32 = 52;
const KNEE_R: i32 = 70;
const ELBOW_R: i32 = 57;
const HAND_R: i32 = 64;
const SHOULDER_R: i32 = 62;
const HEAD_R: (i32, i32, i32) = (190, 205, 182);
const PLAYER_HEIGHT: i64 = 1700;
/// A ten-sided column's faces lie at cos 18° of its radius.
const COS18: Fx = ONE * 951 / 1000;

struct K {
    out: Knight,
    b: Build,
    ink: Fx,
    right: V3,
    up: V3,
    back: V3,
    fwd: V3,
    pose: Pose,
    sc: Scheme,
    bare: u16,
}

impl K {
    fn s(&self, v: i32) -> Fx {
        (v as i64 * ONE as i64 * self.b.height as i64 / (PLAYER_HEIGHT * ONE as i64)) as Fx
    }

    fn w(&self, v: i32) -> Fx {
        (self.s(v) as i64 * self.b.girth_pct as i64 * crate::pose::BODY_GIRTH_PCT as i64 / 10_000) as Fx
    }

    fn proud(&self) -> Fx {
        self.s(PROUD_MM)
    }

    fn at(&self, p: V3) -> Affine {
        frame(self.right, self.up, self.back, p)
    }

    /// `x` right, `y` up and `z` back of `p` (millimetres; x and z widened).
    fn off(&self, p: V3, x: i32, y: i32, z: i32) -> V3 {
        p.add(self.right.scale(self.w(x)))
            .add(self.up.scale(self.s(y)))
            .add(self.back.scale(self.w(z)))
    }

    fn plated(&self, seg: usize) -> bool {
        self.bare & bit(seg) == 0
    }

    /// The body's colour on part `seg`: frame under plate, mail when bare.
    fn under(&self, seg: usize) -> (Layer, Rgb) {
        if self.plated(seg) {
            (Layer::Frame, self.sc.frame)
        } else {
            (Layer::Mail, self.sc.mail)
        }
    }

    fn push(&mut self, seg: usize, layer: Layer, role: Role, mesh: Mesh, at: Affine, rgb: Rgb) {
        let ink = self.ink;
        self.out.parts.push(Tagged {
            part: Part {
                mesh,
                at,
                rgb,
                ink,
                glow: 0,
                halo: 0,
            },
            seg: seg as u8,
            layer,
            role,
        });
    }

    fn glow(&mut self, seg: usize, at: Affine, rgb: Rgb) {
        self.out.parts.push(Tagged {
            part: Part {
                mesh: Mesh::Dot,
                at,
                rgb,
                ink: 0,
                glow: ONE,
                halo: ONE / 3,
            },
            seg: seg as u8,
            layer: Layer::Cloth,
            role: Role::Trim,
        });
    }

    /// A faceted ball of radii `r` in the body's axes at `p`.
    fn ball(&mut self, seg: usize, layer: Layer, role: Role, p: V3, r: V3, rgb: Rgb) {
        let a = put(&self.at(p), V3::ZERO, v3(2 * r.x, 2 * r.y, 2 * r.z));
        self.push(seg, layer, role, Mesh::Facet, a, rgb);
    }

    /// A box `size` across centred at `p` in the body's axes, turned by `turn`.
    fn boxed(
        &mut self,
        seg: usize,
        layer: Layer,
        role: Role,
        mesh: Mesh,
        p: V3,
        turn: &Affine,
        size: V3,
        rgb: Rgb,
    ) {
        let a = put_turned(&self.at(p), V3::ZERO, turn, size);
        self.push(seg, layer, role, mesh, a, rgb);
    }

    /// A ten-sided column of radius `r` from `a` to `b`.
    fn limb(&mut self, seg: usize, layer: Layer, role: Role, a: V3, b: V3, r: Fx, rgb: Rgb) {
        let at = bar(a, b, (2 * r, 2 * r), self.right);
        self.push(seg, layer, role, Mesh::Cyl, at, rgb);
    }

    /// A plate round the limb `a`–`b` of radius `r`, from share `t0` to `t1`
    /// of the way: turned as the limb is, so its faces stand `PROUD_MM` ×
    /// cos 18° off the limb's.
    #[allow(clippy::too_many_arguments)]
    fn sleeve(&mut self, seg: usize, a: V3, b: V3, t0: Fx, t1: Fx, r: Fx, rgb: Rgb) {
        let (p, q) = (a.lerp(b, t0), a.lerp(b, t1));
        let rr = r + self.proud();
        let at = bar(p, q, (2 * rr, 2 * rr), self.right);
        self.push(seg, Layer::Plate, Role::Body, Mesh::Cyl, at, rgb);
    }
}

/// The half-width at height `dy` from the middle of an ellipsoid of
/// half-width `r` and half-height `ry`.
fn across_at(r: Fx, ry: Fx, dy: Fx) -> Fx {
    let k = fx::div(dy, ry.max(1));
    fx::mul(r, fx::sqrt((ONE - fx::mul(k, k)).max(0)))
}

/// A knight of `kind` on `pose`, its plate off the parts in `bare`.
pub fn model(kind: Kind, pose: &Pose, bare: u16) -> Knight {
    let t = table();
    let b = build(kind);
    let fwd = facing(pose.facing, 0);
    let up0 = pose.chest.sub(pose.pelvis).norm();
    let up = if up0 == V3::ZERO { V3::UP } else { up0 };
    let back0 = fwd.neg();
    let back = back0.sub(up.scale(back0.dot(up))).norm();
    let back = if back == V3::ZERO { v3(0, 0, ONE) } else { back };
    let right = up.cross(back).norm();
    let mut k = K {
        out: Knight::default(),
        b,
        ink: t.ink_figure,
        right,
        up,
        back,
        fwd: back.neg(),
        pose: *pose,
        sc: scheme(kind),
        bare,
    };
    let b_knight = kind == Kind::KnightB;
    body(&mut k);
    legs(&mut k, b_knight);
    torso(&mut k, b_knight);
    arms(&mut k, b_knight);
    helm(&mut k, b_knight);
    k.out
}

/// The plates on part `s` of a knight of `kind` on `pose`, as they are
/// when they break off (`wounds::shed`).
pub fn plate(kind: Kind, pose: &Pose, bare: u16, s: usize) -> Vec<Part> {
    model(kind, pose, bare & !bit(s))
        .parts
        .iter()
        .filter(|t| t.seg as usize == s && t.layer == Layer::Plate)
        .map(|t| t.part)
        .collect()
}

/// The figure's own shapes, every one in the frame colour (mail where
/// bare).
fn body(k: &mut K) {
    let p = k.pose;
    for s in [LEFT, RIGHT] {
        let (lt, ct) = k.under(THIGH[s]);
        let (ls, cs) = k.under(SHIN[s]);
        let ankle = p.foot[s].add(V3::UP.scale(k.s(70)));
        let r = k.w(THIGH_R);
        k.limb(THIGH[s], lt, Role::Body, p.hip[s], p.knee[s], r, ct);
        let kr = v3(k.w(KNEE_R), k.w(KNEE_R), k.w(KNEE_R));
        k.ball(THIGH[s], lt, Role::Body, p.knee[s], kr, ct);
        let r = k.w(SHIN_R);
        k.limb(SHIN[s], ls, Role::Body, p.knee[s], ankle, r, cs);
        let shoe = p.foot[s].add(V3::UP.scale(k.s(44))).add(k.fwd.scale(k.s(34)));
        let fr = v3(k.w(70), k.s(52), k.s(120));
        k.ball(SHIN[s], ls, Role::Body, shoe, fr, cs);
    }
    let (l, c) = k.under(TORSO);
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
        let (lu, cu) = k.under(UPPER_ARM[s]);
        let (lf, cf) = k.under(FOREARM[s]);
        let sr = k.w(SHOULDER_R);
        k.ball(UPPER_ARM[s], lu, Role::Body, p.shoulder[s], v3(sr, sr, sr), cu);
        let r = k.w(UPPER_ARM_R);
        k.limb(UPPER_ARM[s], lu, Role::Body, p.shoulder[s], p.elbow[s], r, cu);
        let er = k.w(ELBOW_R);
        k.ball(UPPER_ARM[s], lu, Role::Body, p.elbow[s], v3(er, er, er), cu);
        let r = k.w(FOREARM_R);
        k.limb(FOREARM[s], lf, Role::Body, p.elbow[s], p.hand[s], r, cf);
        let hr = k.w(HAND_R);
        k.ball(FOREARM[s], lf, Role::Body, p.hand[s], v3(hr, hr, hr), cf);
    }
    let (lh, ch) = k.under(HEAD);
    let r = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
    k.ball(HEAD, lh, Role::Body, p.head, r, ch);
}

/// The belly's middle and radii (`figure.rs`'s).
fn belly(k: &K) -> (V3, V3) {
    let p = k.pose;
    let mid = p.pelvis.lerp(p.chest, ONE / 2);
    let half = p.chest.sub(p.pelvis).len() / 2 + k.s(60);
    (mid.add(k.up.scale(k.s(20))), v3(k.w(160), half, k.w(122)))
}

/// The shoulders' yoke (`figure.rs`'s).
fn yoke(k: &K) -> (V3, V3) {
    (k.off(k.pose.chest, 0, -26, 4), v3(k.w(188), k.s(100), k.w(118)))
}

fn legs(k: &mut K, b_knight: bool) {
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
            // the knee cop, a ball over the knee's, in the second tone
            let kr = k.w(KNEE_R + 16) + k.proud();
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
            let shade = if b_knight { sc.scorch } else { sc.main };
            k.ball(SHIN[s], Layer::Plate, Role::Body, shoe, fr, shade);
        }
    }
}

fn torso(k: &mut K, b_knight: bool) {
    let p = k.pose;
    let sc = k.sc;
    let pr = k.proud();
    let none = Affine::IDENTITY;
    let (bc, br) = belly(k);
    let (yc, yr) = yoke(k);
    // the cloth first: it never breaks off
    if b_knight {
        // the tabard down the back, a gilt mark on it
        let depth = yr.z.max(br.z) + pr + k.s(34);
        let top = k.off(p.chest, 0, -20, 0);
        let bottom = k.off(p.pelvis, 0, 100, 0);
        let mid = top.lerp(bottom, ONE / 2).add(k.back.scale(depth));
        let tall = top.dist(bottom);
        k.boxed(
            TORSO,
            Layer::Cloth,
            Role::Body,
            Mesh::Cube,
            mid,
            &none,
            v3(k.w(300), tall, k.s(16)),
            sc.accent,
        );
        let mark = k.off(p.chest, 0, -150, 0).add(k.back.scale(depth + k.s(8) + pr));
        let turn = Affine::rot_x(fx::QUARTER_TURN);
        k.boxed(
            TORSO,
            Layer::Cloth,
            Role::Trim,
            Mesh::Pyramid,
            mark,
            &turn,
            v3(k.w(90), k.s(16), k.s(110)),
            sc.second,
        );
        // the fur mantle over the shoulders, its edge in ragged tufts
        let mantle = k.off(p.chest, 0, 30, 10);
        let mr = v3(k.w(232), k.s(104), k.w(168));
        k.ball(TORSO, Layer::Cloth, Role::Body, mantle, mr, sc.cloth);
        for j in 0..10 {
            let a = fx::TURN * j / 10 + fx::TURN / 20;
            let at = mantle
                .add(k.right.scale(fx::mul(fx::sin(a), mr.x * 92 / 100)))
                .add(k.back.scale(fx::mul(fx::cos(a), mr.z * 92 / 100)))
                .sub(k.up.scale(k.s(80)));
            let flip = Affine::rot_x(fx::HALF_TURN);
            let len = k.s(if j % 2 == 0 { 120 } else { 80 });
            k.boxed(
                TORSO,
                Layer::Cloth,
                Role::Body,
                Mesh::Cone,
                at,
                &flip,
                v3(k.w(70), len, k.w(70)),
                sc.cloth,
            );
        }
    } else {
        // the torn cloak behind, flaring a little, its hem in tatters
        let depth = yr.z + pr + k.s(46);
        let top = k.off(p.chest, 0, 60, 0).add(k.back.scale(depth));
        let bottom = k.off(p.pelvis, 0, -470, 0).add(k.back.scale(depth + k.s(30)));
        let tall = top.dist(bottom);
        let at = bar(bottom, top, (k.w(560), k.s(18)), k.right);
        k.push(TORSO, Layer::Cloth, Role::Body, Mesh::Cube, at, sc.cloth);
        let _ = tall;
        for (j, x) in [-230, -115, 0, 115, 230].into_iter().enumerate() {
            let len = k.s([150, 90, 170, 110, 140][j]);
            let at = bottom
                .add(k.right.scale(k.w(x)))
                .sub(k.up.scale(len / 2 - k.s(10)));
            let flip = Affine::rot_x(fx::HALF_TURN);
            k.boxed(
                TORSO,
                Layer::Cloth,
                Role::Body,
                Mesh::Pyramid,
                at,
                &flip,
                v3(k.w(110), len, k.s(18)),
                sc.scorch,
            );
        }
        // a tail of cloth hanging in front of the fauld
        let front = k.w(152) * 100 / 95 + pr * 2 + k.s(40);
        let at = k.off(p.pelvis, 0, -170, 0).add(k.fwd.scale(front));
        k.boxed(
            TORSO,
            Layer::Cloth,
            Role::Body,
            Mesh::Cube,
            at,
            &none,
            v3(k.w(130), k.s(300), k.s(14)),
            sc.cloth,
        );
    }
    if !k.plated(TORSO) {
        return;
    }
    // the cuirass: a breastplate over the yoke and a plackart over the belly
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
    // the gorget round the neck
    let chin = p.head.sub(k.up.scale(k.s(150)));
    let gr = k.s(112);
    let neck_top = p.head.sub(k.up.scale(k.s(120)));
    k.limb(
        TORSO,
        Layer::Plate,
        Role::Body,
        p.chest.sub(k.up.scale(k.s(20))),
        neck_top,
        gr,
        sc.second,
    );
    let _ = chin;
    // how wide the plackart is at a height `y` over the pelvis
    let plackart_at = |k: &K, y: i32| {
        let dy = k.off(p.pelvis, 0, y, 0).sub(bc).dot(k.up);
        (across_at(br.x + pr, br.y, dy), across_at(br.z + pr, br.y, dy))
    };
    if b_knight {
        // the gilt ridge down the breastplate
        let front = yc.add(k.fwd.scale(yr.z + pr));
        k.boxed(
            TORSO,
            Layer::Plate,
            Role::Trim,
            Mesh::Cube,
            front,
            &none,
            v3(k.s(34), k.s(200), k.s(28)),
            sc.second,
        );
        // the belt, where the skirt starts: over the plackart and the hips
        let (px, pz) = plackart_at(k, 60);
        let hx = k.w(152) * 95 / 100;
        let belt_x = fx::div(px.max(hx) + pr, COS18);
        let belt_z = fx::div(pz.max(k.w(112)) + pr, COS18);
        let a = k.off(p.pelvis, 0, 40, 0);
        let b2 = k.off(p.pelvis, 0, 84, 0);
        let at = bar(a, b2, (2 * belt_x, 2 * belt_z), k.right);
        k.push(TORSO, Layer::Plate, Role::Waist, Mesh::Cyl, at, sc.second);
        // four lames, each wider than the one above, to the knee; the
        // lowest scorched, a gilt band at each hem
        let mut rx = belt_x + k.s(22);
        let mut rz = belt_z + k.s(18);
        for i in 0..4 {
            let top = 50 - 92 * i;
            let bottom = top - 118;
            let a = k.off(p.pelvis, 0, bottom, 0);
            let b2 = k.off(p.pelvis, 0, top, 0);
            let (role, colour) = if i == 3 {
                (Role::Hem, sc.scorch)
            } else {
                (Role::Body, sc.main)
            };
            let at = bar(a, b2, (2 * rx, 2 * rz), k.right);
            k.push(TORSO, Layer::Plate, role, Mesh::Cyl, at, colour);
            let band = k.s(PROUD_MM + 4);
            let lo = k.off(p.pelvis, 0, bottom - 8, 0);
            let hi = k.off(p.pelvis, 0, bottom + 12, 0);
            let at = bar(lo, hi, (2 * (rx + band), 2 * (rz + band)), k.right);
            k.push(TORSO, Layer::Plate, Role::Trim, Mesh::Cyl, at, sc.second);
            if i == 3 {
                // embers caught in the scorched hem
                for (x, y) in [(-120, 40), (40, 26), (150, 52)] {
                    let d = fx::div(k.w(x), rx);
                    let z = fx::mul(rz, fx::sqrt((ONE - fx::mul(d, d)).max(0)));
                    let at = k.off(p.pelvis, x, bottom + y, 0).add(k.fwd.scale(z));
                    let e = k.s(22);
                    let a = put(&k.at(at), V3::ZERO, v3(e, e, e));
                    let glow = k.sc.glow;
                    k.glow(TORSO, a, glow);
                }
            }
            rx += k.w(46);
            rz += k.w(30);
        }
    } else {
        // the fauld: a skirt of plate over the hips, in the second tone
        let (px, pz) = plackart_at(k, 70);
        let fx_ = fx::div(px.max(k.w(152)) + pr, COS18);
        let fz = fx::div(pz.max(k.w(112)) + pr, COS18);
        let a = k.off(p.pelvis, 0, -140, 0);
        let b2 = k.off(p.pelvis, 0, 80, 0);
        let at = bar(a, b2, (2 * fx_, 2 * fz), k.right);
        k.push(TORSO, Layer::Plate, Role::Hip, Mesh::Cyl, at, sc.second);
        // bronze clasps where the cloak hangs from the breastplate: round
        // studs (smooth, so an arm swung across one never shares a face)
        for side in [-1, 1] {
            let at = k
                .off(p.chest, side * 120, 50, 0)
                .add(k.fwd.scale(yr.z * 80 / 100 + pr));
            let r = k.s(30);
            let a = put(&k.at(at), V3::ZERO, v3(2 * r, 2 * r, 2 * r));
            k.push(TORSO, Layer::Plate, Role::Trim, Mesh::Dot, a, sc.accent);
        }
    }
}

fn arms(k: &mut K, b_knight: bool) {
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
            if b_knight {
                // a round pauldron
                let at = k.off(p.shoulder[s], side * 34, 34, 0);
                let r = v3(k.w(130), k.s(88), k.w(124));
                k.ball(ua, Layer::Plate, Role::Pauldron, at, r, sc.main);
            } else {
                // three lames down off the shoulder, wider than the hips
                for i in 0..3 {
                    let at = k.off(p.shoulder[s], side * (58 + 14 * i), 64 - 78 * i, 0);
                    let tilt = Affine::rot_z(-side * fx::deg(16 + 9 * i));
                    let size = v3(k.w(250 - 8 * i), k.s(110 - 6 * i), k.w(250 - 22 * i));
                    let colour = if i % 2 == 0 { sc.main } else { sc.second };
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
            let hr = k.w(HAND_R) + k.proud();
            k.ball(fa, Layer::Plate, Role::Body, p.hand[s], v3(hr, hr, hr), sc.second);
        }
    }
}

fn helm(k: &mut K, b_knight: bool) {
    if !k.plated(HEAD) {
        return;
    }
    let p = k.pose;
    let sc = k.sc;
    let pr = k.proud();
    let none = Affine::IDENTITY;
    let hr = v3(k.s(HEAD_R.0), k.s(HEAD_R.1), k.s(HEAD_R.2));
    // the slit at `plate::VISOR_MM`, the gap the rule tests for
    let slit_y = k.s((VISOR_MM.0 + VISOR_MM.1) / 2);
    let slit_h = k.s(VISOR_MM.1 - VISOR_MM.0);
    if b_knight {
        // a rounded helm, a crown of spikes on a gilt band
        let c = p.head.add(k.up.scale(k.s(8)));
        let r = v3(hr.x + pr + k.s(4), hr.y + pr + k.s(10), hr.z + pr + k.s(4));
        k.ball(HEAD, Layer::Plate, Role::Helm, c, r, sc.main);
        let band_y = k.s(150);
        let dy = band_y - k.s(8);
        let bx = fx::div(across_at(r.x, r.y, dy) + pr, COS18);
        let bz = fx::div(across_at(r.z, r.y, dy) + pr, COS18);
        let band_h = k.s(44);
        let a = p.head.add(k.up.scale(band_y - band_h / 2));
        let b2 = p.head.add(k.up.scale(band_y + band_h / 2));
        let at = bar(a, b2, (2 * bx, 2 * bz), k.right);
        k.push(HEAD, Layer::Plate, Role::Crown, Mesh::Cyl, at, sc.second);
        for (j, h) in [240, 170, 120, 100, 96, 100, 120, 170].into_iter().enumerate() {
            let az = fx::TURN * j as i32 / 8;
            let foot = p
                .head
                .add(k.fwd.scale(fx::mul(fx::cos(az), bz * 88 / 100)))
                .add(k.right.scale(fx::mul(fx::sin(az), bx * 88 / 100)))
                .add(k.up.scale(band_y + band_h / 2 - k.s(12)));
            let tall = k.s(h);
            let at = foot.add(k.up.scale(tall / 2));
            let rb = k.s(30);
            k.boxed(
                HEAD,
                Layer::Plate,
                Role::Crown,
                Mesh::Cone,
                at,
                &none,
                v3(2 * rb, tall, 2 * rb),
                sc.second,
            );
        }
        // the slit and the breath slot, dark, cut across the helm's front
        let z = across_at(r.z, r.y, slit_y - k.s(8));
        let at = p.head.add(k.up.scale(slit_y)).add(k.fwd.scale(z));
        k.boxed(
            HEAD,
            Layer::Plate,
            Role::Slit,
            Mesh::Cube,
            at,
            &none,
            v3(k.s(120), slit_h, k.s(28)),
            sc.frame,
        );
        let z2 = across_at(r.z, r.y, -k.s(70));
        let at = p.head.sub(k.up.scale(k.s(62))).add(k.fwd.scale(z2));
        let tall = k.s(124) + k.s(VISOR_MM.0);
        k.boxed(
            HEAD,
            Layer::Plate,
            Role::Trim,
            Mesh::Cube,
            at,
            &none,
            v3(k.s(18), tall, k.s(26)),
            sc.frame,
        );
    } else {
        // the bucket: a flat-topped column round the head
        let rx = fx::div(hr.x + pr, COS18);
        let rz = fx::div(hr.z + pr, COS18);
        let (lo, hi) = (k.s(-215), k.s(250));
        let a = p.head.add(k.up.scale(lo));
        let b2 = p.head.add(k.up.scale(hi));
        let at = bar(a, b2, (2 * rx, 2 * rz), k.right);
        k.push(HEAD, Layer::Plate, Role::Helm, Mesh::Cyl, at, sc.main);
        // the small ridge along its crown
        let at = p.head.add(k.up.scale(hi + k.s(12)));
        k.boxed(
            HEAD,
            Layer::Plate,
            Role::Trim,
            Mesh::Cube,
            at,
            &none,
            v3(k.s(30), k.s(44), 2 * rz * 86 / 100),
            sc.second,
        );
        // the slit and the breath slot on its flat front face
        let face = fx::mul(rz, COS18);
        let at = p.head.add(k.up.scale(slit_y)).add(k.fwd.scale(face));
        k.boxed(
            HEAD,
            Layer::Plate,
            Role::Slit,
            Mesh::Cube,
            at,
            &none,
            v3(k.s(124), slit_h, k.s(26)),
            sc.frame,
        );
        let top = k.s(VISOR_MM.0);
        let bottom = k.s(-160);
        let at = p.head.add(k.up.scale((top + bottom) / 2)).add(k.fwd.scale(face));
        k.boxed(
            HEAD,
            Layer::Plate,
            Role::Trim,
            Mesh::Cube,
            at,
            &none,
            v3(k.s(18), top - bottom, k.s(22)),
            sc.frame,
        );
    }
}

#[cfg(test)]
mod tests;
