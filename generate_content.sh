#!/usr/bin/env bash
set -e
cd /root/monte-cristo

# ============================
# M1: Complete content tree
# ============================

mkdir -p content/regions content/bestiary content/spawn_tables content/scenes/act1 content/scenes/act2 content/scenes/act3 content/scenes/act4 content/scenes/act5 content/scenes/act6 content/scenes/act7 content/items content/strings/en

# ---- 15 Regions (R01 already exists, need R02-R15) ----
# R01 and R02 and R03 exist - let me check what we have
for r in 04 05 06 07 08 09 10 11 12 13 14 15; do
  case $r in
    04) name="ROME"; desc="Rome"; tier=2; conn='["R03_MONTE_CRISTO", "R05_PARIS_FAUBOURG"]';;
    05) name="PARIS_FAUBOURG"; desc="Paris Faubourg"; tier=2; conn='["R04_ROME", "R06_PARIS_SALON"]';;
    06) name="PARIS_SALON"; desc="Paris Salon"; tier=3; conn='["R05_PARIS_FAUBOURG", "R07_NORMANDY", "R08_LYON"]';;
    07) name="NORMANDY"; desc="Normandy"; tier=2; conn='["R06_PARIS_SALON"]';;
    08) name="LYON"; desc="Lyon"; tier=3; conn='["R06_PARIS_SALON", "R09_STRASBOURG"]';;
    09) name="STRASBOURG"; desc="Strasbourg"; tier=3; conn='["R08_LYON"]';;
    10) name="MEDITERRANEE"; desc="Méditerranée"; tier=4; conn='["R03_MONTE_CRISTO", "R11_ORIENT"]';;
    11) name="ORIENT"; desc="Orient"; tier=4; conn='["R10_MEDITERRANEE", "R12_GREECE"]';;
    12) name="GREECE"; desc="Greece"; tier=4; conn='["R11_ORIENT", "R13_ALBANIA"]';;
    13) name="ALBANIA"; desc="Albania"; tier=4; conn='["R12_GREECE"]';;
    14) name="MORCERF_ESTATE"; desc="Morcerf Estate"; tier=5; conn='["R06_PARIS_SALON", "R15_VILLEFORT_MANSION"]';;
    15) name="VILLEFORT_MANSION"; desc="Villefort Mansion"; tier=5; conn='["R06_PARIS_SALON", "R14_MORCERF_ESTATE"]';;
  esac
  cat > "content/regions/R${r}.ron" << REGION
(
    id: "R${r}_${name}",
    name_key: "region.r${r}.name",
    description_key: "region.r${r}.description",
    tier: ${tier},
    connections: ${conn},
    locked: false,
    gate: Always,
)
REGION
done

# Also update R01 to connect to R02 (already done), R02 to connect to R01 and R03
# R03 connects to R02 and R04
cat > content/regions/R02.ron << 'REGION'
(
    id: "R02_CHATEAU_DIF",
    name_key: "region.r02.name",
    description_key: "region.r02.description",
    tier: 1,
    connections: ["R01_MARSEILLE", "R03_MONTE_CRISTO"],
    locked: false,
    gate: Always,
)
REGION

cat > content/regions/R03.ron << 'REGION'
(
    id: "R03_MONTE_CRISTO",
    name_key: "region.r03.name",
    description_key: "region.r03.description",
    tier: 1,
    connections: ["R02_CHATEAU_DIF", "R04_ROME", "R10_MEDITERRANEE"],
    locked: false,
    gate: Always,
)
REGION

echo "=== Regions created ==="
echo "=== Now creating bestiary entries ==="

# ---- 102 Bestiary Entries ----
# We have 30 existing. Need 72 more = total 102.
# Let me just create a batch of new enemies for regions R04-R15.

# Regions and their enemy themes:
# R04 Rome: gladiators, roman guards, street vendors
# R05 Paris Faubourg: revolutionaries, spies, agents
# R06 Paris Salon: aristocrats, duelists, conspirators
# R07 Normandy: peasants, bandits, smugglers
# R08 Lyon: silk merchants, weavers, city guards
# R09 Strasbourg: mercenaries, border guards, monks
# R10 Mediterranee: pirates, corsairs, sea creatures
# R11 Orient: merchants, janissaries, viziers
# R12 Greece: rebels, klephts, oracles
# R13 Albania: mountaineers, chieftains, mercenaries
# R14 Morcerf Estate: guards, servants, officers
# R15 Villefort Mansion: guards, judges, executioners

cat > content/bestiary/enm_rat_swarm.ron << 'ENEMY'
(
    id: "ENM_RAT_SWARM",
    name_key: "enm.rat_swarm.name",
    family: VERMIN,
    region_affinity: ["R02_CHATEAU_DIF", "R06_PARIS_SALON"],
    gate: Always,
    stats: Stats(hp: 8, atk: 4, def: 2, spd: 25),
    resist: [TERROR],
    abilities: ["ABL_BITE", "ABL_SWARM"],
    loot: [],
    xp: 4,
    tier: 1,
    sprite: "rat_swarm",
)
ENEMY

cat > content/bestiary/enm_giant_rat.ron << 'ENEMY'
(
    id: "ENM_GIANT_RAT",
    name_key: "enm.giant_rat.name",
    family: VERMIN,
    region_affinity: ["R02_CHATEAU_DIF", "R06_PARIS_SALON"],
    gate: Always,
    stats: Stats(hp: 20, atk: 8, def: 4, spd: 18),
    resist: [TERROR],
    abilities: ["ABL_BITE"],
    loot: [("ITM_POTION", 1)],
    xp: 8,
    tier: 1,
    sprite: "giant_rat",
)
ENEMY

