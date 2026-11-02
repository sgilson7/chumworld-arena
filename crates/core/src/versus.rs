//! An online duel as the game runs it (M5): hosting and joining, by room
//! code or by pasted codes, the rollback session (`rollback.rs`) on the
//! duel's world (`seat.rs`), and what the page's transport is told to do.
//!
//! The page's transport (`web/net/rtc.js`, through `web/online.js`) decides
//! nothing. The core tells it what to do as a list of numbered commands in
//! the view model (`Cmd`): open a link (by room code or for pasted codes),
//! make an invitation, make a reply to one, accept a reply, close. The page
//! runs each command once, in order, and reports back through `act`
//! (`online.offer`, `online.answer`, `online.error`) and the shim's
//! `online_connected`, `online_left` and `online_message`.
//!
//! Nothing here runs until the reader chooses Host, Join or Pasted codes
//! on the menu; until then the view model asks for no link, and the page
//! loads nothing for online play (CA-4).

use serde::Serialize;

use crate::input::Input;
use crate::online;
use crate::rollback::Rollback;
use crate::world::World;

/// Which side of the duel this browser is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// opened the duel: fighter 0, Knight A
    Host,
    /// joined it: the second seat, fighter 1, Knight B
    Guest,
}

/// How the two browsers are introduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Way {
    /// a room code over the relays in `web/net/config.js`
    Room,
    /// pasted codes, with no relay
    Paste,
}

/// The letters a room code is made of: no I or O, which read as 1 and 0.
pub const CODE_LETTERS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";
/// How many letters a room code has.
pub const CODE_LENGTH: usize = 5;
/// No word from the other browser for this long ends the duel: 10 s.
pub const TIMEOUT_US: i64 = 10_000_000;
/// The status key for a timeout (the others are `online.rs`'s and the
/// transport's).
pub const TIMEOUT: &str = "online.timeout";
/// The transport's error keys: each has a string in `data/`.
pub const TRANSPORT_ERRORS: &[&str] = &[
    "online.no_bundle",
    "online.no_relay",
    "online.no_path",
    "online.signal_failed",
    "online.bad_code",
];
/// The statuses that end an online duel.
pub const ENDED: &[&str] = &[
    online::LEFT_ROOM,
    online::RELOAD,
    online::FULL,
    online::DESYNC,
    TIMEOUT,
];
/// How many of this browser's own checksums the view model shows.
pub const SUMS_SHOWN: usize = 8;

/// One thing for the page's transport to do. `act` is `open`, `invite`,
/// `reply`, `accept` or `close`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Cmd {
    pub id: u32,
    pub act: String,
    pub host: bool,
    /// `room` or `code` (an open)
    pub mode: String,
    /// the room's name (an open by room code)
    pub room: String,
    /// the pasted code to reply to or to accept
    pub code: String,
}

#[derive(Debug, Clone)]
pub struct Versus {
    pub role: Role,
    pub way: Way,
    /// the room code, made by the host or typed by the guest
    pub code: String,
    /// the host's invitation, as made or as pasted
    pub offer: String,
    /// the guest's reply, as made or as pasted
    pub answer: String,
    pub session: Rollback<World>,
    /// the transport's error, or a timeout
    pub error: Option<&'static str>,
    /// the page's clock when the other browser was last heard
    pub heard_us: Option<i64>,
    /// this player has chosen Rematch and waits for the other
    pub rematch: bool,
    /// a duel has begun (both hellos accepted)
    pub started: bool,
}

impl Versus {
    pub fn new(role: Role, way: Way, build: &str) -> Versus {
        let seat = match role {
            Role::Host => 0,
            Role::Guest => 1,
        };
        Versus {
            role,
            way,
            code: String::new(),
            offer: String::new(),
            answer: String::new(),
            session: Rollback::new(seat, build, crate::seat::duel()),
            error: None,
            heard_us: None,
            rematch: false,
            started: false,
        }
    }

    /// The status key the screen shows.
    pub fn status(&self) -> &'static str {
        self.error.unwrap_or(self.session.link.status)
    }

    /// The duel has ended: the other left, was turned away, went quiet,
    /// disagreed, or the transport failed.
    pub fn ended(&self) -> bool {
        self.error.is_some() || ENDED.contains(&self.session.link.status)
    }

    /// Both browsers are linked and the duel runs.
    pub fn live(&self) -> bool {
        self.session.link.peer.is_some() && !self.ended()
    }

    /// The world as this browser's page draws it: the guest's turned round
    /// (`seat::face`), so its own knight is the player.
    pub fn view(&self) -> World {
        let mut w = self.session.world.clone();
        if self.role == Role::Guest {
            crate::seat::face(&mut w);
        }
        w
    }

    /// This browser's checksums, newest last, as `tick:sum` in hex.
    pub fn sums(&self) -> String {
        let made = &self.session.made;
        let from = made.len().saturating_sub(SUMS_SHOWN);
        made.iter()
            .skip(from)
            .map(|(t, s)| format!("{t}:{s:016x}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// This tick's record as it goes out: Rematch, once chosen, rides on it
    /// (`seat::REMATCH`).
    pub fn record(&self, mut i: Input) -> Input {
        if self.rematch {
            i.keys |= crate::seat::REMATCH;
        }
        i
    }
}

/// The reader's movement, as an online record carries it: scaled by the
/// look speed, its up and down turned over if the reader inverts the pitch.
pub fn scaled(k: &crate::save::Kept, dx: i32, dy: i32) -> (i32, i32) {
    let s = |v: i32| (v as i64 * k.look_pct as i64 / 100) as i32;
    (s(dx), if k.invert_pitch { -s(dy) } else { s(dy) })
}

/// A room code from a draw of chance: `CODE_LENGTH` letters.
pub fn room_code(rng: &mut crate::rng::Rng) -> String {
    (0..CODE_LENGTH)
        .map(|_| CODE_LETTERS[rng.below(CODE_LETTERS.len() as u32) as usize] as char)
        .collect()
}

/// A typed room code, read loosely (spaces dropped, any case), or none if
/// it cannot be one.
pub fn read_code(typed: &str) -> Option<String> {
    let code: String = typed
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    let ok = code.len() == CODE_LENGTH && code.bytes().all(|b| CODE_LETTERS.contains(&b));
    ok.then_some(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_room_code_is_five_letters_and_is_read_back_loosely() {
        let mut rng = crate::rng::Rng::new(3);
        let code = room_code(&mut rng);
        assert_eq!(code.len(), 5);
        assert!(code
            .bytes()
            .all(|b| b.is_ascii_uppercase() && b != b'I' && b != b'O'));
        assert_eq!(read_code(&format!(" {} ", code.to_lowercase())), Some(code));
        assert_eq!(read_code("AB CDE"), Some("ABCDE".into()));
        for bad in ["", "ABCD", "ABCDEF", "ABCD1", "ABCDO", "ABÇDE"] {
            assert_eq!(read_code(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn the_reader_s_look_speed_and_pitch_go_into_the_record() {
        let mut k = crate::save::Kept::default();
        assert_eq!(scaled(&k, 1_000, -300), (1_000, -300));
        k.look_pct = 150;
        k.invert_pitch = true;
        // 1 000 × 150% = 1 500; −300 × 150% = −450, turned over to 450
        assert_eq!(scaled(&k, 1_000, -300), (1_500, 450));
    }
}
