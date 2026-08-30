# PR #64 (copy-effects design) — review findings

**Delete this file when the last row is closed.** Second-opinion review of
`plans/copy-effects-architecture.md` + `plans/references/copy-census.py`,
2026-08-30. Lives on #64's own branch so the fix session and the findings
travel together; it merges into `replacement/rb-pipeline` with the PR.

**The verdict was: sound.** Verified against `tmnt.txt` and the code, not
re-argued: the seam (CR 613.2c — copiable values are layer 1's *output*), the
snapshot rule and its CR 613.8 refutation (stress-tested: stacked copy rows
compose by timestamp, mutual-copy boards cannot recurse, independence holds),
the `CardData`-swap rejection, §5.1's RC-4 split, §5.4's sublayer inversion
(real at every cited site, plus a fourth: the code block at
`layers-architecture.md:64`), and §4.7's leg 2. The census reproduces exactly,
`--decompose` reconciles to the card (2,890 = 2,243+80+71+396+100; 120
commanders = 88+32), and the gates pass on the branch (`check_claude_md`
199/200, `specdb` clean, 746 tests, zero warnings). Close these rows, re-run
`check_claude_md`, and merge — into `replacement/rb-pipeline`, per the #63
pattern.

| # | Area | Finding | Verdict |
|---|---|---|---|
| V1 | §2.4, §5.3, §7; `cards-unlocked-ledger.md` Part 5 | **The CV-1/CV-1b boundary does not map onto the engine, and CV-1's named consumer contradicts the doc's own census.** Three strands, one rewrite. **(a)** §7 names Cytoshape as CV-1's `AffectedSet::SourceOnly` consumer, but Cytoshape is in the census's filter-scoped 13 (`--scope` prints it; §2.4's own example list includes it), which §5.3 defers to CV-1b. The ledger's CV-1 examples compound it: Vesuvan Doppelganger is Tier B (and its "becomes a copy" half is a *triggered* ability — critical-path item 6), Copy Artifact is Tier B, Shapesharer is in the filter 13. No valid CV-1 consumer is named anywhere; real candidates are **Dimir Doppelganger** or **Lazav, the Multifarious** (activated, self-scoped, no trigger). **(b)** The split is two-valued where the engine has three `AffectedSet`s. "Target creature becomes a copy" lowers to **`Fixed`** — CR 611.2c locks a resolution effect's affected set when it begins, and 611.2c appears nowhere in the doc despite deciding this. Tier B's 4 "filter-scoped" cards (Essence of the Wild et al.) are not row-filters at all: the filter lives on the `ReplacementDef` (Kalitas-shaped, discovered live off the effective list) and every row produced is scoped to the entering object — so those 4 are likely not item-10-blocked. And a `Fixed` row's item-10 exposure is exactly the same-turn pump-spell exposure §5.3 itself calls "not new and not worse here" — so the CV-1b block on item 10 is under-argued: either tolerate the exposure (Giant Growth already ships with it) or state the asymmetry that makes becoming-a-wrong-creature worse than +3/+3. **(c)** §5.3's teardown claim ("source *and* subject → `remove_by_source`, already implemented") silently assumes copy rows carry **the affected permanent as `source`**. RB's convention is `source = ctx.source` = the resolving stack object, which is never on the battlefield (`rb-review.md` H2) — under it, an activated self-copy with `Duration::Indefinite` (Dimir Doppelganger) leaves a row that outlives the permanent, an *unbounded* item-10 exposure. Registering copy rows with the affected permanent as source is probably right — but it is a deviation from the registry convention and must be stated, not assumed. | `design` |
| V2 | `codebase-state.md`, detector table | Two remnants of the withdrawn draft-§6 scoping survived into the corrections: "now item 5c, phases **CV-1–CV-5**" (the doc says CV-1–CV-7) and "[CR 729] **scoped out of v1** by the new doc" (§6/§10 withdrew exactly that — nothing in the cluster is out of v1; CR 729 is *scheduled and undesigned*, CV-7). | `doc` |
| V3 | §3.2 | `CopiableValues.mana_cost: ManaCost` must be **`Option<ManaCost>`** — `EffectiveCharacteristics` carries the `Option`, and §4.6's own headline case (capturing a face-down creature's CR 708.2a characteristics, which include *no mana cost*) is unrepresentable in the sketched type. Tokens without costs, same. | `fix` |
| V4 | `cards-unlocked-ledger.md` Part 5 | A blank line between the CV-5 and CV-6 rows splits the markdown table in two (the second renders headerless). | `fix` |
| V5 | §11 | The owed list misses one neighbor site: `cant-effects-architecture.md:1282` still says Layer 1 lands when "RC-4 fills the CR 616.1c bucket" — §5.1 moved that producer to CV-2. Same staleness class as `types/replacement.rs:316`, which §11 does list. | `doc` |
| V6 | §3.1 | CR 707.2c is cited for the general capture-at-first-application rule, but its text scopes itself to *static-ability-generated* copy effects. The resolution-side capture timing rests on CR 707.2's definition plus CR 611.2c's lock-in — fold the correct cites into V1's rewrite. | `doc` |

**After the rows close:** re-run `python plans/check_claude_md.py` (the file is
at 199/200 — V2's edits are in `codebase-state.md`, but check anyway), confirm
the census tables did not move, merge #64 into `replacement/rb-pipeline`, and
proceed to `plans/handoffs/rb-review.md` → "Start here" step 3.
