//! The ring and the order of one tick.
//!
//! `World` holds everything that changes in a duel; `step` advances it one
//! tick (1/60 s) from the player's input record of that tick, and nothing
//! else changes it: the same seed and the same records give the same world,
//! tick for tick, and `hash` says so in one number.
//!
//! M0 (CA-2): this is CHUMWORLD's `world.rs` (commit d9cb5b5) with the world
//! taken out. What is left is one flat stone floor, `RADIUS` across from the
//! middle, with a wall at its edge that keeps every body in (`keep_in`), and
//! the fighters on it. The tick keeps CHUMWORLD's order for the parts that
//! remain; each step CHUMWORLD ran that the ring does not is listed in
//! SECOND-ORDER.md (R2.1 to R2.6).
//!
//! # The tick, in order
//!
//! 1. a hard blow's hit-stop holds the world still (`feel::hold`);
//! 2. the player's aim turns by the record's mouse movement, and every
//!    fighter holds on the one it fights (`player::look_mouse`, `hold_on`);
//! 3. pilots read the world and return each foe's `Input` (CA-3);
//! 4. bodies move on their feet (`motion`), and the ring's wall keeps them in;
//! 5. wounds steer, then arms, then blades, then health, then the feel of
//!    the blow, then the blades settle;
//! 6. blood and severed parts fly and land; poses are kept; pilots remember.

use crate::camera::{self, Aim};
use crate::fx::{self, Fx, ONE};
use crate::geom::{v3, V3};
use crate::input::{Input, Mouse};
use crate::look::Look;
use crate::page::Strings;
use crate::player::Body;
use crate::render::Shapes;
use crate::rng::Rng;

/// How far the ring's floor runs from its middle to the foot of its wall.
pub const RADIUS: Fx = fx::int(20);
/// A body's circle against the wall: CHUMWORLD's (`solid::RADIUS`).
pub const BODY_RADIUS: Fx = ONE * 35 / 100;
/// The heading the duel opens with: CHUMWORLD's start (`world::START_YAW`),
/// so the duel `port_keeps_chumworld_duel` replays is the game's own opening.
pub const START_YAW: i32 = fx::deg(20);
/// Where the player stands at the start: about 3 m behind the middle along
/// `START_YAW` (3 sin 20°, 3 cos 20°), so the foe spawned 6 m ahead
/// (`player::SPAWN_AHEAD`) stands about as far beyond it.
pub const START: V3 = v3(ONE * 103 / 100, 0, ONE * 282 / 100);

/// A line over the game, by id; `shown` false keeps it in the page but out of
/// sight, so the page can fade it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HudItem {
    pub id: String,
    pub text: String,
    pub shown: bool,
}

/// How the player's mouse and view work (M1, CA-Q6: A).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    /// CHUMWORLD's, as ported (d9cb5b5): the camera over the right
    /// shoulder; drawn, the mouse moves the sword hand and the view holds on
    /// the nearest foe, and the right button looks. The port's proof,
    /// `port_keeps_chumworld_duel`, runs on this path (R3.1).
    Shoulder,
    /// The eye in the helmet (`player::eye`): the mouse turns the view and
    /// the body, the reach shell and the guard ride with the view, the
    /// right button frees the hand (the view then holds still), and a held
    /// left button reads a cut's gesture. Nothing turns the view for the
    /// player unless `AimHelp::SoftPull` is set. The game's own (`game.rs`).
    FirstPerson,
}

/// Help keeping the foe in view (T5, CA-Q16: C and the person's note).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AimHelp {
    /// none: the view turns only by the mouse (the default)
    None,
    /// a gentle turn of the view's heading while the foe held on is near
    /// the view's edge (`player::soft_pull`)
    SoftPull,
}

/// The field of view a reader may set, degrees across the view (CA-Q15: B).
pub const FOV_MIN: i32 = 100;
pub const FOV_MAX: i32 = 110;
pub const FOV_DEFAULT: i32 = 105;

