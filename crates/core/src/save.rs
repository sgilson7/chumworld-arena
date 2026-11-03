//! `save` (M2, CA-Q4: C, T2): the one browser save, and the same save as a
//! JSON file.
//!
//! A save holds the reader's settings (look speed, inverted pitch, field of
//! view, aim help) and the knight's look, A or B (`Knight`; the knights
//! themselves arrive in M3, so the choice is kept before anything shows it,
//! R4.8). M6: it holds the ladder's progress, each rung's marks
//! (`ladder::Progress`), and the look may be a won rung's opponent's
//! (`Knight::Rung`). It holds nothing about the person: no name, no date or
//! clock reading, nothing of the device.
//!
//! M6 writes version 2 and reads versions 1 and 2: a version 1 save (the
//! MVP's, with no ladder) reads as the same settings and look with no rung
//! won (`read`, `migrate`).
//!
//! The text (`text`) is JSON: a format name and a version, a checksum over
//! the `game` object as written (FNV-1a, 64 bits, over its compact text with
//! sorted keys, as CHUMWORLD's save does), and `game` itself. `read` checks
//! every field, its type and its range, and the checksum, so a hand edit, a
//! corrupt file or another version is refused whole with the reason
//! (`Refusal`), and the game is as it was.
//!
//! The core does no input or output: it returns the text, and the page
//! (`web/save.js`, the one file that names browser storage, `one_save.rs`)
//! keeps it under one key and offers it as a download.

use serde_json::{json, Map, Value};

use crate::page::Strings;
use crate::world::AimHelp;

/// What the file calls itself, and the version this game writes and reads.
pub const FORMAT: &str = "chumworld-arena-save";
pub const VERSION: u64 = 2;
/// M6: the MVP's version, read and brought up to this one (no rung won).
pub const VERSION_MVP: u64 = 1;
/// Longer than this is not a save.
pub const TEXT_MOST: usize = 16_384;

/// The knight's look (CA-Q13: A): knight A, the black tower, or knight B,
/// the ivory crown; M6: or the look of a won rung's opponent, by the rung's
/// number from the foot (`ladder::RUNGS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Knight {
    A,
    B,
    Rung(u8),
}

impl Knight {
    pub fn id(self) -> &'static str {
        match self {
            Knight::A => "a",
            Knight::B => "b",
            Knight::Rung(i) => crate::ladder::RUNGS[i as usize].key,
        }
    }

    /// M3: the body the player wears in a duel: Knight A or Knight B (the
    /// computer's knight wears the other, `rounds::begin`). M6 (CA-Q9: A): a
    /// rung's look is drawn over Knight A's body, so its plate, reach and
    /// strength stay a knight's.
    pub fn kind(self) -> crate::pose::Kind {
        match self {
            Knight::A => crate::pose::Kind::KnightA,
            Knight::B => crate::pose::Kind::KnightB,
            Knight::Rung(_) => crate::pose::Kind::KnightA,
        }
    }

    /// M6: what the player's figure is drawn as (`Fighter::dress`).
    pub fn dress(self) -> crate::pose::Kind {
        match self {
            Knight::Rung(i) => crate::ladder::RUNGS[i as usize].kind,
            other => other.kind(),
        }
    }

    pub fn named(s: &str) -> Option<Knight> {
        match s {
            "a" => Some(Knight::A),
            "b" => Some(Knight::B),
            other => crate::ladder::rung_keyed(other).map(|i| Knight::Rung(i as u8)),
        }
    }

    /// M6: the look may be worn with this progress: the knights always, a
    /// rung's opponent's once the rung is won.
    pub fn worn_by(self, p: &crate::ladder::Progress) -> bool {
        match self {
            Knight::Rung(i) => p.won(i as usize),
            _ => true,
        }
    }

    /// M6: every look this progress may wear, the knights first, then each
    /// won rung's from the foot.
    pub fn all(p: &crate::ladder::Progress) -> Vec<Knight> {
        let mut out = vec![Knight::A, Knight::B];
        out.extend(p.won_rungs().map(|i| Knight::Rung(i as u8)));
        out
    }
}

