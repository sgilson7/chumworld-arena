//! `feel` (M1 Duel, the polish pass D2): what makes a blow land. A hard
//! blow or a hard block stops the world for a few ticks (the hit-stop), the
//! camera takes a small knock that rings out, blades that meet throw a
//! spark, and a body that is struck staggers back (`motion::Gait::stagger`).
//! Everything here is in the core and runs on the tick (CW-1, CW-4), and
//! everything decays to exactly nothing, so a still view still draws the
//! same pixels frame after frame (CW-18).
//!
//! It also keeps the tick's events for the story (M5): who went down this
//! tick (`State::downed`), and `World::player_down`.
//!
//! How a module joins the world (state, step, draw, scene, screen, act):
//! the extension contract in `world.rs`.

use crate::fx::{self, Fx, ONE};
use crate::geom::{v3, Affine, V3};
use crate::input::Input;
use crate::look::{rgb, Look};
use crate::player;
use crate::prim::Mesh;
use crate::render::Shapes;
use crate::world::World;

/// A blow or a block at least this fast (metres a second) is hard: it stops
/// the world for `STOP_MIN` ticks, a tick more for each `STOP_PER` faster,
/// at most `STOP_MAX`.
pub const HARD: Fx = fx::int(5);
pub const STOP_MIN: u8 = 2;
pub const STOP_PER: Fx = fx::int(4);
pub const STOP_MAX: u8 = 4;
/// The camera's knock: 6 mm for each metre a second, at most 5 cm, along
/// the blow; each tick it swings back through the middle and keeps
/// `RING_PCT` of itself, and under `QUIET` it is nothing.
pub const KNOCK_PER_MPS: Fx = ONE * 6 / 1000;
pub const KNOCK_MAX: Fx = ONE * 5 / 100;
pub const RING_PCT: i32 = 55;
pub const QUIET: Fx = ONE / 2000;
/// Blades meeting faster than this throw a spark that burns `SPARK_TICKS`
/// (M7: a third of a second, from M1's 14, so a still can catch it).
pub const SPARK_MIN: Fx = ONE * 3 / 2;
pub const SPARK_TICKS: u8 = 20;
/// A struck body staggers for `STAGGER_TICKS`, shoved back at 4 cm/s for
/// each point of health taken, at most `SHOVE_MAX`.
pub const STAGGER_TICKS: u8 = 14;
pub const SHOVE_PER_POINT: Fx = ONE * 4 / 100;
pub const SHOVE_MAX: Fx = ONE * 18 / 10;

/// The animation pass (T10): the eye tilts with the player's own cut: `TILT_PER_MPS` for each metre a
/// second the point crosses the view, at most `TILT_MAX`, closing
/// `TILT_EASE_PCT` of the gap each tick; a blow taken throws it
/// `TILT_HIT` away from the striker. Under `TILT_QUIET` with nothing
/// driving it, it is level exactly.
pub const TILT_PER_MPS: i32 = fx::deg(3) / 5;
pub const TILT_MAX: i32 = fx::deg(7);
pub const TILT_EASE_PCT: i32 = 18;
pub const TILT_HIT: i32 = fx::deg(5);
/// Blades the player's meets at `HARD` or faster throw the eye this much,
/// away from the other fighter.
pub const TILT_CLASH: i32 = fx::deg(3);
pub const TILT_QUIET: i32 = fx::deg(1) / 20;
/// A blow taken reddens the view's edge: `HURT_PER_POINT` of full for each
/// point of health lost, at most full; it keeps `HURT_KEEP_PCT` of itself
/// each tick and is nothing under `HURT_QUIET`.
pub const HURT_PER_POINT: Fx = ONE / 25;
pub const HURT_KEEP_PCT: i32 = 94;
pub const HURT_QUIET: Fx = ONE / 100;
/// A blade that draws blood keeps it for the round: `BLOOD_PER_POINT` of
/// full for each point of health it takes, at most `BLOOD_MOST`, drawn as
/// a stipple over its outer blade (`models::held::blood_on`).
pub const BLOOD_PER_POINT: Fx = ONE / 60;
pub const BLOOD_MOST: Fx = ONE * 9 / 20;