/// The reader's settings the combat reads. CHUMWORLD kept them in its menus
/// (`ui::Settings`); the defaults are its defaults, and M1's control is
/// CHUMWORLD's until the game sets the first-person one (`game.rs`, R3.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// the sword hand's speed under the mouse, percent
    pub hand_pct: i32,
    /// the swing cue: the cut the mouse would pick (`swing::cue`)
    pub cue: bool,
    /// blood is shown (`wounds.rs`)
    pub blood: bool,
    /// how the mouse and the view work (M1)
    pub control: Control,
    /// help keeping the foe in view (M1, T5)
    pub aim_help: AimHelp,
    /// the field of view across the view, degrees, `FOV_MIN` to `FOV_MAX`
    pub fov: i32,
    /// moving the mouse up looks down (first person only)
    pub invert_pitch: bool,
    /// M4: stamina, feints, follow-ups, ripostes and the kick
    /// (`chivalry.rs`); off on the port's path, on in the game (R6.1)
    pub chivalry: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            hand_pct: 100,
            cue: true,
            blood: true,
            control: Control::Shoulder,
            aim_help: AimHelp::None,
            fov: FOV_DEFAULT,
            invert_pitch: false,
            chivalry: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World {
    /// ticks since the world began
    pub tick: u64,
    /// chance for this session, from the page's seed (`rng.rs`)
    pub rng: Rng,
    /// where the player looks
    pub aim: Aim,
    /// the player's feet; its arms, health and footing are `me`
    pub player: Body,
    pub me: crate::player::Fighter,
    /// the foes, their pilots' book and memory, the blades' contacts, the
    /// motion's test settings, and the physics view
    pub foes: Vec<crate::pilot::Foe>,
    pub pilots: crate::pilot::State,
    pub blades: crate::blade::State,
    pub motion: crate::motion::State,
    pub sticks: bool,
    /// the hit-stop, the camera's knock, sparks and the tick's events
    pub feel: crate::feel::State,
    /// blood in flight and on the ground, parts taken off, the cues
    pub wounds: crate::wounds::State,
    pub settings: Settings,
    /// M1: first person, the sword hand's pitch on its shell less the
    /// view's own pitch from its opening (`player::ride_guard`), so the
    /// guard rides up and down with the view
    pub guard_pitch: i32,
    /// M2: the rounds of a duel, when the duel is fought in rounds
    /// (`rounds.rs`); none on the port's path
    pub rounds: Option<crate::rounds::Rounds>,
    /// M1: the frame is drawn from the player's eye (the game sets it for
    /// a duel in first person): the player's head is then not drawn
    pub eye_view: bool,
    /// M5: the second seat, online: the foe is a person in another browser,
    /// seen from its own helmet (`seat.rs`); none offline
    pub seat: Option<crate::seat::Seat>,
    /// T11: watch mode: the pilot that flies the player's fighter, its
    /// record taking the page's place each tick (`pilot::decide_with`)
    pub watch: Option<crate::pilot::Brain>,
}

impl World {
    /// The ring with the player alone at `START`, sword sheathed. A foe
    /// comes with `duel.spawn` (`player::act`).
    pub fn new(seed: u64) -> World {
        World {
            tick: 0,
            rng: Rng::new(seed),
            aim: Aim::new(START_YAW),
            player: Body::new(START, START_YAW),
            me: crate::player::Fighter::new(
                crate::pose::Kind::Player,
                crate::pose::Held::ShortSword,
                0,
                START,
                START_YAW,
                false,
            ),
            foes: Vec::new(),
            pilots: crate::pilot::State::new(),
            blades: crate::blade::State::default(),
            motion: crate::motion::State::default(),
            sticks: false,
            feel: crate::feel::State::default(),
            wounds: crate::wounds::State::default(),
            settings: Settings::default(),
            guard_pitch: crate::arm::GUARD_PITCH,
            rounds: None,
            eye_view: false,
            seat: None,
            watch: None,
        }
    }

    /// Whether the player sees from the helmet (M1).
    pub fn first_person(&self) -> bool {
        self.settings.control == Control::FirstPerson
    }

    /// The ground's height at (x, z): the ring's floor is flat at 0
    /// (CHUMWORLD read its terrain here, R2.2).
    pub fn ground(&self, _x: Fx, _z: Fx) -> Fx {
        0
    }

