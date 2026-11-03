//! `ladder` (M6, T6; T11 the Single Player Campaign): the single-player
//! run of duels, its battles in regions on a map, what opens each, and what
//! winning them gives.
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

use crate::pose::{Held, Kind};

/// T11 (CA-Q21: A): what opens a battle, each a line into it on the
/// Campaign page's Hasse diagram: battle `j` won; won without losing a
/// round (both marks); won holding a weapon; or `n` of a list won.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Need {
    Won(usize),
    Clean(usize),
    With(usize, Held),
    AnyOf(&'static [usize], u8),
}

/// T11 (CA-Q18: A): a region of the campaign's map, by its key in the
/// strings (`region_<key>`), and the marks won in the region before it
/// that open it (none for the first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub key: &'static str,
    pub gate: u8,
}

pub const REGIONS: [Region; 4] = [
    Region { key: "yard", gate: 0 },
    Region {
        key: "marches",
        gate: 4,
    },
    Region { key: "toll", gate: 4 },
    Region {
        key: "tourney",
        gate: 6,
    },
];

/// One battle (M6's rung): its rank's letter (the seven ladder rungs; none
/// for the new battles), the key its pilot, its strings and the save use
/// (`data/pilots.json`, `who_<key>`, `dossier_<key>`), the body its
/// opponent fights in, its region, the weapon beating it unlocks (the
/// opponent carries it), and what opens it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rung {
    pub rank: &'static str,
    pub key: &'static str,
    pub kind: Kind,
    pub region: usize,
    pub held: Option<Held>,
    pub needs: &'static [Need],
}

const fn rung(
    rank: &'static str,
    key: &'static str,
    kind: Kind,
    region: usize,
    held: Option<Held>,
    needs: &'static [Need],
) -> Rung {
    Rung {
        rank,
        key,
        kind,
        region,
        held,
        needs,
    }
}

/// The battles, by region from the Yard to the High Tourney. The seven
/// ladder rungs keep their order (C needs E, B a clean C, A needs B, S a
/// clean A); the six new battles each carry a new weapon, each opponent
/// in its own costume (R18).
pub const RUNGS: [Rung; 13] = [
    rung("F", "squire", Kind::Squire, 0, None, &[]),
    rung("E", "sellsword", Kind::Sellsword, 0, None, &[Need::Won(0)]),
    rung("D", "hedge", Kind::Hedge, 0, None, &[Need::Won(0)]),
    rung("C", "veteran", Kind::Veteran, 1, None, &[Need::Won(1)]),
    rung(
        "",
        "woodsman",
        Kind::Woodsman,
        1,
        Some(Held::Axe),
        &[Need::Won(2)],
    ),
    rung(
        "",
        "friar",
        Kind::Friar,
        1,
        Some(Held::Quarterstaff),
        &[Need::Won(3), Need::Won(4)],
    ),
    rung("B", "banneret", Kind::Banneret, 2, None, &[Need::Clean(3)]),
    rung("", "warden", Kind::Warden, 2, Some(Held::Mace), &[Need::Won(5)]),
    rung(
        "",
        "reiver",
        Kind::Reiver,
        2,
        Some(Held::Broadsword),
        &[Need::With(4, Held::Axe)],
    ),
    rung(
        "",
        "pikeman",
        Kind::Pikeman,
        2,
        Some(Held::Spear),
        &[Need::AnyOf(&[6, 7, 8], 2)],
    ),
    rung(
        "A",
        "champion",
        Kind::Champion,
        3,
        None,
        &[Need::Won(6), Need::Won(7)],
    ),
    rung(
        "",
        "headsman",
        Kind::Headsman,
        3,
        Some(Held::GreatAxe),
        &[Need::Won(8)],
    ),
    rung(
        "S",
        "mirror",
        Kind::Mirror,
        3,
        None,
        &[Need::Clean(10), Need::Clean(11)],
    ),
];

/// The weapons a player may hold, in a fixed order: the longsword from the
/// start, the others once their carrier is beaten (CA-Q19: C). A battle
/// won keeps which of them it was won with, one bit each.
pub const WEAPONS: [Held; 7] = [
    Held::Longsword,
    Held::Broadsword,
    Held::Axe,
    Held::GreatAxe,
    Held::Mace,
    Held::Quarterstaff,
    Held::Spear,
];

/// A weapon's bit in `Progress::with`, if a player may hold it.
pub fn weapon_bit(h: Held) -> Option<u8> {
    WEAPONS.iter().position(|&w| w == h).map(|k| 1 << k)
}

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

/// How far through the campaign the player is: each battle's marks, and
/// the weapons each was won with (`WEAPONS`' bits).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Progress {
    pub marks: [u8; RUNGS.len()],
    pub with: [u8; RUNGS.len()],
}

