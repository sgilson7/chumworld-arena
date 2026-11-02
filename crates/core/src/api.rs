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
    }
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
    ]
}

/// The score in words: "You 1 · Knight 0".
fn score(g: &Game, r: &crate::rounds::Rounds) -> String {
    fill(
        &g.strings.get("round_score"),
        &[("you", r.wins[0].to_string()), ("foe", r.wins[1].to_string())],
    )
}

/// How a round ended, in words: who fell and to what.
fn fall_text(g: &Game, r: &crate::rounds::Rounds) -> (String, String) {
    use crate::rounds::{Blow, Fall};
    let s = &g.strings;
    let n = [("n", r.round.to_string())];
    match r.fall {
        Some(Fall::One { who, blow }) => {
            let heading = fill(&s.get(if who == 1 { "round_to_you" } else { "round_to_foe" }), &n);
            let text = match blow {
                Blow::Sand => s.get(if who == 1 {
                    "fell_foe_sand"
                } else {
                    "fell_you_sand"
                }),
                Blow::Blade => s.get(if who == 1 {
                    "fell_foe_blade"
                } else {
                    "fell_you_blade"
                }),
                Blow::Cut(k) => fill(
                    &s.get(if who == 1 { "fell_foe_cut" } else { "fell_you_cut" }),
                    &[("blow", s.get(k.key()))],
                ),
            };
            (heading, text)
        }
        _ => (fill(&s.get("round_again"), &n), s.get("fell_both")),
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
                button(g, "screen.show", "menu", "leave_duel", false),
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
                panels.push(Panel {
                    heading: s.get(if won { "result_won" } else { "result_lost" }),
                    text: fill(
                        &s.get(if won {
                            "result_won_text"
                        } else {
                            "result_lost_text"
                        }),
                        &[("you", r.wins[0].to_string()), ("foe", r.wins[1].to_string())],
                    ),
                    note: if waiting {
                        s.get("online.rematch_wait")
                    } else {
                        String::new()
                    },
                    actions: vec![rematch, button(g, "screen.show", "menu", "back_to_menu", false)],
                    ..panel("result")
                });
            }
        }
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
        version: g.ui_version,
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
            fields: vec![field(g, "room-out", "room_code", &v.code, true, true)],
            actions: vec![instead("online.host"), back],
            ..panel("online")
        },
        (Role::Guest, Way::Room) if v.code.is_empty() => Panel {
            heading: s.get("join"),
            text: s.get("join_room_text"),
            fields: vec![field(g, "room-in", "room_code", "", false, true)],
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
        assert_eq!(labels, ["Practice", "Host", "Join", "Pasted codes", "Settings"]);
        assert_eq!(p["actions"][0]["engage"], true);
        assert_eq!(p["actions"][0]["value"], "duel");
        // M5: Host and Join open, by room code, and Pasted codes beside them
        assert_eq!(p["actions"][1]["enabled"], true);
        assert_eq!(p["actions"][1]["name"], "online.host");
        assert_eq!(p["actions"][2]["name"], "online.join");
        assert_eq!(p["actions"][3]["value"], "paste");
        // and nothing asks the transport for a link until one is chosen (CA-4)
        assert!(v["ok"]["online"]["cmds"].as_array().unwrap().is_empty());
        assert!(
            v["ok"]["bars"].as_array().unwrap().is_empty(),
            "no bars before the duel"
        );
        // the settings: four rows, each with its value in words
        let v = json(&act(&mut g, "screen.show", "settings"));
        let rows = v["ok"]["panels"][0]["settings"].as_array().unwrap();
        let values: Vec<&str> = rows.iter().map(|r| r["value"].as_str().unwrap()).collect();
        assert_eq!(values, ["100%", "Off", "105°", "None"]);
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
        let mut host = game();
        let mut guest = game();
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
