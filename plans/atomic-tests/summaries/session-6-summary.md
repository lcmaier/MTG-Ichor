# Session 6 — Condensed Summary (CR 609–616)

> **Scope:** Effects, One-Shot Effects, Continuous Effects, Text-Changing Effects, Layer System (613), Replacement Effects (614), Prevention Effects (615), Interaction of Replacement/Prevention (616)
> **Full session:** `session-6.md` | **Design discussions:** `session-6-audit-response.md`
> **Date:** 2026-04-06

---

## ATOM Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-609.2-001 | 609.2 | Effects default to permanents only | Phase 5 (L10) | L10 | |
| ATOM-609.2-002 | 609.2 | Effects applying to non-permanent zones (Thalia cost increase) | Phase 5 (L15) | L15 | |
| ATOM-609.3-001 | 609.3 | Impossible partial effect: discard more than hand size | Phase 8 | NEW-609.3 | |
| ATOM-609.3-002 | 609.3 | Impossible partial effect: mill with insufficient library | Phase 8 | NEW-609.3 | |
| ATOM-609.4-001 | 609.4 | "As though flash" scoped to stated effect only | Phase 8 | NEW-609.4 | |
| ATOM-609.4a-001 | 609.4a | Two "as though" effects combine | Phase 8 | NEW-609.4 | |
| ATOM-609.4b-001 | 609.4b | "Mana of any type" doesn't change actual mana identity | Phase 8 | NEW-609.4 | |
| ATOM-609.4b-002 | 609.4b | "Any color" vs "any type" distinction ({C} can't pay colored) | Phase 8 | NEW-609.4 | |
| ATOM-609.4b-003 | 609.4b | Mana restrictions persist through "as though" spending | Phase 8 | NEW-609.4 | |
| ATOM-609.7a-001 | 609.7a | All valid damage source categories selectable | Phase 6 | NEW-609.7a | |
| ATOM-609.7b-001 | 609.7b | Prevention shield rechecks source properties | Phase 6 | NEW-609.7b | |
| ATOM-609.7c-001 | 609.7c | Static prevention applies to non-battlefield sources | Phase 6 | NEW-609.7c | |
| ATOM-610.3-001 | 610.3 | "Until leaves" zone-change return effect (O-Ring) | Phase 7 | NEW-610.3 | |
| ATOM-610.3a-001 | 610.3a | "Until" event already occurred before spell resolves | Phase 7 | NEW-610.3 | |
| ATOM-610.3b-001 | 610.3b | "Until" event occurred before triggered ability resolves | Phase 7 | NEW-610.3 | |
| ATOM-610.3c-001 | 610.3c | "Until" return goes to owner's control | Phase 7 | NEW-610.3 | |
| ATOM-610.3c-002 | 610.3c | "Until" return under specific controller override | Phase 7 | NEW-610.3 | |
| ATOM-610.3d-001 | 610.3d | Simultaneous "until" returns are simultaneous | Phase 7 | NEW-610.3 | |
| ATOM-610.5-001 | 610.5 | Static ability grants Convoke at cast time | Phase 7 | NEW-610.5 | |
| ATOM-610.5-002 | 610.5 | Grant persists after granting source destroyed | Phase 7 | NEW-610.5 | |
| ATOM-611.2a-001 | 611.2a | Spell continuous effect with stated duration expires | Phase 5 (L07) | L07 | |
| ATOM-611.2a-002 | 611.2a | No stated duration → lasts indefinitely | Phase 5 (L02) | L02 | |
| ATOM-611.2b-001 | 611.2b | "For as long as" duration never starts → no effect | Phase 8 | NEW-611.2b | |
| ATOM-611.2c-001 | 611.2c | Characteristic-modifying effect locks in affected set | Phase 5 (L07) | L07 | |
| ATOM-611.2c-002 | 611.2c | Game-rule-modifying effect does NOT lock in | Phase 6 | NEW-611.2c-mix | |
| ATOM-611.2c-003 | 611.2c | Mixed effect: char-mod locks in, rule-mod dynamic | Phase 5+6 | NEW-611.2c-mix | |
| ATOM-611.2d-001 | 611.2d | Variable X locked at resolution | Phase 5 (L07) | L07 | |
| ATOM-611.2e-001 | 611.2e | "Is [characteristic]" applies simultaneously with entering | Phase 7+5 | NEW-611.2e | |
| ATOM-611.3a-001 | 611.3a | Static ability effect applies dynamically | Phase 5 (L08) | L08 | |
| ATOM-611.3b-001 | 611.3b | Static effect ceases when source leaves battlefield | Phase 5 (L08) | L08 | |
| ATOM-611.3c-001 | 611.3c | Static effect applies as permanent enters, not after | Phase 5 (L08) | L08 | |
| ATOM-611.3d-001 | 611.3d | Static grant persists after source leaves | Phase 7+5 | NEW-611.3d | |
| ATOM-611.3d-002 | 611.3d | Dream Devourer foretell grant persists | Phase 7+5 | NEW-611.3d | |
| ATOM-612.2-001 | 612.2 | Text change respects word context (color word) | Phase 5 (L12) | L12 | |
| ATOM-612.2a-001 | 612.2a | Text change affects creature-type-derived token names | Phase 5 (L12)+8 | L12 | |
| ATOM-612.3-001 | 612.3 | Granted abilities immune to text change on object | Phase 5 (L12) | L12 | |
| ATOM-612.8-001 | 612.8 | Effect that sets name replaces all existing names | Phase 5 (L12) | L12 | |
| ATOM-613.1-001 | 613.1 | Layer system applies effects in order 1–7 | Phase 5 (L04, L10) | L04, L10 | |
| ATOM-613.1a-001 | 613.1a | Layer 1 copy before P/T modification | Phase 6 | NEW-613.1a | |
| ATOM-613.1b-001 | 613.1b | Layer 2 control change before type/color/ability/PT | Phase 5 (L11) | L11 | |
| ATOM-613.1c-001 | 613.1c | Layer 3 text change before type/color/ability/PT | Phase 5 (L12) | L12 | |
| ATOM-613.1d-001 | 613.1d | Layer 4 type change before color/ability/PT (Opalescence) | Phase 5 (L10, L19) | L10, L19 | |
| ATOM-613.1e-001 | 613.1e | Layer 5 color change before ability/PT | Phase 5 (L10) | L10 | |
| ATOM-613.1f-001 | 613.1f | Layer 6 ability changes before P/T (Humility + Tarmogoyf) | Phase 5 (L09, L19) | L09, L19 | |
| ATOM-613.1f-002 | 613.1f | Keyword counters grant abilities in L6 | Phase 5 (L09) | NEW-613.1f-kw | |
| ATOM-613.1g-001 | 613.1g | Layer 7 P/T is final (counter + Growth additive) | Phase 5 (L06, L07) | L06, L07 | |
| ATOM-613.2-001 | 613.2 | Layer 1 sublayer ordering: 1a before 1b | Phase 8 | NEW-613.2 | |
| ATOM-613.2a-001 | 613.2a | Layer 1a copiable effects establish base chars | Phase 6 | NEW-613.2a | |
| ATOM-613.2c-001 | 613.2c | Post-Layer 1 chars are copiable values (clone of clone) | Phase 6 | NEW-613.2c | |
| ATOM-613.3-001 | 613.3 | CDAs apply before non-CDAs in L2–6 (L5 color test) | Phase 5 (L04, L10) | L04 | |
| ATOM-613.4a-001 | 613.4a | L7a CDAs set P/T before other effects (Tarmogoyf) | Phase 5 (L04, L17) | L04, L17 | |
| ATOM-613.4b-001 | 613.4b | L7b P/T setting overrides L7a CDA | Phase 5 (L04, L07) | L04, L07 | |
| ATOM-613.4b-002 | 613.4b | L7b with subsequent L7c modifiers | Phase 5 (L04, L06, L07) | L04, L06, L07 | |
| ATOM-613.4c-001 | 613.4c | L7c counters and effects stack additively | Phase 5 (L06, L07, L08) | L06, L07, L08 | |
| ATOM-613.4d-001 | 613.4d | Basic P/T switch | Phase 5 (L04) | L04 | |
| ATOM-613.4d-002 | 613.4d | Switch with subsequent modifier removal | Phase 5 (L04) | L04 | |
| ATOM-613.4d-003 | 613.4d | Double switch cancels out | Phase 5 (L04) | L04 | |
| ATOM-613.4d-004 | 613.4d | Modifier after switch applies unswitched then re-switches | Phase 5 (L04) | L04 | |
| ATOM-613.5-001 | 613.5 | Color change in L5 immediately triggers L7 re-evaluation | Phase 5 (L08, L10) | L08, L10 | |
| ATOM-613.5-002 | 613.5 | Complex multi-layer interaction (Gray Ogre CR example) | Phase 5 (L04, L06, L07, L08) | L04, L06, L07, L08 | |
| ATOM-613.6-001 | 613.6 | Multi-layer effect applies to same locked set | Phase 5 (L05) | L05 | |
| ATOM-613.6-002 | 613.6 | Act of Treason: control in L2, haste in L6 | Phase 5 (L09, L11) | L09, L11 | |
| ATOM-613.6-003 | 613.6 | Multi-layer effect persists after generating ability removed | Phase 5 (L05) | L05 | |
| ATOM-613.7-001 | 613.7 | Timestamp ordering via conflicting L7b set-PT effects | Phase 5 (L03) | L03 | |
| ATOM-613.7a-001 | 613.7a | Static ability uses later of object vs granting effect timestamp | Phase 5 | NEW-613.7a | |
| ATOM-613.7b-001 | 613.7b | Spell effect timestamp set at creation | Phase 5 (L07) | L07 | |
| ATOM-613.7c-001 | 613.7c | Counter timestamp updates when new counter of same kind added | Phase 5 | NEW-613.7c | |
| ATOM-613.7d-001 | 613.7d | Object timestamp set on zone entry | Phase 5 (L03) | L03 | |
| ATOM-613.7e-001 | 613.7e | Equipment re-timestamp on attach | Phase 5 | NEW-613.7e | |
| ATOM-613.7m-001 | 613.7m | APNAP ordering for simultaneous timestamps | Phase 5 (L03) | L03 | |
| ATOM-613.7n-001 | 613.7n | Static ability timestamp < resolving effect when simultaneous | Phase 5 (L08) | L08 | |
| ATOM-613.8-001 | 613.8 | Dependency overrides timestamp order | Phase 5 (L14) | L14 | |
| ATOM-613.8a-001 | 613.8a | Blood Moon + Urborg dependency analysis | Phase 5 (L14, L17) | L14, L17 | |
| ATOM-613.8a-002 | 613.8a | Dependency condition (b): existence change | Phase 5 (L14) | L14 | |
| ATOM-613.8a-003 | 613.8a | CDA guard: CDA + non-CDA always independent | Phase 5 (L14) | L14 | |
| ATOM-613.8b-001 | 613.8b | Circular dependency falls back to timestamp | Phase 5 (L14) | L14 | |
| ATOM-613.8c-001 | 613.8c | Dependency re-evaluation after each application | Phase 5 (L14) | L14 | |
| ATOM-613.9-001 | 613.9 | Conflicting ability effects: later timestamp wins | Phase 5 (L09) | L09 | |
| ATOM-613.9-002 | 613.9 | Color change causes downstream effect to apply | Phase 5 (L08, L10) | L08, L10 | |
| ATOM-613.10-001 | 613.10 | Player-affecting continuous effects applied after object chars | Phase 5 (L15) | L15 | |
| ATOM-613.11-001 | 613.11 | Game-rule-modifying effects apply after L1–L7 | Phase 5 (L15) | L15 | |
| ATOM-613.11-002 | 613.11 | Cost modification: increases before reductions before Trinisphere | Phase 5 (L15) | L15 | |
| ATOM-614.4-001 | 614.4 | Replacement must exist before the event | Phase 6 | NEW-614.4 | |
| ATOM-614.5-001 | 614.5 | Replacement doesn't self-repeat (two doublers = 4x) | Phase 6 | NEW-614.5 | |
| ATOM-614.6-001 | 614.6 | Replaced event never happens; modified event triggers | Phase 6+7 | NEW-614.6 | |
| ATOM-614.7-001 | 614.7 | Replacement of non-event is a no-op | Phase 6 | NEW-614.7 | |
| ATOM-614.7a-001 | 614.7a | Zero damage is non-event; "+1" doesn't upgrade 0 | Phase 6 | NEW-614.7 | ALREADY-IMPLEMENTED (partial) |
| ATOM-614.8-001 | 614.8 | Regeneration replaces destruction | Phase 6 | NEW-614.8 | |
| ATOM-614.8-002 | 614.8 | Regeneration: damage triggers still fire | Phase 6+7 | NEW-614.8 | |
| ATOM-614.9-001 | 614.9 | Damage redirection: destination gone → no-op | Phase 6 | NEW-614.9 | |
| ATOM-614.10-001 | 614.10 | Skip replaces step/phase with nothing | Phase 6 | NEW-614.10 | |
| ATOM-614.10-002 | 614.10 | Skip mid-step doesn't end current step | Phase 6 | NEW-614.10 | |
| ATOM-614.10a-001 | 614.10a | Two skip effects consume two occurrences | Phase 6 | NEW-614.10 | |
| ATOM-614.10b-001 | 614.10b | Skip with follow-up defers to next real occurrence | Phase 6 | NEW-614.10 | |
| ATOM-614.11-001 | 614.11 | Draw replacement applies even with empty library | Phase 6 | NEW-614.11 | |
| ATOM-614.11-002 | 614.11 | Lab Maniac: draw replacement win condition | Phase 6 | NEW-614.11 | |
| ATOM-614.11a-001 | 614.11a | Draw replacement completes before sequence resumes | Phase 6 | NEW-614.11 | |
| ATOM-614.11b-001 | 614.11b | Additional action on drawn card lost if draw replaced | Phase 6 | NEW-614.11 | |
| ATOM-614.12-001 | 614.12 | ETB replacement uses look-ahead chars (Scarwood + Jailer) | Phase 6 | NEW-614.12 | |
| ATOM-614.12-002 | 614.12 | ETB look-ahead: own static abilities apply (Voice of All) | Phase 6 | NEW-614.12 | |
| ATOM-614.12-003 | 614.12 | ETB look-ahead: self doesn't affect self (Orb of Dreams) | Phase 6 | NEW-614.12 | |
| ATOM-614.12a-001 | 614.12a | ETB replacement choices made before entering | Phase 6 | NEW-614.12 | |
| ATOM-614.13-001 | 614.13 | ETB replacement causes other zone changes (Devour) | Phase 6 | NEW-614.13 | |
| ATOM-614.13a-001 | 614.13a | Can't choose entering/simultaneous objects (Sutured Ghoul) | Phase 6 | NEW-614.13 | |
| ATOM-614.13b-001 | 614.13b | Object can't be chosen for multiple ETB replacements | Phase 6 | NEW-614.13 | |
| ATOM-614.15-001 | 614.15 | Self-replacement applies before other replacements | Phase 6 | NEW-614.15 | |
| ATOM-614.15-002 | 614.15 | Self-replacement real card: Aang's Journey kicked search | Phase 6 | NEW-614.15 | |
| ATOM-614.16-001 | 614.16 | Token replacement applies to tokens from other replacements | Phase 6 | NEW-614.16 | |
| ATOM-614.17-001 | 614.17 | "Can't" overrides prevention | Phase 6 | NEW-614.17 | META-101.2 |
| ATOM-614.17a-001 | 614.17a | "Can't" must pre-exist the event | Phase 6 | NEW-614.17 | |
| ATOM-614.17b-001 | 614.17b | Can't pay costs involving impossible events | Phase 6 | NEW-614.17 | META-101.2 |
| ATOM-614.17c-001 | 614.17c | "Can't" event only replaceable by self-replacement changing type | Phase 6 | NEW-614.17 | META-101.2 |
| ATOM-614.17d-001 | 614.17d | ETB "can't" uses look-ahead (enters tapped) | Phase 6 | NEW-614.17 | META-101.2, META-614.17d |
| ATOM-615.4-001 | 615.4 | Prevention must exist before damage event | Phase 6 | NEW-615.4 | |
| ATOM-615.5-001 | 615.5 | Prevention with additional effect fires after prevention | Phase 6 | NEW-615.5 | |
| ATOM-615.6-001 | 615.6 | Prevented damage never happens; suppresses triggers | Phase 6+7 | NEW-615.6 | |
| ATOM-615.7-001 | 615.7 | Prevention shield depletes per damage point | Phase 6 | NEW-615.7 | |
| ATOM-615.7-002 | 615.7 | Multiple simultaneous sources: controller chooses allocation | Phase 6 | NEW-615.7 | |
| ATOM-615.8-001 | 615.8 | Prevent next instance from source: amount-independent | Phase 6 | NEW-615.8 | |
| ATOM-615.9-001 | 615.9 | Prevention shield rechecks source properties | Phase 6 | NEW-615.9 | |
| ATOM-615.10-001 | 615.10 | Static prevention applies per-event independently (Pyroclasm) | Phase 6 | NEW-615.10 | |
| ATOM-615.11-001 | 615.11 | Per-creature prevention shield assigned at resolution | Phase 6 | NEW-615.11 | |
| ATOM-615.12-001 | 615.12 | Unpreventable damage: shields preserved | Phase 6 | NEW-615.12 | META-101.2 |
| ATOM-615.12-002 | 615.12 | Unpreventable damage: additional effect conditional on 0 | Phase 6 | NEW-615.12 | |
| ATOM-615.12a-001 | 615.12a | Prevention on unpreventable: single application, no loop | Phase 6 | NEW-615.12 | |
| ATOM-616.1-001 | 616.1 | Player chooses which replacement/prevention to apply | Phase 6 | NEW-616.1 | |
| ATOM-616.1a-001 | 616.1a | Self-replacement must be chosen first | Phase 6 | NEW-616.1 | |
| ATOM-616.1b-001 | 616.1b | Control-changing replacement second priority | Phase 6 | NEW-616.1 | |
| ATOM-616.1c-001 | 616.1c | Copy replacement third priority (Essence of the Wild) | Phase 6 | NEW-616.1 | |
| ATOM-616.1f-001 | 616.1f | Iterative application until no more apply | Phase 6 | NEW-616.1 | |
| ATOM-616.1g-001 | 616.1g | Outer event replacement before inner event replacement | Phase 6 | NEW-616.1 | |
| ATOM-616.2-001 | 616.2 | Replacement chains: new replacement applies to modified event | Phase 6 | NEW-616.2 | |

**Total ATOMs: 94**

---

## BOUNDARY-DEF Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| BOUNDARY-DEF-609.7a-001 | 609.7a | Valid damage source choices (permanent vs hand card) | Phase 6 | NEW-609.7a | |
| BOUNDARY-DEF-614.1a-001 | 614.1a | "Instead" identifies replacement effect | Phase 6 | NEW-614.1a-d | |
| BOUNDARY-DEF-614.1b-001 | 614.1b | "Skip" identifies replacement effect | Phase 6 | NEW-614.1a-d | |
| BOUNDARY-DEF-614.1c-001 | 614.1c | ETB modification patterns are replacement effects | Phase 6 | NEW-614.1a-d | |
| BOUNDARY-DEF-614.1d-001 | 614.1d | Continuous ETB modification is replacement effect | Phase 6 | NEW-614.1a-d | |
| BOUNDARY-DEF-615.1a-001 | 615.1a | "Prevent" identifies prevention effect | Phase 6 | NEW-615.4 | |

**Total BOUNDARY-DEFs: 6**

---

## COMP Index

| ID | Rule(s) | Summary | Phase | Ticket | Composes |
|----|---------|---------|-------|--------|----------|
| COMP-613-HUMILITY-OPALESCENCE-001 | 613.1f+1g+4b+7 | Humility + Opalescence timestamp | Phase 5 (L19, L20) | L19, L20 | ATOM-613.1f-001, ATOM-613.4b-001, ATOM-613.7-001 (was -002, subsumed) |
| COMP-613-BLOOD-MOON-URBORG-001 | 613.8a+8 | Blood Moon + Urborg dependency | Phase 5 (L14, L17, L20) | L14, L17, L20 | ATOM-613.8a-001, ATOM-613.8-001 |
| COMP-613-TARMOGOYF-HUMILITY-001 | 613.1f+4a+4b | Tarmogoyf under Humility = 1/1 | Phase 5 (L19, L20) | L19, L20 | ATOM-613.1f-001, ATOM-613.4a-001, ATOM-613.4b-001 |
| COMP-614-616-DOUBLE-REPLACEMENT-001 | 614.5+616.1 | Two doublers with player choice = 8x | Phase 6 | NEW | ATOM-614.5-001, ATOM-616.1-001 |
| COMP-614-DAMAGE-ORDERING-001 | 614.5+616.1 | Non-commutative damage mods: +1 then ×2 vs ×2 then +1 | Phase 6 | NEW | ATOM-614.5-001, ATOM-616.1-001 |
| COMP-615-UNPREVENTABLE-SHIELD-001 | 615.12+615.7 | Unpreventable damage preserves shield | Phase 6 | NEW | ATOM-615.12-001, ATOM-615.7-001 |
| COMP-613-SVOGTHOS-001 | 613.6+613.4a | Svogthos multi-layer activation (L4+L5+L7a) | Phase 5 (L04, L05, L10) | L04, L05, L10 | ATOM-613.6-001, ATOM-613.4a-001, ATOM-613.3-001 |
| COMP-613-LAYERS-FULL-STACK-001 | 613.1a–g | Full 7-layer stack on single permanent | Phase 5+6 | L04–L12 | ATOM-613.1a-001 through ATOM-613.1g-001 |

**Total COMPs: 8**

---

## META Entries

### META-101.2 — "Can't overrides can" Concrete Tests

| Test ID | Rule | System |
|---------|------|--------|
| ATOM-614.17-001 | 614.17 | Prevention override |
| ATOM-614.17b-001 | 614.17b | Cost payment |
| ATOM-614.17c-001 | 614.17c | Self-replacement exception |
| ATOM-614.17d-001 | 614.17d | ETB "can't" |
| ATOM-615.12-001 | 615.12 | Unpreventable damage |

### META-614.17d — ETB "can't" uses look-ahead

Per-ETB-type meta rule: each ETB replacement type the engine supports should have a corresponding 614.17d "can't" test. Representative test: ATOM-614.17d-001 (enters tapped).

---

## Classification Summary Table

### PURE-DEF (no test needed) — 18 rules

609.1, 609.5, 609.6, 609.7, 610.1, 610.2, 611.1, 611.2, 611.3, 612.1, 612.4, 613.4, 614.1, 614.2, 614.3, 614.14, 615.1, 615.2, 615.3, 616.1e

### OUT-OF-SCOPE — 4 rules

| Rule | Reason |
|------|--------|
| 612.9 | Name stickers — Un-set/sticker mechanic |
| 613.7h | Plane/phenomenon/scheme cards — Planechase/Archenemy |
| 613.7j | Conspiracy card timestamp — Conspiracy draft |
| 613.7k | Sticker timestamp — Un-set mechanic |

### DEFERRED — 20 rules

| Rule | Target Phase | Reason |
|------|-------------|--------|
| 610.4 | Phase 9 | Phasing |
| 610.4a | Phase 9 | Phasing |
| 610.4b | Phase 9 | Phasing |
| 610.4c | Phase 9 | Phasing |
| 610.4d | Phase 9 | Phasing |
| 611.2f | Phase 7 | "Next spell" continuous effects (D8) |
| 612.5 | Post-v1 | Exchange of Words — single card |
| 612.6 | Post-v1 | Volrath's Shapeshifter — single card |
| 612.7 | Post-v1 | Spy Kit — single card |
| 612.10 | Phase 8 | Splice keyword |
| 613.2b | Phase 8 | Face-down Layer 1b (D3/Morph) |
| 613.7f | Phase 8 | Face-up/face-down timestamp (Morph/Transform) |
| 613.7g | Phase 8 | Transform/convert timestamp |
| 613.7i | Post-v1 | Vanguard stretch goal |
| 614.1e | Phase 8 | "As turned face up" replacement (Morph) |
| 614.12b | Phase 8 | Multiple simultaneous ETB choices |
| 614.12c | Phase 8 | Anchor word ETB choices |
| 614.13c | Phase 8 | ETB mill/exile from library exclusion |
| 615.13 | Phase 7 | Triggered abilities on prevention |
| 616.1d | Phase 8 | Back-face-up ETB replacement priority (Transform) |

### ALREADY-IMPLEMENTED

614.7a (partial — zero damage no-op exists in `actions.rs`, but replacement effect layer doesn't)

---

## NEW Tickets

| Ticket | Rule(s) | Phase | Description |
|--------|---------|-------|-------------|
| NEW-609.3 | 609.3 | Phase 8 | Partial effect execution for impossible instructions |
| NEW-609.4 | 609.4, 609.4a, 609.4b | Phase 8 | "As though" effect scoping and composition |
| NEW-609.7a | 609.7a | Phase 6 | Damage source choice validation |
| NEW-609.7b | 609.7b | Phase 6 | Prevention shield source property rechecking |
| NEW-609.7c | 609.7c | Phase 6 | Static prevention covers non-battlefield sources |
| NEW-610.3 | 610.3–610.3d | Phase 7 | "Until leaves" zone-change return effects (D9) |
| NEW-610.5 | 610.5 | Phase 7 | Static ability grants at cast time |
| NEW-611.2b | 611.2b | Phase 8 | "For as long as" duration pre-check (D7) |
| NEW-611.2c-mix | 611.2c | Phase 5+6 | Mixed characteristic/rule effect independent sets |
| NEW-611.2e | 611.2e | Phase 7+5 | Simultaneous "is [type]" ETB characteristic |
| NEW-611.3d | 611.3d | Phase 7+5 | Static grant persists after source leaves |
| NEW-613.1a | 613.1a | Phase 6 | Layer 1 copy effect ordering (D1) |
| NEW-613.2 | 613.2 | Phase 8 | Layer 1 sublayer ordering (D3) |
| NEW-613.2a | 613.2a | Phase 6 | Layer 1a copiable effects (D1) |
| NEW-613.2c | 613.2c | Phase 6 | Copiable values post-Layer-1 (D1) |
| NEW-613.7a | 613.7a | Phase 5 | Static ability timestamp = later of object vs grant (D5) |
| NEW-613.7c | 613.7c | Phase 5 | Counter timestamps within L7c (D6) |
| NEW-613.7e | 613.7e | Phase 5 | Aura/Equipment re-timestamp on attach (D4) |
| NEW-613.1f-kw | 613.1f | Phase 5 | Keyword counters in Layer 6 (D10) |
| NEW-614.1a-d | 614.1a–d | Phase 6 | Replacement/prevention effect classification |
| NEW-614.4 | 614.4 | Phase 6 | Replacement timing enforcement |
| NEW-614.5 | 614.5 | Phase 6 | Single-application rule |
| NEW-614.6 | 614.6 | Phase 6+7 | Replaced event suppression + triggers |
| NEW-614.7 | 614.7, 614.7a | Phase 6 | Non-event replacement no-op |
| NEW-614.8 | 614.8 | Phase 6 | Regeneration as destruction-replacement |
| NEW-614.9 | 614.9 | Phase 6 | Damage redirection with invalid destination |
| NEW-614.10 | 614.10, 614.10a, 614.10b | Phase 6 | Skip replacement effects |
| NEW-614.11 | 614.11, 614.11a, 614.11b | Phase 6 | Draw replacement effects |
| NEW-614.12 | 614.12, 614.12a | Phase 6 | ETB look-ahead + choice timing |
| NEW-614.13 | 614.13, 614.13a, 614.13b | Phase 6 | ETB auxiliary zone changes |
| NEW-614.15 | 614.15 | Phase 6 | Self-replacement priority |
| NEW-614.16 | 614.16 | Phase 6 | Token/counter replacement chains |
| NEW-614.17 | 614.17, 614.17a–d | Phase 6 | "Can't" effects (META-101.2) |
| NEW-615.4 | 615.4 | Phase 6 | Prevention timing enforcement |
| NEW-615.5 | 615.5 | Phase 6 | Prevention additional effects |
| NEW-615.6 | 615.6 | Phase 6+7 | Prevented damage trigger suppression |
| NEW-615.7 | 615.7 | Phase 6 | Prevention shield depletion + allocation choice |
| NEW-615.8 | 615.8 | Phase 6 | Instance-based prevention |
| NEW-615.9 | 615.9 | Phase 6 | Prevention source property recheck |
| NEW-615.10 | 615.10 | Phase 6 | Static per-event prevention |
| NEW-615.11 | 615.11 | Phase 6 | Per-creature shield assignment at resolution |
| NEW-615.12 | 615.12, 615.12a | Phase 6 | Unpreventable damage shield preservation |
| NEW-616.1 | 616.1, 616.1a–c, 616.1f–g | Phase 6 | Multiple replacement ordering + priority + iteration |
| NEW-616.2 | 616.2 | Phase 6 | Replacement chaining across event modifications |

**Total NEW tickets: 42** (2 in Phase 5, 30 in Phase 6, 4 in Phase 7, 4 in Phase 8, 2 in Phase 5+6/7+5)

---

## Gap Report

| Gap | Description | Recommended Action |
|-----|-------------|-------------------|
| G1 | Oracle routing migration (L13) — no CR rule | Covered by L13 ticket's own tests. No ATOM needed. |
| G2 | Fuzz regression (L21) — no CR rule | L21 ticket's acceptance criteria. Not a CR test. |
| G3 | LKI system (L18) — rule 608.2h in Session 5 scope | Cross-reference Session 5 for LKI tests. |
| G4 | Phase 6 execute_action middleware insertion | Engineering task covered by Phase 6 design. |
| G5 | Replacement effect registration and matching | Engineering infrastructure. Test via behavioral ATOMs. |

---

## Statistics

| Category | Count |
|----------|-------|
| ATOM tests | 94 |
| BOUNDARY-DEF tests | 6 |
| COMP tests | 8 |
| PURE-DEF | 18 |
| OUT-OF-SCOPE | 4 |
| DEFERRED | 20 |
| META-101.2 concrete tests | 5 |
| NEW tickets | 42 |
| Total sub-rules processed | 138 |

> **Note:** session-6.md statistics section lists 7 BOUNDARY-DEFs and 9 COMPs; actual counted entries are 6 and 8 respectively. Likely an off-by-one in the post-audit stat update.
