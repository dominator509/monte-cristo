# PREFLIGHT -- MONTE CRISTO

This file is L2 SPECIFICATION. It is the complete, exhaustive enumeration of every external
need this project will ever have. There is exactly ONE interactive moment in the life of a
run: you read this file, satisfy every REQUIRED entry, copy .env.example to .env, fill it,
and run `sh scripts/preflight.sh` until it prints `preflight: ok`.

After that line appears the run never stops for a credential, an account, a payment, a
permission, or a question.

## 0. The good news

MONTE CRISTO has no credentials. None. There is no API key, no account, no token, no
service, no database server, and no network dependency at runtime. This is a design
constraint of the project (see PROJECT_BRIEF.md, Non-Goals) and not an oversight.

What preflight verifies instead is: the pinned toolchain, the required command line tools,
three writable directories, and reachability of the crate registry (or a vendored offline
mirror). Every one of these has a read-only probe.

The absence of secrets is not assumed. `scripts/security-check.sh` scans the whole tree on
every verify run and fails on any committed-secret hit, so the property stays true.

## 1. Toolchain

### 1.1 Rust toolchain -- REQUIRED
- Purpose: everything. All five crates. Consumed by every ExecPlan EP-000 through EP-010.
- Version: exactly 1.83.0, pinned by `rust-toolchain.toml` created in EP-001 M1.
- Obtain: install rustup from the official distribution channel for your platform, then
  `rustup toolchain install 1.83.0` and `rustup component add rustfmt clippy`.
- Probe: scripts/probes/rust_toolchain.sh
- Cost: free.

### 1.2 Cargo auxiliary tooling -- REQUIRED
- Purpose: dependency audit and licence policy (cargo-deny), fuzzing of the save and content
  parsers (cargo-fuzz, EP-006), coverage measurement (cargo-llvm-cov, EP-007).
- Versions: cargo-deny 0.16.2, cargo-fuzz 0.12.0, cargo-llvm-cov 0.6.15.
- Obtain: `cargo install --locked cargo-deny@0.16.2 cargo-fuzz@0.12.0 cargo-llvm-cov@0.6.15`
- Probe: scripts/probes/cargo_tools.sh
- Cost: free.
- Note: cargo-fuzz requires a nightly toolchain for the fuzz targets only. EP-006 installs
  `nightly-2025-01-15` for that single purpose. Production builds never use nightly.

### 1.3 Base POSIX tooling -- REQUIRED
- Purpose: the pack's own scripts. git, awk, grep, sed, tar, sha256sum (or shasum), mktemp.
- Obtain: present on any standard Linux or macOS install; on Windows use the Git for Windows
  bundled sh, or WSL2. Windows target builds are produced by cross-compilation from Linux in
  EP-009 and do not require a Windows host.
- Probe: covered by the tool loop inside scripts/preflight.sh itself.
- Cost: free.

### 1.4 Graphics runtime for the shell -- REQUIRED on the build host
- Purpose: mc_shell links against the platform windowing and GPU stack (macroquad 0.4.13).
  Needed to build, not to run the headless test suite.
- Linux: a working X11 or Wayland development environment, plus `libasound2-dev` for audio.
- macOS: the Xcode command line tools.
- Windows cross-build: the `x86_64-pc-windows-gnu` target and `mingw-w64` linker.
- Probe: scripts/probes/graphics_stack.sh
- Cost: free.
- Fallback: none needed. If this probe fails you can still run every headless gate, but
  EP-005 and EP-009 cannot complete, so preflight treats it as REQUIRED rather than letting
  the run discover the gap at EP-005.

## 2. Directories

All three are operator-declared, are the ONLY paths this software may ever touch, and are
enforced by path confinement in EP-006. They must exist and be writable before the run.

### 2.1 MC_CONTENT_DIR -- REQUIRED
- Purpose: the RON content source tree that the bake step reads. Created and populated by
  EP-002 and EP-003; must exist and be writable at preflight time.
- Suggested value: `./content`
- Probe: scripts/probes/content_dir.sh

### 2.2 MC_ARTIFACT_DIR -- REQUIRED
- Purpose: where release tarballs and SHA256SUMS are staged by EP-009 and EP-010.
- Suggested value: `./target/artifacts`
- Probe: scripts/probes/artifact_dir.sh

### 2.3 MC_DATA_DIR -- REQUIRED
- Purpose: saves, settings, logs, and local crash reports at runtime and during test.
- Suggested value: `./.local-data`
- Probe: scripts/probes/data_dir.sh

## 3. Registry reachability

### 3.1 CARGO_REGISTRY_MODE -- REQUIRED
- Purpose: decides whether the build fetches crates from the network or uses a vendored
  offline mirror. Exactly one of `online` or `vendored`.
- `online`: the build host can reach the crate registry. This is the default.
- `vendored`: EP-000 will run `cargo vendor` once into `vendor/` and configure
  `.cargo/config.toml` for offline builds. Choose this for an air-gapped workstation. If you
  choose `vendored` you must have populated `vendor/` from a networked machine first, or
  EP-000 M2 will do it while the network is still available and then assert offline builds
  work.
- Probe: scripts/probes/registry_mode.sh
- Cost: free.

## 4. Non-secrets used as configuration

### 4.1 MC_REFERENCE_MACHINE -- REQUIRED
- Purpose: a short free-text identifier of the machine the performance budgets in
  PERFORMANCE sections are measured against. Recorded into the bench report so that a budget
  failure can be attributed. Example: `ryzen7-5800u-integrated`.
- Probe: `-` (presence only).

### 4.2 MC_HEADLESS -- REQUIRED
- Purpose: set to `1` in all scripted gates so mc_shell never attempts to open a window
  during automated runs. The tape harness and every live-fire proof rely on this.
- Value: `1`
- Probe: `-` (presence only; scripts/preflight.sh asserts it is exactly `1`).

## 5. Explicitly NOT required

Recorded here so that a future reader does not go looking: no database server, no message
broker, no object store, no SMTP, no payment processor, no OAuth provider, no analytics
sink, no error-tracking service, no CDN, no container runtime, no hosted CI account, no code
signing certificate for version 1.0, and no app store developer account. If any node of the
graph ever appears to need one of these, that is a specification defect: record it, take the
node's FALLBACK, or block with a report naming this section.

## 6. Machine table

PREFLIGHT-TABLE-BEGIN
MC_CONTENT_DIR|REQUIRED|scripts/probes/content_dir.sh
MC_ARTIFACT_DIR|REQUIRED|scripts/probes/artifact_dir.sh
MC_DATA_DIR|REQUIRED|scripts/probes/data_dir.sh
MC_RUST_VERSION|REQUIRED|scripts/probes/rust_toolchain.sh
MC_CARGO_TOOLS|REQUIRED|scripts/probes/cargo_tools.sh
MC_GRAPHICS_STACK|REQUIRED|scripts/probes/graphics_stack.sh
CARGO_REGISTRY_MODE|REQUIRED|scripts/probes/registry_mode.sh
MC_REFERENCE_MACHINE|REQUIRED|-
MC_HEADLESS|REQUIRED|-
PREFLIGHT-TABLE-END

## 7. If preflight fails

The failure line names the exact missing item. Fix that one item and rerun. Do not proceed
past a failing preflight; every node of the graph assumes these facts hold. A preflight
failure is the only legitimate stop before the run begins.
