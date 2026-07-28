#!/usr/bin/env python3
"""Generate all 45 spawn tables for Monte Cristo."""
import os
import glob

CONTENT = "/root/monte-cristo/content"

def get_enemy_id(path):
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line.startswith('id:'):
                return line.split('"')[1]
    return None

def get_enemy_affinities(path):
    """Get region_affinity list from an enemy file."""
    with open(path) as f:
        content = f.read()
    # Find the region_affinity line(s)
    lines = content.split('\n')
    in_aff = False
    affs = []
    for line in lines:
        stripped = line.strip()
        if 'region_affinity:' in stripped:
            in_aff = True
            rest = stripped.split(':', 1)[1].strip()
            if rest.startswith('['):
                # Parse inline list
                lst = rest.strip('[],').split(',')
                for item in lst:
                    item = item.strip().strip('"')
                    if item:
                        affs.append(item)
                # Might continue on next line if list is multi-line
                if rest.endswith(']'):
                    in_aff = False
        elif in_aff:
            stripped = line.strip().rstrip(',')
            if stripped.endswith(']'):
                stripped = stripped[:-1]
            item = stripped.strip().strip('"')
            if item:
                affs.append(item)
            if ']' in line:
                in_aff = False
    return affs

# Read all existing enemies
print("Reading existing enemies...")
enemy_affinities = {}
enemy_paths = {}
for f in sorted(glob.glob(f"{CONTENT}/bestiary/enm_*.ron")):
    eid = get_enemy_id(f)
    if eid:
        affs = get_enemy_affinities(f)
        enemy_affinities[eid] = affs
        enemy_paths[eid] = f

print(f"Total enemies: {len(enemy_affinities)}")

# Map enemies to regions based on their affinities
# Key: short region ID (R01, R02, ...), Value: list of enemy IDs
region_enemies = {}
for eid, affs in enemy_affinities.items():
    for aff in affs:
        # Extract short region ID (e.g., "R01" from "R01_MARSEILLE")
        parts = aff.split('_')
        if parts and parts[0].startswith('R'):
            short = parts[0]
            if len(parts[0]) == 3:  # R01, R02, etc.
                if short not in region_enemies:
                    region_enemies[short] = []
                if eid not in region_enemies[short]:
                    region_enemies[short].append(eid)

print(f"Regions with enemies: {sorted(region_enemies.keys())}")
for r in sorted(region_enemies.keys()):
    print(f"  {r}: {len(region_enemies[r])} enemies")

# Generate 3 spawn tables per region
os.makedirs(f"{CONTENT}/spawn_tables", exist_ok=True)

# Keep existing R01-s1.ron and R03-s1.ron - just add the rest
# Remove all my generated p* files
for f in glob.glob(f"{CONTENT}/spawn_tables/R*-s1-p*.ron"):
    os.remove(f)

total_encounters = 0
spawn_tables_created = 0

for short_rid in sorted(region_enemies.keys()):
    eids = region_enemies[short_rid]
    if not eids:
        continue
    
    # Check if file already exists (skip existing)
    existing_path = f"{CONTENT}/spawn_tables/{short_rid}-s1.ron"
    if os.path.exists(existing_path):
        # Count its entries
        with open(existing_path) as f:
            content = f.read()
            existing_entries = content.count('(enemy:')
        total_encounters += existing_entries
        spawn_tables_created += 1
        print(f"  Keeping existing: {existing_path} ({existing_entries} entries)")
        continue
    
    # Create 3 pool spawn tables for this region
    for pool in [1, 2, 3]:
        # Distribute enemies across pools
        pool_eids = [e for i, e in enumerate(eids) if (i % 3) == (pool - 1)]
        if not pool_eids:
            pool_eids = eids[:5]  # fallback
        
        fname = f"{CONTENT}/spawn_tables/{short_rid}-s1-p{pool}.ron"
        with open(fname, 'w') as f:
            f.write(f'SpawnTable(\n')
            f.write(f'    region: "{short_rid}",\n')
            f.write(f'    chapter_stage: 1,\n')
            f.write(f'    pool: {pool},\n')
            f.write(f'    entries: [\n')
            for eid in pool_eids:
                weight = max(6, min(30, (abs(hash(eid)) % 25) + 6))
                f.write(f'        (enemy: "{eid}", weight: {weight}, gate: Always),\n')
            f.write(f'    ],\n')
            f.write(f')\n')
        
        total_encounters += len(pool_eids)
        spawn_tables_created += 1

print(f"\nSpawn tables created/kept: {spawn_tables_created}")
print(f"Total encounters (spawn entries): {total_encounters}")

# Show what we have
for f in sorted(glob.glob(f"{CONTENT}/spawn_tables/R*.ron")):
    with open(f) as fh:
        content = fh.read()
        entries = content.count('(enemy:')
        region = "unknown"
        for line in content.split('\n'):
            if 'region:' in line:
                region = line.split('"')[1]
        print(f"  {os.path.basename(f)}: region={region}, {entries} entries")