/// A spark where two blades met: where, how old, how bright it began.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spark {
    pub at: V3,
    pub age: u8,
    pub size: Fx,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    /// ticks of hit-stop left
    pub stop: u8,
    /// the camera's knock this tick and last (metres, drawn between)
    pub knock: V3,
    pub prev_knock: V3,
    pub sparks: Vec<Spark>,
    /// the fighters who went down this tick, by number (the story's hook)
    pub downed: Vec<u8>,
    /// M7, test-only (`feel.freeze_spark`): the world stops still once a
    /// spark is this many ticks old, for a picture of it; `frozen` once it has
    pub freeze_spark: Option<u8>,
    pub frozen: bool,
    /// T10: the eye's tilt this tick and last (angle units, right positive)
    pub tilt: i32,
    pub prev_tilt: i32,
    /// T10: how red the view's edge is from a blow taken, 0 to `ONE`
    pub hurt: Fx,
    /// T10: how bloody each fighter's blade is, by fighter number, 0 to
    /// `BLOOD_MOST`; a new round starts clean (`rounds` resets this state)
    pub blooded: Vec<Fx>,
}

/// M7: a spark's size for blades meeting at `speed`: 12.5 cm, and 2.5 cm
/// more for each metre a second, at most a third of a metre (M1's was half
/// that, and lost behind the bodies).
pub fn spark_size(speed: Fx) -> Fx {
    (ONE / 8 + fx::mul(speed, ONE / 40)).min(ONE / 3)
}

/// How long a blow or block at `speed` stops the world.
pub fn stop_ticks(speed: Fx) -> u8 {
    if speed < HARD {
        return 0;
    }
    (STOP_MIN as i32 + (speed - HARD) / STOP_PER).min(STOP_MAX as i32) as u8
}

/// The camera's knock for a blow at `speed` along `way`.
pub fn knock(speed: Fx, way: V3) -> V3 {
    way.norm().scale(fx::mul(KNOCK_PER_MPS, speed).min(KNOCK_MAX))
}

/// The knock a tick on: back through the middle, `RING_PCT` of it kept,
/// nothing once it is under `QUIET`.
pub fn ring(k: V3) -> V3 {
    let n = v3(
        -k.x * RING_PCT / 100,
        -k.y * RING_PCT / 100,
        -k.z * RING_PCT / 100,
    );
    if n.len() < QUIET {
        V3::ZERO
    } else {
        n
    }
}

/// Step 4, after health: this tick's blows and blocks stop the world, knock
/// the camera, throw sparks and stagger the struck; what was set going
/// before rings down.
pub fn step(w: &mut World) {
    let mut stop = 0u8;
    let mut knocked: Option<V3> = None;
    for c in w.blades.clashes.clone() {
        if c.speed >= SPARK_MIN {
            w.feel.sparks.push(Spark {
                at: c.at,
                age: 0,
                size: spark_size(c.speed),
            });
        }
        stop = stop.max(stop_ticks(c.speed));
        if (c.a == 0 || c.b == 0) && c.speed >= HARD {
            let other = if c.a == 0 { c.b } else { c.a } as usize;
            let way = player::body(w, other).pos.sub(w.player.pos);
            knocked = Some(knock(c.speed, v3(way.x, 0, way.z)));
            // T10: the blades' meeting jolts the eye away from the other
            thrown(w, v3(-way.x, 0, -way.z), TILT_CLASH);
        }
    }
    for h in w.blades.hits.clone() {
        let taken = crate::health::damage(h.speed, h.part);
        if taken <= 0 {
            continue;
        }
        let (from, to) = (
            player::body(w, h.by as usize).pos,
            player::body(w, h.on as usize).pos,
        );
        let flat = v3(to.x - from.x, 0, to.z - from.z);
        let way = if flat == V3::ZERO {
            v3(0, 0, ONE)
        } else {
            flat.norm()
        };
        let (b, f) = player::parts_mut(w, h.on as usize);
        f.gait.stagger = STAGGER_TICKS;
        f.gait.stagger_way = way;
        let shove = (SHOVE_PER_POINT * taken).min(SHOVE_MAX) / crate::ticks::TICKS_PER_SECOND as i32;
        b.vel = b.vel.add(way.scale(shove));
        stop = stop.max(stop_ticks(h.speed));
        if h.on == 0 || h.by == 0 {
            knocked = Some(knock(h.speed, way));
        }
        let by = h.by as usize;
        if w.feel.blooded.len() <= by {
            w.feel.blooded.resize(by + 1, 0);
        }
        w.feel.blooded[by] = (w.feel.blooded[by] + BLOOD_PER_POINT * taken).min(BLOOD_MOST);
        if h.on == 0 {
            // thrown away from the striker, and the edge of the view reddens
            thrown(w, way, TILT_HIT);
            w.feel.hurt = (w.feel.hurt + HURT_PER_POINT * taken).min(ONE);
        }
    }
    w.feel.stop = w.feel.stop.max(stop);
    ring_down(w);
    if let Some(k) = knocked {
        w.feel.knock = k;
    }
    lean(w);
}

