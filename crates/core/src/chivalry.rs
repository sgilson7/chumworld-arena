//! `chivalry` (M4, CA-Q7: B, T3): stamina, the feint, the follow-up, the
//! riposte and the kick, on top of CHUMWORLD's physical swordplay.
//!
//! A parry stays what CHUMWORLD made it: a blade meeting a blade
//! (`blade.rs`). Nothing here blocks or turns a blow aside; these rules only
//! change what a cut, a parry and a kick cost, how strong the arms are when
//! the cost has run the pool dry, and how a cut begun straight after a cut
//! or a parry comes.
//!
//! - **Stamina.** A pool for each fighter, in hundredths of a point. A cut
//!   costs by its way when it is chosen (`swing::step`); a blade meeting a
//!   striking blade costs the one not striking (the parrier); a feint and a
//!   kick cost their own. It comes back while the fighter has no cut under
//!   way and is not kicking, once `regen_wait_ticks` have passed since the
//!   last cost. At none the fighter is winded until it is back to
//!   `winded_until`: its arms work at `winded_strength_pct`, through the
//!   servo's strength (`player::goals`), so its cuts come slower and, the
//!   damage going by the blade's speed (`health::damage`), land softer.
//!   Nothing is refused for want of stamina but a feint and a kick.
//! - **Feint.** The right button pressed while a cut is in its first
//!   `feint_ticks` of wind-up breaks the cut off: the hand goes back to the
//!   guard and the feint costs `feint_cost`.
//! - **Follow-up.** A cut that ends without meeting a blade (landed or
//!   missed) leaves `follow_up_window_ticks` in which a new cut winds up in
//!   at most `follow_up_wind_ticks`, up to `follow_ups` in a row.
//! - **Riposte.** A parry leaves `riposte_window_ticks` in which a new cut
//!   winds up in at most `riposte_wind_ticks`, drives at
//!   `riposte_strength_pct`, and presses on against a blade for
//!   `riposte_press_ticks` before the cut is spent on it.
//! - **Kick.** F, drawn and between cuts, for `kick_cost`: at its
//!   `kick_hit_tick` a foe whose feet are within `kick_reach_cm` and within
//!   `kick_cone_deg` of straight ahead is staggered (`motion::Gait::stagger`,
//!   as `feel` staggers a struck body), shoved back, its cut under way
//!   broken off and its guard pushed `kick_aside_deg` aside.
//!
//! Every rule is in Q16.16 integers and runs on the tick from the input
//! records alone (CA-1, CA-3); rollback replays it. The numbers are
//! `data/chivalry.json`'s, each with its reasoning in SECOND-ORDER.md
//! (R6.x). The layer is off on the port's path: `World::settings.chivalry`
//! is false in `World::new`, so `port_keeps_chumworld_duel` replays
//! CHUMWORLD unchanged, and the game sets it on every world it makes (R6.1).

use std::sync::OnceLock;

use serde::Deserialize;

use crate::fx::{self, deg, Fx, ONE};
use crate::geom::{facing, right_of, v3, V3};
use crate::input::{self, Input};
use crate::player::{self, Fighter, Sheath};
use crate::swing::{Cut, Phase, Way};
use crate::world::World;

/// What each cut costs, whole points, by its way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CutCost {
    pub level: i32,
    pub chop: i32,
    pub rising: i32,
    pub diagonal: i32,
    pub thrust: i32,
}

/// `data/chivalry.json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub stamina: i32,
    pub cut_cost: CutCost,
    pub parry_cost: i32,
    pub regen_per_s: i32,
    pub regen_wait_ticks: u16,
    pub winded_until: i32,
    pub winded_strength_pct: i32,
    pub feint_cost: i32,
    pub feint_ticks: u8,
    pub follow_up_window_ticks: u8,
    pub follow_ups: u8,
    pub follow_up_wind_ticks: u8,
    pub riposte_window_ticks: u8,
    pub riposte_wind_ticks: u8,
    pub riposte_strength_pct: i32,
    pub riposte_press_ticks: u8,
    pub kick_cost: i32,
    pub kick_hit_tick: u8,
    pub kick_ticks: u8,
    pub kick_reach_cm: i32,
    pub kick_cone_deg: i32,
    pub kick_stagger_ticks: u8,
    pub kick_shove_cmps: i32,
    pub kick_aside_deg: i32,
    pub kick_lift_cm: i32,
    pub kick_out_cm: i32,
    pub event_ticks: u8,
    pub pilot_feint_after_ticks: u8,
    pub guard_side_deg: i32,
    pub guard_pitch_deg: i32,
    pub guard_reach_pct: i32,
    pub guard_line_pct: [i32; 3],
    pub guard_shift_deg: i32,
    pub pilot_guard_ticks: u32,
}

