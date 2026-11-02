//! The game as the page sees it: the screens (M2: title, main menu,
//! settings, the duel, pause, a round's result, the duel's result), the
//! clock that turns display frames into ticks, the frame the page draws, the
//! save (`save.rs`), and the one door for settings, screens and flags (`act`).
//!
//! CHUMWORLD's `game.rs` (commit d9cb5b5) with its menus, travel, sound,
//! save and detail levels taken out (R2.6), and M2's screens put back in
//! their place. Practice is a duel in rounds (`rounds.rs`) against the
//! computer's knight; the screen follows the rounds' phase (a fight, the
//! beat after a fall, the result). Each display frame the page
//! calls `advance` with its timestamp, the held action bits and the mouse
//! and wheel movement since the last frame, then reads the frame (`view`,
//! `instances`, `draws`, and `static_values` when `static_version` changes)
//! and, when `ui_version` changes, the view model (`api::ui`). The page does
//! no arithmetic: the core has already placed everything between the last
//! two ticks (`alpha`).

use crate::camera::{self, Aim};
use crate::fx::{self, Fx, ONE};
use crate::geom::V3;
use crate::glossary::Entry;
use crate::input::{Input, Mouse, Pending};
use crate::look::Look;
use crate::page::Strings;
use crate::prim::MeshTable;
use crate::render::{self, Camera, Frame, Group, Scene, Shapes};
use crate::ticks::Clock;
use crate::world::{AimHelp, World};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// the title over the ring
    Title,
    /// the main menu: Practice, Host, Join, Settings
    Menu,
    /// the reader's settings and the save
    Settings,
    /// a round being fought
    Duel,
    Paused,
    /// the beat after a fall: who fell, and the blow
    Round,
    /// the duel won or lost: rematch or back
    Result,
}

impl Screen {
    pub fn name(self) -> &'static str {
        match self {
            Screen::Title => "title",
            Screen::Menu => "menu",
            Screen::Settings => "settings",
            Screen::Duel => "duel",
            Screen::Paused => "paused",
            Screen::Round => "round",
            Screen::Result => "result",
        }
    }

    /// The duel runs on this screen.
    pub fn plays(self) -> bool {
        matches!(self, Screen::Duel | Screen::Round | Screen::Result)
    }
}

/// The foe Practice puts in the ring: the computer's knight (M2, R4.4),
/// thinking with the `knight` pilot. Its body is the knight the player does
/// not wear (M3, CA-Q13: A; `rounds::begin`).
pub const PRACTICE_FOE: &str = "knight";
/// The most pixels the drawing buffer may hold (2560 × 1600); a larger
/// screen draws at a lower resolution, scaled up by the browser.
pub const MAX_PIXELS: i64 = 2560 * 1600;
/// On a software renderer (no graphics card) the page draws at this share of
/// the screen's pixel density, percent.
pub const SOFTWARE_SCALE_PCT: i32 = 50;
/// How long the pause hint shows once a duel starts, ticks (6 s).
pub const HINT_TICKS: u64 = 360;
/// The title's and the menu's camera drifts round the ring's middle this
/// fast, angle units a tick (2° a second).
pub const TITLE_DRIFT: i32 = fx::TURN / 360 * 2 / 60;
/// And looks down this far, from this far back: across the ring to its far
/// wall and stands (M2, R4.1).
pub const TITLE_PITCH: i32 = -fx::deg(12);
pub const TITLE_BACK: Fx = fx::int(15);
/// The settings' steps: the look speed by this many percent, the field of
/// view by a degree.
pub const LOOK_STEP_PCT: i32 = 10;
pub const FOV_STEP: i32 = 1;

#[derive(Debug, Clone)]
pub struct Game {
    pub strings: Strings,
    pub glossary: Vec<Entry>,
    pub look: Look,
    pub clock: Clock,
    pub pending: Pending,
    pub world: World,
    pub screen: Screen,
    /// the screen the settings go back to
    pub settings_from: Screen,
    /// the page's seed, and duels begun, each a world of its own chance
    pub seed: u64,
    pub duels: u64,
    /// what the save keeps, and the counts the page acts on: a new save to
    /// keep in the browser (`save_seq`), a download asked for (`save_export`)
    pub kept: crate::save::Kept,
    pub save_seq: u32,
    pub save_export: u32,
    /// a line about the save for the settings and the menu
    pub notice: String,
    /// acts run on each new duel's world as it begins (`practice.stage`):
    /// staging for tests and pictures
    pub staged: Vec<(String, String)>,
    /// the drawing buffer's size and the CSS size's aspect (Q16)
    pub pixels: (i32, i32),
    pub aspect: Fx,
    css: (i32, i32),
    dpr_pct: i32,
    pub software: bool,
    groups: Vec<Group>,
    /// the primitives and the ring's meshes, uploaded once
    pub meshes: MeshTable,
    pub scene: Scene,
    pub frame: Frame,
    /// share of a tick the drawing is past the last tick
    pub alpha: Fx,
    /// ticks of the duel since it began, for the pause hint
    pub play_ticks: u64,
    /// ticks the title and menu have shown, for their drifting camera
    pub title_ticks: u64,
    pub ui_version: u32,
}

