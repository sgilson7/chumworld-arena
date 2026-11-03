//! The door into the core. The shim holds one `game::Game` and forwards the
//! page's calls; everything the page draws in HTML comes from `ui`, a view
//! model, and everything it sends back goes through `act(name, value)`,
//! answered `{"ok": <ui>}` or `{"error": "..."}` with a message the page
//! shows as it is. (After Peter's Mandible, by way of CHUMWORLD.)
//!
//! The view model names the screen, its panels (heading, text, rows of key
//! and meaning, buttons with the action each sends), the lines over the
//! duel, each fighter's health as a bar with its words, and the glossary's
//! definitions with their numbers filled in by the core. The page builds
//! the DOM from it without knowing what a panel means. A button marked
//! `engage` also asks the browser for the pointer, which only a click may do.

use serde::Serialize;

use crate::game::{Game, Screen};
use crate::input;
use crate::look;
use crate::prim::{self, MeshId};
use crate::render;
use crate::world::{AimHelp, HudItem};

pub fn ok<T: Serialize>(value: &T) -> String {
    serde_json::json!({ "ok": value }).to_string()
}

pub fn error(message: String) -> String {
    serde_json::json!({ "error": message }).to_string()
}

/// One button: the action name and value the page sends back through `act`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Action {
    pub name: String,
    pub value: String,
    pub label: String,
    pub enabled: bool,
    /// also take the pointer (pointer lock) on this click
    pub engage: bool,
    /// M2: ask the reader for a file first, and send its text as the value
    pub file: bool,
    /// M5: send the text of this field (by its id) as the value
    #[serde(skip_serializing_if = "String::is_empty")]
    pub from: String,
}

/// M5: a box of text on a panel: a code to copy (`fixed`) or one to paste
/// or type in. The page sets `data-ready` on a fixed box that holds text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Field {
    pub id: String,
    pub label: String,
    pub value: String,
    pub fixed: bool,
    /// one line (a room code) or several (a pasted code)
    pub line: bool,
}

/// M5: online play as the page's transport and checks need it: the
/// commands for the transport (`versus::Cmd`), the status in words, and
/// this browser's own checksums (`tick:sum`, newest last).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OnlineView {
    pub cmds: Vec<crate::versus::Cmd>,
    pub status: String,
    pub sums: String,
}

/// M2: one setting as the settings screen shows it: what it is, its value
/// in words, and the buttons that change it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Setting {
    pub id: String,
    pub label: String,
    pub value: String,
    pub actions: Vec<Action>,
}

/// M2: what the page does with the save: keep the save's text in the
/// browser when `seq` changes, offer it as a download named `file_name`
/// when `export` changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SaveView {
    pub seq: u32,
    pub export: u32,
    pub file_name: String,
}

/// One row of a panel's table: a key and what it does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Row {
    pub key: String,
    pub does: String,
}

/// M6: one rung of the ladder as its rank card shows it: the rank's letter,
/// the opponent, whether it is locked, open or won (`state`, and in words
/// `text`), its marks of the most it can hold (`marks_text` says them in
/// words, besides the page's drawn marks), and the button that opens its
/// dossier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Card {
    pub id: String,
    pub rank: String,
    pub name: String,
    pub state: String,
    pub text: String,
    pub marks: u8,
    pub marks_most: u8,
    pub marks_text: String,
    pub action: Action,
}

/// T11 (CA-Q18: A): one region of the campaign's map: its name, whether it
/// is open (and if not, what opens it, in words), and its battles' cards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RegionView {
    pub id: String,
    pub heading: String,
    pub open: bool,
    pub text: String,
    pub cards: Vec<Card>,
}

/// One block of the screen. Empty fields are left out of the JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Panel {
    pub id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub heading: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub text: String,
    /// M2: a second, smaller line (a disabled button's reason, the save)
    #[serde(skip_serializing_if = "String::is_empty")]
    pub note: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub settings: Vec<Setting>,
    /// M5: boxes of text, after the text and before the buttons
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<Field>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub rows_heading: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<Row>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<Action>,
    /// M6: the ladder's rank cards, after the text
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cards: Vec<Card>,
    /// T11: the campaign's map, region by region, each with its cards
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<RegionView>,
}

/// A term and its definition, for the page's hover hints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hint {
    pub term: String,
    pub definition: String,
}

/// One fighter's health as the page shows it: who, the words ("72 / 100"),
/// and the share left in hundredths of a percent, which the page sets as
/// the bar's width. The words carry it besides the bar's length (BRIEF.md,
/// Accessibility).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Bar {
    pub id: String,
    pub who: String,
    pub label: String,
    pub text: String,
    /// the share left, 0 to 10 000
    pub share: i32,
    /// M4: the fighter's stamina, while stamina is on (`chivalry.rs`)
    pub stamina: Option<Meter>,
}

/// M4: a fighter's stamina as the page shows it: the label, the words
/// ("62 / 100", and "Winded" at none), the share left in hundredths of a
/// percent, and whether it is winded, which the page shows as a hatched
/// bar besides the word (BRIEF.md, Accessibility).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Meter {
    pub label: String,
    pub text: String,
    pub share: i32,
    pub winded: bool,
}

/// Everything the page draws in HTML.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Ui {
    pub screen: String,
    pub title: String,
    pub panels: Vec<Panel>,
    pub hud: Vec<HudItem>,
    /// the player's health, then the foe's, in the duel only
    pub bars: Vec<Bar>,
    pub hints: Vec<Hint>,
    /// the text the page shows if it cannot draw (no WebGL 2)
    pub no_webgl: String,
    /// whether the game wants the pointer held (pointer lock): the page lets
    /// go of it when this is false, and takes it again on a click when true
    pub capture: bool,
    pub save: SaveView,
    /// M5: online play
    pub online: OnlineView,
    /// T11: watch mode's fight and trees (`watch::View`), on its screen only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watch: Option<crate::watch::View>,
    pub version: u32,
}

fn controls(g: &Game) -> (String, Vec<Row>) {
    let s = &g.strings;
    let row = |k: &str, d: &str| Row {
        key: s.get(k),
        does: s.get(d),
    };
    (
        s.get("controls"),
        // M1: the first-person controls (CA-Q6: A)
        vec![
            row("key_move", "control_move"),
            row("key_run", "control_run"),
            row("key_look", "control_look"),
            row("key_pause", "control_pause"),
            row("key_draw", "control_draw"),
            row("key_hand", "control_hand"),
            row("key_swing", "control_swing"),
            row("key_reach", "control_reach"),
            row("key_dodge", "control_dodge"),
            // M4: the feint and the kick
            row("key_feint", "control_feint"),
            row("key_kick", "control_kick"),
        ],
    )
}

fn button(g: &Game, name: &str, value: &str, label: &str, engage: bool) -> Action {
    Action {
        name: name.into(),
        value: value.into(),
        label: g.strings.get(label),
        enabled: true,
        engage,
        file: false,
        from: String::new(),
    }
}

fn panel(id: &str) -> Panel {
    Panel {
        id: id.into(),
        heading: String::new(),
        text: String::new(),
        note: String::new(),
        settings: Vec::new(),
        fields: Vec::new(),
        rows_heading: String::new(),
        rows: Vec::new(),
        actions: Vec::new(),
        cards: Vec::new(),
        regions: Vec::new(),
    }
}