    /// One tick, in the order the module doc gives.
    pub fn step(&mut self, input: &Input) {
        // a world frozen for a picture of a spark (test-only): only the view turns
        if self.feel.frozen {
            crate::feel::frozen(self, input);
            return;
        }
        self.tick += 1;
        // T11: watched, a pilot's record stands in for the page's
        let watched;
        let input = match self.watch.clone() {
            Some(b) => {
                let (i, b) = crate::pilot::decide_with(self, 0, b);
                self.watch = Some(b);
                watched = i;
                &watched
            }
            None => input,
        };
        // (M2: between rounds, and once a duel is over, the player's record
        // is set aside)
        let input = &crate::rounds::input(self, input);
        // M5: the second seat's record is set aside the same way
        crate::seat::gate(self);
        // 1. a hard blow holds the world still for a few ticks
        if self.feel.stop > 0 {
            crate::feel::hold(self, input);
            crate::wounds::hold(self); // and the blood and pieces
            return;
        }
        // 2. the aim (drawn, the mouse moves the hand instead, and the view
        // holds on the foe)
        let look = crate::player::look_mouse(self, input);
        self.aim = self.aim.turned(look);
        // M5: the second seat's view, as the player's
        crate::seat::look(self);
        crate::player::hold_on(self);
        // 3. each foe's record, from its pilot
        let inputs = crate::pilot::step(self);
        // 4. bodies, and the ring's wall
        crate::motion::step(self, input, &inputs);
        keep_in(self);
        // 5. wounds tick (a dropped weapon settles or is taken up), arms,
        // blades, health, the feel of the blow (M1: in first person the
        // guard rides with the view first)
        crate::wounds::steer(self, input, &inputs);
        let ridden = crate::player::ride_guard(self);
        let seat_ridden = crate::seat::ride_guard(self);
        crate::arm::step(self, input, &inputs);
        crate::player::keep_guard(self, ridden);
        crate::seat::keep_guard(self, seat_ridden);
        crate::blade::step(self);
        crate::health::step(self);
        crate::feel::step(self);
        crate::chivalry::step(self, input, &inputs); // M4
        crate::blade::settle(self); // after the blow's stagger
                                    // 6. blood flies and lands, parts taken off fall and rest
        crate::wounds::step(self);
        crate::player::finish(self, input, &inputs);
        crate::pilot::remember(self);
        // M2: a round's clock, sudden death, a fall ending the round
        crate::rounds::step(self);
    }

    /// What moves, drawn `alpha` of the way from the last tick to this one.
    pub fn draw(&self, alpha: Fx, look: &Look, out: &mut Shapes) {
        crate::player::draw(self, alpha, look, out);
        crate::feel::draw(self, alpha, look, out);
        crate::wounds::draw(self, alpha, look, out);
    }

    /// Lines over the game.
    pub fn hud(&self, strings: &Strings, out: &mut Vec<HudItem>) {
        // the cut the mouse would pick while the left button is held
        crate::swing::hud(self, strings, out);
        // taking up a dropped weapon in the other hand
        crate::wounds::hud(self, strings, out);
        // M4: a feint, a follow-up, a riposte, a kick
        crate::chivalry::hud(self, strings, out);
    }

    /// A setting or flag named `module.name`.
    pub fn act(&mut self, name: &str, value: &str) -> Result<(), String> {
        match name.split_once('.') {
            Some(("duel", rest)) => crate::player::act(self, rest, value),
            Some(("motion", rest)) => crate::motion::act(self, rest, value),
            Some(("feel", rest)) => crate::feel::act(self, rest, value),
            Some(("wounds", rest)) => crate::wounds::act(self, rest, value),
            Some(("rounds", rest)) => crate::rounds::act(self, rest, value),
            Some(("chivalry", rest)) => crate::chivalry::act(self, rest, value),
            _ => Err(format!("Unknown action \"{name}\".")),
        }
    }

    /// One number for the whole state, for replays and determinism tests: an
    /// FNV-1a hash of the state's values in field order (`value_hash`).
    pub fn hash(&self) -> u64 {
        value_hash(&format!("{self:?}"))
    }

    /// The camera for this frame: the aim with `pending` mouse movement
    /// applied as the next tick will apply it, over the feet at `alpha`,
    /// `back` from the shoulder's pivot (`camera::BACK` in play).
    pub fn camera(&self, alpha: Fx, pending: Mouse, back: Fx) -> crate::render::Camera {
        // drawn, the mouse moves the hand, not the view
        let aim = if self.me.mouse_looks {
            let flip = self.first_person() && self.settings.invert_pitch;
            self.aim.turned(Mouse {
                dy: if flip { -pending.dy } else { pending.dy },
                ..pending
            })
        } else {
            self.aim
        };
        if self.first_person() {
            // M1: the eye in the helmet, knocked by a hard blow as the
            // shoulder camera is; the game sets the field of view by the
            // reader's setting and the screen's shape (`game.rs`)
            let eye = crate::player::eye(self, alpha, aim).add(crate::feel::knock_at(self, alpha));
            // T10: and tilted with the player's own cut and by a blow taken
            let mut cam = camera::eye_view(eye, aim, camera::FOV);
            cam.roll = crate::feel::tilt_at(self, alpha);
            return cam;
        }
        let feet = self.player.feet(alpha);
        let mut cam = camera::view_from(feet, aim, back, |x, z| self.ground(x, z));
        // a hard blow knocks the view a little; held on a foe, the view
        // frames the duel from a little further right
        cam.eye = cam
            .eye
            .add(crate::feel::knock_at(self, alpha))
            .add(crate::player::framing(self, alpha, aim.yaw));
        // (CHUMWORLD's sprint pulled the view back and its boom kept the
        // camera out of walls, houses and trees; the ring has neither, R2.5)
        cam
    }

