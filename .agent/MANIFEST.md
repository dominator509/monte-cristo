# MANIFEST -- MONTE CRISTO 6LAYER blueprint pack

Every file in the pack, its one-line purpose, and its layer.

Layers: L1 CONTROL (laws, immutable during a run) - L2 SPECIFICATION - L3 GRAPH -
L4 EXECUTION - L5 VERIFICATION - L6 STATE (the only always-writable layer).

| File | Layer | Purpose |
|---|---|---|
| `.agent/EXECUTION_RULES.md` | L1 | One-page condensed law list for the executor |
| `.agent/GRAPH.md` | L1/L3 | Build arc narrative, the GRAPH-TABLE, node law, dispatch table, ledger grammar, checkpoint rules |
| `.agent/LOOPS.md` | L1 | Every loop declared and bounded: run, node, milestone ladder, readiness, watchdogs, re-grounding, blocked report |
| `.agent/MANIFEST.md` | L2 | This file: every pack file with its purpose and layer, plus the total count |
| `.agent/PLANS.md` | L4 | The ExecPlan standard: machine header, fourteen sections, nine-field milestone grammar, quality bar |
| `.agent/adapters/RECIPE.md` | L1 | How to graft any future agent platform in two steps |
| `.agent/checklists/agent-readiness.md` | L5 | Plan self-containment audit before executing any node |
| `.agent/checklists/final-review.md` | L5 | Fresh verify, expected-files audit, acceptance walk, documentation truth |
| `.agent/checklists/implementation.md` | L5 | Per-milestone: re-ground, scope fence, evidence, ledger, commit |
| `.agent/checklists/incident-response.md` | L5 | Detect, triage, mitigate, communicate, resolve, verify, document, follow up |
| `.agent/checklists/preflight.md` | L5 | Environment truth plus clean tree and no open lease |
| `.agent/checklists/production-readiness.md` | L5 | Mirror of PRODUCTION_READINESS.md, block by block |
| `.agent/checklists/release.md` | L5 | Version, changelog determinism note, artifacts, checksums, determinism drill, MANUAL stop |
| `.agent/checklists/rollback.md` | L5 | Trigger confirmation, execution, verification, communication, postmortem |
| `.agent/checklists/validation.md` | L5 | Every gate in order with its sentinel, plus project-specific spot checks |
| `.agent/execplans/EP-000-discovery-and-toolchain.md` | L4 | Pin the toolchain, create the three roots, resolve the registry, prove preflight and the scheduler |
| `.agent/execplans/EP-001-foundation.md` | L4 | Five-crate workspace, committed lockfile, formatting, linting, layer DAG, one real test, CI, verify green |
| `.agent/execplans/EP-002-core-domain.md` | L4 | mc_core in full: the entire simulation, pure and deterministic, with its property tests |
| `.agent/execplans/EP-003-data-and-persistence.md` | L4 | RON schemas, bake with seven validators, content pack, versioned saves, migration, forced failures |
| `.agent/execplans/EP-004-api-or-service-layer.md` | L4 | Command bus, StateView, tick contract, tape format, record and replay, the Act I tape |
| `.agent/execplans/EP-005-user-interface-or-client.md` | L4 | macroquad shell: render target, palettes, battle UI, Confidence presentation, audio, accessibility, advisory |
| `.agent/execplans/EP-006-auth-security-and-permissions.md` | L4 | Confinement, parser hardening, three fuzz targets, no-socket proof, redaction, the security gate |
| `.agent/execplans/EP-007-testing-hardening.md` | L4 | Complete the content tree, property tests, the golden full-campaign tape, the twelve live-fire proofs, coverage |
| `.agent/execplans/EP-008-observability-and-operations.md` | L4 | Structured local logging, rotation, metrics, debug overlay, crash reports, runbooks, operational smoke |
| `.agent/execplans/EP-009-deployment-and-release.md` | L4 | Cross-target build, artifact staging, checksum manifest, cross-target determinism drill, rollback drill |
| `.agent/execplans/EP-010-production-readiness-and-ship.md` | L4 | Verify from scratch, reality and live fire, audits, reviews, ship gate, tag, MANUAL stop |
| `.agent/prompts/continue-execplan.md` | L4 | Resume an interrupted node, re-verifying the last checked milestone first |
| `.agent/prompts/debug-validation-failure.md` | L4 | The ladder operationalized for one failing command, with project-specific first questions |
| `.agent/prompts/execute-active-execplan.md` | L4 | Run one named node under the same laws |
| `.agent/prompts/final-review.md` | L4 | Full verify from scratch, expected-files audit, acceptance walk, documentation truth |
| `.agent/prompts/run-graph.md` | L4 | The master hands-off prompt: boot and dispatch until ALL_DONE or NODE_BLOCKED |
| `.agent/reality-allow` | L5 | Allowlist, seeded with a single never-matching line |
| `.agent/reality-patterns` | L5 | One ERE per line of forbidden implementation markers |
| `.agent/specs/SPEC-000-product-scope.md` | L2 | Scope, the twelve outcomes verbatim, act vocabulary, content non-goals as testable invariants |
| `.agent/specs/SPEC-001-core-domain.md` | L2 | mc_core contract: fixed point, RNG, world, tick order, ATB, statuses, spawns, budget, curriculum, poison, scenes, final encounter |
| `.agent/specs/SPEC-002-data-model.md` | L2 | Content layout, bake pipeline, schemas, save model, migration, validation rules, test-data lifecycle |
| `.agent/specs/SPEC-003-api-contracts.md` | L2 | Command bus, StateView, tick contract, tape format, replay contract, error contract |
| `.agent/specs/SPEC-004-ui-ux-behavior.md` | L2 | Render target, palettes, sprites, screens, advisory, input, the eight accessibility requirements, headless mode |
| `.agent/specs/SPEC-005-auth-and-permissions.md` | L2 | Security baseline where auth does not apply: confinement, parser hardening, fuzzing, no network, no unsafe |
| `.agent/specs/SPEC-006-error-handling.md` | L2 | Error taxonomy, rejections versus errors, user-facing presentation, the forced-failure matrix |
| `.agent/specs/SPEC-007-observability.md` | L2 | Logging, redaction, metrics, overlay, crash reports, and the explicit non-behaviours |
| `.agent/specs/SPEC-008-production-readiness.md` | L2 | The ship gate, evidence rules, quality floors, regression protection, what is deliberately not gated |
| `.agent/specs/SPEC-009-content-bestiary-and-regions.md` | L2 | Locked content vocabulary: 15 regions, closed family set, 102 bestiary entries, curriculum grants, poison table, campaign gating |
| `.agent/specs/SPEC-010-narrative-and-confidences.md` | L2 | What a Confidence is and is not, trust thresholds, mask, personas, key items, the Edouard invariant, the finale |
| `.agent/state/LEDGER.md` | L6 | Append-only run state, seeded with RUN_INIT. The only always-writable file |
| `.agent/templates/adr-template.md` | L4 | Context, decision, consequences, alternatives rejected, verification |
| `.agent/templates/execplan-template.md` | L4 | All fourteen sections and the nine-field milestone grammar |
| `.agent/templates/runbook-template.md` | L4 | Symptom, diagnosis, action, verification, escalation, prevention |
| `.agent/templates/spec-template.md` | L4 | Behaviour-first spec shape with locked vocabulary and a validation table |
| `.agent/templates/test-case-template.md` | L4 | Given, when, then, plus the mandatory why-this-is-not-a-mock clause |
| `.env.example` | L2 | Environment template: every table variable, commented, shaped dummy values |
| `.gitignore` | L2 | Ignores .env, build output, runtime roots; never ignores .agent or scripts |
| `.hermes/6layer.md` | L1 | Hermes adapter: PRIME BLOCK verbatim |
| `.openclaw/6layer.md` | L1 | OpenClaw adapter: PRIME BLOCK verbatim |
| `AGENTS.md` | L1 | Canonical control plane: mission, boot sequence, hierarchy, graph protocol, STOP list, anti-drift, anti-hallucination, anti-fixation, reality law, ship gate |
| `ARCHITECTURE.md` | L2 | System overview, crate layer DAG, the fourteen numbered invariants, flows, forbidden moves, review checklist |
| `ASSUMPTIONS.md` | L2 | Fourteen assumptions with reason, risk, verification command, and whether they block |
| `CLAUDE.md` | L1 | Claude Code adapter: PRIME BLOCK verbatim, zero volatile content |
| `COMMANDS.md` | L4 | The only legal command source: gates and sentinels, non-interactive block, parity check, forbidden commands |
| `CONTRIBUTING.md` | L4 | Setup, coding standards, comment-cites-invariant rule, commit format, PR and review checklists |
| `DECISIONS.md` | L2 | Decision table and fourteen ADRs covering every pre-decided fork |
| `DEPLOYMENT.md` | L2 | Artifact definition, release flow, exact steps, the MANUAL publish step, deployment STOP conditions |
| `ENVIRONMENT.md` | L2 | Exact tool and crate versions, environment variable reference, setup, parity, troubleshooting |
| `HOW-TO-USE.md` | L4 | Operator instructions: materialize, bootstrap, launch hands-off, observe, relay, block handling, ship |
| `OBSERVABILITY.md` | L2 | Local-only logging, metrics and budgets, debug overlay, crash reports, acceptance criteria |
| `OPERATIONS.md` | L2 | Health checks, failure modes with diagnostics, backup and restore, maintenance, operational safety rules |
| `PREFLIGHT.md` | L2 | Exhaustive external needs: toolchain, subcommands, three roots, registry mode, machine table. No credentials exist in this project |
| `PRODUCTION_READINESS.md` | L2 | The ship standard as a checklist with a verifying command or artifact per line |
| `PROJECT_BRIEF.md` | L2 | Problem, users, the twelve outcomes verbatim, goals, binding non-goals, success metrics |
| `RELEASE.md` | L2 | Release types, versioning, the mandatory changelog Determinism subsection, checklist, approvals |
| `ROADMAP.md` | L3 | Strategic narrative mirroring the GRAPH-TABLE one to one. Forbids direct implementation |
| `ROLLBACK.md` | L2 | Triggers, application and data rollback, verification, communication, postmortem, the real drill |
| `SECURITY.md` | L2 | Threat model, three trust boundaries, confinement, dependency policy, redaction, hardening checklist, STOP conditions |
| `TESTING.md` | L5 | Pyramid, test double zone, forced-failure matrix, determinism suite, coverage floors, validation matrix |
| `docs/6Layer-MasterPrompt-v2-GRAPHLOCK-MONTECRISTO.md` | L2 | The master prompt with Section 1 INPUTS filled for this project; provenance of the pack |
| `docs/GAME_DESIGN.md` | L2 | Normative content design: acts, combat, 102-entry bestiary by region, Confidences, final boss, fidelity ledger |
| `scripts/build.sh` | L5 | Cross-target release build, content bake, artifact staging, SHA256SUMS |
| `scripts/dependency-audit.sh` | L5 | cargo-deny advisories, bans, licences, sources |
| `scripts/format-check.sh` | L5 | rustfmt in check mode across the workspace |
| `scripts/graph-next.sh` | L5 | VERBATIM: deterministic scheduler over the GRAPH-TABLE and the ledger |
| `scripts/install.sh` | L5 | Fetch dependencies from the committed lockfile, offline when vendored |
| `scripts/ledger.sh` | L5 | VERBATIM: append-only event writer and derived status reader |
| `scripts/lint.sh` | L5 | Clippy at deny level, POSIX check over every shipped script, crate layer DAG enforcement |
| `scripts/live-fire.sh` | L5 | The twelve proofs LF-01 to LF-12 against the real entry point. Prints live-fire: ok |
| `scripts/preflight.sh` | L5 | FILLED: files, tools, exact version assertions, environment, probe table walk. Prints preflight: ok |
| `scripts/probes/artifact_dir.sh` | L5 | Read-only probe: artifact root exists and is writable |
| `scripts/probes/cargo_tools.sh` | L5 | Read-only probe: three cargo subcommands at exact versions |
| `scripts/probes/content_dir.sh` | L5 | Read-only probe: content root exists and is writable |
| `scripts/probes/data_dir.sh` | L5 | Read-only probe: data root exists and is writable |
| `scripts/probes/graphics_stack.sh` | L5 | Read-only probe: platform windowing and audio headers present |
| `scripts/probes/registry_mode.sh` | L5 | Read-only probe: registry reachable, or vendor tree populated |
| `scripts/probes/rust_toolchain.sh` | L5 | Read-only probe: pinned rustc and required components |
| `scripts/production-readiness-check.sh` | L5 | verify.sh plus every automatable production-readiness line |
| `scripts/reality-gate.sh` | L5 | FILLED: lexical no-mock gate over the five crate source trees and content |
| `scripts/security-check.sh` | L5 | The nine-item hardening checklist including mc_core purity and the secret scan |
| `scripts/smoke-test.sh` | L5 | Operational health checks plus the cold-start budget |
| `scripts/test-e2e.sh` | L5 | Tape replay through the real command bus and every committed tape hash |
| `scripts/test-integration.sh` | L5 | Real files on a real filesystem, plus a no-residue assertion |
| `scripts/test-unit.sh` | L5 | Library and property tests |
| `scripts/typecheck.sh` | L5 | Full workspace type resolution |
| `scripts/verify.sh` | L5 | Every gate in order. The single definition of green; CI invokes only this |

TOTAL FILES: 101