/// T10: the eye thrown `by` toward the side of the view that `way` (across
/// the ground) points to, held within `TILT_MAX`.
pub fn thrown(w: &mut World, way: V3, by: i32) {
    let side = if way.dot(right_of(w.aim.yaw)) >= 0 { 1 } else { -1 };
    w.feel.tilt = (w.feel.tilt + side * by).clamp(-TILT_MAX, TILT_MAX);
}

/// The view's right across the ground for a heading.
fn right_of(yaw: i32) -> V3 {
    crate::geom::facing(yaw, 0).cross(V3::UP).norm()
}

/// T10: the eye leans after the player's own point as it crosses the view,
/// and comes back level.
fn lean(w: &mut World) {
    w.feel.prev_tilt = w.feel.tilt;
    let tip = player::fighter(w, 0).arm.pts[crate::arm::TIP];
    let across = tip.vel().dot(right_of(w.aim.yaw)) as i64 * crate::ticks::TICKS_PER_SECOND;
    let goal = ((across * TILT_PER_MPS as i64) >> fx::SHIFT).clamp(-TILT_MAX as i64, TILT_MAX as i64) as i32;
    // a hand barely moving does not hold the view off level
    let goal = if goal.abs() < TILT_QUIET { 0 } else { goal };
    let t = w.feel.tilt + (goal - w.feel.tilt) * TILT_EASE_PCT / 100;
    w.feel.tilt = if goal == 0 && t.abs() < TILT_QUIET { 0 } else { t };
    let h = w.feel.hurt * HURT_KEEP_PCT / 100;
    w.feel.hurt = if h < HURT_QUIET { 0 } else { h };
}

/// T10: the blood on fighter `id`'s blade, stippled over its outer blade
/// (nothing with blood off in the settings).
pub fn draw_blood(w: &World, id: usize, item: crate::pose::Held, grip: V3, tip: V3, out: &mut Shapes) {
    let share = w.feel.blooded.get(id).copied().unwrap_or(0);
    if share == 0 || !w.settings.blood {
        return;
    }
    if let Some(at) = crate::models::blood_on(item, grip, tip) {
        out.dithered(
            Mesh::Cube,
            at,
            rgb(crate::wounds::spec().colours.blood_dark),
            share,
        );
    }
}

/// T10: the eye's tilt now, `alpha` of the way from last tick to this.
pub fn tilt_at(w: &World, alpha: Fx) -> i32 {
    w.feel.prev_tilt + fx::mul(w.feel.tilt - w.feel.prev_tilt, alpha)
}

/// The knock rings, sparks age and burn out.
fn ring_down(w: &mut World) {
    let s = &mut w.feel;
    s.prev_knock = s.knock;
    s.knock = ring(s.knock);
    for sp in &mut s.sparks {
        sp.age += 1;
    }
    s.sparks.retain(|sp| sp.age < SPARK_TICKS);
    // M7: held still for a picture once a spark is old enough
    if let Some(age) = s.freeze_spark {
        if s.sparks.iter().any(|sp| sp.age >= age) {
            s.frozen = true;
            s.prev_knock = s.knock;
            settle(w);
        }
    }
}

