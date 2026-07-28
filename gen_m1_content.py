#!/usr/bin/env python3
"""Add remaining spawn tables and confidence scenes for M1."""
import os

CONTENT = "/root/monte-cristo/content"

# ========== Add 4 more spawn tables (pools 4-5 for R01 and R03) ==========
print("Adding spawn tables...")

# R01 extra pools
for pool in [2, 3]:
    enemies = ["ENM_STREET_URCHIN", "ENM_STREET_THUG", "ENM_SMUGGLER", "ENM_HARBOUR_THUG", 
               "ENM_GENDARME", "ENM_BAR_BOUNCER", "ENM_FISHERMAN", "ENM_DOCKHAND", "ENM_SAILOR", "ENM_BEGGAR"]
    pool_enemies = [e for i, e in enumerate(enemies) if (i % 2) == (pool - 2)]
    with open(f"{CONTENT}/spawn_tables/R01-s1-p{pool}.ron", 'w') as f:
        f.write(f'SpawnTable(\n')
        f.write(f'    region: "R01",\n')
        f.write(f'    chapter_stage: 1,\n')
        f.write(f'    pool: {pool + 10},\n')
        f.write(f'    entries: [\n')
        for eid in pool_enemies:
            weight = max(6, min(30, (abs(hash(eid)) % 25) + 6))
            f.write(f'        (enemy: "{eid}", weight: {weight}, gate: Always),\n')
        f.write(f'    ],\n')
        f.write(f')\n')

# R03 extra pools
for pool in [2, 3]:
    enemies = ["ENM_CELL_RAT", "ENM_PRISON_GUARD", "ENM_JAILER", "ENM_CHAIN_GANG",
               "ENM_DUNGEON_MASTER", "ENM_TOWER_SENTRY", "ENM_WARDEN", "ENM_INTERROGATOR",
               "ENM_TURNKEY", "ENM_GUARD_DOG", "ENM_CONVICT", "ENM_MUTINEER", "ENM_SOLITARY_INMATE",
               "ENM_CELL_SPIDER", "ENM_CORRIDOR_PATROL", "ENM_GUARD", "ENM_EXECUTIONER",
               "ENM_LE_RONGEUR", "ENM_ANTOINE_STRANGLER"]
    pool_enemies = [e for i, e in enumerate(enemies) if i % 2 == (pool - 2)]
    with open(f"{CONTENT}/spawn_tables/R03-s1-p{pool}.ron", 'w') as f:
        f.write(f'SpawnTable(\n')
        f.write(f'    region: "R03",\n')
        f.write(f'    chapter_stage: 1,\n')
        f.write(f'    pool: {pool + 20},\n')
        f.write(f'    entries: [\n')
        for eid in pool_enemies:
            weight = max(6, min(30, (abs(hash(eid)) % 25) + 6))
            f.write(f'        (enemy: "{eid}", weight: {weight}, gate: Always),\n')
        f.write(f'    ],\n')
        f.write(f')\n')

print("Spawn tables added.")

# ========== Add 14 more boss enemies ==========
print("Adding boss enemies...")

