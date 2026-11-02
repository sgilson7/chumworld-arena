//! `motion`: the feet (M1 Duel, task D). Walking, running, footwork with the
//! sword drawn, the speedster's ice (CW-8), a short dodge-step, the facing
//! rule, and the stepping gait that fills a `Pose`'s legs. Steps at 3 in
//! `World::step`, from each fighter's `Input` record alone (CW-3).
//!
//! **Grip** is the share of the gap between the velocity a fighter has and
//! the velocity its keys ask for that its feet close each tick. Bare feet
//! close a third of it (`GRIP_BASE`): a walk starts and stops in a few
//! ticks. Each speedster piece multiplies grip by 55/100 and the speed by
//! 1 + 0.2 (journal, "The speedster's armour"; CW-8), so five pieces run at
//! twice the speed on a twentieth of the grip and slide fourteen metres to a
//! stop. The values for 0 to 5 pieces are pinned in the tests below, worked
//! out by hand.
//!
//! **Facing.** Sheathed, a body turns toward where it moves. Drawn, it turns
//! toward the foe it holds on (`Fighter::engaged`), or, holding on no one,
//! the way it looks.
//!
//! How a module joins the world (state, step, draw, scene, screen, act):
//! the extension contract in `world.rs`.

use crate::blade;
use crate::fx::{self, deg, Fx, ONE};
use crate::geom::{facing, right_of, v3, yaw_of, V3};
use crate::input::{self, Input};
use crate::player::{self, Body, Fighter};
use crate::pose::{Build, Kind};
use crate::ticks::TICKS_PER_SECOND;
use crate::world::World;

const TICKS: i32 = TICKS_PER_SECOND as i32;

/// Walking and running speeds, metres a second: a brisk walk and a quick
/// run (the person, 2026-10-20: "your overall movement speed should be
/// faster"; PLAN.md's 2.2 and 5 m/s before, SECOND-ORDER.md R9.40).
pub const WALK: Fx = fx::int(3);
pub const RUN: Fx = fx::int(7);
/// Footwork with the sword drawn, whatever Shift does: M1's duelling pace,
/// 2.2 m/s, which the pilots' reach bands are tuned to (R9.40).
pub const FOOTWORK: Fx = ONE * 22 / 10;
/// On a road, lane or trail a sheathed body goes this much faster, eased
/// in and out over `ROAD_EASE` ticks as it steps on or off (R9.41).
pub const ROAD_PCT: i32 = 18;
pub const ROAD_EASE: i32 = 30;
/// Bare feet's grip: a third of the gap closed each tick.
pub const GRIP_BASE: Fx = ONE / 3;
/// The most pieces there are.
pub const PIECES: u8 = 5;
/// The most a body turns in a tick toward where it should face (600° a
/// second sheathed, 360° drawn: a duellist turns more deliberately).
pub const TURN_RATE: i32 = deg(10);
pub const TURN_RATE_DRAWN: i32 = deg(6);
/// The dodge-step (Space): this fast for this many ticks, then a wait.
pub const DODGE_SPEED: Fx = fx::int(6);
pub const DODGE_TICKS: u8 = 8;
pub const DODGE_WAIT: u8 = 40;
/// Held on something (a foe, or the vine), the dodge-step lasts half as
/// long again, so goes half as far again (CW-Q19's default, T21).
pub const DODGE_HELD_TICKS: u8 = DODGE_TICKS * 3 / 2;
/// The gait: a stride this long at a walk, longer with speed; feet lift this
/// high at a walk.
pub const STRIDE: Fx = ONE * 62 / 100;
pub const STRIDE_PER_MPS: Fx = ONE * 9 / 100;
pub const LIFT: Fx = ONE * 11 / 100;

/// Settings that are not any one body's. CHUMWORLD kept the speedster
/// pieces forced for a test here; the ring has no speedster (R2.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {}

