# Session 7A — Atomic Test Specifications: Rules 700.x + 701.x (General + Keyword Actions)

> **CR Source:** `MTG-Rules/LLM-Chapter-Splits/ch7-pt-1.txt` (rules 700.1–701.68)
> **Scope:** General additional rules (700.x) + keyword actions (701.x). No keyword abilities (702.x — Sessions 7B/8).
> **Cross-references:** design_doc.md, roadmap.md, implementation-plan-final.md

> **Convention note (post-audit):** Pseudocode signatures like `attach(game, id, target)` are conceptual. In the actual codebase: mutations are methods on `Game`/`GameState` or free functions taking `&mut GameState` (in `engine/`); read-only queries are free functions taking `&GameState` (in `oracle/`). E.g., `game.state.attach(attachment_id, target_id)` or `oracle::compute_devotion(&game.state, player_id, color)`.

---

## Chunk Plan

| Chunk | Rules | Description |
|-------|-------|-------------|
| 1 | 700.1–700.15 | General additional rules (events, modal, piles, dies, devotion, historic, party, modified, etc.) |
| 2 | 701.1–701.8 | Activate, Attach, Behold, Cast, Counter, Create, Destroy |
| 3 | 701.9–701.14 | Discard, Double, Triple, Exchange, Exile, Fight |
| 4 | 701.15–701.21 | Goad, Investigate, Mill, Play, Regenerate, Reveal, Sacrifice |
| 5 | 701.22–701.28 | Scry, Search, Shuffle, Surveil, Tap/Untap, Transform, Convert |
| 6 | 701.29–701.42 | Fateseal, Clash, Planeswalk, Set in Motion, Abandon, Proliferate, Detain, Populate, Monstrosity, Vote, Bolster, Manifest, Support, Meld |
| 7 | 701.43–701.55 | Exert, Explore, Assemble, Adapt, Amass, Learn, Venture, Connive, Open Attraction, Roll to Visit, Incubate, Ring Tempts, Face Villainous Choice |
| 8 | 701.56–701.68 | Time Travel, Discover, Cloak, Collect Evidence, Suspect, Forage, Manifest Dread, Endure, Harness, Airbend, Earthbend, Waterbend, Blight |
| Final | — | Classification Summary Table, Composition Tests, Gap Report |

---

## Chunk 1 — Rules 700.1–700.15 (General Additional Rules)

### 700.1 — Events
**Classification: PURE-DEF.** Defines "event" as a concept. No independent mechanical consequence. Prerequisite for triggered abilities (Phase 7) and replacement effects (Phase 6).

### 700.2 — Modal Spells and Abilities

**700.2 (header):** PURE-DEF — defines what "modal" means. Prerequisite for 700.2a–i.

**ATOM-700.2a-001**
- **Rule:** 700.2a — Controller of modal spell/activated ability chooses modes as part of casting/activating. Illegal modes can't be chosen.
- **Mechanism:** Modal spell casting — mode choice at cast time via `choose_modes` DP method
- **Minimal Board:** P0 has a modal spell (e.g., Charm with 3 modes) in hand. One mode targets a creature, but no creatures exist on battlefield.
- **Action:** P0 casts the Charm spell.
- **Expected Result:** The mode requiring a creature target cannot be chosen. Only modes with legal targets (or no targets) are selectable.
- **Phase:** Phase 5-Pre (T18 — casting pipeline 601.2b compliance)
- **Ticket:** T18

**ATOM-700.2b-001**
- **Rule:** 700.2b — Controller of modal triggered ability chooses modes when putting it on the stack. If no mode can be chosen, ability is removed from stack.
- **Mechanism:** Modal triggered ability — mode choice at stack placement, removal if no legal mode
- **Minimal Board:** P0 controls a permanent with a modal triggered ability. Both modes require targets that don't exist.
- **Action:** The trigger condition is met; ability would go on stack.
- **Expected Result:** No mode can be chosen; ability is removed from the stack entirely (not fizzled — never placed).
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** NEW — modal triggered ability mode selection + removal on no legal mode

**ATOM-700.2c-001**
- **Rule:** 700.2c — Mode-conditional targeting: targets for a mode are chosen only if that mode was selected.
- **Mechanism:** Casting pipeline — target selection conditioned on chosen modes
- **Minimal Board:** P0 has a modal spell with Mode A (target creature, +2/+2) and Mode B (draw a card). P0 chooses Mode B only. One creature on battlefield.
- **Action:** P0 casts the spell choosing Mode B.
- **Expected Result:** No target is chosen or required. The spell resolves drawing a card. The creature is irrelevant.
- **Phase:** Phase 5-Pre (T18)
- **Ticket:** T18

**ATOM-700.2d-001**
- **Rule:** 700.2d — Same mode normally can't be chosen twice; "choose the same mode more than once" overrides this.
- **Mechanism:** Mode uniqueness enforcement in `choose_modes`
- **Minimal Board:** P0 has a "choose two" modal spell with 3 modes, none of which say "same mode more than once."
- **Action:** P0 attempts to choose Mode A twice.
- **Expected Result:** Rejected — must choose two different modes.
- **Phase:** Phase 5-Pre (T18)
- **Ticket:** T18

**ATOM-700.2d-002**
- **Rule:** 700.2d — When "same mode more than once" is allowed, choosing a mode multiple times treats it as appearing that many times in sequence. Repeated modes are grouped together in declaration order.
- **Mechanism:** Modal spell with repeated mode — effect applied N times in declared order
- **Minimal Board:** P0 has a spell "choose three, you may choose same mode more than once" with Mode A: deal 2 damage to target creature, Mode B: draw a card. P0 chooses Mode A (targeting X), Mode A (targeting Y), Mode B. Creature X (5/5), Creature Y (3/3) on battlefield.
- **Action:** P0 casts, choosing modes in order: A(X), A(Y), B.
- **Expected Result:** On resolution: deal 2 to X, deal 2 to Y, draw a card — in that order. The two damage modes resolve sequentially before the draw. The spell reads "Deal 2 damage to X. Deal 2 damage to Y. Draw a card." (not interleaved as "Deal 2 to X. Draw a card. Deal 2 to Y.").
- **Phase:** Phase 8 (modal spell resolution)
- **Ticket:** NEW — repeated mode resolution + execution ordering

**ATOM-700.2e-001**
- **Rule:** 700.2e — Opponent chooses mode for a spell/ability when specified. If multiple opponents, controller picks which opponent chooses.
- **Mechanism:** Mode delegation to opponent via DP
- **Minimal Board:** P0 casts a spell that says "an opponent chooses one —" with two modes. P1 is the only opponent.
- **Action:** Spell is cast. P1 must choose the mode.
- **Expected Result:** P1's `choose_modes` is invoked (not P0's). Chosen mode is stored on StackEntry.
- **Phase:** Phase 5-Pre (T18)
- **Ticket:** T18

**700.2f:** PURE-DEF — changing target can't change mode. No independent testable behavior beyond the targeting system (Session 5).

**ATOM-700.2g-001**
- **Rule:** 700.2g — A copy of a modal spell copies the modes chosen. Controller of copy can't choose different modes.
- **Mechanism:** Spell copy preserves mode selection
- **Minimal Board:** P0 casts a modal spell choosing Mode A. An effect copies that spell.
- **Action:** Copy is placed on stack.
- **Expected Result:** Copy has Mode A locked in. No mode choice prompt for the copy's controller.
- **Phase:** Phase 7 (spell copying — D19)
- **Ticket:** NEW — spell copy preserves modes (D19)

**ATOM-700.2h-001**
- **Rule:** 700.2h — Modal spells with per-mode additional costs: costs must be paid for each chosen mode.
- **Mechanism:** Per-mode additional cost aggregation in casting pipeline
- **Minimal Board:** P0 has a modal spell: Mode A (pay 2 life, deal 3 damage) / Mode B (sacrifice a creature, destroy target). P0 chooses both modes.
- **Action:** P0 casts choosing both. Total cost = base mana + 2 life + sacrifice a creature.
- **Expected Result:** Both additional costs are aggregated into total cost and must be paid during 601.2f–h.
- **Phase:** Phase 5-Pre (T18)
- **Ticket:** T18

**ATOM-700.2i-001**
- **Rule:** 700.2i — Pawprint ({P}) modal spells: choose modes up to total pawprint budget.
- **Mechanism:** Pawprint-budget mode selection
- **Minimal Board:** P0 has spell "choose up to 3 {P} worth of modes." Mode A costs 1{P}, Mode B costs 2{P}, Mode C costs 1{P}.
- **Action:** P0 casts, choosing Mode B (2{P}) + Mode C (1{P}) = 3{P} total.
- **Expected Result:** Legal — total ≤ budget. Choosing all three (4{P}) would be illegal.
- **Phase:** Phase 8 (niche modal variant)
- **Ticket:** NEW — pawprint modal budget

### 700.3 — Piles

**700.3 (header):** PURE-DEF — introduces pile concept.

**ATOM-700.3a-001**
- **Rule:** 700.3a — Each affected object must be put into exactly one pile.
- **Mechanism:** Pile-forming algorithm — no object in multiple piles
- **Minimal Board:** Fact or Fiction resolves, revealing 5 cards from library top.
- **Action:** Opponent separates cards into two piles.
- **Expected Result:** Each card is in exactly one pile. Engine rejects an assignment where a card appears in both piles or neither.
- **Phase:** Phase 8 (Fact or Fiction — pile effects)
- **Ticket:** NEW — pile formation and validation

**700.3b:** PURE-DEF — pile is not an object, objects remain individuals. No independent test.

**ATOM-700.3c-001**
- **Rule:** 700.3c — Objects in piles don't leave their current zone during pile formation.
- **Mechanism:** Pile formation does NOT trigger zone-change events
- **Minimal Board:** Fact or Fiction resolves. 5 cards revealed from top of library.
- **Action:** Opponent separates into piles. Cards are still in library until the effect puts them into hand/graveyard.
- **Expected Result:** No zone-change events fire during pile formation. Zone changes happen only when the chosen pile is put into hand and the other into graveyard.
- **Phase:** Phase 8
- **Ticket:** NEW — pile formation zone invariant

**700.3d:** PURE-DEF — pile can contain zero objects. Covered by 700.3a test (opponent can put all 5 in one pile, 0 in the other).

### 700.4 — Dies

**700.4:** PURE-DEF / ALREADY-IMPLEMENTED — "dies" = "put into graveyard from battlefield." The engine already routes creature death through `move_object` to graveyard. The delta log (Phase 7) will tag these as "died" events. No additional test needed here — concrete "dies" trigger tests belong to Phase 7 (Session 9A, rule 603).

### 700.5 — Devotion

**ATOM-700.5-001**
- **Rule:** 700.5 — Devotion to a color = count of mana symbols of that color among mana costs of permanents a player controls.
- **Mechanism:** `compute_devotion(game, player_id, color) -> u32` oracle query
- **Minimal Board:** P0 controls: Permanent A (cost {R}{R}{R}), Permanent B (cost {1}{R}), Permanent C (cost {2}{G}).
- **Action:** Query P0's devotion to red.
- **Expected Result:** Devotion to red = 4 (3 from A + 1 from B + 0 from C).
- **Phase:** Phase 8 (devotion predicate)
- **Ticket:** NEW — devotion computation

**ATOM-700.5-002**
- **Rule:** 700.5 — Devotion to two colors counts symbols that are either color (hybrid {R/G} counts once, not twice).
- **Mechanism:** `compute_devotion(game, player_id, color1, color2)` — hybrid symbol counting
- **Minimal Board:** P0 controls Permanent A (cost {R/G}{R}{G}).
- **Action:** Query P0's devotion to red and green.
- **Expected Result:** Devotion = 3 ({R/G} counts once as it is red-or-green, {R} counts, {G} counts).
- **Phase:** Phase 8
- **Ticket:** NEW — devotion computation (two-color)

**ATOM-700.5a-001**
- **Rule:** 700.5a — Devotion calculated after copy/control/text effects but before other characteristic modifications. Exception to 613.10.
- **Mechanism:** Devotion uses partial-layer result (after L1–L3 but before L4–L7). The layer system needs a `compute_at_layer(obj, layer_3)` query hook for devotion.
- **Minimal Board:** P0 controls Purphoros ({3}{R}, "not a creature if devotion to red < 5") and permanents with 4 total {R} symbols. An anthem gives all creatures +1/+1 (L7c).
- **Action:** Check if Purphoros is a creature.
- **Expected Result:** Devotion computed at L3 result = 4+1(Purphoros) = 5. Purphoros IS a creature. The anthem applies after this determination. (Devotion is calculated before type-changing effects apply, per 700.5a.)
- **Phase:** Backlog — modal spells and devotion (was Phase 5-Layers)
- **Ticket:** NEW — devotion partial-layer computation (6b-D14)
- **Tags:** dependency, layers
- **Architectural note:** Devotion computation must be tightly coupled to the layer system, not the general devotion oracle query. The oracle `compute_devotion` for non-layer contexts (e.g., "devotion to red ≥ 5" conditions on spells) can call the layer system's partial result. The layer system owns the computation.

**ATOM-700.5a-002**
- **Rule:** 700.5a — Devotion-modifying effects: Altar of the Pantheon ("Your devotion to each color and each combination of colors is increased by one.") changes the devotion result used by the partial-layer query.
- **Mechanism:** Devotion modifier — an effect that adjusts the devotion count rather than adding mana symbols
- **Minimal Board:** P0 controls Purphoros ({3}{R}, "not a creature if devotion to red < 5"), permanents with 3 total {R} symbols, and Altar of the Pantheon ("Your devotion to each color and combination of colors is increased by one."). Purphoros contributes 1 {R}. Base devotion = 4 + 1 (Altar) = 5.
- **Action:** Layer system recalculates.
- **Expected Result:** Devotion to red = 5 (3 others + 1 Purphoros + 1 Altar). 5 ≥ 5 → Purphoros IS a creature. Without Altar, devotion would be 4 < 5 and Purphoros would not be a creature. This tests that devotion-modifying effects (not just mana symbols) feed into the partial-layer devotion computation.
- **Phase:** Backlog — modal spells and devotion (was Phase 5-Layers)
- **Ticket:** NEW — devotion modifier effect test

### 700.6 — Historic

**BOUNDARY-DEF-700.6-001**
- **Rule:** 700.6 — "Historic" = legendary supertype OR artifact card type OR Saga subtype.
- **Mechanism:** `is_historic(game, object_id) -> bool` predicate
- **Minimal Board:** Object A: legendary creature. Object B: non-legendary artifact. Object C: Saga enchantment. Object D: non-legendary non-artifact non-Saga creature.
- **Action:** Query `is_historic` for each.
- **Expected Result:** A = true (legendary), B = true (artifact), C = true (Saga), D = false.
- **Phase:** Phase 8 (batch predicates)
- **Ticket:** NEW — historic predicate

### 700.7 — "This [something]"

**700.7:** DEFERRED — Phase 7 (triggered ability object identity / epoch tracking). The rule defines that "this creature" in an ability still refers to the object even if it no longer has the referenced quality. This is part of the epoch/ObjectRef identity system designed in session 4 (rule 400.7). Concrete tests will be generated in the triggered ability session.

### 700.8 — Party

**700.8 + 700.8a–d:** DEFERRED — Phase 8 (batch predicates). Party mechanic requires creature type checking across four types (Cleric, Rogue, Warrior, Wizard) with optimal assignment (700.8b). Niche Zendikar Rising mechanic.

### 700.9 — Modified

