//! Rollback (M5, CA-Q10: A, CA-5): one browser's side of an online duel
//! that does not wait for the other's input.
//!
//! The lockstep session (`online::Session`) runs a tick only once it holds
//! both seats' records for it. This session runs the tick with the other
//! seat's record guessed: the last one it has (`predict`). Before each tick
//! it keeps a copy of the whole world (`Sim` is `Clone`, so whatever the
//! world holds is saved; no field is listed by hand). When the other seat's
//! true record for a tick it has run arrives and differs from the guess, it
//! puts back the copy from before that tick and runs again to where it was,
//! with every record it now holds. The world it ends on is the world the
//! lockstep session reaches from the same records (`tests` below).
//!
//! Limits:
//!
//! - it runs at most `WINDOW` ticks past the first tick whose record from
//!   the other seat has not come (`frontier`); there it waits, so a
//!   rollback never runs more than `WINDOW` ticks again (CA-6's probe);
//! - its own records are used `DELAY` ticks after they are made
//!   (`online::DELAY`), which gives them time to arrive before they are
//!   needed;
//! - checksums are taken only of worlds made from true records on both
//!   sides, so both browsers sum the same world for the same tick.
//!
//! Messages, hellos, the room being full, garbage and checksums are the
//! kit's (`online::Link`).

use crate::input::Input;
use crate::online::{Link, Msg, Peer, Sim, DELAY, DESYNC, GARBLED, SEATS, SUM_EVERY};
use std::collections::{BTreeMap, VecDeque};

/// The most ticks run past the last confirmed one: 8, 133 ms (CA-Q10: A).
pub const WINDOW: u32 = 8;

/// The furthest ahead an honest record from the other seat can be: its own
/// window, both delays and a tick of the order frames run in.
pub const LEAD: u32 = WINDOW + 2 * DELAY + 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    /// rollbacks done, ticks run again in all, and the most in one
    pub rollbacks: u32,
    pub resimulated: u32,
    pub most: u32,
    /// ticks the window held this browser back
    pub stalls: u32,
}

#[derive(Debug, Clone)]
pub struct Rollback<W: Sim> {
    pub seat: usize,
    pub link: Link,
    /// the world as of `tick`, run on true and guessed records
    pub world: W,
    /// the next tick to run
    pub tick: u32,
    /// each seat's records by tick: this browser's own as made, the other's
    /// as they arrive
    inputs: [BTreeMap<u32, Input>; SEATS],
    /// the first tick whose record from the other seat has not come
    pub frontier: u32,
    /// the other seat's record each tick was run with (true or guessed)
    used: BTreeMap<u32, Input>,
    /// the world before each tick from the oldest one that may be run again
    snaps: VecDeque<(u32, W)>,
    /// the earliest tick to run again, once a guess is found wrong
    redo: Option<u32>,
    /// the next tick to sum
    next_sum: u32,
    pub stats: Stats,
    /// this browser's checksums, newest last (the last `MADE_KEPT`)
    pub made: VecDeque<(u32, u64)>,
}

/// How many of its own checksums a session keeps for the page to show.
pub const MADE_KEPT: usize = 16;

impl<W: Sim> Rollback<W> {
    pub fn new(seat: usize, build: &str, world: W) -> Rollback<W> {
        let mut inputs: [BTreeMap<u32, Input>; SEATS] = Default::default();
        for seat_inputs in inputs.iter_mut() {
            for t in 0..DELAY {
                seat_inputs.insert(t, Input::default());
            }
        }
        Rollback {
            seat,
            link: Link::new(build),
            world,
            tick: 0,
            inputs,
            frontier: DELAY,
            used: BTreeMap::new(),
            snaps: VecDeque::new(),
            redo: None,
            next_sum: SUM_EVERY,
            stats: Stats::default(),
            made: VecDeque::new(),
        }
    }

    fn other(&self) -> usize {
        1 - self.seat
    }

    pub fn take_outbox(&mut self) -> Vec<(Peer, String)> {
        self.link.take_outbox()
    }