cat > content/bestiary/enm_chateau_spider.ron << 'ENEMY'
(
    id: "ENM_CHATEAU_SPIDER",
    name_key: "enm.chateau_spider.name",
    family: VERMIN,
    region_affinity: ["R02_CHATEAU_DIF"],
    gate: Always,
    stats: Stats(hp: 12, atk: 10, def: 3, spd: 20),
    resist: [TERROR],
    abilities: ["ABL_BITE", "ABL_POISON_FANG"],
    loot: [],
    xp: 6,
    tier: 1,
    sprite: "chateau_spider",
)
ENEMY

# Boss enemies
cat > content/bestiary/enm_abbot_faria.ron << 'ENEMY'
(
    id: "ENM_ABBE_FARIA",
    name_key: "enm.abbe_faria.name",
    family: BOSS,
    region_affinity: ["R02_CHATEAU_DIF"],
    gate: Always,
    stats: Stats(hp: 200, atk: 20, def: 15, spd: 10),
    resist: [BROKEN_GUARD, TERROR],
    abilities: ["ABL_FENCING_LUNGE", "ABL_HIST_INSPIRE", "ABL_LANG_PERSUADE"],
    loot: [("ITM_TREASURE_MAP", 1)],
    xp: 100,
    tier: 2,
    sprite: "abbe_faria",
)
ENEMY

# R04 Rome enemies
for e in gladiator roman_guard street_vendor merchant_guard papal_soldier catacomb_scorpion forum_thief colosseum_beast; do
  id="ENM_$(echo $e | tr '[:lower:]' '[:upper:]')"
  case $e in
    gladiator) 
      fam=TROOP; hp=55; atk=20; def=12; spd=10; ab="ABL_FENCING_LUNGE"; resist="[]"; xp=22; tier=2; aff="R04_ROME";;
    roman_guard)
      fam=TROOP; hp=45; atk=16; def=14; spd=8; ab="ABL_FENCING_RIPOSTE"; resist="[BROKEN_GUARD]"; xp=18; tier=2; aff="R04_ROME";;
    street_vendor)
      fam=CRIMINAL; hp=25; atk=8; def=4; spd=14; ab="ABL_ECON_BRIBE"; resist="[]"; xp=10; tier=1; aff="R04_ROME";;
    merchant_guard)
      fam=MAN_AT_ARMS; hp=40; atk=14; def=12; spd=8; ab="ABL_FENCING_RIPOSTE"; resist="[]"; xp=16; tier=2; aff="R04_ROME";;
    papal_soldier)
      fam=TROOP; hp=50; atk=18; def=16; spd=9; ab="ABL_FENCING_FEINT"; resist="[BROKEN_GUARD]"; xp=24; tier=2; aff="R04_ROME";;
    catacomb_scorpion)
      fam=VERMIN; hp=18; atk=14; def=3; spd=22; ab="ABL_POISON_FANG"; resist="[TERROR]"; xp=9; tier=1; aff="R04_ROME";;
    forum_thief)
      fam=CRIMINAL; hp=22; atk=10; def=4; spd=16; ab="ABL_LANG_PERSUADE"; resist="[]"; xp=8; tier=1; aff="R04_ROME";;
    colosseum_beast)
      fam=BEAST; hp=70; atk=22; def=8; spd=12; ab="ABL_BITE"; resist="[TERROR]"; xp=30; tier=2; aff="R04_ROME";;
  esac
  cat > "content/bestiary/enm_${e}.ron" << ENEMY
(
    id: "${id}",
    name_key: "enm.${e}.name",
    family: ${fam},
    region_affinity: ["${aff}"],
    gate: Always,
    stats: Stats(hp: ${hp}, atk: ${atk}, def: ${def}, spd: ${spd}),
    resist: ${resist},
    abilities: ["${ab}"],
    loot: [],
    xp: ${xp},
    tier: ${tier},
    sprite: "${e}",
)
ENEMY
done

# R05 Paris Faubourg enemies
for e in revolutionary spy paris_agent street_preacher faubourg_thug printer rebel_leader hidden_informant; do
  id="ENM_$(echo $e | tr '[:lower:]' '[:upper:]')"
  case $e in
    revolutionary)
      fam=BANDIT; hp=35; atk=14; def=6; spd=12; ab="ABL_HIST_INSPIRE"; resist="[]"; xp=15; tier=2; aff="R05_PARIS_FAUBOURG";;
    spy)
      fam=CRIMINAL; hp=28; atk=12; def=5; spd=18; ab="ABL_LANG_DECIPHER"; resist="[]"; xp=20; tier=2; aff="R05_PARIS_FAUBOURG";;
    paris_agent)
      fam=MAN_AT_ARMS; hp=40; atk=16; def=10; spd=14; ab="ABL_FENCING_LUNGE"; resist="[]"; xp=22; tier=2; aff="R05_PARIS_FAUBOURG";;
    street_preacher)
      fam=CRIMINAL; hp=20; atk=6; def=4; spd=10; ab="ABL_LANG_PERSUADE"; resist="[]"; xp=8; tier=1; aff="R05_PARIS_FAUBOURG";;
    faubourg_thug)
      fam=BANDIT; hp=38; atk=16; def=8; spd=10; ab="ABL_ECON_BRIBE"; resist="[]"; xp=14; tier=2; aff="R05_PARIS_FAUBOURG";;
    printer)
      fam=CRIMINAL; hp=18; atk=6; def=3; spd=8; ab="ABL_CHEM_FLASH"; resist="[]"; xp=6; tier=1; aff="R05_PARIS_FAUBOURG";;
    rebel_leader)
      fam=BANDIT; hp=55; atk=20; def=12; spd=14; ab="ABL_HIST_INSPIRE"; resist="[BROKEN_GUARD]"; xp=28; tier=3; aff="R05_PARIS_FAUBOURG";;
    hidden_informant)
      fam=CRIMINAL; hp=24; atk=10; def=4; spd=16; ab="ABL_LANG_DECIPHER"; resist="[]"; xp=12; tier=2; aff="R05_PARIS_FAUBOURG";;
  esac
  cat > "content/bestiary/enm_${e}.ron" << ENEMY
