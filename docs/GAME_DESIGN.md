# MONTE CRISTO -- Game Design Document v2

A 16-bit-style graphic RPG adaptation of Alexandre Dumas, *The Count of Monte Cristo* (1844).
Engine model: Chrono Trigger field combat (visible enemies, no random encounters, ATB with combination Techs) over a Final Fantasy VI style ensemble cast and ability progression.

This document is L2 SPECIFICATION input. It is normative for content. Where it conflicts with .agent/specs/*, the specs win.

---

## 0. What changed from v1

1. Conversation is no longer a combat system. There are no dialogue HP bars, no dialogue turn order, no dialogue resource meters. Dialogue is authored story content called a **Confidence**, carrying two hidden values only: per-character Trust and a single global Mask value.
2. Combat is the primary loop. 15 regions, 102 bestiary entries, ~180 hand-placed encounters plus terrain-gated roaming spawns.
3. **General Fernand, Comte de Morcerf, is the final boss.** The Morcerf campaign is gated to close last.
4. Documents are key items that unlock scenes and regions. They are not ammunition.

---

## 1. Design laws

L1. **No supernatural.** Every enemy is a man, an animal, or the environment. No demons, no magic, no undead. Period-plausible only.
L2. **Terrain determines the bestiary.** An enemy may only appear in regions whose terrain and period justify it. Sewer rats do not appear on Monte Cristo; janissaries do not appear in the Rhone marsh.
L3. **Story flags gate spawns.** Benedetto's bravos do not exist in the Paris streets before the Caderousse burglary. Region tables are flag-filtered.
L4. **No grinding.** Every region carries a finite encounter budget per chapter; repeat experience decays to zero. Power comes from the Curriculum and from story milestones.
L5. **Dialogue advances character; combat advances pressure.** A Confidence never has a fail state that costs the player a life. A battle never resolves a relationship.
L6. **Wounds persist.** Health does not fully restore between encounters inside a chapter. Rest points are authored and scarce.
L7. **The book wins ties.** Where fun and fidelity conflict and the conflict is not load-bearing, fidelity wins and the deviation is recorded in Section 9.

---

## 2. Act structure

| Act | Setting and dates | Hours | Combat weight |
|---|---|---|---|
| I | Marseille and Elba, Feb 1815 | 3.0 | Light. Brawls, vermin, the arrest |
| II | Chateau d'If, 1815-1829 | 7.0 | Heavy. Vermin, bad prisoners, guards, the Ravelin |
| III | The sea, the Ligurian coast, Monte Cristo, 1829 | 4.5 | Heavy. Boarding actions, beasts, the grotto |
| IV | Pont du Gard, Marseille, Yanina, Corsica, 1829-1838 | 5.5 | Medium. Highwaymen, Ottoman troops, vendetta |
| V | Rome, Carnival, Feb 1838 | 4.0 | Heavy. Catacombs, Vampa's band |
| VI | Paris, Auteuil, Rome, 1838-1839 | 12.0 | Heavy. Quarries, sewers, Benedetto, the final boss |
| VII | Chateau d'If, Marseille, Monte Cristo | 2.5 | None. Epilogue |

### Act I -- Marseille
Playable as Edmond Dantes, nineteen, deliberately weak. The *Pharaon* hold (bilge rats, weevils), the Elba landing (sentries, a feral goat track), the dockside brawl at La Reserve, and the arrest. Danglars drafts the denunciation in the arbour; witnessing it is optional and pays off in Act II.

### Act II -- Chateau d'If
The novel's fourteen years become the game's training dungeon. A month-per-turn calendar (168 turns) with four actions: Dig, Study, Endure, Observe.

Years 1-6 alone: the cell, the corridor, the yard. Enemies are vermin and men. The yard is a genuine hazard -- Antoine the Strangler will kill a level 4 Edmond and is trivial to a level 14 Edmond who has taken Fencing to rank 4.

Year 6: Faria breaks through. Faria is a party member and a Tutor. The **Curriculum** opens (Section 5.2). Faria's tunnel opens the **Ravelin**, an optional flooded dungeon under the fortress: cistern eels, shore crabs, cave rats, blind fish, collapsing vaults, and the game's hardest optional encounter, the Thing in 12.

Faria reconstructs the conspiracy; the player walks Act I again as memory and fills the Web of Debt. The tunnel collapses, Faria dies, and the sack sequence is a timed stealth escape with one knife and a breath meter.

### Act III -- Sea, coast, island
The *Jeune-Amelie*: three months of contraband running along the Ligurian coast. Ship-to-ship boarding actions against Sardinian coastguard cutters, sharks in the water, wolves and boar in the maquis. The goat hunt on Monte Cristo, the staged fall, the powder charge, the second grotto, the treasure. Then the Chateau d'If archive: the Count buys the prison register and takes the unsigned denunciation page.

### Act IV -- The Reckoning Tour
Four episodic chapters, each teaching one system.
1. Pont du Gard: Busoni and Caderousse; highwaymen on the causeway; marsh fever as a persistent status.
2. Marseille: Lord Wilmore saves the house of Morrel on a three-month timer. The only unambiguously happy chapter in the game, placed here on purpose.
3. Yanina: played as Haydee, age twelve. Janissaries take the fortress; Selim dies at the powder magazine; the slave market. Establishes the Yanina dossier items.
4. Corsica and the household: Bertuccio's vendetta in the maquis, Ali's rescue from the Bey's guards, the Auteuil villa, the Roman palazzo.

### Act V -- Rome
The Colosseum by night, the pardon of Peppino, the Piazza del Popolo execution, the Carnival, the abduction of Albert, and the catacombs of San Sebastiano -- the first full dungeon, ending in Diavolaccio and then a duel with Luigi Vampa that closes with a handshake.

### Act VI -- Paris
Four campaigns on a shared Season Clock of 24 fortnights. Three run in parallel; the fourth is gated behind them.

**CADEROUSSE (Greed).** Benedetto, the false Cavalcanti, the burglary of the Champs-Elysees house, the pursuit across Montfaucon through the rat tide, the murder in the street, Busoni over a dying man.

**DANGLARS (Money).** The Montlhery semaphore puzzle, the Bourse, the unlimited credit account, the Cavalcanti marriage contract, the arrest at the signing, the flight to Rome, and Vampa's cave -- an encounter presented with a full battle interface in which the only functioning command is Wait.

**VILLEFORT (Family).** Auteuil, Bertuccio's paralysis, the buried box. Heloise's poisonings on a visible schedule. Playable Valentine in three sickroom chapters with the Noirtier blink-cipher. Benedetto's trial. Then Edouard.

**MORCERF (Honour) -- gated, closes last.** The Yanina dossier completes only when Danglars' bank inquiry to Yanina returns (which requires the Danglars campaign at stage 4) and the Chamber of Peers procedure is obtainable (which requires the Villefort household collapsed). Beauchamp and the press, the Chamber of Peers hearing played as Haydee, Albert's challenge, Mercedes' plea, Albert's public withdrawal at the Opera -- and then Fernand comes to the house.

### Act VII -- Epilogue
No combat. The Count buys the fortress and walks down to Cell 34 alone; the same rats that nearly killed him at twenty die in one hit or are simply stepped over. Maximilien's ordeal on Monte Cristo, the hashish, Valentine alive. The white sail. *Attendre et esperer.*

---

## 3. Combat

Chrono Trigger active-time battles fought in place on the field map. No transition wipe, no random encounters, no separate battle arena.

- Party of three. Positional area techs. Dual and triple Techs.
- Enemies are visible on the field and can be avoided, ambushed from behind (pre-emptive), or walked into carelessly (back attack).
- **Encounter budget:** each region-chapter has a spawn pool of N. Cleared spawns return on map re-entry until the pool is spent; experience per repeat decays by 30 percent compounding and floors at zero. There is nothing to farm.
- **Wounds:** damage taken persists between encounters within a chapter. Only authored rest points (the cell bunk, the ship's berth, the palazzo, the Champs-Elysees house) restore fully.
- Status effects are period-grounded: Bleeding, Fever (marsh, prison), Fouled Powder, Winded, Blinded (lime, lantern), Poisoned (five named compounds), Broken Guard, Terror.

### 3.1 Party

| Character | Acts | Combat identity |
|---|---|---|
| Edmond Dantes / The Count | all | Fencing school; the only character who learns from the Curriculum |
| Abbe Faria | II | Tutor commands, chemistry improvisation, low HP, dies |
| Jacopo | III-VII | Speed, knives, steals, sea legs |
| Ali | IV-VII | Mute. Lasso, thrown blade, silent-order first strike |
| Bertuccio | IV-VII | Vendetta scaling; freezes at Auteuil, scripted and unavoidable |
| Haydee | IV-VII | Yatagan and testimony; Yanina memory techs |
| Maximilien Morrel | VI-VII | Spahi cavalry sabre; damage scales inversely with his own HP |
| Albert de Morcerf | V | Temporary. Cheerful, mediocre, doomed |
| Luigi Vampa / Peppino | V, VI | Temporary allies |
| Selim | IV | Temporary, dies at the magazine |
| Valentine de Villefort | VI | Playable, non-combat sickroom chapters |

### 3.2 Signature techs

| Tech | Members | Effect |
|---|---|---|
| Faria's Parry | Edmond | Counter-stance; converts one incoming hit into a free action |
| The Genoese Riposte | Edmond + Jacopo | Jacopo trips, Edmond runs through |
| Silent Order | Edmond + Ali | Ali acts on a hand signal before the gauge fills |
| Vendetta | Bertuccio | Damage scales with chapters elapsed since his last kill |
| Spahi Charge | Maximilien | Damage scales inversely with remaining HP |
| Yanina Remembers | Haydee + any | Inflicts Terror on human enemies; no effect on animals |
| Lime and Lantern | Edmond + Faria | Chemistry: blinds a group, ignites fouled powder |
| The Cliff Path | Edmond + Jacopo + Ali | Triple; only usable on rock terrain |

---

## 4. Bestiary

102 entries across 15 regions. Families: VERMIN, BEAST, SEA, MAN_AT_ARMS, CRIMINAL, PRISONER, TROOP, HAZARD, BOSS.

### R01 Marseille and the Port (Act I)
Wharf Rat; Bilge Rat; Weevil Swarm; Dock Cur; Harbour Gull; Drunk Stevedore; Press-Gang Thug; Catalan Fisher-Tough; Quayside Cutpurse; Customs Inspector (non-lethal, ends in a fine); Gendarme Squad (the arrest, unwinnable).
Mini-boss: **Pierre the Wharf Bull**, a dock brawler who fights for a barrel of wine.

### R02 Elba and the Tyrrhenian (Act I flashback)
Elban Sentry; Grenadier of the Guard (hostile only if alarmed); Feral Goat; Sea Hawk; Sirocco (HAZARD, wind pushes the party on cliff tiles).

### R03 Chateau d'If -- Cells and Yard (Act II)
Cell Rat; Rat Swarm; Louse Cloud (HAZARD, inflicts Fever); Cell Centipede; Prison Bat; Mold Bloom (HAZARD); Turnkey; Drunk Turnkey; Sergeant of the Watch; Wall Sentry; Governor's Mastiff; Toulon Escapee; The Coiner; Brother Anselme (the madman of 27; can be calmed instead of fought); The Silent Man of 19; Gallery Bullies (three-man group).
Mini-boss: **Antoine the Strangler**, a lifer, fought in the yard over a bread ration.
Boss: **Le Rongeur**, the old rat of the cistern, nesting on forty years of prisoners' hidden goods.

### R04 The Ravelin and Drowned Galleries (Act II, optional)
Cistern Eel; Shore Crab; Blind Cave Fish; Cave Rat; Sea Louse; Collapsing Vault (HAZARD); Patrol Lantern (HAZARD, stealth fail state).
Superboss: **The Thing in 12**, a prisoner forgotten since the Terror. Killing him is possible. Feeding him is better, and is the only way to obtain the Ravelin Key.

### R05 The Sea and the Jeune-Amelie (Act III)
Blue Shark; Barracuda; Storm Petrel Flock; Rival Smuggler; Corsair Boarder; Genoese Cutter Marine; Sardinian Coastguard; Powder Fire (HAZARD); Squall (HAZARD).
Boss: **Coastguard Lieutenant Sarti**, a boarding action across two decks.

### R06 Ligurian Coast and Maquis (Act III)
Wild Boar; Grey Wolf; Wolf Pack Leader; Maquis Adder; Golden Eagle; Chamois (flees, drops nothing, worth hunting for a recipe); Contraband Runner; Hill Brigand; Carabiniere Patrol.
Boss: **Il Cinghiale Vecchio**, the old boar of the goat-hunt chapter.

### R07 Isle of Monte Cristo (Act III and VII)
Feral Goat; Rock Viper; Cave Crab; Grotto Salamander; Herring Gull Swarm; Smuggler Sentry; Rockfall (HAZARD); Firedamp (HAZARD, ignites on lantern use).
Boss: **The Grotto Collapse**, an environmental encounter: a timed escape, no enemy sprite.

### R08 Pont du Gard and the Rhone Marsh (Act IV)
Marsh Mosquito Cloud (inflicts Fever); Marsh Viper; Bog Leech; Wild Dog Pack; Highwayman; Deserter of the Hundred Days; Toll-Bridge Bandit.
Boss: **The Auberge Ambush**, three highwaymen who take Busoni for a rich traveller.

### R09 Yanina and the Levant (Act IV, played as Haydee)
Janissary; Ottoman Sipahi; Delibas Skirmisher; Kapikulu Grenadier; Fortress Dog; Slaver's Guard; Powder Magazine (HAZARD).
Boss: **Kourshid's Captain**, who takes the fortress.
Scripted loss: **The Magazine**, Selim's death; unwinnable by design.

### R10 Corsica and the Vendetta Maquis (Act IV)
Corsican Boar; Maquis Wolf; Vendetta Assassin; Bandit d'Honneur; Village Mastiff; Customs Officer.
Boss: **The Rogliano Ambush**.

### R11 Rome -- Colosseum, Catacombs, Pontine (Act V)
Colosseum Cur; Roman Alley Cat; Catacomb Rat Swarm; Tomb Bat Cloud; Ossuary Beetle; Carnival Pickpocket; Drunken Masker (non-lethal); Vampa Sentry; Vampa Bandit; Vampa Carbineer; Pontine Buffalo; Malaria Fog (HAZARD).
Boss: **Diavolaccio**, Vampa's lieutenant, in the ossuary.
Boss: **Luigi Vampa**, a duel that ends in a handshake and a permanent ally flag.

### R12 Paris -- Streets, Sewers, Montfaucon, Quarries (Act VI)
Sewer Rat Swarm; Montfaucon Rat Tide (a survival encounter, not a kill encounter); Knacker's Dog; Quarry Scavenger; Grave-Robber; Mudlark; Toulon Escapee (Paris variant); Benedetto's Bravo; Cutpurse; Faubourg Brawler; Barriere Toughs; Bois de Vincennes Poacher; Flooded Gallery (HAZARD).
Dungeon: **The Quarries beneath Montrouge**, Benedetto's hideout.
Boss: **Benedetto**, twin knives, the only fight the Count takes personally.

### R13 Auteuil and the Villefort House (Act VI)
Cellar Rat; Garden Mastiff; Poisoner's Cat (steals items, never dies, always escapes); Night Watchman; Grave-digger; Nightjar (harmless, ambient).
Boss: **Heloise's Cabinet**, a chemistry encounter fought against a poison and a clock, not against a person.

### R14 Chateau d'If Revisited (Act VII)
Cell Rat; Rat Swarm; Cistern Eel; Smuggler Squatter. All demoted to one-hit trash. This is the point.

### R15 Champs-Elysees and the Hotel de Morcerf (Act VI finale)
Morcerf Footman; Yanina Veteran; Duelling Master.
**FINAL BOSS: General Fernand, Comte de Morcerf.**

---

## 5. Systems

### 5.1 Confidences (dialogue -- not combat)
An authored scene between the player character and one or two named characters. Branching tree, 3 to 8 minutes, portrait bust-ups with expression frames, no meters shown, no turns, no failure state.

Each Confidence may: set or clear story flags; adjust Trust for a named character by a small integer; adjust the global Mask value; consume or grant a key item; unlock a region, a party member, a Tech, or a variant of a later scene.

Trust is never shown as a number. It surfaces as changed dialogue, changed scene variants, and in four cases a changed outcome (Mercedes' recognition, Albert's withdrawal, Haydee's willingness to testify, Maximilien's confidence). Mask is a single global 0-100 value that drops only at scripted moments, and its only mechanical effects are which persona may enter which map and which of four endings-texture variants the epilogue uses.

45 authored Confidence scenes. Approximately 210,000 words of dialogue total.

### 5.2 Faria's Curriculum
Seven disciplines, ranks 1-5, bought with Study turns in Act II and extended by tutors afterward. This is the character build, and it is combat-relevant.

| Discipline | Grants |
|---|---|
| Fencing | Every Edmond weapon tech, parry timing window, counter-stance |
| Chemistry | Antidotes, incendiaries, lime blinding, poison identification, the Heloise encounter |
| Natural Philosophy | Lockpicking, mechanism puzzles, the If escape, the grotto charge, trap disarm |
| Mathematics | Semaphore cipher, the Bourse, enemy stat preview, critical-hit timing |
| Languages | Persona access, Haydee recruitment, region access in Rome and the Levant |
| History and Politics | Chamber of Peers procedure, Web of Debt completion, Trust ceilings |
| Economics | Credit, expenditure efficiency, the Danglars campaign |

Under-invest in Chemistry and the player reaches Act VI unable to identify what is killing Valentine. The game does not warn them. Faria did.

### 5.3 Personas
Five: Abbe Busoni, Lord Wilmore, Sinbad the Sailor, the Count of Monte Cristo, Edmond Dantes. Swapped at safe houses. Each gates map and scene access and carries its own sprite set and dialogue register. Edmond Dantes unlocks twice, both times in the final act, and one of those uses is the killing blow of the final boss.

### 5.4 The Web of Debt
A full-screen graph of four target nodes and their proven vulnerabilities. Assembled from testimony in Act II, completed across Acts IV-VI. It is also the quest log and the fast-travel map.

### 5.5 Expenditure and Notoriety
After Act III there is no gold counter, only a Ledger. Purchases are authored, not shopped. Each costs Notoriety. High Notoriety opens Paris doors and accelerates the four antagonists' counter-investigations. The optimal play is to spend catastrophically once per campaign at the moment of maximum effect, which is what the Count does in the book.

### 5.6 Poison and tolerance
Five compounds with onset, dose, and cumulative tolerance curves. Brucine tolerance builds across days and is the literal mechanism of Valentine's survival, administered by Noirtier over weeks. Fully simulated, fully testable, and the subject of the Heloise encounter.

### 5.7 The Season Clock
Act VI runs on 24 fortnights. Every campaign action costs one. Events fire on schedule whether the player attends or not. The four campaigns cannot all be played optimally.

---

## 6. The final boss: General Fernand, Comte de Morcerf

**Trigger.** All three of MORCERF_YANINA_DOSSIER, MORCERF_ALBERT_WITHDRAWN, MERCEDES_RECOGNITION must be set. The first requires the Danglars campaign at stage 4 (his bank's inquiry to Yanina returns the answer) and the Villefort household collapsed (Chamber of Peers procedure obtainable).

**Setting.** The Champs-Elysees house, night, September 1839. Fernand arrives armed and alone.

**Phase 1 -- The General.** A full high-tier ATB fight. Fernand is the only enemy in the game trained in the same fencing school as Edmond and the only one who can counter Faria's Parry. Techs: Yanina Volley, Officer's Guard, Cavalry Trample, Spahi Charge (deliberately the mirror of Maximilien's). The player fights alone; no party.

**Phase 2 -- The Name.** At zero HP Fernand does not fall. The standard command menu is replaced by a four-entry list. Three entries do nothing. The fourth, NAME_YOURSELF, is greyed out unless all three dossier flags are set, and ends the encounter when used. Damage can never end phase 2. This is the mechanical statement of the book's thesis.

**Phase 3 -- The Pursuit.** Not a fight. A scripted run through night Paris to the Hotel de Morcerf, arriving as Mercedes and Albert leave. A shot from an upstairs window. The player does not enter the room.

---

## 7. Art, audio, and platform target

Authentic SNES-era constraints, rendered by a modern engine.

- 256x224 internal resolution, integer-scaled. 16x16 tiles. Two scrolling background layers plus one overlay.
- 15-bit colour, act-locked palettes. Marseille: cerulean and sand. If: six greys and one red. Rome: ochre and torch. Paris: black, gold, wine, gaslight. Monte Cristo: white marble.
- Vertical gradient bands on every sky, emulating HDMA.
- A Mode-7-equivalent affine layer for the *Pharaon*, the Mediterranean map, the grotto reveal, and the If flyover.
- Sprites: 24x32 field, 48x64 battle, 64x80 Confidence portraits with eight expression frames.
- Eight-channel sample-based music, 34 tracks. Woodwind-forward for Acts I-III, string quartet and harpsichord for Paris, solo reed and hand drum for Haydee. The If theme is one clarinet and a water drip.

---

## 8. Scope

| Metric | Target |
|---|---|
| Main path | 36-40 hours |
| Maps | 118 |
| Bestiary entries | 102 |
| Hand-placed encounters | 180 |
| Roaming spawn tables | 15 regions x 3 chapter stages |
| Confidence scenes | 45 |
| Boss encounters | 21 |
| Script | 340,000 words total, 210,000 of it dialogue |
| Music tracks | 34 |

**New Game+** carries the Curriculum and unlocks Omniscience: every NPC's private thoughts become readable in the register of Dumas' narrator. Act I becomes unbearable. That is the point.

---

## 9. Fidelity ledger

**Kept whole.** The Elba letter and its unread contents. Villefort burning it. Fourteen years. Faria's tutelage and his reconstruction of the conspiracy. The burial sack. The Spada treasure. Busoni, Wilmore, Sinbad. The Morrel rescue and the rebuilt *Pharaon*. Carnival, Vampa, Albert's abduction. Haydee and the sale of Yanina. Benedetto's parentage and the Auteuil garden. Heloise's poisonings and Barrois' death. Noirtier's eyes and the broken d'Epinay betrothal. Valentine's survival by brucine tolerance. Danglars' telegraph ruin, Eugenie's flight, the starvation cave, the release with fifty thousand francs. The confrontation in which the Count names himself to Fernand. Fernand's suicide. Villefort's madness. Edouard's death. The Count's doubt. Maximilien's ordeal. Haydee. Wait and hope.

**Compressed.** The 1829-1838 gap becomes four playable chapters instead of narration. Eugenie's elopement is one chapter rather than a thread. Cavalcanti pere folds into the coaching sequence.

**Deviations, flagged.**
1. **Order.** The novel destroys Morcerf before Danglars and Villefort. The game gates Morcerf last so that the confrontation with Fernand is the finale, and so that the Count enters it already broken by Edouard. Cost: a few weeks of compressed chronology.
2. **The final boss is a fight.** In the novel the confrontation is a scene, not a duel. The game makes phase 1 a real battle and then refuses to let damage end it, which is the closest mechanical translation of the original scene available.
3. **The bestiary is invented.** The novel contains almost no combat. Every encounter layer is new, and constrained by design laws L1 and L2 to period-plausible, terrain-appropriate, non-supernatural threats.
4. **The Ravelin, the Montrouge quarries, and Montfaucon** are real places used as invented dungeons.
5. **Playable Haydee and playable Valentine.** Their events are the novel's; the camera is not.

**Refused.** No alternate endings. No romance route for Mercedes. No sparing Villefort. No saving Edouard. The Mercy Ledger -- a hidden count of the Count's graces -- changes the epilogue's framing text and final palette only, and changes nothing that happens.