**BOUNDARY-DEF-700.9-001**
- **Rule:** 700.9 — "Modified" = has counters OR is equipped OR is enchanted by an Aura its controller controls.
- **Mechanism:** `is_modified(game, object_id) -> bool` predicate
- **Minimal Board:** Creature A: has a +1/+1 counter. Creature B: equipped by an Equipment. Creature C: enchanted by an Aura controlled by C's controller. Creature D: enchanted by an Aura controlled by opponent. Creature E: no counters, no attachments.
- **Action:** Query `is_modified` for each.
- **Expected Result:** A = true (counters), B = true (equipped), C = true (friendly Aura), D = false (opponent's Aura), E = false.
- **Phase:** Phase 8 (batch predicates)
- **Ticket:** NEW — modified predicate

### 700.10 — "Activated This Turn"

**700.10:** DEFERRED — Phase 8. Requires per-permanent per-turn activation tracking. The delta log is the right mechanism: `was_activated_this_turn(game, permanent_id) -> bool` scans the current turn's deltas for `ActivateAbility { source: permanent_id }` matches. Currently only one card uses this rule, but it's recent enough to recur. (T19 introduces `activation_count` tracking for a related but distinct concept.)

### 700.11 — Descended

**700.11:** DEFERRED — Phase 8 (batch predicates). Requires per-turn tracking of permanent cards entering graveyard from battlefield. Delta log emits `ZoneChange { object_id, origin: Battlefield, destination: Graveyard }` deltas. The "descended" check has its own logic: scan current turn's deltas for ZoneChange deltas where origin=Battlefield and destination=Graveyard, then check if the moved object was a permanent card (not just a permanent — a token that goes to graveyard and ceases to exist is not a "permanent card"). Niche Ixalan mechanic.

**Design note — Orphaned ObjectIds:** Tokens that cease to exist (rule 111.8) leave behind orphaned ObjectId entries in the central `HashMap<ObjectId, GameObject>` and in the delta log. This is part of a broader pattern: delayed triggers can reference stale ObjectIds, LKI snapshots hold references to removed objects, etc. **Current stance:** do NOT eagerly clean up orphaned IDs. The cost of retaining them is low (a few hundred bytes per token per game), and eager cleanup risks invalidating LKI references, delta log entries, and any system scanning historical state. A per-turn or per-game GC pass could reclaim IDs whose `last_zone_change_epoch` is old enough that no effect could reference them, but this is a Phase 10 optimization concern, not a correctness issue.

### 700.12 — Outlaw

**BOUNDARY-DEF-700.12-001**
- **Rule:** 700.12 — "Outlaw" = Assassin, Mercenary, Pirate, Rogue, or Warlock creature type.
- **Mechanism:** `is_outlaw(game, object_id) -> bool` predicate
- **Minimal Board:** Creature A: type Rogue. Creature B: type Warrior.
- **Action:** Query `is_outlaw` for each.
- **Expected Result:** A = true (Rogue), B = false (Warrior not in outlaw set).
- **Phase:** Phase 8 (batch predicates)
- **Ticket:** NEW — outlaw predicate

**700.12a:** PURE-DEF — "outlaws a player controls" refers to outlaw permanents unless specified. Covered by boundary test above.

### 700.13 — Committing a Crime

**700.13:** DEFERRED — Phase 8 (requires per-event tracking: "targets at least one opponent, or permanent/spell/ability an opponent controls, or card in opponent's graveyard"). Niche Thunder Junction mechanic.

### 700.14 — Expend

**700.14:** DEFERRED — Phase 8 (requires per-turn mana-spent-on-spells accumulator). Niche Bloomburrow mechanic.

### 700.15 — "Enters"

**700.15:** PURE-DEF — "'enters' is short for 'enters the battlefield.'" Terminology shorthand. No mechanical consequence beyond what ETB already means.

--- End of Chunk 1 ---

## Chunk 2 — Rules 701.1–701.8 (Activate, Attach, Behold, Cast, Counter, Create, Destroy)

### 701.1 — Keyword Actions (intro)
**Classification: PURE-DEF.** Introductory text explaining that some verbs are game keywords. No mechanical consequence.

### 701.2 — Activate

**701.2a:** ALREADY-IMPLEMENTED — "To activate an activated ability is to put it onto the stack and pay its costs." This is the core `activate_ability` pathway in `engine/cast.rs`. Rule 602 governs the full procedure. The sub-rule about "only controller/owner can activate unless the object says otherwise" is enforced by legality checks. Fully covered by Phase 2 + Phase 4.5 tests.

### 701.3 — Attach

**ATOM-701.3a-001**
- **Rule:** 701.3a — To attach an Aura/Equipment/Fortification means to take it from where it is and put it onto an object/player. Can't attach to something it couldn't enchant/equip/fortify.
- **Mechanism:** `attach(game, attachment_id, target_id)` — moves attachment onto target, validates legality
- **Minimal Board:** Equipment E (on battlefield, unattached). Creature C (on battlefield). Enchantment X (not an Equipment/Aura).
- **Action:** Attach E to C.
- **Expected Result:** E is now attached to C. E's `attached_to` field = C's ObjectId.
- **Phase:** Phase 5-Pre (T15 — Aura/Equipment attach)
- **Ticket:** T15

**ATOM-701.3a-002**
- **Rule:** 701.3a — Can't attach to an object it couldn't enchant/equip/fortify.
- **Mechanism:** Attach legality check — Equipment can only equip creatures
- **Minimal Board:** Equipment E (on battlefield). Non-creature permanent P (e.g., an artifact with no creature type).
- **Action:** Attempt to attach E to P.
- **Expected Result:** Attach fails / is illegal. E remains unattached.
- **Phase:** Phase 5-Pre (T15)
- **Ticket:** T15

**ATOM-701.3b-001**
- **Rule:** 701.3b — If an effect tries to attach to something it can't be attached to, it doesn't move.
- **Mechanism:** Failed attach = no movement
- **Minimal Board:** Equipment E attached to Creature C. Effect tries to attach E to a player (Equipment can't equip players).
- **Action:** Effect resolves attempting the illegal attach.
- **Expected Result:** E stays attached to C. No state change.
- **Phase:** Phase 5-Pre (T15)
- **Ticket:** T15

**ATOM-701.3b-002**
- **Rule:** 701.3b — Attaching to the same object it's already attached to does nothing.
- **Mechanism:** No-op on same-target reattach
- **Minimal Board:** Equipment E attached to Creature C.
- **Action:** Effect says "attach E to C" (already attached).
- **Expected Result:** No state change. No new timestamp. No triggered abilities fire for "becoming attached."
- **Phase:** Phase 5-Pre (T15)
- **Ticket:** T15

**ATOM-701.3b-003**
- **Rule:** 701.3b — Attaching a non-Aura/Equipment/Fortification object does nothing.
- **Mechanism:** Type check — only valid attachment subtypes can attach
- **Minimal Board:** Creature C1, Creature C2 on battlefield. Effect says "attach C1 to C2."
- **Action:** Effect resolves.
- **Expected Result:** Nothing happens. C1 doesn't move or change state.
- **Phase:** Phase 5-Pre (T15)
- **Ticket:** T15

**ATOM-701.3c-001**
- **Rule:** 701.3c — Attaching to a different object/player gives the attachment a new timestamp.
- **Mechanism:** Timestamp refresh on reattach
- **Minimal Board:** Equipment E attached to Creature A (timestamp T1). Creature B on battlefield.
- **Action:** Effect moves E from A to B.
- **Expected Result:** E is now attached to B. E's timestamp > T1 (new timestamp allocated).
- **Phase:** Phase 5-Pre (T15)
- **Ticket:** T15
- **Tags:** dependency, layers (timestamp ordering for layer system)

**ATOM-701.3d-001**
- **Rule:** 701.3d — To "unattach" an Equipment means it stays on battlefield but equips nothing.
- **Mechanism:** `unattach(game, equipment_id)` — clears `attached_to`, Equipment remains on battlefield
- **Minimal Board:** Equipment E attached to Creature C.
- **Action:** An effect unattaches E from C.
- **Expected Result:** E is on battlefield, `attached_to` = None. C has no equipment bonus.
- **Phase:** Phase 5-Pre (T15b — unattach)
- **Ticket:** T15b

**ATOM-701.3d-002**
- **Rule:** 701.3d — "Becoming unattached" includes the case where the equipped creature leaves the battlefield.
- **Mechanism:** Zone-change cleanup triggers unattach
- **Minimal Board:** Equipment E attached to Creature C.
- **Action:** Creature C is destroyed (moves to graveyard).
- **Expected Result:** E becomes unattached (remains on battlefield, `attached_to` = None). This counts as E "becoming unattached from" C.
- **Phase:** Phase 5-Pre (T15b)
- **Ticket:** T15b

### 701.4 — Behold

**701.4 + 701.4a–b:** DEFERRED — Phase 8. Niche Aetherdrift mechanic. Requires reveal-from-hand or choose-permanent DP interaction. One-line entry.

### 701.5 — Cast

**701.5a:** ALREADY-IMPLEMENTED — "To cast a spell is to take it from the zone it's in, put it on the stack, and pay its costs." This is the core `cast_spell` pathway in `engine/cast.rs`. Rule 601 governs full procedure. Fully covered by Phase 2 tests.

**701.5b:** PURE-DEF — "To cast a card is to cast it as a spell." Terminological clarification. No additional test.

### 701.6 — Counter

**701.6a:** ALREADY-IMPLEMENTED — "To counter a spell or ability means to cancel it, removing it from the stack." Implemented in `engine/stack.rs` (counter_spell) and `engine/resolve.rs`. Countered spells go to owner's graveyard. Tested in Phase 2 (Counterspell card, fizzle tests).

**701.6b:** ALREADY-IMPLEMENTED — No cost refund on counter. The engine never refunds costs; costs are paid during 601.2f–h and are irrevocable. Documented in `cast.rs` comments. No additional test needed.

### 701.7 — Create (Tokens)

**ATOM-701.7a-001**
- **Rule:** 701.7a — To create tokens, put the specified number with specified characteristics onto the battlefield.
- **Mechanism:** `create_token(game, controller, token_def) -> ObjectId` — creates GameObject with token flag, places on battlefield
- **Minimal Board:** P0 controls nothing. A spell resolves with effect "Create two 1/1 white Soldier creature tokens."
- **Action:** Effect resolves.
- **Expected Result:** Two new GameObjects on battlefield, each: token=true, power=1, toughness=1, color=white, types={Creature}, subtypes={Soldier}, controller=P0, owner=P0.
- **Phase:** Phase 8 (token creation)
- **Ticket:** NEW — create_token primitive

**ATOM-701.7b-001**
- **Rule:** 701.7b — Replacement effects applying to token creation apply BEFORE continuous effects modify the token's characteristics. Replacement effects applying to the token entering the battlefield apply AFTER continuous effects.
- **Mechanism:** Two-stage replacement: creation-replacement → continuous effects → ETB-replacement
- **Minimal Board:** P0 controls an anthem "Creatures you control get +1/+1." A replacement effect says "If you would create a token, create two instead" (e.g., Doubling Season). P0 creates a 1/1 Soldier token.
- **Action:** Token creation resolves.
- **Expected Result:** Doubling Season (creation replacement) fires first → two tokens. Anthem (continuous effect) applies to each → each is 2/2. Any ETB replacement effects would see the 2/2 tokens.
- **Phase:** Phase 8 (depends on Phase 6 replacement effects + Phase 5 layers)
- **Ticket:** NEW — token creation replacement ordering
- **Tags:** dependency, replacement-effects, layers

**701.7c:** PURE-DEF — Errata note about old "put onto the battlefield" wording. No mechanical consequence.

### 701.8 — Destroy

**701.8a:** ALREADY-IMPLEMENTED — "To destroy a permanent, move it from the battlefield to its owner's graveyard." Implemented via `Primitive::Destroy` in `engine/resolve.rs` which calls `execute_action(GameAction::ZoneChange { destination: Graveyard })`. Tested in Phase 2 (Lightning Bolt killing creatures via SBA, direct destroy effects).

**ATOM-701.8b-001**
- **Rule:** 701.8b — The ONLY ways a permanent can be "destroyed" are: (1) an effect using "destroy", (2) SBA for lethal damage (704.5g), (3) SBA for deathtouch damage (704.5h). Any other graveyard move is NOT "destroyed."
- **Mechanism:** Destroy flag / delta tag distinguishing destroy from sacrifice/other graveyard moves
- **Minimal Board:** Creature A (3/3) on battlefield. Creature B (2/2) on battlefield.
- **Action:** (1) Cast "Destroy target creature" on A. (2) Sacrifice B (via a cost).
- **Expected Result:** A's zone change is tagged as "destroyed" in the delta log. B's zone change is tagged as "sacrificed" — NOT "destroyed." Regeneration shields would apply to A but not to B.
- **Phase:** Phase 6 (replacement effects need destroy vs. sacrifice distinction)
- **Ticket:** NEW — destroy delta tagging (D20)
- **Tags:** dependency, replacement-effects

**ATOM-701.8c-001**
- **Rule:** 701.8c — A regeneration effect replaces a destruction event.
- **Mechanism:** Regeneration shield as replacement effect for destroy
- **Minimal Board:** Creature C (3/3) with a regeneration shield active.
- **Action:** Effect says "Destroy C."
- **Expected Result:** Destruction is replaced: all damage removed from C, C is tapped, C is removed from combat if attacking/blocking. C remains on battlefield.
- **Phase:** Phase 6 (replacement effects — regeneration)
- **Ticket:** NEW — regeneration replacement (ties to 701.19)
- **Tags:** dependency, replacement-effects

--- End of Chunk 2 ---

## Chunk 3 — Rules 701.9–701.14 (Discard, Double, Triple, Exchange, Exile, Fight)

### 701.9 — Discard

**701.9a:** ALREADY-IMPLEMENTED — "To discard a card, move it from its owner's hand to that player's graveyard." Implemented via `move_object` with destination=Graveyard, source=Hand. Discard-to-hand-size implemented in cleanup step. `choose_discard` DP method exists. Tested in Phase 1/Pre-Phase 3.

**ATOM-701.9b-001**
- **Rule:** 701.9b — By default, the affected player chooses which card to discard. Some effects require random discard or allow another player to choose.
- **Mechanism:** Random discard variant — engine picks card at random instead of via DP
- **Minimal Board:** P0 has 3 cards in hand. Effect says "P0 discards a card at random."
- **Action:** Effect resolves.
- **Expected Result:** One of P0's 3 hand cards is selected uniformly at random and moved to graveyard. P0's `choose_discard` is NOT invoked.
- **Phase:** Phase 8 (random discard variant)
- **Ticket:** NEW — random discard primitive

**ATOM-701.9b-002**
- **Rule:** 701.9b — Another player choosing which card to discard.
- **Mechanism:** Opponent-chosen discard — opponent's DP `choose_discard` invoked with P0's hand revealed
- **Minimal Board:** P0 has 3 cards in hand. Effect says "Target opponent chooses a card from P0's hand. P0 discards that card."
- **Action:** Effect resolves. P1 sees P0's hand and picks a card.
- **Expected Result:** P1's `choose_discard` is invoked (not P0's). The chosen card is discarded from P0's hand.
- **Phase:** Phase 8
- **Ticket:** NEW — opponent-chosen discard

**ATOM-701.9c-001**
- **Rule:** 701.9c — If a discarded card is put into a hidden zone without being revealed, its characteristics are undefined. If a cost required discarding a card with a specific characteristic, the cost payment is illegal.
- **Mechanism:** Discard-to-hidden-zone characteristic undefined check
- **Minimal Board:** P0 has a replacement effect "If you would discard a card, exile it face-down instead." P0 must "discard a creature card" as a cost.
- **Action:** P0 attempts to pay the cost by discarding a creature card.
- **Expected Result:** The card goes to exile face-down. Its characteristics are undefined in that hidden zone. If the cost required "a creature card," the payment is illegal — game rewinds to before cost payment (rule 732).
- **Phase:** Phase 8 (niche interaction — hidden zone discard)
- **Ticket:** NEW — discard-to-hidden-zone illegality check
- **Tags:** dependency, replacement-effects
- **Cross-ref:** Directly relevant to `plans/atomic-tests/603-2f-complexity.md` (triggered ability interaction with hidden-zone discard).

### 701.10 — Double

**ATOM-701.10a-001**
- **Rule:** 701.10a — Doubling a creature's power/toughness creates a continuous effect that modifies (not sets) P/T. See 613.4c.
- **Mechanism:** Double creates +X/+Y effect in layer 7c (not 7b)
- **Minimal Board:** Creature C base 3/4. Effect "double C's power and toughness" resolves.
- **Action:** Effect resolves.
- **Expected Result:** C gets +3/+4 (continuous effect in L7c). C is now 6/8. A subsequent "set P/T to 0/1" (L7b) would make it 0/1+3/4 = 3/5 if applied in correct layer order.
- **Phase:** Backlog — CR 701 keyword actions (was Phase 5-Layers)
- **Ticket:** NEW — double P/T as L7c continuous effect

**ATOM-701.10b-001**
- **Rule:** 701.10b — Doubling power: creature gets +X/+0 where X = current power at resolution. Doubling toughness: +0/+X where X = current toughness. Both: +X/+Y.
- **Mechanism:** Snapshot power at resolution time, apply as +X/+Y
- **Minimal Board:** Creature C has effective power 3, effective toughness 4. Effect "double C's power" resolves.
- **Action:** Effect resolves.
- **Expected Result:** C gets +3/+0. C is now 6/4. Toughness unchanged.
- **Phase:** Backlog — CR 701 keyword actions (was Phase 5-Layers)
- **Ticket:** NEW — double P/T snapshot

**ATOM-701.10c-001**
- **Rule:** 701.10c — If power < 0 when doubled, creature gets -X/-0 where X = |power|. (Negative doubled = more negative.)
- **Mechanism:** Negative power doubling
- **Minimal Board:** Creature C has effective power -2, toughness 3. Effect "double C's power and toughness."
- **Action:** Effect resolves.
- **Expected Result:** C gets -2/+3 (power: -2 doubled = gets -2/-0; toughness: 3 doubled = gets +0/+3). C is now -4/6.
- **Phase:** Backlog — CR 701 keyword actions (was Phase 5-Layers)
- **Ticket:** NEW — negative power doubling

**ATOM-701.10d-001**
- **Rule:** 701.10d — Doubling a player's life total: player gains or loses life to reach 2× current total.
- **Mechanism:** Life total doubling via gain/loss (not set)
- **Minimal Board:** P0 at 7 life.
- **Action:** Effect "double P0's life total."
- **Expected Result:** P0 gains 7 life (to reach 14). This is a life gain event — triggers "whenever you gain life" and is affected by "can't gain life" restrictions.
- **Phase:** Phase 8
- **Ticket:** NEW — double life total

**ATOM-701.10d-002**
- **Rule:** 701.10d — Doubling life when life total is negative.
- **Mechanism:** Life total doubling with negative life
- **Minimal Board:** P0 at -3 life (possible with Platinum Angel).
- **Action:** Effect "double P0's life total."
- **Expected Result:** P0 loses 3 life (to reach -6). This is a life loss event.
- **Phase:** Phase 8
- **Ticket:** NEW — double life total (negative)

**ATOM-701.10e-001**
- **Rule:** 701.10e — Doubling counters: give as many of that kind as already present.
- **Mechanism:** Counter doubling
- **Minimal Board:** Creature C has 3 +1/+1 counters.
- **Action:** Effect "double the number of +1/+1 counters on C."
- **Expected Result:** 3 more +1/+1 counters added. C now has 6 +1/+1 counters.
- **Phase:** Phase 8
- **Ticket:** NEW — double counters

**ATOM-701.10f-001**
- **Rule:** 701.10f — Doubling mana: add equal amount of that type.
- **Mechanism:** Mana pool doubling
- **Minimal Board:** P0 has {R}{R}{R} in mana pool (one {R} was produced by a source with restriction "spend only to cast creature spells").
- **Action:** Effect "double the amount of red mana in P0's mana pool" (e.g., Mana Flare or Doubling Cube).
- **Expected Result:** P0 adds {R}{R}{R} to pool. Pool now has {R}{R}{R}{R}{R}{R}. The 3 new {R} carry NO restrictions from the original sources — they are unrestricted mana (see Doubling Cube ruling in CR ch.1). The original restricted {R} retains its restriction.
- **Phase:** Phase 8
- **Ticket:** NEW — double mana (unrestricted)

**ATOM-701.10g-001**
- **Rule:** 701.10g — Doubling damage: source deals twice that much. This is a replacement effect.
- **Mechanism:** Damage doubling replacement effect
- **Minimal Board:** P0 controls "If a source you control would deal damage, it deals double that damage instead." P0's creature (3/3) deals 3 damage to P1.
- **Action:** Damage event resolves.
- **Expected Result:** Replacement fires: 3 → 6 damage dealt to P1. This is a single replacement (not applied recursively to itself).
- **Phase:** Phase 6 (replacement effects)
- **Ticket:** NEW — damage doubling replacement
- **Tags:** dependency, replacement-effects

### 701.11 — Triple

**ATOM-701.11a-001**
- **Rule:** 701.11a — Tripling P/T creates a continuous effect (L7c modification, not L7b set).
- **Mechanism:** Triple creates +X/+Y where X = 2× power, Y = 2× toughness
- **Minimal Board:** Creature C base 3/4.
- **Action:** Effect "triple C's power and toughness."
- **Expected Result:** C gets +6/+8. C is now 9/12.
- **Phase:** Backlog — CR 701 keyword actions (was Phase 5-Layers)
- **Ticket:** NEW — triple P/T as L7c continuous effect

**ATOM-701.11b-001**
- **Rule:** 701.11b — Tripling power: +X/+0 where X = 2× current power. Tripling toughness: +0/+X where X = 2× current toughness.
- **Mechanism:** Snapshot and multiply by 2 for the bonus
- **Minimal Board:** Creature C power 4, toughness 2. Effect "triple C's power."
- **Action:** Effect resolves.
- **Expected Result:** C gets +8/+0. C is now 12/2. Toughness unchanged.
- **Phase:** Backlog — CR 701 keyword actions (was Phase 5-Layers)
- **Ticket:** NEW — triple power only

**ATOM-701.11c-001**
- **Rule:** 701.11c — Negative power tripling: -X where X = 2× |power|.
- **Mechanism:** Negative power tripling
- **Minimal Board:** Creature C power -2, toughness 3. Effect "triple C's power and toughness."
- **Action:** Effect resolves.
- **Expected Result:** C gets -4/+6. C is now -6/9.
- **Phase:** Backlog — CR 701 keyword actions (was Phase 5-Layers)
- **Ticket:** NEW — negative power tripling

### 701.12 — Exchange

**ATOM-701.12a-001**
- **Rule:** 701.12a — If the entire exchange can't be completed, no part occurs.
- **Mechanism:** All-or-nothing exchange validation
- **Minimal Board:** Spell: "Exchange control of target creature you control and target creature an opponent controls." P0 targets Creature A (P0 controls) and Creature B (P1 controls). Before resolution, Creature B is destroyed.
- **Action:** Spell resolves. B is gone.
- **Expected Result:** Exchange can't complete (B doesn't exist). Nothing happens. P0 still controls A.
- **Phase:** Phase 8 (exchange effects)
- **Ticket:** NEW — exchange all-or-nothing