(
    id: "${id}",
    name_key: "enm.${e}.name",
    family: ${fam},
    region_affinity: ["${aff}"],
    gate: Always,
    stats: Stats(hp: ${hp}, atk: ${atk}, def: ${def}, spd: ${spd}),
    resist: ${resist},
    abilities: ["${ab}"],
    loot: [],
    xp: ${xp},
    tier: ${tier},
    sprite: "${e}",
)
ENEMY
done

# R06 Paris Salon enemies
for e in aristocrat duelist conspirator salon_hostess royal_guard masked_stranger court_intriguer dancing_master; do
  id="ENM_$(echo $e | tr '[:lower:]' '[:upper:]')"
  case $e in
    aristocrat)
      fam=MAN_AT_ARMS; hp=30; atk=10; def=8; spd=10; ab="ABL_ECON_FUND"; resist="[]"; xp=14; tier=2; aff="R06_PARIS_SALON";;
    duelist)
      fam=MAN_AT_ARMS; hp=45; atk=22; def=10; spd=18; ab="ABL_FENCING_FEINT"; resist="[]"; xp=26; tier=3; aff="R06_PARIS_SALON";;
    conspirator)
      fam=CRIMINAL; hp=32; atk=14; def=6; spd=14; ab="ABL_LANG_PERSUADE"; resist="[]"; xp=18; tier=3; aff="R06_PARIS_SALON";;
    salon_hostess)
      fam=CRIMINAL; hp=20; atk=6; def=4; spd=12; ab="ABL_LANG_DECIPHER"; resist="[]"; xp=10; tier=2; aff="R06_PARIS_SALON";;
    royal_guard)
      fam=TROOP; hp=55; atk=20; def=18; spd=10; ab="ABL_FENCING_RIPOSTE"; resist="[BROKEN_GUARD]"; xp=28; tier=3; aff="R06_PARIS_SALON";;
    masked_stranger)
      fam=CRIMINAL; hp=35; atk=16; def=8; spd=18; ab="ABL_CHEM_FLASH"; resist="[]"; xp=20; tier=3; aff="R06_PARIS_SALON";;
    court_intriguer)
      fam=CRIMINAL; hp=28; atk=12; def=6; spd=15; ab="ABL_LANG_PERSUADE"; resist="[]"; xp=16; tier=3; aff="R06_PARIS_SALON";;
    dancing_master)
      fam=MAN_AT_ARMS; hp=35; atk=14; def=8; spd=20; ab="ABL_FENCING_FEINT"; resist="[]"; xp=18; tier=2; aff="R06_PARIS_SALON";;
  esac
  cat > "content/bestiary/enm_${e}.ron" << ENEMY
(
    id: "${id}",
    name_key: "enm.${e}.name",
    family: ${fam},
    region_affinity: ["${aff}"],
    gate: Always,
    stats: Stats(hp: ${hp}, atk: ${atk}, def: ${def}, spd: ${spd}),
    resist: ${resist},
    abilities: ["${ab}"],
    loot: [],
    xp: ${xp},
    tier: ${tier},
    sprite: "${e}",
)
ENEMY
done

# R07 Normandy enemies
for e in peasant smuggler_normandy cattle_driver coastal_patrol monastery_monk abbot local_militia; do
  id="ENM_$(echo $e | tr '[:lower:]' '[:upper:]')"
  case $e in
    peasant)
      fam=CRIMINAL; hp=15; atk=6; def=3; spd=8; ab="ABL_LANG_PERSUADE"; resist="[]"; xp=5; tier=1; aff="R07_NORMANDY";;
    smuggler_normandy)
      fam=SMUGGLER; hp=35; atk=14; def=7; spd=14; ab="ABL_ECON_BRIBE"; resist="[]"; xp=16; tier=2; aff="R07_NORMANDY";;
    cattle_driver)
      fam=CRIMINAL; hp=30; atk=12; def=8; spd=8; ab="ABL_CHEM_STIM"; resist="[]"; xp=10; tier=1; aff="R07_NORMANDY";;
    coastal_patrol)
      fam=TROOP; hp=40; atk=15; def=12; spd=10; ab="ABL_FENCING_RIPOSTE"; resist="[]"; xp=18; tier=2; aff="R07_NORMANDY";;
    monastery_monk)
      fam=CRIMINAL; hp=25; atk=8; def=6; spd=8; ab="ABL_NATPHIL_ANALYZE"; resist="[]"; xp=12; tier=1; aff="R07_NORMANDY";;
    abbot)
      fam=MAN_AT_ARMS; hp=40; atk=12; def=10; spd=8; ab="ABL_HIST_INSPIRE"; resist="[]"; xp=20; tier=2; aff="R07_NORMANDY";;
    local_militia)
      fam=TROOP; hp=35; atk=12; def=10; spd=8; ab="ABL_FENCING_LUNGE"; resist="[]"; xp=14; tier=2; aff="R07_NORMANDY";;
  esac
  cat > "content/bestiary/enm_${e}.ron" << ENEMY
(
    id: "${id}",
    name_key: "enm.${e}.name",
    family: ${fam},
    region_affinity: ["${aff}"],
    gate: Always,
    stats: Stats(hp: ${hp}, atk: ${atk}, def: ${def}, spd: ${spd}),
    resist: ${resist},
    abilities: ["${ab}"],
    loot: [],
    xp: ${xp},
    tier: ${tier},
    sprite: "${e}",
)
ENEMY
done