impl Game {
    /// A game from the strings, the glossary, the look and the page's seed.
    pub fn new(strings_json: &str, glossary_md: &str, look_json: &str, seed: u32) -> Result<Game, String> {
        let strings = crate::page::parse_strings(strings_json).map_err(|e| e.message())?;
        // a slip in the glossary costs a hover hint, not the game; the
        // check (`cargo test`) fails on it
        let glossary = crate::glossary::load(glossary_md);
        let look = Look::parse(look_json)?;
        let mut meshes = MeshTable::default();
        let groups = crate::ring::scene(&mut meshes);
        let kept = crate::save::Kept::default();
        let mut g = Game {
            strings,
            glossary,
            look,
            clock: Clock::default(),
            pending: Pending::default(),
            world: World::new(seed as u64),
            screen: Screen::Title,
            settings_from: Screen::Menu,
            seed: seed as u64,
            duels: 0,
            kept,
            save_seq: 0,
            save_export: 0,
            notice: String::new(),
            staged: Vec::new(),
            pixels: (1, 1),
            aspect: ONE,
            css: (1, 1),
            dpr_pct: 100,
            software: false,
            groups,
            meshes,
            scene: Scene::default(),
            frame: Frame::default(),
            alpha: 0,
            play_ticks: 0,
            title_ticks: 0,
            ui_version: 1,
        };
        g.world = g.fresh_world();
        g.draw();
        Ok(g)
    }

    /// An empty ring with the player in it, played from the helmet (M1: the
    /// port's own path, `Control::Shoulder`, stays the world's default,
    /// R3.1), with the reader's settings.
    fn fresh_world(&self) -> World {
        let mut w = World::new(self.seed ^ self.duels.wrapping_mul(0x9e37_79b9_7f4a_7c15));
        w.settings.control = crate::world::Control::FirstPerson;
        w.settings.chivalry = true; // M4 (R6.1)
        apply(&mut w, &self.kept);
        w
    }

    /// A new duel in rounds against the computer's knight.
    fn start_practice(&mut self) {
        self.duels += 1;
        self.world = self.fresh_world();
        crate::rounds::begin(&mut self.world, PRACTICE_FOE, self.kept.knight.kind())
            .expect("the knight is in the pilots' book");
        for (n, v) in self.staged.clone() {
            // each was tried on the world when it was staged
            let _ = self.world.act(&n, &v);
        }
        self.play_ticks = 0;
    }

    /// The CSS size and the device pixel ratio (percent) in; the drawing
    /// buffer's size out.
    pub fn resize(&mut self, css_w: i32, css_h: i32, dpr_pct: i32) -> [i32; 2] {
        self.css = (css_w.max(1), css_h.max(1));
        self.dpr_pct = dpr_pct.clamp(25, 400);
        let scale = if self.software {
            self.dpr_pct * SOFTWARE_SCALE_PCT / 100
        } else {
            self.dpr_pct
        };
        let mut w = self.css.0 as i64 * scale as i64 / 100;
        let mut h = self.css.1 as i64 * scale as i64 / 100;
        if w * h > MAX_PIXELS {
            let k = fx::isqrt_u64((MAX_PIXELS * 1_000_000 / (w * h)) as u64) as i64;
            w = w * k / 1000;
            h = h * k / 1000;
        }
        self.pixels = (w.max(1) as i32, h.max(1) as i32);
        self.aspect = fx::ratio(self.css.0, self.css.1);
        self.draw();
        [self.pixels.0, self.pixels.1]
    }

