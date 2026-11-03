// The page draws what the core returns and decides nothing.
//
// CHUMWORLD's page (d9cb5b5) less its sound and figure; M2's screens and
// settings come from the core's view model like every panel, and the save
// is kept by web/save.js. It
// loads the data files, starts the core through the shim, and each display
// frame passes the clock, the held actions and the mouse and wheel movement
// in, then draws the frame that comes out (gl.js). The screen's panels, the
// health bars and the lines over the duel are built from the core's view
// model (`ui`) without knowing what any panel means; every button sends its
// action back through `act`, the core's one door. Esc, a lost pointer and a
// hidden tab are reported to the core as they happen; the core decides what
// each means.
import init, { Game, numbers } from "./pkg/tool_shim.js";
import { makeRenderer } from "./gl.js";
import { makeInput } from "./input.js";
import { makeSave } from "./save.js";

const el = (id) => document.getElementById(id);
const make = (tag, text, className) => {
  const node = document.createElement(tag);
  if (text !== undefined) node.textContent = text;
  if (className) node.className = className;
  return node;
};
const body = document.body;

let hints = [];

// A label whose text is a glossary term shows the definition on hover.
function mark(node) {
  const hint = hints.find((h) => h.term === node.textContent);
  if (hint) {
    node.dataset.term = hint.term;
    node.title = hint.definition;
  }
}

function ready() {
  el("loading").hidden = true;
  body.dataset.ready = "true";
}