/// What a save keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kept {
    /// the look speed, percent (`camera::LOOK_PCT_MIN` to `LOOK_PCT_MAX`)
    pub look_pct: i32,
    pub invert_pitch: bool,
    /// degrees across (`world::FOV_MIN` to `FOV_MAX`)
    pub fov: i32,
    pub aim_help: AimHelp,
    pub knight: Knight,
    /// M6: each rung's marks
    pub ladder: crate::ladder::Progress,
}

impl Default for Kept {
    fn default() -> Kept {
        Kept {
            look_pct: 100,
            invert_pitch: false,
            fov: crate::world::FOV_DEFAULT,
            aim_help: AimHelp::None,
            knight: Knight::A,
            ladder: crate::ladder::Progress::default(),
        }
    }
}

fn aim_help_id(a: AimHelp) -> &'static str {
    match a {
        AimHelp::None => "none",
        AimHelp::SoftPull => "soft_pull",
    }
}

/// FNV-1a, 64 bits: the checksum over the game object's compact text.
pub fn checksum(text: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}")
}

/// The keys of a save's `game` object and of its settings: what a save
/// holds, and nothing else.
pub const GAME_KEYS: [&str; 3] = ["knight", "ladder", "settings"];
/// M6: the MVP's save (version 1) had no ladder.
pub const GAME_KEYS_MVP: [&str; 2] = ["knight", "settings"];
pub const SETTING_KEYS: [&str; 4] = ["aim_help", "fov", "invert_pitch", "look_pct"];
pub const TOP_KEYS: [&str; 4] = ["checksum", "format", "game", "version"];

/// The `game` object of a save.
pub fn game_value(k: &Kept) -> Value {
    json!({
        "knight": k.knight.id(),
        "ladder": k.ladder.marks,
        "settings": {
            "aim_help": aim_help_id(k.aim_help),
            "fov": k.fov,
            "invert_pitch": k.invert_pitch,
            "look_pct": k.look_pct,
        },
    })
}

/// A save's text: the format, the version, the checksum, the game.
pub fn text(k: &Kept) -> String {
    let game = game_value(k);
    let top = json!({
        "format": FORMAT,
        "version": VERSION,
        "checksum": checksum(&game.to_string()),
        "game": game,
    });
    serde_json::to_string_pretty(&top).unwrap_or_default()
}

/// Why a save was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// not JSON, empty, or far too long
    Unreadable,
    /// JSON, but not this game's save
    Format,
    /// this game's save, of another version
    Version,
    /// changed since it was written
    Checksum,
    /// a field missing, of the wrong type, out of range or unknown
    Field(String),
}

impl Refusal {
    /// The reason, as the refusal line's `{why}`.
    pub fn why(&self, s: &Strings) -> String {
        match self {
            Refusal::Unreadable => s.get("save_why_unreadable"),
            Refusal::Format => s.get("save_why_format"),
            Refusal::Version => s.get("save_why_version"),
            Refusal::Checksum => s.get("save_why_checksum"),
            Refusal::Field(f) => crate::api::fill(&s.get("save_why_field"), &[("field", f.clone())]),
        }
    }
}

fn bad(field: &str) -> Refusal {
    Refusal::Field(field.into())
}

fn object<'a>(v: &'a Value, field: &str) -> Result<&'a Map<String, Value>, Refusal> {
    v.as_object().ok_or_else(|| bad(field))
}