impl Spec {
    pub fn parse(json: &str) -> Result<Spec, String> {
        let mut v: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if let Some(o) = v.as_object_mut() {
            o.retain(|k, _| !k.starts_with('_'));
        }
        let s: Spec = serde_json::from_value(v).map_err(|e| e.to_string())?;
        if s.kick_hit_tick == 0 || s.kick_hit_tick >= s.kick_ticks {
            return Err("the kick lands after it starts and before it ends".into());
        }
        if s.kick_stagger_ticks > crate::feel::STAGGER_TICKS {
            return Err("a kick staggers no longer than a blow (feel::STAGGER_TICKS)".into());
        }
        Ok(s)
    }
}

/// The layer's numbers, read once.
pub fn spec() -> &'static Spec {
    static SPEC: OnceLock<Spec> = OnceLock::new();
    SPEC.get_or_init(|| Spec::parse(include_str!("../../../data/chivalry.json")).expect("data/chivalry.json"))
}

/// Hundredths of a point in a whole point.
pub const PART: i32 = 100;

/// What a cut is, besides its way: a plain cut, the `n`th follow-up in a
/// run, or a riposte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mark {
    #[default]
    Plain,
    FollowUp(u8),
    Riposte,
}

/// What the HUD says for a moment after it happens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Feint,
    FollowUp(u8),
    Riposte,
    KickLands,
    KickMisses,
    Kicked,
}

/// One fighter's share of the layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chiv {
    /// stamina, hundredths of a point
    pub stamina: i32,
    pub winded: bool,
    /// ticks since the last cost or the last tick a cut was under way
    pub rest: u16,
    /// ticks left in which a new cut is a follow-up, and how many have
    /// followed up in this run
    pub follow: u8,
    pub run: u8,
    /// ticks left in which a new cut is a riposte
    pub riposte: u8,
    /// ticks into a kick (0: none)
    pub kick: u8,
    /// ticks since this fighter last began a cut or a kick (a pilot reads
    /// it as a guard held back, `pilot::Cond::FoeTurtles`)
    pub idle: u16,
    /// the last event and its age, for the HUD
    pub event: Option<(Event, u8)>,
    /// T7: the right button's guard held (`guard_shell`)
    pub guard: Option<Guard>,
}

/// T7: a guard held with the right button: how far the mouse has shifted
/// it (angle units right and up, each within `guard_shift_deg`), and the
/// hand's target on its shell before it, which a release goes back to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Guard {
    pub shift: (i32, i32),
    pub from: (i32, i32, Fx),
}

impl Chiv {
    pub fn new() -> Chiv {
        Chiv {
            stamina: spec().stamina * PART,
            winded: false,
            rest: 0,
            follow: 0,
            run: 0,
            riposte: 0,
            kick: 0,
            idle: 0,
            event: None,
            guard: None,
        }
    }

    /// Stamina in whole points, rounded down.
    pub fn points(&self) -> i32 {
        self.stamina / PART
    }

    /// The share left, percent, rounded down.
    pub fn pct(&self) -> i32 {
        self.stamina * 100 / (spec().stamina * PART).max(1)
    }

    /// A cost paid: never below none; the wait before it comes back starts
    /// again.
    pub fn pay(&mut self, points: i32) {
        self.stamina = (self.stamina - points * PART).max(0);
        self.rest = 0;
        if self.stamina == 0 {
            self.winded = true;
        }
    }
}

impl Default for Chiv {
    fn default() -> Chiv {
        Chiv::new()
    }
}

/// T7: whether fighter `id` holds the right button's guard this tick: in
/// first person (off on the port's path, where the right button looks),
/// drawn, standing, between cuts and kicks, the right button held.
pub fn guards(w: &World, id: usize, inp: &Input) -> bool {
    let f = player::fighter(w, id);
    w.first_person()
        && inp.button(input::HOLD_LOOK)
        && f.sheath == Sheath::Drawn
        && !f.health.down()
        && f.arm.swing.cut.is_none()
        && f.chiv.kick == 0
}