async function main() {
  const strings = await (await fetch("data/strings.json")).text();
  el("loading-text").textContent = JSON.parse(strings).loading;
  const [glossary, look, controls, display, vertex, fragment] = await Promise.all([
    fetch("data/glossary.md").then((r) => r.text()),
    fetch("data/look.json").then((r) => r.text()),
    fetch("data/controls.json").then((r) => r.json()),
    fetch("data/display.json").then((r) => r.json()),
    fetch("data/shaders/scene.vert").then((r) => r.text()),
    fetch("data/shaders/scene.frag").then((r) => r.text()),
  ]);
  await init();
  const N = JSON.parse(numbers());

  // chance for this session, drawn once and handed to the core (rng.rs)
  const seed = new Uint32Array(1);
  crypto.getRandomValues(seed);
  let game;
  try {
    game = new Game(strings, glossary, look, seed[0]);
  } catch (e) {
    el("message").textContent = String(e);
    ready();
    return;
  }

  const canvas = el("view");
  const meshes = JSON.parse(game.meshes());
  const renderer = makeRenderer(canvas, N, { vertex, fragment, software: display.software }, meshes, (id) => game.mesh(id));
  if (!renderer) {
    body.dataset.webgl = "none";
    el("message").textContent = JSON.parse(game.ui()).ok.no_webgl;
    ready();
    return;
  }
  body.dataset.webgl = "2";
  body.dataset.software = String(renderer.software);
  if (renderer.software) game.act("display.software", "true");
  renderer.setUniforms(game.uniforms(), game.clear_color());

  let ui = null;
  const input = makeInput(canvas, controls, N.actions, () => {});
  // the browser's save and the save file (web/save.js); the core decides
  // what is in it and when it is written
  const save = makeSave();
  const params = new URLSearchParams(location.search);

  // Sends one action through the core's door and draws what comes back.
  // `gesture`: the act came from a key or click, so if the core now wants
  // the pointer the page may take it again.
  function act(name, value, gesture) {
    const result = JSON.parse(game.act(name, value));
    if (result.error) {
      el("message").textContent = result.error;
      return;
    }
    el("message").textContent = "";
    show(result.ok);
    if (gesture && result.ok.capture && !input.engaged()) input.engage();
  }

  function button(a) {
    const node = make("button", a.label);
    node.disabled = !a.enabled;
    node.dataset.action = a.name;
    node.dataset.value = a.value;
    node.addEventListener("click", () => {
      // the browser grants the pointer only inside a click
      if (a.engage) input.engage();
      node.blur();
      // a button that asks for a file sends the file's text; one that names
      // a box sends the box's text (M5)
      if (a.file) save.pick((text) => act(a.name, text));
      else if (a.from) act(a.name, el(a.from) ? el(a.from).value : "");
      else act(a.name, a.value);
    });
    return node;
  }

  function panel(p) {
    const box = make("section", undefined, "panel");
    box.id = p.id;
    if (p.heading) box.append(make("h1", p.heading));
    if (p.text) box.append(make("p", p.text));
    // M6: the ladder's rank cards
    if (p.cards) {
      const grid = make("div", undefined, "cards");
      grid.append(...p.cards.map(card));
      box.append(grid);
    }
    (p.settings || []).forEach((s) => {
      const row = make("div", undefined, "setting");
      row.id = `setting-${s.id}`;
      const value = make("output", s.value);
      row.append(make("span", s.label, "label"), value, ...s.actions.map(button));
      box.append(row);
    });
    (p.fields || []).forEach((f) => box.append(field(f)));
    (p.actions || []).forEach((a) => box.append(button(a)));
    if (p.note) box.append(make("p", p.note, "note"));
    if (p.rows) {
      if (p.rows_heading) box.append(make("h2", p.rows_heading));
      const table = make("table", undefined, "controls");
      for (const r of p.rows) {
        const row = table.insertRow();
        row.insertCell().append(make("kbd", r.key));
        row.insertCell().append(make("span", r.does));
      }
      box.append(table);
    }
    return box;
  }

  // M6: one rank card: the rank's letter, the opponent, its state in words,
  // and its marks drawn as diamonds, filled when won, with the count in
  // words for a screen reader and a hover (BRIEF.md, Accessibility). The
  // whole card opens the rung's dossier.
  function card(c) {
    const node = make("button", undefined, "card");
    node.id = `card-${c.id}`;
    node.dataset.state = c.state;
    node.dataset.marks = String(c.marks);
    node.dataset.action = c.action.name;
    node.dataset.value = c.action.value;
    node.setAttribute("aria-label", `${c.action.label}: ${c.name}. ${c.text}. ${c.marks_text}.`);
    const marks = make("span", undefined, "marks");
    marks.title = c.marks_text;
    for (let i = 0; i < c.marks_most; i += 1) marks.append(make("span", undefined, i < c.marks ? "mark won" : "mark"));
    node.append(make("span", c.rank, "rank"), make("span", c.name, "name"), make("span", c.text, "state"), marks);
    node.addEventListener("click", () => {
      node.blur();
      act(c.action.name, c.action.value);
    });
    return node;
  }

  // M5: a box of text: a code to copy (fixed, marked ready once it holds
  // one) or a box to paste or type a code into.
  function field(f) {
    const label = make("label", undefined, "field");
    const box = make(f.line ? "input" : "textarea");
    box.id = f.id;
    box.value = f.value;
    box.readOnly = f.fixed;
    box.spellcheck = false;
    box.autocomplete = "off";
    box.dataset.ready = String(Boolean(f.fixed && f.value));
    if (f.fixed) box.addEventListener("focus", () => box.select());
    label.append(make("span", f.label, "label"), box);
    return label;
  }

  // M5: online play. Nothing for it loads until the core gives the
  // transport its first command (Host, Join or Pasted codes); then the one
  // page file that reaches the transport is imported (web/online.js).
  let online = null;
  let onlineLoading = null;
  function follow(o) {
    el("online-sums").textContent = o.sums;
    if (online) online.follow(o);
    else if (o.cmds.length && !onlineLoading) {
      onlineLoading = import("./online.js").then((m) => {
        online = m.makeOnline(game, (name, value) => act(name, value));
        online.follow(ui.online);
      });
    }
  }

  // One fighter's health: who, the words, and a bar the core's share fills.
  function bar(b) {
    let node = el(`bar-${b.id}`);
    if (!node) {
      node = make("div", undefined, "bar");
      node.id = `bar-${b.id}`;
      const head = make("header");
      head.append(make("span", "", "who"), make("span", "", "text"));
      const meter = make("progress");
      meter.max = N.bar_whole;
      node.append(head, meter);
      el("bars").append(node);
    }
    node.querySelector(".who").textContent = b.who;
    node.querySelector(".text").textContent = b.text;
    node.querySelector("progress").value = b.share;
    node.querySelector("progress").setAttribute("aria-label", `${b.who} ${b.label} ${b.text}`);
    node.dataset.share = String(b.share);
    stamina(node, b);
    return node;
  }

  // M4: the fighter's stamina under its health, while the core sends it: a
  // thinner notched bar, its words, and a hatched bar and the word when
  // winded (BRIEF.md, Accessibility).
  function stamina(node, b) {
    let box = node.querySelector(".stamina");
    if (!b.stamina) {
      if (box) box.remove();
      return;
    }
    if (!box) {
      box = make("div", undefined, "stamina");
      const head = make("header");
      head.append(make("span", "", "label"), make("span", "", "text"));
      const meter = make("progress");
      meter.max = N.bar_whole;
      box.append(head, meter);
      node.append(box);
    }
    const m = b.stamina;
    box.querySelector(".label").textContent = m.label;
    box.querySelector(".text").textContent = m.text;
    box.querySelector("progress").value = m.share;
    box.querySelector("progress").setAttribute("aria-label", `${b.who} ${m.label} ${m.text}`);
    box.classList.toggle("winded", m.winded);
    node.dataset.stamina = String(m.share);
  }

  // Builds the screen from the core's view model.
  function show(u) {
    const changed = !ui || ui.screen !== u.screen;
    ui = u;
    hints = u.hints;
    document.title = u.title;
    body.dataset.screen = u.screen;
    // (what a reader has typed or pasted into a box stays through a redraw)
    const typed = new Map([...el("overlay").querySelectorAll("input, textarea")].filter((n) => !n.readOnly).map((n) => [n.id, n.value]));
    el("overlay").replaceChildren(...u.panels.map(panel));
    typed.forEach((value, id) => {
      if (el(id) && !el(id).readOnly) el(id).value = value;
    });
    el("overlay").hidden = u.panels.length === 0;
    el("bars").replaceChildren(...u.bars.map(bar));
    for (const item of u.hud) {
      let node = el(`hud-${item.id}`);
      if (!node) {
        node = make("p", item.text, "hud-item");
        node.id = `hud-${item.id}`;
        el("hud").append(node);
      }
      node.textContent = item.text;
      node.classList.toggle("shown", item.shown);
    }
    document.querySelectorAll("#overlay h1, #overlay h2, #overlay td span, #overlay button, #overlay .label").forEach(mark);
    if (!u.capture) input.release();
    save.show(u.save, () => game.save_text());
    // a screen of buttons opens with its first one focused, for the keyboard
    const first = el("overlay").querySelector("button");
    if (changed && first) first.focus({ preventScroll: true });
    follow(u.online);
  }

  // Esc, a lost pointer and a hidden tab: the core decides what they mean.
  window.addEventListener("keydown", (e) => {
    if (e.code === "Escape") act("key.escape", "");
  });
  let locked = false;
  document.addEventListener("pointerlockchange", () => {
    const now = Boolean(document.pointerLockElement);
    // only a lock that was held and is now lost: a browser that grants the
    // lock and drops it at once plays on without it
    if (locked && !now) act("pointer.lost", "");
    locked = now;
  });
  // M5: a page closing or navigated away leaves an online duel first
  window.addEventListener("pagehide", () => act("page.leaving", ""));
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) act("page.hidden", "");
  });
  // in the duel, a click on the game takes the pointer again if it was let go
  canvas.addEventListener("mousedown", () => {
    if (ui && ui.capture && !input.engaged()) input.engage();
  });

  // a reader's look speed from the address (?look=<percent>), and test
  // settings, each through the core's door like any setting: ?sticks=1
  // shows the physics view, ?act=name:value any act
  const lookSpeed = params.get("look");
  if (lookSpeed) act("look.speed", lookSpeed);
  if (params.get("sticks")) act("duel.sticks", "on");
  for (const asked of params.getAll("act")) {
    const [name, value] = asked.split(":");
    act(name, value || "");
  }
  // M5: this build's name for the online hello, from this module's own
  // stamped address (scripts/build.sh), so two builds are told apart
  act("online.build", new URL(import.meta.url).searchParams.get("v") || "dev");
  // this page's address, for the host's join link
  act("online.address", location.origin + location.pathname);
  // the save this browser keeps, if any, read by the core before anything shows
  act("save.stored", save.stored());
  // opened from a join link (?join=CODE): the core shows Join with the code
  const joining = params.get("join");
  if (joining) act("online.link", joining);
  show(JSON.parse(game.ui()).ok);

  function resize() {
    const [w, h] = game.resize(canvas.clientWidth, canvas.clientHeight, window.devicePixelRatio || 1);
    renderer.resize(w, h);
  }
  window.addEventListener("resize", resize);
  resize();

  let staticVersion = 0;
  let shownHurt = -1;
  let uiVersion = 0;
  function frame(now) {
    const { bits, dx, dy, wheel } = input.read();
    game.advance(now, bits, dx, dy, wheel);
    if (game.static_version() !== staticVersion) {
      staticVersion = game.static_version();
      renderer.setStatic(game.static_values());
    }
    renderer.frame(game.view(), game.instances(), game.draws());
    // T10: the view's edge reddened by a blow taken, as the core says
    const hurt = game.hurt();
    if (hurt !== shownHurt) {
      shownHurt = hurt;
      el("hurt").style.opacity = String(hurt);
    }
    if (online) online.pump();
    if (game.ui_version() !== uiVersion) {
      uiVersion = game.ui_version();
      show(JSON.parse(game.ui()).ok);
    }
    if (body.dataset.ready !== "true") ready();
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
}

main();