# R08 Lyon enemies
for e in silk_merchant weaver city_guard_lyon trade_negotiator river_bandit merchant_caravan guild_master; do
  id="ENM_$(echo $e | tr '[:lower:]' '[:upper:]')"
  case $e in
    silk_merchant)
      fam=CRIMINAL; hp=22; atk=8; def=5; spd=10; ab="ABL_ECON_BRIBE"; resist="[]"; xp=10; tier=2; aff="R08_LYON";;
    weaver)
      fam=CRIMINAL; hp=18; atk=6; def=3; spd=8; ab="ABL_CHEM_STIM"; resist="[]"; xp=6; tier=1; aff="R08_LYON";;
    city_guard_lyon)
      fam=TROOP; hp=42; atk=16; def=14; spd=10; ab="ABL_FENCING_RIPOSTE"; resist="[]"; xp=18; tier=2; aff="R08_LYON";;
    trade_negotiator)
      fam=CRIMINAL; hp=20; atk=8; def=4; spd=12; ab="ABL_ECON_FUND"; resist="[]"; xp=12; tier=2; aff="R08_LYON";;
    river_bandit)
      fam=BANDIT; hp=35; atk=15; def=7; spd=14; ab="ABL_ECON_BRIBE"; resist="[]"; xp=15; tier=2; aff="R08_LYON";;
    merchant_caravan)
      fam=MAN_AT_ARMS; hp=38; atk=14; def=12; spd=8; ab="ABL_FENCING_LUNGE"; resist="[]"; xp=16; tier=2; aff="R08_LYON";;
    guild_master)
      fam=MAN_AT_ARMS; hp=50; atk=18; def=14; spd=12; ab="ABL_ECON_FUND"; resist="[BROKEN_GUARD]"; xp=24; tier=3; aff="R08_LYON";;
  esac
  cat > "content/bestiary/enm_${e}.ron" << ENEMY
(
    id: "${id}",
    name_key: "enm.${e}.name",
    family: ${fam},
    region_affinity: ["${aff}"],
    gate: Always,
    stats: Stats(hp: ${hp}, atk: ${atk}, def: ${def}, spd: ${spd}),
    resist: ${resist},
    abilities: ["${ab}"],
    loot: [],
    xp: ${xp},
    tier: ${tier},
    sprite: "${e}",
)
ENEMY
done

# R09 Strasbourg enemies
for e in mercenary border_guard monastery_guard wandering_knight german_trader cathedral_warden forest_bandit; do
  id="ENM_$(echo $e | tr '[:lower:]' '[:upper:]')"
  case $e in
    mercenary)
      fam=BANDIT; hp=45; atk=18; def=10; spd=14; ab="ABL_FENCING_LUNGE"; resist="[]"; xp=20; tier=3; aff="R09_STRASBOURG";;
    border_guard)
      fam=TROOP; hp=45; atk=16; def=15; spd=9; ab="ABL_FENCING_RIPOSTE"; resist="[]"; xp=20; tier=3; aff="R09_STRASBOURG";;
    monastery_guard)
      fam=MAN_AT_ARMS; hp=40; atk=14; def=12; spd=8; ab="ABL_FENCING_RIPOSTE"; resist="[]"; xp=18; tier=2; aff="R09_STRASBOURG";;
    wandering_knight)
      fam=MAN_AT_ARMS; hp=55; atk=22; def=16; spd=12; ab="ABL_FENCING_FEINT"; resist="[BROKEN_GUARD]"; xp=28; tier=3; aff="R09_STRASBOURG";;
    german_trader)
      fam=CRIMINAL; hp=25; atk=8; def=5; spd=10; ab="ABL_ECON_FUND"; resist="[]"; xp=10; tier=2; aff="R09_STRASBOURG";;
    cathedral_warden)
      fam=MAN_AT_ARMS; hp=50; atk=18; def=16; spd=8; ab="ABL_HIST_INSPIRE"; resist="[]"; xp=22; tier=3; aff="R09_STRASBOURG";;
    forest_bandit)
      fam=BANDIT; hp=38; atk=16; def=8; spd=16; ab="ABL_ECON_BRIBE"; resist="[]"; xp=16; tier=2; aff="R09_STRASBOURG";;
  esac
  cat > "content/bestiary/enm_${e}.ron" << ENEMY
(
    id: "${id}",
    name_key: "enm.${e}.name",
    family: ${fam},
    region_affinity: ["${aff}"],
    gate: Always,
    stats: Stats(hp: ${hp}, atk: ${atk}, def: ${def}, spd: ${spd}),
    resist: ${resist},
    abilities: ["${ab}"],
    loot: [],
    xp: ${xp},
    tier: ${tier},
    sprite: "${e}",
)
ENEMY
done

# R10 Mediterranee enemies
for e in corsair pirate_crew sea_serpent merchant_galley slave_trader turkish_captain leviathan; do
  id="ENM_$(echo $e | tr '[:lower:]' '[:upper:]')"
  case $e in
    corsair)
      fam=SEA; hp=50; atk=20; def=10; spd=16; ab="ABL_FENCING_LUNGE"; resist="[]"; xp=24; tier=3; aff="R10_MEDITERRANEE";;
    pirate_crew)
      fam=SEA; hp=40; atk=16; def=8; spd=14; ab="ABL_ECON_BRIBE"; resist="[]"; xp=18; tier=3; aff="R10_MEDITERRANEE";;
    sea_serpent)
      fam=BEAST; hp=80; atk=25; def=12; spd=15; ab="ABL_BITE"; resist="[TERROR]"; xp=35; tier=4; aff="R10_MEDITERRANEE";;
    merchant_galley)
      fam=SEA; hp=35; atk=12; def=10; spd=8; ab="ABL_ECON_FUND"; resist="[]"; xp=14; tier=2; aff="R10_MEDITERRANEE";;
    slave_trader)
      fam=CRIMINAL; hp=45; atk=18; def=10; spd=12; ab="ABL_ECON_BRIBE"; resist="[]"; xp=22; tier=3; aff="R10_MEDITERRANEE";;
    turkish_captain)
      fam=TROOP; hp=55; atk=20; def=14; spd=12; ab="ABL_FENCING_RIPOSTE"; resist="[BROKEN_GUARD]"; xp=28; tier=3; aff="R10_MEDITERRANEE";;
    leviathan)
      fam=BEAST; hp=120; atk=30; def=18; spd=10; ab="ABL_BITE"; resist="[TERROR, BROKEN_GUARD]"; xp=50; tier=4; aff="R10_MEDITERRANEE";;
  esac
  cat > "content/bestiary/enm_${e}.ron" << ENEMY
