# The work order after TR-2a (decided 2026-09-24 and 2026-09-25)

**Where to resume.** The owner set this order across #186's review. It lives
here until the process PR folds it into the documents that own it:
- the ordering into `CLAUDE.md`'s critical path;
- the route into `roadmap-v2.md` §3a;
- the splits and re-counts into `triggers-architecture.md` §12.

That PR then deletes this file.

## The order

1. **The process PR** (this branch, `process/close-out-and-replan`):
   - the close-out as one checklist, and `plans/close_out.py`;
   - the redundant-clippy-allow gate;
   - the re-plan and the AI floors, recorded in their owning documents.
2. **The bounded-state PR:**
   - item 42's event log leaves `GameState`, and trigger bindings copy their bound facts at dispatch;
   - TR-2a's whole-game history is bounded;
   - the clone probe is committed, with CI checks on allocations and bytes per clone;
   - a throwaway probe measures the observation-encoding cost k.
3. **TR-2b:** "may", CR 118.12's answer, and the `departed` frames. Its first commit folds the "your" conditions into one variant with `whose: PlayerSet`.
4. **CV-2.**
5. **CV-1b with item 10 (CR 400.7).** TR-3's returns need it, and the owner chose the CV order over pulling item 10 forward.
6. **TR-3a:** the delayed-trigger registry, with Final Fortune.
7. **TR-3b:** reflexive triggers, returns and "until", with Flickerwisp, Cornered Crook and Banishing Light.
8. **Items 176/177:** the zone-change record, designed and reviewed, as its own PR.
9. **TR-4a:** the frame.
10. **TR-4b:** the records.
11. **TR-5a:** combat, targeting, ability damage and excess damage. It needs only TR-2b, so it may move up.
12. **TR-5b:** counters, prevention, the trigger multiplier, tokens and scry.
13. **TR-6:** state triggers, and the loop detector counting decisions (two or more legal answers), not prompts.
14. **TR-7:** the Krark-Clan Ironworks loop, the track's showcase.
15. **Item 6's close audit.** The readiness pass reads the AI floors: loaded decisions per core-second, and callgrind instructions per decision.
16. **The information-model design:** backlog §2.9 merged with §2.34, reviewed before any build and ahead of Phase 8. The build keeps its back-stop.
17. **Phase 8 breadth**, opening with C0: the cards re-filed by set, `cards::helpers`, and the duration helper. **Then Phase 9, then Phase 10.**

The other interleaved items keep their slots in `CLAUDE.md`'s critical path, for example the mana-provenance design before Phase 8.

## Decisions that ride with it

- **The AI floors are adopted** as the backstop under item 138's ratchet.
  - At least 10,000 decisions per loaded physical core-second at Commander scale.
  - A full-state clone of at most 10 µs.
  - At most 128 KB per state, bounded by the board's high-water mark.

  Floors 2 and 3 need the bounded-state PR. The derivation is the owner's report, "MTG engine AI performance floors" (2026-09-25).
- **The history bound** goes in the bounded-state PR, not in #186 (merged as e0e5ed0).
- **Review-round commit subjects** start with `review - `, or `review round N - `.