/// A definition with each `{placeholder}` replaced by the core's number.
pub fn fill(definition: &str, values: &[(&str, String)]) -> String {
    let mut out = definition.to_string();
    for (k, v) in values {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

/// A fighter's bar: its health now (never below none) of its most.
fn bar(g: &Game, id: usize, who: String) -> Bar {
    let h = crate::player::fighter(&g.world, id).health;
    let now = h.hp.max(0);
    let most = h.most.max(1);
    let text = fill(
        &g.strings.get("health_of"),
        &[("now", now.to_string()), ("most", most.to_string())],
    );
    Bar {
        id: if id == 0 { "you".into() } else { format!("foe{id}") },
        who,
        label: g.strings.get("health"),
        text,
        share: (now as i64 * 10_000 / most as i64).min(10_000) as i32,
        stamina: stamina(g, id),
    }
}

/// M4: a fighter's stamina meter, while the layer is on.
fn stamina(g: &Game, id: usize) -> Option<Meter> {
    if !crate::chivalry::on(&g.world) {
        return None;
    }
    let c = crate::player::fighter(&g.world, id).chiv;
    let most = crate::chivalry::spec().stamina;
    let mut text = fill(
        &g.strings.get("stamina_of"),
        &[("now", c.points().to_string()), ("most", most.to_string())],
    );
    if c.winded {
        text = fill(&g.strings.get("stamina_winded"), &[("of", text)]);
    }
    Some(Meter {
        label: g.strings.get("stamina"),
        text,
        share: (c.stamina as i64 * 10_000 / (most as i64 * crate::chivalry::PART as i64)).min(10_000) as i32,
        winded: c.winded,
    })
}

/// Each fighter's bar: the player's, then each foe's, named by its kind.
pub fn bars(g: &Game) -> Vec<Bar> {
    let s = &g.strings;
    (0..crate::player::count(&g.world))
        .map(|id| {
            let who = if id == 0 {
                s.get("who_you")
            } else {
                // M2: by the pilot's entry; M3: a foe in a knight's plate by
                // the knight it wears ("The ivory crown"), whatever pilot
                // thinks for it
                let f = &g.world.foes[id - 1];
                if crate::plate::armoured(f.fighter.kind) {
                    s.get(&format!("who_{}", crate::pilot::kind_name(f.fighter.kind)))
                } else {
                    s.get(&format!("who_{}", f.brain.key))
                }
            };
            bar(g, id, who)
        })
        .collect()
}

/// The settings screen's rows: the look speed, the pitch, the field of
/// view and the aim help, each with its value in words.
fn settings(g: &Game) -> Vec<Setting> {
    let s = &g.strings;
    let k = &g.kept;
    let steps = |name: &str| {
        vec![
            button(g, name, "-", "setting_less", false),
            button(g, name, "+", "setting_more", false),
        ]
    };
    let toggle = |name: &str| vec![button(g, name, "toggle", "setting_change", false)];
    let on_off = |b: bool| s.get(if b { "setting_on" } else { "setting_off" });
    vec![
        Setting {
            id: "look_speed".into(),
            label: s.get("setting_look_speed"),
            value: fill(&s.get("setting_percent"), &[("n", k.look_pct.to_string())]),
            actions: steps("settings.look_speed"),
        },
        Setting {
            id: "invert_pitch".into(),
            label: s.get("setting_invert_pitch"),
            value: on_off(k.invert_pitch),
            actions: toggle("settings.invert_pitch"),
        },
        Setting {
            id: "fov".into(),
            label: s.get("setting_fov"),
            value: fill(&s.get("setting_degrees"), &[("n", k.fov.to_string())]),
            actions: steps("settings.fov"),
        },
        Setting {
            id: "aim_help".into(),
            label: s.get("setting_aim_help"),
            value: s.get(match k.aim_help {
                AimHelp::None => "aim_help_none",
                AimHelp::SoftPull => "aim_help_soft_pull",
            }),
            actions: toggle("settings.aim_help"),
        },
        // M6: the look, the knights' or a won rung's opponent's (CA-Q9: A)
        Setting {
            id: "look".into(),
            label: s.get("setting_look"),
            value: look_name(g, k.knight),
            actions: toggle("settings.look"),
        },
        // T11 (CA-Q19: C): the weapon, the longsword or one the campaign
        // has given
        Setting {
            id: "weapon".into(),
            label: s.get("setting_weapon"),
            value: s.get(&format!("weapon_{}", crate::watch::held_key(k.weapon))),
            actions: toggle("settings.weapon"),
        },
    ]
}

/// M6: a look's name: the knight's or the opponent's working label.
pub fn look_name(g: &Game, k: crate::save::Knight) -> String {
    g.strings
        .get(&format!("who_{}", crate::pilot::kind_name(k.dress())))
}

/// M6: what the round and result lines call the foe: the knight in
/// Practice and online, a rung's opponent on the ladder ("squire").
fn foe_name(g: &Game) -> String {
    let key = if g.climbing && g.versus.is_none() {
        crate::ladder::RUNGS[g.rung].key
    } else {
        "knight"
    };
    g.strings.get(&format!("short_{key}")).to_lowercase()
}

/// The name at the head of a sentence: its first letter a capital.
fn capital(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}

/// M6: where an opponent wears plate, in words: every part, or the parts
/// one by one, a pair as both (`plate::covered`, the one table).
fn plate_words(g: &Game, kind: crate::pose::Kind) -> String {
    use crate::wounds::{bit, FOREARM, HEAD, SHIN, THIGH, TORSO, UPPER_ARM};
    let s = &g.strings;
    let c = crate::plate::covered(kind);
    if c == (1 << crate::blade::PARTS) - 1 {
        return s.get("plate_every");
    }
    let on = |p: usize| c & bit(p) != 0;
    let mut words = Vec::new();
    if on(HEAD) {
        words.push(s.get("part_head"));
    }
    if on(TORSO) {
        words.push(s.get("part_torso"));
    }
    for (pair, key) in [
        (UPPER_ARM, "part_upper_arm"),
        (FOREARM, "part_forearm"),
        (THIGH, "part_thigh"),
        (SHIN, "part_shin"),
    ] {
        let part = [("part", s.get(key))];
        let [l, r] = [on(pair[crate::pose::LEFT]), on(pair[crate::pose::RIGHT])];
        match (l, r) {
            (true, true) => words.push(fill(&s.get("plate_both"), &part)),
            (true, false) => words.push(fill(&s.get("plate_left"), &part)),
            (false, true) => words.push(fill(&s.get("plate_right"), &part)),
            _ => {}
        }
    }
    capital(&words.join(", "))
}

/// M6: a rung's state in words, for its card and its dossier.
fn rung_text(g: &Game, i: usize) -> (String, String) {
    use crate::ladder::State;
    let s = &g.strings;
    match g.kept.ladder.state(i) {
        State::Locked => (
            "locked".into(),
            fill(&s.get("rung_locked"), &[("needs", needs_text(g, i))]),
        ),
        State::Open => ("open".into(), s.get("rung_open")),
        State::Won(m) => ("won".into(), s.get(&format!("rung_won_{m}"))),
    }
}

/// A name that begins "The " as it reads mid-sentence: "the Marches".
fn mid(name: &str) -> String {
    match name.strip_prefix("The ") {
        Some(rest) => format!("the {rest}"),
        None => name.to_string(),
    }
}

/// T11: what battle `i` still needs, in words: its region's gate, then
/// each of its lines not yet met (CA-Q21: A).
pub fn needs_text(g: &Game, i: usize) -> String {
    use crate::ladder::{Need, REGIONS, RUNGS};
    let s = &g.strings;
    let p = &g.kept.ladder;
    let who = |j: usize| s.get(&format!("who_{}", RUNGS[j].key)).to_lowercase();
    let mut out = Vec::new();
    let r = RUNGS[i].region;
    if !p.region_open(r) {
        out.push(fill(
            &s.get("need_gate"),
            &[
                ("n", REGIONS[r].gate.to_string()),
                ("region", mid(&s.get(&format!("region_{}", REGIONS[r - 1].key)))),
            ],
        ));
    }
    for &n in RUNGS[i].needs {
        if p.meets(n) {
            continue;
        }
        out.push(match n {
            Need::Won(j) => fill(&s.get("need_won"), &[("name", who(j))]),
            Need::Clean(j) => fill(&s.get("need_clean"), &[("name", who(j))]),
            Need::With(j, h) => fill(
                &s.get("need_with"),
                &[
                    ("name", who(j)),
                    (
                        "weapon",
                        s.get(&format!("weapon_{}", crate::watch::held_key(h)))
                            .to_lowercase(),
                    ),
                ],
            ),
            Need::AnyOf(list, k) => fill(
                &s.get("need_any"),
                &[
                    ("n", k.to_string()),
                    (
                        "names",
                        list.iter()
                            .map(|&j| who(j))
                            .collect::<Vec<_>>()
                            .join(&s.get("need_or")),
                    ),
                ],
            ),
        });
    }
    out.join(&s.get("need_and"))
}

/// T11: the campaign's map: each region, open or what opens it, and its
/// battles' cards.
fn regions(g: &Game) -> Vec<RegionView> {
    use crate::ladder::REGIONS;
    let s = &g.strings;
    let all = cards(g);
    REGIONS
        .iter()
        .enumerate()
        .map(|(r, reg)| {
            let open = g.kept.ladder.region_open(r);
            RegionView {
                id: reg.key.into(),
                heading: s.get(&format!("region_{}", reg.key)),
                open,
                text: if open {
                    s.get("region_open")
                } else {
                    fill(
                        &s.get("region_shut"),
                        &[
                            ("n", reg.gate.to_string()),
                            ("region", mid(&s.get(&format!("region_{}", REGIONS[r - 1].key)))),
                            ("have", g.kept.ladder.region_marks(r - 1).to_string()),
                        ],
                    )
                },
                cards: crate::ladder::RUNGS
                    .iter()
                    .zip(all.iter())
                    .filter(|(b, _)| b.region == r)
                    .map(|(_, c)| c.clone())
                    .collect(),
            }
        })
        .collect()
}

/// M6: the ladder's rank cards, foot first (T11: every battle's, in order).
fn cards(g: &Game) -> Vec<Card> {
    let s = &g.strings;
    crate::ladder::RUNGS
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let (state, text) = rung_text(g, i);
            let marks = g.kept.ladder.marks[i];
            let most = crate::ladder::MARKS_MOST;
            let name = s.get(&format!("who_{}", r.key));
            Card {
                id: r.key.into(),
                rank: r.rank.into(),
                name: name.clone(),
                state,
                text,
                marks,
                marks_most: most,
                marks_text: fill(
                    &s.get("marks_of"),
                    &[("n", marks.to_string()), ("most", most.to_string())],
                ),
                action: Action {
                    label: if r.rank.is_empty() {
                        name.clone()
                    } else {
                        fill(&s.get("rank"), &[("rank", r.rank.to_string())])
                    },
                    ..button(g, "ladder.rung", r.key, "rank", false)
                },
            }
        })
        .collect()
}