    /// One display frame: the page's clock in microseconds, the held action
    /// bits, and the mouse and wheel movement in hundredths of a CSS pixel.
    pub fn advance(&mut self, now_us: i64, bits: u32, dx: i32, dy: i32, wheel: i32) {
        let ticks = self.clock.frame(now_us, bits);
        if self.screen.plays() {
            self.pending.add(dx, dy, wheel);
            let before = self.stamp();
            for (k, b) in ticks.iter().enumerate() {
                let m = if k == 0 {
                    self.pending.take()
                } else {
                    Mouse::default()
                };
                self.world.step(&Input::from_page(*b, m));
                self.play_ticks += 1;
            }
            self.follow_rounds();
            if before != self.stamp() {
                self.ui_version += 1;
            }
            self.alpha = self.clock.alpha();
        } else if self.in_duel() {
            // paused, or the settings over a paused duel: the world holds
            // still; time passing is forgotten, so resuming runs no catch-up
            self.pending.clear();
            self.clock.rest();
        } else {
            self.pending.clear();
            self.title_ticks += ticks.len() as u64;
            self.alpha = self.clock.alpha();
        }
        self.draw();
    }

    /// Whether a duel is under way behind this screen (fought, paused, or
    /// the settings opened from its pause).
    pub fn in_duel(&self) -> bool {
        self.screen.plays()
            || self.screen == Screen::Paused
            || (self.screen == Screen::Settings && self.settings_from == Screen::Paused)
    }

    /// Everything the lines over the duel show, in one value: the page reads
    /// the screen again when it changes.
    fn stamp(&self) -> (bool, u64, u64, u32) {
        let duel = crate::player::hud_stamp(&self.world);
        let wound = crate::wounds::stamp(&self.world);
        let rounds = self
            .world
            .rounds
            .as_ref()
            .map(|r| {
                (r.round as u32) << 24
                    | (r.wins[0] as u32) << 20
                    | (r.wins[1] as u32) << 16
                    | crate::rounds::seconds_left(r)
            })
            .unwrap_or(0);
        (
            self.hint_shown(),
            crate::world::value_hash(&format!("{duel:?}")),
            crate::world::value_hash(&format!("{wound:?}")),
            rounds,
        )
    }

    /// The screen follows the rounds: a fight, the beat after a fall, the
    /// duel's result.
    fn follow_rounds(&mut self) {
        let Some(r) = self.world.rounds.as_ref() else {
            return;
        };
        let want = match r.phase {
            crate::rounds::Phase::Fight => Screen::Duel,
            crate::rounds::Phase::Beat(_) => Screen::Round,
            crate::rounds::Phase::Over => Screen::Result,
        };
        if self.screen != want {
            self.screen = want;
            self.pending.clear();
            self.ui_version += 1;
        }
    }

    pub fn hint_shown(&self) -> bool {
        self.screen == Screen::Duel && self.play_ticks < HINT_TICKS
    }

    /// The title's and the menu's slow turn round the ring's middle.
    fn title_aim(&self) -> Aim {
        let drift =
            (self.title_ticks as i64 * TITLE_DRIFT as i64 + fx::mul(self.alpha, TITLE_DRIFT) as i64) as i32;
        Aim {
            yaw: fx::wrap(crate::world::START_YAW.wrapping_add(drift)),
            pitch: TITLE_PITCH,
            rest: (0, 0),
            pct: self.world.aim.pct,
        }
    }

    pub fn camera(&self) -> Camera {
        if !self.in_duel() {
            // M2 (R2.11): the title and the menu look across the ring from
            // behind its middle; a duel cuts straight to the eye
            return camera::view_from(V3::ZERO, self.title_aim(), TITLE_BACK, |x, z| {
                self.world.ground(x, z)
            });
        }
        let mut cam = self.world.camera(self.alpha, self.pending.peek(), camera::BACK);
        if self.world.first_person() {
            cam.fov = camera::vertical_fov(self.world.settings.fov, self.aspect);
        }
        cam
    }

    /// Builds the frame the page draws next.
    fn draw(&mut self) {
        self.world.eye_view = self.in_duel() && self.world.first_person();
        let cam = self.camera();
        let origin = render::origin_for(cam.eye);
        if origin != self.scene.origin || self.scene.version == 0 {
            self.scene = Scene::build(&self.groups, origin, self.scene.version + 1);
        }
        let mut moving = Shapes::default();
        moving.sky(cam.eye);
        self.world.draw(self.alpha, &self.look, &mut moving);
        // blades and other thin things edge-on keep a width
        render::keep_visible(&mut moving.list, cam.eye);
        self.frame = render::Frame::build(&self.scene, &moving, &cam, self.aspect);
    }

    pub(crate) fn show(&mut self, s: Screen) {
        if self.screen != s {
            self.screen = s;
            self.pending.clear();
            self.ui_version += 1;
            self.draw();
        }
    }