/// The point halfway between where fighter `a`'s blade and `b`'s come
/// nearest, each blade taken from its pommel to its point in 16 steps.
fn nearest_blades(w: &World, a: usize, b: usize) -> V3 {
    let line = |id: usize| {
        let f = player::fighter(w, id);
        (f.arm.pts[crate::arm::POMMEL].p, f.arm.pts[crate::arm::TIP].p)
    };
    let ((a0, a1), (b0, b1)) = (line(a), line(b));
    let mut best = (i32::MAX, a1);
    for i in 0..=16 {
        let p = a0.lerp(a1, ONE * i / 16);
        for j in 0..=16 {
            let q = b0.lerp(b1, ONE * j / 16);
            let d = p.dist(q);
            if d < best.0 {
                best = (d, p.lerp(q, ONE / 2));
            }
        }
    }
    best.1
}

/// Everything drawn where it stands: each last tick set to this one.
fn settle(w: &mut World) {
    for id in 0..player::count(w) {
        let (b, f) = player::parts_mut(w, id);
        b.prev_pos = b.pos;
        b.prev_facing = b.facing;
        f.prev_pose = f.pose;
        f.arm.prev_target = f.arm.target;
        f.prev_framing = f.framing;
    }
    // (CHUMWORLD also settled what was thrown, the camera's boom, the
    // house's hand-over and the fire's clock; the ring has none, R2.3)
}

/// M7: a tick of a world frozen for a picture (`feel.freeze_spark`): nothing
/// moves, but the mouse turns the view, so the spark can be seen from any
/// side.
pub fn frozen(w: &mut World, input: &Input) {
    w.aim = w.aim.turned(crate::input::Mouse {
        dx: input.dx,
        dy: input.dy,
        wheel: input.wheel,
    });
}

/// `feel.<name>`: `freeze_spark` (test-only) stops the world still once a
/// spark is `value` ticks old (0 to 19), for a picture; `off` lets it go.
pub fn act(w: &mut World, name: &str, value: &str) -> Result<(), String> {
    match name {
        "freeze_spark" => {
            if value.trim() == "off" {
                w.feel.freeze_spark = None;
                w.feel.frozen = false;
                return Ok(());
            }
            let age: u8 = value
                .trim()
                .parse()
                .ok()
                .filter(|a| *a < SPARK_TICKS)
                .ok_or_else(|| {
                    format!("A spark's age is a whole number of ticks under {SPARK_TICKS}, not \"{value}\".")
                })?;
            w.feel.freeze_spark = Some(age);
            Ok(())
        }
        // test-only: a spark staged where the player's blade and the first
        // foe's come nearest, `value` ticks old, as a meeting at 8 m/s throws
        // it, and the world frozen there, for a picture of a spark when a
        // driven browser cannot time a clash (R9.10)
        "stage_spark" => {
            let age: u8 = value
                .trim()
                .parse()
                .ok()
                .filter(|a| *a < SPARK_TICKS)
                .ok_or_else(|| {
                    format!("A spark's age is a whole number of ticks under {SPARK_TICKS}, not \"{value}\".")
                })?;
            if player::count(w) < 2 {
                return Err("A spark needs a foe's blade to meet: start a test duel first.".into());
            }
            let at = nearest_blades(w, 0, 1);
            w.feel.sparks.push(Spark {
                at,
                age,
                size: spark_size(fx::int(8)),
            });
            w.feel.freeze_spark = Some(age);
            w.feel.frozen = true;
            settle(w);
            Ok(())
        }
        _ => Err(format!("Unknown action \"feel.{name}\".")),
    }
}

/// A tick of hit-stop in place of the world's: nothing moves, so every body
/// is drawn where it stands (each last tick set to this one), but the view
/// still turns to the mouse while it looks, the sword hand's target still
/// takes the mouse, sparks burn and the knock rings. Keys pressed now are
/// read when the world runs again.
pub fn hold(w: &mut World, input: &Input) {
    w.feel.stop -= 1;
    if w.me.mouse_looks {
        // (M1: first person may turn the pitch over, `player::view_mouse`)
        w.aim = w.aim.turned(crate::player::view_mouse(w, input));
    }
    crate::arm::move_shell(w, 0, input);
    // M5: and the second seat's knight's
    crate::seat::hold(w);
    settle(w);
    w.blades.hits.clear();
    w.blades.clashes.clear();
    w.feel.downed.clear();
    ring_down(w);
    // T10: the tilt holds where it is through the stop
    w.feel.prev_tilt = w.feel.tilt;
}

