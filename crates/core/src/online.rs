//! Online play, core side: what crosses the wire, and the bookkeeping each
//! browser keeps. Part of the online kit (`kits/online/README.md`).
//!
//! The transport in `web/net/rtc.js` connects browsers and carries text. It
//! hands this module three things: a peer arrived, a peer left, and a message
//! from a peer. This module answers by leaving text in an outbox, which the
//! transport sends. It never sees a connection, so `cargo test` can play a
//! whole duel between two simulated browsers, with lag, in a few
//! milliseconds.
//!
//! This project's changes to the kit (M5, SECOND-ORDER.md R8.x):
//!
//! - an input on the wire is the game's own record (`input::Input`), with
//!   its JSON derived from the type, so a field added to the record rides
//!   along with no change here;
//! - the world is anything that steps on both seats' records and sums
//!   itself (`Sim`): the kit's toy world below, for the kit's tests, and the
//!   duel (`seat.rs`);
//! - the connection, hello, bye and garbage handling the kit's session kept
//!   inline sit in `Link`, which the kit's lockstep `Session` here and the
//!   rollback session (`rollback.rs`, CA-5) share;
//! - a checksum that arrives before this browser has made its own is kept
//!   until it has (the kit's dropped it).
//!
//! The kit's lockstep session stays, with its required tests: it is the
//! reference the rollback session must agree with (`rollback.rs`).
//!
//! Every status is a key for a string in `data/` (`online.reload`,
//! `online.garbled`, ...), so the wording sits with the other copy and the
//! rule that chose it sits here.
//!
//! Required tests, each implemented below:
//!
//! - every message survives encoding and decoding;
//! - a browser on another protocol or another build is turned away, and both
//!   ends say why;
//! - garbage on the wire changes the status and nothing else, and a message
//!   from a peer that never said hello is ignored;
//! - a browser waits for the other seat's input; it does not guess;
//! - two copies joined by a lagged wire end on the same world as a copy fed
//!   every input on time;
//! - a peer that leaves is reported, and a third browser is told the room is
//!   full;
//! - two worlds that disagree are reported as a desync.

use crate::input::Input;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

/// Bumped whenever a message changes shape or a rule changes how a tick
/// runs. The transport puts it in the room name (`room_name`), so a stale
/// tab and a fresh one still meet, and the hello turns the stale one away
/// with a reason. A room name holding the build hash would keep them apart,
/// and both would wait for nobody.
pub const PROTOCOL: u32 = 1;

/// The name that keeps this game's rooms apart from every other app on the
/// same public relays: the first part of each room's name.
pub const APP_ID: &str = "chumworld-arena";

/// Ticks between a local input and the tick that uses it (M5: two ticks,
/// 33 ms, which a link of under 33 ms one way covers with no rollback).
pub const DELAY: u32 = 2;

/// Ticks between checksums (half a second).
pub const SUM_EVERY: u32 = 30;

/// The two seats: 0 hosts, 1 joins.
pub const SEATS: usize = 2;

/// Status keys. Each has a string in `data/` under the same key.
pub const WAITING: &str = "online.waiting";
pub const PLAYING: &str = "online.playing";
pub const RELOAD: &str = "online.reload";
pub const GARBLED: &str = "online.garbled";
pub const LEFT_ROOM: &str = "online.left";
pub const FULL: &str = "online.full";
pub const DESYNC: &str = "online.desync";

/// The reasons a `Bye` may carry. Anything else on the wire reads as `LEFT_ROOM`.
const REASONS: &[&str] = &[RELOAD, FULL];

/// The transport's id for another browser. It stays the same while
/// connections come and go under it.
pub type Peer = u32;

/// Everything that crosses the wire. JSON text, tagged by `t`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "lowercase")]
pub enum Msg {
    Hello {
        protocol: u32,
        build: String,
    },
    /// one seat's record for one tick, as the record's own type writes it
    Input {
        tick: u32,
        input: Input,
    },
    Sum {
        tick: u32,
        sum: u64,
    },
    Bye {
        why: String,
    },
}

pub fn encode(m: &Msg) -> String {
    serde_json::to_string(m).expect("a Msg always serialises")
}

pub fn decode(text: &str) -> Option<Msg> {
    serde_json::from_str(text).ok()
}

/// The room name the transport joins: the app id, the protocol and the code
/// the players typed. No build hash.
pub fn room_name(app_id: &str, code: &str) -> String {
    format!("{app_id}-p{PROTOCOL}-{code}")
}

