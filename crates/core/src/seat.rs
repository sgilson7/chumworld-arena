//! The second seat (M5, CA-Q10: A): online, the foe in the ring is a
//! person in another browser, seen from inside its own helmet.
//!
//! Both browsers run the same `World`, tick for tick, from the same two
//! records (`online.rs`, `rollback.rs`): the host's is the player's
//! (fighter 0, Knight A), the guest's is the second seat's (fighter 1,
//! Knight B). The world was written round fighter 0, whose view is
//! `World::aim` and whose guard rides `World::guard_pitch`; a foe looks
//! where its pilot holds on. The seat gives fighter 1 the same two things
//! (`Seat::aim`, `Seat::guard_pitch`) and runs the player's first-person
//! rules on it, through the same functions (`player::first_person_looks`,
//! `ride_guard_of`, `keep_guard_of`). Its record reaches the tick through
//! the pilots' door (`pilot::Control::Seat`), so the bodies, arms, blades
//! and wounds take it exactly as they take a pilot's (CA-3).
//!
//! What stays the player's alone: the soft pull (online duels play with no
//! aim help) and the middle button's change of foe (there is one foe).
//!
//! The guest's page draws the same world turned round (`face`): the seat's
//! knight put in the player's place, so the camera, the bars, the lines and
//! the result read from the guest's side with no second set of screens.

use crate::camera::Aim;
use crate::input::Input;
use crate::online::{Sim, SEATS};
use crate::pose::Kind;
use crate::world::World;

/// The second seat's fighter number.
pub const ID: usize = 1;

/// The bit a seat's record holds while its player asks for a rematch, once
/// the duel is over. The top key bit: no key sends it (`input::ACTIONS`);
/// the game sets it (`versus.rs`). When both seats' records hold it on one
/// tick, the duel begins again on that tick in both browsers.
pub const REMATCH: u32 = 1 << 23;

/// The seed every online duel's world starts from: both browsers must start
/// alike, and the duel's chance is the world's (`rng.rs`).
pub const SEED: u64 = 0x00c4_a3b1_7e5d_2f09;

/// The knights: the host wears A, the guest B (CA-Q13: A).
pub const HOST_KNIGHT: Kind = Kind::KnightA;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seat {
    /// where the seat's knight looks, as `World::aim` is the player's
    pub aim: Aim,
    /// its guard's pitch against its view, as `World::guard_pitch`
    pub guard_pitch: i32,
    /// this tick's record
    pub input: Input,
}

/// Whether fighter `id` is the second seat's knight.
pub fn is(w: &World, id: usize) -> bool {
    id == ID && w.seat.is_some() && w.foes.len() == 1
}

/// A new online duel: the ring in first person, both knights at their
/// opening marks, round 1. Every setting the tick reads is the default in
/// both browsers (no aim help, no inverted pitch, look speed 100%): each
/// reader's own look speed and pitch are applied to the record before it is
/// sent (`versus::scaled`).
pub fn duel() -> World {
    let mut w = World::new(SEED);
    w.settings.control = crate::world::Control::FirstPerson;
    w.seat = Some(Seat {
        aim: Aim::new(crate::world::START_YAW),
        guard_pitch: crate::arm::GUARD_PITCH,
        input: Input::default(),
    });
    crate::rounds::begin(&mut w, crate::game::PRACTICE_FOE, HOST_KNIGHT)
        .expect("the knight is in the pilots' book");
    w
}

/// At each round's opening (`rounds::opening`): the foe is stood up as the
/// player is (`Fighter::new`, not at a pilot's guard), on the seat's
/// record, looking at the player from its mark.
pub fn opening(w: &mut World, at: crate::geom::V3, yaw: i32) {
    if w.seat.is_none() || w.foes.len() != 1 {
        return;
    }
    let pct = w.aim.pct;
    let f = &mut w.foes[0];
    let kind = f.fighter.kind;
    f.fighter = crate::player::Fighter::new(kind, crate::models::kind_held(kind), 1, at, yaw, true);
    f.brain.control = crate::pilot::Control::Seat;
    if let Some(s) = w.seat.as_mut() {
        s.aim = Aim {
            yaw,
            pitch: crate::camera::PITCH_START,
            rest: (0, 0),
            pct,
        };
        s.guard_pitch = crate::arm::GUARD_PITCH;
    }
}

