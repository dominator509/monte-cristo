# SPEC-009 -- Content: regions, bestiary, encounters, progression

Locked vocabulary. Identifiers are used exactly as written. Source of the content design is
docs/GAME_DESIGN.md; where they disagree, this file wins.

## 1. Regions

| Id | Name | Act | Terrain character |
|---|---|---|---|
| R01 | Marseille and the Port | ACT_I_MARSEILLE | warm dockside, ship holds, Catalan village |
| R02 | Elba and the Tyrrhenian | ACT_I_MARSEILLE | rocky garrison coast |
| R03 | Chateau d'If, Cells and Yard | ACT_II_IF | stone cells, corridors, exercise yard |
| R04 | The Ravelin and Drowned Galleries | ACT_II_IF | flooded understructure |
| R05 | The Sea and the Jeune-Amelie | ACT_III_SEA | open water, decks, boarding actions |
| R06 | Ligurian Coast and Maquis | ACT_III_SEA | scrub hills, goat tracks |
| R07 | Isle of Monte Cristo | ACT_III_SEA | bare rock, grottoes |
| R08 | Pont du Gard and the Rhone Marsh | ACT_IV_TOUR | inn, causeway, fever marsh |
| R09 | Yanina and the Levant | ACT_IV_TOUR | fortress, magazine, slave market |
| R10 | Corsica and the Vendetta Maquis | ACT_IV_TOUR | thorn scrub, village |
| R11 | Rome: Colosseum, Catacombs, Pontine | ACT_V_ROME | ruins, ossuaries, marsh road |
| R12 | Paris: Streets, Sewers, Montfaucon, Quarries | ACT_VI_PARIS | gaslight, sewers, knackery, quarries |
| R13 | Auteuil and the Villefort House | ACT_VI_PARIS | garden, cellar, sickroom |
| R14 | Chateau d'If Revisited | ACT_VII_EPILOGUE | decayed fortress |
| R15 | Champs-Elysees and the Hotel de Morcerf | ACT_VI_PARIS | townhouse interiors, night streets |

## 2. Enemy families (closed set -- design law L1)

`VERMIN`, `BEAST`, `SEA`, `MAN_AT_ARMS`, `CRIMINAL`, `PRISONER`, `TROOP`, `BANDIT`,
`HAZARD`, `BOSS`.

Any other value fails the bake. There is no supernatural family and there never will be;
adding one requires changing this spec, which requires an ADR (ADR-011).

## 3. Bestiary, by region

Total 102 entries. Identifiers are `ENM_<UPPER_SNAKE>`.

**R01 (11):** WHARF_RAT, BILGE_RAT, WEEVIL_SWARM, DOCK_CUR, HARBOUR_GULL, DRUNK_STEVEDORE,
PRESS_GANG_THUG, CATALAN_FISHER_TOUGH, QUAYSIDE_CUTPURSE, CUSTOMS_INSPECTOR,
GENDARME_SQUAD. Mini-boss: PIERRE_WHARF_BULL.

**R02 (5):** ELBAN_SENTRY, GRENADIER_OF_THE_GUARD, FERAL_GOAT, SEA_HAWK, SIROCCO (HAZARD).

**R03 (17):** CELL_RAT, RAT_SWARM, LOUSE_CLOUD (HAZARD), CELL_CENTIPEDE, PRISON_BAT,
MOLD_BLOOM (HAZARD), TURNKEY, DRUNK_TURNKEY, SERGEANT_OF_THE_WATCH, WALL_SENTRY,
GOVERNORS_MASTIFF, TOULON_ESCAPEE, THE_COINER, BROTHER_ANSELME, SILENT_MAN_OF_19,
GALLERY_BULLIES. Mini-boss: ANTOINE_THE_STRANGLER. Boss: LE_RONGEUR.

**R04 (8):** CISTERN_EEL, SHORE_CRAB, BLIND_CAVE_FISH, CAVE_RAT, SEA_LOUSE,
COLLAPSING_VAULT (HAZARD), PATROL_LANTERN (HAZARD). Superboss: THE_THING_IN_12.