    pub fn connected(&mut self, peer: Peer) {
        self.link.connected(peer);
    }

    pub fn left(&mut self, peer: Peer) {
        self.link.left(peer);
    }

    /// Text from a browser. A record for a tick already run, that differs
    /// from the guess it was run with, marks that tick to run again.
    pub fn message(&mut self, from: Peer, text: &str) {
        let Some(Msg::Input { tick, input }) = self.link.message(from, text) else {
            return;
        };
        let other = self.other();
        let honest = tick >= self.frontier && tick <= self.tick + LEAD;
        if !honest || self.inputs[other].contains_key(&tick) {
            self.link.status = GARBLED;
            return;
        }
        self.inputs[other].insert(tick, input);
        if tick < self.tick && self.used.get(&tick) != Some(&input) {
            self.redo = Some(self.redo.map_or(tick, |r| r.min(tick)));
        }
        while self.inputs[other].contains_key(&self.frontier) {
            self.frontier += 1;
        }
    }

    /// This browser's record, made now, for the tick `DELAY` ahead; sent
    /// once. A frame that runs no tick adds nothing.
    pub fn local(&mut self, input: Input) {
        if self.link.peer.is_none() {
            return;
        }
        let tick = self.tick + DELAY;
        if self.inputs[self.seat].contains_key(&tick) {
            return;
        }
        self.inputs[self.seat].insert(tick, input);
        self.link.send_peer(&Msg::Input { tick, input });
    }

    /// The other seat's record for tick `t`: the true one if it has come,
    /// else the last one that has (the guess).
    fn predict(&self, t: u32) -> Input {
        let theirs = &self.inputs[self.other()];
        theirs
            .get(&t)
            .or_else(|| theirs.range(..t).next_back().map(|(_, i)| i))
            .copied()
            .unwrap_or_default()
    }

    /// Both seats' records for tick `t`, in seat order.
    fn records(&self, t: u32, theirs: Input) -> [Input; SEATS] {
        let mine = self.inputs[self.seat].get(&t).copied().unwrap_or_default();
        if self.seat == 0 {
            [mine, theirs]
        } else {
            [theirs, mine]
        }
    }

    /// Runs tick `t` from the world as it stands, keeping the copy before it.
    fn run(&mut self, t: u32) {
        let theirs = self.predict(t);
        self.snaps.push_back((t, self.world.clone()));
        self.used.insert(t, theirs);
        let records = self.records(t, theirs);
        self.world.tick(records);
    }

    /// A wrong guess found: back to the copy before its tick, and every
    /// tick since run again.
    fn roll_back(&mut self) {
        let Some(from) = self.redo.take() else {
            return;
        };
        while self.snaps.back().is_some_and(|(t, _)| *t > from) {
            self.snaps.pop_back();
        }
        let Some((at, world)) = self.snaps.pop_back() else {
            return;
        };
        debug_assert_eq!(at, from, "the copy before a wrong guess is kept");
        self.world = world;
        for t in from..self.tick {
            self.run(t);
        }
        let n = self.tick - from;
        self.stats.rollbacks += 1;
        self.stats.resimulated += n;
        self.stats.most = self.stats.most.max(n);
    }

    /// Checksums of every tick now made from true records on both sides.
    fn sum_ready(&mut self) {
        while self.next_sum <= self.tick && self.next_sum <= self.frontier && self.redo.is_none() {
            let t = self.next_sum;
            let sum = if t == self.tick {
                self.world.sum()
            } else {
                match self.snaps.iter().find(|(s, _)| *s == t) {
                    Some((_, w)) => w.sum(),
                    None => return,
                }
            };
            self.link.summed(t, sum);
            self.made.push_back((t, sum));
            if self.made.len() > MADE_KEPT {
                self.made.pop_front();
            }
            self.next_sum += SUM_EVERY;
        }
    }

