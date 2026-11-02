//! `plate` (M3, CA-Q8: A): plate armour as a rule of the duel.
//!
//! **Where plate is.** A knight (`Kind::KnightA`, `Kind::KnightB`) wears a
//! plate on each of the ten parts `blade::body` gives a body (head, torso,
//! each upper arm and forearm, each thigh and shin), the parts `wounds.rs`
//! already keeps damage on. Each plate has an integrity (`INTEGRITY`, kept
//! in `wounds::Limbs::plate`, whole numbers, so it is in the world's hash
//! and rolls back with it). Every other kind wears none: its integrities are
//! all 0, and a blow on it is CHUMWORLD's blow, unchanged.
//!
//! **What plate takes.** A blow on a plated part goes through at
//! `CUT_THROUGH_PCT` of CHUMWORLD's damage for a cut (plate takes 70% off)
//! and `THRUST_THROUGH_PCT` for a thrust (20% off). A blow is a thrust when
//! the blade travels along its own line, point first: within 45° of the
//! pommel-to-tip direction (`is_thrust`).
//!
//! **Gaps.** Three places on a plated part take a blow whole (`gap`), tested
//! on the contact in the part's own frame: where the blade's nearest point
//! is along the part's spine (a share of the way from its top joint), and
//! the way from the spine out to the blade.
//!
//! - the armpit: the upper arm's first `ARMPIT_PCT` from the shoulder, on
//!   the side facing the torso (within 60° of the way from the shoulder to
//!   the chest, square to the arm);
//! - the visor's slit: on the head, between `VISOR_MM` above the head's
//!   middle, from the front (within 40° of the way the body faces), where
//!   `models/knight.rs` draws the slit;
//! - the back of the knee: the thigh's last `KNEE_PCT` above the knee or
//!   the shin's first `KNEE_PCT` below it, from behind (within 60° of the
//!   way back).
//!
//! **Wear.** What a plate takes off a blow (CHUMWORLD's damage less what
//! went through) comes off its integrity. At 0 the plate breaks off: it
//! falls as a piece (`wounds::shed`, CHUMWORLD's severed-part flight) and
//! the part shows the mail beneath. A blow through a gap wears nothing.
//!
//! **Severing.** A part whose plate is on when it is struck is never taken
//! off, whatever the blow's speed (`Hit::plated`); a bare part comes off at
//! CHUMWORLD's thresholds (`wounds::mark`). The blow that breaks a plate
//! does not also sever: the plate took it.
//!
//! How a module joins the world (state, step, draw, scene, screen, act):
//! the extension contract in `world.rs`. `land` runs at step 4, inside
//! `health::step` before health is taken, and in `wounds::stage`.

use crate::arm;
use crate::blade::{Hit, PARTS};
use crate::fx::{Fx, ONE};
use crate::geom::{facing, V3};
use crate::player;
use crate::pose::{self, Kind, Pose, LEFT, RIGHT};
use crate::world::World;
use crate::wounds::{self, FOREARM, HEAD, SHIN, THIGH, TORSO, UPPER_ARM};

/// Of CHUMWORLD's damage, what goes through plate: a cut, a thrust (R5.3).
pub const CUT_THROUGH_PCT: i32 = 30;
pub const THRUST_THROUGH_PCT: i32 = 80;
/// A blow is a thrust when the blade travels within 45° of its own line,
/// point first (cos 45° = 0.707).
pub const THRUST_COS: Fx = ONE * 707 / 1000;

/// Each part's plate when whole, by `blade::body`'s index: three times
/// what plate takes off a 10 m/s cut on that part (R5.4). A 10 m/s cut is
/// CHUMWORLD's 21 on the torso, of which 6 goes through and 15 is taken
/// (45); on the head 31, 9 and 22 (66); on an arm 12, 3 and 9 (27); on a
/// leg 14, 4 and 10 (30). So the third such cut breaks a plate off.
pub const INTEGRITY: [i32; PARTS] = [66, 45, 27, 27, 27, 27, 30, 30, 30, 30];

/// The armpit: the upper arm's first quarter from the shoulder (R5.5).
pub const ARMPIT_PCT: i32 = 25;
/// The back of the knee: the thigh's last quarter, the shin's first.
pub const KNEE_PCT: i32 = 25;
/// The visor's slit, millimetres above the head's middle on the player's
/// 1.70 m (scaled to the build), and the narrowest a blow can come in by.
pub const VISOR_MM: (i32, i32) = (8, 32);
/// How square to the opening a blow must come: within 60° of the way into
/// the armpit or the knee (cos 60°), within 40° of the face (cos 40°).
pub const SIDE_COS: Fx = ONE / 2;
pub const VISOR_COS: Fx = ONE * 766 / 1000;

/// A kind that wears plate.
pub fn armoured(kind: Kind) -> bool {
    matches!(kind, Kind::KnightA | Kind::KnightB) || ladder_plate(kind)
}

/// M6 (R7.302): the champion and the mirror wear plate on all ten parts,
/// as a knight does.
fn ladder_plate(kind: Kind) -> bool {
    matches!(kind, Kind::Champion | Kind::Mirror)
}

/// A body's plates when it comes into the ring: whole for a knight, none
/// for anyone else.
pub fn full(kind: Kind) -> [i32; PARTS] {
    if armoured(kind) {
        INTEGRITY
    } else {
        [0; PARTS]
    }
}

