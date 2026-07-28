# ENVIRONMENT -- MONTE CRISTO

## 1. Required tools and exact versions

Mirrors the checks in scripts/preflight.sh. Versions are exact, not minimums, except where
noted.

| Tool | Version | Why | Checked by |
|---|---|---|---|
| rustc / cargo | 1.83.0 exactly | pinned by rust-toolchain.toml; determinism requires a fixed compiler | scripts/probes/rust_toolchain.sh |
| rustfmt | shipped with 1.83.0 | format gate | scripts/format-check.sh |
| clippy | shipped with 1.83.0 | lint gate at deny level | scripts/lint.sh |
| cargo-deny | 0.17.0 | advisory, licence, ban, source gates | scripts/probes/cargo_tools.sh |
| cargo-fuzz | 0.12.0 | fuzzing the two untrusted parsers | scripts/probes/cargo_tools.sh |
| cargo-llvm-cov | 0.6.15 | coverage floors | scripts/probes/cargo_tools.sh |
| nightly toolchain | nightly-2025-01-15 | required by cargo-fuzz only; never used for production builds | EP-006 M1 |
| git | 2.34 or newer | the coordination bus | scripts/preflight.sh tool loop |
| awk, grep, sed, tar, mktemp | POSIX | the pack's own scripts | scripts/preflight.sh tool loop |
| sha256sum or shasum | any | release manifests | scripts/build.sh |
| mingw-w64 | 12 or newer | Windows cross-build target only | scripts/probes/graphics_stack.sh |

## 2. Pinned crate dependencies

Exact versions, committed in Cargo.lock, never resolved as `latest`.

| Crate | Version | Used by | Purpose |
|---|---|---|---|
| macroquad | 0.4.13 | mc_shell | window, GPU 2D, input, audio |
| postcard | 1.0.10 | mc_core, mc_data | save serialization |
| serde | 1.0.215 | mc_core, mc_data, mc_tape | derive only |
| ron | 0.8.1 | mc_data, mc_tools | content authoring format |
| blake3 | 1.5.5 | mc_data, mc_tape | content and save digests, state hashing |
| indexmap | 2.7.0 | mc_core | ordered maps (INV-03) |
| thiserror | 2.0.6 | all | typed errors |
| tracing | 0.1.41 | mc_shell, mc_tools | structured local logging |
| tracing-subscriber | 0.3.19 | mc_shell, mc_tools | JSON file appender |
| clap | 4.5.23 | mc_tools | CLI |
| proptest | 1.5.0 | dev only | property tests |
| insta | 1.41.1 | dev only | snapshot tests |
| criterion | 0.5.1 | dev only | benches |
| arbitrary | 1.4.1 | dev only | fuzz input shaping |

mc_core's dependency set is deliberately tiny and contains nothing capable of I/O,
threading, clock access, or ambient randomness (INV-01). That is enforced by
scripts/security-check.sh, not by convention.

## 3. Environment variable reference

| Name | Required | Environments | Example | Secret | Description | Validation |
|---|---|---|---|---|---|---|
| MC_CONTENT_DIR | yes | all | ./content | no | RON content source root; a confinement root | must exist, be a directory, be writable |
| MC_ARTIFACT_DIR | yes | build, release | ./target/artifacts | no | release staging; a confinement root | must exist, be a directory, be writable |
| MC_DATA_DIR | yes | all | ./.local-data | no | saves, settings, logs, crash reports; a confinement root | must exist, be a directory, be writable |
| MC_RUST_VERSION | yes | all | 1.83.0 | no | asserted against rustc | must equal rust-toolchain.toml |
| MC_CARGO_TOOLS | yes | all | 0.16.2,0.12.0,0.6.15 | no | pinned cargo subcommand versions | three comma-separated semver values |
| MC_GRAPHICS_STACK | yes | build | x11 | no | platform windowing stack for mc_shell | one of x11, wayland, macos, mingw |
| CARGO_REGISTRY_MODE | yes | all | online | no | registry or vendored mirror | one of online, vendored |
| MC_REFERENCE_MACHINE | yes | bench | ryzen7-5800u-integrated | no | attribution for performance budgets | non-empty |
| MC_HEADLESS | yes | all automation | 1 | no | suppresses window creation in mc_shell | must be exactly 1 in scripted gates |
| CI | set by scripts | automation | true | no | non-interactive mode | -- |
| RUST_BACKTRACE | set by scripts | automation | 1 | no | diagnostics | -- |

Consistent with PREFLIGHT.md section 6. If you add a variable, add it to both files and to
.env.example in the same commit.

## 4. Local setup

    cp .env.example .env
    # edit .env: set the three directories and MC_REFERENCE_MACHINE
    mkdir -p ./content ./target/artifacts ./.local-data
    rustup toolchain install 1.83.0
    rustup component add rustfmt clippy --toolchain 1.83.0
    cargo install --locked cargo-deny@0.16.2 cargo-fuzz@0.12.0 cargo-llvm-cov@0.6.15
    sh scripts/preflight.sh    # must print: preflight: ok

## 5. Test environment

Identical to local, plus `MC_HEADLESS=1` (exported by every script) and a writable system
temporary directory. Tests never touch MC_DATA_DIR; each creates and removes its own
temporary directory.

## 6. Staging environment

"Staging" for a desktop game is a clean machine with only the release artifact installed.
EP-009 performs this as a real drill: extract the tarball into an empty directory, run the
golden tape replay against the shipped binary, and compare the hash.

## 7. Production environment

The player's machine. The only requirements are the platform minimums in PROJECT_BRIEF and a
writable data directory. No runtime dependency is installed by this software.

## 8. Configuration validation

`mc_shell` and `mc_tools` validate the environment once at startup: the three roots are
canonicalised and asserted to exist and be writable; `CARGO_REGISTRY_MODE` is checked only
by build scripts, never at runtime; an invalid value fails immediately with a message naming
the variable and the expected shape. Configuration is never partially applied.

## 9. Parity rules

Local, CI, and release builds use the same pinned toolchain, the same lockfile, and the same
scripts. CI adds nothing (ADR-013). The only permitted difference between environments is
the value of the three directory variables. If a gate passes locally and fails in CI, the
cause is an environment variable or a missing tool, and preflight will name it.

## 10. Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `preflight: FAIL - missing required tool: cargo-deny` | subcommand not installed | run the cargo install line in section 4 |
| `preflight: FAIL - env var not set: MC_DATA_DIR` | .env incomplete | fill it from .env.example |
| `preflight: FAIL - credential probe failed: MC_GRAPHICS_STACK` | no windowing dev packages | install the platform dev packages named in PREFLIGHT 1.4 |
| `DETERMINISM_HASH_MISMATCH` | INV-01 violation | find the float, the hash-order iteration, the clock, or the thread. Never patch the test |
| `LAYER_VIOLATION` | a crate imported upward | fix the import, never the check |
| build fails only offline | CARGO_REGISTRY_MODE=vendored but vendor/ empty | run `cargo vendor` on a networked machine, commit the config |
| shell fails to start in CI | MC_HEADLESS not set | it is exported by every script; you invoked cargo directly |
