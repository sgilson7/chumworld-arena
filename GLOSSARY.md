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

### Banneret

The ladder's opponent at rung B: fluted plate from head to foot, a rounded helm with a plume, a blue tabard with a gilt band and a cape behind. A working label until the person names it.

Not: Lord, paladin

### Black helm

The knight in blackened plate, a bucket helm and wide shoulders, with a torn cloak behind: the player's knight in Practice unless the save chooses the other. A working label until the person names it.

Not: Black Knight, black knight

### Champion

The ladder's second-hardest opponent, rung A: broad tourney plate in pale steel and gilt, huge pauldrons, a red skirt and a flat-topped great helm with two gilt wings. A working label until the person names it.

Not: Champion Gundyr, Gundyr

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

### Dossier

What the ladder tells you about an opponent before you fight it: its rank, what it wears, where it wears plate, how it fights and what winning gives.

Not: profile, bio

### Draw

Taking the sword out of its sheath with X. Drawn, the mouse still turns you; hold the right button to guard instead.

Not: unsheathe

### Duel

One fight between two fighters in the arena, each moving only through the keys and mouse of its input.

Not: match, bout, deathmatch

### Feint

Breaking off a cut before it falls: press the right button while the sword still draws back for the cut. It costs {chivalry.feint_cost} stamina and brings the sword back to your guard.

Not: fake, bluff

### Field of view

How wide you see from inside the helmet, in degrees across the screen, from {fov.min} to {fov.max}.

Not: FOV, zoom

### Follow-up

A cut begun just as your last one ends, whether it landed or missed: it draws back faster. Up to {chivalry.follow_ups} follow in a row; a cut a blade stops ends the run.

Not: combo, combination

### Gap

An opening in plate where a blow lands whole: the armpit, the visor's slit and the back of the knee.

Not: weak spot, chink

### Guard

How you hold your sword between cuts. It rides with your view; hold the right button to raise the blade across your body in front, and move the mouse to shift it up to {chivalry.guard_shift} degrees each way. A blow that meets it is a parry.

Not: stance

### Health

How much harm a body can take. A blade takes more the faster it moves when it hits. You start with {health.player}.

Not: HP, hit points

### Hedge knight

The ladder's fourth opponent from the foot (rung D): a knight in half plate under a long green surcoat torn at the hem, cloth hose below the knee, and a pointed helm over a mail collar. A working label until the person names it.

Not: Ser Duncan

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

### Kick

A push with the foot, with F, while your sword is drawn and not cutting. A foe within {chivalry.kick_reach} centimetres in front staggers back, its cut breaks off and its guard is knocked aside. It costs {chivalry.kick_cost} stamina.

Not: shove, boot

### Knight

A fighter in plate. In Practice the computer fights as a knight through the same input a player gives.

Not: bot, enemy AI, NPC, Black Knight

### Ladder

The single-player run of arena fights against harder opponents in turn, each with its own costume: {ladder.rungs} rungs from rank {ladder.foot} to rank {ladder.top}, each a duel won by the first to win {rounds.to_win} rounds.

Not: campaign, tower, story mode

### Level cut

A cut across from one side to the other with the blade held flat: hold the left button and move the mouse sideways. It runs high, at the middle or low, by where the sword hand was.

Not: horizontal slash

### Look

The costume your knight is drawn in: the black helm, the ivory crown, or the costume of an opponent whose rung you have won. A look changes how you are drawn and nothing else: your plate, reach and strength stay a knight's.

Not: skin, outfit

### Mail

The rings of steel under plate. A part whose plate has broken off shows its mail, and a blow there lands whole.

Not: chainmail, chain mail

### Mark

What a won rung shows on its rank card: one for a win, and a second for a win without losing a round. A rung keeps its best, up to {ladder.marks}.

Not: star, medal

### Mirror

The ladder's last opponent, rung S: blackened plate with bright edges and a smooth egg-shaped helm with no face. A working label until the person names it.

Not: Mirror Knight, mirror knight

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

### Rank

A rung's letter on the ladder, from {ladder.foot} at the foot to {ladder.top} at the top.

Not: tier

### Rank card

A rung of the ladder as the ladder shows it: its rank, its opponent, whether it is locked, open or won, and its marks.

### Relay

A public machine on the internet that introduces two browsers by room code. Once they have met, the duel passes between the browsers directly and the relay carries none of it.

Not: server, lobby

### Riposte

A cut begun within {chivalry.riposte_s} seconds of your blade parrying one. It draws back faster, drives harder, and keeps pressing for a moment when a blade meets it.

Not: counter, counterattack

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

### Rung

One duel on the ladder against one opponent. The first rung is open; winning a rung opens the one above it and gives its opponent's look.

Not: stage, mission

### Save

What the game keeps in this browser between visits: your settings, your knight's look and the marks you have won on the ladder. You can download it as a file and load it again.

Not: profile, account, cloud save

### Sellsword

The ladder's second opponent from the foot (rung E): a hired fighter in a studded leather coat with steel plates inside, steel on the forearms, a helm with a tail and a chin plate, and one big plate on the right shoulder only. A working label until the person names it.

Not: Sellsword Twinblades, Sellsword set

### Squire

The ladder's first opponent (rung F): a young fighter in a padded quilted coat, a mail hood round an open face and a broad-brimmed steel hat, his only plate. A working label until the person names it.

### Stamina

What your arms and legs have left to give. Cuts, parries, feints and kicks cost it, and it comes back at {chivalry.regen} a second while you are not cutting. Each round starts you with {chivalry.stamina}.

Not: energy, endurance

### Sudden death

After {rounds.sudden} seconds a round enters sudden death: only the inner ring is safe, and outside it you lose {rounds.sand_rate} health a second.

Not: overtime, storm

### Team deathmatch

Two teams of up to five fighters each in one arena.

Not: TDM

### Thrust

A straight stab ahead: hold the left button and turn the wheel away from you.

Not: lunge, poke

### Veteran

The ladder's opponent at rung C, the sheet's "pit veteran": bare upper arms, one huge pauldron on the left shoulder, a leather coat over a mail skirt and an open helm with a T-shaped face. Only his helm, the pauldron's arm and his forearms wear plate. A working label until the person names it.

Not: gladiator, berserker

### Walk

Moving at {walk.speed} metres a second with W, A, S or D, relative to where you look.

### Winded

Out of stamina. Your arms work at {chivalry.winded_pct}% of their strength, so cuts come slow and land soft, until your stamina is back to {chivalry.winded_until}.

Not: exhausted, tired