**ATOM-701.12b-001**
- **Rule:** 701.12b — Exchange control of two permanents: each player gains control of the other's permanent simultaneously.
- **Mechanism:** Simultaneous control swap
- **Minimal Board:** P0 controls Creature A. P1 controls Creature B. Effect "exchange control of A and B."
- **Action:** Effect resolves.
- **Expected Result:** P0 now controls B, P1 now controls A. Both transitions happen simultaneously. Control-change timestamps are identical.
- **Phase:** Phase 8
- **Ticket:** NEW — control exchange
- **Architectural note:** Delta log emits both control changes as a single batch (same epoch). The CR does not specify an internal ordering for simultaneous state changes within a single effect resolution (rule 101.4 governs player *choices*, not engine-internal event ordering; rule 613.7m governs *timestamps* for continuous effects entering simultaneously). For engine determinism, emit in a fixed order (e.g., by ObjectId) — but do NOT claim this is rules-mandated.

**ATOM-701.12b-002**
- **Rule:** 701.12b — Exchange control of two permanents controlled by the same player does nothing.
- **Mechanism:** Same-controller exchange = no-op
- **Minimal Board:** P0 controls Creature A and Creature B. Effect "exchange control of A and B."
- **Action:** Effect resolves.
- **Expected Result:** Nothing happens. Both remain under P0's control.
- **Phase:** Phase 8
- **Ticket:** NEW — same-controller exchange no-op

**ATOM-701.12c-001**
- **Rule:** 701.12c — Exchange life totals: each player gains/loses life to equal the other's previous total.
- **Mechanism:** Life exchange via gain/loss (not raw set)
- **Minimal Board:** P0 at 20 life. P1 at 5 life.
- **Action:** Effect "P0 and P1 exchange life totals."
- **Expected Result:** P0 loses 15 life (to 5). P1 gains 15 life (to 20). These are gain/loss events — affected by replacement effects ("can't gain life" etc.).
- **Phase:** Phase 8
- **Ticket:** NEW — life total exchange

**ATOM-701.12c-002**
- **Rule:** 701.12c — Player who can't gain life can't receive a higher total via exchange (see rule 119.7). Per 701.12a, if any part of the exchange can't complete, the entire exchange fails.
- **Mechanism:** "Can't gain life" blocks upward exchange → entire exchange fails
- **Minimal Board:** P0 at 5 life, P0 has "P0 can't gain life." P1 at 20 life. Effect "P0 and P1 exchange life totals."
- **Action:** Effect resolves.
- **Expected Result:** P0 would need to gain 15 life (5→20), but P0 can't gain life (rule 119.7). P0's half of the exchange is impossible. Per 701.12a, no part of the exchange occurs. P0 stays at 5, P1 stays at 20. Neither player's life total changes.
- **Phase:** Phase 8
- **Ticket:** NEW — life exchange blocked by can't-gain
- **Tags:** dependency, replacement-effects
- **Cross-ref:** Rules 119.7 (can't gain life) and 119.8 (can't lose life). Symmetric case: if the *higher* life player has "can't lose life" (119.8), the downward half fails → entire exchange also fails per 701.12a.

**701.12d:** DEFERRED — Phase 8. Zone-to-zone card exchange (Shared Fate type effects). Niche.

**ATOM-701.12d-001** (post-audit addition)
- **Rule:** 701.12d — Mass zone exchange: "Exchange your hand and your graveyard."
- **Mechanism:** Simultaneous batch zone change — all cards in hand go to graveyard, all cards in graveyard go to hand
- **Minimal Board:** P0 has 3 cards in hand (A, B, C). P0 has 5 cards in graveyard (D, E, F, G, H). Effect: "Exchange your hand and your graveyard."
- **Action:** Effect resolves.
- **Expected Result:** A, B, C are now in P0's graveyard. D, E, F, G, H are now in P0's hand. Zone changes are simultaneous. No "discard" or "draw" events fire — these are direct zone changes.
- **Phase:** Phase 8
- **Ticket:** NEW — mass zone exchange

**701.12e:** DEFERRED — Phase 8. Attachment transfer during zone exchange. Niche.

**701.12f:** DEFERRED — Phase 8. Empty-zone exchange. Niche.

**ATOM-701.12g-001**
- **Rule:** 701.12g — Exchange two numerical values: each becomes the other. If life total → gain/loss. If P/T → continuous effect (613.4b). Does NOT apply to "switch P/T."
- **Mechanism:** Numerical value exchange — routes through appropriate system per value type
- **Minimal Board:** Creature C is 2/5. Effect "exchange C's power and toughness" — wait, 701.12g explicitly says this rule doesn't apply to switch P/T. So: P0 at 15 life, Creature C has power 4. Effect "exchange P0's life total and C's power."
- **Action:** Effect resolves.
- **Expected Result:** P0 loses 11 life (to 4). C gets a continuous effect setting power to 15 (L7b). These are distinct mechanism paths.
- **Phase:** Phase 8
- **Ticket:** NEW — numerical value exchange

**701.12h:** DEFERRED — Phase 8. Exchange of Words text-box exchange. Single card, niche.

### 701.13 — Exile

**701.13a:** ALREADY-IMPLEMENTED — "To exile an object, move it to the exile zone." Implemented via `move_object` with destination=Exile zone. Used by multiple effects already. Tested in Phase 2.

### 701.14 — Fight

**ATOM-701.14a-001**
- **Rule:** 701.14a — When two creatures fight, each deals damage equal to its power to the other.
- **Mechanism:** `fight(game, creature_a, creature_b)` — mutual damage based on power
- **Minimal Board:** Creature A (3/3). Creature B (2/4).
- **Action:** A and B fight.
- **Expected Result:** A takes 2 damage (from B's power). B takes 3 damage (from A's power). Both simultaneously.
- **Phase:** Phase 8 (fight keyword action)
- **Ticket:** NEW — fight primitive

**ATOM-701.14b-001**
- **Rule:** 701.14b — If one or both creatures are no longer on the battlefield or no longer creatures, neither fights or deals damage.
- **Mechanism:** Fight validity check at resolution
- **Minimal Board:** Creature A and Creature B targeted by "A fights B." Before resolution, A is destroyed.
- **Action:** Fight effect resolves.
- **Expected Result:** Neither creature fights or deals damage. B takes no damage.
- **Phase:** Phase 8
- **Ticket:** NEW — fight validity check

**ATOM-701.14b-002**
- **Rule:** 701.14b — If one is an illegal target at resolution, neither fights.
- **Mechanism:** Fight target legality check
- **Minimal Board:** Creature A and Creature B targeted. B gains hexproof from the fight spell's controller before resolution.
- **Action:** Fight resolves.
- **Expected Result:** B is illegal target. Neither deals damage.
- **Phase:** Phase 8
- **Ticket:** NEW — fight illegal target

**ATOM-701.14c-001**
- **Rule:** 701.14c — A creature fighting itself deals damage equal to twice its power to itself.
- **Mechanism:** Self-fight = double power self-damage
- **Minimal Board:** Creature A (4/6). Effect "A fights itself."
- **Action:** Effect resolves.
- **Expected Result:** A takes 8 damage (2 × 4 power).
- **Phase:** Phase 8
- **Ticket:** NEW — self-fight

**ATOM-701.14d-001**
- **Rule:** 701.14d — Fight damage is NOT combat damage.
- **Mechanism:** Fight damage tagged as non-combat
- **Minimal Board:** Creature A (3/3) has triggered ability "Whenever this creature deals combat damage to a creature, draw a card." Creature B (2/2).
- **Action:** A and B fight.
- **Expected Result:** A deals 3 non-combat damage to B. B deals 2 non-combat damage to A. The "deals combat damage" trigger does NOT fire (fight damage is non-combat). No card is drawn.
- **Phase:** Phase 8
- **Ticket:** NEW — fight non-combat damage flag

--- End of Chunk 3 ---

## Chunk 4 — Rules 701.15–701.21 (Goad, Investigate, Mill, Play, Regenerate, Reveal, Sacrifice)

### 701.15 — Goad

**701.15 + 701.15a–d:** DEFERRED — Phase 9 (multiplayer). Goad is a multiplayer-centric mechanic (attack a player other than the goader). Requires multiplayer attack-target selection. One-line entries:
- **701.15a:** DEFERRED Phase 9 — goad designation creation.
- **701.15b:** DEFERRED Phase 9 — goaded attack requirements.
- **701.15c:** DEFERRED Phase 9 — multiple goad sources create additional requirements.
- **701.15d:** DEFERRED Phase 9 — same player re-goading has no additional effect.

### 701.16 — Investigate

**ATOM-701.16a-001**
- **Rule:** 701.16a — "Investigate" means "Create a Clue token." (See 111.10f.)
- **Mechanism:** Investigate = `create_token(game, controller, CLUE_TOKEN_DEF)`
- **Minimal Board:** P0 controls nothing. Effect "Investigate."
- **Action:** Effect resolves.
- **Expected Result:** P0 gets a Clue token (colorless artifact with "{2}, Sacrifice this artifact: Draw a card.").
- **Phase:** Phase 8 (token creation + predefined token types)
- **Ticket:** NEW — investigate / Clue token

### 701.17 — Mill

**ATOM-701.17a-001**
- **Rule:** 701.17a — To mill N, put top N cards of library into graveyard.
- **Mechanism:** `mill(game, player_id, n)` — moves top N from library to graveyard
- **Minimal Board:** P0 has 10 cards in library.
- **Action:** P0 mills 3.
- **Expected Result:** Top 3 cards of P0's library are in P0's graveyard (in order: top card entered GY first). Library has 7 cards.
- **Phase:** Phase 8 (mill keyword action)
- **Ticket:** NEW — mill primitive

**ATOM-701.17b-001**
- **Rule:** 701.17b — Can't mill more cards than in library. If instructed to, mill as many as possible. Can't choose to mill more than library size. Can't pay a cost that requires milling more than library size.
- **Mechanism:** Mill capped to library size; cost payment check
- **Minimal Board:** P0 has 2 cards in library. Effect says "mill 5."
- **Action:** Effect resolves.
- **Expected Result:** P0 mills 2 (all remaining). Library is empty. No error.
- **Phase:** Phase 8
- **Ticket:** NEW — mill cap to library size

**ATOM-701.17b-002**
- **Rule:** 701.17b — Can't pay a cost requiring milling N if library has fewer than N cards.
- **Mechanism:** Mill cost legality check
- **Minimal Board:** P0 has 2 cards in library. Ability cost: "Mill 4."
- **Action:** P0 attempts to activate the ability.
- **Expected Result:** Cost is unpayable. Activation is illegal.
- **Phase:** Phase 8
- **Ticket:** NEW — mill cost legality

**701.17c:** DEFERRED — Phase 8. Milled card tracking in public zones. Niche reference-tracking for effects like "if a creature card was milled this way."

**701.17d:** DEFERRED — Phase 8. Multi-milled-card ability references. Niche rules for "each milled card" resolution.

### 701.18 — Play

**701.18a:** ALREADY-IMPLEMENTED — "To play a land means to put it onto the battlefield from the zone it's in." Implemented in `engine/zones.rs` (`play_land`). Timing guards (main phase, active player, empty stack, haven't played land this turn) enforced. Tested in Phase 1/Pre-Phase 3.

**701.18b:** DEFERRED — Phase 8. "To play a card means to play that card as a land or cast it as a spell." Routing logic. **Architectural note (post-audit):** Effects like "Exile the top 3 cards of your library. You may play them this turn." grant permission to both play lands and cast spells from exile. Rather than a dedicated `PlayPermission` enum on every object, this is better modeled as a **continuous effect** generated by the exiling spell/ability that modifies the legality check:
- The casting pipeline already queries `can_cast_from(zone, card_id)` — this check consults active continuous effects.
- The land-play pipeline queries `can_play_land_from(zone, card_id)` — same pattern.
- A "play" permission is simply a continuous effect that answers `true` to *both* queries for the affected cards.
- Alternative costs (e.g., "you may cast it without paying its mana cost") are stored on the continuous effect, not on the card.
This avoids adding a new enum to every game object and instead leverages the existing continuous effect infrastructure (Phase 5-Layers).

**701.18c:** PURE-DEF — "Play" as in "play the game" (e.g., "Play with top card revealed"). No independent mechanical consequence for the keyword action definition.

**701.18d:** PURE-DEF — Errata: old "playing" a spell → now "casting." No mechanical consequence.

**701.18e:** PURE-DEF — Errata: old "playing" an ability → now "activating." No mechanical consequence.

### 701.19 — Regenerate

**ATOM-701.19a-001**
- **Rule:** 701.19a — Resolving spell/ability regeneration creates a replacement effect (shield) protecting the permanent the next time it would be destroyed this turn. Replacement: remove all damage, tap it, remove from combat.
- **Mechanism:** Regeneration shield as one-shot replacement effect on destroy
- **Minimal Board:** Creature C (3/3) with 2 damage. P0 activates "Regenerate C" — shield is created.
- **Action:** Later, effect says "Destroy C."
- **Expected Result:** Shield activates: all damage removed (2→0), C is tapped, C removed from combat if applicable. C stays on battlefield. Shield is consumed (one-use per creation).
- **Phase:** Phase 6 (replacement effects)
- **Ticket:** NEW — regeneration shield (one-shot replacement)
- **Tags:** replacement-effects

**ATOM-701.19a-002**
- **Rule:** 701.19a — Shield only protects once ("the next time"). Second destruction in same turn is not prevented.
- **Mechanism:** Shield consumption — single use
- **Minimal Board:** Creature C has one regeneration shield. C would be destroyed twice this turn (e.g., two sequential destroy effects).
- **Action:** First destroy → shield consumed. Second destroy → no shield.
- **Expected Result:** First destroy is replaced (C survives, tapped, damage removed). Second destroy succeeds — C goes to graveyard.
- **Phase:** Phase 6
- **Ticket:** NEW — regeneration shield single-use

**ATOM-701.19b-001**
- **Rule:** 701.19b — Static ability regeneration replaces destruction EACH TIME (not one-shot). "Instead" replacement pattern.
- **Mechanism:** Persistent regeneration replacement (static ability)
- **Minimal Board:** Mossbridge Troll (5/5, static ability: "If this creature would be destroyed, regenerate it."). Two sequential destroy effects target it.
- **Action:** First destroy resolves. Then second destroy resolves.
- **Expected Result:** Both destructions are replaced: each time, all damage removed, Troll is tapped, removed from combat. Unlike 701.19a (one-shot shield), this static replacement applies every time — it is never consumed.
- **Phase:** Phase 6
- **Ticket:** NEW — static regeneration replacement

**ATOM-701.19c-001**
- **Rule:** 701.19c — "Can't be regenerated" doesn't prevent creating shields — it prevents them from applying.
- **Mechanism:** Anti-regeneration flag blocks shield application, not creation
- **Minimal Board:** Creature C. Effect "C can't be regenerated this turn." P0 activates "Regenerate C" — shield is created (ability resolves normally).
- **Action:** Effect "Destroy C."
- **Expected Result:** Shield exists but can't apply (C can't be regenerated). C is destroyed normally (goes to graveyard).
- **Phase:** Phase 6
- **Ticket:** NEW — can't-be-regenerated blocks shield application

### 701.20 — Reveal

**ATOM-701.20a-001**
- **Rule:** 701.20a — To reveal a card, show it to all players. Remains revealed as long as relevant to the effect.
- **Mechanism:** `reveal(game, card_ids)` — marks cards as revealed, visible to all players
- **Minimal Board:** P0 has cards in hand (hidden zone). Effect "Reveal the top card of your library."
- **Action:** Effect resolves.
- **Expected Result:** Top card of library is marked as revealed. All players can see it. It remains revealed until the effect finishes processing.
- **Phase:** Phase 8 (reveal keyword action)
- **Ticket:** NEW — reveal primitive

**701.20b:** PURE-DEF — Revealing doesn't cause a zone change. No independent test.

**701.20c:** PURE-DEF — An already-revealed card can be revealed again. No independent test.

**701.20d:** DEFERRED — Phase 8. Revealed cards that are shuffled stop being revealed and become new objects. Niche interaction.

**701.20e:** PURE-DEF — "Look at" follows same rules as reveal but shown only to specified player. No independent test beyond reveal mechanism.

**Architectural note (post-audit) — Momentary vs persistent reveal:** Two distinct patterns exist:
1. **Momentary reveal:** "Reveal the top card of your library" — fire-and-forget event during effect resolution. Card visibility resets when the effect finishes.
2. **Persistent reveal:** "Play with the top card of your library revealed" — static ability creating a continuous effect. Card remains revealed as long as the condition holds (checked every time visibility is queried).
The `reveal()` primitive handles case 1. Case 2 is a continuous effect, but it does NOT operate through the 7-layer model. Per rule 613.1, layers determine "the values of an object's *characteristics*" — and being revealed is not a characteristic (characteristics are name, mana cost, color, type, subtype, supertype, rules text, abilities, power, toughness, loyalty). Per rule 613.10, continuous effects that affect *players* rather than object characteristics are applied in timestamp order after layer resolution. Persistent reveal falls into this category (it modifies what a player can see, not what an object is). The engine needs a separate visibility query (`is_revealed(card_id) -> bool`) that checks active continuous effects granting reveal status, applied in timestamp order per 613.10. Both cases share the same UI display mechanism but have fundamentally different lifetimes.

### 701.21 — Sacrifice

**ATOM-701.21a-001**
- **Rule:** 701.21a — To sacrifice a permanent, its controller moves it from battlefield to owner's graveyard. Can't sacrifice non-permanents or things you don't control. Sacrifice is NOT destruction — regeneration/indestructible don't apply.
- **Mechanism:** `sacrifice(game, controller, permanent_id)` — validates ownership, bypasses destroy replacement
- **Minimal Board:** P0 controls Creature C (3/3, indestructible).
- **Action:** P0 sacrifices C.
- **Expected Result:** C moves to owner's graveyard. Indestructible does NOT prevent this (sacrifice ≠ destroy). No regeneration shield applies.
- **Phase:** Phase 5-Pre (sacrifice as cost — already partially implemented in `pay_costs` for `Cost::SacrificeSelf`)
- **Ticket:** T15 (expanded sacrifice primitive)