/// A world both browsers run alike: it steps on both seats' records and
/// sums itself in one number. Cloned whole for rollback's snapshots, so a
/// field added to the world is saved and restored with no change here.
pub trait Sim: Clone {
    fn tick(&mut self, inputs: [Input; SEATS]);
    fn sum(&self) -> u64;
}

/// Input bits for the kit's example world.
pub const LEFT: u32 = 1;
pub const RIGHT: u32 = 2;

/// The kit's example world: two markers on a line. Its tests run on it;
/// the duel is `seat.rs`'s.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct World {
    pub pos: [i32; SEATS],
    pub met: u32,
}

impl Sim for World {
    fn tick(&mut self, inputs: [Input; SEATS]) {
        for (seat, i) in inputs.iter().enumerate() {
            if i.keys & LEFT != 0 {
                self.pos[seat] -= 1;
            }
            if i.keys & RIGHT != 0 {
                self.pos[seat] += 1;
            }
        }
        if self.pos[0] == self.pos[1] {
            self.met += 1;
        }
    }

    /// FNV-1a over the state, in a fixed order.
    fn sum(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut mix = |v: u64| {
            h ^= v;
            h = h.wrapping_mul(0x0100_0000_01b3);
        };
        for p in self.pos {
            mix(p as u32 as u64);
        }
        mix(self.met as u64);
        h
    }
}

/// One browser's link to the other: who it is, whether it said hello, the
/// status, the outbox, and the checksums waiting for their match.
#[derive(Debug, Clone)]
pub struct Link {
    pub status: &'static str,
    build: String,
    /// The other browser, once its hello has been accepted.
    pub peer: Option<Peer>,
    /// A browser that has connected and not yet said hello.
    greeting: Option<Peer>,
    /// Checksums of ticks this browser ran, and the other's, by tick, kept
    /// until both are here.
    mine: BTreeMap<u32, u64>,
    theirs: BTreeMap<u32, u64>,
    /// The tick of the last checksum both browsers agreed on.
    pub agreed: Option<(u32, u64)>,
    outbox: VecDeque<(Peer, String)>,
}

impl Link {
    pub fn new(build: &str) -> Link {
        Link {
            status: WAITING,
            build: build.to_string(),
            peer: None,
            greeting: None,
            mine: BTreeMap::new(),
            theirs: BTreeMap::new(),
            agreed: None,
            outbox: VecDeque::new(),
        }
    }

    pub fn send(&mut self, to: Peer, m: &Msg) {
        self.outbox.push_back((to, encode(m)));
    }

    /// Sends to the other browser, if there is one.
    pub fn send_peer(&mut self, m: &Msg) {
        if let Some(p) = self.peer {
            self.send(p, m);
        }
    }

    /// Everything to send, in order.
    pub fn take_outbox(&mut self) -> Vec<(Peer, String)> {
        self.outbox.drain(..).collect()
    }

    /// The transport says a browser arrived.
    pub fn connected(&mut self, peer: Peer) {
        if self.peer.is_some() || self.greeting.is_some() {
            self.send(
                peer,
                &Msg::Bye {
                    why: FULL.to_string(),
                },
            );
            return;
        }
        self.greeting = Some(peer);
        let hello = Msg::Hello {
            protocol: PROTOCOL,
            build: self.build.clone(),
        };
        self.send(peer, &hello);
    }

    /// The transport says a browser left.
    pub fn left(&mut self, peer: Peer) {
        if self.peer == Some(peer) || self.greeting == Some(peer) {
            self.peer = None;
            self.greeting = None;
            self.status = LEFT_ROOM;
        }
    }

    /// This browser leaves: the other is told, and the link is closed.
    pub fn leave(&mut self) {
        if let Some(p) = self.peer.or(self.greeting) {
            self.send(
                p,
                &Msg::Bye {
                    why: LEFT_ROOM.to_string(),
                },
            );
        }
        self.peer = None;
        self.greeting = None;
    }