/// M6: one rung's dossier: who, how it fights, its rank, what it wears,
/// where its plate is, its marks, and what winning it gives; Fight when
/// the rung is open.
fn dossier(g: &Game) -> Panel {
    let s = &g.strings;
    let i = g.rung;
    let r = crate::ladder::RUNGS[i];
    let (_, state) = rung_text(g, i);
    let row = |k: &str, d: String| Row {
        key: s.get(k),
        does: d,
    };
    let open = g.kept.ladder.open(i);
    Panel {
        heading: s.get(&format!("who_{}", r.key)),
        text: s.get(&format!("dossier_{}", r.key)),
        note: if open { String::new() } else { state.clone() },
        rows_heading: if r.rank.is_empty() {
            s.get(&format!("region_{}", crate::ladder::REGIONS[r.region].key))
        } else {
            fill(&s.get("rank"), &[("rank", r.rank.to_string())])
        },
        rows: {
            let carried = r.held.unwrap_or(crate::pose::Held::Longsword);
            let weapon = s.get(&format!("weapon_{}", crate::watch::held_key(carried)));
            vec![
                row("dossier_wears", s.get(&format!("wears_{}", r.key))),
                row("dossier_weapon", weapon.clone()),
                row("dossier_plate", plate_words(g, r.kind)),
                row("dossier_marks", state),
                row(
                    "dossier_reward",
                    if r.held.is_some() {
                        fill(
                            &s.get("dossier_reward_weapon"),
                            &[("weapon", weapon.to_lowercase())],
                        )
                    } else {
                        s.get("dossier_reward_text")
                    },
                ),
            ]
        },
        actions: vec![
            Action {
                enabled: open,
                ..button(g, "ladder.fight", r.key, "fight", true)
            },
            button(g, "screen.show", "ladder", "back_to_ladder", false),
        ],
        ..panel("dossier")
    }
}

/// The score in words: "You 1 · Knight 0" (M6: "You 1 · Squire 0").
fn score(g: &Game, r: &crate::rounds::Rounds) -> String {
    fill(
        &g.strings.get("round_score"),
        &[
            ("you", r.wins[0].to_string()),
            ("foe", r.wins[1].to_string()),
            ("Name", capital(&foe_name(g))),
        ],
    )
}

/// How a round ended, in words: who fell and to what.
fn fall_text(g: &Game, r: &crate::rounds::Rounds) -> (String, String) {
    use crate::rounds::{Blow, Fall};
    let s = &g.strings;
    let name = foe_name(g);
    let n = [("n", r.round.to_string()), ("name", name.clone())];
    let named = |key: &str| fill(&s.get(key), &[("name", name.clone())]);
    match r.fall {
        Some(Fall::One { who, blow }) => {
            let heading = fill(&s.get(if who == 1 { "round_to_you" } else { "round_to_foe" }), &n);
            let text = match blow {
                Blow::Sand => named(if who == 1 {
                    "fell_foe_sand"
                } else {
                    "fell_you_sand"
                }),
                Blow::Blade => named(if who == 1 {
                    "fell_foe_blade"
                } else {
                    "fell_you_blade"
                }),
                Blow::Cut(k) => fill(
                    &s.get(if who == 1 { "fell_foe_cut" } else { "fell_you_cut" }),
                    &[("blow", s.get(k.key())), ("name", name.clone())],
                ),
            };
            (heading, text)
        }
        _ => (fill(&s.get("round_again"), &n), named("fell_both")),
    }
}