    /// Forgets copies and records no rollback or checksum can need again.
    fn prune(&mut self) {
        let keep = self.frontier.min(self.next_sum).min(self.tick);
        while self.snaps.front().is_some_and(|(t, _)| *t < keep) {
            self.snaps.pop_front();
        }
        let below = keep.saturating_sub(1);
        self.used = self.used.split_off(&keep);
        for seat_inputs in self.inputs.iter_mut() {
            *seat_inputs = seat_inputs.split_off(&below);
        }
    }

    /// Whether the next tick would run past the window.
    pub fn held_back(&self) -> bool {
        self.tick >= self.frontier + WINDOW
    }

    /// Ticks run on a guess of the other seat's record.
    pub fn guessed(&self) -> u32 {
        self.tick.saturating_sub(self.frontier)
    }

    /// Runs again what a wrong guess spoiled, then the next tick if the
    /// window allows and this browser's record for it is made. Returns
    /// whether the next tick ran.
    pub fn step(&mut self) -> bool {
        if self.link.peer.is_none() || self.link.status == DESYNC {
            return false;
        }
        self.roll_back();
        self.sum_ready();
        let t = self.tick;
        if self.held_back() {
            self.stats.stalls += 1;
            return false;
        }
        if !self.inputs[self.seat].contains_key(&t) {
            return false;
        }
        self.run(t);
        self.tick += 1;
        self.sum_ready();
        self.prune();
        true
    }
}

#[cfg(test)]
// (the two browsers are indexed by seat, as the wire addresses them)
#[allow(clippy::needless_range_loop)]
mod tests {
    use super::*;
    use crate::input::{BACK, FORWARD, HOLD_LOOK, LEFT, RIGHT, SWING};
    use crate::online::tests::{keys, HOST, JOINER};
    use crate::online::{encode, Session, PLAYING};

    /// A seat's record for a tick: walking, turning, freeing the hand and
    /// cutting, changing every few ticks so guesses go wrong often. Fixed by
    /// the seat and the tick alone, so every run sees the same records.
    fn script(seat: usize, t: u32) -> Input {
        if t < DELAY {
            return Input::default();
        }
        let k = t.wrapping_mul(2_654_435_761).rotate_left(seat as u32 * 7 + 3) ^ (t / 11);
        let walk = [FORWARD, FORWARD | LEFT, FORWARD | RIGHT, BACK, 0][(k % 5) as usize];
        let cut = (t / 13 + seat as u32).is_multiple_of(4);
        Input {
            keys: walk,
            buttons: if cut {
                SWING
            } else if k.is_multiple_of(7) {
                HOLD_LOOK
            } else {
                0
            },
            dx: ((k >> 8) % 1_201) as i32 - 600,
            dy: ((k >> 16) % 401) as i32 - 200,
            wheel: 0,
        }
    }

    /// The world one browser reaches with every record on time.
    fn truth(ticks: u32) -> crate::world::World {
        let mut w = crate::seat::duel();
        for t in 0..ticks {
            w.tick([script(0, t), script(1, t)]);
        }
        w
    }

    /// Two rollback sessions that have exchanged hellos.
    fn pair<W: Sim>(world: W) -> (Rollback<W>, Rollback<W>) {
        let mut host = Rollback::new(0, "b", world.clone());
        let mut join = Rollback::new(1, "b", world);
        host.connected(JOINER);
        join.connected(HOST);
        for (_, text) in host.take_outbox() {
            join.message(HOST, &text);
        }
        for (_, text) in join.take_outbox() {
            host.message(JOINER, &text);
        }
        (host, join)
    }

    /// A wire that holds each message a number of frames fixed by its
    /// order, from none to `most`, and hands over what is due in the order
    /// it falls due: later messages often overtake earlier ones.
    struct Wire {
        held: Vec<(usize, u64, usize, String)>,
        sent: u64,
        most: usize,
    }

    impl Wire {
        fn put(&mut self, frame: usize, to: usize, text: String) {
            self.sent += 1;
            let lag = (self.sent.wrapping_mul(0x9e37_79b9) >> 7) as usize % (self.most + 1);
            self.held.push((frame + lag, self.sent, to, text));
        }