    /// Text from a browser. Anything that cannot be a message, or that no
    /// honest copy would send, changes the status and nothing else. Hello
    /// and bye are handled here; an input or a checksum from the accepted
    /// peer is handed back for the session.
    pub fn message(&mut self, from: Peer, text: &str) -> Option<Msg> {
        let known = self.peer == Some(from) || self.greeting == Some(from);
        if !known {
            return None;
        }
        let Some(msg) = decode(text) else {
            self.status = GARBLED;
            return None;
        };
        match msg {
            Msg::Hello { protocol, build } => {
                if protocol != PROTOCOL || build != self.build {
                    self.send(
                        from,
                        &Msg::Bye {
                            why: RELOAD.to_string(),
                        },
                    );
                    self.greeting = None;
                    self.peer = None;
                    self.status = RELOAD;
                } else {
                    self.greeting = None;
                    self.peer = Some(from);
                    self.status = PLAYING;
                }
                None
            }
            // a bye may come before a hello: the room is full, or this
            // browser's build was turned away
            Msg::Bye { why } => {
                self.status = REASONS.iter().copied().find(|r| *r == why).unwrap_or(LEFT_ROOM);
                self.peer = None;
                self.greeting = None;
                None
            }
            _ if self.peer != Some(from) => {
                // data before a hello: the transport holds it back, so a
                // copy that sends it anyway is not one of ours
                self.status = GARBLED;
                None
            }
            Msg::Sum { tick, sum } => {
                self.theirs.insert(tick, sum);
                self.compare();
                None
            }
            m => Some(m),
        }
    }

    /// This browser's checksum of tick `tick`, sent and kept for comparing.
    pub fn summed(&mut self, tick: u32, sum: u64) {
        if self.peer.is_none() {
            return;
        }
        self.mine.insert(tick, sum);
        self.send_peer(&Msg::Sum { tick, sum });
        self.compare();
    }

    /// Each tick both browsers have summed: a mismatch is a desync.
    fn compare(&mut self) {
        let both: Vec<u32> = self
            .mine
            .keys()
            .filter(|t| self.theirs.contains_key(t))
            .copied()
            .collect();
        for t in both {
            let (a, b) = (self.mine.remove(&t), self.theirs.remove(&t));
            if a != b {
                self.status = DESYNC;
            } else if let Some(a) = a {
                self.agreed = Some((t, a));
            }
        }
    }
}

/// One browser's side of a duel played in lockstep: the kit's session, and
/// the reference the rollback session agrees with.
#[derive(Debug)]
pub struct Session {
    pub seat: usize,
    pub link: Link,
    /// Each seat's inputs by tick.
    pub inputs: [BTreeMap<u32, Input>; SEATS],
    /// The next tick this browser will run.
    pub tick: u32,
}

