//! `rounds` (M2, CA-Q14: A): a duel fought in rounds, best of three.
//!
//! A round ends when a knight falls (its health at none, `Health::down`).
//! A short beat follows (`BEAT_TICKS`) that names who fell and the blow
//! that felled them (`Fall`); then either one side has won `TO_WIN` rounds
//! and the duel is over, or both stand again at their opening marks
//! (`opening`) for the next round. If both fall on the same tick, neither
//! wins it and the round is fought again under the same number.
//!
//! **Sudden death.** After `SUDDEN_TICKS` (90 s) a round enters sudden
//! death: only the inner ring, `SAFE_RADIUS` (6 m) from the middle, is
//! safe ground. A fighter standing outside it loses one health every
//! `SAND_TICKS` ticks it stays out (5 a second, R4.3), and the count
//! starts again once it steps back in.
//!
//! The rounds step at the end of the tick (`World::step`, after the blades
//! and health have had their say) and set the player's record aside
//! between rounds and once the duel is over (`input`), so nothing the
//! player does in the beat reaches the duel. They are off on the port's
//! path (`World::rounds` is none), so CHUMWORLD's recorded duel replays as
//! it did.

use crate::fx::{self, Fx, ONE};
use crate::geom::{facing, yaw_of, V3};
use crate::input::Input;
use crate::player::{self, Body, Fighter};
use crate::pose::{Held, Kind};
use crate::swing::Way;
use crate::world::World;

/// Rounds a side must win to win the duel: two of three.
pub const TO_WIN: u8 = 2;
/// A round enters sudden death after this long: 90 seconds.
pub const SUDDEN_TICKS: u32 = 90 * crate::ticks::TICKS_PER_SECOND as u32;
/// In sudden death, the safe ground: the inner ring, 6 m from the middle
/// (12 m across).
pub const SAFE_RADIUS: Fx = fx::int(6);
/// Outside it, one health is lost each this many ticks: 5 a second.
pub const SAND_TICKS: u32 = 12;
/// The beat after a fall, before the next round or the duel's result:
/// three seconds.
pub const BEAT_TICKS: u32 = 3 * crate::ticks::TICKS_PER_SECOND as u32;
/// The opening marks stand the fighters this far apart besides both
/// reaches (2 × reach + 4 m when they reach alike).
pub const OPENING_SPACE: Fx = fx::int(4);

/// What felled a fighter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blow {
    /// a cut of this kind, by the other fighter
    Cut(Kind2),
    /// a blade moved by the hand alone, no cut under way
    Blade,
    /// sudden death: standing outside the inner ring
    Sand,
}

/// A cut's kind as a fall names it (a level cut either way is one kind).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind2 {
    Level,
    Chop,
    Rising,
    Diagonal,
    Thrust,
}

impl Kind2 {
    pub fn of(way: Way) -> Kind2 {
        match way {
            Way::Level(_) => Kind2::Level,
            Way::Chop => Kind2::Chop,
            Way::Rising => Kind2::Rising,
            Way::Diagonal(_) => Kind2::Diagonal,
            Way::Thrust => Kind2::Thrust,
        }
    }

    /// The blow's string key.
    pub fn key(self) -> &'static str {
        match self {
            Kind2::Level => "blow_level",
            Kind2::Chop => "blow_chop",
            Kind2::Rising => "blow_rising",
            Kind2::Diagonal => "blow_diagonal",
            Kind2::Thrust => "blow_thrust",
        }
    }
}

/// How a round ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fall {
    /// fighter `who` fell (0 the player, 1 the foe), by `blow`
    One { who: u8, blow: Blow },
    /// both fell on the same tick: the round is fought again
    Both,
}

/// Where the duel stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// a round under way
    Fight,
    /// the beat after a fall, ticks left
    Beat(u32),
    /// one side has won `TO_WIN` rounds
    Over,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rounds {
    /// the round under way or just ended, from 1
    pub round: u8,
    /// rounds won: the player's, the foe's
    pub wins: [u8; 2],
    /// ticks of this round's fighting
    pub ticks: u32,
    pub phase: Phase,
    /// how the last round ended
    pub fall: Option<Fall>,
    /// what last took health from each fighter
    pub last_blow: [Option<Blow>; 2],
    /// ticks each fighter has stood outside the inner ring in sudden death,
    /// since it last lost a health to it or stepped back in
    pub outside: [u32; 2],
    /// the pilot the foe is spawned with each round (`pilot::pilot_named`)
    pub foe: String,
}