        fn due(&mut self, frame: usize) -> Vec<(usize, String)> {
            let mut now: Vec<_> = self.held.iter().filter(|m| m.0 <= frame).cloned().collect();
            self.held.retain(|m| m.0 > frame);
            now.sort_by_key(|m| (m.0, m.1));
            now.into_iter().map(|m| (m.2, m.3)).collect()
        }
    }

    /// CA-5: the same records through lockstep and through rollback, over a
    /// wire that delays and reorders, reach the world a browser fed every
    /// record on time reaches, in both browsers.
    #[test]
    fn rollback_reaches_the_world_lockstep_reaches_under_delay_and_reordering() {
        let ticks = 240;
        let want = truth(ticks);

        // the kit's lockstep session, the reference, over a 3-frame lag
        let mut ls = [Session::new(0, "b"), Session::new(1, "b")];
        ls[0].connected(JOINER);
        ls[1].connected(HOST);
        for (to, from) in [(1, 0), (0, 1)] {
            for (_, text) in ls[from].take_outbox() {
                ls[to].message([HOST, JOINER][from], &text);
            }
        }
        let mut lw = [crate::seat::duel(), crate::seat::duel()];
        let mut wire: VecDeque<(usize, usize, String)> = VecDeque::new();
        for frame in 0..2_000usize {
            for seat in 0..SEATS {
                let s = &mut ls[seat];
                let k = script(seat, s.tick + DELAY);
                s.local(k);
                for (_, text) in s.take_outbox() {
                    wire.push_back((frame + 3, 1 - seat, text));
                }
            }
            while wire.front().is_some_and(|m| m.0 <= frame) {
                let (_, to, text) = wire.pop_front().unwrap();
                ls[to].message([HOST, JOINER][1 - to], &text);
            }
            for seat in 0..SEATS {
                if ls[seat].tick < ticks {
                    ls[seat].step(&mut lw[seat]);
                }
            }
        }
        assert_eq!([ls[0].tick, ls[1].tick], [ticks, ticks], "lockstep stalled");
        assert!(lw[0] == want && lw[1] == want, "lockstep left the true world");

        for most in [0usize, 2, 5, 9] {
            let (host, join) = pair(crate::seat::duel());
            let mut rb = [host, join];
            let mut wire = Wire {
                held: Vec::new(),
                sent: 0,
                most,
            };
            let mut frame = 0usize;
            while (rb.iter().any(|s| s.tick < ticks || s.guessed() > 0) || !wire.held.is_empty())
                && frame < 4_000
            {
                for seat in 0..SEATS {
                    let s = &mut rb[seat];
                    let k = script(seat, s.tick + DELAY);
                    s.local(k);
                    for (_, text) in s.take_outbox() {
                        wire.put(frame, 1 - seat, text);
                    }
                }
                for (to, text) in wire.due(frame) {
                    rb[to].message([HOST, JOINER][1 - to], &text);
                }
                for s in rb.iter_mut() {
                    if s.tick < ticks {
                        s.step();
                    } else {
                        // the last ticks' true records still to come
                        s.roll_back();
                        s.sum_ready();
                    }
                }
                frame += 1;
            }
            for (seat, s) in rb.iter().enumerate() {
                assert_eq!(s.tick, ticks, "lag up to {most}: seat {seat} stalled");
                assert_eq!(s.link.status, PLAYING, "lag up to {most}: seat {seat}");
                assert!(
                    s.world == want,
                    "lag up to {most}: seat {seat} left the true world"
                );
                assert!(
                    s.world == lw[seat],
                    "lag up to {most}: seat {seat} and lockstep differ"
                );
                assert!(s.stats.most <= WINDOW, "{:?}", s.stats);
            }
            // a lag the input delay covers needs no guess at all
            if most > DELAY as usize {
                assert!(
                    rb.iter().all(|s| s.stats.rollbacks > 0),
                    "lag up to {most}: no guess went wrong"
                );
            }
            assert!(
                rb[0].link.agreed.is_some_and(|(t, _)| t >= ticks - SUM_EVERY),
                "lag up to {most}: checksums agreed {:?}",
                rb[0].link.agreed
            );
        }
    }