impl Session {
    pub fn new(seat: usize, build: &str) -> Session {
        let mut inputs: [BTreeMap<u32, Input>; SEATS] = Default::default();
        for seat_inputs in inputs.iter_mut() {
            for t in 0..DELAY {
                seat_inputs.insert(t, Input::default());
            }
        }
        Session {
            seat,
            link: Link::new(build),
            inputs,
            tick: 0,
        }
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

    /// Text from a browser.
    pub fn message(&mut self, from: Peer, text: &str) {
        let Some(Msg::Input { tick, input }) = self.link.message(from, text) else {
            return;
        };
        let other = 1 - self.seat;
        let window = self.tick..=self.tick + DELAY + DELAY;
        if !window.contains(&tick) || self.inputs[other].contains_key(&tick) {
            self.link.status = GARBLED;
            return;
        }
        self.inputs[other].insert(tick, input);
    }

    /// This browser's input, made now. It is used `DELAY` ticks ahead and
    /// sent once. Called every display frame; a frame that runs no tick
    /// adds nothing.
    pub fn local(&mut self, input: Input) {
        let Some(peer) = self.link.peer else { return };
        let tick = self.tick + DELAY;
        if self.inputs[self.seat].contains_key(&tick) {
            return;
        }
        self.inputs[self.seat].insert(tick, input);
        self.link.send(peer, &Msg::Input { tick, input });
    }

    /// Runs the next tick if both seats' inputs for it are here. Returns
    /// whether it ran. A browser without the other's input waits.
    pub fn step<W: Sim>(&mut self, world: &mut W) -> bool {
        let t = self.tick;
        let (Some(&a), Some(&b)) = (self.inputs[0].get(&t), self.inputs[1].get(&t)) else {
            return false;
        };
        world.tick([a, b]);
        for seat_inputs in self.inputs.iter_mut() {
            seat_inputs.remove(&t);
        }
        self.tick += 1;
        if self.tick.is_multiple_of(SUM_EVERY) {
            self.link.summed(self.tick, world.sum());
        }
        true
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub const HOST: Peer = 1;
    pub const JOINER: Peer = 2;

    pub fn keys(k: u32) -> Input {
        Input {
            keys: k,
            ..Input::default()
        }
    }

    /// The keys each seat presses for a tick: a fixed script, so a run can
    /// be compared with a copy fed every input on time.
    fn script(seat: usize, tick: u32) -> Input {
        if tick < DELAY {
            return Input::default();
        }
        keys(match (tick * (seat as u32 + 3)) % 5 {
            0 | 1 => LEFT,
            2 => RIGHT,
            3 => LEFT | RIGHT,
            _ => 0,
        })
    }

    fn truth(ticks: u32) -> World {
        let mut w = World::default();
        for t in 0..ticks {
            w.tick([script(0, t), script(1, t)]);
        }
        w
    }

    /// A host and a joiner that have exchanged hellos.
    fn pair(build_host: &str, build_join: &str) -> (Session, Session) {
        let mut host = Session::new(0, build_host);
        let mut join = Session::new(1, build_join);
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

    #[test]
    fn every_message_round_trips() {
        let all = [
            Msg::Hello {
                protocol: PROTOCOL,
                build: "abc123".into(),
            },
            Msg::Input {
                tick: 7,
                input: Input {
                    keys: 3,
                    buttons: 2,
                    dx: -12_345,
                    dy: 678,
                    wheel: -100,
                },
            },
            Msg::Sum {
                tick: 30,
                sum: u64::MAX,
            },
            Msg::Bye { why: FULL.into() },
        ];
        for m in all {
            assert_eq!(decode(&encode(&m)), Some(m.clone()), "{m:?}");
        }
    }

    /// M5: an input's JSON is the record's own, field by field, so a field
    /// added to the record crosses the wire with no change here.
    #[test]
    fn an_input_on_the_wire_is_the_record_s_own_json() {
        let input = Input {
            keys: 5,
            buttons: 1,
            dx: -40,
            dy: 3,
            wheel: 0,
        };
        let text = encode(&Msg::Input { tick: 9, input });
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["t"], "input");
        assert_eq!(v["tick"], 9);
        assert_eq!(v["input"], serde_json::to_value(input).unwrap());
    }

    #[test]
    fn the_room_name_carries_the_protocol_and_not_the_build() {
        let name = room_name("my-game", "ABCDE");
        assert_eq!(name, format!("my-game-p{PROTOCOL}-ABCDE"));
    }

    #[test]
    fn another_build_or_protocol_is_turned_away_with_a_reason() {
        let (host, join) = pair("b1", "b2");
        assert_eq!(host.link.status, RELOAD);
        assert_eq!(join.link.status, RELOAD);
        assert!(host.link.peer.is_none() && join.link.peer.is_none());

        let mut host = Session::new(0, "b1");
        host.connected(JOINER);
        host.take_outbox();
        let old = encode(&Msg::Hello {
            protocol: PROTOCOL + 1,
            build: "b1".into(),
        });
        host.message(JOINER, &old);
        assert_eq!(host.link.status, RELOAD);
        let out = host.take_outbox();
        assert_eq!(out.len(), 1);
        assert_eq!(decode(&out[0].1), Some(Msg::Bye { why: RELOAD.into() }));

        // the far end reads the reason as well as the departure
        let mut other = Session::new(1, "b9");
        other.connected(HOST);
        other.message(HOST, &out[0].1);
        assert_eq!(other.link.status, RELOAD);
    }

    #[test]
    fn garbage_on_the_wire_changes_only_the_status() {
        let (mut host, _join) = pair("b", "b");
        let mut world = World::default();
        host.local(keys(LEFT));
        host.take_outbox();
        let before = (host.tick, host.inputs.clone(), host.link.peer);
        for junk in [
            "{nope",
            "",
            "{}",
            "[1,2,3]",
            r#"{"t":"input","tick":"x","input":{"keys":1,"buttons":0,"dx":0,"dy":0,"wheel":0}}"#,
            r#"{"t":"input","tick":4000000000,"input":{"keys":1,"buttons":0,"dx":0,"dy":0,"wheel":0}}"#,
            r#"{"t":"teleport","to":9}"#,
        ] {
            host.link.status = PLAYING;
            host.message(JOINER, junk);
            assert_eq!(host.link.status, GARBLED, "{junk:?}");
            assert_eq!((host.tick, host.inputs.clone(), host.link.peer), before);
            assert!(host.take_outbox().is_empty(), "{junk:?} drew a reply");
        }
        assert!(host.step(&mut world), "the prefilled first tick still runs");

        // a browser that never said hello is not heard at all
        host.link.status = PLAYING;
        host.message(9, "{nope");
        host.message(
            9,
            &encode(&Msg::Input {
                tick: 3,
                input: keys(1),
            }),
        );
        assert_eq!(host.link.status, PLAYING);

        // an unknown reason on a bye reads as a plain departure
        host.message(
            JOINER,
            &encode(&Msg::Bye {
                why: "<b>x</b>".into(),
            }),
        );
        assert_eq!(host.link.status, LEFT_ROOM);
    }

    #[test]
    fn a_copy_waits_for_the_other_seat_and_does_not_guess() {
        let (mut host, _join) = pair("b", "b");
        let mut world = World::default();
        for _ in 0..DELAY {
            host.local(Input::default());
            assert!(host.step(&mut world));
        }
        host.local(keys(RIGHT));
        assert!(!host.step(&mut world), "ran a tick without the joiner's input");
        assert_eq!(host.tick, DELAY);
        host.message(
            JOINER,
            &encode(&Msg::Input {
                tick: DELAY,
                input: Input::default(),
            }),
        );
        assert!(host.step(&mut world));
    }

    #[test]
    fn two_lagged_copies_reach_the_world_of_a_copy_fed_on_time() {
        let ticks = 300;
        for lag in [0usize, 1, 3, 7] {
            let (mut host, mut join) = pair("b", "b");
            let mut worlds = [World::default(), World::default()];
            // (arrives at frame, to seat, text)
            let mut wire: VecDeque<(usize, usize, String)> = VecDeque::new();
            let mut frame = 0usize;
            while (host.tick < ticks || join.tick < ticks) && frame < 10_000 {
                for seat in 0..SEATS {
                    let s = if seat == 0 { &mut host } else { &mut join };
                    let k = script(seat, s.tick + DELAY);
                    s.local(k);
                    for (_, text) in s.take_outbox() {
                        wire.push_back((frame + lag, 1 - seat, text));
                    }
                }
                let mut later = VecDeque::new();
                while let Some((at, to, text)) = wire.pop_front() {
                    if at > frame {
                        later.push_back((at, to, text));
                    } else if to == 0 {
                        host.message(JOINER, &text);
                    } else {
                        join.message(HOST, &text);
                    }
                }
                wire = later;
                if host.tick < ticks {
                    host.step(&mut worlds[0]);
                }
                if join.tick < ticks {
                    join.step(&mut worlds[1]);
                }
                frame += 1;
            }
            assert_eq!(host.tick, ticks, "lag {lag}: the host stalled");
            assert_eq!(join.tick, ticks, "lag {lag}: the joiner stalled");
            assert_eq!(worlds[0], truth(ticks), "lag {lag}: host");
            assert_eq!(worlds[1], truth(ticks), "lag {lag}: joiner");
            assert_eq!(host.link.status, PLAYING, "lag {lag}");
            assert_eq!(join.link.status, PLAYING, "lag {lag}");
        }
    }

    #[test]
    fn a_peer_that_leaves_is_reported_and_a_third_is_told_the_room_is_full() {
        let (mut host, _join) = pair("b", "b");
        host.connected(3);
        let out = host.take_outbox();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].0, 3);
        assert_eq!(decode(&out[0].1), Some(Msg::Bye { why: FULL.into() }));
        assert_eq!(host.link.peer, Some(JOINER), "the third browser took no seat");

        host.left(3);
        assert_eq!(host.link.status, PLAYING, "a stranger leaving changes nothing");
        host.left(JOINER);
        assert_eq!(host.link.status, LEFT_ROOM);
        assert!(host.link.peer.is_none());
    }