/// One body's stepping gait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Gait {
    /// where in the stride cycle the feet are (angle units: a full turn is
    /// two steps)
    pub phase: i32,
    /// how far the feet swing, `ONE` at a walk, eased toward the speed
    pub swing: Fx,
    /// ticks left in a dodge, and before the next may start
    pub dodge: u8,
    pub dodge_wait: u8,
    pub dodge_way: V3,
    /// how much faster the body slides than its feet step, metres a second,
    /// eased: a sliding body leans back and spreads its arms
    pub slide: Fx,
    /// ticks left of a stagger from a blow (`feel`), and the way it was
    /// knocked, across the ground: the feet barely answer the keys, the
    /// body leans back along it, the arms lose half their strength
    pub stagger: u8,
    pub stagger_way: V3,
    /// how far onto a road the body is, 0 to ONE, eased over `ROAD_EASE`
    /// ticks as it steps on or off (`road_share`)
    pub road: Fx,
    /// M7 (R9.177): how far a thrust has the body lunging forward, `ONE`
    /// fully (`swing::LUNGE_LEAN`), set by the cut, eased in and out
    pub lunge: Fx,
    /// M7 (P4): bent down to take up a weapon, `ONE` fully (`wounds::bend`);
    /// a hop on one leg: ticks into the hop, and how high the body is off
    /// the ground
    pub stoop: Fx,
    pub hop: u16,
    pub lift: Fx,
    /// T21 (phase A): off its feet, and how fast it rises (+) or falls (−),
    /// a tick (`jump.rs`)
    pub air: bool,
    pub vy: Fx,
    /// TE1: the highest the feet have been since they left the ground, and
    /// on the tick they land, how far they came down from it (`jump.rs`)
    pub peak: Fx,
    pub fell: Fx,
    /// T37: on the ground, how fast the land under the feet carried them up
    /// last tick (a crest or a ramp's lip throws them on at it)
    pub carry: Fx,
}

/// T21: in the air a body keeps this share of its feet's grip (a sixth),
/// so a hop goes where it was going (`jump.rs`).
pub const AIR_GRIP_DIV: Fx = 6;

/// The road's share moved one tick toward on (ONE) or off (0).
pub fn road_share(share: Fx, on: bool) -> Fx {
    let goal = if on { ONE } else { 0 };
    fx::approach(share, goal, (ONE + ROAD_EASE - 1) / ROAD_EASE)
}

/// A speed asked for, with a road's share of the road's bonus.
pub fn on_road(speed: Fx, share: Fx) -> Fx {
    speed + fx::mul(fx::mul(speed, ONE * ROAD_PCT / 100), share)
}

/// Grip: bare feet's `GRIP_BASE`. (CHUMWORLD took 0.55 of it for each
/// speedster piece worn, CW-8; the ring has none, so `n` is always 0 and
/// its share was exactly `ONE`, R2.4.)
pub fn grip(_n: u8) -> Fx {
    GRIP_BASE
}

/// A speed in metres a second as metres a tick. (CHUMWORLD multiplied it by
/// 1 + 0.2 for each speedster piece first; with none that was `ONE`, R2.4.)
pub fn per_tick(speed: Fx, _n: u8) -> Fx {
    speed / TICKS
}

/// How many speedster pieces the player wears: none in the ring (R2.4).
pub fn pieces(_w: &World) -> u8 {
    0
}

/// `vel` moved `grip` of the way toward `want`, at least one unit a tick
/// on each axis that is not there yet, so it always arrives.
pub fn approach(vel: V3, want: V3, grip: Fx) -> V3 {
    let one = |v: i32, w: i32| {
        let d = w - v;
        let c = fx::mul(d, grip);
        v + if c == 0 { d.signum() } else { c }
    };
    v3(one(vel.x, want.x), one(vel.y, want.y), one(vel.z, want.z))
}

/// The velocity a body's keys ask for: the movement keys turned by `yaw`
/// (the camera's heading for the player, the way it looks for a foe).
pub fn wanted(input: &Input, yaw: i32, drawn: bool, n: u8) -> V3 {
    let (x, z) = input.walk_axes();
    let way = right_of(yaw)
        .scale(x * ONE)
        .sub(facing(yaw, 0).scale(z * ONE))
        .norm();
    let speed = if drawn {
        FOOTWORK
    } else if input.key(input::RUN) {
        RUN
    } else {
        WALK
    };
    way.scale(per_tick(speed, n))
}

/// One tick of every body's feet.
pub fn step(w: &mut World, input: &Input, foes: &[Input]) {
    let n_player = pieces(w);
    for id in 0..player::count(w) {
        let inp = if id == 0 { *input } else { foes[id - 1] };
        let yaw = player::look_yaw(w, id);
        let n = if id == 0 { n_player } else { 0 };
        let toward = player::held_at(w, id);
        // (the ring has no road, R2.2)
        let (ground, road) = {
            let b = player::body(w, id);
            (w.ground(b.pos.x, b.pos.z), false)
        };
        let fp = id == 0 && w.first_person();
        let (b, f) = player::parts_mut(w, id);
        f.gait.road = road_share(f.gait.road, road);
        step_body(b, f, &inp, yaw, n, toward, ground);
        // M1: in first person the body faces where the view does
        if fp && !f.health.down() {
            b.facing = yaw;
        }
    }
    separate(w);
}