**R05 (9):** BLUE_SHARK, BARRACUDA, STORM_PETREL_FLOCK, RIVAL_SMUGGLER, CORSAIR_BOARDER,
GENOESE_CUTTER_MARINE, SARDINIAN_COASTGUARD, POWDER_FIRE (HAZARD), SQUALL (HAZARD).
Boss: LIEUTENANT_SARTI.

**R06 (9):** WILD_BOAR, GREY_WOLF, WOLF_PACK_LEADER, MAQUIS_ADDER, GOLDEN_EAGLE, CHAMOIS,
CONTRABAND_RUNNER, HILL_BRIGAND, CARABINIERE_PATROL. Boss: IL_CINGHIALE_VECCHIO.

**R07 (8):** FERAL_GOAT_MC, ROCK_VIPER, CAVE_CRAB, GROTTO_SALAMANDER, HERRING_GULL_SWARM,
SMUGGLER_SENTRY, ROCKFALL (HAZARD), FIREDAMP (HAZARD). Boss: THE_GROTTO_COLLAPSE (HAZARD
boss, no enemy sprite).

**R08 (7):** MARSH_MOSQUITO_CLOUD, MARSH_VIPER, BOG_LEECH, WILD_DOG_PACK, HIGHWAYMAN,
DESERTER_OF_THE_HUNDRED_DAYS, TOLL_BRIDGE_BANDIT. Boss: THE_AUBERGE_AMBUSH (group).

**R09 (7):** JANISSARY, OTTOMAN_SIPAHI, DELIBAS_SKIRMISHER, KAPIKULU_GRENADIER,
FORTRESS_DOG, SLAVERS_GUARD, POWDER_MAGAZINE (HAZARD). Boss: KOURSHIDS_CAPTAIN.
Scripted loss: THE_MAGAZINE.

**R10 (6):** CORSICAN_BOAR, MAQUIS_WOLF, VENDETTA_ASSASSIN, BANDIT_DHONNEUR,
VILLAGE_MASTIFF, CUSTOMS_OFFICER. Boss: THE_ROGLIANO_AMBUSH.

**R11 (12):** COLOSSEUM_CUR, ROMAN_ALLEY_CAT, CATACOMB_RAT_SWARM, TOMB_BAT_CLOUD,
OSSUARY_BEETLE, CARNIVAL_PICKPOCKET, DRUNKEN_MASKER, VAMPA_SENTRY, VAMPA_BANDIT,
VAMPA_CARBINEER, PONTINE_BUFFALO, MALARIA_FOG (HAZARD). Bosses: DIAVOLACCIO, LUIGI_VAMPA.

**R12 (13):** SEWER_RAT_SWARM, MONTFAUCON_RAT_TIDE, KNACKERS_DOG, QUARRY_SCAVENGER,
GRAVE_ROBBER, MUDLARK, TOULON_ESCAPEE_PARIS, BENEDETTOS_BRAVO, CUTPURSE, FAUBOURG_BRAWLER,
BARRIERE_TOUGHS, VINCENNES_POACHER, FLOODED_GALLERY (HAZARD). Boss: BENEDETTO.

**R13 (6):** CELLAR_RAT, GARDEN_MASTIFF, POISONERS_CAT, NIGHT_WATCHMAN, GRAVE_DIGGER,
NIGHTJAR. Boss: HELOISES_CABINET (HAZARD boss: a poison and a clock, not a person).

**R14 (4):** CELL_RAT, RAT_SWARM, CISTERN_EEL, SMUGGLER_SQUATTER. All demoted to tier 1
trash by Act VII scaling; the demotion is the point (docs/GAME_DESIGN.md section 2, Act VII).

**R15 (3 plus the boss):** MORCERF_FOOTMAN, YANINA_VETERAN, DUELLING_MASTER.
**FINAL BOSS: ENM_FERNAND_GENERAL.**

## 4. Encounter counts

180 hand-placed encounters. 45 spawn tables (15 regions x 3 chapter stages). 21 boss
encounters. `cargo run -p mc_tools -- report bestiary` prints the live counts and EP-007
asserts them against this table.

## 5. Party roster

`CHR_EDMOND` (alias `CHR_COUNT` after Act III), `CHR_FARIA`, `CHR_JACOPO`, `CHR_ALI`,
`CHR_BERTUCCIO`, `CHR_HAYDEE`, `CHR_MAXIMILIEN`, `CHR_ALBERT`, `CHR_VAMPA`, `CHR_PEPPINO`,
`CHR_SELIM`, `CHR_VALENTINE` (non-combat).

