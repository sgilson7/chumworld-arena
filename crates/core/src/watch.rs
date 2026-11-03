//! T11 (CA-Q17: A): watch mode, after Vagrancy's BT lab. Two pilots fight
//! while the reader reads their trees. The left pilot flies the player's
//! fighter through the same record a person's keys make
//! (`World::watch`, `pilot::decide_with`, CA-3); the right one is the foe,
//! as in Practice. The camera circles the pair. Time runs at full, half,
//! quarter or eighth speed, pauses, and steps one tick. Each pilot's last
//! reading of its tree (`pilot::Trace`) is drawn by the page from `View`:
//! each rule's checks true, false or not reached, and the rule that fired.
//!
//! Like everything here, the world under watch is the duel's own world:
//! the same seed and the same two pilots fight the same fight.

use crate::fx::{self, deg, Fx, ONE};
use crate::game::Game;
use crate::geom::{facing, v3, V3};
use crate::input::Input;
use crate::pilot::{Cond, Move};
use crate::player;
use crate::render::Camera;
use serde::Serialize;

/// Who can be watched, in the order the pickers step through them: the
/// computer's knight, then the campaign's opponents (by their book keys).
pub const PILOTS: [&str; 12] = [
    "knight",
    "sentry",
    "press",
    "bully",
    "trickster",
    "squire",
    "sellsword",
    "hedge",
    "veteran",
    "banneret",
    "champion",
    "mirror",
];
/// T11: the styles, named by their own names rather than their bodies.
pub const STYLES: [&str; 4] = ["sentry", "press", "bully", "trickster"];
/// T11: the weapons a side can be given in place of its own (`None`).
pub const WEAPONS: [Option<crate::pose::Held>; 8] = {
    use crate::pose::Held;
    [
        None,
        Some(Held::Longsword),
        Some(Held::Broadsword),
        Some(Held::Axe),
        Some(Held::GreatAxe),
        Some(Held::Mace),
        Some(Held::Quarterstaff),
        Some(Held::Spear),
    ]
};
/// Time runs at a whole share of full speed (the BT lab's four).
pub const SPEEDS: [u32; 4] = [1, 2, 4, 8];
/// The camera: this far from the pair's middle, looking at a point this
/// high, at this pitch; the reader drags it round and wheels it nearer.
pub const ORBIT_DIST: Fx = fx::int(8);
/// The camera looks this share of its distance to the right of the pair,
/// so they sit in the open view between the controls and the tree.
pub const ORBIT_SHIFT_PCT: i32 = 7;
pub const ORBIT_DIST_MIN: Fx = fx::int(3);
pub const ORBIT_DIST_MAX: Fx = fx::int(14);
pub const ORBIT_LOOK_UP: Fx = ONE * 11 / 10;
pub const ORBIT_PITCH: i32 = -deg(16);
pub const ORBIT_PITCH_MIN: i32 = -deg(60);
pub const ORBIT_PITCH_MAX: i32 = -deg(3);
/// A drag of 100 px turns the camera 30°; a wheel notch moves it 10%.
pub const ORBIT_DEG_PER_100_PX: i32 = 30;
pub const ORBIT_ZOOM_PCT: i32 = 10;
/// A frame's time is held to this, so a tab left behind runs no burst.
pub const FRAME_MOST_US: i64 = 100_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Watch {
    /// indexes into `PILOTS`
    pub left: usize,
    pub right: usize,
    /// an index into `SPEEDS`
    pub speed: usize,
    pub paused: bool,
    /// ticks asked for one at a time while paused
    pub steps: u32,
    /// whose tree is shown: 0 left, 1 right
    pub show: usize,
    /// each fighter's running move drawn over it
    pub overlay: bool,
    pub yaw: i32,
    pub pitch: i32,
    pub dist: Fx,
    /// the watched clock, microseconds, and the page's clock last frame
    pub time_us: i64,
    pub last_us: Option<i64>,
    /// fights started, so each has its own seed
    pub fights: u64,
    /// T11: each side's weapon, an index into `WEAPONS` (0: its own)
    pub weapons: [usize; 2],
}