/// One body's tick: speed, grip, the dodge, facing and the gait.
pub fn step_body(
    b: &mut Body,
    f: &mut Fighter,
    input: &Input,
    yaw: i32,
    n: u8,
    toward: Option<V3>,
    ground: Fx,
) {
    b.prev_pos = b.pos;
    b.prev_facing = b.facing;
    b.prev_vel = b.vel;
    // (M7: a fighter whose weapon has dropped moves as one with it put away)
    let drawn = f.drawn() && !f.health.limbs.loose;
    let down = f.health.down();
    let mut want = if down {
        V3::ZERO
    } else {
        wanted(input, yaw, drawn, n)
    };
    // sheathed, a road's share of the road's bonus (drawn, footwork holds)
    if !drawn && f.gait.road > 0 {
        let s = on_road(want.len(), f.gait.road);
        want = want.norm().scale(s);
    }
    // staggered, the feet answer the keys a quarter
    if f.gait.stagger > 0 {
        want = v3(want.x / 4, want.y / 4, want.z / 4);
        f.gait.stagger -= 1;
    }
    // the dodge-step: Space with a direction, once the last has worn off
    let dodge = !down && !f.gait.air && f.gait.stagger == 0 && f.pressed(input, input::DODGE);
    let g = &mut f.gait;
    g.dodge_wait = g.dodge_wait.saturating_sub(1);
    if dodge && g.dodge_wait == 0 && want != V3::ZERO {
        g.dodge = if toward.is_some() {
            DODGE_HELD_TICKS
        } else {
            DODGE_TICKS
        };
        g.dodge_wait = DODGE_WAIT;
        g.dodge_way = want.norm();
    }
    if g.dodge > 0 {
        g.dodge -= 1;
        want = g.dodge_way.scale(per_tick(DODGE_SPEED, n));
    }
    // M7 (P4): toppled, hopping on one leg or crawling with none
    want = crate::wounds::gait_want(f, want, n);
    // T21: in the air, a sixth of the grip
    let held = if f.gait.air {
        grip(n) / AIR_GRIP_DIV
    } else {
        grip(n)
    };
    b.vel = approach(b.vel, want, held);
    if b.vel.len() <= 1 {
        b.vel = V3::ZERO;
    }
    b.pos = b.pos.add(b.vel);
    b.pos.y = ground;
    // facing
    let goal = if down {
        None
    } else if drawn {
        Some(match toward {
            Some(t) if t.sub(b.pos).len_xz() > ONE / 10 => yaw_of(t.sub(b.pos)),
            _ => yaw,
        })
    } else if b.vel != V3::ZERO && want != V3::ZERO {
        Some(yaw_of(b.vel))
    } else {
        None
    };
    if let Some(goal) = goal {
        let rate = if drawn { TURN_RATE_DRAWN } else { TURN_RATE };
        let turn = fx::wrap(goal - b.facing).clamp(-rate, rate);
        b.facing = fx::wrap(b.facing + turn);
    }
    // feet stride as fast as they are asked to go, at most as fast as the
    // body goes: a body sliding on ice with nothing held glides on still feet
    let asked = if want.len() < b.vel.len() { want } else { b.vel };
    stride(&mut f.gait, asked, b.vel);
    hop(&mut f.gait, f.health.limbs.hopping(), b.vel);
}

/// M7 (P4): the limping gait on one leg. Moving, the body hops: each hop
/// lasts `hop_ticks`, lifting it up to `hop_mm` and down again; standing, it
/// finishes the hop it is in and stays down. (Its speed is a hop's share of
/// a walk, `wounds::gait_want`; the standing leg is set under the body in
/// `wounds::shape_pose`.)
pub fn hop(g: &mut Gait, hopping: bool, vel: V3) {
    let sp = crate::wounds::spec();
    if !hopping {
        g.hop = 0;
        g.lift = 0;
        return;
    }
    let n = sp.hop_ticks;
    if g.hop > 0 || vel.len_xz() > 0 {
        g.hop = (g.hop + 1) % n;
    }
    // half a sine over the hop, in the air two thirds of it
    let air = n * 2 / 3;
    g.lift = if g.hop < air {
        let a = g.hop as i32 * fx::HALF_TURN / air as i32;
        (sp.hop_mm as i64 * ONE as i64 / 1000 * fx::sin(a) as i64 / ONE as i64) as Fx
    } else {
        0
    };
}