    /// The menu, the duel behind it ended.
    fn leave_for_menu(&mut self) {
        if self.in_duel() {
            self.world = self.fresh_world();
        }
        self.show(Screen::Menu);
    }

    /// The reader's settings changed: into the world, and a new save to keep.
    fn kept_changed(&mut self) {
        apply(&mut self.world, &self.kept);
        self.save_seq += 1;
        self.ui_version += 1;
    }

    /// The save's text, for the page to keep or download.
    pub fn save_text(&self) -> String {
        crate::save::text(&self.kept)
    }

    /// A save read: kept and applied, or the refusal's line.
    fn load(&mut self, text: &str, from_file: bool) {
        match crate::save::read(text) {
            Ok(k) => {
                self.kept = k;
                apply(&mut self.world, &self.kept);
                if from_file {
                    // a file read is kept in this browser too
                    self.save_seq += 1;
                    self.notice = self.strings.get("save_loaded");
                }
            }
            // no save in this browser yet: nothing to say
            Err(_) if !from_file && text.trim().is_empty() => {}
            Err(why) => {
                let key = if from_file {
                    "save_refused_file"
                } else {
                    "save_refused_stored"
                };
                self.notice = crate::api::fill(&self.strings.get(key), &[("why", why.why(&self.strings))]);
            }
        }
        self.ui_version += 1;
    }

    /// The one door for screens, settings and flags. The error is the
    /// message the page shows.
    pub fn act(&mut self, name: &str, value: &str) -> Result<(), String> {
        match name {
            "screen.show" => match value {
                "title" => self.show(Screen::Title),
                "menu" => self.leave_for_menu(),
                "settings" => {
                    if self.screen != Screen::Settings {
                        self.settings_from = self.screen;
                    }
                    self.notice.clear();
                    self.show(Screen::Settings);
                }
                "back" => {
                    let to = if self.screen == Screen::Settings {
                        self.settings_from
                    } else {
                        Screen::Menu
                    };
                    self.show(to);
                }
                "duel" => {
                    // from the menu or a result, a new duel; from a pause,
                    // the duel goes on where it was
                    if !self.in_duel() || self.screen == Screen::Result {
                        self.start_practice();
                    }
                    self.show(Screen::Duel);
                    self.follow_rounds();
                }
                "paused" => {
                    if self.in_duel() {
                        self.show(Screen::Paused);
                    }
                }
                other => return Err(format!("There is no screen \"{other}\".")),
            },
            // Esc, a lost pointer and a hidden tab pause a duel; Esc also
            // closes the settings
            "key.escape" | "pointer.lost" | "page.hidden" => {
                if matches!(self.screen, Screen::Duel | Screen::Round) {
                    self.show(Screen::Paused);
                } else if self.screen == Screen::Settings && name == "key.escape" {
                    self.show(self.settings_from);
                }
            }
            // a reader's look speed, percent (?look= on the page's address)
            "look.speed" => {
                let pct: i32 = value.trim().parse().map_err(|_| {
                    format!("The look speed is a whole percentage, such as 100, not \"{value}\".")
                })?;
                self.kept.look_pct = pct.clamp(camera::LOOK_PCT_MIN, camera::LOOK_PCT_MAX);
                apply(&mut self.world, &self.kept);
            }
            "settings.look_speed" => {
                let step = step_of(value)?;
                self.kept.look_pct = (self.kept.look_pct + step * LOOK_STEP_PCT)
                    .clamp(camera::LOOK_PCT_MIN, camera::LOOK_PCT_MAX);
                self.kept_changed();
            }
            "settings.fov" => {
                let step = step_of(value)?;
                self.kept.fov =
                    (self.kept.fov + step * FOV_STEP).clamp(crate::world::FOV_MIN, crate::world::FOV_MAX);
                self.kept_changed();
            }
            "settings.invert_pitch" => {
                self.kept.invert_pitch = !self.kept.invert_pitch;
                self.kept_changed();
            }
            "settings.aim_help" => {
                self.kept.aim_help = match self.kept.aim_help {
                    AimHelp::None => AimHelp::SoftPull,
                    AimHelp::SoftPull => AimHelp::None,
                };
                self.kept_changed();
            }
            "save.knight" => {
                self.kept.knight = crate::save::Knight::named(value.trim())
                    .ok_or_else(|| format!("A knight's look is \"a\" or \"b\", not \"{value}\"."))?;
                self.kept_changed();
            }
            // the save this browser keeps, handed over by the page at the start
            "save.stored" => self.load(value, false),
            // a save file the reader chose
            "save.file" => self.load(value, true),
            // "name=value": an act run on every duel's world as it begins,
            // tried on a duel's world now so a bad one is refused at once
            "practice.stage" => {
                let (n, v) = value
                    .split_once('=')
                    .ok_or_else(|| format!("A staged act is name=value, not \"{value}\"."))?;
                let mut trial = self.fresh_world();
                crate::rounds::begin(&mut trial, PRACTICE_FOE, self.kept.knight.kind())?;
                trial.act(n, v)?;
                self.staged.push((n.to_string(), v.to_string()));
            }
            "save.download" => {
                self.save_export += 1;
                self.ui_version += 1;
            }
            "display.software" => {
                self.software = value == "true";
                let (w, h) = self.css;
                self.resize(w, h, self.dpr_pct);
            }
            _ => self.world.act(name, value)?,
        }
        Ok(())
    }

