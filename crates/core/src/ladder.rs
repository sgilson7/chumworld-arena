//! `ladder` (M6, T6): the single-player run of duels against harder
//! opponents in turn, and what winning them gives.
//!
//! Seven rungs, ranked F at the foot to S at the top (`RUNGS`). Each rung is
//! one opponent in its own costume (`models/knight/`, `champion.rs`,
//! `mirror.rs`) driven by its own pilot (`data/pilots.json`, the entry and
//! the tree under the rung's key), fought as Practice is fought: best of
//! three rounds (`rounds.rs`).
//!
//! The rules, each tested below:
//!
//! - The first rung is open; a rung opens once the rung below it is won
//!   (`Progress::open`).
//! - Winning a rung marks it: one mark for a win, a second for a win without
//!   losing a round (`marks_for`). A rung keeps its best marks; losing takes
//!   none away (`Progress::record`).
//! - Winning a rung gives its opponent's look to wear in Practice and on the
//!   ladder, and nothing else (CA-Q9: A): the player's body stays a knight's,
//!   with a knight's plate, reach and strength (`save::Knight::body`), and
//!   only the drawing wears the look (`Fighter::dress`).
//! - Progress lives in the save (`save.rs`); a save whose marks could not
//!   have been won in order, or whose look is not yet won, is refused
//!   (`Progress::valid`, `save::read`).

use crate::pose::Kind;

/// One rung: its rank's letter, the key its pilot, its strings and the save
/// use (`data/pilots.json`, `who_<key>`, `dossier_<key>`), and the body its
/// opponent fights in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rung {
    pub rank: &'static str,
    pub key: &'static str,
    pub kind: Kind,
}

/// The rungs from the foot of the ladder to the top.
pub const RUNGS: [Rung; 7] = [
    Rung {
        rank: "F",
        key: "squire",
        kind: Kind::Squire,
    },
    Rung {
        rank: "E",
        key: "sellsword",
        kind: Kind::Sellsword,
    },
    Rung {
        rank: "D",
        key: "hedge",
        kind: Kind::Hedge,
    },
    Rung {
        rank: "C",
        key: "veteran",
        kind: Kind::Veteran,
    },
    Rung {
        rank: "B",
        key: "banneret",
        kind: Kind::Banneret,
    },
    Rung {
        rank: "A",
        key: "champion",
        kind: Kind::Champion,
    },
    Rung {
        rank: "S",
        key: "mirror",
        kind: Kind::Mirror,
    },
];

/// The most marks a rung can hold: a win, and a win without losing a round.
pub const MARKS_MOST: u8 = 2;

/// The rung whose key is `key`.
pub fn rung_keyed(key: &str) -> Option<usize> {
    RUNGS.iter().position(|r| r.key == key)
}

/// The marks a duel's result earns: none for a loss, one for a win, two for
/// a win without losing a round. `wins` is the rounds won, the player's then
/// the foe's, once the duel is over.
pub fn marks_for(wins: [u8; 2]) -> u8 {
    if wins[0] < crate::rounds::TO_WIN {
        0
    } else if wins[1] == 0 {
        MARKS_MOST
    } else {
        1
    }
}

/// How far up the ladder the player is: each rung's marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Progress {
    pub marks: [u8; RUNGS.len()],
}

/// How a rank card shows its rung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Locked,
    Open,
    Won(u8),
}

impl Progress {
    /// The rung may be fought: the first always, any other once the rung
    /// below it is won.
    pub fn open(&self, i: usize) -> bool {
        i < RUNGS.len() && (i == 0 || self.marks[i - 1] > 0)
    }

    pub fn won(&self, i: usize) -> bool {
        i < RUNGS.len() && self.marks[i] > 0
    }

    pub fn state(&self, i: usize) -> State {
        if self.won(i) {
            State::Won(self.marks[i])
        } else if self.open(i) {
            State::Open
        } else {
            State::Locked
        }
    }

    /// A duel on rung `i` is over with these rounds won: the rung keeps the
    /// better of its marks and the duel's. The marks the duel earned, or
    /// none when the rung was not open (nothing changes).
    pub fn record(&mut self, i: usize, wins: [u8; 2]) -> u8 {
        if !self.open(i) {
            return 0;
        }
        let got = marks_for(wins);
        self.marks[i] = self.marks[i].max(got);
        got
    }

    /// Marks a player could have won: none above `MARKS_MOST`, and no rung
    /// marked above one that is not.
    pub fn valid(&self) -> bool {
        (0..RUNGS.len()).all(|i| self.marks[i] <= MARKS_MOST && (self.marks[i] == 0 || self.open(i)))
    }

