# Brief

Stage 1. The person's own words come first; the rest can be written by the agent from them and read by the person. `C1 Frame the problem and set the rules`, `B1 Establish context`.

## The description, word for word

> get yourself familiarized with the builder repository and clone it from https://github.com/sgilson7/builder-setting into this repo then remove the .git. You will be using builder setting to help constrain and direct your actions as you help act as as a video game  creator and co-designer / developer, with the goal of deploying it to a github pages so its freely browser accessible, with an emphasis on optimization for running in the browser.  Please also see https://sgilson7.github.io/projects/ for examples of other browser based tools / games built in builder like the PDF redactor, youtube to mp3, long time coming, extremely lifelike fishing  etc. Also see ptonkle.github.io for games built using builder by my good friend. First thing is I want you to do is interpret builder, and write an analysis of how you will use it to help you develop this game, and write into the first tab of the design journal, the design journal process is described in builder; when you write the design journal, use the local ape-race directories design journal as a representative of a good journaling process. you may access the other builder made games within the documents directory on this host machine.  The game I want is as follows: I want the same combat as chumworld (another builder made game by sgilson7), but in a first person, emphasising one on one swordplay like chivalry, but first focusing on a single player 1v1 campaign in the style of vagrancy and armored core 6 vr arena missions to get the combat flow working. in the end, I want the game to support up to 5v5 team death match. I want the art style to be a little grittier and more realistic, so look at the costume design flow of ape race and I want to use that for designing character assets.  start by creating a tab in the design log on how you will use builder to make this game. then make the 2nd tab your intrepretation of the brief, then the third is a synthesis of the brief into one coherent game with mockups, flow diagrams, menus, gameplay mockups, meta game progression mockups, design decisions for the designer to make. the MVP you are building a mockup of should be an online supported, 1v1 sword duel in first person in a circular arena, using the chumworld swordplay merged with chivalry, and the there should be two different character models that both have havy platemail on, and should invoke the black knights of dark souls or the burnt ivory king of dark souls 2 or the pursuer. for art style, use protection of the spheres, chumworld, armored core 6, and chivalry as your design inspirations for characters, weapons, costume design, etc.

## The tool

A first-person sword-duel game for a desktop browser, served from GitHub Pages. Its combat is CHUMWORLD's (a physics-simulated arm steered by the mouse, WASD for the feet), seen from inside the helmet and paced like Chivalry. The first release (the MVP) is one 1v1 duel in a circular arena, against the computer or against another browser online, between two heavily armoured knights. Later releases add a single-player ladder of arena fights in the manner of Vagrancy's arena and Armored Core 6's simulator missions, and then team deathmatch up to 5v5. The design journal's three opening pages read Builder, read this description back point by point, and put the whole game together with mockups; the questions there (CA-Q1 …) settle what this paragraph leaves open.

## Done means

Draft conditions for the MVP, to be settled with the person through the journal's questions before `PLAN.md`.

- The game opens to a menu with Practice (against the computer), Host and Join; both lead to the same 1v1 duel in a circular arena.
- The view is first person from inside the helmet, with the player's own arm and sword visible; the mouse steers the sword hand through CHUMWORLD's arm model and WASD moves the feet.
- A hit is decided by blade contact with the body, using CHUMWORLD's damage rules as the starting point, adjusted for plate armour; a parry is a blade meeting a blade.
- Two knight models exist, each in full heavy plate, distinct in silhouette at duel distance, built through the Ape Race costume flow (sheet, model in code, in-game turnaround shots, review).
- Two browsers can meet by room code or pasted code (Builder's online kit) and fight one duel to a result; loading and idling make no off-origin request.
- A duel against the computer runs to a result with an opponent that uses the same arm and body rules as the player.
- The game holds its frame rate on the person's machine (target in `MEASUREMENTS.md`), and drops detail on a software renderer.
- `./scripts/check.sh` passes, and the game is published on GitHub Pages once the person grants it.

## Decisions that stay with the person

- The game's name, and the names of knights, arenas and opponents (agent-generated lists are worklist rows).
- How far the combat departs from CHUMWORLD's (feel is the person's judgement from play).
- Whether the game keeps anything between visits. Answered 2026-11-02 (CA-Q4: C): one save in the browser for settings and the knight's look from the MVP on, and the ladder's progress from M6; the same save can be written to and read from a JSON file. It holds game state only, nothing about the person.
- The look; whether a picture matches the named inspirations is the person's judgement. No asset, likeness or name from Dark Souls, Chivalry or Armored Core is copied; they are read for silhouette, mood and proportion only.
- Any third-party asset, sound or likeness.
- When and where the game is published.

## Accessibility

Team, health and stamina are shown with shape, pattern or text as well as colour. Input: keyboard and mouse on a desktop browser with pointer lock and its fallback. Phones are out of scope unless the person says otherwise.

## Style

Grittier and more realistic than CHUMWORLD, drawing on Protection of the Spheres, CHUMWORLD, Armored Core 6 and Chivalry, as read in the journal's synthesis page. The 2D art pipeline's default pack is not used for the game.

## Words

Terms from the description: *duel*, *arm* (the simulated arm), *arena*, *plate* (armour), *parry*, *team deathmatch*, *ladder* (the single-player campaign of arena fights). They go into `GLOSSARY.md` before any string uses them.

## Autonomy

The person's grant, word for word (2026-11-02), after answering CA-Q1 to CA-Q16 in the journal (CA-Q2: B, publish):

> alright ive answered all the questions, continue and build the mvp, dont stop till the mvp is deployed, i give explicit approval to deploy, then once its deployed get the ladder built with a unique costume per enemy, then deploy that too

The plan is approved in advance. The agent may create the public repository `sgilson7/chumworld-arena`, push to it, and publish it to GitHub Pages at `https://sgilson7.github.io/chumworld-arena/` with `./scripts/publish.sh` once `./scripts/check.sh` passes: first the MVP (M0–M5), then the ladder (M6) with a unique costume for each opponent. The repository name is the agent's choice, since the person named none (R1.2). Where this brief is silent, the agent decides, records the decision in `SECOND-ORDER.md`, and continues. The agent may not loosen a Builder requirement or a boundary test, and may not add a server, an account, analytics or any off-origin request beyond the declared online feature (Builder's online kit, loaded only on Host or Join). The page may keep one save in the browser (settings, the knight's look, and from M6 the ladder's progress), and write and read the same save as a JSON file (CA-Q4: C). The work may be split among subagents bound by this project's `CLAUDE.md`; the main agent coordinates, reviews and keeps the journal.

Later grants, word for word:

- 2026-11-03, after the animation pass (T10) was committed and held: "deploy as is, add all changes to design log". Published from `734cc61`.
- 2026-11-03, after answering CA-Q17 to CA-Q23 ("i answered the questions, go ahead and build it"): "ok, deploy in between each major change, so i want to be able to try the beat aside live after its ready while you work on watch mode". Each major change of T11 and T12 is published once `check.sh` passes.

## Journal

`default`

## Allowed core dependencies

`serde`, `serde_json` (reference default).