(
    id: "${id}",
    name_key: "enm.${e}.name",
    family: ${fam},
    region_affinity: ["${aff}"],
    gate: Always,
    stats: Stats(hp: ${hp}, atk: ${atk}, def: ${def}, spd: ${spd}),
    resist: ${resist},
    abilities: ["${ab}"],
    loot: [],
    xp: ${xp},
    tier: ${tier},
    sprite: "${e}",
)
ENEMY
done

# R11 Orient enemies
for e in janissary grand_vizier oriental_merchant desert_scorpion palace_guard bazaar_thief caravan_guard; do
  id="ENM_$(echo $e | tr '[:lower:]' '[:upper:]')"
  case $e in
    janissary)
      fam=TROOP; hp=55; atk=22; def=16; spd=12; ab="ABL_FENCING_LUNGE"; resist="[BROKEN_GUARD]"; xp=28; tier=3; aff="R11_ORIENT";;
    grand_vizier)
      fam=MAN_AT_ARMS; hp=60; atk=24; def=16; spd=14; ab="ABL_LANG_PERSUADE"; resist="[BROKEN_GUARD]"; xp=32; tier=4; aff="R11_ORIENT";;
    oriental_merchant)
      fam=CRIMINAL; hp=30; atk=10; def=6; spd=10; ab="ABL_ECON_FUND"; resist="[]"; xp=14; tier=2; aff="R11_ORIENT";;
    desert_scorpion)
      fam=VERMIN; hp=22; atk=16; def=4; spd=24; ab="ABL_POISON_FANG"; resist="[TERROR]"; xp=12; tier=2; aff="R11_ORIENT";;
    palace_guard)
      fam=TROOP; hp=50; atk=20; def=18; spd=10; ab="ABL_FENCING_RIPOSTE"; resist="[]"; xp=24; tier=3; aff="R11_ORIENT";;
    bazaar_thief)
      fam=CRIMINAL; hp=20; atk=10; def=3; spd=18; ab="ABL_LANG_DECIPHER"; resist="[]"; xp=8; tier=1; aff="R11_ORIENT";;
    caravan_guard)
      fam=MAN_AT_ARMS; hp=42; atk=16; def=14; spd=10; ab="ABL_FENCING_RIPOSTE"; resist="[]"; xp=18; tier=2; aff="R11_ORIENT";;
  esac
  cat > "content/bestiary/enm_${e}.ron" << ENEMY
(
    id: "${id}",
    name_key: "enm.${e}.name",
    family: ${fam},
    region_affinity: ["${aff}"],
    gate: Always,
    stats: Stats(hp: ${hp}, atk: ${atk}, def: ${def}, spd: ${spd}),
    resist: ${resist},
    abilities: ["${ab}"],
    loot: [],
    xp: ${xp},
    tier: ${tier},
    sprite: "${e}",
)
ENEMY
done

# R12 Greece enemies
for e in greek_rebel klepht oracle_priestess spartan_ghost aegean_sailor cult_member olympian_sentinel; do
  id="ENM_$(echo $e | tr '[:lower:]' '[:upper:]')"
  case $e in
    greek_rebel)
      fam=BANDIT; hp=45; atk=18; def=10; spd=14; ab="ABL_HIST_INSPIRE"; resist="[]"; xp=22; tier=3; aff="R12_GREECE";;
    klepht)
      fam=BANDIT; hp=35; atk=16; def=8; spd=18; ab="ABL_ECON_BRIBE"; resist="[]"; xp=18; tier=3; aff="R12_GREECE";;
    oracle_priestess)
      fam=CRIMINAL; hp=30; atk=12; def=6; spd=14; ab="ABL_NATPHIL_ANALYZE"; resist="[]"; xp=20; tier=3; aff="R12_GREECE";;
    spartan_ghost)
      fam=HAZARD; hp=60; atk=24; def=20; spd=12; ab="ABL_FENCING_LUNGE"; resist="[TERROR]"; xp=35; tier=4; aff="R12_GREECE";;
    aegean_sailor)
      fam=SEA; hp=35; atk=14; def=8; spd=12; ab="ABL_FENCING_RIPOSTE"; resist="[]"; xp=16; tier=2; aff="R12_GREECE";;
    cult_member)
      fam=CRIMINAL; hp=32; atk=14; def=6; spd=12; ab="ABL_LANG_PERSUADE"; resist="[]"; xp=16; tier=3; aff="R12_GREECE";;
    olympian_sentinel)
      fam=HAZARD; hp=75; atk=26; def=18; spd=14; ab="ABL_HIST_INSPIRE"; resist="[TERROR, BROKEN_GUARD]"; xp=40; tier=4; aff="R12_GREECE";;
  esac
  cat > "content/bestiary/enm_${e}.ron" << ENEMY
