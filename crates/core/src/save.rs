//! `save` (M2, CA-Q4: C, T2): the one browser save, and the same save as a
//! JSON file.
//!
//! A save holds the reader's settings (look speed, inverted pitch, field of
//! view, aim help) and the knight's look, A or B (`Knight`; the knights
//! themselves arrive in M3, so the choice is kept before anything shows it,
//! R4.8). From M6 it will hold the ladder's progress. It holds nothing about
//! the person: no name, no date or clock reading, nothing of the device.
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
pub const VERSION: u64 = 1;
/// Longer than this is not a save.
pub const TEXT_MOST: usize = 16_384;

/// The knight's look (CA-Q13: A): knight A, the black tower, or knight B,
/// the ivory crown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Knight {
    A,
    B,
}

impl Knight {
    pub fn id(self) -> &'static str {
        match self {
            Knight::A => "a",
            Knight::B => "b",
        }
    }

    /// M3: the body the player wears in a duel: Knight A or Knight B (the
    /// computer's knight wears the other, `rounds::begin`).
    pub fn kind(self) -> crate::pose::Kind {
        match self {
            Knight::A => crate::pose::Kind::KnightA,
            Knight::B => crate::pose::Kind::KnightB,
        }
    }

    pub fn named(s: &str) -> Option<Knight> {
        match s {
            "a" => Some(Knight::A),
            "b" => Some(Knight::B),
            _ => None,
        }
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
}

impl Default for Kept {
    fn default() -> Kept {
        Kept {
            look_pct: 100,
            invert_pitch: false,
            fov: crate::world::FOV_DEFAULT,
            aim_help: AimHelp::None,
            knight: Knight::A,
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
pub const GAME_KEYS: [&str; 2] = ["knight", "settings"];
pub const SETTING_KEYS: [&str; 4] = ["aim_help", "fov", "invert_pitch", "look_pct"];
pub const TOP_KEYS: [&str; 4] = ["checksum", "format", "game", "version"];

/// The `game` object of a save.
pub fn game_value(k: &Kept) -> Value {
    json!({
        "knight": k.knight.id(),
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
    if m.get("version").and_then(|v| v.as_u64()) != Some(VERSION) {
        return Err(Refusal::Version);
    }
    keys_are(m, &TOP_KEYS, "save")?;
    let game = &m["game"];
    let sum = m["checksum"].as_str().ok_or_else(|| bad("checksum"))?;
    if checksum(&game.to_string()) != sum {
        return Err(Refusal::Checksum);
    }
    let g = object(game, "game")?;
    keys_are(g, &GAME_KEYS, "game")?;
    let knight = g["knight"]
        .as_str()
        .and_then(Knight::named)
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
    })
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
        }
    }

    #[test]
    fn a_save_reads_back_as_it_was_written() {
        for k in [Kept::default(), odd()] {
            assert_eq!(read(&text(&k)), Ok(k));
        }
    }

    /// The text by hand: the game object's compact text with sorted keys,
    /// and nothing in it but the settings and the knight.
    #[test]
    fn a_save_holds_the_settings_and_the_knight_and_nothing_else() {
        let t = text(&odd());
        let v: Value = serde_json::from_str(&t).unwrap();
        assert_eq!(v["format"], "chumworld-arena-save");
        assert_eq!(v["version"], 1);
        let compact = r#"{"knight":"b","settings":{"aim_help":"soft_pull","fov":110,"invert_pitch":true,"look_pct":140}}"#;
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
        let old = good.replace("\"version\": 1", "\"version\": 0");
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
}