impl Default for Watch {
    fn default() -> Watch {
        Watch {
            // the veteran and the banneret
            left: 8,
            right: 9,
            // the BT lab starts at an eighth of full speed, slow enough to
            // follow a tree (Vagrancy, 2026-10-07)
            speed: 3,
            paused: false,
            steps: 0,
            show: 0,
            overlay: true,
            yaw: crate::world::START_YAW + deg(90),
            pitch: ORBIT_PITCH,
            dist: ORBIT_DIST,
            time_us: 0,
            last_us: None,
            fights: 0,
            weapons: [0, 0],
        }
    }
}

/// A new watched fight: the two pilots at their opening marks, the
/// world in CHUMWORLD's shoulder scheme (where the mouse moves the hand,
/// as a pilot's record does for a foe) with the Chivalry layer on.
pub fn start(g: &mut Game) {
    let v = &mut g.watching;
    v.fights += 1;
    v.paused = false;
    v.steps = 0;
    v.time_us = 0;
    v.last_us = None;
    let (left, right) = (PILOTS[v.left], PILOTS[v.right]);
    let salt = g.seed ^ v.fights.wrapping_mul(0x2545_f491_4f6c_dd1d);
    let mut w = crate::world::World::new(salt);
    w.settings.control = crate::world::Control::Shoulder;
    w.settings.chivalry = true;
    crate::game::apply(&mut w, &g.kept);
    let (kind, _) = crate::pilot::pilot_named(left).expect("a watched pilot");
    crate::rounds::begin(&mut w, right, kind).expect("a watched pilot");
    if let Some(r) = w.rounds.as_mut() {
        r.held = v.weapons.map(|k| WEAPONS[k]);
    }
    crate::rounds::opening(&mut w);
    w.watch = crate::pilot::brain(left, salt ^ 0x5bd1_e995);
    g.world = w;
    g.clock.rest();
    g.ui_version += 1;
}

/// One display frame of watch mode: the watched clock runs at its speed,
/// or a tick asked for while paused.
pub fn advance(g: &mut Game, now_us: i64) {
    let v = &mut g.watching;
    let elapsed = v
        .last_us
        .map(|l| (now_us - l).clamp(0, FRAME_MOST_US))
        .unwrap_or(0);
    v.last_us = Some(now_us);
    let mut ran = 0;
    if !v.paused {
        v.time_us += elapsed / SPEEDS[v.speed] as i64;
        let t = v.time_us;
        ran = g.clock.frame(t, 0).len();
        for _ in 0..ran {
            g.world.step(&Input::default());
        }
        g.alpha = g.clock.alpha();
    } else {
        while g.watching.steps > 0 {
            g.watching.steps -= 1;
            g.world.step(&Input::default());
            ran += 1;
        }
        g.alpha = ONE;
    }
    if ran > 0 {
        g.ui_version += 1;
    }
}

