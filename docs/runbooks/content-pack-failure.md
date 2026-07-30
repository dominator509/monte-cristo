# Runbook: content pack failure

**Applies to:** published artifact | player machine
**Severity:** blocking

## Symptom
The game exits before the title screen and reports that `content.pack` is missing or invalid.

## Diagnosis
| Step | Command | Interpretation |
|---|---|---|
| 1 | `./monte-cristo --verify-content` | `content: ok` proves the installed pack is intact; any other result identifies a missing or corrupt pack. |
| 2 | `sha256sum -c SHA256SUMS` | All entries marked `OK` prove the downloaded release artifacts match the manifest. |

## Action
MANUAL: reinstall the complete release archive from the trusted distribution location. Do
not hand-edit `content.pack` or its digest.

## Verification
Run `./monte-cristo --verify-content`; it must print `content: ok`.

## If this does not work
Retain the failing archive and command output, then use the escalation path in
`OPERATIONS.md` section 10.

## Prevention
`scripts/smoke-test.sh` verifies the baked pack through the release shell, and
`scripts/reality-gate.sh` rejects simulated content loading.