/// The object's keys are exactly `want`.
fn keys_are(m: &Map<String, Value>, want: &[&str], field: &str) -> Result<(), Refusal> {
    let mut have: Vec<&str> = m.keys().map(|k| k.as_str()).collect();
    have.sort();
    let mut want = want.to_vec();
    want.sort();
    if have == want {
        Ok(())
    } else {
        let odd = have
            .iter()
            .find(|k| !want.contains(k))
            .or_else(|| want.iter().find(|k| !have.contains(k)))
            .map(|k| format!("{field}.{k}"))
            .unwrap_or_else(|| field.to_string());
        Err(Refusal::Field(odd))
    }
}

fn int_in(v: &Value, field: &str, lo: i32, hi: i32) -> Result<i32, Refusal> {
    v.as_i64()
        .filter(|n| (lo as i64..=hi as i64).contains(n))
        .map(|n| n as i32)
        .ok_or_else(|| bad(field))
}

/// A save's text read back, every field checked; or why it was refused.
pub fn read(text: &str) -> Result<Kept, Refusal> {
    if text.trim().is_empty() || text.len() > TEXT_MOST {
        return Err(Refusal::Unreadable);
    }
    let top: Value = serde_json::from_str(text).map_err(|_| Refusal::Unreadable)?;
    let m = top.as_object().ok_or(Refusal::Unreadable)?;
    if m.get("format").and_then(|f| f.as_str()) != Some(FORMAT) {
        return Err(Refusal::Format);
    }
    let version = m.get("version").and_then(|v| v.as_u64());
    if version != Some(VERSION) && version != Some(VERSION_MVP) {
        return Err(Refusal::Version);
    }
    keys_are(m, &TOP_KEYS, "save")?;
    let game = &m["game"];
    let sum = m["checksum"].as_str().ok_or_else(|| bad("checksum"))?;
    if checksum(&game.to_string()) != sum {
        return Err(Refusal::Checksum);
    }
    let g = object(game, "game")?;
    // M6: a version 1 save is brought up to this one with no rung won
    let ladder = if version == Some(VERSION_MVP) {
        keys_are(g, &GAME_KEYS_MVP, "game")?;
        crate::ladder::Progress::default()
    } else {
        keys_are(g, &GAME_KEYS, "game")?;
        ladder_of(&g["ladder"])?
    };
    let knight = g["knight"]
        .as_str()
        .and_then(Knight::named)
        .filter(|k| k.worn_by(&ladder))
        .ok_or_else(|| bad("knight"))?;
    let set = object(&g["settings"], "settings")?;
    keys_are(set, &SETTING_KEYS, "settings")?;
    let aim_help = match set["aim_help"].as_str() {
        Some("none") => AimHelp::None,
        Some("soft_pull") => AimHelp::SoftPull,
        _ => return Err(bad("settings.aim_help")),
    };
    Ok(Kept {
        look_pct: int_in(
            &set["look_pct"],
            "settings.look_pct",
            crate::camera::LOOK_PCT_MIN,
            crate::camera::LOOK_PCT_MAX,
        )?,
        invert_pitch: set["invert_pitch"]
            .as_bool()
            .ok_or_else(|| bad("settings.invert_pitch"))?,
        fov: int_in(
            &set["fov"],
            "settings.fov",
            crate::world::FOV_MIN,
            crate::world::FOV_MAX,
        )?,
        aim_help,
        knight,
        ladder,
    })
}

