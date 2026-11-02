# Glossary

The source of truth for what each word in this tool means. Strings, tests, the plan and the README use a term in the sense given here. The page shows a definition when a reader hovers the term.

Write each entry as a `### Term` heading, then one or two plain sentences, then an optional `Not:` line listing words that must not be used for it. A number a rule decides is a `{placeholder}` the core fills, so the glossary cannot disagree with the rules. Keep entries in alphabetical order. `crates/core/tests/copy.rs` checks the format, the placeholders and the `Not:` words; `TONE.md` governs the prose.

<!--
An example entry. It sits inside this comment, so it does not count.

### Spout

Where slush comes out. Each line has up to {line.spouts} spouts.

Not: tap, dispenser, nozzle
-->

### Arena

The stone ring a duel is fought in: a flat floor {ring.radius} metres from its middle to the wall that keeps both fighters in.

Not: colosseum, pit, Armored Core, Dark Souls, Chivalry

### Arm

The simulated arm that holds the sword: shoulder, elbow and hand joined by sticks the core moves a tick at a time. The mouse steers its hand; the arm decides how fast it gets there.

Not: rig, skeleton

### Bandit

A foe the computer fights with a short sword, through the same input a player gives.

Not: bot, enemy AI, NPC

### Black helm

The knight in blackened plate, a bucket helm and wide shoulders, with a torn cloak behind: the player's knight in Practice. A working label until the person names it.

Not: Black Knight, black knight

### Chop

A cut straight down from above the head: hold the left button and move the mouse down.

Not: overhead slash

### Cut

What the sword does while the left button is held and the mouse moves: the way the mouse goes picks a level cut, a chop, a rising cut, a diagonal cut or a thrust.

Not: slash, swipe, combo

### Diagonal cut

A cut along a slant, from high on one side to low on the other or the other way: hold the left button and move the mouse clearly on a slant.

### Dodge-step

A short quick step to one side with Space and a movement key. You cannot dodge-step again straight away.

Not: roll

### Draw

Taking the sword out of its sheath with X. Drawn, the mouse moves the sword hand and the view holds on the foe.

Not: unsheathe

### Duel

One fight between two fighters in the arena, each moving only through the keys and mouse of its input.

Not: match, bout, deathmatch

### Gap

An opening in plate where a blow lands whole: the armpit, the visor's slit and the back of the knee.

Not: weak spot, chink

### Health

How much harm a body can take. A blade takes more the faster it moves when it hits. You start with {health.player}.

Not: HP, hit points

### Hedge knight

The ladder's fourth opponent from the foot (rung D): a knight in half plate under a long green surcoat torn at the hem, cloth hose below the knee, and a pointed helm over a mail collar. A working label until the person names it.

Not: Ser Duncan

### Ivory crown

The knight in pale plate with gilt, a crown of spikes and a skirt of plate burnt at the hem: the computer's knight in Practice. A working label until the person names it.

Not: Ivory King, Burnt Ivory King

### Ladder

The single-player run of arena fights against harder opponents in turn, each with its own costume.

Not: campaign, tower, story mode

### Level cut

A cut across from one side to the other with the blade held flat: hold the left button and move the mouse sideways. It runs high, at the middle or low, by where the sword hand was.

Not: horizontal slash

### Mail

The rings of steel under plate. A part whose plate has broken off shows its mail, and a blow there lands whole.

Not: chainmail, chain mail

### Parry

Turning a blow aside with your own blade: the blades meet and neither reaches a body. It is the arm and blade meeting, not a button.

Not: deflect

### Plate

The armour a knight wears: shaped steel over the body, with gaps at the joints. It takes most of a cut and a little of a thrust, wears with each blow it takes, and breaks off once battered enough. A part in plate is never cut off.

Not: platemail, plate mail

### Practice

A duel against the computer, from the start screen.

Not: training mode, tutorial

### Rising cut

A cut from low in front up over the shoulder: hold the left button and move the mouse up.

### Run

Moving at {run.speed} metres a second while Shift is held with a movement key, sword sheathed.

Not: sprint

### Sellsword

The ladder's second opponent from the foot (rung E): a hired fighter in a studded leather coat with steel plates inside, steel on the forearms, a helm with a tail and a chin plate, and one big plate on the right shoulder only. A working label until the person names it.

Not: Sellsword Twinblades, Sellsword set

### Squire

The ladder's first opponent (rung F): a young fighter in a padded quilted coat, a mail hood round an open face and a broad-brimmed steel hat, his only plate. A working label until the person names it.

### Team deathmatch

Two teams of up to five fighters each in one arena.

Not: TDM

### Thrust

A straight stab ahead: hold the left button and turn the wheel away from you.

Not: lunge, poke

### Walk

Moving at {walk.speed} metres a second with W, A, S or D, the way the camera faces.
