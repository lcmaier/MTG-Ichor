# Authoring trail — not a source

These summaries were produced alongside the corpus in 2026-04 and are frozen at
2026-08-19. **Nothing reads them.**

Until 2026-08-24 they generated `global-test-index.md` and `phase-index-*.md` via
`extract-phase-index.py`, while `specdb.py` built the database from `sessions/`.
Two parsers over two tiers: corrections landed in the sessions, these stayed put,
and the two ended up disagreeing by 27 entries and on the size of three phases
(Phase 7: 202 against 133). The indexes now come from the same parse that builds
the database, and the extraction script is archived.

The authored source of truth is `../sessions/*.md`. Corrections go there.