/// T7: a tick of the right button's guard. The hand's target goes to the
/// guard in front (`guard_side_deg`, `guard_pitch_deg`, `guard_reach_pct`
/// on the shell, the blade along `guard_line`, `player::goals`), shifted by
/// the mouse within `guard_shift_deg` each way; the arm's servo takes the
/// hand there, so the guard is a pose the blades meet like any other.
/// Returns whether it is held; let go, the hand's target goes back to where
/// it was before.
pub fn guard_shell(w: &mut World, id: usize, inp: &Input) -> bool {
    let held = guards(w, id, inp);
    let s = spec();
    let f = player::parts_mut(w, id).1;
    let a = &mut f.arm;
    if !held {
        if let Some(g) = f.chiv.guard.take() {
            a.shell.side = g.from.0;
            a.shell.pitch = g.from.1;
            a.shell.reach = g.from.2;
        }
        return false;
    }
    let g = f.chiv.guard.get_or_insert(Guard {
        shift: (0, 0),
        from: (a.shell.side, a.shell.pitch, a.shell.reach),
    });
    let most = deg(s.guard_shift_deg);
    let step = |d: i32| (d as i64 * crate::arm::UNITS_PER_100_PX / 10_000) as i32;
    g.shift = (
        (g.shift.0 + step(inp.dx)).clamp(-most, most),
        (g.shift.1 - step(inp.dy)).clamp(-most, most),
    );
    let arm = a.reach();
    a.shell.side = deg(s.guard_side_deg) + g.shift.0;
    a.shell.pitch = deg(s.guard_pitch_deg) + g.shift.1;
    a.shell.reach = arm * s.guard_reach_pct / 100;
    a.shell.swing = crate::arm::Shell::eased(a.shell.swing, (0, 0, 0));
    a.swing.gesture = (0, 0, 0);
    true
}

/// T7: the blade's line in the right button's guard: raised across the
/// body, its point up and to the left, for a body facing `yaw`.
pub fn guard_line(yaw: i32) -> V3 {
    let [r, u, f] = spec().guard_line_pct;
    right_of(yaw)
        .scale(r * ONE / 100)
        .add(V3::UP.scale(u * ONE / 100))
        .add(facing(yaw, 0).scale(f * ONE / 100))
        .norm()
}

/// Whether the layer is on in this world (off on the port's path, R6.1).
pub fn on(w: &World) -> bool {
    w.settings.chivalry
}

/// What a cut of this way costs, whole points.
pub fn cut_cost(way: Way) -> i32 {
    let c = spec().cut_cost;
    match way {
        Way::Level(_) => c.level,
        Way::Chop => c.chop,
        Way::Rising => c.rising,
        Way::Diagonal(_) => c.diagonal,
        Way::Thrust => c.thrust,
    }
}

/// A cut just chosen: its cost paid, and, begun inside a riposte's or a
/// follow-up's window, marked and quickened. A riposte comes first when
/// both windows are open.
pub fn begin_cut(c: &mut Chiv, cut: &mut Cut) {
    let s = spec();
    c.pay(cut_cost(cut.way));
    c.idle = 0;
    if c.riposte > 0 {
        cut.mark = Mark::Riposte;
        cut.wind_ticks = s.riposte_wind_ticks;
        cut.strength_pct = s.riposte_strength_pct;
        cut.press = s.riposte_press_ticks;
        c.riposte = 0;
        c.follow = 0;
        c.run = 0;
        c.event = Some((Event::Riposte, 0));
    } else if c.follow > 0 && c.run < s.follow_ups {
        c.run += 1;
        cut.mark = Mark::FollowUp(c.run);
        cut.wind_ticks = s.follow_up_wind_ticks;
        c.follow = 0;
        c.event = Some((Event::FollowUp(c.run), 0));
    } else {
        c.follow = 0;
        c.run = 0;
    }
}

/// A cut over: one that met no blade opens the follow-up's window; one a
/// blade stopped ends the run.
pub fn cut_ended(c: &mut Chiv, blocked: bool) {
    if blocked {
        c.follow = 0;
        c.run = 0;
    } else {
        c.follow = spec().follow_up_window_ticks;
    }
}