impl Rounds {
    /// In sudden death now.
    pub fn sudden(&self) -> bool {
        self.phase == Phase::Fight && self.ticks >= SUDDEN_TICKS
    }

    /// The side that has won the duel, once it is over.
    pub fn winner(&self) -> Option<u8> {
        (self.phase == Phase::Over).then(|| if self.wins[0] >= TO_WIN { 0 } else { 1 })
    }
}

/// A duel in rounds begun: the player against the pilot `foe` (by the name
/// `pilot::pilot_named` takes), round 1 at the opening marks.
pub fn begin(w: &mut World, foe: &str) -> Result<(), String> {
    crate::pilot::pilot_named(foe).ok_or_else(|| format!("There is no pilot \"{foe}\"."))?;
    w.rounds = Some(Rounds {
        round: 1,
        wins: [0, 0],
        ticks: 0,
        phase: Phase::Fight,
        fall: None,
        last_blow: [None, None],
        outside: [0, 0],
        foe: foe.to_string(),
    });
    opening(w);
    Ok(())
}

/// The fighters stood at their opening marks, whole, swords drawn: on the
/// line through the ring's middle along `world::START_YAW`, the player
/// behind the middle and the foe ahead of it, apart by both reaches and
/// `OPENING_SPACE`, each facing the other. Blades, blood, the feel of the
/// last blow and the pilots' memory are cleared.
pub fn opening(w: &mut World) {
    let Some(r) = w.rounds.as_ref() else {
        return;
    };
    let (kind, key) = crate::pilot::pilot_named(&r.foe).expect("begun with a known pilot");
    let yaw = crate::world::START_YAW;
    let me = Fighter::new(Kind::Player, Held::ShortSword, 0, V3::ZERO, yaw, true);
    let salt = w.rng.next_u64();
    let them = crate::pilot::foe_keyed(&w.pilots.book, kind, key, V3::ZERO, yaw, salt);
    let gap = crate::pilot::reach(&me) + crate::pilot::reach(&them.fighter) + OPENING_SPACE;
    let ahead = facing(yaw, 0);
    let mine = ahead.scale(-gap / 2);
    let theirs = ahead.scale(gap - gap / 2);
    // each built where it stands, so its arm starts there and not at the middle
    w.player = Body::new(mine, yaw);
    w.me = Fighter::new(Kind::Player, Held::ShortSword, 0, mine, yaw, true);
    let back = yaw_of(mine.sub(theirs));
    let mut foe = crate::pilot::foe_keyed(&w.pilots.book, kind, key, theirs, back, salt);
    foe.brain.control = w.pilots.control.clone();
    w.foes = vec![foe];
    w.aim.yaw = yaw;
    w.aim.pitch = crate::camera::PITCH_START;
    w.aim.rest = (0, 0);
    w.guard_pitch = crate::arm::GUARD_PITCH;
    w.blades = crate::blade::State::default();
    w.feel = crate::feel::State::default();
    w.wounds = crate::wounds::State::default();
    w.pilots.memory = crate::pilot::Memory::default();
}

/// The player's record as the duel takes it: nothing between rounds or
/// once the duel is over.
pub fn input(w: &World, input: &Input) -> Input {
    match &w.rounds {
        Some(r) if r.phase != Phase::Fight => Input::default(),
        _ => *input,
    }
}

/// The end of a tick: blows marked, the round's clock, sudden death, a
/// fall ending the round, the beat counted down.
pub fn step(w: &mut World) {
    let Some(mut r) = w.rounds.take() else {
        return;
    };
    match r.phase {
        Phase::Fight => fight(w, &mut r),
        Phase::Beat(left) if left > 1 => r.phase = Phase::Beat(left - 1),
        Phase::Beat(_) => {
            if r.wins.iter().any(|&n| n >= TO_WIN) {
                r.phase = Phase::Over;
            } else {
                if r.fall != Some(Fall::Both) {
                    r.round += 1;
                }
                r.phase = Phase::Fight;
                r.ticks = 0;
                r.last_blow = [None, None];
                r.outside = [0, 0];
                w.rounds = Some(r);
                opening(w);
                return;
            }
        }
        Phase::Over => {}
    }
    w.rounds = Some(r);
}