/// M6: the ladder's marks, one whole number a rung from the foot, each
/// from none to `MARKS_MOST`, won in order (`Progress::valid`).
fn ladder_of(v: &Value) -> Result<crate::ladder::Progress, Refusal> {
    let list = v.as_array().ok_or_else(|| bad("ladder"))?;
    let mut p = crate::ladder::Progress::default();
    if list.len() != p.marks.len() {
        return Err(bad("ladder"));
    }
    for (i, m) in list.iter().enumerate() {
        p.marks[i] = int_in(m, "ladder", 0, crate::ladder::MARKS_MOST as i32)? as u8;
    }
    if !p.valid() {
        return Err(bad("ladder"));
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings() -> Strings {
        crate::page::parse_strings(include_str!("../../../data/strings.json")).unwrap()
    }

    fn odd() -> Kept {
        Kept {
            look_pct: 140,
            invert_pitch: true,
            fov: 110,
            aim_help: AimHelp::SoftPull,
            knight: Knight::B,
            ladder: crate::ladder::Progress::default(),
        }
    }

    /// M6: three rungs won (F with two marks, E and D with one), wearing the
    /// sellsword's look.
    fn climbed() -> Kept {
        Kept {
            knight: Knight::Rung(1),
            ladder: crate::ladder::Progress {
                marks: [2, 1, 1, 0, 0, 0, 0],
            },
            ..odd()
        }
    }

    #[test]
    fn a_save_reads_back_as_it_was_written() {
        for k in [Kept::default(), odd(), climbed()] {
            assert_eq!(read(&text(&k)), Ok(k));
        }
    }

    /// The text by hand: the game object's compact text with sorted keys,
    /// and nothing in it but the settings, the knight and (M6) the ladder.
    #[test]
    fn a_save_holds_the_settings_and_the_knight_and_nothing_else() {
        let t = text(&odd());
        let v: Value = serde_json::from_str(&t).unwrap();
        assert_eq!(v["format"], "chumworld-arena-save");
        assert_eq!(v["version"], 2);
        let compact = r#"{"knight":"b","ladder":[0,0,0,0,0,0,0],"settings":{"aim_help":"soft_pull","fov":110,"invert_pitch":true,"look_pct":140}}"#;
        assert_eq!(v["game"].to_string(), compact);
        assert_eq!(v["checksum"], checksum(compact));
        // FNV-1a of "a", by hand: (0xcbf29ce484222325 ^ 0x61) × 0x100000001b3
        let one = (0xcbf2_9ce4_8422_2325u64 ^ 0x61).wrapping_mul(0x0100_0000_01b3);
        assert_eq!(checksum("a"), format!("{one:016x}"));
    }

    /// A bad file is refused whole, with a reason a reader can act on.
    #[test]
    fn a_bad_or_old_save_is_refused_with_its_reason() {
        let s = strings();
        let good = text(&odd());
        assert_eq!(read(""), Err(Refusal::Unreadable));
        assert_eq!(read("not json"), Err(Refusal::Unreadable));
        assert_eq!(read(r#"{"format":"slides","version":1}"#), Err(Refusal::Format));
        let old = good.replace("\"version\": 2", "\"version\": 0");
        assert_ne!(old, good);
        assert_eq!(read(&old), Err(Refusal::Version));
        assert_eq!(
            Refusal::Version.why(&s),
            "it is from another version of the game."
        );
        // a hand edit: the field of view widened past the checksum
        let edited = good.replace("\"fov\": 110", "\"fov\": 109");
        assert_ne!(edited, good);
        assert_eq!(read(&edited), Err(Refusal::Checksum));
        // a field out of range, with a checksum to match: refused by name
        let mut v: Value = serde_json::from_str(&good).unwrap();
        v["game"]["settings"]["fov"] = json!(140);
        v["checksum"] = json!(checksum(&v["game"].to_string()));
        assert_eq!(read(&v.to_string()), Err(Refusal::Field("settings.fov".into())));
        assert_eq!(
            Refusal::Field("settings.fov".into()).why(&s),
            "its \"settings.fov\" is not what a save holds."
        );
        // a field more than a save holds
        let mut v: Value = serde_json::from_str(&good).unwrap();
        v["game"]["name"] = json!("Sam");
        v["checksum"] = json!(checksum(&v["game"].to_string()));
        assert_eq!(read(&v.to_string()), Err(Refusal::Field("game.name".into())));
        // too long to be a save
        assert_eq!(read(&" ".repeat(TEXT_MOST + 1)), Err(Refusal::Unreadable));
    }

    /// M6: the ladder's progress and a won rung's look are written and read
    /// back; the text holds each rung's marks from the foot, by hand.
    #[test]
    fn a_save_keeps_the_ladder_s_marks_and_a_won_rung_s_look() {
        let k = climbed();
        let t = text(&k);
        let v: Value = serde_json::from_str(&t).unwrap();
        let compact = r#"{"knight":"sellsword","ladder":[2,1,1,0,0,0,0],"settings":{"aim_help":"soft_pull","fov":110,"invert_pitch":true,"look_pct":140}}"#;
        assert_eq!(v["game"].to_string(), compact);
        assert_eq!(read(&t), Ok(k));
        // the look's body stays Knight A's; only the drawing is the sellsword
        assert_eq!(k.knight.kind(), crate::pose::Kind::KnightA);
        assert_eq!(k.knight.dress(), crate::pose::Kind::Sellsword);
    }

    /// M6: the MVP's save (version 1, no ladder) loads as the same settings
    /// and look with no rung won. The text is a version 1 save as the MVP
    /// wrote it, its checksum worked out over its own game object.
    #[test]
    fn an_mvp_save_loads_with_no_rung_won() {
        let game =
            r#"{"knight":"b","settings":{"aim_help":"none","fov":108,"invert_pitch":false,"look_pct":90}}"#;
        let t = format!(
            r#"{{"format":"chumworld-arena-save","version":1,"checksum":"{}","game":{game}}}"#,
            checksum(game)
        );
        let k = read(&t).expect("an MVP save loads");
        assert_eq!(k.fov, 108);
        assert_eq!(k.look_pct, 90);
        assert_eq!(k.knight, Knight::B);
        assert_eq!(k.ladder, crate::ladder::Progress::default());
        // written again, it is a version 2 save
        assert!(text(&k).contains("\"version\": 2"));
        // a version 1 save with a ladder is not one the MVP wrote
        let game = r#"{"knight":"b","ladder":[0,0,0,0,0,0,0],"settings":{"aim_help":"none","fov":108,"invert_pitch":false,"look_pct":90}}"#;
        let t = format!(
            r#"{{"format":"chumworld-arena-save","version":1,"checksum":"{}","game":{game}}}"#,
            checksum(game)
        );
        assert_eq!(read(&t), Err(Refusal::Field("game.ladder".into())));
    }

    /// M6: marks that could not have been won, or a look not yet won, are
    /// refused by name even with a checksum to match.
    #[test]
    fn a_save_with_marks_out_of_order_or_an_unwon_look_is_refused() {
        let with = |ladder: Value, knight: &str| {
            let mut v: Value = serde_json::from_str(&text(&climbed())).unwrap();
            v["game"]["ladder"] = ladder;
            v["game"]["knight"] = json!(knight);
            v["checksum"] = json!(checksum(&v["game"].to_string()));
            read(&v.to_string())
        };
        let ladder = Err(Refusal::Field("ladder".into()));
        assert!(with(json!([2, 1, 1, 0, 0, 0, 0]), "sellsword").is_ok());
        assert_eq!(with(json!([0, 1, 0, 0, 0, 0, 0]), "a"), ladder, "E won, F not");
        assert_eq!(with(json!([3, 0, 0, 0, 0, 0, 0]), "a"), ladder, "three marks");
        assert_eq!(with(json!([1, 0, 0]), "a"), ladder, "three rungs");
        assert_eq!(with(json!([1, 0, 0, 0, 0, 0, -1]), "a"), ladder);
        assert_eq!(with(json!("FED"), "a"), ladder);
        // the veteran's look with rung C not won
        assert_eq!(
            with(json!([2, 1, 1, 0, 0, 0, 0]), "veteran"),
            Err(Refusal::Field("knight".into()))
        );
        assert_eq!(
            with(json!([2, 1, 1, 0, 0, 0, 0]), "knight_c"),
            Err(Refusal::Field("knight".into()))
        );
    }
}