/// Whether the right button, pressed now, breaks off this cut: the layer on,
/// the cut in the first `feint_ticks` of its wind-up, and the stamina to pay.
pub fn feints(on: bool, f: &Fighter, inp: &Input, cut: &Cut) -> bool {
    let s = spec();
    on && cut.phase == Phase::WindUp
        && cut.t <= s.feint_ticks
        && f.clicked(inp, input::HOLD_LOOK)
        && f.chiv.stamina >= s.feint_cost * PART
}

/// A feint paid for.
pub fn feint(c: &mut Chiv) {
    c.pay(spec().feint_cost);
    c.follow = 0;
    c.run = 0;
    c.event = Some((Event::Feint, 0));
}

/// The share of its arms' strength a fighter has, percent: winded, the
/// layer's `winded_strength_pct`; otherwise all of it.
pub fn strength_pct(on: bool, f: &Fighter) -> i32 {
    if on && f.chiv.winded {
        spec().winded_strength_pct
    } else {
        100
    }
}

/// Whether fighter `id` has a cut past its wind-up.
fn striking(w: &World, id: usize) -> bool {
    player::fighter(w, id)
        .arm
        .swing
        .cut
        .is_some_and(|c| c.phase != Phase::WindUp)
}

/// Step 5, after the feel of the blow: parries paid for, kicks started and
/// landed, stamina come back, the windows closed.
pub fn step(w: &mut World, input: &Input, foes: &[Input]) {
    if !on(w) {
        return;
    }
    let s = spec();
    // a blade meeting a striking blade, not striking itself: a parry
    for k in w.blades.clashes.clone() {
        for (me, them) in [(k.a as usize, k.b as usize), (k.b as usize, k.a as usize)] {
            if striking(w, them) && !striking(w, me) {
                let c = &mut player::parts_mut(w, me).1.chiv;
                c.pay(s.parry_cost);
                c.riposte = s.riposte_window_ticks;
            }
        }
    }
    for id in 0..player::count(w) {
        let inp = if id == 0 { *input } else { foes[id - 1] };
        kick(w, id, &inp);
        let f = player::parts_mut(w, id).1;
        let busy = f.arm.swing.cut.is_some() || f.chiv.kick > 0;
        let down = f.health.down();
        let c = &mut f.chiv;
        if busy || down {
            c.rest = 0;
        } else {
            c.rest = c.rest.saturating_add(1);
            if c.rest >= s.regen_wait_ticks {
                c.stamina = (c.stamina + s.regen_per_s * PART / crate::ticks::TICKS_PER_SECOND as i32)
                    .min(s.stamina * PART);
            }
        }
        if c.stamina == 0 {
            c.winded = true;
        } else if c.winded && c.stamina >= s.winded_until * PART {
            c.winded = false;
        }
        c.riposte = c.riposte.saturating_sub(1);
        c.follow = c.follow.saturating_sub(1);
        if c.follow == 0 && !busy {
            c.run = 0;
        }
        c.idle = c.idle.saturating_add(1);
        c.event = match c.event {
            Some((e, age)) if age < s.event_ticks => Some((e, age + 1)),
            _ => None,
        };
    }
}

/// Fighter `id`'s kick this tick: begun on F, landed at its hit tick, over
/// at its end.
fn kick(w: &mut World, id: usize, inp: &Input) {
    let s = spec();
    let f = player::fighter(w, id);
    if f.chiv.kick > 0 {
        let t = f.chiv.kick + 1;
        player::parts_mut(w, id).1.chiv.kick = if t >= s.kick_ticks { 0 } else { t };
        if t == s.kick_hit_tick {
            land(w, id);
        }
        return;
    }
    let ready = f.sheath == Sheath::Drawn
        && !f.health.down()
        && f.arm.swing.cut.is_none()
        && f.pressed(inp, input::KICK)
        && f.chiv.stamina >= s.kick_cost * PART;
    if ready {
        let c = &mut player::parts_mut(w, id).1.chiv;
        c.pay(s.kick_cost);
        c.kick = 1;
        c.idle = 0;
    }
}