    /// The numbers glossary placeholders name, as text.
    pub fn glossary_values(&self) -> Vec<(&'static str, String)> {
        let mut v = vec![
            ("walk.speed", fx::decimal(crate::player::WALK, 1)),
            ("run.speed", fx::decimal(crate::player::RUN, 1)),
            ("ring.radius", (crate::world::RADIUS / ONE).to_string()),
            ("health.player", crate::health::PLAYER.to_string()),
            ("fov.min", crate::world::FOV_MIN.to_string()),
            ("fov.max", crate::world::FOV_MAX.to_string()),
        ];
        v.extend(crate::rounds::values());
        v.extend(crate::chivalry::values()); // M4
        v
    }
}

/// What the save keeps, into a world's settings.
pub fn apply(w: &mut World, k: &crate::save::Kept) {
    w.aim.pct = k.look_pct;
    w.settings.invert_pitch = k.invert_pitch;
    w.settings.fov = k.fov;
    w.settings.aim_help = k.aim_help;
}

/// A settings button's step: "+" one up, "-" one down.
fn step_of(value: &str) -> Result<i32, String> {
    match value.trim() {
        "+" => Ok(1),
        "-" => Ok(-1),
        other => Err(format!("A setting steps by \"+\" or \"-\", not \"{other}\".")),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::input::FORWARD;
    use crate::ticks::TICK_US;

    pub fn game() -> Game {
        let mut g = Game::new(
            include_str!("../../../data/strings.json"),
            include_str!("../../../GLOSSARY.md"),
            include_str!("../../../data/look.json"),
            42,
        )
        .expect("the game starts from the repository's data");
        g.resize(1920, 1080, 100);
        g
    }

    /// The time of the last frame.
    fn now(g: &Game) -> i64 {
        g.clock.last().unwrap_or(0)
    }

    /// `n` frames a tick apart, with these bits held and this movement.
    pub fn frames(g: &mut Game, n: i64, bits: u32, dx: i32) {
        for _ in 0..n {
            g.advance(now(g) + TICK_US, bits, dx, 0, 0);
        }
    }

    /// M3 (R4.8): the save's look B puts the player in Knight B and the
    /// computer's knight in Knight A, and its bar reads "The black helm".
    #[test]
    fn the_save_s_knight_look_b_swaps_the_knights() {
        let mut g = game();
        g.act("save.knight", "b").unwrap();
        practice(&mut g);
        assert_eq!(g.world.me.kind, crate::pose::Kind::KnightB);
        assert_eq!(g.world.foes[0].fighter.kind, crate::pose::Kind::KnightA);
        assert_eq!(g.world.foes[0].brain.key, "knight");
        let bars = crate::api::bars(&g);
        assert_eq!(bars[1].who, "The black helm");
    }

    /// M3 in first person: whichever knight the player wears, nothing of
    /// its own figure is drawn ahead of the eye within 25 cm (Knight A's
    /// visor slit stood 12 cm ahead and 11 cm below it, 22.7 cm from the
    /// head's middle, past the player's 22 cm cull, and filled the middle
    /// of the view). Its shoulders, 38 cm off and more, stay.
    #[test]
    fn neither_knight_s_helm_is_drawn_across_its_own_eye() {
        for look in ["a", "b"] {
            let mut g = game();
            g.act("save.knight", look).unwrap();
            practice(&mut g);
            let mut w = g.world.clone();
            w.foes.clear();
            let eye = crate::player::eye(&w, ONE, w.aim);
            let ahead = crate::geom::facing(w.aim.yaw, 0);
            let mut out = crate::render::Shapes::default();
            crate::player::draw(&w, ONE, &crate::look::tests::look(), &mut out);
            let near: Vec<_> = out
                .list
                .iter()
                .filter(|s| s.at.t.dist(eye) < ONE / 4 && s.at.t.sub(eye).dot(ahead) > 0)
                .collect();
            assert!(
                near.is_empty(),
                "look {look}: {} shapes before the eye",
                near.len()
            );
            let shoulders = out.list.iter().filter(|s| s.at.t.dist(eye) < ONE / 2).count();
            assert!(shoulders > 0, "look {look}: the shoulders are still drawn");
        }
    }

    /// The title, then the menu, then Practice: a duel in rounds against
    /// the computer's knight, round 1.
    pub fn practice(g: &mut Game) {
        g.act("screen.show", "menu").unwrap();
        g.act("screen.show", "duel").unwrap();
    }

    #[test]
    fn the_game_opens_on_the_title_then_the_menu_and_practice_begins_a_duel_in_rounds() {
        let mut g = game();
        assert_eq!(g.screen, Screen::Title);
        assert!(g.world.foes.is_empty(), "no foe before Practice");
        g.act("screen.show", "menu").unwrap();
        assert_eq!(g.screen, Screen::Menu);
        g.act("screen.show", "duel").unwrap();
        assert_eq!(g.screen, Screen::Duel);
        assert_eq!(g.world.foes.len(), 1, "Practice puts one foe in the ring");
        assert_eq!(g.world.foes[0].brain.key, "knight");
        assert_eq!(g.world.rounds.as_ref().unwrap().round, 1);
        assert!(g.world.first_person(), "played from the helmet");
        assert!(g.world.eye_view);
        // M3: the player wears Knight A, the computer's knight Knight B
        assert_eq!(g.world.foes[0].fighter.kind, crate::pose::Kind::KnightB);
        assert_eq!(g.world.me.kind, crate::pose::Kind::KnightA);
        assert_eq!(
            g.act("screen.show", "nowhere"),
            Err("There is no screen \"nowhere\".".into())
        );
        assert_eq!(
            g.act("screen.jump", "duel"),
            Err("Unknown action \"screen.jump\".".into())
        );
    }

    /// M2: the screen follows the rounds. A fall shows the round's result
    /// for the beat, then the next round; two rounds won show the duel's
    /// result, and Rematch starts a new duel at round 1.
    #[test]
    fn the_screen_follows_the_rounds_to_the_result_and_rematch_starts_again() {
        let mut g = game();
        practice(&mut g);
        let beat = crate::rounds::BEAT_TICKS as i64;
        for round in 1..=2 {
            g.world.foes[0].fighter.health.hp = 0;
            frames(&mut g, 1, 0, 0);
            assert_eq!(g.screen, Screen::Round, "round {round}");
            frames(&mut g, beat, 0, 0);
        }
        assert_eq!(g.screen, Screen::Result);
        assert_eq!(g.world.rounds.as_ref().unwrap().winner(), Some(0));
        let u = crate::api::ui(&g);
        assert!(!u.capture, "the result lets go of the pointer");
        g.act("screen.show", "duel").unwrap();
        assert_eq!(g.screen, Screen::Duel);
        let r = g.world.rounds.as_ref().unwrap();
        assert_eq!((r.round, r.wins), (1, [0, 0]), "a new duel");
        g.act("screen.show", "menu").unwrap();
        assert_eq!(g.screen, Screen::Menu);
        assert!(g.world.rounds.is_none(), "leaving ends the duel");
    }

    /// M2: each setting steps within its range, goes into the world, and
    /// asks for a new save; the settings go back where they were opened.
    #[test]
    fn the_settings_step_within_their_ranges_and_each_change_is_saved() {
        let mut g = game();
        g.act("screen.show", "menu").unwrap();
        g.act("screen.show", "settings").unwrap();
        assert_eq!(g.screen, Screen::Settings);
        let seq = g.save_seq;
        g.act("settings.fov", "+").unwrap();
        assert_eq!(g.kept.fov, 106);
        assert_eq!(g.world.settings.fov, 106);
        assert_eq!(g.save_seq, seq + 1);
        for _ in 0..20 {
            g.act("settings.fov", "+").unwrap();
        }
        assert_eq!(g.kept.fov, 110, "held at 110");
        for _ in 0..20 {
            g.act("settings.fov", "-").unwrap();
        }
        assert_eq!(g.kept.fov, 100, "held at 100");
        g.act("settings.look_speed", "-").unwrap();
        assert_eq!(g.world.aim.pct, 90);
        g.act("settings.invert_pitch", "toggle").unwrap();
        assert!(g.world.settings.invert_pitch);
        assert_eq!(g.world.settings.aim_help, AimHelp::None, "off by default (T5)");
        g.act("settings.aim_help", "toggle").unwrap();
        assert_eq!(g.world.settings.aim_help, AimHelp::SoftPull);
        assert_eq!(
            g.act("settings.fov", "wide"),
            Err("A setting steps by \"+\" or \"-\", not \"wide\".".into())
        );
        g.act("screen.show", "back").unwrap();
        assert_eq!(g.screen, Screen::Menu);
        // a duel started now keeps them
        g.act("screen.show", "duel").unwrap();
        assert_eq!(g.world.settings.fov, 100);
        assert!(g.world.settings.invert_pitch);
        // opened from a pause, the settings go back to it, the duel held
        g.act("key.escape", "").unwrap();
        g.act("screen.show", "settings").unwrap();
        assert!(g.in_duel());
        g.act("key.escape", "").unwrap();
        assert_eq!(g.screen, Screen::Paused);
    }

    /// M2 (T2): the browser's save read at the start sets the settings; a
    /// bad one leaves the defaults and says why; a file read is kept in the
    /// browser too, and a bad file changes nothing.
    #[test]
    fn a_save_read_at_the_start_or_from_a_file_sets_the_settings_and_a_bad_one_is_refused() {
        let mut g = game();
        let k = crate::save::Kept {
            fov: 102,
            ..crate::save::Kept::default()
        };
        g.act("save.stored", &crate::save::text(&k)).unwrap();
        assert_eq!(g.kept, k);
        assert_eq!(g.world.settings.fov, 102);
        assert_eq!(g.save_seq, 0, "read back, not written again");
        let mut g2 = game();
        g2.act("save.stored", "").unwrap();
        assert_eq!(g2.notice, "", "no save yet: nothing to say");
        g2.act("save.stored", "{}").unwrap();
        assert_eq!(
            g2.notice,
            "The save in this browser was refused: it is a file of another kind, not this game's save. The settings start from their defaults."
        );
        // a file
        g.act("save.file", "garbage").unwrap();
        assert_eq!(
            g.notice,
            "That file was refused: it is not a save file. Your settings are as they were."
        );
        assert_eq!(g.kept, k);
        let k2 = crate::save::Kept {
            fov: 108,
            knight: crate::save::Knight::B,
            ..k
        };
        g.act("save.file", &crate::save::text(&k2)).unwrap();
        assert_eq!(g.kept, k2);
        assert_eq!(g.save_seq, 1, "kept in the browser too");
        assert_eq!(g.save_text(), crate::save::text(&k2));
        let ex = g.save_export;
        g.act("save.download", "").unwrap();
        assert_eq!(g.save_export, ex + 1);
    }

    /// A staged act runs on every duel's world as it begins; a bad one is
    /// refused when it is staged.
    #[test]
    fn a_staged_act_runs_on_each_new_duel_and_a_bad_one_is_refused_at_once() {
        let mut g = game();
        g.act("practice.stage", "duel.health=7").unwrap();
        practice(&mut g);
        assert_eq!(g.world.foes[0].fighter.health.hp, 7);
        g.act("screen.show", "menu").unwrap();
        g.act("screen.show", "duel").unwrap();
        assert_eq!(g.world.foes[0].fighter.health.hp, 7, "and on the next duel");
        assert_eq!(
            g.act("practice.stage", "duel.health=lots"),
            Err("A foe's health is a whole number from 0 to 1000, not \"lots\".".into())
        );
        assert!(g.act("practice.stage", "nothing").is_err());
        assert_eq!(g.staged.len(), 1);
    }

    /// M1 (CA-Q15: B): in the duel the camera is the eye's at the reader's
    /// field of view across the screen (105° across 16:9 is 72.5° high).
    #[test]
    fn the_duel_is_seen_from_the_eye_at_the_field_of_view_set() {
        let mut g = game();
        practice(&mut g);
        let c = g.camera();
        assert!((c.fov - fx::deg(725) / 10).abs() <= fx::deg(3) / 5, "{}", c.fov);
        let head = g.world.me.pose.head;
        assert!(c.eye.dist(head) < ONE / 8, "{:?} {:?}", c.eye, head);
    }

    #[test]
    fn escape_a_lost_pointer_and_a_hidden_tab_pause_the_duel_and_only_the_duel() {
        for name in ["key.escape", "pointer.lost", "page.hidden"] {
            let mut g = game();
            g.act(name, "").unwrap();
            assert_eq!(g.screen, Screen::Title, "{name} on the title changes nothing");
            practice(&mut g);
            g.act(name, "").unwrap();
            assert_eq!(g.screen, Screen::Paused, "{name}");
        }
    }

    #[test]
    fn a_paused_world_holds_still_and_resuming_runs_no_catch_up() {
        let mut g = game();
        practice(&mut g);
        frames(&mut g, 10, FORWARD, 0);
        let tick = g.world.tick;
        g.act("key.escape", "").unwrap();
        let held = g.world.clone();
        // ten seconds paused with W held
        frames(&mut g, 600, FORWARD, 500);
        assert_eq!(g.world, held);
        // D tapped and let go inside a frame shorter than a tick, still paused
        g.advance(now(&g) + TICK_US / 3, crate::input::RIGHT, 0, 0, 0);
        g.act("screen.show", "duel").unwrap();
        g.advance(now(&g) + TICK_US, FORWARD, 0, 0, 0);
        assert_eq!(g.world.tick, tick + 1, "one frame of the duel runs one tick");
        // the tap stayed in the pause: the body still moves straight ahead
        let sideways = g.world.player.vel.dot(crate::geom::right_of(g.world.aim.yaw));
        assert!(
            sideways.abs() <= 4,
            "a key tapped while paused reached play: {sideways}"
        );
        // time paused short of a tick is forgotten too: a pause of six
        // tenths of a tick (on the third left over from the tap above, so
        // it ends 0.93 into a tick), then half a tick of the duel, runs nothing
        g.act("key.escape", "").unwrap();
        g.advance(now(&g) + TICK_US * 6 / 10, FORWARD, 0, 0, 0);
        g.act("screen.show", "duel").unwrap();
        let tick = g.world.tick;
        g.advance(now(&g) + TICK_US / 2, FORWARD, 0, 0, 0);
        assert_eq!(g.world.tick, tick, "the paused part of a tick ran on resuming");
    }

    #[test]
    fn the_same_world_gives_the_same_frame_byte_for_byte() {
        let run = || {
            let mut g = game();
            practice(&mut g);
            frames(&mut g, 90, FORWARD, 120);
            g.frame.clone()
        };
        let (a, b) = (run(), run());
        assert_eq!(a, b);
        assert!(a.instances.len() > 16 && !a.draws.is_empty());
    }

    #[test]
    fn the_body_is_drawn_between_ticks_by_the_clock_s_alpha() {
        let mut g = game();
        practice(&mut g);
        frames(&mut g, 30, FORWARD, 0);
        let (prev, here) = (g.world.player.prev_pos, g.world.player.pos);
        assert_ne!(prev, here);
        g.advance(now(&g) + TICK_US / 2, FORWARD, 0, 0, 0);
        assert_eq!(g.world.player.pos, here, "half a tick runs no tick");
        let half = g.alpha;
        assert!((half - ONE / 2).abs() < ONE / 100, "{half}");
    }

    #[test]
    fn the_look_speed_is_a_setting_held_in_its_range() {
        let mut g = game();
        g.act("look.speed", "25").unwrap();
        assert_eq!(g.world.aim.pct, 25);
        assert_eq!(g.kept.look_pct, 25);
        g.act("look.speed", "9000").unwrap();
        assert_eq!(g.world.aim.pct, 400);
        assert_eq!(
            g.act("look.speed", "fast"),
            Err("The look speed is a whole percentage, such as 100, not \"fast\".".into())
        );
    }

    #[test]
    fn a_software_renderer_draws_half_as_many_pixels_each_way() {
        let mut g = game();
        // 3840 × 2160 is 8 294 400 pixels; capped to 4 096 000 by
        // √(4 096 000 / 8 294 400) = 0.702 each way: 2695 × 1516
        assert_eq!(g.resize(1920, 1080, 200), [2695, 1516]);
        g.act("display.software", "true").unwrap();
        assert_eq!(g.pixels, (1920, 1080));
    }

    #[test]
    fn the_pause_hint_shows_for_the_first_six_seconds_of_the_duel() {
        let mut g = game();
        practice(&mut g);
        // (the knight held still, so no round ends in the six seconds)
        g.act("duel.pilot", "still").unwrap();
        assert!(g.hint_shown());
        frames(&mut g, 359, 0, 0);
        assert!(g.hint_shown());
        frames(&mut g, 2, 0, 0);
        assert!(!g.hint_shown());
    }
}