(
    id: "${id}",
    name_key: "enm.${e}.name",
    family: ${fam},
    region_affinity: ["${aff}"],
    gate: Always,
    stats: Stats(hp: ${hp}, atk: ${atk}, def: ${def}, spd: ${spd}),
    resist: ${resist},
    abilities: ["${ab}"],
    loot: [],
    xp: ${xp},
    tier: ${tier},
    sprite: "${e}",
)
ENEMY
done

# R13 Albania enemies
for e in mountaineer clan_chieftain albanian_mercenary mountain_pass_bandit highland_shaman fortified_outpost greek_merc; do
  id="ENM_$(echo $e | tr '[:lower:]' '[:upper:]')"
  case $e in
    mountaineer)
      fam=BANDIT; hp=38; atk=16; def=10; spd=14; ab="ABL_FENCING_LUNGE"; resist="[]"; xp=18; tier=3; aff="R13_ALBANIA";;
    clan_chieftain)
      fam=BANDIT; hp=55; atk=22; def=14; spd=12; ab="ABL_HIST_INSPIRE"; resist="[BROKEN_GUARD]"; xp=30; tier=4; aff="R13_ALBANIA";;
    albanian_mercenary)
      fam=BANDIT; hp=45; atk=18; def=12; spd=12; ab="ABL_FENCING_RIPOSTE"; resist="[]"; xp=22; tier=3; aff="R13_ALBANIA";;
    mountain_pass_bandit)
      fam=BANDIT; hp=40; atk=16; def=8; spd=16; ab="ABL_ECON_BRIBE"; resist="[]"; xp=16; tier=3; aff="R13_ALBANIA";;
    highland_shaman)
      fam=CRIMINAL; hp=30; atk=12; def=6; spd=14; ab="ABL_CHEM_FLASH"; resist="[]"; xp=20; tier=3; aff="R13_ALBANIA";;
    fortified_outpost)
      fam=TROOP; hp=50; atk=18; def=16; spd=8; ab="ABL_FENCING_RIPOSTE"; resist="[]"; xp=24; tier=3; aff="R13_ALBANIA";;
    greek_merc)
      fam=BANDIT; hp=42; atk=18; def=10; spd=14; ab="ABL_FENCING_LUNGE"; resist="[]"; xp=20; tier=3; aff="R13_ALBANIA";;
  esac
  cat > "content/bestiary/enm_${e}.ron" << ENEMY
(
    id: "${id}",
    name_key: "enm.${e}.name",
    family: ${fam},
    region_affinity: ["${aff}"],
    gate: Always,
    stats: Stats(hp: ${hp}, atk: ${atk}, def: ${def}, spd: ${spd}),
    resist: ${resist},
    abilities: ["${ab}"],
    loot: [],
    xp: ${xp},
    tier: ${tier},
    sprite: "${e}",
)
ENEMY
done

# R14 Morcerf Estate enemies
for e in estate_guard morcerf_officer stable_hand butler estate_steward fernand_bodyguard albert_friend; do
  id="ENM_$(echo $e | tr '[:lower:]' '[:upper:]')"
  case $e in
    estate_guard)
      fam=MAN_AT_ARMS; hp=50; atk=18; def=14; spd=10; ab="ABL_FENCING_RIPOSTE"; resist="[]"; xp=22; tier=3; aff="R14_MORCERF_ESTATE";;
    morcerf_officer)
      fam=TROOP; hp=55; atk=20; def=16; spd=12; ab="ABL_FENCING_LUNGE"; resist="[BROKEN_GUARD]"; xp=26; tier=4; aff="R14_MORCERF_ESTATE";;
    stable_hand)
      fam=CRIMINAL; hp=22; atk=8; def=4; spd=10; ab="ABL_CHEM_STIM"; resist="[]"; xp=8; tier=1; aff="R14_MORCERF_ESTATE";;
    butler)
      fam=MAN_AT_ARMS; hp=35; atk=12; def=10; spd=8; ab="ABL_LANG_PERSUADE"; resist="[]"; xp=14; tier=2; aff="R14_MORCERF_ESTATE";;
    estate_steward)
      fam=CRIMINAL; hp=28; atk=10; def=8; spd=10; ab="ABL_ECON_FUND"; resist="[]"; xp=12; tier=2; aff="R14_MORCERF_ESTATE";;
    fernand_bodyguard)
      fam=MAN_AT_ARMS; hp=60; atk=22; def=18; spd=12; ab="ABL_FENCING_FEINT"; resist="[BROKEN_GUARD]"; xp=30; tier=4; aff="R14_MORCERF_ESTATE";;
    albert_friend)
      fam=MAN_AT_ARMS; hp=40; atk=14; def=8; spd=12; ab="ABL_FENCING_LUNGE"; resist="[]"; xp=16; tier=2; aff="R14_MORCERF_ESTATE";;
  esac
  cat > "content/bestiary/enm_${e}.ron" << ENEMY
(
    id: "${id}",
    name_key: "enm.${e}.name",
    family: ${fam},
    region_affinity: ["${aff}"],
    gate: Always,
    stats: Stats(hp: ${hp}, atk: ${atk}, def: ${def}, spd: ${spd}),
    resist: ${resist},
    abilities: ["${ab}"],
    loot: [],
    xp: ${xp},
    tier: ${tier},
    sprite: "${e}",
)
ENEMY
done