/// The view model for the game as it stands.
pub fn ui(g: &Game) -> Ui {
    let s = &g.strings;
    let (rows_heading, rows) = controls(g);
    let mut panels = Vec::new();
    let rounds = g.world.rounds.as_ref();
    match g.screen {
        Screen::Title => panels.push(Panel {
            heading: s.get("title"),
            text: s.get("title_text"),
            actions: vec![button(g, "screen.show", "menu", "continue", false)],
            ..panel("title")
        }),
        Screen::Menu => {
            panels.push(Panel {
                heading: s.get("title"),
                text: s.get("menu_text"),
                note: s.get("online_menu_note"),
                actions: vec![
                    button(g, "screen.show", "duel", "practice", true),
                    button(g, "screen.show", "ladder", "ladder", false),
                    button(g, "screen.show", "watch", "watch", false),
                    button(g, "online.host", "room", "host", false),
                    button(g, "online.join", "room", "join", false),
                    button(g, "screen.show", "paste", "pasted_codes", false),
                    button(g, "screen.show", "settings", "settings", false),
                ],
                ..panel("menu")
            });
            // the controls beside the menu, read before the first duel
            panels.push(Panel {
                rows_heading,
                rows,
                ..panel("controls")
            });
            if !g.notice.is_empty() {
                panels.push(Panel {
                    text: g.notice.clone(),
                    ..panel("notice")
                });
            }
        }
        Screen::Settings => panels.push(Panel {
            heading: s.get("settings"),
            note: g.notice.clone(),
            settings: settings(g),
            actions: vec![
                button(g, "save.download", "", "save_download", false),
                Action {
                    file: true,
                    ..button(g, "save.file", "", "save_load", false)
                },
                button(g, "screen.show", "back", "back", false),
            ],
            ..panel("settings")
        }),
        Screen::Paused => panels.push(Panel {
            heading: s.get("paused"),
            text: s.get("swing_how"),
            actions: vec![
                button(g, "screen.show", "duel", "resume", true),
                button(g, "screen.show", "settings", "settings", false),
                // M6: a ladder duel is left for the ladder
                if g.climbing {
                    button(g, "screen.show", "ladder", "leave_duel", false)
                } else {
                    button(g, "screen.show", "menu", "leave_duel", false)
                },
            ],
            rows_heading,
            rows,
            ..panel("paused")
        }),
        Screen::Round => {
            if let Some(r) = rounds {
                let (heading, text) = fall_text(g, r);
                panels.push(Panel {
                    heading,
                    text,
                    note: score(g, r),
                    ..panel("round")
                });
            }
        }
        Screen::Paste => panels.push(Panel {
            heading: s.get("pasted_codes"),
            text: s.get("paste_text"),
            actions: vec![
                button(g, "online.host", "code", "paste_host", false),
                button(g, "online.join", "code", "paste_join", false),
                button(g, "screen.show", "menu", "back_to_menu", false),
            ],
            ..panel("paste")
        }),
        Screen::Online => {
            if let Some(v) = g.versus.as_ref() {
                panels.push(online_panel(g, v));
                panels.push(Panel {
                    text: status_text(g, v),
                    ..panel("online-status")
                });
            }
        }
        Screen::Result => {
            if let Some(r) = rounds {
                let won = r.winner() == Some(0);
                // M5: online, a rematch is both players' choice
                let rematch = match g.versus.as_ref() {
                    Some(_) => button(g, "online.rematch", "", "rematch", true),
                    None => button(g, "screen.show", "duel", "rematch", true),
                };
                let waiting = g.versus.as_ref().is_some_and(|v| v.rematch);
                let name = [
                    ("you", r.wins[0].to_string()),
                    ("foe", r.wins[1].to_string()),
                    ("name", foe_name(g)),
                ];
                let mut actions = vec![rematch];
                if g.climbing {
                    actions.push(button(g, "screen.show", "ladder", "back_to_ladder", false));
                }
                actions.push(button(g, "screen.show", "menu", "back_to_menu", false));
                panels.push(Panel {
                    heading: fill(&s.get(if won { "result_won" } else { "result_lost" }), &name),
                    text: fill(
                        &s.get(if won {
                            "result_won_text"
                        } else {
                            "result_lost_text"
                        }),
                        &name,
                    ),
                    note: if waiting {
                        s.get("online.rematch_wait")
                    } else if g.climbing {
                        earned_text(g)
                    } else {
                        String::new()
                    },
                    actions,
                    ..panel("result")
                });
            }
        }
        Screen::Ladder => panels.push(Panel {
            heading: s.get("ladder_heading"),
            text: fill(&s.get("ladder_text"), &crate::ladder::values()),
            regions: regions(g),
            actions: vec![button(g, "screen.show", "menu", "back_to_menu", false)],
            ..panel("ladder")
        }),
        Screen::Dossier => panels.push(dossier(g)),
        Screen::Watch => panels.push(watch_panel(g)),
        Screen::Duel => {}
    }
    let mut hud = vec![HudItem {
        id: "hint".into(),
        text: s.get("hint_pause"),
        shown: g.hint_shown(),
    }];
    if let Some(r) = rounds {
        // the round, the score and the time before sudden death
        let left = crate::rounds::seconds_left(r);
        let clock = format!("{}:{:02}", left / 60, left % 60);
        hud.push(HudItem {
            id: "round".into(),
            text: fill(
                &s.get("round_line"),
                &[
                    ("n", r.round.to_string()),
                    ("score", score(g, r)),
                    ("clock", clock),
                ],
            ),
            shown: g.screen.plays(),
        });
        hud.push(HudItem {
            id: "sudden".into(),
            text: s.get("sudden_death"),
            shown: g.screen == Screen::Duel && r.sudden(),
        });
    }
    if let Some(v) = g.versus.as_ref() {
        // M5: the online status over the duel: shown at its start and
        // whenever it is not plain play
        hud.push(HudItem {
            id: "online".into(),
            text: status_text(g, v),
            shown: g.screen.plays()
                && (v.status() != crate::online::PLAYING || g.play_ticks < crate::game::HINT_TICKS),
        });
    }
    g.world.hud(s, &mut hud);
    let values = g.glossary_values();
    Ui {
        screen: g.screen.name().into(),
        title: s.get("title"),
        panels,
        hud,
        bars: if g.in_duel() { bars(g) } else { Vec::new() },
        hints: g
            .glossary
            .iter()
            .map(|e| Hint {
                term: e.term.clone(),
                definition: fill(&e.definition, &values),
            })
            .collect(),
        no_webgl: s.get("no_webgl"),
        capture: matches!(g.screen, Screen::Duel | Screen::Round),
        save: SaveView {
            seq: g.save_seq,
            export: g.save_export,
            file_name: s.get("save_file_name"),
        },
        online: OnlineView {
            cmds: g.online_cmds.clone(),
            status: g.versus.as_ref().map(|v| status_text(g, v)).unwrap_or_default(),
            sums: g.versus.as_ref().map(|v| v.sums()).unwrap_or_default(),
        },
        watch: (g.screen == Screen::Watch).then(|| crate::watch::view(g)),
        version: g.ui_version,
    }
}

/// T11: watch mode's controls: the two pickers, start again, pause or
/// resume, a tick at a time, the four speeds, whose tree is shown, the
/// overlay, and back to the menu.
fn watch_panel(g: &Game) -> Panel {
    let s = &g.strings;
    let v = &g.watching;
    let named = |name: &str, value: &str, label: String| Action {
        label,
        ..button(g, name, value, "watch", false)
    };
    let names = crate::watch::names(g);
    let mut actions = vec![
        named(
            "watch.left",
            "",
            fill(&s.get("watch_left"), &[("name", names[0].clone())]),
        ),
        named(
            "watch.right",
            "",
            fill(&s.get("watch_right"), &[("name", names[1].clone())]),
        ),
        named(
            "watch.left_weapon",
            "",
            fill(
                &s.get("watch_left_weapon"),
                &[("weapon", crate::watch::weapon_name(g, 0))],
            ),
        ),
        named(
            "watch.right_weapon",
            "",
            fill(
                &s.get("watch_right_weapon"),
                &[("weapon", crate::watch::weapon_name(g, 1))],
            ),
        ),
        button(g, "watch.start", "", "watch_start", false),
        button(
            g,
            "watch.pause",
            "",
            if v.paused { "watch_resume" } else { "watch_pause" },
            false,
        ),
        button(g, "watch.step", "", "watch_step", false),
    ];
    for sp in crate::watch::SPEEDS {
        actions.push(button(
            g,
            "watch.speed",
            &sp.to_string(),
            &format!("speed_{sp}"),
            false,
        ));
    }
    for (k, side) in ["left", "right"].iter().enumerate() {
        let mut b = named(
            "watch.show",
            side,
            fill(&s.get("watch_show"), &[("name", names[k].clone())]),
        );
        b.enabled = v.show != k;
        actions.push(b);
    }
    actions.push(button(
        g,
        "watch.overlay",
        if v.overlay { "off" } else { "on" },
        if v.overlay {
            "watch_overlay_off"
        } else {
            "watch_overlay_on"
        },
        false,
    ));
    actions.push(button(g, "screen.show", "menu", "back_to_menu", false));
    Panel {
        heading: s.get("watch_heading"),
        text: s.get("watch_text"),
        actions,
        ..panel("watch")
    }
}

/// M6: what a ladder duel's result gave, in words: the rung's marks, and
/// the look it gave when it was won the first time.
fn earned_text(g: &Game) -> String {
    let s = &g.strings;
    let r = crate::ladder::RUNGS[g.rung];
    // a ladder rung by its rank ("Rank F"), a new battle by its opponent
    let what = if r.rank.is_empty() {
        s.get(&format!("who_{}", r.key))
    } else {
        fill(&s.get("rank"), &[("rank", r.rank.to_string())])
    };
    let what = [("what", what)];
    match g.earned {
        Some(0) | None => fill(&s.get("earned_none"), &what),
        Some(m) => {
            let mut t = fill(&s.get(&format!("earned_{m}")), &what);
            let look = crate::save::Knight::Rung(g.rung as u8);
            t.push(' ');
            t.push_str(&fill(&s.get("earned_look"), &[("name", look_name(g, look))]));
            // T11: and its weapon, if it carries one
            if let Some(h) = r.held {
                t.push(' ');
                t.push_str(&fill(
                    &s.get("earned_weapon"),
                    &[(
                        "weapon",
                        s.get(&format!("weapon_{}", crate::watch::held_key(h)))
                            .to_lowercase(),
                    )],
                ));
            }
            t
        }
    }
}

/// M5: the online status in words, its numbers filled in by the core.
fn status_text(g: &Game, v: &crate::versus::Versus) -> String {
    fill(
        &g.strings.get(v.status()),
        &[("seconds", (crate::versus::TIMEOUT_US / 1_000_000).to_string())],
    )
}

