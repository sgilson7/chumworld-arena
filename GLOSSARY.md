# Glossary

The source of truth for what each word in this tool means. Strings, tests, the plan and the README use a term in the sense given here. The page shows a definition when a reader hovers the term.

Write each entry as a `### Term` heading, then one or two plain sentences, then an optional `Not:` line listing words that must not be used for it. A number a rule decides is a `{placeholder}` the core fills, so the glossary cannot disagree with the rules. Keep entries in alphabetical order. `crates/core/tests/copy.rs` checks the format, the placeholders and the `Not:` words; `TONE.md` governs the prose.

<!--
An example entry. It sits inside this comment, so it does not count.

### Spout

Where slush comes out. Each line has up to {line.spouts} spouts.

Not: tap, dispenser, nozzle
-->

### Aim help

A setting that keeps the foe in view: none, or a soft pull that turns your view a little when the foe nears its edge. It never turns your view up or down.

Not: aim assist, auto-aim, lock-on

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

The knight in blackened plate, a bucket helm and wide shoulders, with a torn cloak behind: the player's knight in Practice unless the save chooses the other. A working label until the person names it.

Not: Black Knight, black knight

### Chop

A cut straight down from above the head: hold the left button and move the mouse down.

Not: overhead slash

### Cut

What the sword does while the left button is held and the mouse moves: the way the mouse goes picks a level cut, a chop, a rising cut, a diagonal cut or a thrust.

Not: slash, swipe, combo

### Desync

The two browsers in an online duel no longer agree on the duel: each sums its duel every {online.sum_seconds} second, and the sums for the same moment differ. The duel stops, and the game says so; it is a fault in the game.

Not: lag, rubber-banding

### Diagonal cut

A cut along a slant, from high on one side to low on the other or the other way: hold the left button and move the mouse clearly on a slant.

### Dodge-step

A short quick step to one side with Space and a movement key. You cannot dodge-step again straight away.

Not: roll

### Draw

Taking the sword out of its sheath with X. Drawn, the mouse still turns you; hold the right button to move the sword hand instead.

Not: unsheathe

### Duel

One fight between two fighters in the arena, each moving only through the keys and mouse of its input.

Not: match, bout, deathmatch

### Field of view

How wide you see from inside the helmet, in degrees across the screen, from {fov.min} to {fov.max}.

Not: FOV, zoom

### Gap

An opening in plate where a blow lands whole: the armpit, the visor's slit and the back of the knee.

Not: weak spot, chink

### Health

How much harm a body can take. A blade takes more the faster it moves when it hits. You start with {health.player}.

Not: HP, hit points

### Host

Opening a duel for another browser to join online, by a room code or by pasted codes. The host fights as the black helm.

Not: server, lobby

### Inner ring

A ring of greyer stones on the arena's floor, {rounds.safe_across} metres across, round its middle: the safe ground in sudden death.

Not: safe zone

### Ivory crown

The knight in pale plate with gilt, a crown of spikes and a skirt of plate burnt at the hem: the computer's knight in Practice unless the save gives it to the player. A working label until the person names it.

Not: Ivory King, Burnt Ivory King

### Join

Entering a duel another browser has opened online, by its room code or by pasted codes. The one who joins fights as the ivory crown.

Not: connect, matchmaking

### Knight

A fighter in plate. In Practice the computer fights as a knight through the same input a player gives.

Not: bot, enemy AI, NPC, Black Knight

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

### Pasted codes

Meeting online with no relay: the host's page makes an invitation, a long code the host sends to the other player by any means; the other player pastes it and sends back a reply, which the host pastes in turn.

Not: token, key, password

### Plate

The armour a knight wears: shaped steel over the body, with gaps at the joints. It takes most of a cut and a little of a thrust, wears with each blow it takes, and breaks off once battered enough. A part in plate is never cut off.

Not: platemail, plate mail

### Practice

A duel against the computer's knight, from the menu: best of three rounds.

Not: training mode, tutorial

### Relay

A public machine on the internet that introduces two browsers by room code. Once they have met, the duel passes between the browsers directly and the relay carries none of it.

Not: server, lobby

### Rising cut

A cut from low in front up over the shoulder: hold the left button and move the mouse up.

### Room code

{online.code_length} letters the host's page shows and the other player types after choosing Join, so the two browsers find each other through the relays.

Not: lobby code, PIN, password

### Round

One part of a duel: it ends when a knight falls. The first to win {rounds.to_win} rounds wins the duel.

Not: bout, set, match

### Run

Moving at {run.speed} metres a second while Shift is held with a movement key, sword sheathed.

Not: sprint

### Save

What the game keeps in this browser between visits: your settings and your knight's look. You can download it as a file and load it again.

Not: profile, account, cloud save

### Sudden death

After {rounds.sudden} seconds a round enters sudden death: only the inner ring is safe, and outside it you lose {rounds.sand_rate} health a second.

Not: overtime, storm

### Team deathmatch

Two teams of up to five fighters each in one arena.

Not: TDM

### Thrust

A straight stab ahead: hold the left button and turn the wheel away from you.

Not: lunge, poke

### Walk

Moving at {walk.speed} metres a second with W, A, S or D, relative to where you look.