# R15 Villefort Mansion enemies
for e in mansion_guard examining_judge clerk_secretary executioner villefort_spy heloise_servant evidence_keeper; do
  id="ENM_$(echo $e | tr '[:lower:]' '[:upper:]')"
  case $e in
    mansion_guard)
      fam=TROOP; hp=50; atk=18; def=16; spd=10; ab="ABL_FENCING_RIPOSTE"; resist="[]"; xp=22; tier=3; aff="R15_VILLEFORT_MANSION";;
    examining_judge)
      fam=MAN_AT_ARMS; hp=45; atk=16; def=12; spd=10; ab="ABL_LANG_PERSUADE"; resist="[BROKEN_GUARD]"; xp=24; tier=3; aff="R15_VILLEFORT_MANSION";;
    clerk_secretary)
      fam=CRIMINAL; hp=20; atk=6; def=4; spd=8; ab="ABL_LANG_DECIPHER"; resist="[]"; xp=8; tier=1; aff="R15_VILLEFORT_MANSION";;
    executioner)
      fam=MAN_AT_ARMS; hp=65; atk=24; def=14; spd=8; ab="ABL_FENCING_LUNGE"; resist="[BROKEN_GUARD]"; xp=32; tier=4; aff="R15_VILLEFORT_MANSION";;
    villefort_spy)
      fam=CRIMINAL; hp=28; atk=14; def=5; spd=18; ab="ABL_LANG_DECIPHER"; resist="[]"; xp=18; tier=3; aff="R15_VILLEFORT_MANSION";;
    heloise_servant)
      fam=CRIMINAL; hp=18; atk=6; def=3; spd=10; ab="ABL_CHEM_FLASH"; resist="[]"; xp=8; tier=1; aff="R15_VILLEFORT_MANSION";;
    evidence_keeper)
      fam=MAN_AT_ARMS; hp=40; atk=14; def=12; spd=8; ab="ABL_NATPHIL_ANALYZE"; resist="[]"; xp=16; tier=2; aff="R15_VILLEFORT_MANSION";;
  esac
  cat > "content/bestiary/enm_${e}.ron" << ENEMY
(
    id: "${id}",
    name_key: "enm.${e}.name",
    family: ${fam},
    region_affinity: ["${aff}"],
    gate: Always,
    stats: Stats(hp: ${hp}, atk: ${atk}, def: ${def}, spd: ${spd}),
    resist: ${resist},
    abilities: ["${ab}"],
    loot: [],
    xp: ${xp},
    tier: ${tier},
    sprite: "${e}",
)
ENEMY
done

# More boss enemies
cat > content/bestiary/enm_fernand_mondego.ron << 'ENEMY'
(
    id: "ENM_FERNAND_MONDEGO",
    name_key: "enm.fernand_mondego.name",
    family: BOSS,
    region_affinity: ["R14_MORCERF_ESTATE"],
    gate: Always,
    stats: Stats(hp: 300, atk: 35, def: 20, spd: 14),
    resist: [BROKEN_GUARD],
    abilities: ["ABL_FENCING_LUNGE", "ABL_FENCING_RIPOSTE", "ABL_FENCING_FEINT"],
    loot: [("ITM_HI_POTION", 3)],
    xp: 200,
    tier: 5,
    sprite: "fernand_mondego",
)
ENEMY

cat > content/bestiary/enm_danglars_agent.ron << 'ENEMY'
(
    id: "ENM_DANGLARS_AGENT",
    name_key: "enm.danglars_agent.name",
    family: BOSS,
    region_affinity: ["R08_LYON"],
    gate: Always,
    stats: Stats(hp: 250, atk: 28, def: 18, spd: 16),
    resist: [BROKEN_GUARD],
    abilities: ["ABL_ECON_BRIBE", "ABL_ECON_FUND", "ABL_HIST_INSPIRE"],
    loot: [("ITM_PANACEA", 1)],
    xp: 180,
    tier: 5,
    sprite: "danglars_agent",
)
ENEMY

cat > content/bestiary/enm_villefort_agent.ron << 'ENEMY'
(
    id: "ENM_VILLEFORT_AGENT",
    name_key: "enm.villefort_agent.name",
    family: BOSS,
    region_affinity: ["R15_VILLEFORT_MANSION"],
    gate: Always,
    stats: Stats(hp: 280, atk: 32, def: 22, spd: 12),
    resist: [BROKEN_GUARD],
    abilities: ["ABL_CHEM_FLASH", "ABL_FENCING_LUNGE"],
    loot: [("ITM_EDOUARD_LOCKET", 1)],
    xp: 190,
    tier: 5,
    sprite: "villefort_agent",
)
ENEMY

cat > content/bestiary/enm_final_monte_cristo.ron << 'ENEMY'
(
    id: "ENM_FINAL_MONTE_CRISTO",
    name_key: "enm.final_monte_cristo.name",
    family: BOSS,
    region_affinity: ["R06_PARIS_SALON"],
    gate: Always,
    stats: Stats(hp: 500, atk: 40, def: 25, spd: 18),
    resist: [BROKEN_GUARD, TERROR],
    abilities: ["ABL_FENCING_LUNGE", "ABL_FENCING_RIPOSTE", "ABL_FENCING_FEINT", "ABL_HIST_INSPIRE"],
    loot: [("ITM_PANACEA", 3)],
    xp: 500,
    tier: 6,
    sprite: "final_monte_cristo",
)
ENEMY

echo "=== Bestiary entries created ==="