**ATOM-701.21a-002**
- **Rule:** 701.21a — Can't sacrifice a permanent you don't control.
- **Mechanism:** Sacrifice controller validation
- **Minimal Board:** P0 controls Creature A. P1 controls Creature B. Effect tells P0 to sacrifice a creature.
- **Action:** P0 attempts to sacrifice B (P1's creature).
- **Expected Result:** Illegal. P0 can only sacrifice permanents P0 controls. Only A is a valid choice.
- **Phase:** Phase 5-Pre
- **Ticket:** T15

**ATOM-701.21a-003**
- **Rule:** 701.21a — Can't sacrifice something that isn't a permanent.
- **Mechanism:** Sacrifice zone validation — must be on battlefield
- **Minimal Board:** P0 has a card in hand. Effect says "sacrifice a creature."
- **Action:** P0 has no creatures on battlefield.
- **Expected Result:** No valid sacrifice targets. If this is a cost, it's unpayable. If it's an effect, the instruction is impossible and skipped.
- **Phase:** Phase 5-Pre
- **Ticket:** T15

--- End of Chunk 4 ---

## Chunk 5 — Rules 701.22–701.28 (Scry, Search, Shuffle, Surveil, Tap/Untap, Transform, Convert)

### 701.22 — Scry

**ATOM-701.22a-001**
- **Rule:** 701.22a — To "scry N" means look at top N cards, put any number on bottom in any order, rest on top in any order.
- **Mechanism:** `scry(game, player_id, n, decisions)` — DP chooses which go bottom vs top and ordering
- **Minimal Board:** P0 has 5 cards in library. Effect "Scry 3."
- **Action:** P0 looks at top 3. Chooses to put 1 on bottom, keep 2 on top (reordered).
- **Expected Result:** Chosen card on bottom of library. Two remaining on top in player-specified order. Library size unchanged.
- **Phase:** Phase 8 (scry keyword action)
- **Ticket:** NEW — scry primitive

**ATOM-701.22b-001**
- **Rule:** 701.22b — Scry 0 = no scry event. "Whenever a player scries" doesn't trigger.
- **Mechanism:** Scry 0 short-circuit
- **Minimal Board:** P0 has a permanent with "Whenever you scry, draw a card." Effect "Scry 0."
- **Action:** Effect resolves.
- **Expected Result:** No scry event occurs. The "whenever you scry" trigger does NOT fire. No cards are looked at.
- **Phase:** Phase 8
- **Ticket:** NEW — scry 0 no-op

**701.22c:** DEFERRED — Phase 8. Simultaneous scry with APNAP ordering. **Post-audit reclassification:** Not multiplayer-specific — "Each player scries 2" is a valid 2-player effect. Active player scries first, then non-active player. The existing turn structure already tracks APNAP order.

**701.22d:** PURE-DEF — "Whenever a player scries" triggers after the full scry process completes. Trigger timing rule — covered by the triggered ability system (Phase 7).

### 701.23 — Search

**ATOM-701.23a-001**
- **Rule:** 701.23a — To search a zone, look at all cards and find one matching the description.
- **Mechanism:** `search(game, player_id, zone, filter, decisions)` — player sees all cards, DP picks matching card(s)
- **Minimal Board:** P0 has 20 cards in library including 3 basic lands. Effect "Search your library for a basic land card."
- **Action:** P0 searches library.
- **Expected Result:** P0 can see all 20 cards. P0 selects one of the 3 basic lands.
- **Phase:** Phase 8 (search keyword action)
- **Ticket:** NEW — search primitive

**ATOM-701.23b-001**
- **Rule:** 701.23b — When searching a hidden zone for cards with a stated quality, player isn't required to find them even if present.
- **Mechanism:** Optional find in hidden zone search
- **Minimal Board:** P0 has 20 cards in library including 3 basic lands. Effect "Search your library for a basic land card."
- **Action:** P0 searches but chooses to find nothing (fail to find).
- **Expected Result:** Legal — P0 finds no card. Library is shuffled (per typical search effects). No card moves.
- **Phase:** Phase 8
- **Ticket:** NEW — fail-to-find in hidden zone

**701.23c:** PURE-DEF — Searching for undefined quality = can search but finds nothing. Covered by 701.23b logic (fail-to-find).

**ATOM-701.23d-001**
- **Rule:** 701.23d — Searching for a quantity ("a card", "three cards") in hidden zone: MUST find that many (or as many as possible).
- **Mechanism:** Mandatory find for quantity-based search
- **Minimal Board:** P0 has 20 cards in library. Effect "Search your library for a card" (no quality restriction).
- **Action:** P0 searches.
- **Expected Result:** P0 MUST find exactly one card. Cannot fail to find (no stated quality, only a quantity).
- **Phase:** Phase 8
- **Ticket:** NEW — mandatory quantity search

**701.23e:** PURE-DEF — If effect doesn't say "reveal," found cards aren't revealed. Information rule, no independent test.

**701.23f:** DEFERRED — Phase 8. Search-portion replacement (Aven Mindcensor). Niche interaction with replacement effects.

**701.23g:** DEFERRED — Phase 8. Optional search with impossible follow-up actions still allows searching. Niche.

**701.23h:** DEFERRED — Phase 8. Multiple searches of same library = single search event. Niche optimization rule.

**701.23i:** DEFERRED — Phase 8. Simultaneous search with APNAP ordering. **Post-audit reclassification:** Same reasoning as 701.22c — "Each player searches their library" is valid in 2-player.

**701.23j:** DEFERRED — Phase 8. Search outside the game (wish effects). Requires outside-game card pool.

### 701.24 — Shuffle

**701.24a:** ALREADY-IMPLEMENTED — "To shuffle a library, randomize the cards." Implemented in `engine/zones.rs` or equivalent. Used after search effects and at game start. Tested in Phase 1.

**ATOM-701.24b-001**
- **Rule:** 701.24b — When searching then shuffling, found cards are excluded from the shuffle (they stay in position or move to destination), then the rest is shuffled.
- **Mechanism:** Search-then-shuffle excludes found cards from randomization
- **Minimal Board:** P0 searches library, finds Card X, effect says "Shuffle your library, then put Card X on top."
- **Action:** Search resolves. Card X is set aside. Library is shuffled. Card X placed on top.
- **Expected Result:** Card X is on top of library. All other cards are randomized below it. (Note: the card text order matters — "shuffle then put on top" ensures X isn't shuffled away. "Put on top then shuffle" would shuffle X into the library.)
- **Phase:** Phase 8 (search + shuffle interaction)
- **Ticket:** NEW — search-shuffle card exclusion

**701.24c:** PURE-DEF — Shuffle-into-library triggers even if the object isn't in the expected zone. No independent test needed. **Explanation (post-audit):** The engine handles this naturally because `shuffle()` is an independent instruction in the effect sequence, not conditional on the zone change succeeding. Example: "Shuffle Target Card into its owner's library" where Target Card already left the graveyard → (1) zone change attempt no-ops (card not there), (2) shuffle happens anyway. No additional code needed.

**701.24d:** PURE-DEF — Shuffle-a-set-into triggers even if the set is empty. No independent test.

**701.24e:** PURE-DEF — Shuffle of 0 or 1 card library still triggers "whenever shuffled." Trigger-system concern (Phase 7).

**701.24f:** DEFERRED — Phase 7 (triggered abilities). Multiple simultaneous shuffles trigger that many times.

**701.24g:** DEFERRED — Phase 8. Simultaneous shuffle + positional placement (Darksteel Colossus + Gravebane Zombie). Niche corner case.

### 701.25 — Surveil

**ATOM-701.25a-001**
- **Rule:** 701.25a — To "surveil N" means look at top N, put any number into graveyard, rest on top in any order.
- **Mechanism:** `surveil(game, player_id, n, decisions)` — like scry but graveyard instead of bottom
- **Minimal Board:** P0 has 5 cards in library. Effect "Surveil 2."
- **Action:** P0 looks at top 2. Puts 1 in graveyard, keeps 1 on top.
- **Expected Result:** Chosen card in P0's graveyard. Other card on top of library.
- **Phase:** Phase 8 (surveil keyword action)
- **Ticket:** NEW — surveil primitive

**701.25b:** DEFERRED — Phase 8. Additional-card surveil variant. Niche.

**ATOM-701.25c-001**
- **Rule:** 701.25c — Surveil 0 = no surveil event. "Whenever you surveil" doesn't trigger.
- **Mechanism:** Surveil 0 short-circuit (same pattern as scry 0)
- **Minimal Board:** P0 has "Whenever you surveil, draw a card." Effect "Surveil 0."
- **Action:** Effect resolves.
- **Expected Result:** No surveil event. Trigger does NOT fire.
- **Phase:** Phase 8
- **Ticket:** NEW — surveil 0 no-op

**701.25d:** PURE-DEF — "Whenever you surveil" triggers after the full process. Trigger timing — Phase 7 concern.

### 701.26 — Tap and Untap

**701.26a:** ALREADY-IMPLEMENTED — "To tap a permanent, turn it sideways. Only untapped permanents can be tapped." Implemented via `GameAction::Tap` in `engine/actions.rs`. Tapped status tracked in `BattlefieldEntity`. `Cost::Tap` checks untapped prerequisite. Tested extensively in Phases 1–4.5.

**701.26b:** ALREADY-IMPLEMENTED — "To untap a permanent, rotate it back. Only tapped permanents can be untapped." Implemented via `GameAction::Untap` in `engine/actions.rs`. Untap step untaps all of active player's permanents. Tested in Phase 1.

### 701.27 — Transform

**701.27 + 701.27a–g:** DEFERRED — Phase 9 (double-faced cards). Transform requires DFC infrastructure (front/back face tracking, physical face toggle). One-line entries:
- **701.27a:** DEFERRED Phase 9 — only DFCs/double-faced tokens can transform.
- **701.27b:** DEFERRED Phase 9 — transform ≠ turn face down (different game actions).
- **701.27c:** DEFERRED Phase 9 — non-DFC transform instruction = nothing happens.
- **701.27d:** DEFERRED Phase 9 — can't transform into instant/sorcery face.
- **701.27e:** DEFERRED Phase 9 — "transforms into" trigger fires on transform or convert.
- **701.27f:** DEFERRED Phase 9 — once-per-stack-resolution transform guard.
- **701.27g:** DEFERRED Phase 9 — "transformed permanent" = DFC with back face up.

### 701.28 — Convert

**701.28 + 701.28a–f:** DEFERRED — Phase 9 (double-faced cards). Convert follows same rules as transform (701.27a–f apply). One-line entries:
- **701.28a:** DEFERRED Phase 9 — convert follows transform rules.
- **701.28b:** DEFERRED Phase 9 — convert ≠ turn face down.
- **701.28c:** DEFERRED Phase 9 — non-DFC convert = nothing.
- **701.28d:** DEFERRED Phase 9 — can't convert into instant/sorcery face.
- **701.28e:** DEFERRED Phase 9 — once-per-stack-resolution convert guard.
- **701.28f:** DEFERRED Phase 9 — "can't transform" also prevents converting.

--- End of Chunk 5 ---

## Chunk 6 — Rules 701.29–701.42 (Fateseal, Clash, Planeswalk, Set in Motion, Abandon, Proliferate, Detain, Populate, Monstrosity, Vote, Bolster, Manifest, Support, Meld)

### 701.29 — Fateseal

**Post-audit reclassification:** If scry is implemented, fateseal is a one-line wrapper: `scry(game, opponent_id, n, decisions)` instead of `scry(game, self_id, n, decisions)`. Reclassified from DEFERRED to ATOM.

**ATOM-701.29a-001**
- **Rule:** 701.29a — "Fateseal N" = look at top N of target opponent's library, put any number on bottom in any order, rest on top in any order.
- **Mechanism:** `fateseal(game, controller, opponent_id, n, decisions)` — identical to scry but on opponent's library
- **Minimal Board:** P0 controls effect "Fateseal 2." P1 has 5 cards in library.
- **Action:** P0 looks at P1's top 2 cards. P0 puts 1 on bottom of P1's library, keeps 1 on top.
- **Expected Result:** P1's library reordered per P0's choices. P0 makes all decisions (not P1).
- **Phase:** Phase 8 (trivial once scry exists)
- **Ticket:** NEW — fateseal (scry wrapper on opponent's library)

### 701.30 — Clash

**701.30 + 701.30a–d:** DEFERRED — Phase 8. Clash involves both players revealing top cards, comparing mana values, optional bottom-of-library placement. Niche Lorwyn mechanic. One-line entries:
- **701.30a:** DEFERRED Phase 8 — reveal top card, may put on bottom.
- **701.30b:** DEFERRED Phase 8 — "clash with an opponent" = choose opponent, both clash.
- **701.30c:** DEFERRED Phase 8 — simultaneous reveal, APNAP ordering for decisions.
- **701.30d:** DEFERRED Phase 8 — higher mana value wins the clash.

### 701.31 — Planeswalk

**701.31 + 701.31a–d:** OUT-OF-SCOPE — Planechase format. Planechase is permanently out of scope per design doc.
- **701.31a:** OUT-OF-SCOPE — Planechase only.
- **701.31b:** OUT-OF-SCOPE — planar deck manipulation.
- **701.31c:** OUT-OF-SCOPE — planeswalking triggers.
- **701.31d:** OUT-OF-SCOPE — plane terminology.

### 701.32 — Set in Motion

**701.32 + 701.32a–c:** OUT-OF-SCOPE — Archenemy format (team format, permanently out of scope).
- **701.32a:** OUT-OF-SCOPE — scheme cards, Archenemy only.
- **701.32b:** OUT-OF-SCOPE — scheme activation.
- **701.32c:** OUT-OF-SCOPE — sequential scheme setting.

### 701.33 — Abandon

**701.33 + 701.33a–b:** OUT-OF-SCOPE — Archenemy format.
- **701.33a:** OUT-OF-SCOPE — ongoing scheme abandonment.
- **701.33b:** OUT-OF-SCOPE — scheme flip and return.

### 701.34 — Proliferate

**ATOM-701.34a-001**
- **Rule:** 701.34a — Proliferate: choose any number of permanents/players with a counter, give each one additional counter of each kind it already has.
- **Mechanism:** `proliferate(game, chooser, decisions)` — DP selects subset of counter-bearing objects/players, each gets +1 of each counter kind
- **Minimal Board:** Creature A has 2 +1/+1 counters. Creature B has 1 -1/-1 counter. P1 has 2 poison counters. P0 proliferates, choosing A, B, and P1.
- **Action:** Proliferate resolves.
- **Expected Result:** A gets +1 +1/+1 counter (now 3). B gets +1 -1/-1 counter (now 2). P1 gets +1 poison counter (now 3). (Note: +1/+1 and -1/-1 counters are on separate permanents to avoid SBA annihilation confounding the test.)
- **Phase:** Phase 8 (proliferate keyword action)
- **Ticket:** NEW — proliferate primitive

**701.34b:** OUT-OF-SCOPE — Two-Headed Giant poison counter sharing. THG is a team format, permanently out of scope per design doc.

### 701.35 — Detain

**ATOM-701.35a-001**
- **Rule:** 701.35a — Detain a permanent: until controller's next turn, it can't attack, block, or activate abilities.
- **Mechanism:** Detain designation with duration-based restrictions
- **Minimal Board:** P0 casts a spell that detains Creature C (controlled by P1).
- **Action:** Detain resolves.
- **Expected Result:** Until P0's next turn: C can't attack, can't block, and its activated abilities can't be activated. Static and triggered abilities still function.
- **Phase:** Phase 8 (detain keyword action)
- **Ticket:** NEW — detain primitive

### 701.36 — Populate

**ATOM-701.36a-001**
- **Rule:** 701.36a — Populate: choose a creature token you control, create a copy of it.
- **Mechanism:** `populate(game, controller, decisions)` — DP picks creature token, engine creates copy token
- **Minimal Board:** P0 controls Token A (3/3 green Beast) and Token B (1/1 white Soldier). P0 populates.
- **Action:** P0 chooses Token A.
- **Expected Result:** New 3/3 green Beast token created under P0's control. Token A unchanged. Token B unchanged.
- **Phase:** Phase 8 (populate keyword action — depends on token copy)
- **Ticket:** NEW — populate primitive

**ATOM-701.36b-001**
- **Rule:** 701.36b — If you control no creature tokens, populate does nothing.
- **Mechanism:** Populate with no valid choices = no-op
- **Minimal Board:** P0 controls only non-token creatures. Effect "Populate."
- **Action:** Effect resolves.
- **Expected Result:** Nothing happens. No token created.
- **Phase:** Phase 8
- **Ticket:** NEW — populate empty no-op

### 701.37 — Monstrosity

**ATOM-701.37a-001**
- **Rule:** 701.37a — "Monstrosity N" = if not monstrous, put N +1/+1 counters and become monstrous.
- **Mechanism:** Monstrosity action with designation check
- **Minimal Board:** Creature C (not monstrous). C has "Monstrosity 3."
- **Action:** Activate monstrosity.
- **Expected Result:** C gets 3 +1/+1 counters. C becomes monstrous.
- **Phase:** Phase 8 (monstrosity keyword action)
- **Ticket:** NEW — monstrosity primitive

**ATOM-701.37a-002**
- **Rule:** 701.37a — If already monstrous, monstrosity does nothing.
- **Mechanism:** Monstrous guard — blocks re-application
- **Minimal Board:** Creature C is already monstrous.
- **Action:** Activate monstrosity again.
- **Expected Result:** Nothing happens. No counters added. C remains monstrous.
- **Phase:** Phase 8
- **Ticket:** NEW — monstrosity guard

**701.37b:** PURE-DEF — Monstrous is a designation (not ability, not copiable). Leaves battlefield → loses monstrous. Covered by zone-change cleanup.

**701.37c:** DEFERRED — Phase 8. "Monstrosity X" with variable X referenced by other abilities. Niche (Polukranos).

### 701.38 — Vote

**701.38 + 701.38a–d:** DEFERRED — Phase 9 (multiplayer). Voting mechanics are multiplayer-centric (Conspiracy/Commander cards). One-line entries:
- **701.38a:** DEFERRED Phase 9 — vote procedure (turn order).
- **701.38b:** DEFERRED Phase 9 — vote choice types.
- **701.38c:** DEFERRED Phase 9 — "voting" refers only to actual votes.
- **701.38d:** DEFERRED Phase 9 — multiple votes simultaneous.

### 701.39 — Bolster

**ATOM-701.39a-001**
- **Rule:** 701.39a — "Bolster N" = choose creature you control with least toughness (or tied), put N +1/+1 counters on it.
- **Mechanism:** `bolster(game, controller, n, decisions)` — find min-toughness creatures, DP picks one if tied, add counters
- **Minimal Board:** P0 controls Creature A (2/1) and Creature B (3/3). Bolster 2.
- **Action:** Effect resolves. A has lowest toughness (1).
- **Expected Result:** A gets 2 +1/+1 counters. A is now 4/3. (No choice needed since A is uniquely lowest.)
- **Phase:** Phase 8 (bolster keyword action)
- **Ticket:** NEW — bolster primitive

**ATOM-701.39a-002**
- **Rule:** 701.39a — Toughness tie: controller chooses among tied creatures.
- **Mechanism:** Bolster tie-breaking via DP
- **Minimal Board:** P0 controls Creature A (2/1) and Creature B (3/1). Bolster 2.
- **Action:** A and B tied at toughness 1. P0 chooses B.
- **Expected Result:** B gets 2 +1/+1 counters. B is now 5/3.
- **Phase:** Phase 8
- **Ticket:** NEW — bolster tie-break

### 701.40 — Manifest: RECLASSIFIED → ATOM (architectural core)

**Why this can't wait:** Manifest, Cloak (701.58), and Manifest Dread (701.62) all share the same face-down permanent infrastructure. Morph (702.37) and Disguise (702.168) also use it. That's **5 mechanics** depending on a single subsystem. Deferring means we either build it 5 times or do a painful retrofit.

**Architectural requirements surfaced by 701.40a–g:**

1. **Face-down characteristics override system (layer 1b).** When `face_down == true`, the layer system must return: name="", subtypes=[], mana_cost=None, text="", P/T=2/2, creature type only. This is a **layer 1b intervention** (face-down status), which occurs AFTER layer 1a (copy effects). Per rule 613.2b: "Face-down spells and permanents have their characteristics modified as defined in rule 708.2." Per 613.2c: after layer 1 completes, the result is the object's **copiable values**.
   **Critical implication for copies (rule 707.2):** Copiable values are modified by the object's face-down status. A copy of a face-down morph creature copies the *face-down* characteristics: nameless, colorless 2/2 creature with no abilities and no mana cost. The copy enters face-up but has those blank characteristics — it does NOT get the morph ability and cannot turn face up. (See CR example: Clone copying face-down Grinning Demon.) This means layer 1a→1b ordering does NOT help copies "retain" morph abilities. Only the actual face-down card (not a copy of it) knows its real front face and can be turned face up via the morph/manifest procedure.
   The oracle module's `get_effective_power/toughness` and `has_keyword` must respect the face-down override.

2. **Face-down origin tracking.** A face-down permanent needs to know *why* it's face-down: manifested, cloaked, morphed, or disguised. This determines which turn-face-up procedures are legal (701.40b vs 702.37e vs 702.168d). Suggests an enum:
   ```rust
   enum FaceDownOrigin { Manifested, Cloaked, Morphed, Disguised }
   ```
   on `BattlefieldEntity`.

3. **Special action: turn face up.** Rule 701.40b says "any time you have priority" — this is a special action (rule 116.2b), not an activated ability. The engine's priority system needs a `SpecialAction::TurnFaceUp` variant. This must validate: (a) card is creature card, (b) card has mana cost, (c) player can pay.

4. **701.40f: ETB prohibition interaction.** If a replacement effect prevents ETB, the card stays in its previous zone face-up. The `manifest()` function should: (1) check if zone change is legal, (2) if yes: set face-down characteristics + move to battlefield, (3) if no: do nothing (card stays in previous zone, face-up, unmodified). No need to prepare face-down characteristics before checking legality.

5. **701.40g: Instant/sorcery face-up guard.** If a manifested instant/sorcery would turn face up, reveal and leave face-down. This is a replacement-like rule that must be checked in the turn-face-up code path.

**ATOM-701.40a-001**
- **Rule:** 701.40a — Manifest a card: face-down 2/2 creature, no text/name/subtypes/cost.
- **Mechanism:** `manifest(game, card_id)` — set face_down=true, origin=Manifested, move to battlefield
- **Minimal Board:** Card C (a 5/5 creature) in P0's library. Effect "Manifest the top card of your library."
- **Action:** Manifest resolves.
- **Expected Result:** C is on battlefield face-down. Effective characteristics: 2/2 creature, no name, no subtypes, no mana cost, no abilities. `face_down == true`, `face_down_origin == Manifested`.
- **Phase:** Backlog — CR 701 keyword actions (was Phase 5-Pre)
- **Ticket:** NEW — face-down permanent subsystem

**ATOM-701.40b-001**
- **Rule:** 701.40b — Turn manifested creature face up by paying mana cost.
- **Mechanism:** `SpecialAction::TurnFaceUp` — validate creature card + mana cost, pay, flip
- **Minimal Board:** Face-down manifested permanent (actually a 4/4 creature with cost {2}{G}{G}).
- **Action:** P0 uses special action, reveals creature card, pays {2}{G}{G}.
- **Expected Result:** Permanent turns face up. Now 4/4 with all printed characteristics. `face_down == false`.
- **Phase:** Backlog — CR 701 keyword actions (was Phase 5-Pre)
- **Ticket:** NEW — special action turn-face-up

**ATOM-701.40b-002**
- **Rule:** 701.40b — Non-creature card can't turn face up via manifest procedure.
- **Minimal Board:** Face-down manifested permanent (actually an instant card underneath).
- **Action:** P0 attempts special action to turn face up.
- **Expected Result:** Illegal — card is not a creature card. Action rejected.
- **Phase:** Backlog — CR 701 keyword actions (was Phase 5-Pre)
- **Ticket:** NEW — manifest turn-face-up guard

**701.40c–d:** DEFERRED until morph/disguise keyword abilities (702.37/702.168) — these sub-rules just say "you can also use morph/disguise procedure." No independent architectural impact beyond the face-down origin enum already designed above.

**701.40e:** ATOM-testable (sequential manifest). Simple loop; no new infrastructure.

**ATOM-701.40f-001**
- **Rule:** 701.40f — ETB prohibition prevents manifest.
- **Minimal Board:** Effect says "Creatures can't enter the battlefield." P0 tries to manifest top card (a creature).
- **Action:** Manifest attempts zone change.
- **Expected Result:** Card stays in previous zone, face-up, characteristics unmodified.
- **Phase:** Phase 6 (requires replacement effect infrastructure)
- **Ticket:** NEW — manifest ETB prohibition

**ATOM-701.40g-001**
- **Rule:** 701.40g — Instant/sorcery manifested permanent can't turn face up.
- **Minimal Board:** Face-down manifested permanent (actually a sorcery card).
- **Action:** Some effect would turn it face up (e.g., Ixidron leaves).
- **Expected Result:** Controller reveals it's a sorcery. Stays face-down. No "turned face up" trigger.
- **Phase:** Backlog — CR 701 keyword actions (was Phase 5-Pre)
- **Ticket:** NEW — instant/sorcery face-up guard

**701.40h:** PURE-DEF — reference to rule 708.

### 701.41 — Support

**ATOM-701.41a-001**
- **Rule:** 701.41a — "Support N" on a permanent = put +1/+1 counter on each of up to N OTHER target creatures. On instant/sorcery = up to N target creatures (including self if applicable).
- **Mechanism:** Support targeting and counter placement
- **Minimal Board:** P0 controls Permanent P with "Support 2." Creatures A, B, C on battlefield (all different from P).
- **Action:** P0 activates Support, targeting A and B.
- **Expected Result:** A gets +1/+1 counter. B gets +1/+1 counter. C unchanged. P unchanged (can't target self for permanent-based support).
- **Phase:** Phase 8 (support keyword action)
- **Ticket:** NEW — support primitive

**ATOM-701.41a-002** (post-audit addition)
- **Rule:** 701.41a — Support N on an instant/sorcery = up to N target creatures (no "other" restriction).
- **Mechanism:** Support without the "other" exclusion
- **Minimal Board:** P0 controls Creature A and Creature B. P0 casts an instant with "Support 2" (not on a permanent).
- **Action:** P0 targets A and B.
- **Expected Result:** A gets +1/+1 counter. B gets +1/+1 counter. Both are valid targets because the instant/sorcery version has no "other" restriction.
- **Phase:** Phase 8
- **Ticket:** NEW — support on nonpermanent spell

### 701.42 — Meld

**701.42 + 701.42a–c:** DEFERRED — Phase 9 (double-faced cards). Meld requires DFC infrastructure + meld pair tracking + combined back-face representation. One-line entries:
- **701.42a:** DEFERRED Phase 9 — meld puts two cards onto battlefield combined back-face-up.
- **701.42b:** DEFERRED Phase 9 — only matching meld pairs can meld.
- **701.42c:** DEFERRED Phase 9 — non-meldable objects stay in current zone.

--- End of Chunk 6 ---

## Chunk 7 — Rules 701.43–701.55 (Exert, Explore, Assemble, Adapt, Amass, Learn, Venture, Connive, Open Attraction, Roll to Visit, Incubate, Ring Tempts, Face Villainous Choice)

### 701.43 — Exert: RECLASSIFIED → ATOM (untap-step integration)

**Why this can't wait:** Exert modifies the untap step, which currently untaps everything unconditionally. We need a **skip-untap tracking system** that multiple mechanics use (exert, freeze effects, Stasis, etc.).

**Architectural requirements:**
1. **Per-permanent untap-skip counter.** `BattlefieldEntity` needs `skip_next_untap: u32` (counter, not boolean, because multiple exerts stack per 701.43b and all expire together). The untap step checks this: if > 0, don't untap and decrement.
2. **701.43d: Optional attack cost.** "You may exert as it attacks" = an optional cost during declare-attackers. The declare-attackers flow needs a hook for optional attack costs.

**ATOM-701.43a-001**
- **Rule:** 701.43a — Exert: doesn't untap during next untap step.
- **Mechanism:** `exert(game, permanent_id)` — sets skip_next_untap += 1
- **Minimal Board:** Creature C (untapped). Effect "Exert C."
- **Action:** Exert resolves. Next untap step arrives.
- **Expected Result:** C does not untap. All other permanents untap normally. On the following untap step, C untaps normally.
- **Phase:** Backlog — CR 701 keyword actions (was Phase 5-Pre)
- **Ticket:** NEW — skip-untap tracking + exert primitive

**ATOM-701.43b-001**
- **Rule:** 701.43b — Can exert already-exerted permanent; both expire same untap step.
- **Minimal Board:** Creature C (already exerted once this turn). Effect "Exert C" again.
- **Action:** Second exert resolves.
- **Expected Result:** C has skip_next_untap = 2. At next untap step, both expire → C doesn't untap. At following untap step, C untaps normally (not held for two turns).
- **Phase:** Backlog — CR 701 keyword actions (was Phase 5-Pre)
- **Ticket:** NEW — exert stacking

**701.43c:** PURE-DEF — non-battlefield objects can't be exerted. Trivial zone check.

**ATOM-701.43d-001**
- **Rule:** 701.43d — "You may exert as it attacks" = optional attack cost.
- **Mechanism:** Declare-attackers optional cost hook
- **Minimal Board:** Creature C with "You may exert C as it attacks. When you do, [effect]." C declared as attacker.
- **Action:** P0 chooses to exert C as part of attack declaration.
- **Expected Result:** C is exerted (skip_next_untap += 1). Linked triggered ability triggers.
- **Phase:** Phase 7 (linked trigger) but exert primitive itself is Phase 5-Pre
- **Ticket:** NEW — optional attack cost hook

**Architectural note (post-audit) — Untap step restriction interactions:**
Exert's `skip_next_untap` is one of several mechanisms that prevent untapping. The untap step must support:
1. **Per-permanent skip flags** (exert): `skip_next_untap > 0` → don't untap, decrement.
2. **Can't-untap restrictions** (static effects like "Creatures with flying don't untap during their controllers' untap steps"): continuous effects creating restriction predicates. Untap step queries all active restrictions per permanent.
3. **Untap-count limits** ("Players can't untap more than one land during their untap steps"): the untap step becomes a player-choice step — DP chooses which N permanents to untap within the limit.
All three are orthogonal and checked independently. A permanent fails to untap if ANY of these blocks it.

**Phasing note:** Mechanism 1 (exert skip flags) is Phase 5-Pre (D17, per-permanent designations). Mechanisms 2 and 3 are **Phase 5-Layers** — they are continuous effects that modify the untap step's behavior. There is no dedicated ticket for untap-step restrictions yet; they should be added as a sub-ticket under the continuous effects umbrella (Phase 5-Layers or a new ticket under Phase 8 for the untap-step rework).

### 701.44 — Explore: RECLASSIFIED → ATOM (multi-step keyword action)

**Why this can't wait:** Explore is a common mechanic (Ixalan, Lost Caverns) with a multi-step procedure: reveal → branch on card type → conditional counter + optional graveyard. This exercises **conditional branching within effect resolution** and **LKI for zone-changed permanents** (701.44c).

**ATOM-701.44a-001**
- **Rule:** 701.44a — Explore: reveal top card. Land → hand. Nonland → +1/+1 counter + may put in graveyard.
- **Mechanism:** `explore(game, permanent_id, decisions)` — reveal, branch, counter/move
- **Minimal Board:** Creature C (2/2). Top of library is a Forest.
- **Action:** C explores.
- **Expected Result:** Forest revealed. Forest goes to P0's hand. C gets no counter. C is still 2/2.
- **Phase:** Phase 8 (keyword action primitive)
- **Ticket:** NEW — explore primitive

**ATOM-701.44a-002**
- **Rule:** 701.44a — Explore: nonland path.
- **Minimal Board:** Creature C (2/2). Top of library is Lightning Bolt.
- **Action:** C explores. P0 chooses to put Bolt into graveyard.
- **Expected Result:** Bolt revealed. C gets +1/+1 counter (now 3/3). Bolt goes to graveyard.
- **Phase:** Phase 8
- **Ticket:** NEW — explore nonland path

**ATOM-701.44a-003**
- **Rule:** 701.44a — Explore: nonland, choose to keep on top.
- **Minimal Board:** Creature C (2/2). Top of library is Lightning Bolt.
- **Action:** C explores. P0 chooses NOT to put Bolt into graveyard.
- **Expected Result:** Bolt revealed. C gets +1/+1 counter (now 3/3). Bolt goes back on top of library.
- **Phase:** Phase 8
- **Ticket:** NEW — explore keep-on-top

**701.44b:** PURE-DEF — "explores" after process complete. Trigger timing. No independent test.

**ATOM-701.44c-001**
- **Rule:** 701.44c — Explore LKI: permanent left battlefield before exploring.
- **Mechanism:** LKI used to determine controller and object identity
- **Minimal Board:** Creature C instructed to explore. Before explore resolves, C is destroyed.
- **Action:** Explore uses LKI of C. Top of library is a land.
- **Expected Result:** Land goes to C's last controller's hand. No counter placed — C doesn't exist on the battlefield, so the +1/+1 counter placement is impossible (verify no counter is placed on any object, not just C). C's controller still performed the explore action (the "explored" trigger fires using LKI).
- **Phase:** Phase 8 (depends on LKI — Phase 5 epoch system)
- **Ticket:** NEW — explore LKI

**701.44d:** DEFERRED Phase 8 — APNAP simultaneous explore. **Post-audit reclassification:** Not multiplayer-specific. A 2-player card could say "When this enters, this and target creature an opponent controls each explore." That's two creatures controlled by different players exploring simultaneously, requiring APNAP ordering per rule 101.4. Same pattern as 701.22c/701.23i.

### 701.45 — Assemble

**701.45a:** OUT-OF-SCOPE — Unstable / silver-bordered. Contraptions are permanently out of scope.

### 701.46 — Adapt

**ATOM-701.46a-001**
- **Rule:** 701.46a — "Adapt N" = if this permanent has no +1/+1 counters, put N +1/+1 counters on it.
- **Mechanism:** Adapt action with counter-presence check
- **Minimal Board:** Creature C (no +1/+1 counters). C has "Adapt 3."
- **Action:** Activate adapt.
- **Expected Result:** C gets 3 +1/+1 counters.
- **Phase:** Phase 8 (adapt keyword action)
- **Ticket:** NEW — adapt primitive

**ATOM-701.46a-002**
- **Rule:** 701.46a — If permanent already has +1/+1 counters, adapt does nothing.
- **Mechanism:** Adapt guard — blocks if any +1/+1 counters present
- **Minimal Board:** Creature C has 1 +1/+1 counter. C has "Adapt 3."
- **Action:** Activate adapt.
- **Expected Result:** Nothing happens. No counters added.
- **Phase:** Phase 8
- **Ticket:** NEW — adapt guard

### 701.47 — Amass: RECLASSIFIED → ATOM (conditional token creation + subtype granting)

**Why this can't wait:** Amass exercises: (1) "if you don't control X, create one" pattern (used by many mechanics), (2) subtype granting (layer 4 effect), (3) counter placement. All three are core infrastructure.

**ATOM-701.47a-001**
- **Rule:** 701.47a — Amass: no Army → create 0/0 Army token, choose Army, add counters, grant subtype.
- **Mechanism:** `amass(game, controller, subtype, n, decisions)` — conditional create, choose, counters, subtype
- **Minimal Board:** P0 controls no Army creatures. Effect "Amass Zombies 3."
- **Action:** Amass resolves.
- **Expected Result:** 0/0 black Zombie Army token created. 3 +1/+1 counters placed on it (now 3/3). Token has subtypes: Zombie, Army.
- **Phase:** Phase 8 (but architectural implications for token creation + layer 4)
- **Ticket:** NEW — amass primitive

**ATOM-701.47a-002**
- **Rule:** 701.47a — Amass with existing Army: no new token, counters on existing.
- **Minimal Board:** P0 controls Army creature A (2/2 Zombie Army with 2 +1/+1 counters). Effect "Amass Zombies 2."
- **Action:** Amass resolves. P0 chooses A (only Army).
- **Expected Result:** No new token. A gets 2 more +1/+1 counters (now 4/4). A already has Zombie subtype, no change.
- **Phase:** Phase 8
- **Ticket:** NEW — amass existing army

**ATOM-701.47a-003** (post-audit: reworked)
- **Rule:** 701.47a — Amass grants subtype if Army doesn't already have it.
- **Minimal Board:** P0 controls Army creature A (0/0 Army with 2 +1/+1 counters, no Zombie subtype — e.g., previously amassed with "Amass Orcs 2"). Effect "Amass Zombies 2."
- **Action:** Amass resolves. A is the only Army, so it is chosen automatically.
- **Expected Result:** A gets 2 more +1/+1 counters (now 4/4). A gains the Zombie subtype in addition to its other types (layer 4 type-changing effect). A is now an Orc Zombie Army.
- **Phase:** Phase 8
- **Ticket:** NEW — amass subtype grant
- **Note (post-audit):** A player can only control multiple Armies if a token doubler (e.g., Anointed Procession) duplicates the initial 0/0 token. The choice of which Army to place counters on only arises in that composition scenario, which is a token-doubler integration test, not an amass-specific test.

**701.47b–c:** PURE-DEF — trigger timing and "the Army you amassed" reference. No independent test.
**701.47d:** PURE-DEF — errata note.

### 701.48 — Learn

**701.48a:** DEFERRED — Phase 8. Learn = optional discard-then-draw or get Lesson from outside game. Requires outside-game card pool.

### 701.49 — Venture into the Dungeon: Remains DEFERRED Phase 9, with architectural notes

**Reasoning for keeping DEFERRED:** Dungeons require genuinely new infrastructure that doesn't overlap with other mechanics:
- **Dungeon cards as non-object zone occupants** in the command zone (they aren't permanents, spells, or cards in the traditional sense — more like emblems with rooms)
- **Venture marker** = per-player state tracking a position within a graph (rooms + directed edges)
- **Room triggered abilities** = triggered abilities on non-permanent objects
- **"Outside the game" card pool** for dungeon selection

**Architectural sketch for future implementation:**
1. `DungeonCard` struct: `rooms: Vec<Room>`, `edges: Vec<(RoomIdx, RoomIdx)>`, `name: String`
2. `Room` struct: `name: String`, `trigger_effect: Effect`, `is_bottommost: bool`
3. Per-player state: `active_dungeon: Option<ObjectId>`, `venture_marker: Option<RoomIdx>`
4. Venture action: if no dungeon → choose from outside game, place marker on room 0. If has dungeon → advance along edge (DP choice if multiple). If bottommost → complete (trigger + remove), then choose new dungeon.
5. Room triggers fire via delta log: delta = `VentureMarkerMoved { player, dungeon, from_room, to_room }`.

- **701.49a:** DEFERRED Phase 9 — choose dungeon from outside game, place venture marker.
- **701.49b:** DEFERRED Phase 9 — advance marker to adjacent room.
- **701.49c:** DEFERRED Phase 9 — bottommost room → complete dungeon, start new one.
- **701.49d:** DEFERRED Phase 9 — "venture into [quality]" variant.

**Phase:** Phase 9 — genuinely requires outside-game card pool + graph traversal infrastructure. No existing subsystem covers this. Low card count (3 dungeons + ~20 venture cards).

### 701.50 — Connive: RECLASSIFIED → ATOM (draw-discard-conditional-counter)

**Why this can't wait:** Connive is draw → discard → conditional counter, a clean multi-step keyword action. It exercises the same LKI pattern as Explore (701.50c) and the "connive N" variant (701.50e) tests parameterized keyword actions.

**ATOM-701.50a-001**
- **Rule:** 701.50a — Connive: draw, discard. Nonland discarded → +1/+1 counter.
- **Mechanism:** `connive(game, permanent_id, n, decisions)` — draw n, discard n, count nonlands, add counters
- **Minimal Board:** Creature C (2/2). P0 has 2 cards in hand. C connives (N=1).
- **Action:** P0 draws 1 card. P0 discards 1 card (a creature card — nonland).
- **Expected Result:** C gets 1 +1/+1 counter (now 3/3).
- **Phase:** Phase 8
- **Ticket:** NEW — connive primitive

**ATOM-701.50a-002**
- **Rule:** 701.50a — Connive: discard a land → no counter.
- **Minimal Board:** Creature C (2/2). P0 connives. P0 discards a Forest.
- **Expected Result:** C gets no counter. C remains 2/2.
- **Phase:** Phase 8
- **Ticket:** NEW — connive land discard

**ATOM-701.50e-001**
- **Rule:** 701.50e — Connive N: draw N, discard N, counters = nonland count.
- **Minimal Board:** Creature C (1/1). C connives 3. P0 draws 3, discards 3 (2 nonland, 1 land).
- **Expected Result:** C gets 2 +1/+1 counters (now 3/3).
- **Phase:** Phase 8
- **Ticket:** NEW — connive N variant

**701.50b:** PURE-DEF — trigger timing.
**701.50c:** Same LKI pattern as explore (701.44c). Test mirrors ATOM-701.44c-001.
**701.50d:** DEFERRED Phase 8 — APNAP simultaneous connive. **Post-audit reclassification:** Not multiplayer-specific — "Two target creatures each connive" is valid in 2-player with creatures controlled by different players.

### 701.51 — Open an Attraction

**701.51 + 701.51a–c:** OUT-OF-SCOPE — Attraction cards (Unfinity). Attraction decks are permanently out of scope.
- **701.51a:** OUT-OF-SCOPE — Attraction deck required.
- **701.51b:** OUT-OF-SCOPE — Attraction card placement.
- **701.51c:** OUT-OF-SCOPE — Attraction ETB trigger.

### 701.52 — Roll to Visit Your Attractions

**701.52a:** OUT-OF-SCOPE — Attraction cards (Unfinity).

### 701.53 — Incubate: RECLASSIFIED → ATOM (double-faced token, architectural implications)

**Why this can't wait:** Incubate creates a **double-faced token** — currently impossible because `CardData` has no back-face concept. This is the simplest DFC mechanic (no transform triggers, no daybound, just "{2}: Transform this token") and is the ideal first test case for DFC infrastructure.

**Architectural requirements:**
1. **`back_face: Option<Arc<CardData>>` on `CardData`** (or `GameObject`). This is the D3 roadmap item. Incubate is the simplest motivating case.
2. **Transform primitive** that flips between front and back face. Needed by Incubate, and later by all DFC mechanics.
3. **Token with DFC characteristics.** Token creation must support specifying front+back face.

**ATOM-701.53a-001**
- **Rule:** 701.53a — Incubate N: create Incubator token with N +1/+1 counters.
- **Mechanism:** `incubate(game, controller, n)` — create DFC token, add counters
- **Minimal Board:** P0 casts spell with "Incubate 3."
- **Action:** Effect resolves.
- **Expected Result:** Incubator token (front face: colorless artifact, has "{2}: Transform this token") enters battlefield with 3 +1/+1 counters.
- **Phase:** Phase 9 (DFC infrastructure, D3)
- **Ticket:** D3 — double-faced card infrastructure

**ATOM-701.53b-001**
- **Rule:** 701.53b — Incubator token transforms into 0/0 Phyrexian artifact creature.
- **Minimal Board:** Incubator token with 3 +1/+1 counters. P0 pays {2} to transform.
- **Action:** Transform resolves.
- **Expected Result:** Token is now back face: 0/0 colorless Phyrexian artifact creature named "Phyrexian Token." Still has 3 +1/+1 counters → effectively 3/3.
- **Phase:** Phase 9 (D3)
- **Ticket:** D3

**Verdict:** Tests written, but phase stays at Phase 9 (DFC). However, this is now the **anchor use case** for D3 design — the simplest DFC to implement first.

### 701.54 — The Ring Tempts You: RECLASSIFIED → ATOM (emblem + designation + progressive abilities)

**Why this can't wait:** This mechanic requires three new subsystems, all of which are reusable:
1. **Emblem system** — currently nonexistent. Emblems are also created by planeswalker ultimates, so this isn't Ring-specific.
2. **Designation system** — Ring-bearer is a designation (like monstrous, suspected, harnessed). We need a general designation framework, not ad-hoc booleans.
3. **Progressive/leveled abilities on emblems** — "as long as the Ring has tempted you N+ times" is a counter-gated ability set.

**Architectural requirements:**
1. **Emblem as a game object.** Emblems live in the command zone (rule 114). They need to be `GameObject` instances with `Zone::Command`. They have abilities but are not permanents.
2. **Designation registry.** Rather than adding `is_monstrous: bool`, `is_suspected: bool`, etc. as individual fields, use `designations: HashSet<Designation>` on `BattlefieldEntity`:
   ```rust
   enum Designation { Monstrous, Suspected, Harnessed, RingBearer, Renowned, ... }
   ```
3. **Temptation counter** — per-player `ring_temptation_count: u32`. The emblem's abilities are gated on this value.

**Clarification (post-audit):** The designation registry on `BattlefieldEntity` handles permanent designations (monstrous, suspected, harnessed, renowned, ring-bearer). Player-level state (temptation counter) goes on `PlayerState` as a simple `u32`. The Ring emblem is a `GameObject` in the command zone. No union struct is needed — player booleans/counters are cheap and the right abstraction for player-scoped state. The `HashSet<Designation>` is exclusively for permanent-scoped designations.

**ATOM-701.54a-001**
- **Rule:** 701.54a — Ring tempts you: choose creature, it becomes Ring-bearer.
- **Mechanism:** `ring_tempts(game, player, decisions)` — create emblem if needed, increment counter, choose creature, designate
- **Minimal Board:** P0 controls Creatures A and B. First time Ring tempts P0.
- **Action:** Ring tempts P0. P0 chooses creature A.
- **Expected Result:** P0 gets The Ring emblem (command zone). A has Ring-bearer designation. ring_temptation_count = 1. A is legendary (per emblem ability). A can't be blocked by creatures with greater power.
- **Phase:** Phase 8 (emblem system + designation system)
- **Ticket:** NEW — emblem infrastructure + ring temptation

**ATOM-701.54a-002**
- **Rule:** 701.54a — Ring tempts again: new Ring-bearer replaces old.
- **Minimal Board:** A is current Ring-bearer. Ring tempts P0 again (2nd time). P0 chooses B.
- **Expected Result:** A loses Ring-bearer designation. B gains Ring-bearer designation. ring_temptation_count = 2. Emblem now also has "Whenever your Ring-bearer attacks, draw a card, then discard a card."
- **Phase:** Phase 8
- **Ticket:** NEW — ring-bearer redesignation + progressive unlock

**ATOM-701.54c-001**
- **Rule:** 701.54c — Progressive abilities unlock at temptation thresholds.
- **Minimal Board:** P0 has been tempted 4 times. Ring-bearer C attacks.
- **Action:** C deals combat damage to P1.
- **Expected Result:** All four Ring abilities active: (1) legendary + can't be blocked by greater power, (2) attack → draw/discard, (3) blocked → blocker sacrificed at end of combat, (4) combat damage to player → each opponent loses 3 life.
- **Phase:** Phase 8
- **Ticket:** NEW — progressive emblem abilities

**701.54b:** PURE-DEF — Ring-bearer designation not copiable. Covered by designation system.
**701.54d:** PURE-DEF — trigger timing.
**701.54e:** PURE-DEF — condition check.

### 701.55 — Face a Villainous Choice: RECLASSIFIED → ATOM

**Why deferral was wrong:** The original deferral said "multiplayer-oriented" but 701.55a is completely format-agnostic — a player chooses A or B. This is a basic **modal choice during resolution** that works identically in 2-player. The multiplayer part (701.55d: APNAP ordering) is a trivial extension.

**701.55b is architecturally significant:** Players may choose an illegal/impossible option, performing as much as possible. This is an **exception to rule 608.2d** (normally, impossible instructions are skipped entirely). The engine's effect resolution pipeline needs to handle "perform as much as possible" mode for villainous choice branches.

**ATOM-701.55a-001**
- **Rule:** 701.55a — Face villainous choice: choose A or B, perform chosen option.
- **Mechanism:** `face_villainous_choice(game, player, option_a, option_b, decisions)` — DP picks, resolve
- **Minimal Board:** P0 faces choice: "Sacrifice a creature, or each opponent draws 2 cards."
- **Action:** P0 chooses to sacrifice a creature. Sacrifices Creature C.
- **Expected Result:** C goes to graveyard. P1 draws nothing.
- **Phase:** Phase 8
- **Ticket:** NEW — villainous choice primitive

**ATOM-701.55b-001**
- **Rule:** 701.55b — May choose impossible option; perform as much as possible.
- **Minimal Board:** P0 faces choice: "Sacrifice a creature and discard 2 cards, or each opponent gains 5 life." P0 controls no creatures but has 3 cards in hand.
- **Action:** P0 chooses option A (sacrifice + discard) despite having no creatures.
- **Expected Result:** Sacrifice impossible → skipped. Discard 2 cards → performed. This is an exception to 608.2d.
- **Phase:** Phase 8
- **Ticket:** NEW — villainous choice impossible branch

**701.55c:** DEFERRED Phase 6 — replacement effect multiplying a choice. Genuinely needs replacement infrastructure.
**701.55d:** Trivial APNAP extension for multiplayer. DEFERRED Phase 9.

--- End of Chunk 7 ---

## Chunk 8 — Rules 701.56–701.68 (Time Travel, Discover, Cloak, Collect Evidence, Suspect, Forage, Manifest Dread, Endure, Harness, Airbend, Earthbend, Waterbend, Blight)

### 701.56 — Time Travel

**701.56a:** DEFERRED — Phase 8. Time travel involves choosing permanents/suspended cards with time counters and adding or removing one. Requires suspend infrastructure. One-line entry.

### 701.57 — Discover: RECLASSIFIED → ATOM (exile-from-library + free-cast pattern)

**Why this can't wait:** Discover is "exile from library top until condition, then optionally cast for free or put in hand." This is the same structural pattern as Cascade (702.84), one of the most common competitive mechanics.

**Implementation note (post-audit):** Cascade (702.84) is the older, more widely-played mechanic. Implementation order should be: (1) Cascade in Session 7B/8, (2) Discover as a thin wrapper (Discover = Cascade with an MV cap + hand alternative). The tests here define the **shared infrastructure** both need (exile-from-library loop, free-cast pipeline, bottom-in-random-order cleanup).

**Architectural requirements:**
1. **Exile-then-cast-for-free pipeline.** The card is exiled, then optionally cast without paying mana cost. This requires the casting system to support `CastPermission::FreeFromExile` — casting from a non-hand zone for an alternative cost of {0}.
2. **"Bottom of library in random order"** for remaining exiled cards — batch zone change.

**ATOM-701.57a-001**
- **Rule:** 701.57a — Discover N: exile until nonland MV ≤ N, may cast free or put in hand.
- **Mechanism:** `discover(game, controller, n, decisions)` — exile loop, optional free cast, cleanup
- **Minimal Board:** P0's library top 4: Land, Land, Creature (MV 3), Land. Effect "Discover 4."
- **Action:** Exile Land, Land, Creature (MV 3 ≤ 4 → stop). P0 chooses to cast Creature for free.
- **Expected Result:** Creature cast without paying mana cost. Remaining 2 exiled lands go to bottom of library in random order.
- **Phase:** Phase 8
- **Ticket:** NEW — discover primitive (+ CastPermission::FreeFromExile)

**ATOM-701.57a-002**
- **Rule:** 701.57a — Discover: choose not to cast, put in hand instead.
- **Minimal Board:** Same setup. P0 chooses not to cast.
- **Expected Result:** Creature goes to P0's hand. Exiled lands go to bottom in random order.
- **Phase:** Phase 8
- **Ticket:** NEW — discover hand path

**ATOM-701.57a-003**
- **Rule:** 701.57a — Discover: entire library is lands (no valid card found).
- **Minimal Board:** P0's library is all lands. Effect "Discover 5."
- **Action:** Exile entire library. No nonland card found.
- **Expected Result:** All exiled cards go to bottom of library in random order. No cast, no hand. Player "discovered" (701.57b) even though nothing happened.
- **Phase:** Phase 8
- **Ticket:** NEW — discover whiff

**701.57b–c:** PURE-DEF — trigger timing and "discovered card" reference.

### 701.58 — Cloak: RECLASSIFIED → ATOM (manifest variant, shares face-down infrastructure)

**Covered by 701.40 analysis above.** Cloak is manifest + ward {2}. Once face-down infrastructure exists (701.40), Cloak is a thin wrapper:
- Same face-down 2/2 creature characteristics
- Same turn-face-up special action
- Additional: ward {2} while face-down (characteristic-defining effect includes ward)
- `FaceDownOrigin::Cloaked` in the enum

**ATOM-701.58a-001**
- **Rule:** 701.58a — Cloak: face-down 2/2 with ward {2}.
- **Mechanism:** Same as manifest, but origin=Cloaked, face-down characteristics include ward {2}
- **Minimal Board:** Card C in P0's library. Effect "Cloak the top card."
- **Action:** Cloak resolves.
- **Expected Result:** C is face-down 2/2 creature with ward {2}. `face_down_origin == Cloaked`.
- **Phase:** Backlog — CR 701 keyword actions (was Phase 5-Pre)
- **Ticket:** NEW — cloak variant of face-down system

**701.58b–h:** Same structural rules as 701.40b–h. Tests mirror manifest tests with `Cloaked` origin and ward {2}.

### 701.59 — Collect Evidence: RECLASSIFIED → ATOM (graveyard exile as additional cost)

**Why this can't wait:** Collect Evidence is "exile cards from graveyard with total MV ≥ N." This exercises: (1) the cost system's ability to handle non-mana additional costs that involve player choices (which cards to exile), and (2) MV summation across multiple objects.

**ATOM-701.59a-001**
- **Rule:** 701.59a — Collect evidence N: exile cards from GY with total MV ≥ N.
- **Mechanism:** `collect_evidence(game, player, n, decisions)` — DP picks GY cards, validate MV sum, exile
- **Minimal Board:** P0's graveyard: Card A (MV 2), Card B (MV 3), Card C (MV 1). Effect "Collect evidence 4."
- **Action:** P0 exiles A (MV 2) and B (MV 3). Total = 5 ≥ 4. Valid.
- **Expected Result:** A and B exiled from graveyard.
- **Phase:** Phase 8
- **Ticket:** NEW — collect evidence primitive

**ATOM-701.59b-001**
- **Rule:** 701.59b — Can't choose to collect evidence if GY can't reach N.
- **Minimal Board:** P0's graveyard total MV = 3. Effect offers choice to "collect evidence 5."
- **Action:** P0 attempts to choose collect evidence.
- **Expected Result:** Choice unavailable — can't reach MV 5 from graveyard.
- **Phase:** Phase 8
- **Ticket:** NEW — collect evidence legality check

**701.59c:** PURE-DEF — linked ability with additional cost. Covered by linked ability system (Phase 7).

### 701.60 — Suspect

**ATOM-701.60a-001**
- **Rule:** 701.60a — Suspect a creature: it becomes suspected until it leaves the battlefield or an effect removes the designation.
- **Mechanism:** Suspect designation on permanent
- **Minimal Board:** Creature C on battlefield (not suspected).
- **Action:** Effect "Suspect C."
- **Expected Result:** C gains the suspected designation.
- **Phase:** Phase 8 (suspect keyword action)
- **Ticket:** NEW — suspect primitive

**ATOM-701.60c-001**
- **Rule:** 701.60c — A suspected permanent has menace and "This creature can't block."
- **Mechanism:** Suspected grants menace + can't-block restriction
- **Minimal Board:** Creature C is suspected.
- **Action:** Query C's abilities and block legality.
- **Expected Result:** C has menace. C cannot be declared as a blocker.
- **Phase:** Phase 8
- **Ticket:** NEW — suspected grants menace + can't block

**701.60b:** PURE-DEF — Suspected is a designation (not ability, not copiable). Only permanents can have it.

**701.60d:** PURE-DEF — A suspected permanent can't become suspected again (no-op). No independent test — trivially handled by designation check.

### 701.61 — Forage

**701.61a:** DEFERRED — Phase 8. Forage = exile 3 cards from graveyard OR sacrifice a Food. Requires Food token infrastructure. One-line entry.

### 701.62 — Manifest Dread: RECLASSIFIED → ATOM (depends on 701.40 Manifest)

**Once manifest infrastructure exists, this is trivial:** look at top 2, choose 1 to manifest, put other in graveyard. No new subsystems needed beyond manifest.

**ATOM-701.62a-001**
- **Rule:** 701.62a — Manifest dread: look at top 2, manifest one, other to graveyard.
- **Mechanism:** `manifest_dread(game, controller, decisions)` — look at 2, DP picks, manifest, GY
- **Minimal Board:** P0's top 2 library cards: Creature C (5/5), Instant I.
- **Action:** P0 chooses to manifest C.
- **Expected Result:** C is face-down 2/2 on battlefield (manifested). I goes to graveyard.
- **Phase:** Backlog — CR 701 keyword actions (was Phase 5-Pre)
- **Ticket:** NEW — manifest dread (depends on face-down subsystem)

**701.62b:** PURE-DEF — trigger timing.

### 701.63 — Endure

**ATOM-701.63a-001**
- **Rule:** 701.63a — "Endure N": controller creates N/N white Spirit creature token UNLESS they put N +1/+1 counters on the permanent.
- **Mechanism:** Endure choice — counters on self or Spirit token
- **Minimal Board:** Creature C (2/2). Effect "C endures 3."
- **Action:** P0 chooses to put 3 +1/+1 counters on C.
- **Expected Result:** C gets 3 +1/+1 counters. C is now 5/5. No token created.
- **Phase:** Phase 8 (endure keyword action)
- **Ticket:** NEW — endure primitive

**ATOM-701.63a-002**
- **Rule:** 701.63a — Endure: choosing the token path.
- **Mechanism:** Endure token creation
- **Minimal Board:** Creature C (2/2). Effect "C endures 3."
- **Action:** P0 chooses NOT to put counters on C. Token created instead.
- **Expected Result:** A 3/3 white Spirit creature token is created. C remains 2/2.
- **Phase:** Phase 8
- **Ticket:** NEW — endure token path

**ATOM-701.63b-001**
- **Rule:** 701.63b — Endure 0 = nothing happens. No counters, no token.
- **Mechanism:** Endure 0 short-circuit
- **Minimal Board:** Creature C. Effect "C endures 0."
- **Action:** Effect resolves.
- **Expected Result:** Nothing happens. No counters on C. No token created.
- **Phase:** Phase 8
- **Ticket:** NEW — endure 0 no-op

### 701.64 — Harness

**ATOM-701.64a-001**
- **Rule:** 701.64a — "Harness [this permanent]" = if not harnessed, it becomes harnessed.
- **Mechanism:** Harness designation
- **Minimal Board:** Permanent P (not harnessed).
- **Action:** Effect "Harness P."
- **Expected Result:** P gains the harnessed designation.
- **Phase:** Phase 8 (harness keyword action)
- **Ticket:** NEW — harness primitive

**ATOM-701.64a-002**
- **Rule:** 701.64a — If already harnessed, harness does nothing.
- **Mechanism:** Harness guard
- **Minimal Board:** Permanent P (already harnessed).
- **Action:** Effect "Harness P."
- **Expected Result:** Nothing happens. P remains harnessed.
- **Phase:** Phase 8
- **Ticket:** NEW — harness guard

**701.64b:** PURE-DEF — Harnessed is a designation (not ability, not copiable). Leaves battlefield → loses harnessed. Same pattern as monstrous.

### 701.65 — Airbend: RECLASSIFIED → ATOM (exile + cast-from-exile for alternate cost)

**Why this can't wait:** Airbend exiles permanents/spells, then grants their owners a "cast from exile for {2}" permission. This exercises:
1. **Persistent cast-from-exile permission** — the engine needs `CastPermission::AlternateCostFromExile { cost: ManaCost, card_id: ObjectId }` that persists across turns until the card leaves exile. This is distinct from Discover/Cascade's one-shot free cast.
2. **Exile as a removal + tempo mechanic** — the exiled card's *owner* (not controller) gets the cast permission, which matters in multiplayer.
3. **Batch exile** — "airbend one or more permanents and/or spells" can exile multiple objects in one action.

**ATOM-701.65a-001**
- **Rule:** 701.65a — Airbend: exile permanent, owner may cast from exile for {2}.
- **Mechanism:** `airbend(game, player, targets)` — exile each target, create persistent cast permission per card
- **Minimal Board:** P0 controls Creature C (owned by P0). Effect "Airbend C."
- **Action:** Airbend resolves.
- **Expected Result:** C is exiled. P0 has persistent permission: "You may cast C from exile by paying {2} rather than its mana cost." Permission lasts as long as C remains in exile.
- **Phase:** Phase 8 (cast-from-exile pipeline)
- **Ticket:** NEW — airbend primitive (+ CastPermission::AlternateCostFromExile)

**ATOM-701.65a-002**
- **Rule:** 701.65a — Airbend: owner casts exiled card for {2}.
- **Minimal Board:** Card C exiled by airbend. C's mana cost is {3}{R}{R}. P0 (owner) has {2} available.
- **Action:** P0 casts C from exile, paying {2} instead of {3}{R}{R}.
- **Expected Result:** C is cast successfully for {2}. Permission consumed (card left exile → went to stack). On resolution, C enters battlefield/resolves normally.
- **Phase:** Phase 8
- **Ticket:** NEW — airbend alternate cost cast

**ATOM-701.65a-003**
- **Rule:** 701.65a — Airbend: batch exile of multiple objects.
- **Minimal Board:** P0 controls Creatures A and B. Effect "Airbend A and B."
- **Action:** Airbend resolves.
- **Expected Result:** Both A and B exiled. P0 gets separate cast permissions for each. Can cast either independently for {2}.
- **Phase:** Phase 8
- **Ticket:** NEW — airbend batch exile

**701.65b:** PURE-DEF — "whenever you airbend" trigger timing. Triggers once per airbend action regardless of how many objects exiled. No independent test beyond trigger infrastructure (Phase 7).

### 701.66 — Earthbend: RECLASSIFIED → ATOM (land animation + delayed trigger)

**Why this can't wait:** Earthbend exercises three important patterns:
1. **Land animation** — "becomes a 0/0 creature in addition to its other types" is a layer 4 (type) + layer 7b (set P/T) continuous effect. Same pattern as Dryad Arbor, Mutavault, and every manland.
2. **Haste granting** — layer 6 keyword addition.
3. **Delayed triggered ability** — "When that land dies or is put into exile, return it to the battlefield tapped." Delayed triggers are Phase 7 infrastructure.

**ATOM-701.66a-001**
- **Rule:** 701.66a — Earthbend N: land becomes 0/0 creature with haste, gets N +1/+1 counters, delayed return trigger.
- **Mechanism:** Land animation (L4 type-changing + L7b P/T setting) + counter placement + delayed trigger creation
- **Minimal Board:** P0 controls Forest F. Effect "Earthbend 3" targeting F.
- **Action:** Effect resolves.
- **Expected Result:** F is now a 0/0 Forest Land Creature with haste. F has 3 +1/+1 counters (effectively 3/3). Delayed trigger created: "When F dies or is exiled, return it to battlefield tapped under P0's control."
- **Phase:** Phase 7 (delayed triggers) + Phase 5-Layers (land animation)
- **Ticket:** NEW — earthbend primitive (delayed trigger + land animation)

**ATOM-701.66a-002**
- **Rule:** 701.66a — Earthbend: animated land dies, delayed trigger returns it.
- **Minimal Board:** Forest F (animated by earthbend, 3/3 with 3 +1/+1 counters). F takes 3 damage → SBA destroys it.
- **Action:** F dies. Delayed trigger fires.
- **Expected Result:** F returns to battlefield tapped under P0's control. No longer animated (the animation effect ended). No +1/+1 counters (new object). Just a regular tapped Forest.
- **Phase:** Phase 7
- **Ticket:** NEW — earthbend delayed return

**701.66b:** PURE-DEF — "whenever you earthbend" trigger timing.

### 701.67 — Waterbend: RECLASSIFIED → ATOM (alternative mana payment)

**Why this can't wait:** Waterbend lets you tap artifacts/creatures to pay generic mana in a specific cost. This is an **alternative payment method within cost payment** — a pattern shared by convoke (702.51) and improvise (702.126). Getting the cost-payment pipeline right for waterbend means convoke/improvise are nearly free.

**Architectural requirements:**
1. **Cost payment pipeline extension.** Currently, mana costs are paid by tapping lands and spending pool mana. Waterbend adds: "for generic mana in *this specific sub-cost*, you may tap an artifact/creature instead of paying {1}." The cost system needs to support per-cost-component alternative payment methods.
2. **701.67b is the tricky part:** The tap-for-generic only applies to the waterbend cost's generic portion, NOT the total spell cost's generic. This means the engine must track which generic mana belongs to which cost component during resolution.

**ATOM-701.67a-001**
- **Rule:** 701.67a — Waterbend: pay cost, tap artifacts/creatures for generic portion.
- **Mechanism:** Cost payment with artifact/creature tap for generic mana
- **Minimal Board:** Spell with "waterbend {4}" as additional cost. P0 controls 3 untapped artifacts and has 1 mana in pool.
- **Action:** P0 taps 3 artifacts (paying {3} of the {4}) and pays {1} from pool.
- **Expected Result:** Waterbend cost paid. Total: 3 tapped artifacts + 1 mana from pool = {4}.
- **Phase:** Phase 8 (cost payment extension)
- **Ticket:** NEW — waterbend cost + tap-for-generic pipeline

**ATOM-701.67b-001**
- **Rule:** 701.67b — Waterbend tap-for-generic only applies to waterbend cost, not total spell cost.
- **Minimal Board:** Spell costs {1}{U}{U} with "waterbend {6}" additional cost. P0 has {U}{U} in pool and controls 6 untapped creatures.
- **Action:** P0 taps 6 creatures to pay waterbend {6}. P0 pays {U}{U} from pool. P0 must still pay {1} from the spell's own cost (can't use creature-tapping for this).
- **Expected Result:** P0 needs 1 more mana. The 6 creature taps only cover the waterbend {6}, not the spell's {1}.
- **Phase:** Phase 8
- **Ticket:** NEW — waterbend cost isolation

**701.67c:** PURE-DEF — "whenever you waterbend" trigger timing.

### 701.68 — Blight

**ATOM-701.68a-001**
- **Rule:** 701.68a — "Blight N" = put N -1/-1 counters on a creature you control.
- **Mechanism:** `blight(game, controller, n, decisions)` — DP chooses target creature, add -1/-1 counters
- **Minimal Board:** P0 controls Creature A (3/3) and Creature B (2/2). Effect "Blight 2."
- **Action:** P0 chooses Creature A.
- **Expected Result:** A gets 2 -1/-1 counters. A is now 1/1.
- **Phase:** Phase 8 (blight keyword action)
- **Ticket:** NEW — blight primitive

**ATOM-701.68b-001**
- **Rule:** 701.68b — If a player can't put N -1/-1 counters on a creature they control (no creatures), they can't choose to blight.
- **Mechanism:** Blight legality check — must control at least one creature
- **Minimal Board:** P0 controls no creatures. Effect gives P0 a choice to blight.
- **Action:** P0 attempts to choose to blight.
- **Expected Result:** Can't choose to blight. The choice is unavailable.
- **Phase:** Phase 8
- **Ticket:** NEW — blight legality

**701.68c:** PURE-DEF — "Blighted creature" refers to the creature chosen for -1/-1 counters. Terminology. No independent test.

**701.68d:** PURE-DEF — "Whenever you blight" triggers after the process completes. Trigger timing — Phase 7 concern.

--- End of Chunk 8 ---

## Classification Summary Table

| Rule | Classification | Phase | Ticket | Notes |
|------|---------------|-------|--------|-------|
| 700.1 | PURE-DEF | — | — | Event definition |
| 700.2 | PURE-DEF | — | — | Modal definition (header) |
| 700.2a | TESTABLE | Phase 5-Pre | T18 | Modal mode choice at cast time |
| 700.2b | TESTABLE | Phase 7 | NEW | Modal triggered ability mode choice + removal |
| 700.2c | TESTABLE | Phase 5-Pre | T18 | Mode-conditional targeting |
| 700.2d | TESTABLE | Phase 5-Pre / Phase 8 | T18 / NEW | Mode uniqueness + repeated modes |
| 700.2e | TESTABLE | Phase 5-Pre | T18 | Opponent mode choice |
| 700.2f | PURE-DEF | — | — | Target change can't change mode |
| 700.2g | TESTABLE | Phase 7 | NEW (D19) | Copy preserves modes |
| 700.2h | TESTABLE | Phase 5-Pre | T18 | Per-mode additional costs |
| 700.2i | TESTABLE | Phase 8 | NEW | Pawprint budget |
| 700.3 | PURE-DEF | — | — | Piles header |
| 700.3a | TESTABLE | Phase 8 | NEW | One pile per object |
| 700.3b | PURE-DEF | — | — | Pile not an object |
| 700.3c | TESTABLE | Phase 8 | NEW | Pile zone invariant |
| 700.3d | PURE-DEF | — | — | Pile can be empty |
| 700.4 | ALREADY-IMPL | — | — | "Dies" definition |
| 700.5 | TESTABLE | Phase 8 | NEW | Devotion single color |
| 700.5 (2-color) | TESTABLE | Phase 8 | NEW | Devotion two colors |
| 700.5a | TESTABLE | Phase 5-Layers | NEW (6b-D14) | Devotion partial-layer (2 tests: partial-layer + threshold feedback) |
| 700.6 | BOUNDARY-DEF | Phase 8 | NEW | Historic predicate |
| 700.7 | DEFERRED | Phase 7 | — | "This [something]" identity |
| 700.8–700.8d | DEFERRED | Phase 8 | — | Party mechanic |
| 700.9 | BOUNDARY-DEF | Phase 8 | NEW | Modified predicate |
| 700.10 | DEFERRED | Phase 8 | — | "Activated this turn" |
| 700.11 | DEFERRED | Phase 8 | — | Descended |
| 700.12 | BOUNDARY-DEF | Phase 8 | NEW | Outlaw predicate |
| 700.12a | PURE-DEF | — | — | Outlaw permanents clarification |
| 700.13 | DEFERRED | Phase 8 | — | Committing a crime |
| 700.14 | DEFERRED | Phase 8 | — | Expend |
| 700.15 | PURE-DEF | — | — | "Enters" shorthand |
| 701.1 | PURE-DEF | — | — | Keyword actions intro |
| 701.2a | ALREADY-IMPL | — | — | Activate |
| 701.3a | TESTABLE | Phase 5-Pre | T15 | Attach basic |
| 701.3a (illegal) | TESTABLE | Phase 5-Pre | T15 | Attach to invalid target |
| 701.3b (can't attach) | TESTABLE | Phase 5-Pre | T15 | Failed attach no movement |
| 701.3b (same target) | TESTABLE | Phase 5-Pre | T15 | Same-target no-op |
| 701.3b (non-attachment) | TESTABLE | Phase 5-Pre | T15 | Non-Aura/Equip does nothing |
| 701.3c | TESTABLE | Phase 5-Pre | T15 | Reattach new timestamp |
| 701.3d (unattach) | TESTABLE | Phase 5-Pre | T15b | Unattach Equipment |
| 701.3d (creature leaves) | TESTABLE | Phase 5-Pre | T15b | Creature leaves → unattach |
| 701.4–701.4b | DEFERRED | Phase 8 | — | Behold |
| 701.5a | ALREADY-IMPL | — | — | Cast |
| 701.5b | PURE-DEF | — | — | "Cast a card" |
| 701.6a | ALREADY-IMPL | — | — | Counter |
| 701.6b | ALREADY-IMPL | — | — | No cost refund |
| 701.7a | TESTABLE | Phase 8 | NEW | Create tokens |
| 701.7b | TESTABLE | Phase 8 | NEW | Token creation replacement ordering |
| 701.7c | PURE-DEF | — | — | Errata |
| 701.8a | ALREADY-IMPL | — | — | Destroy |
| 701.8b | TESTABLE | Phase 6 | NEW (D20) | Destroy delta tagging |
| 701.8c | TESTABLE | Phase 6 | NEW | Regeneration replaces destroy |
| 701.9a | ALREADY-IMPL | — | — | Discard |
| 701.9b | TESTABLE | Phase 8 | NEW | Random / opponent discard |
| 701.9c | TESTABLE | Phase 8 | NEW | Discard to hidden zone |
| 701.10a | TESTABLE | Phase 5-Layers | NEW | Double P/T as L7c |
| 701.10b | TESTABLE | Phase 5-Layers | NEW | Double snapshot |
| 701.10c | TESTABLE | Phase 5-Layers | NEW | Negative power doubling |
| 701.10d | TESTABLE | Phase 8 | NEW | Double life total |
| 701.10e | TESTABLE | Phase 8 | NEW | Double counters |
| 701.10f | TESTABLE | Phase 8 | NEW | Double mana |
| 701.10g | TESTABLE | Phase 6 | NEW | Damage doubling replacement |
| 701.11a | TESTABLE | Phase 5-Layers | NEW | Triple P/T as L7c |
| 701.11b | TESTABLE | Phase 5-Layers | NEW | Triple power only |
| 701.11c | TESTABLE | Phase 5-Layers | NEW | Negative power tripling |
| 701.12a | TESTABLE | Phase 8 | NEW | Exchange all-or-nothing |
| 701.12b | TESTABLE | Phase 8 | NEW | Control exchange |
| 701.12b (same ctrl) | TESTABLE | Phase 8 | NEW | Same-controller no-op |
| 701.12c | TESTABLE | Phase 8 | NEW | Life total exchange |
| 701.12c (can't gain) | TESTABLE | Phase 8 | NEW | Life exchange + can't gain |
| 701.12d | TESTABLE | Phase 8 | NEW | Zone card exchange (mass zone exchange test added) |
| 701.12e | DEFERRED | Phase 8 | — | Attachment transfer |
| 701.12f | DEFERRED | Phase 8 | — | Empty zone exchange |
| 701.12g | TESTABLE | Phase 8 | NEW | Numerical value exchange |
| 701.12h | DEFERRED | Phase 8 | — | Text-box exchange |
| 701.13a | ALREADY-IMPL | — | — | Exile |
| 701.14a | TESTABLE | Phase 8 | NEW | Fight mutual damage |
| 701.14b (no longer on BF) | TESTABLE | Phase 8 | NEW | Fight validity |
| 701.14b (illegal target) | TESTABLE | Phase 8 | NEW | Fight illegal target |
| 701.14c | TESTABLE | Phase 8 | NEW | Self-fight |
| 701.14d | TESTABLE | Phase 8 | NEW | Fight non-combat damage |
| 701.15a–d | DEFERRED | Phase 9 | — | Goad |
| 701.16a | TESTABLE | Phase 8 | NEW | Investigate / Clue token |
| 701.17a | TESTABLE | Phase 8 | NEW | Mill basic |
| 701.17b | TESTABLE | Phase 8 | NEW | Mill cap + cost legality |
| 701.17c | DEFERRED | Phase 8 | — | Milled card tracking |
| 701.17d | DEFERRED | Phase 8 | — | Multi-mill references |
| 701.18a | ALREADY-IMPL | — | — | Play a land |
| 701.18b | DEFERRED | Phase 8 | — | Play permission via continuous effects (no new enum) |
| 701.18c–e | PURE-DEF | — | — | Play terminology |
| 701.19a | TESTABLE | Phase 6 | NEW | Regen shield (one-shot) |
| 701.19a (single-use) | TESTABLE | Phase 6 | NEW | Regen single-use |
| 701.19b | TESTABLE | Phase 6 | NEW | Static regeneration |
| 701.19c | TESTABLE | Phase 6 | NEW | Can't-be-regenerated |
| 701.20a | TESTABLE | Phase 8 | NEW | Reveal |
| 701.20b–c | PURE-DEF | — | — | Reveal zone/re-reveal |
| 701.20d | DEFERRED | Phase 8 | — | Shuffle stops reveal |
| 701.20e | PURE-DEF | — | — | "Look at" |
| 701.21a | TESTABLE | Phase 5-Pre | T15 | Sacrifice (3 tests) |
| 701.22a | TESTABLE | Phase 8 | NEW | Scry |
| 701.22b | TESTABLE | Phase 8 | NEW | Scry 0 no-op |
| 701.22c | DEFERRED | Phase 8 | — | Simultaneous scry (APNAP, valid in 2-player) |
| 701.22d | PURE-DEF | — | — | Scry trigger timing |
| 701.23a | TESTABLE | Phase 8 | NEW | Search |
| 701.23b | TESTABLE | Phase 8 | NEW | Fail-to-find |
| 701.23c | PURE-DEF | — | — | Undefined quality |
| 701.23d | TESTABLE | Phase 8 | NEW | Mandatory quantity search |
| 701.23e | PURE-DEF | — | — | Found cards not revealed |
| 701.23f–h | DEFERRED | Phase 8 | — | Search variants |
| 701.23i | DEFERRED | Phase 8 | — | Simultaneous search (APNAP, valid in 2-player) |
| 701.23j | DEFERRED | Phase 8 | — | Outside-game search |
| 701.24a | ALREADY-IMPL | — | — | Shuffle |
| 701.24b | TESTABLE | Phase 8 | NEW | Search-shuffle exclusion |
| 701.24c–e | PURE-DEF | — | — | Shuffle edge cases |
| 701.24f | DEFERRED | Phase 7 | — | Simultaneous shuffle triggers |
| 701.24g | DEFERRED | Phase 8 | — | Shuffle + position |
| 701.25a | TESTABLE | Phase 8 | NEW | Surveil |
| 701.25b | DEFERRED | Phase 8 | — | Additional-card surveil |
| 701.25c | TESTABLE | Phase 8 | NEW | Surveil 0 no-op |
| 701.25d | PURE-DEF | — | — | Surveil trigger timing |
| 701.26a | ALREADY-IMPL | — | — | Tap |
| 701.26b | ALREADY-IMPL | — | — | Untap |
| 701.27a–g | DEFERRED | Phase 9 | — | Transform |
| 701.28a–f | DEFERRED | Phase 9 | — | Convert |
| 701.29a | TESTABLE | Phase 8 | NEW | Fateseal (scry wrapper on opponent's library) |
| 701.30a–d | DEFERRED | Phase 8 | — | Clash |
| 701.31a–d | OUT-OF-SCOPE | — | — | Planeswalk (Planechase) |
| 701.32a–c | OUT-OF-SCOPE | — | — | Set in Motion (Archenemy) |
| 701.33a–b | OUT-OF-SCOPE | — | — | Abandon (Archenemy) |
| 701.34a | TESTABLE | Phase 8 | NEW | Proliferate |
| 701.34b | OUT-OF-SCOPE | — | — | THG poison (team format) |
| 701.35a | TESTABLE | Phase 8 | NEW | Detain |
| 701.36a | TESTABLE | Phase 8 | NEW | Populate |
| 701.36b | TESTABLE | Phase 8 | NEW | Populate empty no-op |
| 701.37a | TESTABLE | Phase 8 | NEW | Monstrosity + guard |
| 701.37b | PURE-DEF | — | — | Monstrous designation |
| 701.37c | DEFERRED | Phase 8 | — | Monstrosity X variable |
| 701.38a–d | DEFERRED | Phase 9 | — | Vote |
| 701.39a | TESTABLE | Phase 8 | NEW | Bolster + tie-break |
| 701.40a–h | **ATOM** | Phase 5-Pre | NEW | Manifest — face-down infrastructure (architectural core) |
| 701.41a | TESTABLE | Phase 8 | NEW | Support (2 tests: permanent + nonpermanent spell) |
| 701.42a–c | DEFERRED | Phase 9 | — | Meld |
| 701.43a–d | **ATOM** | Phase 5-Pre | NEW | Exert — skip-untap tracking |
| 701.44a–d | **ATOM** | Phase 8 | NEW | Explore — multi-step keyword + LKI |
| 701.45a | OUT-OF-SCOPE | — | — | Assemble (Unstable) |
| 701.46a | TESTABLE | Phase 8 | NEW | Adapt + guard |
| 701.47a–d | **ATOM** | Phase 8 | NEW | Amass — conditional token + subtype granting |
| 701.48a | DEFERRED | Phase 8 | — | Learn |
| 701.49a–d | DEFERRED | Phase 9 | — | Venture — unique infra, arch notes added |
| 701.50a–e | **ATOM** | Phase 8 | NEW | Connive — draw-discard-counter pattern |
| 701.51a–c | OUT-OF-SCOPE | — | — | Open Attraction (Unfinity) |
| 701.52a | OUT-OF-SCOPE | — | — | Roll to Visit (Unfinity) |
| 701.53a–b | **ATOM** | Phase 9 (D3) | D3 | Incubate — simplest DFC, anchor for D3 |
| 701.54a–e | **ATOM** | Phase 8 | NEW | Ring Tempts You — emblem + designation + progressive |
| 701.55a–d | **ATOM** | Phase 8 | NEW | Villainous Choice — modal choice + impossible branch |
| 701.56a | DEFERRED | Phase 8 | — | Time Travel |
| 701.57a–c | **ATOM** | Phase 8 | NEW | Discover — free-cast pipeline (shared w/ Cascade) |
| 701.58a–h | **ATOM** | Phase 5-Pre | NEW | Cloak — manifest variant + ward {2} |
| 701.59a–c | **ATOM** | Phase 8 | NEW | Collect Evidence — GY exile cost + MV summation |
| 701.60a | TESTABLE | Phase 8 | NEW | Suspect designation |
| 701.60b | PURE-DEF | — | — | Suspected designation rules |
| 701.60c | TESTABLE | Phase 8 | NEW | Suspected grants menace + can't block |
| 701.60d | PURE-DEF | — | — | Re-suspect no-op |
| 701.61a | DEFERRED | Phase 8 | — | Forage |
| 701.62a–b | **ATOM** | Phase 5-Pre | NEW | Manifest Dread — thin wrapper on manifest |
| 701.63a | TESTABLE | Phase 8 | NEW | Endure (counters + token paths) |
| 701.63b | TESTABLE | Phase 8 | NEW | Endure 0 no-op |
| 701.64a | TESTABLE | Phase 8 | NEW | Harness + guard |
| 701.64b | PURE-DEF | — | — | Harnessed designation |
| 701.65a–b | **ATOM** | Phase 8 | NEW | Airbend — exile + persistent cast-from-exile for {2} |
| 701.66a–b | **ATOM** | Phase 5-Layers/7 | NEW | Earthbend — land animation + delayed trigger |
| 701.67a–c | **ATOM** | Phase 8 | NEW | Waterbend — tap-for-generic cost extension |
| 701.68a | TESTABLE | Phase 8 | NEW | Blight |
| 701.68b | TESTABLE | Phase 8 | NEW | Blight legality |
| 701.68c | PURE-DEF | — | — | "Blighted creature" term |
| 701.68d | PURE-DEF | — | — | Blight trigger timing |

## Classification Totals

| Category | Count |
|----------|-------|
| ATOM / TESTABLE tests | ~123 individual test specs (post-audit: +5 from 700.5a-002, 701.12d-001, 701.29a-001, 701.41a-002, reclassifications) |
| BOUNDARY-DEF | 3 (Historic, Modified, Outlaw) |
| PURE-DEF | ~34 sub-rules (701.18b reclassified to DEFERRED) |
| ALREADY-IMPLEMENTED | 13 sub-rules (Activate, Cast, Counter×2, Destroy, Discard, Exile, Play, Shuffle, Tap, Untap, "Dies") |
| DEFERRED | ~50 sub-rules (post-audit: 701.29a, 701.12d promoted to TESTABLE; 701.22c, 701.23i, 701.50d moved Phase 9→Phase 8; 701.18b added) |
| OUT-OF-SCOPE | ~15 sub-rules (Planechase, Archenemy, Unstable, Unfinity, THG) |

## Composition Tests

These tests exercise multi-rule interactions spanning rules classified in this session.

**COMP-7A-001: Sacrifice an indestructible creature bypasses destroy replacement (701.21a + 701.8b)**
- **Minimal Board:** Creature C (indestructible). P0 must sacrifice a creature.
- **Action:** P0 sacrifices C.
- **Expected Result:** C goes to graveyard. Indestructible irrelevant (sacrifice ≠ destroy per 701.8b). No regeneration shield applies (701.21a).
- **Phase:** Phase 5-Pre / Phase 6

**COMP-7A-002: Fight + Lifelink + Non-combat damage flag (701.14a + 701.14d + 702.15)**
- **Minimal Board:** Creature A (3/3, lifelink). Creature B (4/4). P0 at 10 life.
- **Action:** A fights B.
- **Expected Result:** A takes 4 non-combat damage (from B). B takes 3 non-combat damage (from A). Lifelink triggers on A's damage → P0 gains 3 life (now 13). "Whenever deals combat damage" does NOT trigger.
- **Phase:** Phase 8

**COMP-7A-003: Proliferate + Adapt guard interaction (701.34a + 701.46a)**
- **Minimal Board:** Creature C with "Adapt 2" and 0 +1/+1 counters. P0 also controls Creature D with 1 +1/+1 counter.
- **Action:** P0 adapts C (gets 2 +1/+1 counters). Then P0 proliferates, choosing C and D.
- **Expected Result:** C now has 3 +1/+1 counters. D now has 2 +1/+1 counters. If P0 tries to adapt C again → nothing (already has counters, 701.46a guard).
- **Phase:** Phase 8

**COMP-7A-004: Regeneration shield vs. "can't be regenerated" + destroy (701.19a + 701.19c + 701.8a)**
- **Minimal Board:** Creature C (3/3). P0 activates "Regenerate C" (shield created). Then opponent's effect says "C can't be regenerated this turn. Destroy C."
- **Action:** Destroy resolves.
- **Expected Result:** Shield exists but can't apply (701.19c). Destroy succeeds (701.8a). C goes to graveyard.
- **Phase:** Phase 6

**COMP-7A-005: Equipment unattach on creature sacrifice (701.3d + 701.21a)**
- **Minimal Board:** Equipment E attached to Creature C. P0 sacrifices C.
- **Action:** Sacrifice resolves. C goes to graveyard.
- **Expected Result:** E becomes unattached (701.3d — creature leaving battlefield counts as "becoming unattached"). E remains on battlefield with `attached_to = None`.
- **Phase:** Phase 5-Pre

**COMP-7A-006: Double P/T + layer ordering with set-P/T effect (701.10a + 613.4b/c)**
- **Minimal Board:** Creature C base 2/3. Continuous effect "C's base P/T is 0/1" (L7b). Then "Double C's power and toughness" resolves (L7c).
- **Action:** Layer system recalculates.
- **Expected Result:** L7b sets to 0/1. L7c doubles: +0/+1. Final: 0/2. The doubling snapshots the L7b result, not the printed base.
- **Phase:** Phase 5-Layers

## Gap Report

### Gaps Requiring New Tickets

| Gap | Rule(s) | Phase | Priority |
|-----|---------|-------|----------|
| Token creation primitive | 701.7a | Phase 8 | High — many keyword actions create tokens |
| Fight keyword action | 701.14a–d | Phase 8 | Medium — common green mechanic |
| Scry primitive | 701.22a | Phase 8 | High — extremely common |
| Search primitive | 701.23a–d | Phase 8 | High — fundamental (fetchlands, tutors) |
| Surveil primitive | 701.25a | Phase 8 | Medium — common in black/blue |
| Mill primitive | 701.17a–b | Phase 8 | Medium |
| Reveal primitive | 701.20a | Phase 8 | Medium — used by many effects |
| Proliferate | 701.34a | Phase 8 | Medium |
| Destroy delta tagging (destroy vs sacrifice) | 701.8b | Phase 6 | High — needed for replacement effects |
| Regeneration shield system | 701.19a–c, 701.8c | Phase 6 | Medium — classic mechanic |
| Double/Triple P/T as L7c effect | 701.10a–c, 701.11a–c | Phase 5-Layers | Medium |
| Damage doubling replacement | 701.10g | Phase 6 | Medium |
| Exchange effects | 701.12a–c,g | Phase 8 | Low — niche |
| Random/opponent discard variants | 701.9b | Phase 8 | Low |
| Attach/Unattach primitives | 701.3a–d | Phase 5-Pre | High — needed for Equipment/Aura |

### Already Covered — No Action Needed

- **Activate** (701.2a): Phase 2 `activate_ability`
- **Cast** (701.5a): Phase 2 `cast_spell`
- **Counter** (701.6a–b): Phase 2 `counter_spell` + fizzle
- **Destroy** (701.8a): Phase 2 `Primitive::Destroy`
- **Discard** (701.9a): Phase 1 cleanup discard + `choose_discard` DP
- **Exile** (701.13a): Phase 2 `move_object` to exile
- **Play land** (701.18a): Phase 1 `play_land`
- **Shuffle** (701.24a): Phase 1 library randomization
- **Tap/Untap** (701.26a–b): Phase 1 `GameAction::Tap/Untap`
- **"Dies"** (700.4): Phase 1 battlefield→graveyard zone change

### Dependency Chain for Upcoming Phases

```
Phase 5-Pre:  Attach/Unattach (T15), Sacrifice expansion (T15), Modal spells (T18),
              Face-down infrastructure (Manifest/Cloak/Manifest Dread),
              Skip-untap tracking (Exert), Special action framework (TurnFaceUp)
     ↓
Phase 5-Layers:  Double/Triple P/T (L7c), Devotion partial-layer,
                 Earthbend land animation (L4 type + L7b P/T)
     ↓
Phase 6:  Destroy delta tagging (D20), Regeneration shields, Damage doubling replacement,
          Manifest ETB prohibition (701.40f)
     ↓
Phase 7:  Modal triggered abilities (700.2b), Shuffle triggers (701.24f),
          Earthbend delayed triggers (701.66), Exert linked triggers (701.43d)
     ↓
Phase 8:  Token creation, Fight, Scry, Search, Surveil, Mill, Reveal, Proliferate,
          Investigate, Bolster, Monstrosity, Adapt, Detain, Populate, Support,
          Suspect, Endure, Harness, Blight, Exchange, Random discard, Devotion,
          Historic/Modified/Outlaw predicates,
          Explore, Amass, Connive, Ring Tempts You, Villainous Choice,
          Discover (+ CastPermission::FreeFromExile),
          Airbend (+ CastPermission::AlternateCostFromExile), Collect Evidence,
          Waterbend (+ tap-for-generic cost pipeline),
          Emblem system, Designation registry
     ↓
Phase 9:  Goad, Vote, Transform/Convert, Meld, Incubate (DFC anchor, D3),
          Venture (arch notes recorded), multiplayer APNAP variants
```

---

## Reclassification Summary

| Rule | Old Classification | New Classification | Rationale |
|------|-------------------|-------------------|-----------|
| 701.40 (Manifest) | DEFERRED Phase 8 | **ATOM Phase 5-Pre** | Face-down infrastructure needed by 5 mechanics |
| 701.43 (Exert) | DEFERRED Phase 8 | **ATOM Phase 5-Pre** | Untap-step modification, shared pattern |
| 701.44 (Explore) | DEFERRED Phase 8 | **ATOM Phase 8** | Multi-step keyword, LKI testing |
| 701.47 (Amass) | DEFERRED Phase 8 | **ATOM Phase 8** | Conditional creation + subtype granting |
| 701.49 (Venture) | DEFERRED Phase 8 | **DEFERRED Phase 9** | Genuinely unique infrastructure, low card count; arch notes added |
| 701.50 (Connive) | DEFERRED Phase 8 | **ATOM Phase 8** | Draw-discard-counter pattern |
| 701.53 (Incubate) | DEFERRED Phase 8 | **ATOM Phase 9 (D3)** | Simplest DFC case, anchor for D3 design |
| 701.54 (Ring Tempts) | DEFERRED Phase 8 | **ATOM Phase 8** | Emblem + designation + progressive abilities |
| 701.55 (Villainous Choice) | DEFERRED Phase 9 | **ATOM Phase 8** | Works in 2-player; impossible-branch is architecturally significant |
| 701.57 (Discover) | DEFERRED Phase 8 | **ATOM Phase 8** | Same pattern as Cascade; free-cast pipeline |
| 701.58 (Cloak) | DEFERRED Phase 8 | **ATOM Phase 5-Pre** | Manifest variant, shares face-down infra |
| 701.59 (Collect Evidence) | DEFERRED Phase 8 | **ATOM Phase 8** | GY exile as cost, MV summation |
| 701.62 (Manifest Dread) | DEFERRED Phase 8 | **ATOM Phase 5-Pre** | Thin wrapper on manifest |
| 701.65 (Airbend) | DEFERRED Phase 8 | **ATOM Phase 8** | Exile + cast-from-exile for {2}; shared pipeline w/ Discover |
| 701.66 (Earthbend) | DEFERRED Phase 8 | **ATOM Phase 5-Layers/7** | Land animation + delayed trigger patterns |
| 701.67 (Waterbend) | DEFERRED Phase 8 | **ATOM Phase 8** | Cost pipeline extension; shared with convoke/improvise |

## New Architectural Subsystems Identified

| Subsystem | Rules Driving It | First Needed By | Also Used By |
|-----------|-----------------|-----------------|--------------|
| **Face-down permanent infrastructure** | 701.40, 701.58, 701.62 | Phase 5-Pre | Morph (702.37), Disguise (702.168) |
| **Designation registry** (`HashSet<Designation>`) | 701.54, 701.37, 701.60, 701.64 | Phase 8 | Renowned, Monstrous, Suspected, Harnessed |
| **Emblem system** | 701.54 | Phase 8 | Planeswalker ultimates |
| **Skip-untap tracking** | 701.43 | Phase 5-Pre | Freeze effects, Stasis, "doesn't untap" |
| **Special action framework** | 701.40b, 701.58b | Phase 5-Pre | Morph face-up, unmorph, land play (already exists) |
| **Cast-from-exile pipeline** (free + alternate cost) | 701.57, 701.65 | Phase 8 | Cascade (702.84), Suspend, Adventure |
| **Tap-for-generic cost extension** | 701.67 | Phase 8 | Convoke (702.51), Improvise (702.126) |
| **DFC back-face on CardData** | 701.53 | Phase 9 (D3) | Transform, MDFCs, Daybound |
| **Delayed triggered abilities** | 701.66 | Phase 7 | Oblivion Ring pattern, many cards |

--- End of Session 7A ---
