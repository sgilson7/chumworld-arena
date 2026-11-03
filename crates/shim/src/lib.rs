//! tool_shim: moves values between the page and `tool_core::game`.
//!
//! Builder requirement: the shim decides nothing. Each function is one call
//! into the core and a conversion of its arguments or result. The core
//! speaks Q16.16 integers; the values WebGL wants as floats (the matrices,
//! the uniforms, the clear colour, mesh and instance values) are divided by
//! the core's own `ONE` here, and the page's millisecond clock and pixel
//! movement become the whole microseconds and hundredths of a pixel the
//! core expects. (CHUMWORLD's shim, commit d9cb5b5, less its water, sound,
//! save, map and figure calls.)
//! `crates/core/tests/boundary.rs` fails if a branch, a loop or the `?`
//! operator appears in this file.

use tool_core::{api, fx, game};
use wasm_bindgen::prelude::*;

const ONE: f32 = fx::ONE as f32;

fn floats(v: Vec<i32>) -> Vec<f32> {
    v.into_iter().map(|x| x as f32 / ONE).collect()
}

#[wasm_bindgen]
pub struct Game {
    inner: game::Game,
}

#[wasm_bindgen]
impl Game {
    /// The strings, the glossary, the look and a seed the page drew.
    #[wasm_bindgen(constructor)]
    pub fn new(strings: &str, glossary: &str, look: &str, seed: u32) -> Result<Game, JsValue> {
        game::Game::new(strings, glossary, look, seed)
            .map(|inner| Game { inner })
            .map_err(|e| JsValue::from_str(&e))
    }

    /// CSS size and device pixel ratio in; drawing-buffer size out.
    pub fn resize(&mut self, css_w: f64, css_h: f64, dpr: f64) -> Vec<i32> {
        self.inner
            .resize(css_w as i32, css_h as i32, (dpr * 100.0) as i32)
            .to_vec()
    }

    /// The page's clock in milliseconds, the held action bits, and the mouse
    /// and wheel movement in CSS pixels since the last frame.
    pub fn advance(&mut self, now_ms: f64, bits: u32, dx: f64, dy: f64, wheel: f64) {
        self.inner.advance(
            (now_ms * 1000.0) as i64,
            bits,
            (dx * 100.0) as i32,
            (dy * 100.0) as i32,
            (wheel * 100.0) as i32,
        )
    }

    /// Each depth layer's matrix, then the eye.
    pub fn view(&self) -> Vec<f32> {
        floats(self.inner.frame.view.clone())
    }

    pub fn instances(&self) -> Vec<f32> {
        floats(self.inner.frame.instances.clone())
    }

    pub fn draws(&self) -> Vec<u32> {
        self.inner.frame.draws.clone()
    }

    pub fn static_values(&self) -> Vec<f32> {
        floats(self.inner.scene.values.clone())
    }

    pub fn static_version(&self) -> u32 {
        self.inner.scene.version
    }

    pub fn uniforms(&self) -> Vec<f32> {
        floats(tool_core::sky::all_uniforms(&self.inner.look))
    }

    pub fn clear_color(&self) -> Vec<f32> {
        floats(self.inner.look.clear_color())
    }

    /// Every mesh to upload, by number, with its vertex count, as JSON.
    pub fn meshes(&self) -> String {
        api::meshes(&self.inner)
    }

    /// One mesh's vertex values.
    pub fn mesh(&self, id: u32) -> Vec<f32> {
        floats(api::mesh(&self.inner, id))
    }

    /// The view model, as JSON.
    pub fn ui(&self) -> String {
        api::ok(&api::ui(&self.inner))
    }

    pub fn ui_version(&self) -> u32 {
        self.inner.ui_version
    }

    /// T10: how red the view's edge is this frame, from 0 to 1.
    pub fn hurt(&self) -> f32 {
        self.inner.hurt() as f32 / 1000.0
    }

    /// The save's text, for the page to keep in the browser or download.
    pub fn save_text(&self) -> String {
        self.inner.save_text()
    }

    /// A screen, setting or flag: the new view model, or the error, as JSON.
    pub fn act(&mut self, name: &str, value: &str) -> String {
        api::act(&mut self.inner, name, value)
    }

    /// M5: the transport says a browser arrived.
    pub fn online_connected(&mut self, peer: u32) {
        self.inner.online_connected(peer)
    }

    /// M5: the transport says a browser left.
    pub fn online_left(&mut self, peer: u32) {
        self.inner.online_left(peer)
    }

    /// M5: text from a browser.
    pub fn online_message(&mut self, peer: u32, text: &str) {
        self.inner.online_message(peer, text)
    }

    /// M5: what the transport is to send, as JSON pairs of peer and text.
    pub fn online_outbox(&mut self) -> String {
        self.inner.online_outbox()
    }
}

/// Sizes, layouts, bits and pass states, as JSON.
#[wasm_bindgen]
pub fn numbers() -> String {
    api::numbers()
}