fn fight(w: &mut World, r: &mut Rounds) {
    r.ticks += 1;
    // this tick's blows, named by the cut that struck them
    for h in &w.blades.hits {
        let (on, by) = (h.on as usize, h.by as usize);
        if on > 1 || by >= player::count(w) {
            continue;
        }
        let cut = player::fighter(w, by).arm.swing.cut.map(|c| c.way);
        r.last_blow[on] = Some(match cut {
            Some(way) => Blow::Cut(Kind2::of(way)),
            None => Blow::Blade,
        });
    }
    // sudden death: the sand outside the inner ring takes its toll
    if r.ticks > SUDDEN_TICKS {
        for id in 0..player::count(w).min(2) {
            let at = player::body(w, id).pos;
            let out = fx::length(at.x, at.z) > SAFE_RADIUS;
            let h = &mut player::parts_mut(w, id).1.health;
            if !out || h.down() {
                r.outside[id] = 0;
                continue;
            }
            r.outside[id] += 1;
            if r.outside[id] >= SAND_TICKS {
                r.outside[id] = 0;
                h.hp -= 1;
                r.last_blow[id] = Some(Blow::Sand);
            }
        }
    }
    let down: Vec<bool> = (0..2)
        .map(|id| id < player::count(w) && player::fighter(w, id).health.down())
        .collect();
    let fall = match (down[0], down[1]) {
        (true, true) => Some(Fall::Both),
        (true, false) => Some(Fall::One {
            who: 0,
            blow: r.last_blow[0].unwrap_or(Blow::Blade),
        }),
        (false, true) => Some(Fall::One {
            who: 1,
            blow: r.last_blow[1].unwrap_or(Blow::Blade),
        }),
        _ => None,
    };
    if let Some(f) = fall {
        if let Fall::One { who, .. } = f {
            r.wins[1 - who as usize] += 1;
        }
        r.fall = Some(f);
        r.phase = Phase::Beat(BEAT_TICKS);
    }
}

/// `act("rounds.<name>", value)`: `begin` a duel in rounds against the
/// pilot named; `clock` sets the round's ticks and `score` the rounds won
/// ("1,0"), staging sudden death or a duel's last round for a test or a
/// picture.
pub fn act(w: &mut World, name: &str, value: &str) -> Result<(), String> {
    match name {
        "begin" => begin(w, value.trim()),
        "clock" => {
            let t: u32 = value
                .trim()
                .parse()
                .map_err(|_| format!("A round's clock is a whole number of ticks, not \"{value}\"."))?;
            match w.rounds.as_mut() {
                Some(r) => {
                    r.ticks = t;
                    Ok(())
                }
                None => Err("There is no duel in rounds to set the clock of.".into()),
            }
        }
        "score" => {
            let parsed: Option<Vec<u8>> = value.split(',').map(|v| v.trim().parse().ok()).collect();
            let wins = match parsed.as_deref() {
                Some(&[a, b]) if a < TO_WIN && b < TO_WIN => [a, b],
                _ => {
                    return Err(format!(
                    "A score is the rounds each side has won, under {TO_WIN}, as \"1,0\", not \"{value}\"."
                ))
                }
            };
            match w.rounds.as_mut() {
                Some(r) => {
                    r.wins = wins;
                    r.round = 1 + a_plus_b(wins);
                    Ok(())
                }
                None => Err("There is no duel in rounds to set the score of.".into()),
            }
        }
        _ => Err(format!("Unknown action \"rounds.{name}\".")),
    }
}

fn a_plus_b(w: [u8; 2]) -> u8 {
    w[0] + w[1]
}

/// Whole seconds left before sudden death, none once it has come.
pub fn seconds_left(r: &Rounds) -> u32 {
    let tps = crate::ticks::TICKS_PER_SECOND as u32;
    SUDDEN_TICKS.saturating_sub(r.ticks).div_ceil(tps)
}