/// The seat's record as the duel takes it: nothing between rounds or once
/// the duel is over, as the player's (`rounds::input`).
pub fn gate(w: &mut World) {
    let Some(i) = w.seat.as_ref().map(|s| s.input) else {
        return;
    };
    let gated = crate::rounds::input(w, &i);
    if let Some(s) = w.seat.as_mut() {
        s.input = gated;
    }
}

/// Step 2 for the seat: its mouse turns its view (unless it moves the hand
/// or gathers a cut), and its knight looks where the view does.
pub fn look(w: &mut World) {
    if !is(w, ID) {
        return;
    }
    let Some(input) = w.seat.as_ref().map(|s| s.input) else {
        return;
    };
    let looks = crate::player::first_person_looks(&w.foes[0].fighter, &input);
    let m = crate::player::view_mouse(w, &input);
    w.foes[0].fighter.mouse_looks = looks;
    let Some(s) = w.seat.as_mut() else {
        return;
    };
    if looks {
        s.aim = s.aim.turned(m);
    }
    let (yaw, pitch) = (s.aim.yaw, s.aim.pitch);
    let f = &mut w.foes[0].fighter;
    f.look_yaw = yaw;
    f.look_pitch = pitch;
}

/// A hard blow's hit-stop (`feel::hold`): the seat's view turns and its
/// hand moves, as the player's do.
pub fn hold(w: &mut World) {
    if !is(w, ID) {
        return;
    }
    let Some(input) = w.seat.as_ref().map(|s| s.input) else {
        return;
    };
    if w.foes[0].fighter.mouse_looks {
        let m = crate::player::view_mouse(w, &input);
        if let Some(s) = w.seat.as_mut() {
            s.aim = s.aim.turned(m);
        }
    }
    crate::arm::move_shell(w, ID, &input);
}

/// The view's pitch from its opening: the guard rides by this.
fn lift(s: &Seat) -> i32 {
    s.aim.pitch - crate::camera::PITCH_START
}

/// Before the arms move: the seat's guard rides with its view.
pub fn ride_guard(w: &mut World) -> Option<i32> {
    if !is(w, ID) {
        return None;
    }
    let s = w.seat.as_ref()?;
    let (guard, l) = (s.guard_pitch, lift(s));
    crate::player::ride_guard_of(&mut w.foes[0].fighter, guard, l)
}

/// After the arms move: what moved the seat's hand besides the ride sets
/// where its guard sits against its view.
pub fn keep_guard(w: &mut World, ridden: Option<i32>) {
    if !is(w, ID) {
        return;
    }
    let f = &w.foes[0].fighter;
    if let Some(s) = w.seat.as_mut() {
        let l = lift(s);
        crate::player::keep_guard_of(f, &mut s.guard_pitch, l, ridden);
    }
}

/// The duel is over and both seats' records ask for a rematch.
pub fn both_ask_rematch(w: &World, inputs: &[Input; SEATS]) -> bool {
    let over = w
        .rounds
        .as_ref()
        .is_some_and(|r| r.phase == crate::rounds::Phase::Over);
    over && inputs.iter().all(|i| i.keys & REMATCH != 0)
}

/// One online tick: both records into the world. The host's record is the
/// player's; the guest's is the seat's. A rematch both asked for begins the
/// duel again on this tick, from round 1, keeping the world's tick and
/// chance so both browsers go on alike.
impl Sim for World {
    fn tick(&mut self, inputs: [Input; SEATS]) {
        if both_ask_rematch(self, &inputs) {
            crate::rounds::begin(self, crate::game::PRACTICE_FOE, HOST_KNIGHT)
                .expect("the knight is in the pilots' book");
        }
        if let Some(s) = self.seat.as_mut() {
            s.input = inputs[ID];
        }
        self.step(&inputs[0]);
    }

    fn sum(&self) -> u64 {
        self.hash()
    }
}

/// The other side's number for a fighter, in a two-seat duel.
fn flip(id: u8) -> u8 {
    if id < 2 {
        1 - id
    } else {
        id
    }
}