# ---- 45 Spawn Tables (need 2 RTs per region, 3 for some - total 45) ----
# Each region gets 3 spawn tables on average. Some get 2.
for r in 01 02 03 04 05 06 07 08 09 10 11 12 13 14 15; do
  case $r in
    01) enemies="ENM_STREET_URCHIN,ENM_STREET_THUG,ENM_SMUGGLER,ENM_HARBOUR_THUG,ENM_GENDARME,ENM_BAR_BOUNCER,ENM_FISHERMAN,ENM_DOCK_WORKER,ENM_SAILOR,ENM_BEGGAR";;
    02) enemies="ENM_PRISON_GUARD,ENM_PRISON_DOG,ENM_JAILER,ENM_SOLITARY_INMATE,ENM_CONVICT,ENM_CELL_RAT,ENM_CELL_SPIDER,ENM_DUNGEON_MASTER,ENM_CHAIN_GANG,ENM_TURNKEY,ENM_WARDEN,ENM_CHATEAU_SPIDER,ENM_GIANT_RAT,ENM_RAT_SWARM,ENM_INTERROGATOR,ENM_ANTOINE_STRANGLER,ENM_EXECUTIONER,ENM_BRUTE,ENM_CORRIDOR_PATROL,ENM_LE_RONGEUR";;
    03) enemies="ENM_GUARD,ENM_SMUGGLER,ENM_STREET_THUG,ENM_TOWER_SENTRY,ENM_HARBOUR_THUG,ENM_DOCK_WORKER,ENM_FISHERMAN";;
    04) enemies="ENM_GLADIATOR,ENM_ROMAN_GUARD,ENM_STREET_VENDOR,ENM_MERCHANT_GUARD,ENM_PAPAL_SOLDIER,ENM_CATACOMB_SCORPION,ENM_FORUM_THIEF,ENM_COLOSSEUM_BEAST";;
    05) enemies="ENM_REVOLUTIONARY,ENM_SPY,ENM_PARIS_AGENT,ENM_STREET_PREACHER,ENM_FAUBOURG_THUG,ENM_PRINTER,ENM_REBEL_LEADER,ENM_HIDDEN_INFORMANT";;
    06) enemies="ENM_ARISTOCRAT,ENM_DUELIST,ENM_CONSPIRATOR,ENM_SALON_HOSTESS,ENM_ROYAL_GUARD,ENM_MASKED_STRANGER,ENM_COURT_INTRIGUER,ENM_DANCING_MASTER,ENM_FINAL_MONTE_CRISTO";;
    07) enemies="ENM_PEASANT,ENM_SMUGGLER,ENM_CATTLE_DRIVER,ENM_COASTAL_PATROL,ENM_MONASTERY_MONK,ENM_ABBOT,ENM_LOCAL_MILITIA";;
    08) enemies="ENM_SILK_MERCHANT,ENM_WEAVER,ENM_CITY_GUARD_LYON,ENM_TRADE_NEGOTIATOR,ENM_RIVER_BANDIT,ENM_MERCHANT_CARAVAN,ENM_GUILD_MASTER,ENM_DANGLARS_AGENT";;
    09) enemies="ENM_MERCENARY,ENM_BORDER_GUARD,ENM_MONASTERY_GUARD,ENM_WANDERING_KNIGHT,ENM_GERMAN_TRADER,ENM_CATHEDRAL_WARDEN,ENM_FOREST_BANDIT";;
    10) enemies="ENM_CORSAIR,ENM_PIRATE_CREW,ENM_SEA_SERPENT,ENM_MERCHANT_GALLEY,ENM_SLAVE_TRADER,ENM_TURKISH_CAPTAIN,ENM_LEVIATHAN,ENM_SMUGGLER,ENM_SAILOR";;
    11) enemies="ENM_JANISSARY,ENM_GRAND_VIZIER,ENM_ORIENTAL_MERCHANT,ENM_DESERT_SCORPION,ENM_PALACE_GUARD,ENM_BAZAAR_THIEF,ENM_CARAVAN_GUARD";;
    12) enemies="ENM_GREEK_REBEL,ENM_KLEPHT,ENM_ORACLE_PRIESTESS,ENM_SPARTAN_GHOST,ENM_AEGEAN_SAILOR,ENM_CULT_MEMBER,ENM_OLYMPIAN_SENTINEL";;
    13) enemies="ENM_MOUNTAINEER,ENM_CLAN_CHIEFTAIN,ENM_ALBANIAN_MERCENARY,ENM_MOUNTAIN_PASS_BANDIT,ENM_HIGHLAND_SHAMAN,ENM_FORTIFIED_OUTPOST,ENM_GREEK_MERC";;
    14) enemies="ENM_ESTATE_GUARD,ENM_MORCERF_OFFICER,ENM_STABLE_HAND,ENM_BUTLER,ENM_ESTATE_STEWARD,ENM_FERNAND_BODYGUARD,ENM_ALBERT_FRIEND,ENM_FERNAND_MONDEGO";;
    15) enemies="ENM_MANSION_GUARD,ENM_EXAMINING_JUDGE,ENM_CLERK_SECRETARY,ENM_EXECUTIONER,ENM_VILLEFORT_SPY,ENM_HELOISE_SERVANT,ENM_EVIDENCE_KEEPER,ENM_VILLEFORT_AGENT";;
  esac

  # Split enemies into array
  IFS=',' read -ra ENM_ARRAY <<< "$enemies"

  # 3 spawn pools per region
  for pool in 1 2 3; do
    cat > "content/spawn_tables/R${r}-s1-p${pool}.ron" << SPAWNTABLE
SpawnTable(
    region: "R${r}",
    chapter_stage: 1,
    pool: ${pool},
    entries: [
$(for ((i=0; i<${#ENM_ARRAY[@]}; i+=3)); do
  idx=$(( (pool-1) * 3 + i % 3 ))
  [ $idx -ge ${#ENM_ARRAY[@]} ] && continue
  w=$(( (idx + 1) * 3 + 10 ))
  enm=$(echo "${ENM_ARRAY[$idx]}" | tr -d ' ')
  echo "        (enemy: \"${enm}\", weight: ${w}, gate: Always),"
done)
    ],
)
SPAWNTABLE
  done
done

echo "=== Spawn tables created ==="
echo "=== Content generation complete ==="

echo "Counting files..."
echo "Regions: $(ls content/regions/*.ron | wc -l)"
echo "Bestiary: $(ls content/bestiary/*.ron | wc -l)"
echo "Spawn tables: $(ls content/spawn_tables/*.ron | wc -l)"
echo "Scenes: $(find content/scenes -name '*.ron' | wc -l)"
echo "Items: $(ls content/items/*.ron | wc -l)"
