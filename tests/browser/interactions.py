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
    want = [STRINGS[k].lower() for k in ("practice", "ladder", "watch", "host", "join", "pasted_codes", "settings")]
    check(labels == want, f"the menu offers Practice, Ladder, Watch, Host, Join, Pasted codes and Settings {labels}")
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

    # M6, T11: the campaign. Four regions (ladder.rs REGIONS), thirteen
    # battles whose seven ranks run F to S, only F open in a new save and
    # only the Yard's region open; F's dossier offers Fight, S's does not;
    # the woodsman's names the axe it carries; F's duel is against
    # the squire, whose bar names it (data/strings.json who_squire) and who
    # starts with 100 health (data/pilots.json squire)
    page.click('#paused button[data-value="menu"]')
    page.wait_for_selector('body[data-screen="menu"]', timeout=10000)
    page.click('#menu button[data-value="ladder"]')
    page.wait_for_selector('body[data-screen="ladder"]', timeout=10000)
    regions = [n.get_attribute("data-open") for n in page.query_selector_all("#ladder .region")]
    check(regions == ["true", "false", "false", "false"], f"four regions, the Yard open {regions}")
    ranks = [n.inner_text().strip() for n in page.query_selector_all("#ladder .card .rank") if n.inner_text().strip()]
    check(ranks == ["F", "E", "D", "C", "B", "A", "S"], f"the seven ranks F to S among the battles {ranks}")
    states = [n.get_attribute("data-state") for n in page.query_selector_all("#ladder .card")]
    check(states == ["open"] + ["locked"] * 12, f"only rank F is open in a new save {states}")
    page.click("#card-woodsman")
    page.wait_for_selector('body[data-screen="dossier"]', timeout=10000)
    check(STRINGS["weapon_axe"] in page.inner_text("#dossier"), "the woodsman's dossier names the axe he carries")
    page.click('#dossier button[data-value="ladder"]')
    page.wait_for_selector('body[data-screen="ladder"]', timeout=10000)
    check(len(page.query_selector_all("#card-squire .mark")) == 2, "a card draws room for two marks")
    check(len(page.query_selector_all("#ladder .mark.won")) == 0, "no mark is won yet")
    page.click("#card-mirror")
    page.wait_for_selector('body[data-screen="dossier"]', timeout=10000)
    check(page.inner_text("#dossier h1") == STRINGS["who_mirror"], "rank S's dossier names the mirror")
    check(page.is_disabled('#dossier button[data-action="ladder.fight"]'), "a locked rung offers no fight")
    page.click('#dossier button[data-value="ladder"]')
    page.wait_for_selector('body[data-screen="ladder"]', timeout=10000)
    page.click("#card-squire")
    page.wait_for_selector('body[data-screen="dossier"]', timeout=10000)
    check(STRINGS["dossier_squire"] in page.inner_text("#dossier"), "the squire's dossier says how he fights")
    page.click('#dossier button[data-action="ladder.fight"]')
    page.wait_for_selector('body[data-screen="duel"]', timeout=10000)
    page.wait_for_selector("#bar-foe1", timeout=10000)
    check(page.inner_text("#bar-foe1 .who") == STRINGS["who_squire"], "rank F's duel is against the squire")
    check(page.inner_text("#bar-foe1 .text") == "100 / 100", "the squire starts whole: 100 of 100")
    page.keyboard.press("Escape")
    page.wait_for_selector('body[data-screen="paused"]', timeout=10000)
    page.click('#paused button[data-value="ladder"]')
    page.wait_for_selector('body[data-screen="ladder"]', timeout=10000)
    check(True, "leaving a ladder duel goes back to the ladder")

    # T11: watch mode. From the menu, two fighters duel: the line over the
    # fight counts ticks, the shown fighter's tree is drawn with one row per
    # rule of the veteran's tree in data/pilots.json, Pause stops the count,
    # and a step moves it on by exactly one.
    trees = json.loads((pathlib.Path(__file__).resolve().parents[2] / "data" / "pilots.json").read_text())["trees"]
    page.click('#ladder button[data-value="menu"]')
    page.wait_for_selector('body[data-screen="menu"]', timeout=10000)
    page.click('#menu button[data-value="watch"]')
    page.wait_for_selector('body[data-screen="watch"]', timeout=10000)
    page.wait_for_selector("#watch-tree .rule", timeout=10000)
    check(len(page.query_selector_all("#watch-tree .rule")) == len(trees["veteran"]), "the veteran's tree is drawn, a row a rule")
    check(page.inner_text("#watch-tree h2").startswith("The veteran"), "the tree is the left fighter's, the veteran")
    page.click('#watch button[data-action="watch.speed"][data-value="1"]')
    page.wait_for_function("() => /Tick (\\d+)/.exec(document.querySelector('#watch-line').textContent)[1] > 60", timeout=20000)
    check(True, "the fight runs: past tick 60")
    page.click('#watch button[data-action="watch.pause"]')
    page.wait_for_function("() => document.querySelector('#watch-line').textContent.endsWith('Paused')", timeout=10000)
    tick = lambda: int(page.inner_text("#watch-line").split("·")[0].split()[1])
    t0 = tick()
    page.wait_for_timeout(300)
    check(tick() == t0, "paused, the count holds")
    page.click('#watch button[data-action="watch.step"]')
    page.wait_for_function(f"() => document.querySelector('#watch-line').textContent.startsWith('Tick {t0 + 1} ')", timeout=10000)
    check(True, "a step moves it on by one tick")
    check(len(page.query_selector_all("#watch-tree .check")) > 0, "the checks are drawn")
