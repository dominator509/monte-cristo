# SPEC-006 -- Error handling

## 1. Principles

1. Every fallible public function returns `Result<T, E>` with a `thiserror` enum. No public
   function panics.
2. Errors are typed and matchable. A caller never parses a message string to decide what
   happened.
3. `unwrap` and `expect` appear only in tests, benches, and `main` startup where failure is
   genuinely fatal and the message names the cause.
4. mc_core has no error path that depends on the environment, because it has no environment.
   Its only errors are domain errors.
5. A rejected player command is not an error. It is a normal event.

## 2. Error taxonomy

| Enum | Crate | Variants (abridged) |
|---|---|---|
| `CoreError` | mc_core | `InvalidCommand`, `NoActiveBattle`, `NoActiveScene`, `SaveDecode` |
| `ContentError` | mc_data | `Parse{file,line}`, `DanglingRef{id}`, `Orphan{id}`, `UnknownFamily{got}`, `AffinityContradiction`, `MissingString{key}`, `DigestMismatch` |
| `SaveError` | mc_data | `Truncated`, `DigestMismatch`, `UnsupportedVersion{got,max}`, `ContentMismatch`, `FieldTooLarge{field,got,max}`, `Io` |
| `TapeError` | mc_tape | `BadMagic`, `NonMonotonicTick`, `ContentMismatch`, `Truncated`, `UnknownCommand{discriminant}` |
| `FsError` | mc_shell | `OutsideRoot{root}`, `NotFound`, `NotWritable`, `Io` |
| `ShellError` | mc_shell | `Config{var}`, `Content(ContentError)`, `Save(SaveError)`, `Fs(FsError)` |

## 3. Rejections versus errors

`apply_commands` returns `Vec<CoreEvent>`; an invalid command produces
`CoreEvent::Rejected { command, reason }`. Pressing confirm on an empty menu is not a fault
and must not be logged as one. This distinction is what lets a replay tolerate a tape
recorded against slightly different UI state without becoming unsafe.

## 4. User-facing error presentation

Every error the player can see states: what happened, which file or setting is involved
(relative to a declared root, never an absolute path), what the software did about it, and
what the player can do. Example:

    Save failed: the data directory is not writable.
      directory: saves/
      Your previous save has not been modified.
      Check permissions on the game data folder, then try again.

Never show a raw backtrace, a Rust type name, or an absolute path.

## 5. Failure modes and required handling

| Failure | Handling | Test |
|---|---|---|
| content.pack missing | clear startup message with the expected path; exit nonzero | `forced_failures.rs` |
| content digest mismatch | refuse to load; print expected and actual; exit nonzero | `forced_failures.rs` |
| save truncated | `SaveError::Truncated`; slot shown as damaged; other slots unaffected | `forced_failures.rs` |
| save digest mismatch | `SaveError::DigestMismatch`; same treatment | `forced_failures.rs` |
| save schema newer | `SaveError::UnsupportedVersion`; tell the player a newer build wrote it | `forced_failures.rs` |
| data dir read-only | typed error at startup; game runs but cannot save, and says so once | `forced_failures.rs` |
| disk full during save | typed error; previous save intact; partial file removed | `forced_failures.rs` |
| path traversal attempt | `FsError::OutsideRoot`; logged at warn | `fsroot_confine.rs` |
| malformed tape | `TapeError`; replay exits nonzero naming the first bad entry | `forced_failures.rs` |
| arithmetic saturation in core | saturate, emit `CoreEvent::ArithmeticSaturated`, continue | `prop_fixed_point.rs` |
| panic anywhere in the shell | crash report written locally with tick and state hash; nothing transmitted | `crash_report.rs` |

## 6. Recovery

The software recovers where recovery is honest and refuses where it is not. It restores
default settings if the settings file is unreadable, because defaults are correct. It refuses
to load a save whose digest fails, because guessing at corrupted state is worse than
refusing. That asymmetry is the whole policy.

## 7. Logging of errors

`error` for anything the player notices; `warn` for a recovered fault or a security-relevant
rejection; never `info` for a failure. Every logged error includes its typed variant name so
that a log line maps back to a match arm.

## 8. Validation

`crates/*/tests/forced_failures.rs` implements the whole of section 5 by really forcing each
condition -- truncating real files, flipping real bytes, filling a real small filesystem,
chmodding a real directory. None of it is simulated (reality law, section 6.3).