/// `watch.<name>`: `left` and `right` step a picker on (or name a pilot),
/// `start` fights again, `speed` takes 1, 2, 4 or 8, `pause` pauses or
/// resumes, `step` runs one tick while paused, `show` takes left or right,
/// `overlay` on or off, `orbit` "dx dy" in pixels, `zoom` a wheel's sign.
pub fn act(g: &mut Game, name: &str, value: &str) -> Result<(), String> {
    let v = &mut g.watching;
    let value = value.trim();
    let next = |at: usize| -> Result<usize, String> {
        if value.is_empty() {
            return Ok((at + 1) % PILOTS.len());
        }
        PILOTS
            .iter()
            .position(|p| *p == value)
            .ok_or_else(|| format!("There is no pilot \"{value}\" to watch."))
    };
    match name {
        "left" => {
            v.left = next(v.left)?;
            start(g);
        }
        "right" => {
            v.right = next(v.right)?;
            start(g);
        }
        "start" => start(g),
        // T11: each side's weapon steps on through `WEAPONS`
        "left_weapon" | "right_weapon" => {
            let k = (name == "right_weapon") as usize;
            v.weapons[k] = (v.weapons[k] + 1) % WEAPONS.len();
            start(g);
        }
        "speed" => {
            let s: u32 = value
                .parse()
                .map_err(|_| format!("A speed is 1, 2, 4 or 8, not \"{value}\"."))?;
            v.speed = SPEEDS
                .iter()
                .position(|&x| x == s)
                .ok_or_else(|| format!("A speed is 1, 2, 4 or 8, not \"{value}\"."))?;
            v.paused = false;
        }
        "pause" => v.paused = !v.paused,
        "step" => {
            v.paused = true;
            v.steps += 1;
        }
        "show" => {
            v.show = match value {
                "left" => 0,
                "right" => 1,
                _ => return Err(format!("A tree to show is left or right, not \"{value}\".")),
            }
        }
        "overlay" => {
            v.overlay = match value {
                "on" => true,
                "off" => false,
                _ => return Err(format!("The overlay is on or off, not \"{value}\".")),
            }
        }
        "orbit" => {
            // the browser's movement may carry a fraction: the whole pixels
            let mut it = value.split_whitespace().map(whole);
            let (Some(Some(dx)), Some(Some(dy))) = (it.next(), it.next()) else {
                return Err(format!(
                    "An orbit is two whole numbers of pixels, not \"{value}\"."
                ));
            };
            let per =
                |px: i32| (px.clamp(-2000, 2000) as i64 * deg(ORBIT_DEG_PER_100_PX) as i64 / 100) as i32;
            v.yaw = fx::wrap(v.yaw - per(dx));
            v.pitch = (v.pitch - per(dy)).clamp(ORBIT_PITCH_MIN, ORBIT_PITCH_MAX);
        }
        "zoom" => {
            // the wheel's movement as the browser gives it: only its way
            let way = sign(value).ok_or_else(|| format!("A zoom is a number, not \"{value}\"."))?;
            let d = v.dist as i64 * (100 + way.signum() as i64 * ORBIT_ZOOM_PCT as i64) / 100;
            v.dist = (d as Fx).clamp(ORBIT_DIST_MIN, ORBIT_DIST_MAX);
        }
        _ => return Err(format!("Unknown action \"watch.{name}\".")),
    }
    g.ui_version += 1;
    Ok(())
}

/// The whole part of a number as the browser writes it ("12", "-3.5").
fn whole(text: &str) -> Option<i32> {
    let mut parts = text.splitn(2, '.');
    let int = parts.next()?;
    let frac = parts.next().unwrap_or("0");
    if !frac.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if int == "-" || int == "-0" {
        return Some(0);
    }
    int.parse().ok()
}

/// The way of a number as the browser writes it: −1, 0 or 1.
fn sign(text: &str) -> Option<i32> {
    let digits = text.trim_start_matches('-');
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    let any = digits.chars().any(|c| ('1'..='9').contains(&c));
    Some(match (any, text.starts_with('-')) {
        (false, _) => 0,
        (true, true) => -1,
        (true, false) => 1,
    })
}

/// The pair's middle, `alpha` of the way from last tick to this.
fn middle(g: &Game) -> V3 {
    let w = &g.world;
    let a = w.player.feet(g.alpha);
    let b = if player::count(w) > 1 {
        player::body(w, 1).feet(g.alpha)
    } else {
        a
    };
    a.lerp(b, ONE / 2)
}

/// The camera circling the pair: `dist` back from a point `ORBIT_LOOK_UP`
/// over their middle, along the watch's heading and pitch.
pub fn camera(g: &Game) -> Camera {
    let v = &g.watching;
    let fwd = facing(v.yaw, v.pitch);
    let right = facing(v.yaw, 0).cross(V3::UP).norm();
    let at = middle(g)
        .add(v3(0, ORBIT_LOOK_UP, 0))
        .add(right.scale(v.dist * ORBIT_SHIFT_PCT / 100));
    Camera {
        eye: at.sub(fwd.scale(v.dist)),
        fwd,
        fov: crate::camera::vertical_fov(crate::camera::WATCH_FOV_DEG, g.aspect),
        roll: 0,
    }
}