    /// The rungs won, by number from the foot.
    pub fn won_rungs(&self) -> impl Iterator<Item = usize> + '_ {
        (0..RUNGS.len()).filter(|&i| self.won(i))
    }
}

/// The numbers the ladder's strings and glossary entries name.
pub fn values() -> Vec<(&'static str, String)> {
    vec![
        ("ladder.rungs", RUNGS.len().to_string()),
        ("ladder.marks", MARKS_MOST.to_string()),
        ("ladder.foot", RUNGS[0].rank.to_string()),
        ("ladder.top", RUNGS[RUNGS.len() - 1].rank.to_string()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marked(m: [u8; 7]) -> Progress {
        Progress { marks: m }
    }

    #[test]
    fn the_rungs_run_f_to_s_each_its_own_opponent() {
        let ranks: Vec<&str> = RUNGS.iter().map(|r| r.rank).collect();
        assert_eq!(ranks, ["F", "E", "D", "C", "B", "A", "S"]);
        let keys: Vec<&str> = RUNGS.iter().map(|r| r.key).collect();
        assert_eq!(
            keys,
            [
                "squire",
                "sellsword",
                "hedge",
                "veteran",
                "banneret",
                "champion",
                "mirror"
            ]
        );
        // each rung's key names its body for the pilot book (the act's name)
        for r in RUNGS {
            assert_eq!(crate::pilot::kind_named(r.key), Some(r.kind), "{}", r.key);
        }
        assert_eq!(rung_keyed("veteran"), Some(3));
        assert_eq!(rung_keyed("knight"), None);
    }

    /// Only the first rung is open at the start; winning a rung opens the
    /// one above it and no other.
    #[test]
    fn the_first_rung_is_open_and_each_win_opens_the_next() {
        let mut p = Progress::default();
        let open: Vec<bool> = (0..7).map(|i| p.open(i)).collect();
        assert_eq!(open, [true, false, false, false, false, false, false]);
        assert_eq!(p.state(0), State::Open);
        assert_eq!(p.state(1), State::Locked);
        // a loss opens nothing
        assert_eq!(p.record(0, [1, 2]), 0);
        assert!(!p.open(1));
        // a win at rung F opens E, and only E
        assert_eq!(p.record(0, [2, 1]), 1);
        let open: Vec<bool> = (0..7).map(|i| p.open(i)).collect();
        assert_eq!(open, [true, true, false, false, false, false, false]);
        assert_eq!(p.state(0), State::Won(1));
        // a locked rung cannot be recorded
        assert_eq!(p.record(3, [2, 0]), 0);
        assert_eq!(p.marks[3], 0);
        assert!(!p.open(7), "there is no eighth rung");
    }

    /// One mark for a win, two for a win without losing a round; a rung keeps
    /// its best.
    #[test]
    fn a_win_marks_its_rung_once_and_a_clean_win_twice_and_the_best_is_kept() {
        assert_eq!(marks_for([2, 0]), 2);
        assert_eq!(marks_for([2, 1]), 1);
        assert_eq!(marks_for([1, 2]), 0);
        assert_eq!(marks_for([0, 2]), 0);
        let mut p = Progress::default();
        assert_eq!(p.record(0, [2, 0]), 2);
        assert_eq!(p.record(0, [2, 1]), 1, "the duel earned one");
        assert_eq!(p.marks[0], 2, "but the rung keeps its two");
        assert_eq!(p.record(0, [0, 2]), 0);
        assert_eq!(p.marks[0], 2, "a loss takes none away");
        assert_eq!(p.record(1, [2, 1]), 1);
        assert_eq!(p.record(1, [2, 0]), 2);
        assert_eq!(p.marks[1], 2, "a better win raises it");
    }

    /// Marks that could not have been won in order are not a player's.
    #[test]
    fn marks_above_an_unwon_rung_or_past_two_are_not_valid() {
        assert!(Progress::default().valid());
        assert!(marked([2, 1, 1, 2, 0, 0, 0]).valid());
        assert!(marked([2, 2, 2, 2, 2, 2, 2]).valid());
        assert!(!marked([0, 1, 0, 0, 0, 0, 0]).valid(), "E won with F not");
        assert!(!marked([1, 1, 0, 1, 0, 0, 0]).valid(), "C won with D not");
        assert!(!marked([3, 0, 0, 0, 0, 0, 0]).valid(), "three marks");
        let won: Vec<usize> = marked([2, 1, 1, 0, 0, 0, 0]).won_rungs().collect();
        assert_eq!(won, [0, 1, 2]);
    }
}
