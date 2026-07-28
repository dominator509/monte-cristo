#!/usr/bin/env python3
"""Generate complete content tree for Monte Cristo."""
import os
import glob

CONTENT = "/root/monte-cristo/content"

# Read existing enemy IDs
existing_ids = set()
for f in glob.glob(f"{CONTENT}/bestiary/enm_*.ron"):
    with open(f) as fh:
        for line in fh:
            line = line.strip()
            if line.startswith('id:'):
                eid = line.split('"')[1]
                existing_ids.add(eid)
                break

print(f"Existing enemies: {len(existing_ids)}")
for eid in sorted(existing_ids):
    print(f"  {eid}")

# Read existing region IDs
existing_regions = set()
for f in glob.glob(f"{CONTENT}/regions/R*.ron"):
    with open(f) as fh:
        for line in fh:
            line = line.strip()
            if line.startswith('id:'):
                rid = line.split('"')[1]
                existing_regions.add(rid)
                break
print(f"Existing regions: {len(existing_regions)}")
for r in sorted(existing_regions):
    print(f"  {r}")