## 6. Curriculum grants

| Discipline | R1 | R2 | R3 | R4 | R5 |
|---|---|---|---|---|---|
| FENCING | ABL_LUNGE | ABL_PARRY | TECH_FARIAS_PARRY | ABL_RIPOSTE | TECH_COUNTER_SCHOOL |
| CHEMISTRY | ABL_ANTIDOTE | ABL_IDENTIFY_POISON | ABL_LIME_BLIND | TECH_LIME_AND_LANTERN | ABL_SYNTHESIZE |
| NATURAL_PHILOSOPHY | ABL_PICK_LOCK | ABL_DISARM_TRAP | ABL_CHARGE_SET | ABL_MECHANISM | ABL_ESCAPE_PLAN |
| MATHEMATICS | ABL_STAT_PREVIEW | ABL_CRIT_TIMING | ABL_SEMAPHORE | ABL_BOURSE_READ | ABL_ODDS |
| LANGUAGES | ABL_ITALIAN | ABL_ENGLISH | ABL_GREEK | ABL_ARABIC | ABL_SPANISH |
| HISTORY_POLITICS | ABL_WEB_READ | ABL_PROCEDURE | ABL_PRECEDENT | ABL_TRUST_CEILING | ABL_CHAMBER |
| ECONOMICS | ABL_CREDIT | ABL_LEDGER | ABL_EXPENDITURE | ABL_INQUIRY | ABL_RUIN |

`ABL_INQUIRY` at ECONOMICS rank 4 is what enables Danglars' bank to send the letter to
Yanina, which is one of the two gates on the Morcerf campaign (section 8).

## 7. Poison table

| Id | Onset (ticks) | Potency (Fx per tick) | Tolerance step | Lethal dose |
|---|---|---|---|---|
| PSN_BRUCINE | 240 | 0.25 | 0.10 | 4.0 |
| PSN_ACONITE | 120 | 0.60 | 0.04 | 3.0 |
| PSN_BELLADONNA | 300 | 0.30 | 0.06 | 3.5 |
| PSN_ARSENIC | 900 | 0.15 | 0.08 | 6.0 |
| PSN_HYDROCYANIC | 30 | 1.50 | 0.00 | 1.5 |

Valentine's survival: Noirtier administers PSN_BRUCINE at 0.5 dose across 18 authored days,
raising her tolerance to 1.8, which exceeds Heloise's administered dose of 1.6. Simulated,
not scripted; `poison_tolerance.rs` asserts it.

## 8. Campaign gating (ADR-009)

    MORCERF_YANINA_DOSSIER requires All([DANGLARS_STAGE_4, ABL_INQUIRY_USED, VILLEFORT_COLLAPSE])
    final encounter requires All([MORCERF_YANINA_DOSSIER, MORCERF_ALBERT_WITHDRAWN, MERCEDES_RECOGNITION])

This is what forces the Morcerf campaign to close last and makes Fernand the final boss.

## 9. Flag vocabulary (excerpt; full list in content/flags.ron)

`ACT1_ARREST`, `IF_FARIA_JOINED`, `IF_ESCAPE`, `TREASURE_FOUND`, `CADEROUSSE_TESTIMONY`,
`CADEROUSSE_BURGLARY`, `MORREL_SAVED`, `HAYDEE_RECRUITED`, `VAMPA_ALLY`, `QUARRIES_OPENED`,
`DANGLARS_STAGE_1` through `DANGLARS_STAGE_5`, `VILLEFORT_COLLAPSE`, `VILLEFORT_MADNESS`,
`EDOUARD_DEAD`, `MERCEDES_RECOGNITION`, `MORCERF_ALBERT_WITHDRAWN`, `MORCERF_YANINA_DOSSIER`,
`FERNAND_NAMED`, `EPILOGUE_SAIL`.

Reserved and forbidden (their use fails the bake, enforcing SPEC-000 section 4):
`MERCEDES_ROMANCE`, `VILLEFORT_SPARED`, `EDOUARD_SAVED`, `ENDING_ALT`.