    /// A misprediction forced: the host runs five ticks guessing the
    /// joiner stood still; the joiner was walking and turning. When the true
    /// records come, the host goes back five ticks and ends on the world
    /// made from the true records.
    #[test]
    fn a_forced_misprediction_recovers_to_the_same_state() {
        let (mut host, _join) = pair(crate::seat::duel());
        let walk = Input {
            keys: FORWARD,
            dx: 3_000,
            ..Input::default()
        };
        let ahead = DELAY + 5;
        for _ in 0..ahead {
            host.local(Input::default());
            assert!(host.step());
        }
        assert_eq!(host.guessed(), 5);
        for t in DELAY..ahead {
            host.message(JOINER, &encode(&Msg::Input { tick: t, input: walk }));
        }
        host.local(Input::default());
        assert!(host.step());
        assert_eq!(host.stats.rollbacks, 1);
        assert_eq!(host.stats.resimulated, 5);
        let mut want = crate::seat::duel();
        for t in 0..=ahead {
            let theirs = if (DELAY..ahead).contains(&t) {
                walk
            } else if t == ahead {
                // the tick after: the guess repeats the joiner's last record
                walk
            } else {
                Input::default()
            };
            want.tick([Input::default(), theirs]);
        }
        assert!(host.world == want, "the host did not recover the true world");
        assert!(host.world.seat.as_ref().unwrap().aim.yaw != crate::seat::duel().seat.unwrap().aim.yaw);
    }

    /// A copy never runs more than `WINDOW` ticks past its confirmed ticks;
    /// a record for a tick outside the window, or a second one for a tick,
    /// changes the status and nothing else.
    #[test]
    fn a_copy_never_runs_more_than_its_window_past_its_confirmed_ticks() {
        let (mut host, _join) = pair(crate::online::World::default());
        let mut ran = 0;
        for _ in 0..50 {
            host.local(keys(crate::online::RIGHT));
            ran += host.step() as u32;
        }
        // the two prefilled ticks, then the window
        assert_eq!(ran, DELAY + WINDOW);
        assert_eq!(host.guessed(), WINDOW);
        assert!(host.stats.stalls > 0);
        // one true record moves the window on by one tick
        host.message(
            JOINER,
            &encode(&Msg::Input {
                tick: DELAY,
                input: keys(0),
            }),
        );
        assert!(host.step());
        assert!(!host.step());

        let before = (host.tick, host.frontier, host.world.clone());
        for junk in [
            // a tick long confirmed, whose record has since been forgotten
            Msg::Input {
                tick: 0,
                input: keys(1),
            },
            Msg::Input {
                tick: DELAY,
                input: keys(1),
            },
            Msg::Input {
                tick: host.tick + LEAD + 1,
                input: keys(1),
            },
        ] {
            host.link.status = PLAYING;
            host.message(JOINER, &encode(&junk));
            assert_eq!(host.link.status, GARBLED, "{junk:?}");
            assert_eq!((host.tick, host.frontier, host.world.clone()), before);
        }
    }

    /// M4 (and T7): a seat's record that cuts (paying stamina), feints with
    /// the right button in the wind-up, holds the right button's guard, and
    /// kicks with F, walking in so the kick can land.
    fn chivalry_script(seat: usize, t: u32) -> Input {
        if t < DELAY {
            return Input::default();
        }
        let k = (t + seat as u32 * 17) % 60;
        let (buttons, dx, dy) = match k {
            0 => (SWING, 0, 6_000),
            1..=4 => (SWING, 0, 0),
            5 => (SWING | HOLD_LOOK, 0, 0),
            6..=8 => (0, 0, 0),
            9..=24 => (HOLD_LOOK, if k.is_multiple_of(3) { 400 } else { -150 }, 100),
            40 => (SWING, -6_000, 0),
            41..=55 => (SWING, 0, 0),
            _ => (0, 0, 0),
        };
        Input {
            keys: if k == 30 {
                crate::input::KICK
            } else if k < 30 {
                FORWARD
            } else {
                0
            },
            buttons,
            dx,
            dy,
            wheel: 0,
        }
    }