/// The numbers the glossary names.
pub fn values() -> Vec<(&'static str, String)> {
    let tps = crate::ticks::TICKS_PER_SECOND as u32;
    vec![
        ("rounds.to_win", TO_WIN.to_string()),
        ("rounds.sudden", (SUDDEN_TICKS / tps).to_string()),
        ("rounds.safe_across", (SAFE_RADIUS * 2 / ONE).to_string()),
        ("rounds.sand_rate", (tps / SAND_TICKS).to_string()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::int;
    use crate::geom::v3;

    fn duel() -> World {
        let mut w = World::new(3);
        begin(&mut w, "knight").unwrap();
        crate::player::act(&mut w, "pilot", "still").unwrap();
        w
    }

    fn put(w: &mut World, id: usize, at: V3) {
        let b = player::parts_mut(w, id).0;
        b.pos = at;
        b.prev_pos = at;
    }

    /// By hand: the player's reach and the knight's are both the short
    /// sword's on a body of their build; the gap between their feet is
    /// both reaches and 4 m, split either side of the middle along the
    /// opening heading.
    #[test]
    fn a_round_opens_with_the_fighters_two_reaches_and_four_metres_apart_across_the_middle() {
        let w = duel();
        let (p, f) = (w.player.pos, w.foes[0].body.pos);
        let want = crate::pilot::reach(&w.me) + crate::pilot::reach(&w.foes[0].fighter) + int(4);
        assert!((p.dist(f) - want).abs() <= 4, "{} {want}", p.dist(f));
        // either side of the middle, by half the gap each
        assert!((fx::length(p.x, p.z) - want / 2).abs() <= 4);
        assert!((fx::length(f.x, f.z) - want / 2).abs() <= 4);
        // facing each other, both drawn and whole
        assert_eq!(w.player.facing, crate::world::START_YAW);
        assert!(fx::wrap(w.foes[0].body.facing - yaw_of(p.sub(f))).abs() <= 2);
        assert_eq!(w.me.sheath, crate::player::Sheath::Drawn);
        assert_eq!(w.me.health.hp, 100);
        assert_eq!(
            w.foes[0].fighter.health.hp, 100,
            "the knight's 100 (data/pilots.json)"
        );
        // a short sword reaches about a metre and a third, so they stand
        // about 6.7 m apart: more than 6, less than 7.5
        assert!(want > int(6) && want < int(15) / 2, "{want}");
    }

    /// A fall ends the round: the other side wins it, the beat runs three
    /// seconds naming who fell and the blow, then the next round opens with
    /// both whole at their marks.
    #[test]
    fn a_fall_wins_the_round_for_the_other_side_and_the_next_round_opens_after_the_beat() {
        let mut w = duel();
        w.foes[0].fighter.health.hp = 0;
        w.step(&Input::default());
        let r = w.rounds.clone().unwrap();
        assert_eq!(r.wins, [1, 0]);
        assert_eq!(r.phase, Phase::Beat(BEAT_TICKS));
        assert_eq!(
            r.fall,
            Some(Fall::One {
                who: 1,
                blow: Blow::Blade
            })
        );
        // 180 ticks of beat: the last of them opens round 2
        for _ in 0..BEAT_TICKS - 1 {
            w.step(&Input::default());
        }
        assert_eq!(w.rounds.as_ref().unwrap().round, 1);
        w.step(&Input::default());
        let r = w.rounds.as_ref().unwrap();
        assert_eq!((r.round, r.phase, r.ticks), (2, Phase::Fight, 0));
        assert_eq!(w.foes[0].fighter.health.hp, 100, "stood again, whole");
        assert!(!w.me.health.down());
    }

    /// Best of three: two rounds to the player, whatever came between,
    /// end the duel with the player the winner; the beat still runs first.
    #[test]
    fn two_rounds_won_end_the_duel() {
        let mut w = duel();
        let fell = |w: &mut World, id: usize| {
            player::parts_mut(w, id).1.health.hp = 0;
            for _ in 0..=BEAT_TICKS {
                w.step(&Input::default());
            }
        };
        fell(&mut w, 1);
        fell(&mut w, 0);
        assert_eq!(w.rounds.as_ref().unwrap().wins, [1, 1]);
        assert_eq!(w.rounds.as_ref().unwrap().round, 3);
        assert_eq!(w.rounds.as_ref().unwrap().winner(), None);
        fell(&mut w, 1);
        let r = w.rounds.as_ref().unwrap();
        assert_eq!(r.wins, [2, 1]);
        assert_eq!(r.phase, Phase::Over);
        assert_eq!(r.winner(), Some(0));
        // once over, the player's record is set aside
        let walk = Input {
            keys: crate::input::FORWARD,
            dx: 900,
            ..Input::default()
        };
        assert_eq!(input(&w, &walk), Input::default());
    }

    /// Both down on one tick: no one wins the round and it is fought again
    /// under the same number.
    #[test]
    fn both_falling_at_once_replays_the_round() {
        let mut w = duel();
        w.me.health.hp = 0;
        w.foes[0].fighter.health.hp = -3;
        w.step(&Input::default());
        assert_eq!(w.rounds.as_ref().unwrap().fall, Some(Fall::Both));
        assert_eq!(w.rounds.as_ref().unwrap().wins, [0, 0]);
        for _ in 0..BEAT_TICKS {
            w.step(&Input::default());
        }
        let r = w.rounds.as_ref().unwrap();
        assert_eq!((r.round, r.wins, r.phase), (1, [0, 0], Phase::Fight));
    }

    /// Sudden death by hand. Until 90 s nothing is lost on the sand; from
    /// then on a fighter outside the inner ring (6 m) loses 1 health every
    /// 12 ticks, 5 a second; inside, nothing. Ten seconds outside: 50.
    #[test]
    fn after_ninety_seconds_standing_outside_the_inner_ring_costs_five_health_a_second() {
        let mut w = duel();
        // the player 9 m out, the knight 3 m out, neither moving
        put(&mut w, 0, v3(int(9), 0, 0));
        put(&mut w, 1, v3(-int(3), 0, 0));
        w.rounds.as_mut().unwrap().ticks = SUDDEN_TICKS - 60;
        for _ in 0..60 {
            w.step(&Input::default());
        }
        assert_eq!(w.me.health.hp, 100, "the first 90 s cost nothing");
        assert!(w.rounds.as_ref().unwrap().sudden());
        for _ in 0..600 {
            w.step(&Input::default());
        }
        assert_eq!(w.me.health.hp, 50, "600 ticks outside: 600 / 12 = 50");
        assert_eq!(w.foes[0].fighter.health.hp, 100, "inside the inner ring: nothing");
        assert_eq!(w.rounds.as_ref().unwrap().last_blow[0], Some(Blow::Sand));
    }

    /// The sand fells a fighter like a blow: the round goes to the other
    /// side and the fall names the sand.
    #[test]
    fn a_fall_on_the_sand_in_sudden_death_names_the_sand() {
        let mut w = duel();
        put(&mut w, 0, v3(0, 0, int(10)));
        put(&mut w, 1, v3(0, 0, -int(2)));
        w.me.health.hp = 2;
        w.rounds.as_mut().unwrap().ticks = SUDDEN_TICKS;
        for _ in 0..24 {
            w.step(&Input::default());
        }
        let r = w.rounds.as_ref().unwrap();
        assert_eq!(
            r.fall,
            Some(Fall::One {
                who: 0,
                blow: Blow::Sand
            })
        );
        assert_eq!(r.wins, [0, 1]);
    }

    #[test]
    fn the_seconds_before_sudden_death_count_down_by_whole_seconds() {
        let mut w = duel();
        let r = w.rounds.as_mut().unwrap();
        assert_eq!(seconds_left(r), 90);
        r.ticks = 1;
        assert_eq!(seconds_left(r), 90, "89.98 s rounds up");
        r.ticks = SUDDEN_TICKS - 60;
        assert_eq!(seconds_left(r), 1);
        r.ticks = SUDDEN_TICKS;
        assert_eq!(seconds_left(r), 0);
    }
}
