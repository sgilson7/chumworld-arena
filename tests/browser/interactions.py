"""The project's own browser interactions, run by check.py in every engine.

check.py has already loaded the page, waited for it to report ready, and let
it idle. Drive the duel here the way a person would, and assert what the page
shows with `check(condition, "what this shows")`.

Expected values are worked out by hand from the brief and the data (C4),
never copied from the tool's own output: the player starts with 100 health
(health.rs, PLAYER) and the computer's knight, in Knight B's plate (M3), with
100 (data/pilots.json, knight); the field of view opens at 105 and steps by
one (world.rs, game.rs). Waits are on
selectors and data-* attributes the page sets, never on a fixed time.
"""

import json
import pathlib

STRINGS = json.loads((pathlib.Path(__file__).resolve().parents[2] / "data" / "strings.json").read_text())


def run(page, check, engine):
    check(page.get_attribute("body", "data-screen") == "title", "the core's title screen is shown")
    # (the title sets its heading in capitals with CSS, so case is not compared)
    check(page.inner_text("#title h1").lower() == STRINGS["title"].lower(), "the title from data/strings.json is drawn")
    check(page.inner_text("#message") == "", "no error message is shown")
    check(page.get_attribute("body", "data-webgl") == "2", "the duel draws with WebGL 2")
    page.click("#title button")
    page.wait_for_selector('body[data-screen="menu"]', timeout=10000)
    labels = [b.inner_text().strip().lower() for b in page.query_selector_all("#menu button")]
    want = [STRINGS[k].lower() for k in ("practice", "host", "join", "pasted_codes", "settings")]
    check(labels == want, f"the menu offers Practice, Host, Join, Pasted codes and Settings {labels}")
    # M5: Host and Join are open; this check never presses either, so it still
    # sees no WebSocket and no request off the origin (CA-4, online_fence.rs)
    check(page.is_enabled('#menu button[data-action="online.host"]'), "Host is open")
    check(STRINGS["online_menu_note"] in page.inner_text("#menu"), "the menu says how online play meets")

    # the settings: the field of view up a degree, kept in this browser's save
    page.click('#menu button[data-value="settings"]')
    page.wait_for_selector('body[data-screen="settings"]', timeout=10000)
    check(page.inner_text("#setting-fov output") == "105°", "the field of view opens at 105°")
    page.click('#setting-fov button[data-value="+"]')
    page.wait_for_function("document.querySelector('#setting-fov output').textContent === '106°'", timeout=10000)
    check(True, "one step up makes it 106°")
    stored = page.evaluate("() => { try { return window.localStorage.getItem('chumworld-arena.save') || ''; } catch (e) { return 'blocked'; } }")
    check(stored == "blocked" or '"fov": 106' in stored, "the change is kept in the browser's one save")
    page.click('#settings button[data-value="back"]')
    page.wait_for_selector('body[data-screen="menu"]', timeout=10000)

    page.click('#menu button[data-value="duel"]')
    page.wait_for_selector('body[data-screen="duel"]', timeout=10000)
    page.wait_for_selector("#bar-you", timeout=10000)
    check(page.inner_text("#bar-you .who") == STRINGS["who_you"], "the player's bar is named")
    check(page.inner_text("#bar-you .text") == "100 / 100", "the player starts whole: 100 of 100")
    check(page.inner_text("#bar-foe1 .who") == STRINGS["who_knight_b"], "the foe's bar names Knight B")
    check(page.inner_text("#bar-foe1 .text") == "100 / 100", "Knight B starts whole: 100 of 100")
    check(page.get_attribute("#bar-foe1", "data-share") == "10000", "Knight B's bar is full")
    check(page.inner_text("#hud-round").startswith("Round 1"), "the round's line names round 1")
    page.keyboard.press("Escape")
    page.wait_for_selector('body[data-screen="paused"]', timeout=10000)
    check(page.inner_text("#paused h1") == STRINGS["paused"], "Esc pauses the duel")