bosses = [
    ("enm_boss_rome_colosseum", "ENM_COLOSSEUM_CHAMPION", "R04_ROME", 250, 30, 20, 12,
     ["ABL_FENCING_LUNGE", "ABL_FENCING_RIPOSTE"], 180, 4),
    ("enm_boss_paris_spymaster", "ENM_SPYMASTER", "R06_PARIS_SALON", 220, 28, 15, 18,
     ["ABL_LANG_DECIPHER", "ABL_CHEM_FLASH"], 170, 4),
    ("enm_boss_normandy_smuggler_king", "ENM_SMUGGLER_KING", "R07_NORMANDY", 200, 26, 14, 14,
     ["ABL_ECON_BRIBE", "ABL_FENCING_LUNGE"], 150, 4),
    ("enm_boss_lyon_guildmaster", "ENM_LYON_GUILDMASTER", "R08_LYON", 230, 28, 18, 12,
     ["ABL_ECON_FUND", "ABL_FENCING_RIPOSTE"], 160, 4),
    ("enm_boss_strasbourg_knight", "ENM_STRASBOURG_KNIGHT", "R09_STRASBOURG", 260, 32, 22, 10,
     ["ABL_FENCING_FEINT", "ABL_HIST_INSPIRE"], 190, 4),
    ("enm_boss_med_captain", "ENM_MED_CAPTAIN", "R10_MEDITERRANEE", 240, 30, 16, 16,
     ["ABL_FENCING_LUNGE", "ABL_ECON_BRIBE"], 175, 4),
    ("enm_boss_orient_pasha", "ENM_ORIENT_PASHA", "R11_ORIENT", 280, 34, 20, 14,
     ["ABL_FENCING_LUNGE", "ABL_FENCING_RIPOSTE", "ABL_HIST_INSPIRE"], 200, 5),
    ("enm_boss_greece_tyrant", "ENM_GREECE_TYRANT", "R12_GREECE", 270, 32, 18, 14,
     ["ABL_FENCING_LUNGE", "ABL_HIST_INSPIRE"], 195, 5),
    ("enm_boss_albania_warlord", "ENM_ALBANIA_WARLORD", "R13_ALBANIA", 260, 30, 16, 14,
     ["ABL_ECON_BRIBE", "ABL_FENCING_LUNGE"], 185, 5),
    ("enm_boss_mercedes_protector", "ENM_MERCEDES_PROTECTOR", "R14_MORCERF_ESTATE", 200, 24, 14, 12,
     ["ABL_FENCING_RIPOSTE"], 140, 4),
    ("enm_boss_heloise_confrontation", "ENM_HELOISE_CONFRONT", "R15_VILLEFORT_MANSION", 220, 26, 12, 16,
     ["ABL_CHEM_FLASH", "ABL_LANG_PERSUADE"], 160, 4),
    ("enm_boss_marseille_master", "ENM_MARSEILLE_MASTER", "R01_MARSEILLE", 180, 24, 14, 14,
     ["ABL_FENCING_LUNGE"], 130, 3),
    ("enm_boss_rome_charioteer", "ENM_ROME_CHARIOTEER", "R04_ROME", 210, 28, 12, 20,
     ["ABL_FENCING_FEINT", "ABL_CHEM_STIM"], 155, 3),
    ("enm_boss_greece_philosopher", "ENM_GREECE_PHILOSOPHER", "R12_GREECE", 200, 24, 10, 18,
     ["ABL_NATPHIL_ANALYZE", "ABL_MATH_PROBABILITY"], 145, 3),
]

for (filename, eid, region, hp, atk, df, spd, abilities, xp, tier) in bosses:
    ab_str = ",\n        ".join(f'"{a}"' for a in abilities)
    with open(f"{CONTENT}/bestiary/{filename}.ron", 'w') as f:
        f.write(f'(\n')
        f.write(f'    id: "{eid}",\n')
        f.write(f'    name_key: "enm.{filename.replace("enm_boss_", "")}.name",\n')
        f.write(f'    family: BOSS,\n')
        f.write(f'    region_affinity: ["{region}"],\n')
        f.write(f'    gate: Always,\n')
        f.write(f'    stats: Stats(hp: {hp}, atk: {atk}, def: {df}, spd: {spd}),\n')
        f.write(f'    resist: [BROKEN_GUARD],\n')
        f.write(f'    abilities: [\n        {ab_str},\n    ],\n')
        f.write(f'    loot: [("ITM_HI_POTION", 2)],\n')
        f.write(f'    xp: {xp},\n')
        f.write(f'    tier: {tier},\n')
        f.write(f'    sprite: "{filename.replace("enm_boss_", "")}",\n')
        f.write(f')\n')

print(f"{len(bosses)} boss enemies added.")

# ========== Create confidence scenes (45 total) ==========
print("\nCreating confidence scenes...")

# 45 confidence scenes - trust-building scenes with character interactions
# Distributed across all 7 acts
act_dirs = {
    1: "act1", 2: "act2", 3: "act3", 4: "act4", 5: "act5", 6: "act6", 7: "act7"
}
act_enum = {
    1: "ActI_ARREST", 2: "ActII_CHATEAU", 3: "ActIII_TREASURE",
    4: "ActIV_TOUR", 5: "ActV_PARIS", 6: "ActVI_JUSTICE", 7: "ActVII_FINAL"
}