/// The gait follows the distance the feet cover: half a turn of phase per
/// stride, the swing eased toward how fast the body goes.
pub fn stride(g: &mut Gait, vel: V3, moving: V3) {
    // how much faster the body goes than its feet: a slide, eased
    let slide = (moving.len_xz() - vel.len_xz()).max(0) * TICKS;
    g.slide = fx::approach(g.slide, slide.min(fx::int(10)), ONE / 4);
    let speed = vel.len_xz() as i64 * TICKS as i64;
    let len = STRIDE + fx::mul(speed as Fx, STRIDE_PER_MPS);
    g.phase = fx::wrap(g.phase + (vel.len_xz() as i64 * fx::HALF_TURN as i64 / len as i64) as i32);
    // the swing is ONE at 2.2 m/s, M1's walk, which the duel's poses were
    // tuned to (R9.40)
    let goal = (speed * ONE as i64 / FOOTWORK as i64).min(ONE as i64 * 3 / 2) as Fx;
    g.swing = fx::approach(g.swing, goal, ONE / 8);
}

/// Bodies this far apart, feet to feet, cannot touch (two orcs' arms and
/// legs reach about 1.4 m between them).
pub const APART: Fx = fx::int(3);
/// The separation is solved this many times a tick, so three bodies in a
/// knot all come apart (M7: 4, was 3; at the 3 m/s walk three let a walker
/// slide round a held guard, R9.42).
pub const SEPARATE_PASSES: usize = 4;

/// A body's weight, kilograms: who gives way to whom when two press
/// together (`separate`).
pub fn mass(kind: Kind) -> i32 {
    match kind {
        Kind::Orc => 130,
        Kind::Leader => 85,
        Kind::AntKing => 170,
        Kind::Merchant => 85,
        Kind::Bandit => 75,
        Kind::Player | Kind::Trainer => 70,
        Kind::Villager => 65,
        Kind::Lookout => 60,
        // M3 (R5.1): a knight and some 25 kg of plate
        Kind::KnightA | Kind::KnightB => 110,
        // M6 (R7.103): padding, a brigandine, half plate
        Kind::Squire => 80,
        Kind::Sellsword => 90,
        Kind::Hedge => 100,
        // M6 (R7.201): a big man in leather and a little plate; a knight
        Kind::Veteran => 95,
        Kind::Banneret => 108,
        // M6 (R7.301): the champion and the mirror in full plate, as a knight
        Kind::Champion | Kind::Mirror => 110,
    }
}

/// Bodies whose trunks and legs (`blade::TRUNK`) sink into each other are
/// pushed apart across the ground, out of the deepest pair, as one exact
/// pair shift weighted by mass: the lighter gives more way, and the two
/// together keep where their centre of mass was and their momentum, to the
/// unit (`k·m_b` one way, `k·m_a` the other, as `arm::Arm::stick` does).
/// The speed at which they closed is taken out the same way, so they do
/// not bounce. A body that is down is a weight nothing standing can shift:
/// the one standing gives all the way. Each body moves through its own
/// position and velocity; no pilot touches it (CW-3).
pub fn separate(w: &mut World) {
    let n = player::count(w);
    for _ in 0..SEPARATE_PASSES {
        let mut caps: Vec<[blade::Capsule; blade::PARTS]> = (0..n)
            .map(|id| blade::body(player::fighter(w, id).kind, &player::pose_of(w, id)))
            .collect();
        let mut held: Vec<Vec<(V3, V3, Fx)>> = (0..n).map(|id| held_rails(player::fighter(w, id))).collect();
        for i in 0..n {
            for j in i + 1..n {
                let (a, b) = (player::body(w, i).pos, player::body(w, j).pos);
                if v3(b.x - a.x, 0, b.z - a.z).len() > APART {
                    continue;
                }
                let Some(push) = deepest(&caps[i], &caps[j], a, b, [&held[i], &held[j]]) else {
                    continue;
                };
                let (fi, fj) = (player::fighter(w, i), player::fighter(w, j));
                let (mi, mj) = match (fi.health.down(), fj.health.down()) {
                    (true, false) => (1, 0),
                    (false, true) => (0, 1),
                    _ => (mass(fi.kind), mass(fj.kind)),
                };
                let (di, dj) = pair_shift(push, mi, mj, true);
                let (vi, vj) = (player::body(w, i).vel, player::body(w, j).vel);
                // the speed they close at along the push, taken out
                let along = push.norm();
                let closing = vj.sub(vi).dot(along);
                let (ui, uj) = if closing < 0 {
                    pair_shift(along.scale(-closing), mi, mj, false)
                } else {
                    (V3::ZERO, V3::ZERO)
                };
                for (id, d, u) in [(i, di, ui), (j, dj, uj)] {
                    let body = player::parts_mut(w, id).0;
                    body.pos = body.pos.add(d);
                    body.vel = body.vel.add(u);
                    for c in caps[id].iter_mut() {
                        c.a = c.a.add(d);
                        c.b = c.b.add(d);
                    }
                    for r in held[id].iter_mut() {
                        r.0 = r.0.add(d);
                        r.1 = r.1.add(d);
                    }
                }
            }
        }
    }
}

