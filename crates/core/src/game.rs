//! The game as the page sees it: the screens (start, duel, paused), the
//! clock that turns display frames into ticks, the frame the page draws, and
//! the one door for settings, screens and flags (`act`).
//!
//! CHUMWORLD's `game.rs` (commit d9cb5b5) with its menus, travel, sound,
//! save and detail levels taken out (R2.6). Each display frame the page
//! calls `advance` with its timestamp, the held action bits and the mouse
//! and wheel movement since the last frame, then reads the frame (`view`,
//! `instances`, `draws`, and `static_values` when `static_version` changes)
//! and, when `ui_version` changes, the view model (`api::ui`). The page does
//! no arithmetic: the core has already placed everything between the last
//! two ticks (`alpha`).

use crate::camera::{self, Aim};
use crate::fx::{self, Fx, ONE};
use crate::glossary::Entry;
use crate::input::{Input, Mouse, Pending};
use crate::look::Look;
use crate::page::Strings;
use crate::prim::MeshTable;
use crate::render::{self, Camera, Frame, Group, Scene, Shapes};
use crate::ticks::Clock;
use crate::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// the start screen: the title and Practice, over the ring
    Start,
    /// the duel
    Duel,
    Paused,
}

impl Screen {
    pub fn name(self) -> &'static str {
        match self {
            Screen::Start => "start",
            Screen::Duel => "duel",
            Screen::Paused => "paused",
        }
    }
}

/// The foe Practice puts in the ring: M3's hook (CA-Q13: A), the computer
/// fights as Knight B (`knight_b`, on the leader's tree until M2 gives the
/// computer's knight its own pilot), and the player is Knight A.
pub const PRACTICE_FOE: &str = "knight_b";
pub const PRACTICE_ME: &str = "knight_a";
/// The most pixels the drawing buffer may hold (2560 × 1600); a larger
/// screen draws at a lower resolution, scaled up by the browser.
pub const MAX_PIXELS: i64 = 2560 * 1600;
/// On a software renderer (no graphics card) the page draws at this share of
/// the screen's pixel density, percent.
pub const SOFTWARE_SCALE_PCT: i32 = 50;
/// How long the pause hint shows once the duel starts, ticks (6 s).
pub const HINT_TICKS: u64 = 360;
/// The start screen's camera drifts round the ring this fast, angle units
/// a tick (2° a second).
pub const TITLE_DRIFT: i32 = fx::TURN / 360 * 2 / 60;
/// And looks down this far, from this far back.
pub const TITLE_PITCH: i32 = -fx::deg(9);
pub const TITLE_BACK: Fx = fx::int(8);
/// The duel eases the camera in from the start screen's view to the
/// shoulder over this many ticks (0.75 s).
pub const ZOOM_TICKS: u64 = 45;

