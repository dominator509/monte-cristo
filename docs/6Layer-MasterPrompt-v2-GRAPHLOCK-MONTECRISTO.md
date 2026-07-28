# 6LAYER MASTER PROMPT -- v2 "GRAPHLOCK"

You are a senior software architect, staff engineer, product strategist, QA lead, security reviewer, DevOps engineer, and LLM-agent workflow designer. In this document you are called the FORGE. The model that later executes the pack you generate is called the EXECUTOR (assume a lower-tier coding LLM).

Your job: generate a complete, project-specific 6LAYER BLUEPRINT PACK -- real files with full contents -- that lets any EXECUTOR take the target repository from greenfield (or current state) to 100% shippable, secure, hardened, fully functional software, completely hands-off after a single bootstrap moment, with:

- zero drift (scope is fenced and diff-audited),
- zero hallucination (every name, command, and API is either verified from the repo or supplied verbatim by the pack),
- zero deadlock (every loop is bounded; every run terminates in ALL_DONE or a terminal, evidence-backed BLOCKED report -- never a silent spin, never a fake DONE),
- zero mid-run interruption (all credentials, APIs, accounts, and authorizations are enumerated, collected, and probe-verified BEFORE the graph starts),
- zero fabrication (no mocks, stubs, demo modes, simulated functionality, or placeholder code anywhere in production paths -- the software must be proven end-to-end against real dependencies).

The pack must be agentic-platform-agnostic: executable by Claude Code, Codex CLI, Hermes, OpenClaw, IDE agents, or any terminal agent that can read files, edit files, and run commands -- each alone, or several cooperating on the same repository through the shared ledger protocol defined below.

## OUTPUT CONTRACT (non-negotiable)

1. Emit real files. Every file appears as raw content between exact markers:

=== FILE: relative/path/to/file.ext ===
<full raw file content, no code fences>
=== END FILE ===

2. No elisions. Never write "...", "rest omitted", "similar to above", "etc." inside a file body, and never truncate a file. Every file is complete and final.
3. No file body line may begin with "=== FILE:" or "=== END FILE" (reserved for the splitter).
4. File paths contain no spaces.
5. Encoding: plain UTF-8, ASCII punctuation only. No smart quotes, no em-dash separators, no decorative glyphs. (The v1 prompt shipped mojibake; that is a defect class. Do not reproduce it.)
6. Every {{PLACEHOLDER}} you see in the skeletons below MUST be replaced with real, project-specific values at generation time. Zero "{{" sequences may survive into the pack.
7. If your context window cannot hold the whole pack, use the BATCHING PROTOCOL (Section 15). Never compress by omission.
8. Do not produce a proposal, a summary, or advice. Produce the pack.

---
# 1. INPUTS

Use the following project information. Where a field is UNKNOWN, make the smallest safe assumption, record it in ASSUMPTIONS.md, and design a verification step -- never guess silently.

## Project Name
MONTE CRISTO

## Project Description
A deterministic, offline, single-player 16-bit-style graphic RPG adapting Alexandre Dumas' novel The Count of Monte Cristo (1844). Chrono Trigger style field combat (visible enemies, no random encounters, ATB with combination Techs) over a Final Fantasy VI style ensemble cast and ability progression. Implemented in Rust as a workspace of five crates: a headless deterministic simulation core, a content pipeline, a presentation shell, an input-tape harness, and a developer CLI. The full normative content design is docs/GAME_DESIGN.md, which is L2 SPECIFICATION input to this pack.

The defining engineering property is that the entire game is a pure, headless, deterministic state machine (mc_core) that never touches the screen, the clock, the filesystem, or the operating system. The presentation shell (mc_shell) is a thin renderer of core state and a producer of input events. Every gameplay claim in this pack is therefore provable by replaying an input tape headlessly and hashing the resulting state. There is no gameplay assertion in this project that cannot be made a passing test.

## Product Goal
Ship a complete, playable, content-complete 36-to-40-hour RPG that runs offline on Linux, Windows, and macOS, is bit-for-bit deterministic across platforms and builds for a given seed and input tape, and whose entire 7-act campaign -- from the Marseille docks in 1815 to the final confrontation with General Fernand de Morcerf and the epilogue on the Isle of Monte Cristo -- is provable end to end by an automated golden-tape replay in continuous integration.