/// A fighter's weapon's rails while a hand holds it (`arm::rails`), where
/// its arm meant it to be (`arm::Arm::meant`): a body cannot walk into a
/// blade held at it.
pub fn held_rails(f: &Fighter) -> Vec<(V3, V3, Fx)> {
    let a = &f.arm;
    if !a.in_hand || a.riding || !(a.hold[0] || a.hold[1]) || f.health.down() {
        return Vec::new();
    }
    a.rails_along(a.meant.0, a.meant.1)
        .into_iter()
        .map(|(p, q, r)| (p, q, r + crate::arm::SKIN))
        .collect()
}

/// A weapon pressing on a body shoves the two apart by at most this a tick
/// (6 cm: more than a walk's 5 cm at 3 m/s, so a guard stops a body walking
/// into it, but a swing into a body only shoves it, as the stagger does;
/// M1's 5 cm against its 2.2 m/s walk's 3.7, R9.40).
pub const BLADE_SHOVE: Fx = ONE * 6 / 100;

/// The push that parts two bodies: across the ground, from the first toward
/// the second, far enough that the pair that needs it most no longer
/// touches; none when they do not touch. The pairs: trunk and legs against
/// trunk and legs, and each one's held weapon (`weapons`) against the
/// other's head and torso, so a guard held at a foe keeps them off it and
/// a blade rests on no face. Two capsules whose nearest points are `dy`
/// apart up and `h` across clear at `√((r₁ + r₂)² − dy²)` across, so a head
/// against a taller body's chest needs more than its sink.
pub fn deepest(
    x: &[blade::Capsule; blade::PARTS],
    y: &[blade::Capsule; blade::PARTS],
    fx: V3,
    fy: V3,
    weapons: [&[(V3, V3, Fx)]; 2],
) -> Option<V3> {
    let mut worst: Option<(Fx, V3)> = None;
    let mut consider = |p: V3, q: V3, r: Fx, most: Fx| {
        let d = q.sub(p);
        if d.len() >= r {
            return;
        }
        let r = r as i64;
        let clear = fx::isqrt_u64((r * r - d.y as i64 * d.y as i64).max(0) as u64) as Fx;
        let need = (clear - d.len_xz()).min(most);
        if need > 0 && worst.is_none_or(|(w, _)| need > w) {
            worst = Some((need, d));
        }
    };
    for &i in &blade::TRUNK {
        for &j in &blade::TRUNK {
            let (_, _, p, q) = blade::closest(x[i].a, x[i].b, y[j].a, y[j].b);
            consider(p, q, x[i].r + y[j].r, Fx::MAX);
        }
    }
    // a weapon of the first against the second's head and torso, and the
    // second's against the first's (its way turned to run first to second)
    for k in [0, 1] {
        for &(a, b, r) in weapons[0] {
            let (_, _, p, q) = blade::closest(a, b, y[k].a, y[k].b);
            consider(p, q, r + y[k].r, BLADE_SHOVE);
        }
        for &(a, b, r) in weapons[1] {
            let (_, _, p, q) = blade::closest(x[k].a, x[k].b, a, b);
            consider(p, q, r + x[k].r, BLADE_SHOVE);
        }
    }
    let (need, d) = worst?;
    // across the ground: the nearest points' way, or with one above the
    // other the feet's, or with those together due east
    let flat = [v3(d.x, 0, d.z), v3(fy.x - fx.x, 0, fy.z - fx.z), v3(ONE, 0, 0)]
        .into_iter()
        .find(|v| v.len() > 0)
        .unwrap_or(v3(ONE, 0, 0));
    Some(flat.norm().scale(need))
}

/// `d` shared between two masses as an exact pair shift: the first moves
/// `−k·m_b`, the second `+k·m_a`, with `k = d / (m_a + m_b)` on each axis,
/// so `m_a·Δa + m_b·Δb` is zero to the unit. A mass of 0 does not move.
/// `whole` rounds `k` away from zero, so the two move at least `d` apart
/// (a push); otherwise toward it, so they never gain speed (a stop).
pub fn pair_shift(d: V3, ma: i32, mb: i32, whole: bool) -> (V3, V3) {
    let m = (ma + mb).max(1);
    let share = |c: i32| {
        if whole {
            (c + c.signum() * (m - 1)) / m
        } else {
            c / m
        }
    };
    let k = v3(share(d.x), share(d.y), share(d.z));
    (
        v3(-k.x * mb, -k.y * mb, -k.z * mb),
        v3(k.x * ma, k.y * ma, k.z * ma),
    )
}

