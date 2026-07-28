# Checklist: production readiness

This mirrors PRODUCTION_READINESS.md. Open that file and work it line by line; every AUTO
line is executed by `sh scripts/production-readiness-check.sh` and every DOC line is verified
by opening the named file.

- [ ] Functional block complete (LF-01 through LF-12, content invariants, no critical bugs)
- [ ] Testing block complete (fresh verify, coverage floors, tape hashes, zero ignored,
      forced-failure suite)
- [ ] Reality block complete (reality gate, live fire, no double leakage, no demo mode)
- [ ] Security block complete (no secrets, redaction, audit, licences, no unsafe, no socket,
      confinement, fuzz corpora)
- [ ] Performance block complete (step p99, frame p99, cold start, replay time, memory
      ceiling, reference machine recorded)
- [ ] Accessibility block complete (remap, glyph parity, motion zero, Wait mode, font,
      captions)
- [ ] Privacy block complete (no collection, documented files and uninstall, advisory screen)
- [ ] Operations block complete (backup and restore verified, save migration, rollback drill,
      runbooks)
- [ ] Release block complete (three artifacts, SHA256SUMS, per-target determinism, licences
      file, tag, changelog, MANUAL step printed not executed)
- [ ] `sh scripts/production-readiness-check.sh` printed `production readiness: ok`