/// The parts with no plate on them, a bit each (`wounds::bit`).
pub fn bare(plate: &[i32; PARTS]) -> u16 {
    (0..PARTS)
        .filter(|&s| plate[s] <= 0)
        .fold(0, |b, s| b | wounds::bit(s))
}

/// One of the three openings in plate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gap {
    Armpit,
    Visor,
    Knee,
}

/// Millimetres on the player, scaled to `kind`'s height.
pub fn scaled(kind: Kind, v: i32) -> Fx {
    let b = pose::build(kind);
    (v as i64 * ONE as i64 * b.height as i64 / (1000 * pose::build(Kind::Player).height as i64)) as Fx
}

/// `v` with its share along the unit `u` taken out, made unit (zero if
/// nothing is left).
fn square_to(v: V3, u: V3) -> V3 {
    v.sub(u.scale(v.dot(u))).norm()
}

/// Which gap, if any, a blow on part `s` of a `kind` on `pose` came in by:
/// `at` the nearest point on the part's spine (`blade::body`'s capsule),
/// `from` the blade's nearest point.
pub fn gap(kind: Kind, pose: &Pose, s: usize, at: V3, from: V3) -> Option<Gap> {
    let caps = crate::blade::body(kind, pose);
    let c = caps.get(s)?;
    let line = c.b.sub(c.a);
    let len = line.len();
    if len == 0 {
        return None;
    }
    let u = line.norm();
    // the share of the way along the spine from its top joint, percent
    let share = (at.sub(c.a).dot(u) as i64 * 100 / len as i64) as i32;
    let d = square_to(from.sub(at), u);
    if d == V3::ZERO {
        return None;
    }
    let side = |s: usize| {
        if s == UPPER_ARM[LEFT] || s == THIGH[LEFT] || s == SHIN[LEFT] {
            LEFT
        } else {
            RIGHT
        }
    };
    let back = facing(pose.facing, 0).neg();
    if s == UPPER_ARM[LEFT] || s == UPPER_ARM[RIGHT] {
        let inward = square_to(pose.chest.sub(pose.shoulder[side(s)]), u);
        if share <= ARMPIT_PCT && inward != V3::ZERO && d.dot(inward) >= SIDE_COS {
            return Some(Gap::Armpit);
        }
    } else if s == HEAD {
        let up = at.sub(c.a).dot(u);
        let (lo, hi) = (scaled(kind, VISOR_MM.0), scaled(kind, VISOR_MM.1));
        let front = square_to(back.neg(), u);
        if (lo..=hi).contains(&up) && d.dot(front) >= VISOR_COS {
            return Some(Gap::Visor);
        }
    } else if s == THIGH[LEFT] || s == THIGH[RIGHT] || s == SHIN[LEFT] || s == SHIN[RIGHT] {
        let near_knee = if s == THIGH[LEFT] || s == THIGH[RIGHT] {
            share >= 100 - KNEE_PCT
        } else {
            share <= KNEE_PCT
        };
        let behind = square_to(back, u);
        if near_knee && behind != V3::ZERO && d.dot(behind) >= SIDE_COS {
            return Some(Gap::Knee);
        }
    }
    let _ = (TORSO, FOREARM);
    None
}

/// A blade travelling `vel` along `axis` (pommel to tip) lands point first.
pub fn is_thrust(vel: V3, axis: V3) -> bool {
    let (v, a) = (vel.norm(), axis.norm());
    v != V3::ZERO && a != V3::ZERO && v.dot(a) >= THRUST_COS
}

/// What of CHUMWORLD's damage goes through, percent: all of it on a bare
/// part or through a gap; otherwise a cut's or a thrust's share.
pub fn through_pct(plated: bool, gap: bool, thrust: bool) -> i32 {
    if !plated || gap {
        100
    } else if thrust {
        THRUST_THROUGH_PCT
    } else {
        CUT_THROUGH_PCT
    }
}

/// Step 4, before health is taken: the blow `hit` met plate or not. Sets
/// `hit.plated` and `hit.through_pct`, wears the plate by what it took, and
/// breaks it off at no integrity.
pub fn land(w: &mut World, hit: &mut Hit) {
    let id = hit.on as usize;
    hit.plated = false;
    hit.through_pct = 100;
    if id >= player::count(w) {
        return;
    }
    let (kind, lost, plate) = {
        let f = player::fighter(w, id);
        (f.kind, f.health.limbs.lost, f.health.limbs.plate)
    };
    let s = wounds::landed_on((hit.seg as usize).min(PARTS - 1), lost);
    if plate[s] <= 0 {
        return;
    }
    hit.plated = true;
    let pose = player::pose_of(w, id);
    let opening = gap(kind, &pose, s, hit.at, hit.from).is_some();
    let thrust = (hit.by as usize) < player::count(w) && {
        let a = &player::fighter(w, hit.by as usize).arm;
        is_thrust(hit.vel, a.pts[arm::TIP].p.sub(a.pts[arm::POMMEL].p))
    };
    hit.through_pct = through_pct(true, opening, thrust);
    let full = crate::health::damage(hit.speed, hit.part);
    let through = crate::health::damage_through(hit.speed, hit.part, hit.through_pct);
    let taken = full - through;
    if taken <= 0 {
        return;
    }
    let left = {
        let l = &mut player::parts_mut(w, id).1.health.limbs;
        l.plate[s] = (l.plate[s] - taken).max(0);
        l.plate[s]
    };
    if left == 0 {
        wounds::shed(w, id, s, hit.vel);
    }
}

#[cfg(test)]
mod tests;