/// Where each foot is: under its hip, swung along the way the body moves
/// and lifted in the half of the cycle it is off the ground.
pub fn feet(b: &Body, g: &Gait, build: &Build, feet_at: V3) -> [V3; 2] {
    let right = right_of(b.facing);
    let way = if b.vel.len_xz() > 0 {
        v3(b.vel.x, 0, b.vel.z).norm()
    } else {
        facing(b.facing, 0)
    };
    let half = fx::mul(STRIDE / 2, g.swing);
    let mut out = [V3::ZERO; 2];
    for (s, foot) in out.iter_mut().enumerate() {
        let side = if s == 0 { -1 } else { 1 };
        let ph = g.phase.wrapping_add(if s == 0 { 0 } else { fx::HALF_TURN });
        let along = fx::mul(fx::sin(ph), half);
        let lift = fx::mul(fx::cos(ph).max(0), fx::mul(LIFT, g.swing.min(ONE)));
        *foot = feet_at
            .add(right.scale(side * build.hip_x))
            .add(way.scale(along))
            .add(v3(0, lift, 0));
    }
    out
}

/// The knee between a hip and a foot, bent toward `fwd`: the two-bone
/// answer, the thigh `a` and shin `c` long.
pub fn knee(hip: V3, foot: V3, a: Fx, c: Fx, fwd: V3) -> V3 {
    let span = foot.sub(hip);
    let d = span.len().clamp(ONE / 20, a + c - ONE / 1000);
    let u = span.norm();
    // along the hip-to-foot line: (a² − c² + d²) / 2d
    let x = fx::div(fx::mul(a, a) - fx::mul(c, c) + fx::mul(d, d), 2 * d);
    let h = fx::sqrt(fx::mul(a, a) - fx::mul(x, x));
    let bend = fwd.sub(u.scale(fwd.dot(u))).norm();
    hip.add(u.scale(x)).add(bend.scale(h))
}