    /// The player is down (at no health, fallen). `feel.downed` names
    /// everyone who went down this tick.
    pub fn player_down(&self) -> bool {
        self.me.health.down()
    }
}

/// Step 4's wall: each body's circle is kept inside the ring. A body whose
/// circle would cross the wall is put back on it, along the line from the
/// middle, and loses the part of its velocity that carried it outward.
pub fn keep_in(w: &mut World) {
    let most = RADIUS - BODY_RADIUS;
    for id in 0..crate::player::count(w) {
        let b = crate::player::parts_mut(w, id).0;
        let r = fx::length(b.pos.x, b.pos.z);
        if r <= most {
            continue;
        }
        let out = v3(b.pos.x, 0, b.pos.z).norm();
        b.pos = v3(fx::mul(out.x, most), b.pos.y, fx::mul(out.z, most));
        let along = b.vel.dot(out);
        if along > 0 {
            b.vel = b.vel.sub(out.scale(along));
        }
    }
}

/// An FNV-1a hash of a derived debug text with its field names left out:
/// each `name: ` before a value is skipped, so the hash is of the values
/// (numbers, variants, the shape of lists and structs) in field order. A
/// renamed field keeps the hash; a value changed, added or moved changes
/// it. Integers print the same under every toolchain, and the core holds
/// no float, so a hash written down stays right.
pub fn value_hash(text: &str) -> u64 {
    let b = text.as_bytes();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut put = |c: u8| {
        h ^= c as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    };
    let word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut i = 0;
    while i < b.len() {
        if word(b[i]) && (i == 0 || !word(b[i - 1])) {
            let mut j = i;
            while j < b.len() && word(b[j]) {
                j += 1;
            }
            // a field's name: a word straight before ": " (not "::")
            let named = j + 1 < b.len() && b[j] == b':' && b[j + 1] == b' ';
            if named {
                i = j + 2;
                continue;
            }
            for &c in &b[i..j] {
                put(c);
            }
            i = j;
            continue;
        }
        put(b[i]);
        i += 1;
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{FORWARD, RUN};

    fn run(inputs: &[Input]) -> World {
        let mut w = World::new(7);
        for i in inputs {
            w.step(i);
        }
        w
    }

    fn walk_and_turn() -> Vec<Input> {
        (0..300)
            .map(|k| Input {
                keys: if k % 90 < 60 { FORWARD } else { FORWARD | RUN },
                dx: if k % 50 < 10 { 400 } else { -35 },
                dy: if k % 70 < 5 { 120 } else { 0 },
                ..Input::default()
            })
            .collect()
    }

    #[test]
    fn the_same_inputs_give_the_same_world_tick_for_tick() {
        let a = run(&walk_and_turn());
        let b = run(&walk_and_turn());
        assert_eq!(a, b);
        assert_eq!(a.hash(), b.hash());
        let mut other = walk_and_turn();
        other[150].dx += 1;
        assert_ne!(run(&other).hash(), a.hash(), "one hundredth of a pixel shows");
    }

    #[test]
    fn the_mouse_turns_the_aim_before_the_body_moves_the_same_tick() {
        // one tick with W and a 100 px turn right: the body walks along the
        // new heading, 12° right of the start
        let mut w = World::new(1);
        w.step(&Input {
            keys: FORWARD,
            dx: 10_000,
            ..Input::default()
        });
        assert_eq!(w.aim.yaw, START_YAW - 2184);
        let moved = w.player.pos.sub(START);
        let heading = crate::geom::yaw_of(moved);
        assert!(
            fx::wrap(heading - w.aim.yaw).abs() <= fx::deg(1) / 2,
            "{heading} {}",
            w.aim.yaw
        );
    }

    #[test]
    fn the_camera_shows_pending_mouse_movement_and_the_tick_does_not_jump_it() {
        let w = World::new(1);
        let m = Mouse {
            dx: 2_500,
            dy: 0,
            wheel: 0,
        };
        let before = w.camera(0, m, camera::BACK);
        let mut after = w.clone();
        after.step(&Input {
            dx: m.dx,
            ..Input::default()
        });
        assert_eq!(before, after.camera(0, Mouse::default(), camera::BACK));
    }

    #[test]
    fn an_unknown_action_is_refused_by_name() {
        assert_eq!(
            World::new(1).act("arm.reach", "1"),
            Err("Unknown action \"arm.reach\".".into())
        );
    }

    /// The ring's floor is flat at 0 everywhere, inside the wall and out.
    #[test]
    fn the_ring_s_floor_is_flat_at_nought() {
        let w = World::new(1);
        for (x, z) in [
            (0, 0),
            (fx::int(19), 0),
            (-fx::int(7), fx::int(13)),
            (fx::int(300), 5),
        ] {
            assert_eq!(w.ground(x, z), 0);
        }
    }

    /// A body run at the wall stops with its circle on it: 20 m less 0.35 m
    /// from the middle, never farther, and keeps no speed outward.
    #[test]
    fn the_wall_keeps_a_running_body_inside_the_ring() {
        let mut w = World::new(2);
        // straight ahead from the start, running, for 20 s: far past the wall
        for _ in 0..1200 {
            w.step(&Input {
                keys: FORWARD | RUN,
                ..Input::default()
            });
            let r = fx::length(w.player.pos.x, w.player.pos.z);
            assert!(r <= RADIUS - BODY_RADIUS, "{r}");
        }
        let r = fx::length(w.player.pos.x, w.player.pos.z);
        // 19.65 m, give or take a millimetre of the push back's rounding
        assert!((r - (RADIUS - BODY_RADIUS)).abs() <= ONE / 1000, "{r}");
        let out = v3(w.player.pos.x, 0, w.player.pos.z).norm();
        assert!(w.player.vel.dot(out) <= 0, "no speed outward");
    }

    #[test]
    fn a_body_inside_the_wall_is_left_where_it_is() {
        let mut w = World::new(2);
        let at = v3(fx::int(19), 0, fx::int(1)); // 19.03 m out
        w.player = Body::new(at, 0);
        w.player.vel = v3(ONE / 60, 0, 0);
        keep_in(&mut w);
        assert_eq!(w.player.pos, at);
        assert_eq!(w.player.vel, v3(ONE / 60, 0, 0));
    }

    /// R2.19: the hash reads values, not names. Two structs alike but for
    /// their field names hash the same; a value changed, or two values
    /// swapped, hash otherwise; a variant's name is a value.
    #[test]
    fn the_world_s_hash_reads_its_values_not_its_field_names() {
        // the same type before and after its fields were renamed
        mod before {
            #[derive(Debug)]
            #[allow(dead_code)]
            pub struct S {
                pub hp: i32,
                pub pos: (i32, i32),
                pub mode: Option<u8>,
            }
        }
        mod after {
            #[derive(Debug)]
            #[allow(dead_code)]
            pub struct S {
                pub health: i32,
                pub at: (i32, i32),
                pub state: Option<u8>,
            }
        }
        use after::S as B;
        use before::S as A;
        #[derive(Debug)]
        #[allow(dead_code)]
        enum M {
            Round(u8),
            Rest(u8),
        }
        let a = format!(
            "{:?}",
            A {
                hp: 5,
                pos: (1, 2),
                mode: Some(3)
            }
        );
        let b = format!(
            "{:?}",
            B {
                health: 5,
                at: (1, 2),
                state: Some(3)
            }
        );
        assert_ne!(a, b);
        assert_eq!(value_hash(&a), value_hash(&b), "renamed fields");
        let swapped = format!(
            "{:?}",
            A {
                hp: 5,
                pos: (2, 1),
                mode: Some(3)
            }
        );
        assert_ne!(value_hash(&a), value_hash(&swapped));
        let more = format!(
            "{:?}",
            A {
                hp: 59,
                pos: (1, 2),
                mode: Some(3)
            }
        );
        assert_ne!(
            value_hash(&more),
            value_hash(&format!(
                "{:?}",
                A {
                    hp: 51,
                    pos: (1, 2),
                    mode: Some(3)
                }
            ))
        );
        let none = format!(
            "{:?}",
            A {
                hp: 5,
                pos: (1, 2),
                mode: None
            }
        );
        assert_ne!(value_hash(&a), value_hash(&none));
        assert_ne!(
            value_hash(&format!("{:?}", M::Round(1))),
            value_hash(&format!("{:?}", M::Rest(1)))
        );
        // FNV-1a of "5" alone, by hand: (0xcbf29ce484222325 ^ 0x35) * 0x100000001b3
        let one = (0xcbf2_9ce4_8422_2325u64 ^ 0x35).wrapping_mul(0x0100_0000_01b3);
        assert_eq!(value_hash("hp: 5"), one);
        // the world itself: the same world hashes the same, a step changes it
        let w = World::new(7);
        assert_eq!(w.hash(), World::new(7).hash());
    }
}