/// The world turned round for the guest's page: the seat's knight in the
/// player's place (its feet, fighter, view and guard), and every per-side
/// record of the rounds, the blades and the blood turned with it. Drawing
/// only: the world the duel runs on is never turned (`versus.rs` turns a
/// copy).
pub fn face(w: &mut World) {
    let Some(s) = w.seat.as_mut() else {
        return;
    };
    if w.foes.len() != 1 {
        return;
    }
    let f = &mut w.foes[0];
    std::mem::swap(&mut w.player, &mut f.body);
    std::mem::swap(&mut w.me, &mut f.fighter);
    std::mem::swap(&mut w.aim, &mut s.aim);
    std::mem::swap(&mut w.guard_pitch, &mut s.guard_pitch);
    for f in [&mut w.me, &mut w.foes[0].fighter] {
        f.engaged = f.engaged.map(flip);
    }
    if let Some(r) = w.rounds.as_mut() {
        r.wins.swap(0, 1);
        r.last_blow.swap(0, 1);
        r.outside.swap(0, 1);
        r.bodies.swap(0, 1);
        if let Some(crate::rounds::Fall::One { who, .. }) = r.fall.as_mut() {
            *who = flip(*who);
        }
    }
    let b = &mut w.blades;
    for pair in b.clashing.iter_mut().chain(b.touching.iter_mut()) {
        *pair = (flip(pair.0), flip(pair.1));
    }
    for h in b.hits.iter_mut() {
        h.by = flip(h.by);
        h.on = flip(h.on);
    }
    for d in w.feel.downed.iter_mut() {
        *d = flip(*d);
    }
    for p in w.wounds.pools.iter_mut() {
        p.0 = flip(p.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{self, FORWARD, HOLD_LOOK, SWING};

    fn run(inputs: &[[Input; SEATS]]) -> World {
        let mut w = duel();
        for i in inputs {
            w.tick(*i);
        }
        w
    }

    /// The seat's knight is Knight B, standing as the player stands: whole,
    /// drawn, on the seat's record, with the same health and arms.
    #[test]
    fn the_guest_s_knight_stands_as_the_host_s_does() {
        let w = duel();
        assert_eq!(w.me.kind, Kind::KnightA);
        let f = &w.foes[0];
        assert_eq!(f.fighter.kind, Kind::KnightB);
        assert_eq!(f.brain.control, crate::pilot::Control::Seat);
        assert_eq!(f.fighter.health, {
            let mut h = w.me.health;
            h.limbs.plate = crate::plate::full(Kind::KnightB);
            h
        });
        assert_eq!(f.fighter.servo_pct, w.me.servo_pct);
        assert_eq!(
            f.fighter.arm.shell, w.me.arm.shell,
            "the same guard as the player's"
        );
        let s = w.seat.as_ref().unwrap();
        // facing the player across the ring: half a turn from the player's view
        // (the heading back is measured to within an angle unit)
        let apart = crate::fx::wrap(s.aim.yaw - w.aim.yaw).abs();
        assert!(crate::fx::TURN / 2 - apart <= 1, "{apart}");
    }

    /// The seat's mouse turns its own view, by the same amount the player's
    /// mouse turns the player's; the player's view stays where it was.
    #[test]
    fn the_guest_s_mouse_turns_the_guest_s_view_as_the_host_s_turns_the_host_s() {
        let turn = Input {
            dx: 10_000,
            ..Input::default()
        };
        let w0 = duel();
        let host = run(&[[turn, Input::default()]]);
        let guest = run(&[[Input::default(), turn]]);
        let s0 = w0.seat.as_ref().unwrap().aim.yaw;
        // 100 px at 0.12° a pixel: 12°, 2 184 angle units, to the right
        assert_eq!(host.aim.yaw, w0.aim.yaw - 2184);
        // (a heading is kept wrapped once turned, so angles compare by difference)
        assert_eq!(crate::fx::wrap(host.seat.as_ref().unwrap().aim.yaw - s0), 0);
        assert_eq!(
            crate::fx::wrap(guest.seat.as_ref().unwrap().aim.yaw - (s0 - 2184)),
            0
        );
        assert_eq!(guest.aim.yaw, w0.aim.yaw);
        // and its knight faces where its view does
        let facing = guest.foes[0].body.facing;
        assert_eq!(crate::fx::wrap(facing - guest.seat.as_ref().unwrap().aim.yaw), 0);
    }

    /// The right button frees the seat's hand: the mouse then moves the
    /// sword hand on its shell and the view holds still, as for the player.
    #[test]
    fn the_guest_s_right_button_moves_the_hand_and_not_the_view() {
        let free = Input {
            buttons: HOLD_LOOK,
            dx: 4_000,
            dy: -3_000,
            ..Input::default()
        };
        let w0 = duel();
        let w = run(&[[Input::default(), free]]);
        assert_eq!(w.seat.as_ref().unwrap().aim, w0.seat.as_ref().unwrap().aim);
        assert!(!w.foes[0].fighter.mouse_looks);
        assert_ne!(w.foes[0].fighter.arm.shell, w0.foes[0].fighter.arm.shell);
    }

    /// The same records give the same world whichever seat they come from:
    /// the host's walk and cut, given to the guest, move the guest's knight
    /// as they moved the host's (each measured from its own mark).
    #[test]
    fn a_record_moves_either_seat_s_knight_alike() {
        let script: Vec<Input> = (0..90)
            .map(|t| Input {
                keys: FORWARD,
                buttons: if (30..50).contains(&t) { SWING } else { 0 },
                dx: if t == 30 { -6_000 } else { 0 },
                ..Input::default()
            })
            .collect();
        let host = run(&script.iter().map(|i| [*i, Input::default()]).collect::<Vec<_>>());
        let guest = run(&script.iter().map(|i| [Input::default(), *i]).collect::<Vec<_>>());
        let w0 = duel();
        let walked = |now: crate::geom::V3, then: crate::geom::V3| now.sub(then).len();
        let a = walked(host.player.pos, w0.player.pos);
        let b = walked(guest.foes[0].body.pos, w0.foes[0].body.pos);
        assert!(a > crate::fx::ONE, "the host walked {a}");
        assert!((a - b).abs() <= crate::fx::ONE / 100, "host {a}, guest {b}");
        // the hand on its shell, which is the body's own frame, went alike,
        // to within the rounding of two bodies facing other ways
        let (h, g) = (host.me.arm.shell, guest.foes[0].fighter.arm.shell);
        for (x, y) in [(h.side, g.side), (h.pitch, g.pitch), (h.reach, g.reach)] {
            assert!((x - y).abs() <= 4, "host {h:?}, guest {g:?}");
        }
        assert_ne!(host.me.arm.shell, w0.me.arm.shell, "the cut moved the hand");
    }

    /// Turned round for the guest's page, the guest's knight is the player
    /// and its view is the world's; the score reads from its side.
    #[test]
    fn the_guest_s_page_sees_the_world_turned_round() {
        let mut w = duel();
        w.rounds.as_mut().unwrap().wins = [1, 0];
        let host_aim = w.aim;
        let seat_aim = w.seat.as_ref().unwrap().aim;
        let mut g = w.clone();
        face(&mut g);
        assert_eq!(g.me.kind, Kind::KnightB);
        assert_eq!(g.foes[0].fighter.kind, Kind::KnightA);
        assert_eq!(g.aim, seat_aim);
        assert_eq!(g.seat.as_ref().unwrap().aim, host_aim);
        assert_eq!(g.rounds.as_ref().unwrap().wins, [0, 1]);
        assert_eq!(g.player, w.foes[0].body);
        // turned twice, it is the world again
        face(&mut g);
        assert_eq!(g, w);
    }

    /// Once the duel is over, a rematch begins only when both seats ask on
    /// the same tick; then both knights stand whole at round 1.
    #[test]
    fn a_rematch_needs_both_seats() {
        let mut w = duel();
        {
            let r = w.rounds.as_mut().unwrap();
            r.phase = crate::rounds::Phase::Over;
            r.wins = [2, 1];
            r.round = 3;
        }
        w.me.health.hp = 0;
        let ask = Input {
            keys: REMATCH,
            ..Input::default()
        };
        w.tick([ask, Input::default()]);
        assert_eq!(w.rounds.as_ref().unwrap().phase, crate::rounds::Phase::Over);
        w.tick([Input::default(), ask]);
        assert_eq!(w.rounds.as_ref().unwrap().phase, crate::rounds::Phase::Over);
        w.tick([ask, ask]);
        let r = w.rounds.as_ref().unwrap();
        assert_eq!(
            (r.round, r.wins, r.phase),
            (1, [0, 0], crate::rounds::Phase::Fight)
        );
        assert!(!w.me.health.down());
        assert_eq!(w.foes[0].brain.control, crate::pilot::Control::Seat);
        // and no key the page sends is the rematch
        assert!(input::ACTIONS.iter().all(|(_, bit)| bit & REMATCH == 0));
    }
}