/// `act("motion.<name>", value)`. CHUMWORLD's only one, `speedster`, is
/// left behind with the speedster (R2.4).
pub fn act(_w: &mut World, name: &str, _value: &str) -> Result<(), String> {
    Err(format!("Unknown action \"motion.{name}\"."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{FORWARD, RUN as RUN_KEY};
    use crate::player::Fighter;
    use crate::pose::Kind;

    fn hold(keys: u32) -> Input {
        Input {
            keys,
            ..Input::default()
        }
    }

    fn fighter() -> Fighter {
        Fighter::new(Kind::Player, crate::pose::Held::ShortSword, 0, V3::ZERO, 0, false)
    }

    #[test]
    fn a_walk_starts_in_a_few_ticks_and_holding_w_walks_north() {
        let mut b = Body::new(V3::ZERO, 0);
        let mut f = fighter();
        // 3 m/s is 3 276 a tick (196 608 / 60); the gap shrinks to ⅔ a
        // tick: after 10 ticks (⅔)¹⁰ = 1.7% is left, 57 units
        for _ in 0..10 {
            step_body(&mut b, &mut f, &hold(FORWARD), 0, 0, None, 0);
        }
        assert!((b.vel.z + 3276).abs() <= 60, "{}", b.vel.z);
        assert_eq!(b.vel.x, 0);
        for _ in 0..30 {
            step_body(&mut b, &mut f, &hold(FORWARD), 0, 0, None, 0);
        }
        assert_eq!(b.vel.z, -3276);
    }

    #[test]
    fn shift_runs_only_while_the_sword_is_sheathed() {
        let n = 0;
        assert_eq!(wanted(&hold(FORWARD | RUN_KEY), 0, false, n).z, -7645);
        assert_eq!(
            wanted(&hold(FORWARD | RUN_KEY), 0, true, n).z,
            -2402,
            "drawn: footwork"
        );
    }

    #[test]
    fn movement_is_relative_to_the_heading_given() {
        // heading a quarter turn left faces west: W walks west (−x)
        let v = wanted(&hold(FORWARD), fx::QUARTER_TURN, false, 0);
        assert!(v.x < -2300 && v.z.abs() <= 2, "{v:?}");
    }

    #[test]
    fn sheathed_a_body_turns_to_its_way_and_drawn_it_turns_to_its_foe() {
        let mut b = Body::new(V3::ZERO, 0);
        let mut f = fighter();
        for _ in 0..40 {
            step_body(&mut b, &mut f, &hold(crate::input::LEFT), 0, 0, None, 0);
        }
        assert_eq!(b.facing, fx::QUARTER_TURN, "walking west faces west");
        // drawn, a foe due east: it turns there at 6° a tick, walking or not
        f.sheath = crate::player::Sheath::Drawn;
        let east = v3(fx::int(5), 0, 0);
        for _ in 0..40 {
            step_body(&mut b, &mut f, &Input::default(), 0, 0, Some(east), 0);
        }
        let want = yaw_of(east.sub(b.pos));
        assert!(fx::wrap(b.facing - want).abs() <= deg(1), "{} {want}", b.facing);
    }

    #[test]
    fn a_dodge_steps_aside_once_then_waits() {
        let mut b = Body::new(V3::ZERO, 0);
        let mut f = fighter();
        let side = Input {
            keys: crate::input::RIGHT | crate::input::DODGE,
            ..Input::default()
        };
        step_body(&mut b, &mut f, &side, 0, 0, None, 0);
        f.last = side;
        for _ in 0..DODGE_TICKS {
            step_body(&mut b, &mut f, &hold(crate::input::RIGHT), 0, 0, None, 0);
        }
        // 6 m/s for 8 ticks with a third of the gap closed each tick: more
        // than half a metre east, against 2.2 m/s walking's 0.25 m
        assert!(b.pos.x > ONE / 2, "{}", b.pos.x);
        // set to 40 on the tick it starts, then one less each tick after
        assert_eq!(f.gait.dodge_wait, DODGE_WAIT - DODGE_TICKS);
    }

    #[test]
    fn the_knee_meets_both_bones_and_bends_forward() {
        // the foot 0.66 m from the hip, inside the 0.72 m both bones reach
        let (hip, foot) = (v3(0, fx::int(1), 0), v3(0, ONE * 35 / 100, -ONE / 10));
        let (a, c) = (ONE * 37 / 100, ONE * 35 / 100);
        let k = knee(hip, foot, a, c, v3(0, 0, -ONE));
        assert!((k.dist(hip) - a).abs() < ONE / 200, "{}", k.dist(hip));
        assert!((k.dist(foot) - c).abs() < ONE / 200, "{}", k.dist(foot));
        assert!(k.z < foot.z.min(hip.z) + ONE / 100, "the knee is forward: {k:?}");
    }

    /// The player at the test duels' spot (level ground in the village,
    /// `player::duel_spot`) facing north, an orc 0.3 m ahead facing them
    /// with its cleaver put away (a blade held at the player would keep
    /// them off, R3.46: this is trunk against trunk), both upright, walking
    /// into each other at 2.2 m/s.
    fn pressed() -> World {
        let mut w = World::new(1);
        crate::player::act(&mut w, "pilot", "still").unwrap();
        crate::player::act(&mut w, "spawn", "orc").unwrap();
        let base = crate::player::duel_spot();
        w.player = Body::new(base, 0);
        w.aim.yaw = 0;
        w.me = Fighter::new(Kind::Player, crate::pose::Held::ShortSword, 0, base, 0, false);
        let at = base.add(v3(0, 0, -ONE * 3 / 10));
        w.foes[0].body = Body::new(at, fx::HALF_TURN);
        let orc = &mut w.foes[0].fighter;
        *orc = Fighter::new(Kind::Orc, crate::pose::Held::Cleaver, 1, at, fx::HALF_TURN, false);
        w.player.vel = v3(0, 0, -2402);
        w.foes[0].body.vel = v3(0, 0, 2402);
        w
    }

    /// By hand: the player's head is 192 mm × 1.70 / 1.70 = 12 582 units
    /// round at 97 649 up; the orc's torso 145 × 2.00 / 1.70 × 135% = 15 091
    /// round, its top (the chest) at 89 051, 0.3 m (19 660) ahead. The head
    /// sits 8 598 above it and needs √(27 673² − 8 598²) = 26 303 across,
    /// 6 643 more (the torsos, 24 593 round, need only 4 933). Shared
    /// 70 : 130, k = 6 643 / 200 rounded up = 34: the player steps back
    /// 34 × 130 = 4 420, the orc 34 × 70 = 2 380. Closing at 4 804 a tick, k = 4 804 / 200 = 24 (down): the
    /// player's −2 402 + 24 × 130 = 718, the orc's 2 402 − 24 × 70 = 722.
    /// Momentum 70 × −2 402 + 130 × 2 402 = 144 120 before and 70 × 718 +
    /// 130 × 722 = 144 120 after.
    #[test]
    fn two_bodies_pressed_together_part_by_mass_and_keep_their_momentum() {
        let mut w = pressed();
        let caps = |w: &World, id| blade::body(player::fighter(w, id).kind, &player::pose_of(w, id));
        let push = deepest(
            &caps(&w, 0),
            &caps(&w, 1),
            w.player.pos,
            w.foes[0].body.pos,
            [&[], &[]],
        );
        assert_eq!(push, Some(v3(0, 0, -6643)));
        separate(&mut w);
        let base = crate::player::duel_spot();
        assert_eq!(w.player.pos, base.add(v3(0, 0, 4420)));
        assert_eq!(w.foes[0].body.pos, base.add(v3(0, 0, -ONE * 3 / 10 - 2380)));
        assert_eq!(w.player.vel, v3(0, 0, 718));
        assert_eq!(w.foes[0].body.vel, v3(0, 0, 722));
        let momentum = |w: &World| 70 * w.player.vel.z as i64 + 130 * w.foes[0].body.vel.z as i64;
        assert_eq!(momentum(&w), 144_120);
        // apart: one more pass finds nothing to push
        let before = (w.player.pos, w.foes[0].body.pos);
        separate(&mut w);
        assert_eq!((w.player.pos, w.foes[0].body.pos), before);
    }

    /// A guard keeps a foe off: the player walks for three seconds at a
    /// still orc 1.6 m off whose cleaver is held level across the line to the
    /// player's chest (its hand 40° down and 10° in), and is held off by the
    /// blade (it reaches 1.4 m),
    /// never nearer than a metre, where without it they would walk the
    /// blade aside and stop only trunk to trunk, 0.4 m apart.
    #[test]
    fn a_body_cannot_walk_into_a_blade_held_at_it() {
        let mut w = World::new(1);
        crate::player::act(&mut w, "pilot", "still").unwrap();
        crate::player::act(&mut w, "spawn", "orc").unwrap();
        crate::player::act(&mut w, "gap", "160").unwrap();
        w.foes[0].fighter.arm.shell.pitch = -deg(40);
        w.foes[0].fighter.arm.shell.side = -deg(10);
        for _ in 0..60 {
            w.step(&Input::default());
        }
        let mut nearest = fx::int(9);
        for _ in 0..180 {
            w.step(&hold(FORWARD));
            nearest = nearest.min(w.foes[0].body.pos.sub(w.player.pos).len_xz());
        }
        assert!(nearest > ONE, "walked to {nearest} of the orc");
    }

    /// The deepest any two bodies' trunks and legs sink into each other.
    pub fn worst_overlap(w: &World) -> Fx {
        let n = player::count(w);
        let caps: Vec<_> = (0..n)
            .map(|id| blade::body(player::fighter(w, id).kind, &player::pose_of(w, id)))
            .collect();
        let mut worst = 0;
        for i in 0..n {
            for j in i + 1..n {
                for &a in &blade::TRUNK {
                    for &b in &blade::TRUNK {
                        worst = worst.max(blade::sunk(&caps[i][a], &caps[j][b]).0);
                    }
                }
            }
        }
        worst
    }

    /// Over a shove (the player walking into a still orc for three
    /// seconds), a duel with an orc's tree and a group of three bandits, no
    /// two bodies sink into each other by more than a few millimetres (the
    /// pair shift rounds up by under 200 units, 3 mm, and a tick's walk of
    /// 2.2 m/s is 37 mm before the separation takes it back).
    #[test]
    fn no_two_bodies_sink_into_each_other_in_a_shove_a_duel_or_a_group() {
        let mut worst = 0;
        let mut w = pressed();
        for t in 0..180 {
            let feet = w.me.pose.foot;
            w.step(&hold(FORWARD));
            worst = worst.max(worst_overlap(&w));
            // the feet keep stepping forward: none jumps 10 cm in a tick
            for (a, b) in feet.iter().zip(w.me.pose.foot.iter()) {
                assert!(
                    t < 2 || a.dist(*b) < ONE / 10,
                    "tick {t}: a foot jumped {}",
                    a.dist(*b)
                );
            }
        }
        // the player shoves the orc back but never passes through it
        assert!(
            w.player.pos.z > w.foes[0].body.pos.z + ONE / 4,
            "{:?}",
            w.player.pos
        );
        let start = crate::player::duel_spot().z - ONE * 3 / 10;
        assert!(
            w.foes[0].body.pos.z < start - ONE,
            "the orc gave way: {:?}",
            w.foes[0].body.pos
        );
        for (seed, kinds) in [(3, &["orc"][..]), (5, &["bandit", "bandit", "bandit"][..])] {
            let mut w = World::new(seed);
            w.me.health = crate::health::Health::new(100_000);
            for k in kinds {
                crate::player::act(&mut w, "spawn", k).unwrap();
            }
            for t in 0..600 {
                let i = Input {
                    keys: if t == 2 {
                        crate::input::DRAW
                    } else if t % 120 < 70 {
                        FORWARD
                    } else {
                        0
                    },
                    dx: if t % 40 < 20 { 900 } else { -900 },
                    buttons: if t % 40 > 28 { crate::input::SWING } else { 0 },
                    ..Input::default()
                };
                w.step(&i);
                worst = worst.max(worst_overlap(&w));
            }
        }
        assert!(worst <= 200, "two bodies sank {worst} units into each other");
    }
}
