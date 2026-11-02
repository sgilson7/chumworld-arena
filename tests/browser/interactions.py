"""The project's own browser interactions, run by check.py in every engine.

check.py has already loaded the page, waited for it to report ready, and let
it idle. Drive the duel here the way a person would, and assert what the page
shows with `check(condition, "what this shows")`.

Expected values are worked out by hand from the brief and the data (C4),
never copied from the tool's own output: the player starts with 100 health
(health.rs, PLAYER) and Practice's foe, Knight B, with 100
(data/pilots.json, knight_b; M3). Waits are on
selectors and data-* attributes the page sets, never on a fixed time.
"""

import json
import pathlib

STRINGS = json.loads((pathlib.Path(__file__).resolve().parents[2] / "data" / "strings.json").read_text())


def run(page, check, engine):
    check(page.get_attribute("body", "data-screen") == "start", "the core's start screen is shown")
    # (the start screen sets its heading in capitals with CSS, so case is not compared)
    check(page.inner_text("#start h1").lower() == STRINGS["title"].lower(), "the title from data/strings.json is drawn")
    check(page.inner_text("#message") == "", "no error message is shown")
    check(page.get_attribute("body", "data-webgl") == "2", "the duel draws with WebGL 2")
    button = page.locator("#start button")
    check(button.inner_text().strip().lower() == STRINGS["practice"].lower(), "the start screen offers Practice")
    button.click()
    page.wait_for_selector('body[data-screen="duel"]', timeout=10000)
    page.wait_for_selector("#bar-you", timeout=10000)
    check(page.inner_text("#bar-you .who") == STRINGS["who_you"], "the player's bar is named")
    check(page.inner_text("#bar-you .text") == "100 / 100", "the player starts whole: 100 of 100")
    check(page.inner_text("#bar-foe1 .who") == STRINGS["who_knight_b"], "the foe's bar names Knight B")
    check(page.inner_text("#bar-foe1 .text") == "100 / 100", "Knight B starts whole: 100 of 100")
    check(page.get_attribute("#bar-foe1", "data-share") == "10000", "Knight B's bar is full")
    page.keyboard.press("Escape")
    page.wait_for_selector('body[data-screen="paused"]', timeout=10000)
    check(page.inner_text("#paused h1") == STRINGS["paused"], "Esc pauses the duel")