    #[test]
    fn worlds_that_disagree_are_reported_as_a_desync() {
        let (mut host, _join) = pair("b", "b");
        let mut world = World::default();
        while host.tick < SUM_EVERY {
            host.local(Input::default());
            let t = host.tick;
            if t >= DELAY {
                host.message(
                    JOINER,
                    &encode(&Msg::Input {
                        tick: t,
                        input: Input::default(),
                    }),
                );
            }
            assert!(host.step(&mut world));
        }
        let mine = world.sum();
        host.message(
            JOINER,
            &encode(&Msg::Sum {
                tick: SUM_EVERY,
                sum: mine ^ 1,
            }),
        );
        assert_eq!(host.link.status, DESYNC);

        // M5: a checksum that comes before this browser's own is kept for it
        let (mut host, _join) = pair("b", "b");
        let mut world = World::default();
        host.message(
            JOINER,
            &encode(&Msg::Sum {
                tick: SUM_EVERY,
                sum: 7,
            }),
        );
        assert_eq!(host.link.status, PLAYING);
        while host.tick < SUM_EVERY {
            host.local(Input::default());
            let t = host.tick;
            if t >= DELAY {
                host.message(
                    JOINER,
                    &encode(&Msg::Input {
                        tick: t,
                        input: Input::default(),
                    }),
                );
            }
            host.step(&mut world);
        }
        assert_eq!(host.link.status, DESYNC, "the early, wrong checksum was compared");
    }
}