/// Every spark: a hot flash that shrinks as it burns, and six motes thrown
/// out from it, all through the glow pass (`Shapes::glow`), in a fixed
/// order. M7: each with a white-hot core drawn solid and unlit (a star of
/// two crossed octahedra), so it reads against a bright sky as well as a
/// dark body; the flash stays near full for its first third.
pub fn draw(w: &World, alpha: Fx, look: &Look, out: &mut Shapes) {
    let hot = rgb(look.palette.lamp);
    let white = [ONE, ONE, ONE * 15 / 16];
    // frozen for a picture, a spark ages no further between ticks either
    let alpha = if w.feel.frozen { 0 } else { alpha };
    for sp in &w.feel.sparks {
        let age = sp.age as i32 * ONE + alpha;
        let left = (SPARK_TICKS as i32 * ONE - age).max(0) / SPARK_TICKS as i32;
        // held near full, then falling: 1 − (1 − left)²
        let bright = ONE - fx::mul(ONE - left, ONE - left);
        let r = fx::mul(sp.size, bright);
        if r <= 0 {
            continue;
        }
        out.glow(
            Mesh::Sphere,
            Affine::translate(sp.at).then(&Affine::scale(v3(r, r, r))),
            hot,
            bright,
        );
        let c = r * 2 / 5;
        out.solid(
            Mesh::Octa,
            Affine::translate(sp.at).then(&Affine::scale(v3(c, c * 5 / 2, c))),
            white,
            ONE,
        );
        out.solid(
            Mesh::Octa,
            Affine::translate(sp.at).then(&Affine::scale(v3(c * 5 / 2, c, c))),
            white,
            ONE,
        );
        let flew = fx::mul(ONE / 2, age / SPARK_TICKS as i32 * 2).min(ONE * 3 / 4);
        let m = r / 3;
        for k in 0..6 {
            let a = fx::TURN * k / 6 + fx::deg(15);
            let d = v3(fx::cos(a), fx::sin(a * 2) / 2 + ONE / 3, fx::sin(a)).norm();
            let at = sp.at.add(d.scale(flew)).sub(v3(0, fx::mul(flew, flew), 0));
            out.glow(
                Mesh::Sphere,
                Affine::translate(at).then(&Affine::scale(v3(m, m, m))),
                hot,
                left,
            );
        }
    }
}

