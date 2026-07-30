# Runbook: save failure

**Applies to:** player machine
**Severity:** blocking

## Symptom
Saving fails, or a save reports an unsupported version or invalid digest.

## Diagnosis
| Step | Command | Interpretation |
|---|---|---|
| 1 | `./monte-cristo --check-paths` | `paths: ok` proves all configured roots exist and are writable. |
| 2 | `./monte-cristo --save-info <file>` | Reports the save schema version and digest without loading the save. |
| 3 | `df -h` | A full data volume explains a write failure even when permissions are correct. |

## Action
MANUAL: preserve the existing save, free space or restore write permission on
`MC_DATA_DIR`, then retry. Install the newer game version if `--save-info` reports that the
save schema is newer. Never downgrade or hand-edit a save.

## Verification
Run `./monte-cristo --check-paths`; it must print `paths: ok`. Create a new save, then run
`./monte-cristo --save-info <file>` and confirm its digest is valid.

## If this does not work
Copy the save and the latest local log before using the escalation path in
`OPERATIONS.md` section 10.

## Prevention
`scripts/smoke-test.sh` verifies root writability. Save parsing and corruption behavior are
covered by the integration and fuzz gates.