/// The foe a kick from fighter `id` meets now: of the standing foes whose
/// feet are within the kick's reach and cone, the nearest (ties by number).
pub fn kicked_by(w: &World, id: usize) -> Option<usize> {
    let s = spec();
    let me = player::body(w, id);
    let team = player::fighter(w, id).team;
    let ahead = facing(me.facing, 0);
    let reach = s.kick_reach_cm * ONE / 100;
    let cone = fx::cos(deg(s.kick_cone_deg));
    (0..player::count(w))
        .filter(|&j| j != id)
        .filter(|&j| player::fighter(w, j).team != team && !player::fighter(w, j).health.down())
        .filter_map(|j| {
            let d = player::body(w, j).pos.sub(me.pos);
            let flat = v3(d.x, 0, d.z);
            let gap = flat.len();
            (gap <= reach && (gap == 0 || flat.norm().dot(ahead) >= cone)).then_some((gap, j))
        })
        .min()
        .map(|(_, j)| j)
}

/// A kick's hit tick: the foe it meets staggers back, its cut breaks off and
/// its guard is pushed aside.
fn land(w: &mut World, id: usize) {
    let s = spec();
    let Some(j) = kicked_by(w, id) else {
        player::parts_mut(w, id).1.chiv.event = Some((Event::KickMisses, 0));
        return;
    };
    player::parts_mut(w, id).1.chiv.event = Some((Event::KickLands, 0));
    let from = player::body(w, id).pos;
    let d = player::body(w, j).pos.sub(from);
    let flat = v3(d.x, 0, d.z);
    let way = if flat == V3::ZERO {
        facing(player::body(w, id).facing, 0)
    } else {
        flat.norm()
    };
    let shove = s.kick_shove_cmps * ONE / 100 / crate::ticks::TICKS_PER_SECOND as i32;
    let (b, f) = player::parts_mut(w, j);
    b.vel = b.vel.add(way.scale(shove));
    f.gait.stagger = f.gait.stagger.max(s.kick_stagger_ticks);
    f.gait.stagger_way = way;
    f.arm.swing.cut = None;
    f.arm.swing.gesture = (0, 0, 0);
    let aside = deg(s.kick_aside_deg);
    let sh = &mut f.arm.shell;
    sh.side = if sh.side >= 0 {
        sh.side + aside
    } else {
        sh.side - aside
    }
    .clamp(crate::arm::SIDE_MIN, crate::arm::SIDE_MAX);
    let c = &mut f.chiv;
    c.follow = 0;
    c.run = 0;
    c.riposte = 0;
    c.event = Some((Event::Kicked, 0));
    if id == 0 || j == 0 {
        w.feel.knock = crate::feel::knock(fx::int(5), way);
    }
}

/// How far a kicking leg is out, `ONE` at the hit tick, rising to it and
/// falling after.
pub fn kick_out(kick: u8) -> Fx {
    let s = spec();
    let t = kick as i32;
    let hit = s.kick_hit_tick as i32;
    let end = s.kick_ticks as i32;
    if t <= 0 {
        0
    } else if t <= hit {
        t * ONE / hit
    } else {
        ((end - t) * ONE / (end - hit)).max(0)
    }
}

/// The kicking foot (the left, the leg ahead in a guard) raised out in
/// front of the body while a kick is under way.
pub fn kick_leg(b: &player::Body, f: &Fighter, foot: &mut [V3; 2]) {
    let out = kick_out(f.chiv.kick);
    if out == 0 {
        return;
    }
    let s = spec();
    let build = f.build();
    let to = b
        .pos
        .add(facing(b.facing, 0).scale(s.kick_out_cm * ONE / 100))
        .add(right_of(b.facing).scale(-build.hip_x))
        .add(v3(0, s.kick_lift_cm * ONE / 100, 0));
    foot[0] = foot[0].lerp(to, out);
}

/// Everything the HUD shows of the layer, in one value: each fighter's
/// stamina in whole points and whether it is winded, and the player's last
/// event.
pub fn stamp(w: &World) -> (Vec<(i32, bool)>, Option<Event>) {
    if !on(w) {
        return (Vec::new(), None);
    }
    (
        (0..player::count(w))
            .map(|id| {
                let c = player::fighter(w, id).chiv;
                (c.points(), c.winded)
            })
            .collect(),
        w.me.chiv.event.map(|e| e.0),
    )
}