    /// CA-5 with M4: a duel of stamina, feints, guards and kicks through
    /// lockstep and through rollback over a delaying, reordering wire
    /// reaches the world a browser fed every record on time reaches.
    #[test]
    fn rollback_reaches_lockstep_s_world_through_stamina_feints_guards_and_kicks() {
        let ticks = 300;
        let mut want = crate::seat::duel();
        let (mut feints, mut guards, mut kicks, mut spent) = (0, 0, 0, false);
        for t in 0..ticks {
            want.tick([chivalry_script(0, t), chivalry_script(1, t)]);
            for id in 0..2 {
                let c = crate::player::fighter(&want, id).chiv;
                feints += c.event.is_some_and(|e| e == (crate::chivalry::Event::Feint, 1)) as u32;
                guards += c.guard.is_some() as u32;
                kicks += (c.kick == 1) as u32;
                spent |= c.stamina < 10_000;
            }
        }
        assert!(want.settings.chivalry, "the online duel plays with the layer on");
        assert!(
            feints > 0 && guards > 0 && kicks > 0 && spent,
            "{feints} {guards} {kicks} {spent}"
        );

        // the kit's lockstep, over a 3-frame lag
        let mut ls = [Session::new(0, "b"), Session::new(1, "b")];
        ls[0].connected(JOINER);
        ls[1].connected(HOST);
        for (to, from) in [(1, 0), (0, 1)] {
            for (_, text) in ls[from].take_outbox() {
                ls[to].message([HOST, JOINER][from], &text);
            }
        }
        let mut lw = [crate::seat::duel(), crate::seat::duel()];
        let mut wire: VecDeque<(usize, usize, String)> = VecDeque::new();
        for frame in 0..2_000usize {
            for seat in 0..SEATS {
                let s = &mut ls[seat];
                let k = chivalry_script(seat, s.tick + DELAY);
                s.local(k);
                for (_, text) in s.take_outbox() {
                    wire.push_back((frame + 3, 1 - seat, text));
                }
            }
            while wire.front().is_some_and(|m| m.0 <= frame) {
                let (_, to, text) = wire.pop_front().unwrap();
                ls[to].message([HOST, JOINER][1 - to], &text);
            }
            for seat in 0..SEATS {
                if ls[seat].tick < ticks {
                    ls[seat].step(&mut lw[seat]);
                }
            }
        }
        assert!(lw[0] == want && lw[1] == want, "lockstep left the true world");

        // rollback, messages held up to 6 frames and reordered
        let (host, join) = pair(crate::seat::duel());
        let mut rb = [host, join];
        let mut wire = Wire {
            held: Vec::new(),
            sent: 0,
            most: 6,
        };
        let mut frame = 0usize;
        while (rb.iter().any(|s| s.tick < ticks || s.guessed() > 0) || !wire.held.is_empty()) && frame < 4_000
        {
            for seat in 0..SEATS {
                let s = &mut rb[seat];
                let k = chivalry_script(seat, s.tick + DELAY);
                s.local(k);
                for (_, text) in s.take_outbox() {
                    wire.put(frame, 1 - seat, text);
                }
            }
            for (to, text) in wire.due(frame) {
                rb[to].message([HOST, JOINER][1 - to], &text);
            }
            for s in rb.iter_mut() {
                if s.tick < ticks {
                    s.step();
                } else {
                    s.roll_back();
                    s.sum_ready();
                }
            }
            frame += 1;
        }
        for (seat, s) in rb.iter().enumerate() {
            assert_eq!(s.tick, ticks, "seat {seat} stalled");
            assert_eq!(s.link.status, PLAYING, "seat {seat}");
            assert!(s.world == want, "seat {seat} left the true world");
            assert!(s.world == lw[seat], "seat {seat} and lockstep differ");
            assert!(s.stats.rollbacks > 0, "no guess went wrong");
        }
    }
}