/// M5: a box of text.
fn field(g: &Game, id: &str, label: &str, value: &str, fixed: bool, line: bool) -> Field {
    Field {
        id: id.into(),
        label: g.strings.get(label),
        value: value.into(),
        fixed,
        line,
    }
}

/// M5: a button that sends a field's text.
fn sends(g: &Game, name: &str, from: &str, label: &str) -> Action {
    Action {
        from: from.into(),
        ..button(g, name, "", label, false)
    }
}

/// M5: the online screen: hosting or joining, by room code or pasted
/// codes, or the duel ended and why.
fn online_panel(g: &Game, v: &crate::versus::Versus) -> Panel {
    use crate::versus::{Role, Way};
    let s = &g.strings;
    let back = button(g, "screen.show", "menu", "back_to_menu", false);
    if v.ended() {
        return Panel {
            heading: s.get("online.ended"),
            text: status_text(g, v),
            actions: vec![back],
            ..panel("online")
        };
    }
    let instead = |role: &str| button(g, role, "code", "paste_instead", false);
    match (v.role, v.way) {
        (Role::Host, Way::Room) => Panel {
            heading: s.get("host"),
            text: s.get("host_room_text"),
            fields: {
                let mut f = vec![field(g, "room-out", "room_code", &v.code, true, true)];
                // a join link, once the page has said where it lives
                if !g.address.is_empty() {
                    let link = format!("{}?join={}", g.address, v.code);
                    f.push(field(g, "room-link", "join_link", &link, true, true));
                }
                f
            },
            actions: vec![instead("online.host"), back],
            ..panel("online")
        },
        (Role::Guest, Way::Room) if v.code.is_empty() => Panel {
            heading: s.get("join"),
            text: s.get(if g.invited.is_empty() {
                "join_room_text"
            } else {
                "join_link_text"
            }),
            fields: vec![field(g, "room-in", "room_code", &g.invited, false, true)],
            actions: vec![
                sends(g, "online.room", "room-in", "join"),
                instead("online.join"),
                back,
            ],
            ..panel("online")
        },
        (Role::Guest, Way::Room) => Panel {
            heading: s.get("join"),
            text: s.get("join_room_text"),
            fields: vec![field(g, "room-out", "room_code", &v.code, true, true)],
            actions: vec![instead("online.join"), back],
            ..panel("online")
        },
        (Role::Host, Way::Paste) => Panel {
            heading: s.get("paste_host"),
            text: s.get("paste_host_text"),
            fields: vec![
                field(g, "offer-out", "invitation", &v.offer, true, false),
                field(g, "answer-in", "reply", "", false, false),
            ],
            actions: vec![sends(g, "online.accept", "answer-in", "accept_reply"), back],
            ..panel("online")
        },
        (Role::Guest, Way::Paste) => Panel {
            heading: s.get("paste_join"),
            text: s.get("paste_join_text"),
            fields: vec![
                field(g, "offer-in", "invitation", "", false, false),
                field(g, "answer-out", "reply", &v.answer, true, false),
            ],
            actions: vec![sends(g, "online.reply", "offer-in", "make_reply"), back],
            ..panel("online")
        },
    }
}

/// `act` as the page calls it: the new view model, or the error.
pub fn act(g: &mut Game, name: &str, value: &str) -> String {
    match g.act(name, value) {
        Ok(()) => ok(&ui(g)),
        Err(e) => error(e),
    }
}

/// Sizes and layout the page needs to upload and draw: every stride, offset,
/// bit, pass state and uniform name, so the page holds no layout of its own.
pub fn numbers() -> String {
    let b = render::BYTES_PER_VALUE;
    let vec4 = 4 * b;
    serde_json::json!({
        "actions": input::ACTIONS.iter().map(|(n, bit)| (n.to_string(), serde_json::json!(bit))).collect::<serde_json::Map<_, _>>(),
        "vertex": {
            "stride": prim::VERTEX_VALUES * b,
            "attributes": [["aPos", 3, 0], ["aNormal", 3, 3 * b], ["aColor", 3, 6 * b]],
        },
        "instance": {
            "stride": render::INSTANCE_VALUES * b,
            "attributes": [["iA", 4, 0], ["iB", 4, vec4], ["iC", 4, 2 * vec4], ["iD", 4, 3 * vec4]],
        },
        "draw_values": render::DRAW_VALUES,
        "buffers": { "static": render::STATIC_BUFFER, "dynamic": render::DYNAMIC_BUFFER },
        "passes": render::pass_states(),
        // the look's uniforms, then the sky's (`sky.rs`)
        "uniforms": look::UNIFORMS.iter().chain(crate::sky::UNIFORMS).map(|(n, call, len)| serde_json::json!([n, call, len, call.contains("Matrix")])).collect::<Vec<_>>(),
        // the view values: each layer's matrix by layer number, then the eye
        "view": { "layers": [[0, 16], [16, 32], [32, 48]], "eye": [48, 51] },
        // a bar's share is hundredths of a percent; the page divides by this
        // to set a width as a share of one
        "bar_whole": 10_000,
    })
    .to_string()
}

/// Every mesh the page uploads, by number, with its vertex count: the
/// primitives, then the ring's.
pub fn meshes(g: &Game) -> String {
    let list: Vec<serde_json::Value> = (0..g.meshes.count() as u32)
        .map(|id| {
            let n = g.meshes.vertices(MeshId(id)).len() / prim::VERTEX_VALUES;
            serde_json::json!({ "id": id, "vertices": n })
        })
        .collect();
    serde_json::Value::Array(list).to_string()
}