/// The string key of an event, and the value it fills (`{n}`).
pub fn event_key(e: Event) -> (&'static str, u8) {
    match e {
        Event::Feint => ("move_feint", 0),
        Event::FollowUp(n) => ("move_follow_up", n),
        Event::Riposte => ("move_riposte", 0),
        Event::KickLands => ("move_kick_lands", 0),
        Event::KickMisses => ("move_kick_misses", 0),
        Event::Kicked => ("move_kicked", 0),
    }
}

/// The line over the game: the player's last move of the layer, for a
/// moment.
pub fn hud(w: &World, s: &crate::page::Strings, out: &mut Vec<crate::world::HudItem>) {
    let e = if on(w) { w.me.chiv.event } else { None };
    out.push(crate::world::HudItem {
        id: "move".into(),
        text: e
            .map(|(e, _)| {
                let (key, n) = event_key(e);
                s.get(key).replace("{n}", &n.to_string())
            })
            .unwrap_or_default(),
        shown: e.is_some(),
    });
}

/// `chivalry.<name>`: `on` (`on`, `off`) turns the layer on or off;
/// `my_stamina` and `foe_stamina` set the player's or every foe's stamina,
/// whole points (for tests and pictures); `foe_move` starts every foe on
/// a move of its tree (`kick`, `feint_cut`), for pictures.
pub fn act(w: &mut World, name: &str, value: &str) -> Result<(), String> {
    let most = spec().stamina;
    let points = || {
        value
            .trim()
            .parse::<i32>()
            .ok()
            .filter(|p| (0..=most).contains(p))
            .ok_or_else(|| format!("Stamina is whole points from 0 to {most}, not \"{value}\"."))
    };
    let set = |c: &mut Chiv, p: i32| {
        c.stamina = p * PART;
        c.winded = p == 0;
        c.rest = 0;
    };
    match name {
        "on" => match value.trim() {
            "on" => {
                w.settings.chivalry = true;
                Ok(())
            }
            "off" => {
                w.settings.chivalry = false;
                Ok(())
            }
            other => Err(format!(
                "Stamina, feints and kicks are on or off, not \"{other}\"."
            )),
        },
        "my_stamina" => {
            let p = points()?;
            set(&mut w.me.chiv, p);
            Ok(())
        }
        "foe_stamina" => {
            let p = points()?;
            for f in &mut w.foes {
                set(&mut f.fighter.chiv, p);
            }
            Ok(())
        }
        // every foe begins this move of its tree now (`kick`, `feint_cut`),
        // driven by its tree from here on: staging for a picture
        "foe_move" => {
            let m = match value.trim() {
                "guard_up" => crate::pilot::Move::GuardUp,
                "kick" => crate::pilot::Move::Kick,
                "feint_cut" => crate::pilot::Move::FeintCut,
                other => {
                    return Err(format!(
                        "A staged move is guard_up, kick or feint_cut, not \"{other}\"."
                    ))
                }
            };
            for f in &mut w.foes {
                f.brain.control = crate::pilot::Control::Tree;
                f.brain.current = Some((m, 0));
            }
            Ok(())
        }
        _ => Err(format!("Unknown action \"chivalry.{name}\".")),
    }
}

/// The numbers the glossary's placeholders name (`GLOSSARY.md`: Feint,
/// Follow-up, Kick, Riposte, Stamina, Winded).
pub fn values() -> Vec<(&'static str, String)> {
    let s = spec();
    let tps = crate::ticks::TICKS_PER_SECOND as i32;
    vec![
        ("chivalry.stamina", s.stamina.to_string()),
        ("chivalry.regen", s.regen_per_s.to_string()),
        ("chivalry.winded_pct", s.winded_strength_pct.to_string()),
        ("chivalry.winded_until", s.winded_until.to_string()),
        ("chivalry.feint_cost", s.feint_cost.to_string()),
        ("chivalry.follow_ups", s.follow_ups.to_string()),
        (
            "chivalry.riposte_s",
            fx::decimal(s.riposte_window_ticks as i32 * ONE / tps, 1),
        ),
        ("chivalry.kick_cost", s.kick_cost.to_string()),
        ("chivalry.kick_reach", s.kick_reach_cm.to_string()),
        ("chivalry.guard_shift", s.guard_shift_deg.to_string()),
    ]
}

#[cfg(test)]
mod tests;