// ---- what the page draws ---------------------------------------------------

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct View {
    /// tick, round, score and speed, in a line
    pub line: String,
    /// the left fighter, then the right
    pub sides: Vec<Side>,
    /// whose tree is shown: 0 or 1
    pub show: usize,
    pub paused: bool,
    /// over each fighter's head, where the page puts its move's name; none
    /// with the overlay off
    pub labels: Vec<Label>,
    pub hint: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Side {
    pub name: String,
    /// the move under way, in words
    pub now: String,
    /// the keys this tick, in words
    pub keys: String,
    /// over the tree: whose it is and which rule ran
    pub heading: String,
    pub rules: Vec<RuleView>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RuleView {
    pub checks: Vec<CheckView>,
    pub act: String,
    /// the rule that fired at the last reading
    pub ran: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckView {
    pub text: String,
    /// "true", "false" or "none" (not reached)
    pub state: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Label {
    /// where the page puts it, as CSS: shares of the view's width and
    /// height in percent
    pub left: String,
    pub top: String,
    pub text: String,
}

fn n(s: &crate::page::Strings, key: &str, k: i64) -> String {
    crate::api::fill(&s.get(key), &[("n", k.to_string())])
}

/// A check in words.
pub fn cond_text(s: &crate::page::Strings, c: &Cond) -> String {
    match c {
        Cond::HurtBelow(k) => n(s, "cond_hurt_below", *k as i64),
        Cond::GapAbove(k) => n(s, "cond_gap_above", *k as i64),
        Cond::GapBelow(k) => n(s, "cond_gap_below", *k as i64),
        Cond::BeyondReach(k) => n(s, "cond_beyond_reach", *k as i64),
        Cond::WithinReach(k) => n(s, "cond_within_reach", *k as i64),
        Cond::HasThrowable => s.get("cond_has_throwable"),
        Cond::TipNearHead(k) => n(s, "cond_tip_near_head", *k as i64),
        Cond::SwingComing(k) => n(s, "cond_swing_coming", *k as i64),
        Cond::Opening(k) => n(s, "cond_opening", *k as i64),
        Cond::Chance(k) => n(s, "cond_chance", *k as i64),
        Cond::FoeDown => s.get("cond_foe_down"),
        Cond::Unarmed => s.get("cond_unarmed"),
        Cond::CanTakeUp => s.get("cond_can_take_up"),
        Cond::WeaponWithin(k) => n(s, "cond_weapon_within", *k as i64),
        Cond::LostSwordArm => s.get("cond_lost_sword_arm"),
        Cond::LostLeg => s.get("cond_lost_leg"),
        Cond::StaminaBelow(k) => n(s, "cond_stamina_below", *k as i64),
        Cond::StaminaAbove(k) => n(s, "cond_stamina_above", *k as i64),
        Cond::JustParried => s.get("cond_just_parried"),
        Cond::FoeTurtles(k) => n(s, "cond_foe_turtles", *k as i64),
        Cond::FoeFeinted(k) => n(s, "cond_foe_feinted", *k as i64),
        Cond::FoeStaggered => s.get("cond_foe_staggered"),
        Cond::Feinted(k) => n(s, "cond_feinted", *k as i64),
        Cond::CanFollow => s.get("cond_can_follow"),
        Cond::FoeGuards => s.get("cond_foe_guards"),
    }
}

/// A move in words.
pub fn move_text(s: &crate::page::Strings, m: Move) -> String {
    s.get(match m {
        Move::Guard => "act_guard",
        Move::CloseIn => "act_close_in",
        Move::Circle => "act_circle",
        Move::RaiseGuard => "act_raise_guard",
        Move::Block => "act_block",
        Move::Attack => "act_attack",
        Move::Overhead => "act_overhead",
        Move::Sweep => "act_sweep",
        Move::Thrust => "act_thrust",
        Move::Backhand => "act_backhand",
        Move::Rising => "act_rising",
        Move::Feint => "act_feint",
        Move::Flee => "act_flee",
        Move::Throw => "act_throw",
        Move::Ring => "act_ring",
        Move::BackOff => "act_back_off",
        Move::Cast => "act_cast",
        Move::TakeUp => "act_take_up",
        Move::RunAway => "act_run_away",
        Move::Kick => "act_kick",
        Move::FeintCut => "act_feint_cut",
        Move::GuardUp => "act_guard_up",
    })
}

/// The keys of a record, in words, in a fixed order.
pub fn keys_text(s: &crate::page::Strings, i: &Input) -> String {
    use crate::input as k;
    let mut out = Vec::new();
    for (bit, key) in [
        (k::FORWARD, "watch_key_forward"),
        (k::BACK, "watch_key_back"),
        (k::LEFT, "watch_key_left"),
        (k::RIGHT, "watch_key_right"),
        (k::RUN, "watch_key_run"),
        (k::DODGE, "watch_key_dodge"),
        (k::KICK, "watch_key_kick"),
    ] {
        if i.key(bit) {
            out.push(s.get(key));
        }
    }
    if i.button(k::SWING) {
        out.push(s.get("watch_key_swing"));
    }
    if i.button(k::HOLD_LOOK) {
        out.push(s.get("watch_key_guard"));
    }
    if i.dx != 0 || i.dy != 0 {
        out.push(crate::api::fill(
            &s.get("watch_key_mouse"),
            &[("dx", i.dx.to_string()), ("dy", i.dy.to_string())],
        ));
    }
    if out.is_empty() {
        s.get("watch_no_keys")
    } else {
        out.join(" · ")
    }
}

fn side(g: &Game, id: usize, name: String) -> Side {
    let s = &g.strings;
    let w = &g.world;
    let brain = if id == 0 {
        w.watch.as_ref()
    } else {
        w.foes.get(id - 1).map(|f| &f.brain)
    };
    let book = &w.pilots.book;
    let now = brain
        .and_then(|b| b.current)
        .map(|(m, _)| move_text(s, m))
        .unwrap_or_else(|| s.get("act_guard"));
    let keys = crate::api::fill(
        &s.get("watch_keys"),
        &[("keys", keys_text(s, &player::fighter(w, id).last))],
    );
    let Some(b) = brain else {
        return Side {
            name,
            now,
            keys,
            heading: String::new(),
            rules: Vec::new(),
        };
    };
    let tree_name = b
        .trace
        .as_ref()
        .map(|t| t.tree.clone())
        .unwrap_or_else(|| book.kinds[b.key.as_str()].tree.clone());
    let tree = &book.trees[tree_name.as_str()];
    let rules: Vec<RuleView> = tree
        .iter()
        .enumerate()
        .map(|(i, r)| RuleView {
            checks: r
                .when
                .iter()
                .enumerate()
                .map(|(k, c)| CheckView {
                    text: cond_text(s, c),
                    state: match b
                        .trace
                        .as_ref()
                        .and_then(|t| t.checks.get(i))
                        .and_then(|v| v.get(k))
                        .copied()
                        .flatten()
                    {
                        Some(true) => "true",
                        Some(false) => "false",
                        None => "none",
                    },
                })
                .collect(),
            act: move_text(s, r.act),
            ran: b.trace.as_ref().and_then(|t| t.fired) == Some(i),
        })
        .collect();
    let heading = match b.trace.as_ref().and_then(|t| t.fired) {
        Some(i) => crate::api::fill(
            &s.get("watch_tree_ran"),
            &[
                ("name", name.clone()),
                ("n", (i + 1).to_string()),
                ("of", rules.len().to_string()),
            ],
        ),
        None => crate::api::fill(&s.get("watch_tree_none"), &[("name", name.clone())]),
    };
    Side {
        name,
        now,
        keys,
        heading,
        rules,
    }
}

/// T11: the weapon side `k` holds, in words ("its own" until one is given).
pub fn weapon_name(g: &Game, k: usize) -> String {
    match WEAPONS[g.watching.weapons[k]] {
        None => g.strings.get("weapon_own"),
        Some(item) => g.strings.get(&format!("weapon_{}", held_key(item))),
    }
}

/// A weapon's key in the strings and the book.
pub fn held_key(item: crate::pose::Held) -> &'static str {
    use crate::pose::Held;
    match item {
        Held::Longsword => "longsword",
        Held::Broadsword => "broadsword",
        Held::Axe => "axe",
        Held::GreatAxe => "great_axe",
        Held::Mace => "mace",
        Held::Quarterstaff => "quarterstaff",
        Held::Spear => "spear",
        Held::Sword => "sword",
        Held::Cleaver => "cleaver",
        Held::Hammer => "hammer",
        _ => "short_sword",
    }
}

/// The two fighters' names, as their bars give them: each by the body it
/// fights in (a knight in the other knight's plate when both are knights).
pub fn names(g: &Game) -> [String; 2] {
    let w = &g.world;
    [0, 1].map(|id| {
        if id >= player::count(w) {
            return String::new();
        }
        // a style by its own name; the knight and the campaign's opponents
        // by the body they fight in
        let key = if id == 0 {
            w.watch.as_ref().map(|b| b.key.as_str())
        } else {
            Some(w.foes[id - 1].brain.key.as_str())
        };
        match key {
            Some(k) if STYLES.contains(&k) => g.strings.get(&format!("who_{k}")),
            _ => g.strings.get(&format!(
                "who_{}",
                crate::pilot::kind_name(player::fighter(w, id).kind)
            )),
        }
    })
}

/// What watch mode shows this frame.
pub fn view(g: &Game) -> View {
    let s = &g.strings;
    let v = &g.watching;
    let w = &g.world;
    let names = names(g);
    let (round, wins) = w
        .rounds
        .as_ref()
        .map(|r| (r.round, r.wins))
        .unwrap_or((1, [0, 0]));
    let speed = if v.paused {
        s.get("watch_paused")
    } else {
        s.get(&format!("speed_{}", SPEEDS[v.speed]))
    };
    let line = crate::api::fill(
        &s.get("watch_line"),
        &[
            ("tick", w.tick.to_string()),
            ("round", round.to_string()),
            ("left", names[0].clone()),
            ("right", names[1].clone()),
            ("a", wins[0].to_string()),
            ("b", wins[1].to_string()),
            ("speed", speed),
        ],
    );
    let sides = vec![side(g, 0, names[0].clone()), side(g, 1, names[1].clone())];
    let mut labels = Vec::new();
    if v.overlay {
        let cam = camera(g);
        for (id, sd) in sides.iter().enumerate() {
            if id >= player::count(w) {
                continue;
            }
            let f = player::fighter(w, id);
            let head = f.prev_pose.lerp(&f.pose, g.alpha).head.add(v3(0, ONE / 3, 0));
            if let Some((x, y)) = cam.screen(g.aspect, head) {
                let pct = |q: Fx| format!("{}%", fx::decimal(q * 100, 1));
                labels.push(Label {
                    left: pct(x),
                    top: pct(y),
                    text: crate::api::fill(&s.get("watch_now"), &[("move", sd.now.clone())]),
                });
            }
        }
    }
    View {
        line,
        sides,
        show: v.show,
        paused: v.paused,
        labels,
        hint: s.get("watch_hint"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::tests::game;
    use crate::game::Screen;

    fn watching() -> Game {
        let mut g = game();
        g.act("screen.show", "menu").unwrap();
        g.act("screen.show", "watch").unwrap();
        g
    }

    /// Frames 1/60 s apart for `secs` seconds of the page's clock.
    fn frames(g: &mut Game, secs: i64) {
        let start = g.watching.last_us.unwrap_or(0);
        for k in 1..=secs * 60 {
            g.advance(start + k * 1_000_000 / 60, 0, 0, 0, 0);
        }
    }

    /// From the menu, Watch opens a fight of the veteran (the player's
    /// fighter, flown by its pilot) against the banneret, in the shoulder
    /// scheme with the Chivalry layer on, at an eighth of full speed, and
    /// the view model carries both trees in full.
    #[test]
    fn watch_opens_a_fight_of_two_pilots_with_both_trees() {
        let g = watching();
        assert_eq!(g.screen, Screen::Watch);
        let w = &g.world;
        assert_eq!(w.watch.as_ref().unwrap().key, "veteran");
        assert_eq!(w.me.kind, crate::pose::Kind::Veteran);
        assert_eq!(w.foes[0].brain.key, "banneret");
        assert_eq!(w.settings.control, crate::world::Control::Shoulder);
        assert!(w.settings.chivalry);
        let u = crate::api::ui(&g);
        let v = u.watch.expect("the watch view");
        assert_eq!(v.sides[0].name, "The veteran");
        assert_eq!(v.sides[1].name, "The banneret");
        let trees = &w.pilots.book.trees;
        assert_eq!(v.sides[0].rules.len(), trees["veteran"].len());
        assert_eq!(v.sides[1].rules.len(), trees["banneret"].len());
        assert!(v.line.ends_with("Eighth speed"), "{}", v.line);
        assert!(u.panels[0]
            .actions
            .iter()
            .any(|a| a.label == "On the left: The veteran"));
        assert!(!u.capture, "the pointer is the reader's, for the buttons");
        // the player's fighter is flown by the veteran's own tree, not the
        // ring tree a foe off the duel reads, and its guard works as a
        // pilot's does in the duel (`chivalry::guards`)
        let mut g = g;
        g.act("watch.speed", "1").unwrap();
        frames(&mut g, 2);
        let b = g.world.watch.as_ref().unwrap();
        assert_eq!(b.trace.as_ref().unwrap().tree, "veteran");
        g.world.me.arm.swing.cut = None;
        g.world.me.chiv.kick = 0;
        g.world.me.chiv.stun = 0;
        let up = Input {
            buttons: crate::input::HOLD_LOOK,
            ..Input::default()
        };
        assert!(crate::chivalry::guards(&g.world, 0, &up));
    }

    /// At an eighth of full speed a second of the page's time runs 7 or 8
    /// ticks (60 / 8 = 7.5); at full speed, 60 (one frame's catch-up either
    /// way). Paused, none; a step, exactly one.
    #[test]
    fn the_watched_clock_runs_at_its_speed_pauses_and_steps_one_tick() {
        let mut g = watching();
        frames(&mut g, 1);
        let t0 = g.world.tick;
        frames(&mut g, 1);
        let eighth = g.world.tick - t0;
        assert!((7..=8).contains(&eighth), "{eighth} ticks at an eighth");
        g.act("watch.speed", "1").unwrap();
        let t0 = g.world.tick;
        frames(&mut g, 1);
        let full = g.world.tick - t0;
        assert!((59..=61).contains(&full), "{full} ticks at full speed");
        g.act("watch.pause", "").unwrap();
        let t0 = g.world.tick;
        frames(&mut g, 1);
        assert_eq!(g.world.tick, t0, "paused");
        g.act("watch.step", "").unwrap();
        frames(&mut g, 1);
        assert_eq!(g.world.tick, t0 + 1, "one step, one tick");
        assert!(g.watching.paused, "and still paused");
        assert_eq!(
            g.act("watch.speed", "3"),
            Err("A speed is 1, 2, 4 or 8, not \"3\".".into())
        );
    }

    /// Each reading of a tree is drawn as the tree was read: the rule that
    /// fired has every check true; each rule above it either has a false
    /// check with nothing after it reached, or (read for an interrupt) was
    /// passed over whole as it does not interrupt; every rule below it is
    /// not reached. Seen across a whole watched fight, for both fighters.
    #[test]
    fn a_tree_is_drawn_as_it_was_read() {
        let mut g = watching();
        g.act("watch.speed", "1").unwrap();
        let mut readings = 0;
        let mut fired = 0;
        let mut left = 0;
        for _ in 0..40 {
            frames(&mut g, 1);
            let book = &g.world.pilots.book;
            let brains = [g.world.watch.clone().unwrap(), g.world.foes[0].brain.clone()];
            for (k, b) in brains.into_iter().enumerate() {
                let Some(t) = b.trace else { continue };
                readings += 1;
                left += (k == 0) as i32;
                let tree = &book.trees[t.tree.as_str()];
                let top = t.fired.unwrap_or(tree.len());
                if t.fired.is_some() {
                    fired += 1;
                    assert!(t.checks[top].iter().all(|c| *c == Some(true)), "{t:?}");
                }
                for (i, checks) in t.checks.iter().enumerate() {
                    if i > top || (i == top && t.fired.is_none()) {
                        assert!(
                            checks.iter().all(|c| c.is_none()),
                            "rule {i} below the one that fired"
                        );
                    } else if i < top {
                        let passed_over = t.interrupts && !tree[i].interrupt;
                        if passed_over {
                            assert!(checks.iter().all(|c| c.is_none()));
                            continue;
                        }
                        let first_false = checks.iter().position(|c| *c == Some(false));
                        let k = first_false.expect("a rule above the fired one has a false check");
                        assert!(checks[..k].iter().all(|c| *c == Some(true)));
                        assert!(checks[k + 1..].iter().all(|c| c.is_none()));
                    }
                }
            }
        }
        assert!(readings > 40 && fired > 20, "{readings} readings, {fired} fired");
        assert!(left > 20, "the left fighter's tree is read too: {left}");
    }

    /// The pickers step through the pilots and start a new fight; one can
    /// be named; a name not in the list is refused. Whose tree is shown and
    /// the overlay change only what is drawn.
    #[test]
    fn the_pickers_step_through_the_pilots_and_refuse_a_stranger() {
        let mut g = watching();
        g.act("watch.left", "").unwrap();
        assert_eq!(
            g.world.watch.as_ref().unwrap().key,
            "banneret",
            "after the veteran"
        );
        g.act("watch.right", "mirror").unwrap();
        assert_eq!(g.world.foes[0].brain.key, "mirror");
        g.act("watch.right", "").unwrap();
        assert_eq!(g.world.foes[0].brain.key, "knight", "round to the first");
        assert_eq!(
            g.act("watch.left", "dragon"),
            Err("There is no pilot \"dragon\" to watch.".into())
        );
        g.act("watch.show", "right").unwrap();
        g.act("watch.overlay", "off").unwrap();
        let v = view(&g);
        assert_eq!((v.show, v.labels.len()), (1, 0));
    }

    /// T11: each side's weapon steps through the list and is held by its
    /// fighter, round after round; "its own" again gives the body's own.
    #[test]
    fn a_side_s_weapon_is_given_and_kept_through_the_rounds() {
        use crate::pose::Held;
        let mut g = watching();
        assert_eq!(g.world.me.held, Held::Longsword, "the veteran's own");
        g.act("watch.left_weapon", "").unwrap();
        g.act("watch.left_weapon", "").unwrap();
        // five steps on the right: its own, longsword, broadsword, axe,
        // great axe, mace
        for _ in 0..5 {
            g.act("watch.right_weapon", "").unwrap();
        }
        assert_eq!(g.world.me.held, Held::Broadsword);
        assert_eq!(g.world.foes[0].fighter.held, Held::Mace);
        assert_eq!(g.world.foes[0].fighter.arm.weapon.item, Held::Mace);
        crate::rounds::opening(&mut g.world);
        assert_eq!(
            (g.world.me.held, g.world.foes[0].fighter.held),
            (Held::Broadsword, Held::Mace),
            "kept"
        );
        for _ in 0..3 {
            g.act("watch.right_weapon", "").unwrap();
        }
        assert_eq!(g.world.foes[0].fighter.held, Held::Longsword, "its own again");
        assert!(view(&g).sides.len() == 2);
    }

    /// A drag of 100 px turns the camera 30° (5 461 angle units) and its
    /// pitch is held between 60° and 3° down; a wheel notch moves it 10%,
    /// held between 3 and 14 m.
    #[test]
    fn the_camera_circles_and_comes_closer_within_its_limits() {
        let mut g = watching();
        let (yaw, d) = (g.watching.yaw, g.watching.dist);
        g.act("watch.orbit", "100 0").unwrap();
        assert_eq!(fx::wrap(yaw - g.watching.yaw), 5461);
        g.act("watch.orbit", "0 -5000").unwrap();
        assert_eq!(g.watching.pitch, ORBIT_PITCH_MAX);
        g.act("watch.orbit", "0 5000").unwrap();
        assert_eq!(g.watching.pitch, ORBIT_PITCH_MIN);
        g.act("watch.zoom", "-1.5").unwrap();
        assert_eq!(g.watching.dist, d * 90 / 100);
        g.act("watch.zoom", "0").unwrap();
        assert_eq!(g.watching.dist, d * 90 / 100, "no way, no change");
        assert!(g.act("watch.zoom", "up").is_err());
        let p = g.watching.pitch;
        g.act("watch.orbit", "0.6 -0.4").unwrap();
        assert_eq!(g.watching.pitch, p, "under a pixel: no turn");
        for _ in 0..40 {
            g.act("watch.zoom", "-1").unwrap();
        }
        assert_eq!(g.watching.dist, ORBIT_DIST_MIN);
        let c = camera(&g);
        let right = facing(g.watching.yaw, 0).cross(V3::UP).norm();
        let at = middle(&g)
            .add(v3(0, ORBIT_LOOK_UP, 0))
            .add(right.scale(ORBIT_DIST_MIN * ORBIT_SHIFT_PCT / 100));
        assert!(
            (c.eye.dist(at) - ORBIT_DIST_MIN).abs() < 64,
            "3 m from the point it looks at"
        );
    }

    /// The same fight twice from the same seed is the same fight (CA-1).
    #[test]
    fn a_watched_fight_replays_exactly() {
        let run = || {
            let mut g = watching();
            g.watching.fights = 0;
            start(&mut g);
            g.act("watch.speed", "1").unwrap();
            frames(&mut g, 20);
            g.world.hash()
        };
        assert_eq!(run(), run());
    }
}
