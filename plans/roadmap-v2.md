# Roadmap v2 — the route to v1

> Written 2026-08-31, the day the CR coverage audit merged (PR #70), to put on
> one page the question the audit made answerable: *what is the path from
> today's tree to an engine that can represent (nearly) all of Magic?*
> Supersedes `roadmap.md` (2026-04-01) except its **Deferred Items → Phase
> Mapping** tables, which stay citable as "roadmap D10" etc.

**What this document owns:** the route — why the spine is ordered the way it
is, what each segment unlocks measured in real cards, the enabling surfaces off
the spine, the milestone criteria, and starting-guess sizes.

**What it does not own.** The ordering itself is `CLAUDE.md` → "Critical path
to v1"; items are cited by that section's stable numbers. Current state is
`codebase-state.md`, the off-path inventory is `backlog.md` (cited by §2.x
entry), and type design lives in the architecture docs. **Where this file
disagrees with any of them, this file is wrong** — fix it here, don't act on it.

**Numbers rot.** Every count below is stamped 2026-08-31; the Scryfall queries
are in the appendix and the specdb numbers name their command. Re-run before
scheduling against one. This file sits in `specdb.py`'s `exclude_docs` beside
the audit doc: its rule citations are route pointers, not designs, and must not
satisfy `orphaned`'s ownership filter.

---

## 1. The destination

**v1 is two use cases** (owner, 2026-08-24): peer-to-peer **4-player Commander
through a GUI**, and **highly parallel AI games over the CLI**. Two-player
Standard is a checkpoint on the way, not a target.

Behind both sits one engine bar, made explicit here:

> **Representability: any black-border card can be authored as data** — an
> ability/primitive/cost composition plus a registry entry — **without an
> engine change.** Writing a card is a normal diff; only a new *mechanic*
> opens an engine phase.

"Nearly all" excludes a small process-breaking tail (§7): cards like Panglacial
Wurm or Shahrazad, which don't lack a missing *surface* but demand a bespoke
violation of the engine's process model. Everything else — ~32,100 black-border
cards — is in scope, gated only by the segments below.

---

## 2. Where the route stands — snapshot 2026-08-31

Point-in-time. `codebase-state.md` wins on state; regenerate the queries before
trusting a number.

- **Spine:** items 1–4 done (layers core, CDAs, Layer 6, Layer 2). Item 5 is
  two phases in (RA, RB) of five (RA–RE). Items 5b, 5c, 6, 7 unstarted.
- **Cards:** 56 registered (`cards/registry.rs`) — deliberately near zero.
  Breadth before surfaces is how the six historical gaps got expensive; the
  bet of the whole ordering is that cards are cheap *after* the surfaces exist.
- **Tests:** 753 `#[test]` functions.
- **Spec corpus** (`python plans/specdb.py stats`): 1,753 atoms —

  | Phase | Atoms | What it is |
  |---|---:|---|
  | Phase 8 | 643 | card breadth: keywords, primitives, per-card behavior |
  | Phase 9 | 230 | formats, Commander, multiplayer |
  | ALREADY-IMPL | 198 | behavior that predates the corpus |
  | Phase 5-Pre | 192 | data model, casting, combat, SBAs |
  | Phase 5-Layers | 151 | continuous effects |
  | Phase 7 | 133 | triggered abilities |
  | Phase 6 | 124 | replacement effects |
  | Backlog | 65 | re-filed to `backlog.md` entries |
  | other | 17 | 14 unclassified, 3 post-v1 |

  The 3.2% FULL-test figure measures the *annotation* discipline (weeks old)
  plus the fact that half the corpus is deliberately future material — not
  engine correctness; the triage confirmed ~159 atoms of
  implemented-but-untested behavior besides (`backlog.md` §5). The gate that
  matters, `owed`, is clean at 9 deliberate keeps.
- **What the audit did for the route** (`cr-coverage-audit.md`): after sweeping
  every fact-bearing type — seventeen in its §4, the last three (`ManaPool`,
  `PlayerState`, `Zone`) added from card-population probes, with the
  enumeration criterion now stated there (2026-08-31) — **exactly one fact had escaped the plan**
  (cost-payment provenance, `codebase-state.md` Deferred Migrations item 30,
  back-stopped at CV), and **one v1 blocker sits off-spine** (the information
  model, `backlog.md` §2.9 → §5 below). Everything else found is additive.
  **The route below therefore carries no "…plus whatever we forgot" term.**
  That is what the audit bought; it implemented nothing.

---

## 3. The spine, and why it is ordered this way

`CLAUDE.md` owns the numbering; this section owns the *because*. The chain is
causal, not preferential — each segment produces the substrate the next one
observes.

**Items 1–4 — layers (done).** Every downstream system asks "what is this
object?", and the stored answer is wrong whenever a continuous effect exists.
Built first because it was already built *late*: 21 call sites had to be
migrated to the oracle layer, which is the project's canonical price for
building on top of an inexpressible fact. The rest of the ordering discipline
descends from that bill.

**Item 5 — replacement effects (RC → RD → RE), before triggers.** CR 614
rewrites an event *before it happens*; CR 603 observes events that *did*
happen. Trigger detection was resolved (2026-08-24) to be the performed-action
event stream — so that stream must already be post-replacement truth, or every
trigger observes fiction. **RC — ETB replacements — is next, and is the
largest single unlock before triggers: ~1,350 cards** (enters tapped, enters
with counters, "as ~ enters, choose…"). For scale, `enters` appears in the
text of 7,071 of 32,115 black-border cards; RC is the replacement half of that
text, item 6 the trigger half.

**Item 5b — "can't" effects (RS-1–RS-4), beside item 5, not after.** CR 101.2
makes a "can't" beat the permission it collides with, and CR 614.17 keeps it
out of the replacement pipeline — so it is checked *ahead* of the pipeline
(`is_blocked`), and the pipeline's own growth depends on it: **RS-1 must land
before RC-4**, and RS-3 (combat) wants item 7 first.
→ `cant-effects-architecture.md`.

**Item 5c — copy effects (CV-1–CV-7), beside 5 and 5b.** A copy row stores
*values*, never a reference — the decision that keeps copies independent of
item 7's dependency algorithm and lets them run beside the replacement work
instead of behind it. CV-2 needs RC-2 (a copy that becomes a permanent enters
through the ETB pipeline). **Item 30's design constraint lands here:** CR
707.10 splits cost-payment provenance in half — a spell copy inherits the paid
*objects*, never the paid *mana* — so the capture must be designed before the
spell-copy phase, and recording it is cheap any time before that. CV-5 makes
the second card face (CR 712) and `CardData.color_indicator` live.
→ `copy-effects-architecture.md`; `codebase-state.md` item 30.

**Item 6 — triggered abilities, with LKI formalization and conditional
statics.** The largest unlock in the game, and it is not close: **14,603 of
32,115 black-border cards — 45% — carry a triggered ability.** Phase 7 holds
133 atoms. LKI rides along (the CR 603.10a frame has been captured at the
chokepoint since RA; item 6 formalizes its consumers), as do conditional
static abilities. **It is also the least-sized item on the path — size it in
the doc before the first PR.** The RB lesson (+5,475 because nobody counted)
at five times the stakes. → `engineering-practices.md` §4.

**Item 7 — the CR 613.8 cluster, after 6, back-stopped before Phase 8.** The
dependency algorithm changes how the layer pass itself runs, so it wants to
land after the systems that mass-produce effect sources (triggers) and
**before card breadth multiplies the ordering-sensitive boards.** The
back-stop is load-bearing and currently enforceable precisely because breadth
has not started: until it lands, no dependency-ordering-sensitive card may be
authored.

**The Commander interleave — after item 5.** Cost modification first
(`backlog.md` §2.1, two phases — the commander tax is the forcing function),
then `GameConfig::commander()`, CR 903.7, CR 800/802. CR 903.9a/b already
work and sit dormant until something sets `GameObject.is_commander`. This is
also when the 25 CR 601 casting-procedure atoms left unjudged by the triage
get their verdicts (§7).

---

## 4. The lattice — enabling surfaces off the spine

Thirteen inventory entries (`backlog.md` §2) plus two ledger tickets. **Every
one is feature-shaped** — the audit found no retrofit cost in any of them — so
their order is free: pull each when its card family is wanted. What is not
free is the internal arrows.

| Surface | Unlocks | Wants first |
|---|---|---|
| §2.8 functioning zones + activation restrictions | flash-as-ability, "activate only as a sorcery"; the surface §2.3's keywords are written in | — |
| §2.3 cast-from-elsewhere gate | flashback, escape, foretell, plot, … ≈ 764 cards | §2.8, §2.1 |
| §2.11 loyalty abilities | every planeswalker — 335 cards | §2.8 |
| §2.1 cost pipeline (representation, then modification) | hybrid/Phyrexian (670 cards), kicker family (257), alternative costs; ≈ 903 cards | — (leads the Commander interleave) |
| §2.2 linked abilities | the O-Ring pattern, "the chosen [value]" cards, kicker's second half | — |
| §2.7 modal spells | 778 cards; consumes the dead `chosen_modes` field | near §2.1 |
| §2.10 color derivation | devotion, protection-from-color, color-matters | pairs with §2.1 |
| §2.5 remaining `Primitive` arms | manifest, cloak, exert, unattach; the face-down subsystem (shared with foretell) | §2.9 for face-down |
| §2.12 step-scoped durations | only 10 cards say "until end of combat" — the weight is the engine hooks (CR 511.3, CR 703.4q), a caution against sizing by card count | — |
| T12c/T12d restricted-mana wiring | the 227 `o:"this mana"` cards; Omnath-class blanket persistence | grants want item 30 and item 6 |
| §2.4 `DecisionProvider` choice shapes | the voting cycle; name-a-card | one trait decision, taken once |
| §2.13 deck-limit validation | a deckbuilding UI that refuses malformed lists | trivial |
| §2.15 player-scoped continuous effects | Winter's hand-size clause; the Reliquary Tower class (62 touch it, 43 remove it); player hexproof (14) | conditional gating: item 6 |
| §2.16 counters on players | energy (145 cards), experience (16); proliferate's player half | — |
| §2.17 extra turns | Time Walk's family, ~60 cards | skips ride the replacement track |
| **§2.9 the information model** | **v1 itself** — see §5 | back-stop before Phase 8's face-down/reveal cards |

---

## 5. Two route amendments this document proposes

**1. Item 30's capture side lands before CV starts.** Recording *what paid* —
the mana atoms, the spent objects — rides the `x_value` rail, is independent
of every scheduled phase, and the information exists only at payment time. One
small standalone PR; the design half (the CR 707.10 split) stays CV's, per
`codebase-state.md` item 30.

**2. The information model (§2.9) gets a scheduled slot with a hard back-stop:
before Phase 8's face-down and reveal cards, and before any Phase 10 GUI/AI
work.** Nothing in the tree answers "can player N see this?"; both v1 use
cases need the answer, and every reveal/look-at card authored against an
omniscient `GameState` bakes "reveal is a no-op" in silently — the layers
retrofit shape over again. The retired corpus tickets already sited the
answer: a per-viewer query beside `oracle/characteristics.rs`. It depends on
nothing in items 5–7 and can be built in parallel whenever.

**Status: proposed.** `CLAUDE.md`'s critical path owns ordering; it takes
these amendments, or doesn't, by the owner's call.

---

## 6. Milestones

Absorbed from `roadmap.md` (2026-04-01) and updated. The ladder is unchanged:
**core-rules-complete** after items 5–7 plus Phase 8, **format-ready** after
Phase 9, **user-ready** after Phase 10.

**Milestone 8 — Commander Playable** (trigger: Phase 9 Commander support
complete): a 4-player Commander game runs to completion; command zone,
commander tax, commander damage, and color identity all enforced; the
`Format` trait dispatches Commander vs Standard correctly. *(Criteria
unchanged; the tax is why §2.1 leads the interleave.)*

**Phase 10 — the v1 wrap:** a Web GUI (Wasm; the target is the middle ground
between XMage's function and Arena's polish), an AI API over the same
4-method `DecisionProvider` that drives CLI/Random/Scripted play
(`ChoiceContext` is serde-serializable by design), parallel fuzz, and
profile-driven performance work. **§2.9 is a prerequisite for both halves** —
the GUI renders only what one player may see, and an AI observation must not
leak a hidden zone. Network play is a stretch goal; `DraftEngine` for Limited
is post-v1.

---

## 7. The excluded tail

Cards that break the **process model** rather than lacking a surface:

- **Reentrant casting** — Panglacial Wurm: cast mid-search, inside another
  action's resolution.
- **Subgames** — Shahrazad.
- **Dexterity and physical arrangement** — Chaos Orb, Falling Star,
  Camouflage, Raging River.
- **Genuinely ill-defined text** — Season of the Witch's "could have
  attacked".
- **Ante** (banned everywhere), **conspiracies**, **attractions/stickers**
  (declared out of scope; audit §5.2 keeps CR 717.6's identity-key constraint
  on the books anyway).

Well under 100 cards of ~32,000 — and, the structural point: **none of them
can silently poison a surface** the way the six historical gaps did. Each
would need a deliberate, bespoke process hack, which is a decision made
explicitly or never. The corpus fences them with a `Post-v1` phase (3 atoms
today).

**One boundary is still undrawn, on purpose:** the 25 CR 601
casting-procedure atoms the triage left without verdicts (modal announcement,
the CR 601.2g mana window, CR 601.4 look-ahead, …). Their judgment comes with
the Commander interleave's cost work; that is the moment the Wurm-class line
gets drawn formally rather than by default. → `backlog.md` §4.

---

## 8. Sizing — starting guesses only

In the project's 1,500–2,500-addition PR band. **Re-derive from a live count
before scheduling any row** (`engineering-practices.md` §4): two estimates in
the audit workstream ran low, and RB ran to +5,475 because nobody counted.

| Segment | Phases | PRs (guess) |
|---|---|---:|
| Item 5 remainder | RC, RD, RE | 3–5 |
| Item 5b | RS-1–RS-4 | ~4 |
| Item 5c | CV-1–CV-7 | ~7 |
| Item 6 | triggers + LKI + conditional statics | 4–6, unsized — size first |
| Item 7 | CR 613.8 cluster | 1–2 |
| Commander interleave | §2.1 ×2, `commander()`, CR 903.7, CR 800/802 | ~4 |
| The lattice | §4's table | 10–13 |
| **To "breadth unconstrained"** | | **~35–40** |

After that, Phase 8's 643 atoms are throughput, not architecture — every card
a normal diff — and the bottleneck moves to card-authoring speed, which is
when the deferred Scryfall import pipeline earns its slot.

---

## 9. Watch items

- **Item 6's true size** — the dominant risk on the route. Unsized.
- **§5's two amendments** — advisory until `CLAUDE.md` adopts them.
- **The `DecisionProvider` trait shape** — §2.4, §2.7's opponent-chooses-a-mode
  and CR 201.4 converge on one decision; take it once, at the first new
  method, because Phase 10 serializes the trait.
- **The 25 CR 601 atoms** — pending; triaged at the Commander interleave.
- **D2b** — ~159 regression tests for behavior that already works; interleave
  in touched-area subsets beside other work, never as a block.
  → `backlog.md` §5.
- **Corpus debt** — CR 702 keyword-depth atoms (authored beside the Phase 8
  work that needs them) and audit §7's two open items.
- **Bookkeeping** — `backlog.md` §2.14's two atoms stay in `owed`'s nine until
  the next §3.3 pass re-files them; the entry already captures their tickets.

---

## 10. Maintaining this file

Update it when a spine segment lands or the route changes; keep every number
dated and query-sourced. **No state claims** (`codebase-state.md` wins), **no
designs** (architecture docs), **no inventory** (`backlog.md`), **no ordering
authority** (`CLAUDE.md`). If an entry here grows past a pointer and a why, it
is trying to become one of those files — move it there.

---

## Appendix — the card-count queries (Scryfall, 2026-08-31)

All with `-is:funny unique:cards` appended; fetch per
`plans/references/` (curl with a UA header — `WebFetch` gets 403'd).

| Count | Query |
|---:|---|
| 32,115 | `game:paper` |
| 14,603 | `(o:"when " or o:"whenever " or o:"at the beginning of")` |
| 7,071 | `o:"enters"` |
| 778 | `(o:"choose one" or o:"choose two" or o:"choose one or both")` |
| 670 | `(is:hybrid or is:phyrexian)` |
| 335 | `t:planeswalker` |
| 257 | `(keyword:kicker or keyword:multikicker)` |
| 227 | `o:"this mana"` |
| 145 | `o:"{E}"` |
| 62 / 43 | `o:"maximum hand size"` / `o:"no maximum hand size"` |
| 60 | `o:"extra turn"` |
| 16 | `o:"experience counter"` |
| 14 | `(o:"you have hexproof" or o:"you have shroud" or o:"players have hexproof")` |
| 10 | `o:"until end of combat"` |

The ~1,350 (RC), ~903 (§2.1) and ~764 (§2.3) figures are the detector
write-up's, quoted from `codebase-state.md`'s 2026-08-31 update.