/// How a battle's card shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Locked,
    Open,
    Won(u8),
}

impl Progress {
    /// The marks won in region `r`.
    pub fn region_marks(&self, r: usize) -> u8 {
        (0..RUNGS.len())
            .filter(|&i| RUNGS[i].region == r)
            .map(|i| self.marks[i])
            .sum()
    }

    /// Region `r` is open: the first always, any other once the region
    /// before it holds its gate's marks.
    pub fn region_open(&self, r: usize) -> bool {
        r < REGIONS.len() && (r == 0 || self.region_marks(r - 1) >= REGIONS[r].gate)
    }

    /// Whether `need` is met.
    pub fn meets(&self, need: Need) -> bool {
        match need {
            Need::Won(j) => self.won(j),
            Need::Clean(j) => j < RUNGS.len() && self.marks[j] >= MARKS_MOST,
            Need::With(j, h) => j < RUNGS.len() && weapon_bit(h).is_some_and(|b| self.with[j] & b != 0),
            Need::AnyOf(list, n) => list.iter().filter(|&&j| self.won(j)).count() >= n as usize,
        }
    }

    /// The battle may be fought: its region is open and every line into it
    /// is met.
    pub fn open(&self, i: usize) -> bool {
        i < RUNGS.len() && self.region_open(RUNGS[i].region) && RUNGS[i].needs.iter().all(|&n| self.meets(n))
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

    /// A duel in battle `i` is over with these rounds won, the player
    /// holding `held`: the battle keeps the better of its marks and the
    /// duel's, and a win keeps the weapon it was won with. The marks the
    /// duel earned, or none when the battle was not open (nothing changes).
    pub fn record(&mut self, i: usize, wins: [u8; 2], held: Held) -> u8 {
        if !self.open(i) {
            return 0;
        }
        let got = marks_for(wins);
        self.marks[i] = self.marks[i].max(got);
        if got > 0 {
            self.with[i] |= weapon_bit(held).unwrap_or(0);
        }
        got
    }

    /// Progress a save may hold: no battle past `MARKS_MOST`, and weapons
    /// kept only on a won battle, each one a player may hold. (M6 also
    /// refused marks won out of order; a ladder save carried into the
    /// campaign may hold wins its new lines would not open, so that check
    /// is gone, R16.3.)
    pub fn valid(&self) -> bool {
        (0..RUNGS.len()).all(|i| {
            self.marks[i] <= MARKS_MOST
                && (self.with[i] == 0 || self.marks[i] > 0)
                && self.with[i] < 1 << WEAPONS.len()
        })
    }

    /// The battles won, by number from the first.
    pub fn won_rungs(&self) -> impl Iterator<Item = usize> + '_ {
        (0..RUNGS.len()).filter(|&i| self.won(i))
    }

    /// The weapons the player may hold: the longsword, and each won
    /// battle's carrier's, in `WEAPONS`' order.
    pub fn weapons(&self) -> Vec<Held> {
        WEAPONS
            .iter()
            .copied()
            .filter(|&h| h == Held::Longsword || self.won_rungs().any(|i| RUNGS[i].held == Some(h)))
            .collect()
    }
}