#[derive(Debug, Clone)]
pub struct Game {
    pub strings: Strings,
    pub glossary: Vec<Entry>,
    pub look: Look,
    pub clock: Clock,
    pub pending: Pending,
    pub world: World,
    pub screen: Screen,
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
    /// ticks of the duel since it first started, for the pause hint
    pub play_ticks: u64,
    /// ticks the start screen has shown, for its drifting camera
    pub title_ticks: u64,
    /// ticks since the duel began from the start screen, for the ease in
    pub zoom_ticks: u64,
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
        let world = World::new(seed as u64);
        let mut meshes = MeshTable::default();
        let groups = crate::ring::scene(&mut meshes);
        let mut g = Game {
            strings,
            glossary,
            look,
            clock: Clock::default(),
            pending: Pending::default(),
            world,
            screen: Screen::Start,
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
            zoom_ticks: ZOOM_TICKS,
            ui_version: 1,
        };
        g.draw();
        Ok(g)
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
        match self.screen {
            Screen::Duel => {
                self.pending.add(dx, dy, wheel);
                let hinted = self.hint_shown();
                // the lines over the duel (health, the cue) changing
                let duel = crate::player::hud_stamp(&self.world);
                let wound = crate::wounds::stamp(&self.world);
                for (k, b) in ticks.iter().enumerate() {
                    let m = if k == 0 {
                        self.pending.take()
                    } else {
                        Mouse::default()
                    };
                    self.world.step(&Input::from_page(*b, m));
                    self.play_ticks += 1;
                    self.zoom_ticks = (self.zoom_ticks + 1).min(ZOOM_TICKS);
                }
                if hinted != self.hint_shown()
                    || duel != crate::player::hud_stamp(&self.world)
                    || wound != crate::wounds::stamp(&self.world)
                {
                    self.ui_version += 1;
                }
                self.alpha = self.clock.alpha();
            }
            Screen::Start => {
                self.pending.clear();
                self.title_ticks += ticks.len() as u64;
                self.alpha = self.clock.alpha();
            }
            Screen::Paused => {
                // the world holds still; time passing while paused is
                // forgotten, so resuming runs no catch-up ticks
                self.pending.clear();
                self.clock.rest();
            }
        }
        self.draw();
    }

    pub fn hint_shown(&self) -> bool {
        self.screen == Screen::Duel && self.play_ticks < HINT_TICKS
    }

    /// The start screen's slow turn round the player.
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

    /// How far back the camera sits: the start screen's wide view, easing
    /// in to the shoulder (`camera::BACK`) once the duel begins.
    fn back(&self) -> Fx {
        if self.screen == Screen::Start {
            return TITLE_BACK;
        }
        let t = ((self.zoom_ticks as i64 * ONE as i64 + self.alpha as i64) / ZOOM_TICKS as i64)
            .min(ONE as i64) as Fx;
        let smooth = fx::mul(fx::mul(t, t), 3 * ONE - 2 * t);
        fx::lerp(TITLE_BACK, camera::BACK, smooth)
    }

    pub fn camera(&self) -> Camera {
        let ground = |x, z| self.world.ground(x, z);
        let feet = self.world.player.feet(self.alpha);
        if self.screen == Screen::Start {
            return camera::view_from(feet, self.title_aim(), self.back(), ground);
        }
        self.world.camera(self.alpha, self.pending.peek(), self.back())
    }

    /// Builds the frame the page draws next.
    fn draw(&mut self) {
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
            if self.screen == Screen::Start && s == Screen::Duel {
                // the duel opens looking where the start screen's camera had
                // drifted to, eases in from its distance, and Practice puts
                // the foe in the ring
                self.world.aim = self.title_aim();
                self.zoom_ticks = 0;
                if self.world.foes.is_empty() {
                    // the foe stands ahead along the opening heading
                    self.world.aim.yaw = crate::world::START_YAW;
                    // M3 hook: the player in Knight A's plate
                    crate::player::act(&mut self.world, "me", PRACTICE_ME).expect("a knight");
                    crate::player::act(&mut self.world, "spawn", PRACTICE_FOE)
                        .expect("the practice foe is in the pilots' book");
                }
            }
            self.screen = s;
            self.pending.clear();
            self.ui_version += 1;
            self.draw();
        }
    }

    /// The one door for screens, settings and flags. The error is the
    /// message the page shows.
    pub fn act(&mut self, name: &str, value: &str) -> Result<(), String> {
        match name {
            "screen.show" => match value {
                "start" => self.show(Screen::Start),
                "duel" => self.show(Screen::Duel),
                "paused" => self.show(Screen::Paused),
                other => return Err(format!("There is no screen \"{other}\".")),
            },
            // Esc, a lost pointer lock and a hidden tab pause the duel
            "key.escape" | "pointer.lost" | "page.hidden" => {
                if self.screen == Screen::Duel {
                    self.show(Screen::Paused);
                }
            }
            // a reader's look speed, percent (?look= on the page's address)
            "look.speed" => {
                let pct: i32 = value.trim().parse().map_err(|_| {
                    format!("The look speed is a whole percentage, such as 100, not \"{value}\".")
                })?;
                self.world.aim.pct = pct.clamp(camera::LOOK_PCT_MIN, camera::LOOK_PCT_MAX);
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
        vec![
            ("walk.speed", fx::decimal(crate::player::WALK, 1)),
            ("run.speed", fx::decimal(crate::player::RUN, 1)),
            ("ring.radius", (crate::world::RADIUS / ONE).to_string()),
            ("health.player", crate::health::PLAYER.to_string()),
        ]
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

    #[test]
    fn the_game_opens_on_the_start_screen_and_practice_goes_through_the_door() {
        let mut g = game();
        assert_eq!(g.screen, Screen::Start);
        assert!(g.world.foes.is_empty(), "no foe before Practice");
        g.act("screen.show", "duel").unwrap();
        assert_eq!(g.screen, Screen::Duel);
        assert_eq!(g.world.foes.len(), 1, "Practice puts one foe in the ring");
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

    /// The foe stands 6 m ahead of the player along the opening heading,
    /// both inside the ring, about 3 m either side of its middle.
    #[test]
    fn practice_stands_the_foe_six_metres_ahead_across_the_middle() {
        let mut g = game();
        // ten seconds of the start screen: its camera has drifted 20°
        frames(&mut g, 600, 0, 0);
        g.act("screen.show", "duel").unwrap();
        let (p, f) = (g.world.player.pos, g.world.foes[0].body.pos);
        let gap = p.dist(f);
        assert!((gap - fx::int(6)).abs() <= ONE / 100, "{gap}");
        for at in [p, f] {
            let r = fx::length(at.x, at.z);
            assert!((r - fx::int(3)).abs() <= ONE / 20, "{r}");
        }
    }

    #[test]
    fn escape_a_lost_pointer_and_a_hidden_tab_pause_the_duel_and_only_the_duel() {
        for name in ["key.escape", "pointer.lost", "page.hidden"] {
            let mut g = game();
            g.act(name, "").unwrap();
            assert_eq!(
                g.screen,
                Screen::Start,
                "{name} on the start screen changes nothing"
            );
            g.act("screen.show", "duel").unwrap();
            g.act(name, "").unwrap();
            assert_eq!(g.screen, Screen::Paused, "{name}");
        }
    }

    #[test]
    fn a_paused_world_holds_still_and_resuming_runs_no_catch_up() {
        let mut g = game();
        g.act("screen.show", "duel").unwrap();
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
            g.act("screen.show", "duel").unwrap();
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
        g.act("screen.show", "duel").unwrap();
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
        g.act("screen.show", "duel").unwrap();
        assert!(g.hint_shown());
        frames(&mut g, 359, 0, 0);
        assert!(g.hint_shown());
        frames(&mut g, 2, 0, 0);
        assert!(!g.hint_shown());
    }
}