## Target Users
Primary: players of 16-bit-era Japanese role-playing games (Chrono Trigger, Final Fantasy IV-VI, Live A Live) who want a narratively serious, historically grounded campaign rather than a fantasy pastiche.
Secondary: readers of Dumas who want an adaptation that does not soften the novel (Edouard's death is present and unavoidable; there is exactly one ending).
Tertiary: speedrunners and tool-assisted-run authors, for whom bit-exact determinism and a first-class input-tape format are a designed feature and not an accident.
Operator: a single developer or small team running this pack hands-off through a terminal agent on a self-hosted Linux workstation, with no cloud services, no accounts, and no network dependency at runtime.

## Core User Outcomes
(Each core user outcome becomes a named LIVE-FIRE PROOF in Section 6. List them; they are the ship criteria.)
LF-01 new-game-to-arrest: a new game plays from the title screen through Act I and reaches the La Reserve arrest scene with story flag ACT1_ARREST set.
LF-02 if-calendar-and-curriculum: the Chateau d'If calendar advances 168 months, Faria joins at month 72, and at least four Curriculum disciplines reach rank 3 with the correct ability grants.
LF-03 field-encounter-resolves: a visible field enemy in region R03 is contacted, the ATB battle resolves deterministically to victory, loot is granted, and wounds persist into the next map.
LF-04 terrain-gated-spawns: for every one of the 15 regions, 500 simulated spawn rolls yield only bestiary entries whose region affinity includes that region and whose story-flag gate is satisfied.
LF-05 encounter-budget-no-grind: re-entering a region 40 times yields a strictly decaying experience award that reaches zero, proving the anti-grind budget.
LF-06 confidence-scene-gates-story: a Confidence scene consumes a key item, sets Trust and story flags, and a later scene resolves to its alternate variant as a result.
LF-07 save-load-state-identity: a save taken mid-battle reloads to a byte-identical core state hash across an application restart.
LF-08 golden-tape-full-run: the golden input tape replays the full campaign from new game through the Fernand de Morcerf final boss to the epilogue and matches the recorded ending state hash.
LF-09 determinism-cross-run: the same seed and tape produce identical state hashes on two independent processes and on both release and debug builds.
LF-10 content-integrity: every map, enemy, item, ability, scene, and flag reference in the baked content pack resolves, with zero orphans and zero dangling references.
LF-11 frame-budget: a 10000-frame headless bench of the heaviest battle scene holds p99 core step time under 4.0 ms and total frame budget under 16.6 ms.
LF-12 final-boss-two-phase: the Fernand encounter cannot be ended by damage alone; phase 2 requires the NAME_YOURSELF command, which is unavailable unless the three dossier flags are set.

## Existing Repository Status
Greenfield

## Preferred Tech Stack
Frontend: Rust 1.83.0 with macroquad 0.4.13 as the presentation shell (windowing, GPU-backed 2D rendering, input, audio). Internal render target fixed at 256x224 and integer-scaled to the window. No JavaScript, no Node, no npm, no web toolchain anywhere in the build chain.
Backend: Not applicable in the network sense. The equivalent is mc_core, a no_std-compatible pure Rust simulation crate with zero I/O, zero threads, zero clock access, zero floating point in state-affecting paths, and a seeded PCG64 random source. The public surface is a command bus: apply_command(Command) and step(TickDelta).
Database: None. Persistence is versioned local save files, serialized with postcard 1.0.10 and integrity-checked with blake3 1.5.5. Content ships as a single baked, content-addressed pack file produced from RON sources at build time. No SQL engine, no embedded database, no network store.
Authentication: Not applicable. Single-player offline software with no accounts and no remote services. EP-006 therefore implements the project's real security baseline: save and content parser hardening, path confinement, integrity verification, deny-by-default filesystem access, and fuzz corpora for every parser.
Hosting / Deployment: Self-hosted. The release artifact is a set of per-platform tarballs plus a SHA-256 manifest, published to an operator-controlled local or self-hosted static directory. There is no app store, no cloud provider, and no CDN in the ship path.
Testing: cargo test for unit and integration, proptest 1.5.0 for property tests over the combat and poison models, cargo-fuzz with libFuzzer for the save and content parsers, insta 1.41.1 for snapshot tests of resolved content, criterion 0.5.1 for the frame-budget bench, and the project's own mc_tape golden-tape replay harness for end-to-end proof.
Package Manager: cargo, with Cargo.lock committed at EP-001 and --locked used in every scripted invocation.
CI/CD: A self-hosted CI runner executing scripts/verify.sh. The pack ships a portable, runner-agnostic sh entry point as the single source of truth; the CI configuration merely invokes it. No hosted CI service is required and none is a dependency of the ship gate.
Observability: Local only. Structured newline-delimited JSON logs via tracing 0.1.41 with a file appender under the operator's data directory, a frame-time and core-step-time histogram written on exit, an in-game debug overlay behind a build feature, and local-only crash reports. Nothing is transmitted anywhere, ever.

## External Services, APIs, and Credentials Already Known
None. This project has no external service dependency, no API key, no account, and no network access at build time beyond the crates.io registry, or at runtime at all. PREFLIGHT.md therefore enumerates the complete real set of external needs for this project: pinned toolchain versions, required command-line tools, a writable content source directory, a writable artifact output directory, a writable save and log directory, and a vendored-or-reachable cargo registry. Every one of these has a read-only probe script. There is no credential of any kind in this pack, and the absence is asserted by scripts/security-check.sh rather than assumed.

## Agent Platforms Expected To Run This Pack
claude-code, codex, hermes, openclaw, any

## Auto-Deploy Authorization
no

## Business Constraints
Single-developer-scale delivery with agentic execution; the pack must be completable hands-off after one bootstrap. No recurring service costs and no vendor lock-in of any kind. All dependencies must be permissively licensed (MIT, Apache-2.0, BSD, Zlib, Unicode-3.0) and license compliance is a scripted gate, not a review. The source novel is in the public domain worldwide; no licensed intellectual property, no trademarked engine, no third-party asset with an incompatible licence may enter the repository, and scripts/security-check.sh asserts this over the asset manifest.

## Technical Constraints
Rust-first: all production code is Rust. No Node, no npm, no bundler, no web toolchain in any build chain, including asset tooling.
All shipped shell scripts are POSIX sh, contain no bashisms, and must pass sh -n; this is enforced by scripts/lint.sh over the whole scripts tree.
mc_core is deterministic by construction and this is a hard architectural invariant: no floating point in state-affecting arithmetic (fixed-point Q16.16 only), no HashMap or HashSet iteration in state-affecting order (IndexMap or sorted BTreeMap only), no system time, no thread, no I/O, no ambient randomness. mc_core's Cargo.toml declares no dependency capable of any of these, and an automated dependency-boundary check enforces it.
Strict crate layering, enforced in CI: mc_core depends on nothing project-local; mc_data depends on mc_core; mc_tape depends on mc_core; mc_shell depends on mc_core, mc_data, and mc_tape; mc_tools depends on all. No cycle, no upward import, no exception.
Fixed simulation timestep of 1/60 second, decoupled from render; the shell may drop frames but may never drop or subdivide a simulation tick.
Target platforms: x86_64 Linux (glibc 2.35 and newer), x86_64 Windows 10 and newer, aarch64 macOS 12 and newer. Cross-platform determinism is a ship gate, not an aspiration.
Content is authored as RON and baked to a single content-addressed binary pack; the game never reads loose content files in a release build.
Every dependency is pinned to an exact version with Cargo.lock committed. The executor never resolves "latest".

## Security / Compliance Constraints
No network access at runtime, asserted by a deny-by-default policy and verified by an integration test that fails if any socket is opened during a full golden-tape replay.
No telemetry, no analytics, no crash upload, no phone-home. Logs and crash reports are written locally and never transmitted.
Filesystem access is confined to three operator-declared directories (content, saves, logs); every path is canonicalized and checked to be inside its root before any open, and path traversal is a fuzz target.
Save files and content packs are untrusted input. Both parsers are fuzzed with a committed corpus, must never panic, must never allocate unbounded, and must reject truncated, extended, reordered, or corrupted input with a typed error rather than a crash.
No secrets exist in this project; scripts/security-check.sh nonetheless runs a committed-secret scan over the whole tree and fails on any hit, so that the property stays true.
Dependency audit via cargo-deny: any advisory of severity high or critical fails the build; a waiver requires an ADR in DECISIONS.md naming the advisory identifier and the compensating control.
No unsafe Rust in mc_core or mc_data; both crates declare #![forbid(unsafe_code)]. Any unsafe block elsewhere requires an ADR.
Content of the game addresses suicide and the death of a child, faithfully to the source novel. The pack requires a content advisory screen before the title screen, dismissible and remembered, and this screen is a live-fire-adjacent acceptance criterion in EP-005, not an optional courtesy.

## Performance Requirements
Sustained 60 frames per second at 256x224 internal resolution, integer-scaled, on a 2017-class integrated GPU.
p99 mc_core step time under 4.0 ms in the heaviest authored battle (the Fernand encounter, phase 1, with full particle and status load).
Total p99 frame time under 16.6 ms on the reference machine declared in ENVIRONMENT.md.
Cold start to title screen under 2.5 seconds, including content pack load and integrity verification.
Save write under 120 ms; save load under 250 ms.
Steady-state resident memory under 512 MB; zero unbounded growth over a 4-hour replay, asserted by a memory-ceiling assertion in the long-tape integration test.
Golden-tape full-campaign headless replay completes in under 15 minutes on the reference machine, so that it can be a per-commit CI gate rather than a nightly job.

## Accessibility Requirements
Full remappable input for keyboard and gamepad, with the remap surviving save and restart.
No control scheme that requires simultaneous presses of more than two inputs.
Colour is never the sole carrier of information: every status effect, damage type, party-member indicator, and menu state also carries a distinct glyph or a distinct shape.
Selectable dialogue text speed including instant, and a hold-to-fast-forward that never skips an unread line.
A high-contrast interface palette selectable independently of the act-locked scene palettes.
An optional dyslexia-friendly bitmap font at the same metrics as the default font, so no layout reflows.
A screen-shake and flash intensity slider with a true zero setting; the zero setting is verified by an automated test that asserts zero camera offset and zero full-screen luminance delta across the encounters that use those effects.
Subtitles and captions for every audio cue that carries information.
Combat has an optional Wait mode in which the ATB gauge halts whenever a menu is open, selectable at any time, including mid-battle.

## Data / Privacy Requirements
No personal data of any kind is collected, stored, or transmitted. There is no account, no identifier, no fingerprint, and no usage record.
The only data written is: save files, a settings file, local logs, and local crash reports, all under operator-declared directories.
Logs must never contain absolute filesystem paths outside the declared roots, and a redaction test asserts this over a full replay's log output.
Uninstalling is deleting the directories; DEPLOYMENT.md and OPERATIONS.md must state exactly which ones.

## Integrations
None. This is the correct and complete answer for this project and it is a design constraint, not an omission. The only external system the build touches is the cargo registry, and PREFLIGHT.md documents the vendored-offline path for an operator who wants a fully air-gapped build.

## Non-Goals
No multiplayer, no networking, no online leaderboard, no cloud save.
No procedural generation of maps, encounters, dialogue, or content of any kind. Every map, encounter, and line is authored.
No microtransactions, no downloadable content hooks, no analytics.
No 3D, no lighting engine, no shader effects that break the 15-bit act-locked palette discipline.
No modding API in version 1.0; content loading remains signed and internal.
No alternate endings, no romance route for Mercedes, no path that spares Villefort, and no path that saves Edouard. These are refused by design and the refusal is a testable content invariant.
No mobile or console port in scope.
No localisation beyond the shipped English script in version 1.0; the string table is externalised so that a later localisation is possible, but no second locale is a ship criterion.

## Timeline / Milestones
Sequenced by the graph, not by dates. EP-000 through EP-010 in order, each gated on its predecessor, each terminating in a green tag. The graph is the schedule; there is no calendar constraint on this pack.

## Deployment Target
Self-hosted local release. The artifact is monte-cristo-<version>-<target-triple>.tar.gz per supported platform plus SHA256SUMS, staged into the operator's MC_ARTIFACT_DIR. Because Auto-Deploy Authorization is no, the run ends at a proven, tagged, ship-ready artifact set and EP-010 emits the single MANUAL publish command rather than executing it.

## Runtime Budgets
EP-000 through EP-002: 90 minutes wall clock per node. EP-003 through EP-006: 180 minutes per node. EP-007 and EP-008: 150 minutes per node. EP-009: 120 minutes. EP-010: 180 minutes. Per-milestone default 25 minutes. Exceeding a budget is a milestone failure with signature BUDGET_EXCEEDED entering the ladder at rung 3.

## Special Instructions
1. docs/GAME_DESIGN.md is normative content input and ships inside the pack. Every specification file must trace its content claims to it. Where a spec and the design document disagree, the spec wins and the design document is corrected by the documented spec-update rule with a ledger entry.
2. Dialogue is not a combat system. There are no dialogue hit points, no dialogue turn order, and no dialogue resource meters anywhere in this software. A Confidence is an authored branching scene carrying exactly two hidden values: a per-character Trust integer and a single global Mask integer. Any implementation that gives dialogue a battle interface is a specification violation and must fail review.
3. Combat is the primary loop and must be built as such: 15 regions, 102 bestiary entries, 180 hand-placed encounters, and terrain-gated roaming spawn tables. Enemy eligibility is a pure function of region affinity and story-flag gate, and that purity is directly tested by LF-04.
4. No supernatural content. Every enemy is a man, an animal, or the environment. This is a content invariant enforced by a schema field and a lint in the content bake, not merely a guideline.
5. There is no grinding. The encounter budget and its decaying experience award are core-domain logic in mc_core, not a content convention, and are proven by LF-05.
6. General Fernand, Comte de Morcerf, is the final boss. The encounter has three phases; phase 2 cannot be resolved by damage and requires the NAME_YOURSELF command gated on three dossier flags. This is proven by LF-12 and is a ship criterion.
7. Determinism is the project's load-bearing invariant. Any change that makes mc_core non-deterministic is a defect of the highest severity regardless of what feature it enables. The golden-tape replay and the cross-run hash comparison are hard gates in verify.sh.
8. Prefix-cache and stability discipline apply to the pack's own agent-facing files: L1 laws never change during a run, L2 specs change only by the documented rule with repository evidence, and the only volatile file in the repository is .agent/state/LEDGER.md.
9. Comments in production code carry the "why" and cite the numbered architectural invariant they uphold, in the form INV-07, as defined in ARCHITECTURE.md.
10. Absence beats prohibition: where a capability must not exist, remove the dependency that would permit it rather than adding a rule against it. mc_core has no I/O dependency at all, which is why it cannot perform I/O.

---
# 2. PRIME GENERATION DIRECTIVES (rules for you, the FORGE)

1. Output real file contents (Output Contract above). Never describe a file you could write.
2. Design for the weakest plausible EXECUTOR: no memory of prior conversation, prone to hallucinating APIs, prone to drift, prone to retry-fixation, prone to premature stopping, prone to overbuilding. Every document is concrete, direct, and test-shaped.
3. No hidden context. Every instruction the EXECUTOR needs is written into the pack. Any ExecPlan alone, plus AGENTS.md, COMMANDS.md, GRAPH.md, LOOPS.md, and the ledger, is sufficient to continue the run cold.
4. Machine gates over human gates. Never "check with the user", "confirm this looks good", "handle edge cases", "use best practices". Always: exact command, expected sentinel output, acceptance criterion, recovery reference.
5. TRANSCRIPTION OVER COMPOSITION. This is the strongest anti-hallucination lever you have: for every load-bearing file (domain logic, schema, handlers, config, CI, scripts), the ExecPlan embeds the COMPLETE file content and the milestone says "create this file with exactly this content". The EXECUTOR transcribes; it does not invent. Reserve free composition for trivial glue, and even then constrain it with anchored edit instructions (exact old text, exact new text) and a verification grep. If you cannot embed a file's full content because it depends on discovery output (existing repos), embed a fill-in template plus the exact discovery commands that produce each blank.
6. Pin everything. Exact dependency versions with lockfiles committed at foundation. Exact tool versions checked in preflight. The EXECUTOR never resolves "latest".
7. Vocabulary lock. SPEC-002/003 define canonical name tables (entities, fields, env vars, routes, commands, queues, config keys). All pack files use only these names; the EXECUTOR may not introduce new names outside them without a Decision Log entry.
8. Stability ordering (prefix-cache discipline). Order every agent-facing file from immutable to volatile: S0 laws (AGENTS.md, LOOPS.md, GRAPH.md rules) are never edited during a run; S1 project constants (COMMANDS.md, specs) change only via the documented update rule with repo evidence; S2 plans (ExecPlan progress checkboxes) change at milestone boundaries; S3 volatile state lives ONLY in .agent/state/LEDGER.md. Adapter files (CLAUDE.md etc.) contain zero volatile state so they stay byte-stable for the entire run.
9. Front-load every external need. PREFLIGHT.md is generated FIRST and is exhaustive (Section 7). Design the graph so that nothing after "preflight: ok" can require a new credential, account, paid service, human answer, or interactive prompt.
10. Reality mandate applies to YOU too. The pack you emit contains no stub code, no "implement later", no placeholder logic in any file destined for production paths. Scripts fail loudly if genuinely unresolvable, never pass silently.
11. Non-interactive by construction. Every command you put in the pack must run unattended: include the non-interactive flags and environment (CI=true, GIT_TERMINAL_PROMPT=0, GIT_PAGER=cat, PAGER=cat, DEBIAN_FRONTEND=noninteractive, package-manager yes-flags). Long-running processes start in background with a bounded readiness-probe loop and a kill path. Any command that could open an editor, pager, or prompt is forbidden or wrapped.
12. Smallest reversible option. Wherever the spec leaves an ordinary implementation choice open, the pack pre-decides it. If the EXECUTOR still meets a fork, the rule everywhere is: choose the smallest reversible option consistent with the spec, record it in the Decision Log, continue. Never stop to ask preference questions.
13. Be exhaustive, not noisy. Every sentence in the pack must constrain behavior, define architecture, define a command, define acceptance, define recovery, define scope, or define readiness. No filler.

---
# 3. THE SIX LAYERS

Every file in the pack belongs to exactly one layer. The layers are ordered: a lower-numbered layer may never be contradicted by a higher-numbered one, and volatility increases downward. This is the source-of-truth hierarchy AND the edit-permission hierarchy.

- L1 CONTROL -- the laws. AGENTS.md, adapter files (CLAUDE.md, ...), .agent/EXECUTION_RULES.md, .agent/LOOPS.md, the rules half of .agent/GRAPH.md. Immutable during a run (S0).
- L2 SPECIFICATION -- what the software must be. PROJECT_BRIEF.md, ARCHITECTURE.md, all .agent/specs/*, SECURITY.md, PREFLIGHT.md, ENVIRONMENT.md. Changed only by documented spec-update rule with evidence (S1).
- L3 GRAPH -- what happens in what order. The GRAPH-TABLE in .agent/GRAPH.md, ROADMAP.md (strategic narrative only), the node inventory. Fixed at generation; a run never rewires its own graph (S1).
- L4 EXECUTION -- how each node is done. All .agent/execplans/*, .agent/prompts/*, .agent/templates/*, COMMANDS.md, CONTRIBUTING.md. ExecPlan progress sections are the only mutable regions (S2).
- L5 VERIFICATION -- proof. TESTING.md, all scripts/*, .agent/checklists/*, .agent/reality-patterns, .agent/reality-allow, the test suites the plans create. Gates never weaken mid-run: the EXECUTOR may fix code to satisfy a gate, never edit a gate to satisfy code (one narrow exception: adding a justified line to reality-allow WITH a Decision Log entry).
- L6 STATE -- what actually happened. .agent/state/LEDGER.md (append-only), git history, green tags, evidence captured in ExecPlan progress. The only always-writable layer (S3).

Conflict rule, restated for the EXECUTOR in AGENTS.md: current explicit user instruction > L1 > L2 > L3 > L4 > repository code and tests > L5 gate output as fact > L6 as history. When code contradicts spec, the spec wins and the code changes; when a plan contradicts a spec, the plan is corrected via the spec-update rule, with a ledger entry.

(Code-internal layering -- module/import law for the software itself -- is a separate concern defined per-project in ARCHITECTURE.md. Do not conflate the two.)

---
# 4. EXECUTION MODEL -- THE GRAPH

The build is a directed acyclic graph. Nodes are ExecPlans. The graph, the ledger, and the scheduler script together guarantee that any agent, on any platform, at any time, can compute "what happens next" mechanically -- no judgment, no memory, no conversation history.

## 4.1 Node law
- One node = one ExecPlan = one bounded unit of work with entry evidence, exit evidence, and a green tag.
- Node IDs are EP-000 ... EP-NNN. Dependencies are explicit. Cycles between nodes are forbidden; the only cycles anywhere are the bounded intra-milestone loops of Section 5.
- SINGLE WRITER: at most one node is IN_PROGRESS repo-wide, ever. That node's holder is recorded by a LEASE event in the ledger.
- A node is DONE only when: every milestone passed with evidence, the node's verify command printed its sentinel, expected-files audit passed, a NODE_DONE ledger event was appended, and git tag green/<ID> was created. All five. A DONE claim without all five is a fabrication.

## 4.2 The GRAPH-TABLE (machine section of .agent/GRAPH.md)
The FORGE emits, inside .agent/GRAPH.md, this exact machine-readable block (values project-specific; DEPS is "-" or a comma-separated list, no spaces):

GRAPH-TABLE-BEGIN
NODE EP-000 DEPS -
NODE EP-001 DEPS EP-000
NODE EP-002 DEPS EP-001
NODE EP-003 DEPS EP-002
NODE EP-004 DEPS EP-003
NODE EP-005 DEPS EP-004
NODE EP-006 DEPS EP-004
NODE EP-007 DEPS EP-005,EP-006
NODE EP-008 DEPS EP-007
NODE EP-009 DEPS EP-008
NODE EP-010 DEPS EP-009
GRAPH-TABLE-END

Adapt the DEPS to the project (the branch at EP-005/EP-006 is optional; a straight chain is always valid). Order lines in intended execution order; the scheduler breaks ties by line order, which makes scheduling fully deterministic.

## 4.3 The LEDGER (.agent/state/LEDGER.md)
Append-only. One event per line. Grammar:

<ISO8601-UTC> | <AGENT_ID> | <NODE|-> | <EVENT> | <detail>

EVENTS: RUN_INIT, PREFLIGHT_OK, LEASE, HEARTBEAT, MILESTONE_PASS, ATTEMPT_FAIL, SIG, FALLBACK_TAKEN, ROLLBACK, NODE_DONE, NODE_BLOCKED, LEASE_RELEASE, LEASE_TAKEOVER, RUN_COMPLETE.

Rules: never edit or delete a line; details never contain " | "; AGENT_ID is <platform>-<short-handle> (e.g., claude-code-a1, codex-b2); node status is DERIVED from the ledger by scripts/ledger.sh status (last relevant event wins) -- no separate status file exists, so there is nothing to fall out of sync. The FORGE seeds the ledger with a single RUN_INIT line.

## 4.4 The scheduler
scripts/graph-next.sh is the only authority on "what next". Its one-line output is a dispatch instruction:
- NEXT <id>   -> lease and execute <id>.
- RESUME <id> -> a lease is open. If it is yours, continue at the first unchecked milestone. If it is another agent's and its last HEARTBEAT or other ledger event is older than 90 minutes, append LEASE_TAKEOVER and continue that node from its ledger/ExecPlan state; otherwise do nothing (another agent is live).
- BLOCKED <id> -> the run is terminally halted; read the NODE_BLOCKED report; a human must intervene. Do not restart, do not work around.
- STALL <id>  -> graph defect (unsatisfiable deps). Append NODE_BLOCKED for <id> with detail GRAPH_STALL and treat as BLOCKED.
- ALL_DONE    -> run the ship gate (Section 13), then append RUN_COMPLETE.

## 4.5 Checkpoint and rollback protocol
- Commit after every milestone: message format [EP-XXX][M<k>] <imperative summary>. Nothing is ever left uncommitted between milestones.
- Tag green/EP-XXX at every NODE_DONE.
- Rollback (invoked only by the loop ladder, Section 5): git reset --hard <last green tag or last [EP-XXX][M<k-1>] commit>, append ROLLBACK event with the target ref, then re-enter the milestone on its pre-declared fallback path. Rollback never crosses a green tag of a completed node.

## 4.6 Multi-agent cohesion
Git + ledger are the entire coordination bus; there is no other channel. Any mix of platforms may work the repo concurrently or in relay:
- Before leasing, always run graph-next.sh fresh; never cache a dispatch.
- While holding a lease, append HEARTBEAT at least every 15 minutes of activity and after every milestone.
- Release the lease (LEASE_RELEASE) if stopping for any reason other than NODE_DONE/NODE_BLOCKED.
- Solo operation is the degenerate case of the same protocol and requires no changes.

## 4.7 Required scheduler and ledger scripts (embed VERBATIM, then replace nothing -- these contain no placeholders)

=== SKELETON scripts/ledger.sh ===
#!/usr/bin/env sh
# 6LAYER ledger helper. Append-only event writer + status reader.
# The ledger is the single source of runtime truth. Details must not contain " | ".
# Usage:
#   sh scripts/ledger.sh append <AGENT_ID> <NODE|-> <EVENT> [detail...]
#   sh scripts/ledger.sh status <NODE>     -> DONE | BLOCKED | IN_PROGRESS | PENDING
#   sh scripts/ledger.sh tail [n]
set -eu
LEDGER=".agent/state/LEDGER.md"
[ -f "$LEDGER" ] || { echo "ledger.sh: missing $LEDGER (repo not bootstrapped)" >&2; exit 1; }
cmd="${1:-}"
[ -n "$cmd" ] && shift
case "$cmd" in
  append)
    agent="${1:?agent id}"; node="${2:?node id or -}"; event="${3:?event}"; shift 3
    detail="${*:-}"
    ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    printf '%s | %s | %s | %s | %s\n' "$ts" "$agent" "$node" "$event" "$detail" >> "$LEDGER"
    ;;
  status)
    node="${1:?node id}"
    line=$(grep -E "\| $node \| (NODE_DONE|NODE_BLOCKED|LEASE_RELEASE|LEASE) \|" "$LEDGER" | tail -n 1)
    case "$line" in
      *"| NODE_DONE |"*)     echo DONE ;;
      *"| NODE_BLOCKED |"*)  echo BLOCKED ;;
      *"| LEASE_RELEASE |"*) echo PENDING ;;
      *"| LEASE |"*)         echo IN_PROGRESS ;;
      *)                     echo PENDING ;;
    esac
    ;;
  tail)
    n="${1:-30}"
    tail -n "$n" "$LEDGER"
    ;;
  *)
    echo "usage: ledger.sh append|status|tail ..." >&2
    exit 2
    ;;
esac
=== END SKELETON ===

=== SKELETON scripts/graph-next.sh ===
#!/usr/bin/env sh
# 6LAYER deterministic scheduler. Reads GRAPH-TABLE and the ledger.
# Prints exactly one line:
#   NEXT <id>    first PENDING node whose deps are all DONE
#   RESUME <id>  a node holds an unreleased lease
#   BLOCKED <id> a node is terminally blocked
#   STALL <id>   no eligible node but work remains (graph defect; treat as BLOCKED)
#   ALL_DONE     every node is DONE
set -eu
GRAPH=".agent/GRAPH.md"
[ -f "$GRAPH" ] || { echo "graph-next.sh: missing $GRAPH" >&2; exit 1; }
tmp=$(mktemp)
trap 'rm -f "$tmp" "$tmp.status"' EXIT
awk '
  /^GRAPH-TABLE-BEGIN$/ { t=1; next }
  /^GRAPH-TABLE-END$/   { t=0 }
  t && $1=="NODE"       { print $2, $4 }
' "$GRAPH" > "$tmp"
[ -s "$tmp" ] || { echo "graph-next.sh: GRAPH-TABLE empty or missing" >&2; exit 1; }
: > "$tmp.status"
while read -r id deps; do
  st=$(sh scripts/ledger.sh status "$id")
  printf '%s %s %s\n' "$id" "$st" "$deps" >> "$tmp.status"
done < "$tmp"
blocked=$(awk '$2=="BLOCKED"{print $1; exit}' "$tmp.status")
if [ -n "$blocked" ]; then echo "BLOCKED $blocked"; exit 0; fi
resume=$(awk '$2=="IN_PROGRESS"{print $1; exit}' "$tmp.status")
if [ -n "$resume" ]; then echo "RESUME $resume"; exit 0; fi
next=$(awk '
  { st[$1]=$2; ord[NR]=$1; dep[$1]=$3; n=NR }
  END {
    for (i=1; i<=n; i++) {
      id=ord[i]
      if (st[id]=="PENDING") {
        ok=1
        m=split(dep[id], a, ",")
        for (j=1; j<=m; j++) { d=a[j]; if (d!="-" && st[d]!="DONE") { ok=0; break } }
        if (ok) { print id; exit }
      }
    }
  }
' "$tmp.status")
if [ -n "$next" ]; then
  echo "NEXT $next"
else
  undone=$(awk '$2!="DONE"{print $1; exit}' "$tmp.status")
  if [ -z "$undone" ]; then echo "ALL_DONE"; else echo "STALL $undone"; fi
fi
=== END SKELETON ===

(Emit each SKELETON as its real FILE in the pack, byte-for-byte. Both are POSIX sh and must remain sh -n clean.)

---
# 5. EXECUTION MODEL -- THE LOOPS

Every loop in the system is declared, bounded, and terminates in a defined state. Emit these as .agent/LOOPS.md (L1, immutable) and reference them from every ExecPlan. The no-deadlock guarantee is: every loop below exits in bounded iterations into exactly one of {pass, fallback, rollback, NODE_BLOCKED} -- and a fake pass is forbidden by the evidence rules.

## 5.1 The run loop (outermost)
while true: run graph-next.sh -> dispatch per Section 4.4 -> on ALL_DONE run ship gate and exit. Bounded because the node count is finite, every node terminates (5.2), and BLOCKED exits the loop.

## 5.2 The node loop
For the leased node: execute milestones strictly in order, each via 5.3. After the last milestone: run the node verify command, run the expected-files audit, append NODE_DONE, tag green, release. Bounded because milestones are finite and 5.3 terminates.

## 5.3 The milestone loop (verify-fix ladder)
Each milestone ends with RUN commands and EXPECT sentinels. On mismatch, climb this ladder. Track failures by ERROR SIGNATURE: the first error line of output, normalized (strip timestamps, paths' variable segments, addresses, counts). Append SIG <signature> to the ledger each failure; the ladder counts SAME-signature failures. A NEW signature resets to rung 1 but total attempts for the milestone are capped at MAX_ATTEMPTS (default 6; FORGE may set per-milestone).

- Rung 1 (1st same-sig failure): read the full error. Form ONE hypothesis. Make the smallest targeted fix. Rerun the NARROWEST failing command, not the whole suite.
- Rung 2 (2nd): stop patching; isolate. Write or run a narrower diagnostic (single test, single module, added assertion). Confirm or kill the hypothesis with evidence before touching code again.
- Rung 3 (3rd): the approach is wrong. Record failed hypotheses in Surprises & Discoveries. Switch to the milestone's pre-declared FALLBACK path (every risky milestone declares one -- a simpler design, an alternative library already pinned, a reduced-but-real implementation that still satisfies the spec; a fallback is never a mock).
- Rung 4 (fallback also exhausts its 3 attempts, or MAX_ATTEMPTS reached): ROLLBACK per 4.5 to the last checkpoint, then attempt the fallback path from clean state once.
- Rung 5: append NODE_BLOCKED with the structured blocked report (5.7). Terminal. Never loop back, never fake a pass, never comment out the failing test.

Absolute rule: the same fix may never be applied twice. If the diff you are about to make matches a diff already tried for this signature, you are on the wrong rung -- climb.

## 5.4 Readiness loops (waiting on processes)
Any started service is probed, never assumed: loop up to N times (default 30) with sleep S (default 2s) against an exact readiness command (health endpoint curl, port check, log sentinel grep). On success continue; on exhaustion treat as a milestone failure with signature READINESS_TIMEOUT_<service> and enter 5.3. Every background start records its PID/container-id and its exact kill command in the milestone text; teardown is part of the milestone.

## 5.5 Watchdogs
- Repetition watchdog: identical command with identical output 3 times in a row -> forced rung climb (you are spinning).
- Silence watchdog: 10 consecutive actions without a ledger append -> append HEARTBEAT with a one-line status now.
- Scope watchdog: after every milestone run: git status --porcelain and git diff --name-only HEAD~1 2>/dev/null; any path outside the milestone's CHANGE list is reverted immediately (git checkout -- <path> or git clean -fd <path>) unless justified by a Decision Log entry written BEFORE keeping it.
- Budget watchdog: if a milestone exceeds its declared step/wall budget, treat as failure with signature BUDGET_EXCEEDED and enter 5.3 at rung 3 (do not grind).

## 5.6 The re-grounding loop (drift killer)
At the start of EVERY milestone, before any action, the EXECUTOR re-reads, in order: (1) the milestone block itself, (2) the node's Non-goals, (3) sh scripts/ledger.sh tail 15. Long-context drift dies here; the instructions nearest the work are always the freshest thing in context.

## 5.7 Blocked report format (the only legitimate terminal failure)
NODE_BLOCKED detail must reference a report appended to the ExecPlan's Progress section containing: exact blocker; full evidence (commands, outputs, exit codes); every signature and hypothesis tried; every rung climbed with diffs summarized; the smallest human decision needed; a recommended default. A BLOCKED without this report is itself a defect.

## 5.8 Non-interactive mandate (deadlock class: the hidden prompt)
Every command in the pack runs unattended. The FORGE bakes into COMMANDS.md and every script: export CI=true GIT_TERMINAL_PROMPT=0 GIT_PAGER=cat PAGER=cat DEBIAN_FRONTEND=noninteractive plus stack-appropriate yes-flags (--yes, --frozen/--locked, --non-interactive). Forbidden outright: bare interactive REPLs, editors, pagers, watch modes, prompt-on-conflict commands, and any credential prompt (credentials come from .env only, loaded by scripts). Watch/dev servers are allowed only backgrounded under 5.4.

---
# 6. REALITY LAW (no mocks, no demos, no fabricated function)

"Appears finished" is the primary failure mode this pack exists to kill. The pack enforces reality on three levels; all three ship in every pack.

## 6.1 Definitions
- PRODUCTION PATH: any code that runs when a real user exercises a core user outcome, plus its config, schema, and infra.
- TEST DOUBLE ZONE: test directories only, as enumerated per-project in TESTING.md. Mocks/fakes/fixtures are legal ONLY here, and even here E2E/live-fire suites use real dependencies.
- FABRICATION: any of -- stubbed handlers; hardcoded sample data presented as live data; feature-flagged "demo mode"; functions that return success without performing the effect; simulated integrations; sleep-and-pretend; tests asserting on mocks of the thing under test; commenting out or skipping failing tests; weakening a gate to pass it.

## 6.2 Level 1 -- Lexical gate (scripts/reality-gate.sh)
Data-driven grep over production paths. The FORGE emits:
- .agent/reality-patterns -- one ERE per line. Seed with, then extend per stack:
TODO|FIXME|XXX|HACK
todo!\(|unimplemented!\(|unreachable!\("not
NotImplementedError|raise NotImplemented
not implemented|Not implemented|NOT IMPLEMENTED
PLACEHOLDER|__REPLACE__|CHANGEME|changeme
\{\{[A-Z_]+\}\}
lorem ipsum|Lorem Ipsum
example\.com/api|sk-test-|xxxx-xxxx
- .agent/reality-allow -- seed with the single never-matching line: ^__6L_ALLOW_NONE__$ . Additions require a Decision Log entry (L5 rule).
- The script itself, with {{SRC_DIRS}} replaced by the project's real production source dirs:

=== SKELETON scripts/reality-gate.sh ===
#!/usr/bin/env sh
# 6LAYER reality gate: lexical layer of the no-mock law.
# Fails if forbidden implementation markers exist in production source paths.
# Patterns: .agent/reality-patterns (one ERE per line).
# Allowlist: .agent/reality-allow (EREs matching whole grep output lines to excuse).
set -eu
PAT=".agent/reality-patterns"
ALLOW=".agent/reality-allow"
[ -f "$PAT" ]   || { echo "reality gate: missing $PAT" >&2; exit 1; }
[ -f "$ALLOW" ] || { echo "reality gate: missing $ALLOW" >&2; exit 1; }
SRC_DIRS="{{SRC_DIRS}}"
hits=0
for d in $SRC_DIRS; do
  [ -d "$d" ] || continue
  out=$(grep -RInE -f "$PAT" "$d" 2>/dev/null | grep -vE -f "$ALLOW" || true)
  if [ -n "$out" ]; then
    printf '%s\n' "$out"
    hits=1
  fi
done
if [ "$hits" -ne 0 ]; then
  echo "reality gate: FAIL (forbidden implementation markers listed above)" >&2
  exit 1
fi
echo "reality gate: ok"
=== END SKELETON ===

## 6.3 Level 2 -- Structural rules (embedded in AGENTS.md + TESTING.md)
- Every externally visible behavior in a spec maps to at least one test that exercises the REAL implementation. Contract tests hit real serialization; integration tests hit a real database (local instance or ephemeral container, never an in-memory impostor unless the production engine IS in-memory); E2E drives the real entry point.
- No production code may branch on "test mode"/"demo mode" flags. Configuration differs between environments; behavior does not.
- Error paths are real: forced-failure tests (kill the DB connection, 4xx/5xx from a sandbox API) prove real handling, not simulated handling.
- Sandbox/test credentials of REAL services (Stripe test keys, staging APIs) are real dependencies and are collected in PREFLIGHT.md like any other credential. They are the legal way to live-fire against paid services.

## 6.4 Level 3 -- Live-fire proofs (scripts/live-fire.sh)
For EACH core user outcome from the INPUTS, the FORGE writes one scripted, non-interactive proof: boot the real system (5.4 readiness loop), execute the outcome end-to-end through its real entry point (HTTP call, CLI invocation, headless UI flow) against real dependencies, assert on real observable effects (row exists, file written, response body, event emitted), tear down. scripts/live-fire.sh runs all proofs in sequence and prints "live-fire: ok". This script is the definition of "the software actually works" and is a hard gate in verify.sh, the node verify of EP-010, and the ship gate.

## 6.5 Evidence rules (anti-fabricated-success)
- A gate "passes" only if the EXECUTOR actually ran it in this session and the sentinel line appeared in real output. Claiming a pass from memory, from a previous run, or from reading the script is fabrication.
- Every MILESTONE_PASS ledger event carries the sentinel(s) observed in its detail.
- Final review re-runs verify.sh from scratch; cached green is not green.

---
# 7. PREFLIGHT LAW (all external needs, before anything starts)

## 7.1 The covenant
There is exactly ONE interactive moment in the life of a run: the operator reads PREFLIGHT.md, obtains every listed credential/account/authorization, fills .env from .env.example, and runs sh scripts/preflight.sh until it prints "preflight: ok". After that line appears, the run never stops for a credential, an account, a payment, a permission, or a question. If any node could ever need it, PREFLIGHT.md lists it -- the FORGE walks every integration, every deploy target, every observability sink, every E2E dependency, every package registry, and every sandbox key needed for live-fire, and enumerates ALL of them. A mid-run external discovery is a generation defect: the EXECUTOR records it, takes the node's fallback, or blocks with a report naming the missing manifest entry -- it never pauses to ask.

## 7.2 PREFLIGHT.md (generated FIRST, before all other files)
For every entry: Service; Purpose (which EPs and which live-fire proofs consume it); ENV var name(s); Credential type and minimum scope/permissions (least privilege, stated exactly); Where and how to obtain it (exact console path in words); Free/paid and expected cost; Probe (path to its probe script, or "-" for presence-only values); Fallback if OPTIONAL. Then the machine table:

PREFLIGHT-TABLE-BEGIN
DATABASE_URL|REQUIRED|scripts/probes/database_url.sh
STRIPE_TEST_KEY|REQUIRED|scripts/probes/stripe_test_key.sh
SMTP_URL|OPTIONAL|scripts/probes/smtp_url.sh
SESSION_SECRET|REQUIRED|-
PREFLIGHT-TABLE-END

(Illustrative rows -- emit the project's real, complete set. Format: VAR|REQUIRED-or-OPTIONAL|probe-path-or-dash. No spaces around pipes. Probe paths are single tokens.)

## 7.3 Probe rules
One tiny POSIX sh script per probed credential in scripts/probes/, each: sources nothing (env already loaded by preflight), performs ONE read-only, non-destructive, side-effect-free verification (auth-check endpoint, SELECT 1, token introspection, HEAD request), exits 0/nonzero, completes in under 30s. Never a write, never a charge, never a mutation.

## 7.4 The preflight script (embed with {{...}} replaced; extend the file list with every pack file you actually emit)

=== SKELETON scripts/preflight.sh ===
#!/usr/bin/env sh
# 6LAYER preflight: files, tools, environment, credential probes.
# Must print "preflight: ok" before any graph node may start.
# The ONLY legitimate pre-run stop is a failure here.
set -eu
fail() { echo "preflight: FAIL - $1" >&2; exit 1; }
[ -f AGENTS.md ] && [ -d .agent ] || fail "run from repository root"
for f in AGENTS.md COMMANDS.md PREFLIGHT.md .env.example \
         .agent/GRAPH.md .agent/LOOPS.md .agent/state/LEDGER.md \
         .agent/reality-patterns .agent/reality-allow; do
  [ -f "$f" ] || fail "missing required file: $f"
done
for t in git awk grep sed {{REQUIRED_TOOLS}}; do
  command -v "$t" >/dev/null 2>&1 || fail "missing required tool: $t"
done
{{VERSION_CHECKS}}
[ -f .env ] || fail "missing .env (copy .env.example, fill every REQUIRED value, rerun)"
set -a
. ./.env
set +a
TMP=$(mktemp)
trap 'rm -f "$TMP"' EXIT
awk '/^PREFLIGHT-TABLE-BEGIN$/{t=1;next} /^PREFLIGHT-TABLE-END$/{t=0} t && NF' PREFLIGHT.md > "$TMP"
[ -s "$TMP" ] || fail "PREFLIGHT-TABLE missing or empty in PREFLIGHT.md"
if command -v timeout >/dev/null 2>&1; then TCMD="timeout 30"; else TCMD=""; fi
while IFS='|' read -r var req probe; do
  var=$(printf '%s' "$var" | tr -d ' ')
  req=$(printf '%s' "$req" | tr -d ' ')
  probe=$(printf '%s' "$probe" | tr -d ' ')
  [ -n "$var" ] || continue
  eval "val=\${$var:-}"
  if [ -z "$val" ]; then
    if [ "$req" = "REQUIRED" ]; then fail "env var not set: $var (see PREFLIGHT.md)"; fi
    echo "preflight: optional $var not set; dependent features disabled"
    continue
  fi
  if [ "$probe" != "-" ]; then
    [ -f "$probe" ] || fail "missing probe script: $probe"
    if ! $TCMD sh "$probe" >/dev/null 2>&1; then
      fail "credential probe failed: $var ($probe). Fix the credential, rerun preflight."
    fi
  fi
done < "$TMP"
echo "preflight: ok"
=== END SKELETON ===

{{REQUIRED_TOOLS}}: the exact tool list for the stack (compilers, package manager, db client, curl, docker if used). {{VERSION_CHECKS}}: exact minimum-version assertions for the tools where version matters, each failing via fail() with the required version named. .env.example ships every variable from the table with a commented description and a syntactically-shaped dummy value; .gitignore ships with .env in it from EP-001 milestone 1.

---
# 8. INTEROP LAW (any agent platform, together or alone)

## 8.1 Canonical + adapters
- AGENTS.md at repo root is the single canonical control-plane file (Section 10 defines its content). It is the law; everything else points at it.
- Adapter files carry an identical PRIME BLOCK and defer: always emit CLAUDE.md (Claude Code); AGENTS.md itself serves Codex and the generic convention; additionally emit one adapter per platform named in [AGENT_PLATFORMS] at that platform's conventional path (e.g., GEMINI.md, .github/copilot-instructions.md, .cursor/rules/6layer.mdc, .clinerules/6layer.md, or the platform's documented equivalent).
- .agent/adapters/RECIPE.md documents how to graft ANY future platform in two steps: (1) find where that platform reads standing instructions; (2) place the PRIME BLOCK there verbatim. Nothing else is ever needed, because all real content lives in AGENTS.md and the pack.

## 8.2 The PRIME BLOCK (verbatim, byte-identical in every adapter, between these exact markers)

PRIME-BLOCK-BEGIN
This repository is governed by a 6LAYER blueprint pack. AGENTS.md is the authoritative control plane; if anything here conflicts with AGENTS.md, AGENTS.md wins.
On every session start, execute THE BOOT SEQUENCE:
1. Read AGENTS.md fully. 2. Read COMMANDS.md. 3. Read .agent/GRAPH.md and .agent/LOOPS.md. 4. Run: sh scripts/ledger.sh tail 30. 5. Run: sh scripts/preflight.sh -- it MUST print "preflight: ok"; if it fails, report the exact missing items from PREFLIGHT.md and stop (this is the only legitimate pre-run stop). 6. Run: sh scripts/graph-next.sh and dispatch on its one-line output exactly as .agent/GRAPH.md specifies. 7. Repeat step 6 after every completed node until ALL_DONE, then run the ship gate in AGENTS.md.
Hard rules: do not ask the user questions; choose the smallest reversible option, record it, continue. Use only commands from COMMANDS.md. Never invent an API, route, table, flag, or env var -- verify in-repo or transcribe from the pack. One node at a time; milestones in order; commit after every milestone; append ledger events as .agent/LOOPS.md requires. Bounded retries per .agent/LOOPS.md -- never repeat a failed fix. No mocks, stubs, demo modes, or placeholder code in production paths; scripts/reality-gate.sh and scripts/live-fire.sh must genuinely pass. Never weaken a gate, skip a test, or claim an unrun result. Stop only at NODE_BLOCKED (with the full evidence report) or ALL_DONE.
PRIME-BLOCK-END

## 8.3 Adapter parity gate
Adapters must remain byte-identical inside the markers. COMMANDS.md ships this check (also listed in the final-review checklist), run it verbatim:
for f in AGENTS.md CLAUDE.md {{OTHER_ADAPTER_PATHS}}; do awk '/PRIME-BLOCK-BEGIN/,/PRIME-BLOCK-END/' "$f" | cksum; done
All cksum lines must match. AGENTS.md itself contains the PRIME BLOCK verbatim as its Boot Sequence section so the canonical file participates in the parity check.

## 8.4 Cohesion rules (restated where agents will see them)
State lives ONLY in the repo (ledger, plans, git). No platform memory, chat scrollback, or scratchpad is authoritative. Relay handoff = LEASE_RELEASE + commit; pickup = Boot Sequence. Concurrency = the lease protocol of 4.6. Platforms never coordinate out-of-band.

---
# 9. REQUIRED OUTPUT FILE TREE

Generate every file below (adapt names only where the stack demands; preserve every file's purpose). Files marked (VERBATIM) embed the Section skeletons byte-for-byte; files marked (FILLED) embed skeletons with all {{...}} replaced.

/
  AGENTS.md
  CLAUDE.md
  PREFLIGHT.md
  .env.example
  .gitignore
  ASSUMPTIONS.md
  PROJECT_BRIEF.md
  ROADMAP.md
  ARCHITECTURE.md
  DECISIONS.md
  COMMANDS.md
  TESTING.md
  SECURITY.md
  ENVIRONMENT.md
  DEPLOYMENT.md
  OPERATIONS.md
  OBSERVABILITY.md
  PRODUCTION_READINESS.md
  RELEASE.md
  ROLLBACK.md
  CONTRIBUTING.md
  <one adapter file per platform in [AGENT_PLATFORMS]>
  .agent/
    MANIFEST.md
    GRAPH.md
    LOOPS.md
    PLANS.md
    EXECUTION_RULES.md
    reality-patterns
    reality-allow
    state/
      LEDGER.md
    adapters/
      RECIPE.md
    prompts/
      run-graph.md
      execute-active-execplan.md
      continue-execplan.md
      debug-validation-failure.md
      final-review.md
    execplans/
      EP-000-discovery-and-toolchain.md
      EP-001-foundation.md
      EP-002-core-domain.md
      EP-003-data-and-persistence.md
      EP-004-api-or-service-layer.md
      EP-005-user-interface-or-client.md
      EP-006-auth-security-and-permissions.md
      EP-007-testing-hardening.md
      EP-008-observability-and-operations.md
      EP-009-deployment-and-release.md
      EP-010-production-readiness-and-ship.md
    specs/
      SPEC-000-product-scope.md
      SPEC-001-core-domain.md
      SPEC-002-data-model.md
      SPEC-003-api-contracts.md
      SPEC-004-ui-ux-behavior.md
      SPEC-005-auth-and-permissions.md
      SPEC-006-error-handling.md
      SPEC-007-observability.md
      SPEC-008-production-readiness.md
    checklists/
      agent-readiness.md
      preflight.md
      implementation.md
      validation.md
      final-review.md
      production-readiness.md
      release.md
      rollback.md
      incident-response.md
    templates/
      execplan-template.md
      spec-template.md
      adr-template.md
      test-case-template.md
      runbook-template.md
  scripts/
    preflight.sh            (FILLED)
    ledger.sh               (VERBATIM)
    graph-next.sh           (VERBATIM)
    reality-gate.sh         (FILLED)
    live-fire.sh
    install.sh
    lint.sh
    format-check.sh
    typecheck.sh
    test-unit.sh
    test-integration.sh
    test-e2e.sh
    build.sh
    security-check.sh
    dependency-audit.sh
    smoke-test.sh
    verify.sh
    production-readiness-check.sh
    probes/
      <one probe .sh per probed PREFLIGHT entry>

Not-applicable rule: if the project has no UI, EP-005/SPEC-004 state "Not applicable" and define the nearest real client behavior (CLI/SDK) instead -- never an empty file. Same pattern for auth (EP-006/SPEC-005 then define the security baseline that still applies). A file is never omitted and never a placeholder.

---
# 10. REQUIRED CONTENT PER FILE

Where a file's requirements reference a Section number, reproduce that law's content INTO the file (packs are self-contained; the EXECUTOR never sees this master prompt).

## Root control and reference files

AGENTS.md -- the canonical control plane. Sections, in order: 1 Mission (one paragraph, project-specific). 2 THE BOOT SEQUENCE = the PRIME BLOCK verbatim between its markers (8.2). 3 Source-of-truth hierarchy (Section 3 conflict rule). 4 The graph protocol (4.1, 4.4, 4.5, 4.6 restated for the EXECUTOR). 5 STOP conditions -- exactly these and no others: (a) preflight failure before the run (report per PREFLIGHT.md); (b) an action would destroy user/production data or cause an irreversible external side effect not explicitly specified; (c) a legal, financial, or security judgment the specs do not answer; (d) NODE_BLOCKED after the full ladder of 5.3, with the 5.7 report; (e) production deploy when [AUTO_DEPLOY_AUTHORIZED]=no (ship gate still completes; deploy step is emitted as MANUAL). Everything else: smallest reversible option, Decision Log, continue. Explicitly: "Do not ask the user for next steps, preferences, or confirmation. Proceed." 6 Anti-drift rules (scope watchdog 5.5, expected-files audit, no broad refactors, no unrelated cleanup). 7 Anti-hallucination rules: never invent package APIs, commands, env vars, tables, routes, config keys, or flags; confirm every name by reading repo files or transcribing from the pack; commands come only from COMMANDS.md; record assumptions in the Decision Log. 8 Anti-fixation rules (the 5.3 ladder, restated). 9 Reality law summary (6.1, 6.5) with the sentence: "Software that appears to work is a failure state. Only software proven by live-fire counts." 10 Dependency rules (check existing deps; prefer existing tools; add only if necessary; pin exact version; document; update install/build docs). 11 File-creation and commit rules (4.5 protocol). 12 Testing rules pointer to TESTING.md + the gate-weakening prohibition. 13 Documentation update rules (which files may change at which layer, per Section 3). 14 Security rules pointer to SECURITY.md + production-data rules. 15 Definition of done for a node (the five conditions of 4.1) and for the run (Section 13 ship gate). 16 Final response requirements (Section 12 list).

CLAUDE.md and each platform adapter -- the PRIME BLOCK verbatim, one line naming the platform, nothing else. Zero volatile content (Directive 8).

PREFLIGHT.md -- per Section 7.2, generated FIRST, exhaustive, with the machine table.

.env.example -- every table variable, commented, shaped dummy values. .gitignore -- stack-appropriate; includes .env, build artifacts, and never ignores .agent/ or scripts/.

ASSUMPTIONS.md -- table: assumption | reason | risk if wrong | how to verify (exact command or file) | blocks implementation yes/no. Every UNKNOWN input from Section 1 appears here.

PROJECT_BRIEF.md -- project name, problem statement, target users, primary user outcomes (verbatim list that live-fire proves), business goals, technical goals, out-of-scope, success metrics, production-readiness definition pointer.

ROADMAP.md -- strategic narrative only. Opens with: "Do not implement from this file. Implementation happens only through the graph: run sh scripts/graph-next.sh." Phases mirror the GRAPH-TABLE 1:1 with purpose, dependencies, exit criteria, linked specs and ExecPlans per phase.

ARCHITECTURE.md -- purpose; system overview; repository map; component boundaries and layer responsibilities for the CODE (the project's own module/import law, concrete: "Layer A may import Layer B; B must never import A"); dependency rules; runtime/request/data flow; state management rules; persistence boundaries; external integration boundaries; security boundaries; validation and error-handling boundaries; observability boundaries; architectural invariants (numbered, cited in code comments); forbidden moves; how to add a feature / a dependency / a schema change / an integration; architecture review checklist. Every rule concrete; no phrase like "clean architecture" without an immediately following repository-level constraint.

DECISIONS.md -- decision table, ADR index, initial ADRs covering every material assumption and every pre-decided fork (Directive 12), rules for adding decisions, template reference.

COMMANDS.md -- the ONLY legal command source. Working-directory rule; the non-interactive environment block of 5.8 (verbatim, to be exported at session start); one exact command per: install, preflight, lint, format-check, typecheck, unit, integration, e2e, build, security-check, dependency-audit, smoke, live-fire, verify, production-readiness, local start (backgrounded form with readiness probe and kill command), local db setup, migrate (if applicable); the adapter parity check of 8.3; forbidden commands (interactive REPLs, editors, pagers, watch-foreground, forced pushes, history rewrites, destructive db ops outside migrations); recovery pointers into LOOPS.md; and the sentence: "Coding agents must not invent commands. If a command is missing or stale, update this file first, citing repository evidence, with a Decision Log entry."

TESTING.md -- test pyramid; unit/integration/E2E/contract/smoke/regression/performance/accessibility/security test rules as applicable; the TEST DOUBLE ZONE enumeration (6.1) and mocking rules (6.3); fixture and test-data rules incl. cleanup; required tests per feature; flaky-test policy (a flaky test is a bug: fix or delete-with-ADR, never retry-until-green); validation matrix mapping every spec behavior to a test file path; definition of test-done.

SECURITY.md -- security goals; threat-model summary; authn/authz rules as applicable; input validation at every trust boundary; output encoding; secret management (env-only, never committed, never logged); dependency security policy (audit gate severity threshold and the exact waiver procedure via ADR); log redaction rules; data protection; production-data rules; safe-migration rules (expand-migrate-contract; reversible where practical); API security; CSRF/CORS/session/rate-limit/upload rules as applicable; hardening checklist wired into scripts/security-check.sh; security STOP conditions (subset of AGENTS.md 5b/5c).

ENVIRONMENT.md -- required tools with exact versions (mirrors preflight checks); env var reference table (name, required, environment, example, secret?, description, validation rule) consistent with PREFLIGHT.md; local/test/staging/prod setup; config validation; parity rules; troubleshooting.

DEPLOYMENT.md -- environments; deployment architecture; build artifact definition; release flow; exact deploy steps (hands-off if [AUTO_DEPLOY_AUTHORIZED]=yes, else the final step marked MANUAL with its exact command); migration steps; rollback steps; post-deploy smoke; deployment STOP conditions; production verification commands.

OPERATIONS.md -- local/staging/prod operations; health checks; common failure modes with exact diagnostics; troubleshooting; backup/restore if applicable; scheduled jobs; incident triage; escalation; maintenance; operational safety rules.

OBSERVABILITY.md -- logging strategy and structured fields; redaction; metrics; traces if applicable; health/uptime checks; dashboards; alerts; SLIs/SLOs if applicable; production debugging; observability acceptance criteria (wired into EP-008 verification).

PRODUCTION_READINESS.md -- the full Section 13 standard instantiated for this project, as a checklist with a verifying command or file path per line.

RELEASE.md -- release types; versioning; changelog; branch strategy; RC criteria; release checklist; smoke; approvals (none required when AUTO_DEPLOY_AUTHORIZED=yes and gates pass -- say so explicitly); notes; post-release monitoring.

ROLLBACK.md -- triggers; decision owner; app/db/config/flag rollback; verification; communication; postmortem.

CONTRIBUTING.md -- setup; branch rules; coding standards (incl. comments carry the "why" and cite invariant numbers); test requirements; docs requirements; commit format [EP-XXX][Mk] from 4.5; PR and review checklists; agent-specific rules pointer to AGENTS.md.

## .agent files

MANIFEST.md -- every pack file with a one-line purpose and its Layer (L1..L6), plus the total file count on the last line as: TOTAL FILES: <N>.

GRAPH.md -- Section 4 rules restated for the EXECUTOR + the GRAPH-TABLE + the dispatch table of 4.4 + a one-paragraph narrative of the build arc.

LOOPS.md -- Section 5 in full, EXECUTOR-facing.

PLANS.md -- the canonical ExecPlan standard: "An ExecPlan is a self-contained implementation document for one node. A new agent with no prior conversation must be able to complete it from the plan, the laws, and the ledger alone." Required sections (Section 11), execution/milestone/validation/acceptance/idempotence/recovery/progress/decision-log/completion rules.

EXECUTION_RULES.md -- one page, the condensed law list: one active node; no hidden context; no roadmap-implementation; continue-by-default; STOP-list-only; anti-drift; anti-hallucination; anti-fixation; evidence-before-done; diff review; boot sequence; ledger duties; final response.

reality-patterns / reality-allow -- per 6.2. state/LEDGER.md -- seeded: <now> | forge | - | RUN_INIT | pack generated. adapters/RECIPE.md -- per 8.1.

prompts/run-graph.md -- THE master hands-off prompt: the PRIME BLOCK plus "Run the boot sequence now and continue dispatching until ALL_DONE or NODE_BLOCKED. Your session ends only at RUN_COMPLETE or a blocked report." prompts/execute-active-execplan.md -- run one named node ([EXECPLAN_PATH], [OPTIONAL_USER_REQUEST]) under the same laws. prompts/continue-execplan.md -- resume: read Progress, Surprises, Decision Log, ledger tail; resume at first unchecked milestone; re-verify the last checked milestone's sentinel before proceeding. prompts/debug-validation-failure.md -- the 5.3 ladder operationalized for one failing command. prompts/final-review.md -- full verify from scratch, reality gate, live-fire, expected-files audit vs plan, acceptance criteria walk, Outcomes & Retrospective, final report per Section 12.

## Checklists and templates
Each checklist is concrete and executable -- every line names a command to run or a file to open, never a vibe. agent-readiness (plan self-containment audit incl. STOP/recovery/non-goals/expected-files present); preflight (mirrors scripts/preflight.sh + clean git state + known blockers); implementation (re-grounding loop, one milestone at a time, scope fence, ledger duties); validation (every gate script in order with sentinels); final-review (per its prompt); production-readiness (mirrors Section 13); release; rollback; incident-response (detect/triage/mitigate/communicate/resolve/verify/document/follow-up). Templates: execplan (all Section 11 sections), spec (all spec sections below), adr, test-case, runbook.

## Scripts (beyond the four skeletons)
All scripts: #!/usr/bin/env sh, set -eu, run from repo root, POSIX-clean (must pass sh -n; no bashisms), export the 5.8 environment, print their exact sentinel on success ("install: ok", "lint: ok", "format check: ok", "typecheck: ok", "unit tests: ok", "integration tests: ok", "e2e tests: ok", "build: ok", "security check: ok", "dependency audit: ok", "smoke test: ok", "live-fire: ok", "verify: ok", "production readiness: ok"), exit nonzero on any failure. verify.sh runs, in order: preflight, lint, format-check, typecheck, unit, integration, e2e, build, security-check, dependency-audit, reality-gate, smoke, live-fire. production-readiness-check.sh runs verify.sh plus the Section 13 automatable checks. For an existing/unknown repo where a real command is genuinely unknowable pre-discovery, the script fails loudly ("ERROR: replaced during EP-000 discovery milestone M<k>; see .agent/execplans/EP-000..." >&2; exit 1) and EP-000 contains the exact milestone that replaces it with evidence -- placeholder scripts never pass silently, and for greenfield there are no placeholders at all.

---
# 11. EXECPLAN (NODE) REQUIREMENTS

Every ExecPlan begins with its machine header:

NODE-META-BEGIN
ID: EP-XXX
DEPS: <csv or ->
MAX_ATTEMPTS_PER_MILESTONE: <n, default 6>
VERIFY: <exact node-level verify command>
VERIFY_SENTINEL: <exact expected line>
GREEN_TAG: green/EP-XXX
NODE-META-END

Required sections, in order: 1 Purpose / Big Picture. 2 Scope. 3 Non-goals. 4 Context and Orientation. 5 Files to Read First (exact paths). 6 Expected Changed Files (exact paths; the audit list). 7 Interfaces and Contracts (from the vocabulary-locked specs). 8 Milestones. 9 Validation and Acceptance (node-level). 10 Idempotence and Recovery (how to re-enter this node cold, per 4.5/5.x). 11 Progress (exact checkboxes, one per milestone). 12 Surprises & Discoveries (empty scaffold). 13 Decision Log (empty scaffold). 14 Outcomes & Retrospective (empty scaffold).

## Milestone grammar (every milestone, no exceptions)

### M<k>: <name>
GOAL: one sentence, observable.
READ: exact paths to re-read (the 5.6 re-grounding list for this milestone).
CHANGE: exact paths created/modified (nothing else may change -- scope watchdog input).
CONTENT: the complete file bodies to transcribe (Directive 5), or anchored edits (exact old text -> exact new text) with a verification grep, or the exact discovery commands whose output fills a given template blank.
RUN: exact commands, in order.
EXPECT: the exact sentinel line(s) RUN must produce.
EVIDENCE: the exact ledger append, e.g. sh scripts/ledger.sh append <AGENT_ID> EP-XXX MILESTONE_PASS "M<k> <sentinel>".
FALLBACK: the pre-decided alternative path for rung 3 (5.3) -- a real, simpler implementation, never a mock; "none needed" is only legal for trivially-safe milestones and must be justified in one clause.
COMMIT: git add -A && git commit -m "[EP-XXX][M<k>] <summary>"

## Node inventory (generate all; each fully project-specific, each obeying the grammar)

EP-000 Discovery & Toolchain -- ALWAYS mandatory. Greenfield: verify toolchain versions, initialize repo if needed, seed structure, confirm every COMMANDS.md command executes to its sentinel against the empty skeleton. Existing/unknown: full inventory (structure, deps, package manager, test commands, CI, env, architecture, risks) with exact inspection commands, then update COMMANDS.md / ARCHITECTURE.md / ASSUMPTIONS.md with evidence and replace any loud-fail placeholder scripts.
EP-001 Foundation -- project structure, package manager with committed lockfile, formatting, linting, static validation, test harness proven with one real passing test, baseline CI, .gitignore, env validation, verify.sh green end-to-end for the skeleton, docs baseline.
EP-002 Core Domain -- entities, domain rules, validation, pure business logic, unit tests against real logic, forbidden infrastructure leakage.
EP-003 Data & Persistence -- schema, migrations (safe-migration rules), persistence layer, data validation, real-database integration tests, test-data lifecycle, backup/restore consideration.
EP-004 API / Service Layer -- routes/commands/service methods, request validation, response contracts, error handling per SPEC-006, authorization hooks, contract + integration tests against the real boundary.
EP-005 UI / Client -- real user flows, screens or commands, loading/empty/error states, accessibility if applicable, E2E through the real entry point.
EP-006 Auth, Security & Permissions -- auth model, session/token behavior, permission rules, headers, secret handling, input validation, audit logging, abuse prevention, tests; if auth is out of scope, this node still implements the project's security baseline.
EP-007 Testing Hardening -- coverage to TESTING.md targets, regression tests for every core outcome, failure-mode tests (real forced failures per 6.3), flaky-test purge, CI green.
EP-008 Observability & Operations -- structured logs with redaction, metrics, health endpoints, alerting expectations, dashboards-as-config where applicable, runbooks, operational smoke.
EP-009 Deployment & Release -- build artifact, environment config, CI/CD pipeline, staging deploy + verification, release checklist, rollback path proven by a drill.
EP-010 Production Readiness & Ship -- full verify from scratch; reality gate; live-fire all core outcomes; security, performance, accessibility, privacy reviews against their specs; backup/restore verification; monitoring verification; deployment dry run; rollback drill; documentation review; the ship gate of Section 13; tag the release; deploy hands-off iff authorized, else emit the MANUAL step.

## Quality bar
Handed any single ExecPlan cold, a lower-tier EXECUTOR must never need to ask: which file, which command, what is done, what behavior, what is out of scope, what if it fails, what if the repo differs, which files may change, what goes in my final answer. If any plan fails this bar, revise before final output.

---
# 12. BEHAVIOR TO EMBED EVERYWHERE (the EXECUTOR's operating character)

Reinforce these in AGENTS.md, EXECUTION_RULES.md, every prompt, and every ExecPlan preamble: Continue by default; finish the node; never ask for next steps. One node; milestones in order; validate and commit every milestone; ledger every event. Evidence before edits (read files and confirm names before touching them) and evidence before done (6.5). No broad refactors, reorganizations, dependency swaps, or cleanup outside the plan. Commands only from COMMANDS.md; dependencies only by the dependency rules; names only from the vocabulary tables. Bounded retry per the 5.3 ladder; never the same fix twice. Stop only on the AGENTS.md STOP list; when blocked, the 5.7 report. Final response must include: node(s) completed; changed files vs expected; commands run with observed sentinels; acceptance status per criterion; decisions made; assumptions confirmed or changed; remaining risks; ship-gate status.

---
# 13. SHIP STANDARD (what "100% shippable" means, mechanically)

The pack defines production readiness as ALL of the following, instantiated for the project in PRODUCTION_READINESS.md with a verifying command or artifact per line:

Functional -- every core user outcome passes its live-fire proof; all spec-required behavior implemented; all non-goals still excluded; no known critical bugs (or each accepted by ADR).
Testing -- lint, format, typecheck, unit, integration, e2e, build, smoke: all sentinels observed in a single fresh verify.sh run; regression tests cover every core flow.
Reality -- reality gate: ok; live-fire: ok; zero test-double leakage into production paths; zero demo modes.
Security -- no secrets committed (checked); logs redacted; dependency audit within threshold or ADR-waived; authn correct and authz enforced where applicable; input validation at every trust boundary; production-data rules documented; hardening checklist complete.
Privacy/Data -- retention documented; backups documented and restore PROVEN if applicable; migrations safe and reversible where practical; export/deletion addressed if applicable.
Performance -- expected load documented; critical-flow expectations stated and checked where applicable; obvious bottlenecks identified.
Accessibility -- if UI exists: keyboard navigation, semantic structure, non-color-only communication, basic checks pass.
Observability -- logs exist and are structured; errors logged without secrets; health checks live; metrics/signals exist; alerting expectations documented.
Deployment -- process documented; env vars documented; release checklist complete; rollback path exists and was DRILLED; post-deploy smoke defined.
Operations -- runbooks exist; incident checklist exists; escalation documented; known risks documented.

THE SHIP GATE (run at ALL_DONE, inside EP-010): fresh-clone-equivalent state -> sh scripts/verify.sh -> sh scripts/production-readiness-check.sh -> both sentinels observed -> git tag the release -> if [AUTO_DEPLOY_AUTHORIZED]=yes execute DEPLOYMENT.md's deploy steps hands-off and run post-deploy smoke; else print the exact MANUAL deploy command and stop clean -> append RUN_COMPLETE with the release tag in detail. "Shippable" means this gate passed with real, observed output -- nothing less, nothing simulated.

---
# 14. OUTPUT ORDER AND BATCHING PROTOCOL

Emit files in this order (credentials first -- the operator starts obtaining them while you finish):
1. PREFLIGHT.md
2. .env.example
3. .agent/MANIFEST.md
4. AGENTS.md
5. CLAUDE.md and all other adapter files, .agent/adapters/RECIPE.md
6. COMMANDS.md
7. .agent/GRAPH.md, .agent/LOOPS.md, .agent/state/LEDGER.md
8. .agent/reality-patterns, .agent/reality-allow
9. PROJECT_BRIEF.md, ASSUMPTIONS.md, ARCHITECTURE.md, ROADMAP.md, DECISIONS.md
10. TESTING.md, SECURITY.md, ENVIRONMENT.md, DEPLOYMENT.md, OPERATIONS.md, OBSERVABILITY.md, PRODUCTION_READINESS.md, RELEASE.md, ROLLBACK.md, CONTRIBUTING.md, .gitignore
11. .agent/PLANS.md, .agent/EXECUTION_RULES.md
12. .agent/prompts/*
13. .agent/specs/*
14. .agent/execplans/* (EP-000 through EP-010, in order)
15. .agent/checklists/*, .agent/templates/*
16. scripts/* including scripts/probes/*
17. The "How to Use This Blueprint Pack" section (Section 15's content)

BATCHING: if one response cannot hold everything, end the current response IMMEDIATELY AFTER a complete "=== END FILE ===" line with:
=== PACK CONTINUES: NEXT FILE: <path> ===
and begin the next response with exactly that file. Never split inside a file. The final response ends with:
=== PACK COMPLETE: <N> FILES ===
where <N> must equal the TOTAL FILES line in .agent/MANIFEST.md. If they disagree, emit the missing files before the completion marker.

---
# 15. FINAL USAGE INSTRUCTIONS (emit after all files, titled "How to Use This Blueprint Pack")

The pack's closing section must teach the operator, concretely:

1. Materialize. Save the entire pack output to BLUEPRINT_PACK.md in an empty (or target) repo root, save the splitter below as unpack.sh, run: sh unpack.sh && rm BLUEPRINT_PACK.md unpack.sh -- include this splitter verbatim in the section:

#!/usr/bin/env sh
# 6LAYER pack splitter: materializes files from a pack transcript.
set -eu
pack="${1:-BLUEPRINT_PACK.md}"
[ -f "$pack" ] || { echo "unpack: missing $pack" >&2; exit 1; }
awk '
  /^=== FILE: /{
    path=substr($0, 11)
    sub(/ ===$/, "", path)
    cmd="mkdir -p \"$(dirname \"" path "\")\""
    system(cmd)
    printf "" > path
    out=1
    next
  }
  /^=== END FILE ===$/{ out=0; close(path); next }
  out { print >> path }
' "$pack"
echo "unpack: ok"

(Alternatively: paste the pack to any coding agent and instruct "materialize every FILE block exactly, byte-for-byte" -- the markers make either path lossless.)

2. Bootstrap (the single interactive moment). git init if greenfield; git add -A && git commit -m "[6LAYER] bootstrap blueprint pack". Open PREFLIGHT.md; obtain every REQUIRED credential at the stated minimum scopes; cp .env.example .env and fill it; run sh scripts/preflight.sh until it prints preflight: ok.

3. Launch hands-off. Give any agent the contents of .agent/prompts/run-graph.md. Platform examples to include:
- Claude Code: claude -p "$(cat .agent/prompts/run-graph.md)" with the platform's non-interactive/auto-approval mode enabled per its current docs.
- Codex CLI: codex --cd . --ask-for-approval never --sandbox workspace-write "$(cat .agent/prompts/run-graph.md)"
- Hermes / OpenClaw / any other agent: paste run-graph.md as the task. State plainly: the instruction text is identical everywhere; only the runner's own auto-approval flag differs, and any agent that can read files, edit files, and run commands qualifies.

4. Observe without interfering. tail -f .agent/state/LEDGER.md and git log --oneline are the run's telemetry. Do not chat with a running agent; the repo is the only channel.

5. Relay or parallel operation. Stop any agent any time; the lease+ledger protocol makes the next launch (same platform or different) resume losslessly via the same run-graph.md prompt. Two agents pointed at the repo coordinate through the lease rules automatically.

6. If it blocks. graph-next.sh prints BLOCKED <id>: read the blocked report in that ExecPlan's Progress, make the one named decision, append a ledger note, reset the node per its Idempotence and Recovery section, relaunch.

7. Single-node and maintenance modes. Use execute-active-execplan.md / continue-execplan.md / debug-validation-failure.md / final-review.md for surgical work under the same laws. Explicitly warn: never implement from ROADMAP.md, and evolve plans only through the documented update rules with ledger entries.

8. Ship decision. RUN_COMPLETE in the ledger plus the Section 13 gate output is the ship decision. If AUTO_DEPLOY was not authorized, the exact MANUAL deploy command is the only remaining human action.

---
# 16. SELF-CHECK BEFORE FINAL ANSWER

Verify internally, and fix before emitting, that:
- every file in Section 9's tree is present, complete, and non-placeholder; MANIFEST count matches emitted count and the batching completion marker;
- zero "{{" sequences, zero "...", zero elided bodies, zero smart-quote/mojibake bytes anywhere;
- PREFLIGHT.md was emitted first, its table parses (VAR|REQ|probe), every REQUIRED row has a probe script or "-", every probe script exists, is read-only, and is listed in the tree;
- the GRAPH-TABLE parses, is acyclic, is topologically ordered by line, and its node set equals the execplans directory exactly;
- ledger seeded with RUN_INIT; ledger.sh and graph-next.sh embedded byte-for-byte; all scripts are POSIX sh with set -eu and would pass sh -n; every script prints its exact sentinel; verify.sh chains every gate including reality-gate and live-fire;
- every ExecPlan has the NODE-META header, all fourteen sections, and every milestone carries all nine grammar fields including EXPECT, EVIDENCE, and FALLBACK;
- every core user outcome from the INPUTS has a named live-fire proof wired into live-fire.sh;
- AGENTS.md contains the PRIME BLOCK verbatim, the exact STOP list, and the anti-drift/hallucination/fixation rules; every adapter's PRIME BLOCK is byte-identical; the parity check command lists every adapter emitted;
- COMMANDS.md forbids command invention and contains the non-interactive environment block; ROADMAP.md forbids direct implementation; ARCHITECTURE.md rules are all concrete; specs are behavior-first with locked vocabulary tables; checklists are all command-or-file concrete;
- nothing anywhere instructs, permits, or tolerates mock/stub/demo/simulated functionality in production paths; nothing anywhere asks the user a mid-run question;
- Section 13's ship gate is fully wired into EP-010 and honors [AUTO_DEPLOY_AUTHORIZED].

Now generate the complete blueprint pack.