/// The numbers the campaign's strings and glossary entries name.
pub fn values() -> Vec<(&'static str, String)> {
    vec![
        ("campaign.rungs", RUNGS.len().to_string()),
        ("campaign.marks", MARKS_MOST.to_string()),
        ("campaign.regions", REGIONS.len().to_string()),
        ("campaign.foot", RUNGS[0].rank.to_string()),
        ("campaign.top", RUNGS[RUNGS.len() - 1].rank.to_string()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Progress with these battles marked, each won with the longsword.
    fn marked(m: [u8; 13]) -> Progress {
        let mut p = Progress {
            marks: m,
            ..Progress::default()
        };
        for (w, &k) in p.with.iter_mut().zip(m.iter()) {
            *w = (k > 0) as u8;
        }
        p
    }

    /// Thirteen battles in four regions; the seven ladder rungs keep their
    /// ranks and order, and each new battle's opponent carries the weapon
    /// beating it unlocks (its pilot's book entry holds it).
    #[test]
    fn thirteen_battles_in_four_regions_keep_the_ladder_s_order() {
        let ranks: Vec<&str> = RUNGS.iter().map(|r| r.rank).filter(|r| !r.is_empty()).collect();
        assert_eq!(ranks, ["F", "E", "D", "C", "B", "A", "S"]);
        let regions: Vec<usize> = RUNGS.iter().map(|r| r.region).collect();
        assert_eq!(regions, [0, 0, 0, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3]);
        let book = crate::pilot::Book::standard();
        for r in RUNGS {
            let (kind, key) = crate::pilot::pilot_named(r.key).unwrap_or_else(|| panic!("{}", r.key));
            assert_eq!((kind, key), (r.kind, r.key), "{}", r.key);
            if let Some(h) = r.held {
                assert_eq!(
                    crate::watch::held_key(h),
                    book.kinds[r.key].held,
                    "{} carries its weapon",
                    r.key
                );
            }
        }
        assert_eq!(rung_keyed("veteran"), Some(3));
        assert_eq!(rung_keyed("knight"), None);
    }

    /// At the start only the squire is open. A win opens the sellsword and
    /// the hedge knight; the Marches stay shut until the Yard holds 4
    /// marks, even with the sellsword won (the veteran's line).
    #[test]
    fn only_the_first_battle_is_open_and_the_marches_wait_for_four_marks() {
        let mut p = Progress::default();
        let open: Vec<usize> = (0..13).filter(|&i| p.open(i)).collect();
        assert_eq!(open, [0]);
        assert_eq!(p.record(0, [1, 2], Held::Longsword), 0, "a loss opens nothing");
        assert_eq!(p.record(0, [2, 1], Held::Longsword), 1);
        let open: Vec<usize> = (0..13).filter(|&i| p.open(i)).collect();
        assert_eq!(open, [0, 1, 2]);
        p.record(1, [2, 1], Held::Longsword);
        p.record(2, [2, 1], Held::Longsword);
        assert_eq!(p.region_marks(0), 3);
        assert!(!p.open(3) && !p.open(4), "3 marks: the Marches are shut");
        p.record(0, [2, 0], Held::Longsword);
        assert_eq!(p.region_marks(0), 4);
        assert!(p.open(3) && p.open(4), "4 marks: the veteran and the woodsman");
        assert!(!p.open(5), "the friar needs both");
        assert_eq!(
            p.record(5, [2, 0], Held::Longsword),
            0,
            "a shut battle records nothing"
        );
    }

    /// Each kind of line: the banneret needs the veteran won clean; the
    /// reiver needs the woodsman won with an axe; the pikeman two of three.
    #[test]
    fn each_kind_of_line_opens_its_battle_and_nothing_less() {
        let mut p = marked([2, 2, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert!(p.region_open(1));
        assert!(!p.region_open(2), "2 marks in the Marches, under 4");
        p.marks[5] = 2;
        p.with[5] = 1;
        assert!(p.region_open(2), "4 marks");
        assert!(!p.open(6), "the veteran won, but not clean");
        p.marks[3] = 2;
        assert!(p.open(6));
        assert!(!p.open(8), "the woodsman won with the longsword");
        p.record(4, [2, 1], Held::Axe);
        assert!(p.open(8), "won with an axe too");
        assert!(!p.open(9));
        p.marks[6] = 1;
        assert!(!p.open(9), "one of three");
        p.marks[8] = 1;
        assert!(p.open(9), "two of three");
    }

    /// One mark for a win, two for a win without losing a round; a battle
    /// keeps its best, and each weapon it was won with.
    #[test]
    fn a_win_marks_its_battle_and_keeps_its_weapon() {
        assert_eq!(marks_for([2, 0]), 2);
        assert_eq!(marks_for([2, 1]), 1);
        assert_eq!(marks_for([1, 2]), 0);
        let mut p = Progress::default();
        assert_eq!(p.record(0, [2, 0], Held::Longsword), 2);
        assert_eq!(p.record(0, [2, 1], Held::Mace), 1);
        assert_eq!(p.marks[0], 2, "the best is kept");
        assert_eq!(p.with[0], 0b1_0001, "the longsword and the mace");
        assert_eq!(p.record(0, [0, 2], Held::Spear), 0);
        assert_eq!(p.with[0], 0b1_0001, "a loss keeps no weapon");
    }

    /// The longsword from the start; a carrier beaten gives its weapon.
    #[test]
    fn beating_a_carrier_gives_its_weapon() {
        assert_eq!(Progress::default().weapons(), [Held::Longsword]);
        let p = marked([2, 2, 2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(p.weapons(), [Held::Longsword, Held::Axe]);
    }

    /// A save's progress is bounded: two marks at most, and weapons kept
    /// only on won battles, each one of the seven. Wins its lines would not
    /// open now are kept (a ladder save carried over, R16.3).
    #[test]
    fn a_save_s_progress_is_bounded() {
        assert!(Progress::default().valid());
        assert!(marked([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]).valid());
        assert!(
            marked([1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0]).valid(),
            "a ladder save's order"
        );
        assert!(
            !marked([3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).valid(),
            "three marks"
        );
        let mut p = marked([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        p.with[1] = 1;
        assert!(!p.valid(), "a weapon kept on a battle not won");
        let mut p = marked([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        p.with[0] = 1 << 7;
        assert!(!p.valid(), "no eighth weapon");
    }
}