/// One mesh's vertex values by number; none for a number past the end.
pub fn mesh(g: &Game, id: u32) -> Vec<i32> {
    g.meshes.vertices(MeshId(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::tests::game;
    use crate::prim::Mesh;

    fn json(text: &str) -> serde_json::Value {
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn the_title_leads_to_the_menu_with_practice_host_join_and_settings() {
        let mut g = game();
        let u = json(&ok(&ui(&g)));
        assert_eq!(u["ok"]["screen"], "title");
        assert_eq!(u["ok"]["panels"][0]["heading"], "Chumworld Arena");
        assert_eq!(u["ok"]["panels"][0]["actions"][0]["value"], "menu");
        let v = json(&act(&mut g, "screen.show", "menu"));
        let p = &v["ok"]["panels"][0];
        assert_eq!(v["ok"]["screen"], "menu");
        let c = &v["ok"]["panels"][1];
        assert_eq!(c["rows"][4]["key"], "X");
        assert_eq!(c["rows"][4]["does"], "Draw or sheathe");
        let labels: Vec<&str> = p["actions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["label"].as_str().unwrap())
            .collect();
        assert_eq!(
            labels,
            [
                "Practice",
                "Single Player Campaign",
                "Watch",
                "Host",
                "Join",
                "Pasted codes",
                "Settings"
            ]
        );
        assert_eq!(p["actions"][0]["engage"], true);
        assert_eq!(p["actions"][0]["value"], "duel");
        // M6: the ladder, under Practice
        assert_eq!(p["actions"][1]["value"], "ladder");
        // T11: Watch, under the ladder
        assert_eq!(p["actions"][2]["value"], "watch");
        // M5: Host and Join open, by room code, and Pasted codes beside them
        assert_eq!(p["actions"][3]["enabled"], true);
        assert_eq!(p["actions"][3]["name"], "online.host");
        assert_eq!(p["actions"][4]["name"], "online.join");
        assert_eq!(p["actions"][5]["value"], "paste");
        // and nothing asks the transport for a link until one is chosen (CA-4)
        assert!(v["ok"]["online"]["cmds"].as_array().unwrap().is_empty());
        assert!(
            v["ok"]["bars"].as_array().unwrap().is_empty(),
            "no bars before the duel"
        );
        // the settings: six rows (M6: the look; T11: the weapon), each with its value in words
        let v = json(&act(&mut g, "screen.show", "settings"));
        let rows = v["ok"]["panels"][0]["settings"].as_array().unwrap();
        let values: Vec<&str> = rows.iter().map(|r| r["value"].as_str().unwrap()).collect();
        assert_eq!(
            values,
            ["100%", "Off", "105°", "None", "The black helm", "Longsword"]
        );
        let acts = v["ok"]["panels"][0]["actions"].as_array().unwrap();
        assert_eq!(acts[1]["file"], true, "loading a save asks for a file");
        assert_eq!(v["ok"]["save"]["file_name"], "chumworld-arena-save.json");
    }

    /// M2: the round's result names who fell and the blow, with the score.
    #[test]
    fn a_round_s_result_names_who_fell_and_the_blow() {
        use crate::rounds::{Blow, Fall, Kind2, Phase};
        let mut g = game();
        act(&mut g, "screen.show", "menu");
        act(&mut g, "screen.show", "duel");
        let r = g.world.rounds.as_mut().unwrap();
        r.phase = Phase::Beat(10);
        r.wins = [1, 0];
        r.fall = Some(Fall::One {
            who: 1,
            blow: Blow::Cut(Kind2::Chop),
        });
        g.screen = Screen::Round;
        let u = json(&ok(&ui(&g)));
        let p = &u["ok"]["panels"][0];
        assert_eq!(p["heading"], "Round 1 to you");
        assert_eq!(p["text"], "The knight fell to your chop.");
        assert_eq!(p["note"], "You 1 · Knight 0");
        let r = g.world.rounds.as_mut().unwrap();
        r.fall = Some(Fall::One {
            who: 0,
            blow: Blow::Sand,
        });
        let u = json(&ok(&ui(&g)));
        assert_eq!(u["ok"]["panels"][0]["heading"], "Round 1 to the knight");
        assert_eq!(u["ok"]["panels"][0]["text"], "You fell outside the inner ring.");
    }

    #[test]
    fn the_duel_shows_both_fighters_health_in_words_and_as_a_share() {
        let mut g = game();
        act(&mut g, "screen.show", "menu");
        let v = json(&act(&mut g, "screen.show", "duel"));
        let bars = v["ok"]["bars"].as_array().unwrap();
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0]["who"], "You");
        assert_eq!(bars[0]["text"], "100 / 100");
        assert_eq!(bars[0]["share"], 10_000);
        // M3: Practice's foe wears Knight B, and its bar reads that knight's
        // working label (data/strings.json `who_knight_b`)
        assert_eq!(bars[1]["who"], "The ivory crown");
        // the knight's 100 of 100 (data/pilots.json)
        assert_eq!(bars[1]["text"], "100 / 100");
        g.world.foes[0].fighter.health.hp = 30;
        let u = json(&ok(&ui(&g)));
        // 30 of 100 is 30%: 3 000 hundredths of a percent
        assert_eq!(u["ok"]["bars"][1]["share"], 3_000);
        assert_eq!(u["ok"]["bars"][1]["text"], "30 / 100");
        g.world.foes[0].fighter.health.hp = -12;
        let u = json(&ok(&ui(&g)));
        assert_eq!(u["ok"]["bars"][1]["text"], "0 / 100", "never below none");
        assert_eq!(u["ok"]["bars"][1]["share"], 0);
        // the round, the score and the clock over the duel
        let round = u["ok"]["hud"]
            .as_array()
            .unwrap()
            .iter()
            .find(|h| h["id"] == "round")
            .unwrap()
            .clone();
        assert_eq!(round["text"], "Round 1 · You 0 · Knight 0 · 1:30");
    }

    #[test]
    fn act_answers_with_the_new_screen_or_the_error() {
        let mut g = game();
        act(&mut g, "screen.show", "menu");
        let v = json(&act(&mut g, "screen.show", "duel"));
        assert_eq!(v["ok"]["screen"], "duel");
        assert!(
            v["ok"]["panels"].as_array().unwrap().is_empty(),
            "nothing covers the duel"
        );
        assert_eq!(v["ok"]["hud"][0]["shown"], true);
        assert_eq!(v["ok"]["capture"], true, "the duel holds the pointer");
        let v = json(&act(&mut g, "key.escape", ""));
        assert_eq!(v["ok"]["panels"][0]["heading"], "Paused");
        assert_eq!(v["ok"]["panels"][0]["actions"][0]["label"], "Resume");
        assert_eq!(v["ok"]["capture"], false, "a pause lets go of the pointer");
        let v = json(&act(&mut g, "screen.jump", "x"));
        assert_eq!(v["error"], "Unknown action \"screen.jump\".");
    }

    #[test]
    fn glossary_numbers_are_filled_by_the_core() {
        let g = game();
        let u = ui(&g);
        for h in &u.hints {
            assert!(
                !h.definition.contains('{'),
                "{}: a placeholder the core does not fill",
                h.term
            );
        }
        let arena = u
            .hints
            .iter()
            .find(|h| h.term == "Arena")
            .expect("Arena is in the glossary");
        assert!(arena.definition.contains("20 metres"), "{}", arena.definition);
    }

    #[test]
    fn numbers_name_every_mesh_pass_and_uniform_the_page_needs() {
        let n = json(&numbers());
        let g = game();
        let m: serde_json::Value = serde_json::from_str(&meshes(&g)).unwrap();
        let list = m.as_array().unwrap();
        assert_eq!(
            list.len(),
            Mesh::ALL.len() + 4,
            "the primitives and the ring's floor, kerb, wall and stands"
        );
        assert_eq!(list[0]["vertices"], 36, "the cube's 12 triangles");
        assert_eq!(n["vertex"]["stride"], 36);
        assert_eq!(n["passes"].as_array().unwrap().len(), render::Pass::ORDER.len());
        assert_eq!(n["instance"]["stride"], 64);
        assert_eq!(n["actions"]["forward"], 1);
        assert_eq!(n["actions"]["swing"], 1 << 24);
        assert_eq!(n["view"]["eye"][1], render::VIEW_VALUES);
    }

    /// M6: the ladder's seven rank cards, foot first, each opening its
    /// dossier; a dossier names the opponent, how it fights, what it wears
    /// and where its plate is (worked from each concept sheet's steel,
    /// `plate::covered`), and offers Fight only on an open rung.
    #[test]
    fn the_ladder_shows_rank_cards_and_each_dossier_its_plate_and_fight() {
        let mut g = game();
        let u = json(&act(&mut g, "screen.show", "ladder"));
        assert_eq!(u["ok"]["screen"], "ladder");
        let p = &u["ok"]["panels"][0];
        assert_eq!(p["heading"], "The campaign");
        assert!(
            p["text"]
                .as_str()
                .unwrap()
                .starts_with("13 battles in 4 regions."),
            "{}",
            p["text"]
        );
        // T11: the map, region by region; the seven ranks among the battles
        let regions = p["regions"].as_array().unwrap();
        let heads: Vec<&str> = regions.iter().map(|r| r["heading"].as_str().unwrap()).collect();
        assert_eq!(
            heads,
            ["The Yard", "The Marches", "The Toll Towns", "The High Tourney"]
        );
        let cards: Vec<serde_json::Value> = regions
            .iter()
            .flat_map(|r| r["cards"].as_array().unwrap().clone())
            .collect();
        let ranks: Vec<&str> = cards.iter().map(|c| c["rank"].as_str().unwrap()).collect();
        assert_eq!(ranks, ["F", "E", "D", "C", "", "", "B", "", "", "", "A", "", "S"]);
        assert_eq!(cards[0]["name"], "The squire");
        assert_eq!(cards[4]["name"], "The woodsman");
        assert_eq!(
            cards[4]["action"]["label"], "The woodsman",
            "a battle with no rank by its name"
        );
        assert_eq!(cards[12]["name"], "The mirror");
        assert_eq!(cards[0]["text"], "Open");
        assert_eq!(cards[1]["text"], "Locked: win the squire.");
        assert_eq!(regions[1]["text"], "Opens at 4 marks in the Yard: 0 so far");
        assert_eq!(cards[1]["marks_text"], "0 of 2 marks");
        assert_eq!(cards[3]["action"]["name"], "ladder.rung");
        assert_eq!(cards[3]["action"]["value"], "veteran");
        assert_eq!(cards[3]["action"]["label"], "Rank C");
        let plate = |g: &mut Game, key: &str| {
            let u = json(&act(g, "ladder.rung", key));
            let rows = u["ok"]["panels"][0]["rows"].as_array().unwrap().clone();
            rows.iter().find(|r| r["key"] == "Plate on").unwrap()["does"]
                .as_str()
                .unwrap()
                .to_string()
        };
        assert_eq!(plate(&mut g, "squire"), "Head");
        assert_eq!(
            plate(&mut g, "sellsword"),
            "Head, torso, right upper arm, both forearms"
        );
        assert_eq!(
            plate(&mut g, "hedge"),
            "Head, torso, both upper arms, both forearms, both thighs"
        );
        assert_eq!(plate(&mut g, "veteran"), "Head, left upper arm, both forearms");
        for k in ["banneret", "champion", "mirror"] {
            assert_eq!(plate(&mut g, k), "Every part", "{k}");
        }
        let u = json(&act(&mut g, "ladder.rung", "F"));
        let d = &u["ok"]["panels"][0];
        assert_eq!(u["ok"]["screen"], "dossier");
        assert_eq!(d["heading"], "The squire");
        assert_eq!(d["rows_heading"], "Rank F");
        assert!(d["text"]
            .as_str()
            .unwrap()
            .starts_with("Slow to see a cut coming"));
        assert_eq!(d["actions"][0]["name"], "ladder.fight");
        assert_eq!(d["actions"][0]["enabled"], true);
        assert_eq!(d["actions"][0]["engage"], true);
        assert_eq!(d["actions"][1]["value"], "ladder");
        let u = json(&act(&mut g, "ladder.rung", "S"));
        let d = &u["ok"]["panels"][0];
        assert_eq!(d["actions"][0]["enabled"], false, "the mirror is locked");
        assert_eq!(d["note"], "Locked: win 6 marks in the Toll Towns and win the champion without losing a round and win the headsman without losing a round.");
    }

    // ---- M5: online play, two games joined by a wire in the test ----

    use crate::ticks::TICK_US;

    /// Hands each game's outbox to the other, as the transport would.
    fn carry(a: &mut Game, b: &mut Game) {
        for _ in 0..4 {
            for (_, text) in serde_json::from_str::<Vec<(u32, String)>>(&a.online_outbox()).unwrap() {
                b.online_message(1, &text);
            }
            for (_, text) in serde_json::from_str::<Vec<(u32, String)>>(&b.online_outbox()).unwrap() {
                a.online_message(1, &text);
            }
        }
    }

    /// A frame a tick later in each game, with these bits held.
    fn frame(g: &mut Game, bits: u32, dx: i32) {
        let now = g.clock.last().unwrap_or(0) + TICK_US;
        g.advance(now, bits, dx, 0, 0);
    }

    /// Host and guest by pasted codes, linked, the duel begun in both.
    fn linked() -> (Game, Game) {
        linked_holding(crate::pose::Held::Longsword, crate::pose::Held::Longsword)
    }

    /// T11: `linked`, the host and the guest each holding a weapon.
    fn linked_holding(h: crate::pose::Held, g: crate::pose::Held) -> (Game, Game) {
        let mut host = game();
        let mut guest = game();
        host.kept.weapon = h;
        guest.kept.weapon = g;
        act(&mut host, "online.host", "code");
        act(&mut guest, "online.join", "code");
        let cmds = |g: &Game| {
            ui(g)
                .online
                .cmds
                .iter()
                .map(|c| c.act.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(cmds(&host), ["open", "invite"]);
        assert_eq!(cmds(&guest), ["open"]);
        act(&mut host, "online.offer", "Oinvitation");
        act(&mut guest, "online.reply", "Oinvitation");
        assert_eq!(ui(&guest).online.cmds.last().unwrap().code, "Oinvitation");
        act(&mut guest, "online.answer", "Areply");
        act(&mut host, "online.accept", "Areply");
        host.online_connected(1);
        guest.online_connected(1);
        carry(&mut host, &mut guest);
        frame(&mut host, 0, 0);
        frame(&mut guest, 0, 0);
        (host, guest)
    }

    #[test]
    fn host_shows_a_room_code_and_join_takes_one() {
        let mut g = game();
        let u = json(&act(&mut g, "online.host", "room"));
        assert_eq!(u["ok"]["screen"], "online");
        let p = &u["ok"]["panels"][0];
        assert_eq!(p["heading"], "Host");
        let code = p["fields"][0]["value"].as_str().unwrap().to_string();
        assert_eq!(code.len(), 5);
        let open = &u["ok"]["online"]["cmds"][0];
        assert_eq!(open["act"], "open");
        assert_eq!(open["mode"], "room");
        assert_eq!(open["room"], format!("chumworld-arena-p1-{code}"));
        assert_eq!(open["host"], true);
        assert_eq!(u["ok"]["panels"][1]["id"], "online-status");
        assert_eq!(u["ok"]["panels"][1]["text"], "Waiting for the other player.");

        let mut j = game();
        let u = json(&act(&mut j, "online.join", "room"));
        assert!(
            u["ok"]["online"]["cmds"].as_array().unwrap().is_empty(),
            "no link before a code"
        );
        assert_eq!(u["ok"]["panels"][0]["fields"][0]["id"], "room-in");
        let bad = json(&act(&mut j, "online.room", "AB1"));
        assert_eq!(bad["error"], "A room code is five letters, such as KMRTW.");
        let u = json(&act(&mut j, "online.room", &code.to_lowercase()));
        let open = &u["ok"]["online"]["cmds"][0];
        assert_eq!(open["room"], format!("chumworld-arena-p1-{code}"));
        assert_eq!(open["host"], false);
        // back to the menu closes the link
        let u = json(&act(&mut j, "screen.show", "menu"));
        let cmds = u["ok"]["online"]["cmds"].as_array().unwrap();
        assert_eq!(cmds.last().unwrap()["act"], "close");
        assert!(j.versus.is_none());
    }

    /// A join link: the host's screen offers this page's address with the
    /// code once the page has said where it lives; opening the link shows
    /// Join with the code typed in and opens nothing until Join is chosen.
    #[test]
    fn host_offers_a_join_link_and_the_link_fills_in_join() {
        let mut g = game();
        let u = json(&act(&mut g, "online.host", "room"));
        assert_eq!(
            u["ok"]["panels"][0]["fields"].as_array().unwrap().len(),
            1,
            "no address, no link"
        );
        act(&mut g, "screen.show", "menu");
        act(&mut g, "online.address", "https://example.test/arena/index.html");
        let u = json(&act(&mut g, "online.host", "room"));
        let f = &u["ok"]["panels"][0]["fields"];
        let code = f[0]["value"].as_str().unwrap().to_string();
        assert_eq!(f[1]["id"], "room-link");
        assert_eq!(f[1]["label"], "Join link");
        assert_eq!(
            f[1]["value"],
            format!("https://example.test/arena/index.html?join={code}")
        );
        assert_eq!(f[1]["fixed"], true);

        let mut j = game();
        let bad = json(&act(&mut j, "online.link", "AB1"));
        assert_eq!(bad["error"], "A room code is five letters, such as KMRTW.");
        assert!(j.versus.is_none(), "a bad link goes nowhere");
        let u = json(&act(&mut j, "online.link", &code.to_lowercase()));
        assert_eq!(u["ok"]["screen"], "online");
        let p = &u["ok"]["panels"][0];
        assert_eq!(p["heading"], "Join");
        assert_eq!(
            p["text"],
            "The room code from your join link is filled in. Choose Join. You are the ivory crown."
        );
        assert_eq!(p["fields"][0]["id"], "room-in");
        assert_eq!(p["fields"][0]["value"], code.as_str());
        assert!(
            u["ok"]["online"]["cmds"].as_array().unwrap().is_empty(),
            "nothing opens before Join"
        );
        let u = json(&act(&mut j, "online.room", &code));
        assert_eq!(
            u["ok"]["online"]["cmds"][0]["room"],
            format!("chumworld-arena-p1-{code}")
        );
        // a later Join from the menu starts with an empty box
        act(&mut j, "screen.show", "menu");
        let u = json(&act(&mut j, "online.join", "room"));
        assert_eq!(u["ok"]["panels"][0]["fields"][0]["value"], "");
        assert_eq!(
            u["ok"]["panels"][0]["text"],
            "Type the room code the host gave you, then choose Join. You are the ivory crown."
        );
    }

    /// Two browsers meet by pasted codes and fight: each sees from its own
    /// knight, the host in Knight A and the guest in Knight B, and their
    /// checksums agree.
    #[test]
    fn two_games_meet_by_pasted_codes_and_agree() {
        let (mut host, mut guest) = linked();
        assert_eq!(host.screen, Screen::Duel);
        assert_eq!(guest.screen, Screen::Duel);
        assert_eq!(host.world.me.kind, crate::pose::Kind::KnightA);
        assert_eq!(guest.world.me.kind, crate::pose::Kind::KnightB);
        assert_eq!(bars(&host)[1].who, "The ivory crown");
        assert_eq!(bars(&guest)[1].who, "The black helm");
        let fwd = crate::input::FORWARD;
        for t in 0..200 {
            frame(&mut host, if t % 40 < 25 { fwd } else { 0 }, 30);
            frame(&mut guest, fwd, -50);
            carry(&mut host, &mut guest);
        }
        let (h, g) = (host.versus.as_ref().unwrap(), guest.versus.as_ref().unwrap());
        assert_eq!(h.status(), crate::online::PLAYING);
        assert_eq!(g.status(), crate::online::PLAYING);
        assert!(h.session.made.len() >= 5, "{:?}", h.session.made);
        let common: Vec<_> = h
            .session
            .made
            .iter()
            .filter(|m| g.session.made.contains(m))
            .collect();
        assert!(common.len() >= 5, "the checksums agree tick for tick");
        assert!(h.session.link.agreed.is_some());
        // the guest's own knight walked, and the host sees it there
        let mut seen_by_host = host.world.foes[0].body.pos;
        let guest_own = guest.world.player.pos;
        if host.versus.as_ref().unwrap().session.tick == guest.versus.as_ref().unwrap().session.tick {
            assert_eq!(seen_by_host, guest_own);
        }
        seen_by_host = h.session.world.foes[0].body.pos;
        assert_ne!(seen_by_host, crate::seat::duel().foes[0].body.pos);
        assert!(ui(&host).online.sums.contains(':'));
    }

    /// T11 (CA-Q19: C): each browser's hello carries its player's weapon,
    /// and both build the duel with both: the host's axe in Knight A's hands
    /// and the guest's mace in Knight B's, seen from either side; the
    /// checksums agree tick for tick; a rematch keeps both weapons.
    #[test]
    fn online_each_player_holds_the_weapon_it_chose() {
        use crate::pose::Held;
        let (mut host, mut guest) = linked_holding(Held::Axe, Held::Mace);
        let (h, g) = (host.versus.as_ref().unwrap(), guest.versus.as_ref().unwrap());
        assert!(h.armed && g.armed);
        assert_eq!(
            (h.session.world.me.held, h.session.world.foes[0].fighter.held),
            (Held::Axe, Held::Mace)
        );
        assert_eq!(
            (g.session.world.me.held, g.session.world.foes[0].fighter.held),
            (Held::Axe, Held::Mace)
        );
        assert_eq!(
            (guest.world.me.held, guest.world.foes[0].fighter.held),
            (Held::Mace, Held::Axe),
            "the guest's own view"
        );
        for t in 0..200 {
            frame(&mut host, if t % 30 < 20 { crate::input::FORWARD } else { 0 }, 40);
            frame(&mut guest, crate::input::FORWARD, -40);
            carry(&mut host, &mut guest);
        }
        let (h, g) = (host.versus.as_ref().unwrap(), guest.versus.as_ref().unwrap());
        assert_eq!(h.status(), crate::online::PLAYING);
        let common = h
            .session
            .made
            .iter()
            .filter(|m| g.session.made.contains(m))
            .count();
        assert!(common >= 5, "the checksums agree");
        // a rematch both ask for keeps the weapons
        let mut w = h.session.world.clone();
        if let Some(r) = w.rounds.as_mut() {
            r.phase = crate::rounds::Phase::Over;
        }
        let ask = crate::input::Input {
            keys: crate::seat::REMATCH,
            ..crate::input::Input::default()
        };
        crate::online::Sim::tick(&mut w, [ask, ask]);
        assert_eq!((w.me.held, w.foes[0].fighter.held), (Held::Axe, Held::Mace));
        // a hello with a weapon no player may hold, or none, is the longsword
        assert_eq!(crate::seat::arms_named("dragon"), Held::Longsword);
        assert_eq!(crate::seat::arms_named(""), Held::Longsword);
    }

    /// The other player leaving, going quiet, or disagreeing, each ends the
    /// duel with its reason in words and a way back to the menu.
    #[test]
    fn a_leaving_a_silence_and_a_desync_each_end_the_duel_in_words() {
        // leaving
        let (mut host, mut guest) = linked();
        act(&mut guest, "screen.show", "menu");
        carry(&mut guest, &mut host);
        frame(&mut host, 0, 0);
        assert_eq!(host.screen, Screen::Online);
        let u = ui(&host);
        assert_eq!(u.panels[0].heading, "The online duel has ended");
        assert_eq!(u.panels[0].text, "The other player left the duel.");
        assert_eq!(u.panels[0].actions[0].value, "menu");

        // the other page closing (pagehide): it leaves first
        let (mut host, mut guest) = linked();
        act(&mut guest, "page.leaving", "");
        assert!(guest.versus.is_none());
        carry(&mut guest, &mut host);
        frame(&mut host, 0, 0);
        assert_eq!(ui(&host).panels[0].text, "The other player left the duel.");

        // the transport losing the other browser
        let (mut host, _guest) = linked();
        host.online_left(1);
        frame(&mut host, 0, 0);
        assert_eq!(ui(&host).panels[0].text, "The other player left the duel.");

        // silence: ten seconds with no word
        let (mut host, _guest) = linked();
        for _ in 0..(10 * 60 + 5) {
            frame(&mut host, 0, 0);
        }
        assert_eq!(host.screen, Screen::Online);
        assert_eq!(
            ui(&host).panels[0].text,
            "No word from the other player for 10 seconds, so the duel has ended."
        );

        // a desync
        let (mut host, mut guest) = linked();
        guest.versus.as_mut().unwrap().session.world.me.health.hp -= 1;
        for _ in 0..90 {
            frame(&mut host, 0, 0);
            frame(&mut guest, 0, 0);
            carry(&mut host, &mut guest);
        }
        assert_eq!(host.screen, Screen::Online);
        assert!(
            ui(&host).panels[0].text.starts_with("Desync: "),
            "{}",
            ui(&host).panels[0].text
        );
    }

    /// Rematch online waits for both players, then the duel begins again.
    #[test]
    fn an_online_rematch_waits_for_both_players() {
        let (mut host, mut guest) = linked();
        for g in [&mut host, &mut guest] {
            let r = g.versus.as_mut().unwrap().session.world.rounds.as_mut().unwrap();
            r.phase = crate::rounds::Phase::Over;
            r.wins = [2, 0];
        }
        for _ in 0..5 {
            frame(&mut host, 0, 0);
            frame(&mut guest, 0, 0);
            carry(&mut host, &mut guest);
        }
        assert_eq!(host.screen, Screen::Result);
        let u = ui(&guest);
        assert_eq!(
            u.panels[0].heading, "The knight wins the duel",
            "the guest lost 0 to 2"
        );
        assert_eq!(u.panels[0].actions[0].name, "online.rematch");
        act(&mut host, "online.rematch", "");
        assert_eq!(
            ui(&host).panels[0].note,
            "Waiting for the other player to choose Rematch."
        );
        for _ in 0..20 {
            frame(&mut host, 0, 0);
            frame(&mut guest, 0, 0);
            carry(&mut host, &mut guest);
        }
        assert_eq!(host.screen, Screen::Result, "one player alone does not begin it");
        act(&mut guest, "online.rematch", "");
        for _ in 0..20 {
            frame(&mut host, 0, 0);
            frame(&mut guest, 0, 0);
            carry(&mut host, &mut guest);
        }
        assert_eq!(host.screen, Screen::Duel);
        assert_eq!(guest.screen, Screen::Duel);
        assert_eq!(host.world.rounds.as_ref().unwrap().wins, [0, 0]);
    }
}