# Characters
chars = ["CHR_EDMOND", "CHR_ABBE_FARIA", "CHR_HAYDEE", "CHR_MERCEDES", 
         "CHR_ALBERT", "CHR_FERNAND", "CHR_DANGLARS", "CHR_VILLEFORT",
         "CHR_VALENTINE", "CHR_NOIRTIER", "CHR_BERTUCCIO", "CHR_HELOISE"]

# Distribute 45 scenes across acts (roughly 6-7 per act)
scene_acts = [1]*7 + [2]*6 + [3]*6 + [4]*7 + [5]*7 + [6]*6 + [7]*6  # = 45

confidence_count = 0
for i, act_num in enumerate(scene_acts):
    aid = f"CF{i+1:02d}"
    act_name = act_enum[act_num]
    act_dir = act_dirs[act_num]
    
    # Pick characters based on act
    if act_num == 1:
        participants = ['"CHR_EDMOND"', '"CHR_MERCEDES"', '"CHR_FERNAND"']
        trust_char = "CHR_MERCEDES"
    elif act_num == 2:
        participants = ['"CHR_EDMOND"', '"CHR_ABBE_FARIA"']
        trust_char = "CHR_ABBE_FARIA"
    elif act_num == 3:
        participants = ['"CHR_EDMOND"', '"CHR_BERTUCCIO"']
        trust_char = "CHR_BERTUCCIO"
    elif act_num == 4:
        participants = ['"CHR_EDMOND"', '"CHR_HAYDEE"', '"CHR_ALBERT"']
        trust_char = chars[(i * 3) % len(chars)]
    elif act_num == 5:
        participants = ['"CHR_EDMOND"', '"CHR_MERCEDES"', '"CHR_ALBERT"', '"CHR_VILLEFORT"']
        trust_char = chars[(i * 5) % len(chars)]
    elif act_num == 6:
        participants = ['"CHR_EDMOND"', '"CHR_VALENTINE"', '"CHR_NOIRTIER"', '"CHR_HELOISE"']
        trust_char = "CHR_VALENTINE"
    else:
        participants = ['"CHR_EDMOND"', '"CHR_HAYDEE"', '"CHR_MERCEDES"']
        trust_char = chars[(i * 7) % len(chars)]
    
    os.makedirs(f"{CONTENT}/scenes/{act_dir}", exist_ok=True)
    
    with open(f"{CONTENT}/scenes/{act_dir}/scn_confidence_{aid.lower()}.ron", 'w') as f:
        f.write(f'Scene(\n')
        f.write(f'    id: "SCN_CONFIDENCE_{aid}",\n')
        f.write(f'    act: {act_name},\n')
        f.write(f'    participants: [{", ".join(participants)}],\n')
        f.write(f'    nodes: [\n')
        f.write(f'        Node(\n')
        f.write(f'            id: "n0",\n')
        f.write(f'            text_key: "scene.confidence.{aid.lower()}.n0",\n')
        f.write(f'            choices: [\n')
        f.write(f'                Choice(\n')
        f.write(f'                    text_key: "scene.confidence.{aid.lower()}.n0.c1",\n')
        f.write(f'                    to: "n1",\n')
        f.write(f'                    trust: Some([\n')
        f.write(f'                        TrustEffect("{trust_char}", 5),\n')
        f.write(f'                    ]),\n')
        f.write(f'                ),\n')
        f.write(f'                Choice(\n')
        f.write(f'                    text_key: "scene.confidence.{aid.lower()}.n0.c2",\n')
        f.write(f'                    to: "n2",\n')
        f.write(f'                ),\n')
        f.write(f'            ],\n')
        f.write(f'        ),\n')
        f.write(f'        Node(\n')
        f.write(f'            id: "n1",\n')
        f.write(f'            text_key: "scene.confidence.{aid.lower()}.n1",\n')
        f.write(f'            choices: [],\n')
        f.write(f'        ),\n')
        f.write(f'        Node(\n')
        f.write(f'            id: "n2",\n')
        f.write(f'            text_key: "scene.confidence.{aid.lower()}.n2",\n')
        f.write(f'            choices: [],\n')
        f.write(f'        ),\n')
        f.write(f'    ],\n')
        f.write(f'    terminal: false,\n')
        f.write(f')\n')
    confidence_count += 1

print(f"{confidence_count} confidence scenes created.")
print("\nM1 content generation complete!")