/// The camera's knock now, `alpha` of the way from the last tick to this.
pub fn knock_at(w: &World, alpha: Fx) -> V3 {
    w.feel.prev_knock.lerp(w.feel.knock, alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hard_blow_stops_the_world_two_to_four_ticks_by_its_speed() {
        // under 5 m/s nothing; then 2, a tick more each 4 m/s, at most 4
        assert_eq!(stop_ticks(HARD - 1), 0);
        assert_eq!(stop_ticks(HARD), 2);
        assert_eq!(stop_ticks(fx::int(9)), 3, "2 + 4 / 4");
        assert_eq!(stop_ticks(fx::int(25) / 2), 3, "2 + 7.5 / 4, down");
        assert_eq!(stop_ticks(fx::int(13)), 4, "2 + 8 / 4");
        assert_eq!(stop_ticks(fx::int(30)), 4, "at most 4");
    }

    /// The knock: 6 mm a metre a second (393 units), at most 5 cm (3 276).
    /// It rings back through the middle keeping 55%, truncated, and is
    /// nothing under half a millimetre (32 units): from 3 276, −1 801, 990,
    /// −544, 299, −164, 90, −49, then 26, under 32: nothing.
    #[test]
    fn the_camera_s_knock_rings_back_through_the_middle_to_exactly_nothing() {
        assert_eq!(knock(fx::int(5), v3(ONE, 0, 0)), v3(1965, 0, 0));
        let mut k = knock(fx::int(10), v3(ONE, 0, 0));
        assert_eq!(k, v3(3276, 0, 0), "held to 5 cm");
        let mut seen = vec![k.x];
        while k != V3::ZERO {
            k = ring(k);
            seen.push(k.x);
        }
        assert_eq!(seen, [3276, -1801, 990, -544, 299, -164, 90, -49, 0]);
    }

    /// A world with the player in Practice, facing heading 0 (−z), so the
    /// view's right is +x exactly, and the point moving `per_tick` along it.
    fn swinging(per_tick: Fx) -> World {
        let mut g = crate::game::tests::game();
        crate::game::tests::practice(&mut g);
        let mut w = g.world.clone();
        w.aim.yaw = 0;
        let tip = &mut w.me.arm.pts[crate::arm::TIP];
        tip.q = tip.p.sub(v3(per_tick, 0, 0));
        w
    }

    /// T10: a point crossing the view at 5 m/s (5 461 units a tick, 327 660
    /// a second) leans the eye toward 109 units a metre a second (3° / 5, 546
    /// / 5): 327 660 × 109 >> 16 = 544. It closes 18% of the gap a tick: 97,
    /// then 97 + 447 × 18 / 100 = 177. Stopped, it comes back by 18% a tick
    /// (177 − 31 = 146) and is level exactly once under a twentieth of a
    /// degree (9 units): 120, 99, 82 … 10, 9, then 8 is under it, so 0, on
    /// the 18th tick. (Easing alone would stall at 5, where 18% of it rounds
    /// to nothing.) At 20 m/s it is held to 7° (1 274).
    #[test]
    fn the_eye_leans_with_its_own_cut_and_comes_back_level_exactly() {
        let mut w = swinging(ONE / 12);
        lean(&mut w);
        assert_eq!(w.feel.tilt, 97);
        lean(&mut w);
        assert_eq!((w.feel.prev_tilt, w.feel.tilt), (97, 177));
        let tip = &mut w.me.arm.pts[crate::arm::TIP];
        tip.q = tip.p;
        lean(&mut w);
        assert_eq!(w.feel.tilt, 146);
        let mut ticks = 1;
        while w.feel.tilt != 0 && ticks < 40 {
            lean(&mut w);
            ticks += 1;
            assert!(w.feel.tilt.abs() < 146, "only ever coming back");
        }
        assert_eq!((w.feel.tilt, ticks), (0, 18));
        let mut w = swinging(ONE / 3);
        for _ in 0..60 {
            lean(&mut w);
        }
        assert!(
            w.feel.tilt <= TILT_MAX && w.feel.tilt > TILT_MAX - 20,
            "{}",
            w.feel.tilt
        );
        assert_eq!(TILT_MAX, 1274);
    }

    /// T10: a blow the player takes throws the eye 5° (910 units) away from
    /// the striker and reddens the view's edge by a 25th of full (2 621) for
    /// each point taken; a blow the player lands does neither. The red keeps
    /// 94% a tick: from full, 61 603 then 57 906, and is nothing under a
    /// hundredth (655).
    #[test]
    fn a_blow_taken_tilts_the_eye_away_and_reddens_the_edge_which_fades_to_nothing() {
        let mut w = swinging(0);
        let foe = player::body(&w, 1).pos;
        let blow = |by: u8, on: u8| crate::blade::Hit {
            by,
            on,
            part: crate::blade::Part::Torso,
            seg: 1,
            vel: V3::ZERO,
            speed: fx::int(9),
            at: V3::ZERO,
            from: V3::ZERO,
            through_pct: 100,
            plated: false,
        };
        let taken = crate::health::damage(fx::int(9), crate::blade::Part::Torso);
        assert!(taken > 0 && taken < 25);
        w.blades.hits.push(blow(0, 1));
        step(&mut w);
        assert_eq!(
            (w.feel.tilt, w.feel.hurt),
            (0, 0),
            "a blow landed is not one taken"
        );
        w.blades.hits.clear();
        w.blades.hits.push(blow(1, 0));
        step(&mut w);
        // thrown from the foe toward the player: right of the view if the
        // foe stands to the left
        let side = if w.player.pos.sub(foe).x >= 0 { 1 } else { -1 };
        // the throw, then the tick's lean back toward level (18% of it)
        assert_eq!(w.feel.tilt, side * (910 - 910 * 18 / 100));
        assert_eq!(w.feel.hurt, 2621 * taken * 94 / 100);
        w.feel.hurt = ONE;
        w.blades.hits.clear();
        lean(&mut w);
        assert_eq!(w.feel.hurt, 61_603);
        lean(&mut w);
        assert_eq!(w.feel.hurt, 57_906);
        // under a hundredth (655) on the 75th tick from full, so by the
        // 80th it is nothing; kept at 94% alone it would still be 455
        for _ in 0..78 {
            lean(&mut w);
        }
        assert_eq!(w.feel.hurt, 0);
    }

    /// T10: blades meeting at 6 m/s (over `HARD`, 5) jolt the eye 3° (546
    /// units) away from the other fighter, then the tick's lean takes 18%
    /// of it back (546 − 98 = 448); at 4 m/s, nothing.
    #[test]
    fn blades_meeting_hard_jolt_the_eye_away_from_the_other() {
        let mut w = swinging(0);
        let foe = player::body(&w, 1).pos;
        let side = if w.player.pos.sub(foe).x >= 0 { 1 } else { -1 };
        let meet = |speed: Fx| crate::blade::Clash {
            a: 0,
            b: 1,
            at: V3::ZERO,
            speed,
        };
        w.blades.clashes.push(meet(fx::int(4)));
        step(&mut w);
        assert_eq!(w.feel.tilt, 0, "under HARD");
        w.blades.clashes.clear();
        w.blades.clashes.push(meet(fx::int(6)));
        step(&mut w);
        assert_eq!(w.feel.tilt, side * 448);
    }

    /// T10: a blade that draws blood keeps it for the round, a 60th of full
    /// (1 092) for each point it takes, at most 9 / 20 (29 491), and it is
    /// stippled over the outer blade in the dark blood colour; with blood
    /// off in the settings, none is drawn. The struck blade stays clean.
    #[test]
    fn a_blade_that_draws_blood_keeps_it_and_shows_it() {
        let mut w = swinging(0);
        let taken = crate::health::damage(fx::int(9), crate::blade::Part::Torso);
        w.blades.hits.push(crate::blade::Hit {
            by: 0,
            on: 1,
            part: crate::blade::Part::Torso,
            seg: 1,
            vel: V3::ZERO,
            speed: fx::int(9),
            at: V3::ZERO,
            from: V3::ZERO,
            through_pct: 100,
            plated: false,
        });
        step(&mut w);
        assert_eq!(w.feel.blooded[0], 1092 * taken);
        assert_eq!(
            w.feel.blooded.get(1).copied().unwrap_or(0),
            0,
            "the struck blade is clean"
        );
        for _ in 0..40 {
            step(&mut w);
        }
        assert_eq!(w.feel.blooded[0], 29_491, "held to 9 / 20");
        let dark = rgb(crate::wounds::spec().colours.blood_dark);
        let drawn = |w: &World| {
            let mut out = Shapes::default();
            player::draw(w, ONE, &crate::look::tests::look(), &mut out);
            out.list
                .iter()
                .filter(|s| s.pass == crate::render::Pass::Dither && s.rgb == dark)
                .count()
        };
        assert_eq!(drawn(&w), 1);
        w.settings.blood = false;
        assert_eq!(drawn(&w), 0, "blood off");
    }

    /// M7: a spark burns 20 ticks, a third of a second: there for 19 ticks
    /// after it is thrown, gone on the 20th; 12.5 cm and 2.5 cm a metre a
    /// second: at 6 m/s, 8 192 + 6 × 1 638 (a fortieth, truncated) = 18 020
    /// units, 27.5 cm; from 8.4 m/s on, a third of a metre.
    #[test]
    fn a_spark_burns_a_third_of_a_second_and_is_sized_by_the_meeting() {
        assert_eq!(spark_size(fx::int(6)), 18_020);
        assert_eq!(spark_size(fx::int(9)), ONE / 3);
        let mut w = World::new(1);
        w.feel.sparks.push(Spark {
            at: V3::ZERO,
            age: 0,
            size: ONE / 4,
        });
        for _ in 0..19 {
            ring_down(&mut w);
        }
        assert_eq!(w.feel.sparks.len(), 1, "still burning after 19 ticks");
        ring_down(&mut w);
        assert!(w.feel.sparks.is_empty(), "gone on the 20th");
    }
}
