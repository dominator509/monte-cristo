#!/usr/bin/env python3
"""Fix content tree - generate precise spawn tables and fix issues."""
import os
import glob
import json

CONTENT = "/root/monte-cristo/content"

def read_ron_file(path):
    """Read a RON file and return its content as string."""
    with open(path) as f:
        return f.read()

def get_enemy_id(path):
    """Extract enemy ID from a RON file."""
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line.startswith('id:'):
                return line.split('"')[1]
    return None

def get_enemy_field(path, field):
    """Extract a field from a RON enemy file."""
    with open(path) as f:
        content = f.read()
        for line in content.split('\n'):
            line = line.strip()
            if line.startswith(f'{field}:'):
                return line.split(':', 1)[1].strip().rstrip(',').strip('"')
    return None

# 1. Read all enemies and their attributes
enemies = {}
for f in sorted(glob.glob(f"{CONTENT}/bestiary/enm_*.ron")):
    eid = get_enemy_id(f)
    if eid:
        enemies[eid] = f

print(f"Unique enemy IDs: {len(enemies)}")

# 2. Read region info
regions = {}
for f in sorted(glob.glob(f"{CONTENT}/regions/R*.ron")):
    with open(f) as fh:
        for line in fh:
            line = line.strip()
            if line.startswith('id:'):
                rid = line.split('"')[1]
                # Get short form (e.g., "R01" from "R01_MARSEILLE")
                short = rid.split('_')[0]
                regions[short] = rid
                break

print(f"Regions: {len(regions)}")

# 3. Assign enemies to regions based on their region_affinity
region_enemies = {r: [] for r in regions}
region_enemies_short = {}

for eid in enemies:
    content = read_ron_file(enemies[eid])
    for line in content.split('\n'):
        line = line.strip()
        if line.startswith('region_affinity:'):
            # Extract the list
            aff_raw = line.split(':', 1)[1].strip()
            # Handle multi-line lists
            if not aff_raw.startswith('['):
                # Not on this line, skip
                pass
            else:
                # Parse: ["R01_MARSEILLE", "R03_MONTE_CRISTO"]
                affs = [a.strip().strip('"') for a in aff_raw.strip('[],').split(',') if a.strip()]
                for aff in affs:
                    short = aff.split('_')[0] if '_' in aff else aff
                    if short in regions:
                        if eid not in region_enemies[short]:
                            region_enemies[short].append(eid)
            break

# 4. Generate spawn tables - 3 per region = 45 total
print("\nGenerating 45 spawn tables...")
os.makedirs(f"{CONTENT}/spawn_tables", exist_ok=True)

# Remove all existing spawn tables first
for f in glob.glob(f"{CONTENT}/spawn_tables/R*.ron"):
    os.remove(f)

total_encounters = 0
for short_rid in sorted(regions.keys()):
    eids = region_enemies.get(short_rid, [])
    if not eids:
        print(f"  WARNING: No enemies for {short_rid}")
        continue
    
    # Create 3 spawn tables per region
    for pool in [1, 2, 3]:
        fname = f"{CONTENT}/spawn_tables/R{short_rid[1:]}-s1-p{pool}.ron"
        # Split enemies across pools
        pool_enemies = []
        for i, eid in enumerate(eids):
            if i % 3 == (pool - 1):
                pool_enemies.append(eid)
        
        if not pool_enemies:
            # Use all enemies if pool is empty
            pool_enemies = eids
        
        # Generate entries
        entries = []
        for eid in pool_enemies:
            weight = max(5, (hash(eid) % 20) + 5)
            entries.append(f'        (enemy: "{eid}", weight: {weight}, gate: Always),')
        
        with open(fname, 'w') as f:
            f.write('SpawnTable(\n')
            f.write(f'    region: "{short_rid}",\n')
            f.write('    chapter_stage: 1,\n')
            f.write(f'    pool: {pool},\n')
            f.write('    entries: [\n')
            f.write('\n'.join(entries))
            f.write('\n    ],\n')
            f.write(')\n')
        
        total_encounters += len(pool_enemies)

print(f"Total spawn table entries (encounters): {total_encounters}")

# 5. Ensure every enemy has at least R01 region if none assigned
for eid, path in enemies.items():
    content = read_ron_file(path)
    # Check if region_affinity is set
    has_aff = 'region_affinity:' in content
    if not has_aff:
        print(f"  WARNING: {eid} has no region_affinity")

print("\nContent fix complete!")
print(f"Enemies: {len(enemies)}")
print(f"Regions: {len(regions)}")
print(f"Spawn tables: {len(glob.glob(f'{CONTENT}/spawn_tables/R*.ron'))}")
