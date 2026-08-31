# Session 4: Atomic Test Specifications — Zones (Chapter 4) & Turn Structure (Chapter 5)

> **CR Sections:** 400–408 (Zones), 500–514 (Turn Structure)
> **Generated:** 2026-04-03
> **Informed by:** design_doc.md, roadmap.md, implementation-plan-final.md, CR Chapters 4 & 5

---

## Scope

- **Chapter 4 (Zones):** Rules 400.1–400.12, 401.1–401.7, 402.1–402.3, 403.1–403.5, 404.1–404.3, 405.1–405.6h, 406.1–406.8, 407.1–407.4, 408.1–408.3
- **Chapter 5 (Turn Structure):** Rules 500.1–500.12, 501.1, 502.1–502.4, 503.1–503.2, 504.1–504.2, 505.1–505.6b, 506.1–506.7g, 507.1–507.2, 508.1–508.8, 509.1–509.4b, 510.1–510.4, 511.1–511.3, 512.1, 513.1–513.2, 514.1–514.3a

---

## Chapter 4: Zones (400–408)

### 400. General

**400.1** — PURE-DEF. Names the seven zones. No independent mechanical consequence beyond zone tracking, which is implicit in the engine's architecture.

---

**400.2** — BOUNDARY-DEF. Defines public vs hidden zones.

**ATOM-400.2-001**
- **Rule:** 400.2 — Graveyard, battlefield, stack, exile, ante, and command are public zones. Library and hand are hidden zones.
- **Mechanism:** Zone visibility classification. Engine must correctly report which zones are public (all cards face-up by default) vs hidden (cards face-down by default).
- **Minimal Board:** A game state with cards in library, hand, battlefield, graveyard, stack, and exile.
- **Action:** Query visibility status of each zone.
- **Expected Result:** Library and hand return "hidden"; battlefield, graveyard, stack, exile return "public." Even if all cards in a hidden zone happen to be revealed, the zone itself is still classified as hidden.
- **Phase:** Backlog — information model (was Phase 1)
- **Ticket:** NEW — zone visibility classification query

---

**400.3** — TESTABLE. Objects go to owner's zone for library/graveyard/hand.

**ATOM-400.3-001**
- **Rule:** 400.3 — If an object would go to any library, graveyard, or hand other than its owner's, it goes to its owner's corresponding zone.
- **Mechanism:** `move_object` must route to owner's zone regardless of controller.
- **Minimal Board:** Player A controls a creature owned by Player B (via control-change effect).
- **Action:** Destroy the creature (moves to graveyard).
- **Expected Result:** Creature goes to Player B's graveyard, not Player A's.
- **Phase:** Phase 1 (zone transitions) — verified in Phase 3 audit
- **Ticket:** ALREADY-IMPLEMENTED — `move_object` routes to `obj.owner`'s graveyard per design_doc §3.9 / audit memory

---

**400.4** — PURE-DEF (parent rule). Introduces the concept that certain card types can't enter certain zones.

**400.4a** — TESTABLE. Instant/sorcery cards can't enter the battlefield.

**ATOM-400.4a-001**
- **Rule:** 400.4a — If an instant or sorcery card would enter the battlefield, it remains in its previous zone.
- **Mechanism:** Zone guard in `move_object` or `init_zone_state` preventing instant/sorcery from entering battlefield.
- **Minimal Board:** An instant card in the graveyard.
- **Action:** An effect attempts to return the instant card to the battlefield.
- **Expected Result:** The instant card remains in the graveyard. No error — the move is silently prevented.
- **Phase:** Phase 5 Pre-Work (T21a)
- **Ticket:** T21a — Zone guards + CastInfo carried to permanent

---

**400.4b** — OUT-OF-SCOPE. Conspiracy, phenomenon, plane, scheme, vanguard cards can't leave the command zone. These card types are not in scope per roadmap.md.

---

**400.5** — TESTABLE. Order of objects in library, graveyard, and stack can't be changed except by effects/rules.

**ATOM-400.5-001**
- **Rule:** 400.5 — The order of objects in a library, graveyard, or on the stack can't be changed except when effects or rules allow it.
- **Mechanism:** Library, graveyard, and stack must be ordered collections (Vec, not HashSet). Insertion and removal must preserve ordering.
- **Minimal Board:** Library with known top-3 cards.
- **Action:** Draw one card.
- **Expected Result:** The drawn card is the top card. Remaining library order is preserved.
- **Phase:** Phase 1 (zone fundamentals)
- **Ticket:** ALREADY-IMPLEMENTED — zones use Vec<ObjectId>

---

**400.6** — TESTABLE. Zone-change replacement effect look-ahead. When an object moves to a public zone, the owner looks at it for abilities that affect the move; replacement effects apply before the move.

**Audit note:** These tests document the zone-change-specific aspects of replacement effects. The full replacement effect system is rule 614 (Chapter 6). These tests should be **cross-referenced** with the 614.x session but do NOT need to be folded in — 400.6 specifically governs the "look-ahead" behavior where the game checks an object's destination-zone abilities *before* the move happens, which is mechanically distinct from the general replacement effect framework. Keep these here as the zone-transition entry point; the 614 session will cover the general replacement effect engine.

**ATOM-400.6-001**
- **Rule:** 400.6 — Replacement effects are applied to zone-change events before the object moves.
- **Mechanism:** `execute_action(ZoneChange)` must run through `apply_replacement_effects` before performing the move. The object's abilities (and other replacement effects) are checked.
- **Minimal Board:** A creature with "enters the battlefield tapped" (e.g., a tapland).
- **Action:** Play the tapland.
- **Expected Result:** The land enters the battlefield tapped (replacement effect modifies the zone-change event).
- **Phase:** Phase 6 (Replacement Effects) — per roadmap.md
- **Ticket:** NEW — replacement effects on zone transitions (D8 in roadmap deferred items)

**ATOM-400.6-002**
- **Rule:** 400.6 — When contradictory effects apply to the same zone change, the object's controller (or owner if no controller) chooses.
- **Mechanism:** When multiple replacement effects try to modify the same zone change in contradictory ways, the controller is prompted via DecisionProvider.
- **Minimal Board:** A creature dying with two simultaneous "destroy" replacement effects that send it to different zones (e.g., one exiles, one returns to hand).
- **Action:** The creature would be destroyed.
- **Expected Result:** Controller chooses which replacement effect to apply. The creature goes to the chosen zone.
- **Phase:** Phase 6 (Replacement Effects)
- **Ticket:** NEW — replacement effect conflict resolution (rule 616)

---

**400.7** — TESTABLE. New object identity on zone change.

**Design decision — Epoch-stamp model for zone-change identity:** Our engine **persists ObjectId across zone changes** (the same `ObjectId` follows a card from hand → stack → battlefield → graveyard). This is architecturally necessary for alchemy-style perpetual effects (if you assign a new ID on zone change, you lose the ability to track perpetual modifications without a side-channel mapping table). To implement 400.7's "new object" semantics on top of persistent IDs, we use an **epoch stamp**:

- `GameState` holds a global monotonic `zone_change_epoch: u64`, incremented on every `move_object` call (and on same-zone re-exile per 400.8).
- Each `GameObject` holds `last_zone_change_epoch: u64`, set to the current global epoch whenever it changes zones.
- Any effect, targeting reference, or delayed trigger that needs to track an object stores an `ObjectRef { id: ObjectId, zone_epoch: u64 }` at the moment it acquires the reference.
- At resolution: `game.objects[ref.id].last_zone_change_epoch != ref.zone_epoch` → the object has changed zones since the reference was taken → the reference is stale (target is illegal / effect fizzles).

This is NOT a generation counter (we don't care *how many* zone changes occurred) — it's a timestamp. Each effect independently answers "has this object changed zones since I looked at it?" with a single `u64` comparison. No clearing, no reference counting, no coordination between concurrent trackers. Cost: one `u64` field per `GameObject` (8 bytes), one global `u64` increment per zone change.

A boolean dirty flag was considered but rejected: with multiple effects simultaneously tracking the same object, there's no safe point to clear the flag without reference counting, which is more complex than the epoch. The epoch is simpler, O(1), and stateless from each effect's perspective.

**Loop interaction / overflow analysis:** A `u64` holds ~1.8×10^19 values. Could a loop exhaust it? No, because the epoch and the loop shortcut system (D26, rule 727) are **orthogonal by design**:

- **Pre-D26 (Phase 7 stub):** The engine has a `MAX_TRIGGER_ITERATIONS` safety cap. Runaway loops hit the cap and stop, same as Arena's "too many triggers" cutoff. Even at 1M zone changes/turn × 1000 turns = 10^9 — a rounding error on `u64`.
- **Post-D26 (Phase 9 full):** Loop shortcuts *don't execute individual iterations*. A player declares "I do this loop N times," the engine identifies the loop body, computes the **net effect per iteration**, and applies the result in one step. The *quantities produced* (life, tokens, etc.) are tracked by `GameNumber` (which uses symbolic representation: `Finite(u64)` / `Shortcut { per_iter, iterations }` / `Relative`). The epoch bumps a **small finite number of times** — once per distinct zone-change type in the loop body, not once per iteration. If the loop body involves "exile creature, return creature," the shortcut applies the net effect and bumps the epoch once to invalidate pre-shortcut references.
- **Without any loop handling (current state):** The `--max-turns` fuzz harness safety valve prevents runaway execution.

The scenario where the epoch *would* overflow — actually executing 10^18+ individual `move_object` calls — means the loop system is broken and the game should have been shortcutted or capped long before reaching that point. The epoch is a **physical execution counter**, not a logical iteration counter. `GameNumber` handles the logical counts; the epoch just needs to reflect "something happened since you last looked."

See `rule-400-7-details.md` for the full article by a certified MTG judge detailing all nine exceptions and their interactions — several of the article's examples (Rancor, Necrotic Plague, Snapcaster Mage, Goblin Dark-Dwellers, Long Road Home) serve as stress tests for this architecture.

**ATOM-400.7-001**
- **Rule:** 400.7 — An object that moves from one zone to another becomes a new object with no memory of its previous existence.
- **Mechanism:** When `move_object` transitions an object between zones, it increments the global `zone_change_epoch` and stamps the object's `last_zone_change_epoch`. Any `ObjectRef` holding a stale epoch will fail validation at resolution.
- **Minimal Board:** A creature on the battlefield targeted by a delayed trigger ("at end of turn, destroy target creature"). The trigger stores `ObjectRef { id, zone_epoch: 42 }`. The creature is bounced to hand via an instant, then replayed. The creature's `last_zone_change_epoch` is now 44 (two zone changes: battlefield→hand, hand→stack→battlefield).
- **Action:** The delayed trigger tries to resolve. It checks `obj.last_zone_change_epoch (44) != stored_epoch (42)`.
- **Expected Result:** The trigger fizzles — the epoch mismatch means the object has changed zones. No damage, no counters, no attachments carry over to the replayed creature.
- **Phase:** Phase 5 Pre-Work (epoch-stamp infrastructure) + Phase 7 (delayed triggers)
- **Ticket:** NEW — `zone_change_epoch` field on `GameObject`, global epoch counter on `GameState`, `ObjectRef` type for stale-reference checking.

**ATOM-400.7-002**
- **Rule:** 400.7 — Blink (exile-and-return) creates a new object with no memory.
- **Mechanism:** A creature exiled by "Long Road Home" and returned at end of turn is a completely new permanent — no damage, no counters (except the +1/+1 from Long Road Home's own effect), no continuous effects, no aura attachments, fresh summoning sickness. The epoch stamp on the object changes on each zone transition (battlefield→exile, exile→battlefield), invalidating all pre-blink references.
- **Minimal Board:** A creature with 2 damage, a +1/+1 counter, and an Aura attached. "Long Road Home" exiles it.
- **Action:** At end of turn, the creature returns to the battlefield.
- **Expected Result:** Returned creature has 0 damage, no +1/+1 counter (but gets the +1/+1 from Long Road Home's own effect), no Aura, has summoning sickness. The Aura that was attached goes to graveyard via SBA. Any effects referencing the pre-blink epoch fail validation.
- **Phase:** Phase 8 (blink effects), but the epoch-stamp infrastructure is Phase 5 Pre-Work
- **Ticket:** NEW — blink identity reset test (Long Road Home pattern)

**ATOM-400.7-003**
- **Rule:** 400.7 — Multiple simultaneous trackers: two effects targeting the same object both see stale epoch after a single zone change.
- **Mechanism:** Effect A and Effect B both store `ObjectRef` to the same creature. The creature is blinked once. Both effects should independently detect the stale epoch.
- **Minimal Board:** A creature on the battlefield. Effect A (delayed trigger: "destroy at end of turn") and Effect B (continuous: "this creature gets +2/+2 until end of turn") both reference it. The creature is blinked.
- **Action:** Both effects try to apply/resolve after the blink.
- **Expected Result:** Both detect `obj.last_zone_change_epoch != stored_epoch` independently. Effect A fizzles, Effect B no longer applies. Neither effect needs to "clear" anything — each checks its own stored epoch against the object's current epoch.
- **Phase:** Phase 5 Pre-Work (epoch-stamp infrastructure)
- **Ticket:** Same as ATOM-400.7-001

---

**400.7a** — TESTABLE. Spell-to-permanent continuity for characteristic-changing effects.

**Audit note:** Better example per article: Deathlace/Thoughtlace targeting a creature spell on the stack changes its color permanently. When the spell resolves, the permanent retains the color change. This is the canonical example from the CR itself.

**ATOM-400.7a-001**
- **Rule:** 400.7a — Effects from spells, activated abilities, and triggered abilities that change the characteristics of a permanent spell on the stack continue to apply to the permanent that spell becomes.
- **Mechanism:** When a permanent spell resolves, continuous effects targeting the spell on the stack must transfer to the resulting permanent.
- **Minimal Board:** A white creature spell (e.g., Savannah Lions) on the stack. Player casts Deathlace (or Thoughtlace) targeting it, changing its color to black (or blue).
- **Action:** The creature spell resolves, becoming a permanent.
- **Expected Result:** The permanent on the battlefield is black (or blue), not white. The color-change effect from the spell cast on the stack persists to the permanent.
- **Phase:** Phase 5 (Continuous Effects) — effects on stack objects transitioning to permanents
- **Ticket:** NEW — stack-to-permanent effect continuity (related to L02 duration tracking)

**ATOM-400.7a-002**
- **Rule:** 400.7a (Article example: Artificial Evolution on Goblin King) — Text-changing effects on a permanent spell on the stack persist to the permanent.
- **Mechanism:** Artificial Evolution changes "Goblin" to "Human" on a Goblin King spell. On the battlefield, the permanent gives Humans +1/+1 and is itself a Human. Its *name* remains "Goblin King" (text-changing effects can't change names).
- **Minimal Board:** A Goblin King spell on the stack. Artificial Evolution resolves targeting it, changing "Goblin" to "Human."
- **Action:** Goblin King resolves.
- **Expected Result:** On the battlefield, the permanent's name is still "Goblin King", but its creature type is Human (not Goblin), and its static ability gives other Humans +1/+1 and mountainwalk.
- **Phase:** Phase 5 (Continuous Effects — text-changing effects, L01/L02)
- **Ticket:** NEW — text-changing effect persistence across stack→battlefield. Note: text-changing effects are a niche Phase 8+ concern but the *mechanism* of effect persistence is Phase 5.

---

**400.7b** — TESTABLE. Static ability grants from spells continue to the permanent.

**ATOM-400.7b-001**
- **Rule:** 400.7b — Effects from static abilities that grant an ability to a permanent spell that functions on the battlefield continue to apply to the permanent that spell becomes.
- **Mechanism:** Static ability effects on a spell (e.g., from a permanent that says "creature spells you control have haste") must persist when the spell resolves into a permanent.
- **Minimal Board:** A permanent granting "creature spells you control have haste" on the battlefield. A creature spell on the stack.
- **Action:** The creature spell resolves.
- **Expected Result:** The resulting creature permanent has haste (the static ability continues to apply via the layer system, not via a transferred effect — this is actually just the layer system working normally since static abilities re-evaluate continuously).
- **Phase:** Phase 5 (Continuous Effects / Layers)
- **Ticket:** L06 (Layer 6: abilities) — static ability application handles this naturally

---

**400.7c** — TESTABLE. Prevention effects on spell continue to permanent.

**Audit note:** Article example: Nicole activates Circle of Protection: Red choosing opponent's Ball Lightning spell. When Ball Lightning resolves, the prevention effect applies to the *permanent's* combat damage. Non-example from article: Hallow on Chandra, Flamecaller prevents damage from Chandra's abilities (same source) but NOT from tokens Chandra creates (different source). The "until end of turn" on Hallow is specifically because of this rule — without it, the prevention would persist indefinitely since the spell→permanent transition doesn't break it.

**ATOM-400.7c-001**
- **Rule:** 400.7c — Prevention effects that apply to damage from a permanent spell on the stack continue to apply to damage from the permanent that spell becomes.
- **Mechanism:** Prevention effects targeting a spell must transfer to the resulting permanent.
- **Minimal Board:** Circle of Protection: Red is on the battlefield. Opponent casts Ball Lightning ({R}{R}{R} 6/1 haste trample). COP:Red's ability is activated choosing Ball Lightning while it's on the stack.
- **Action:** Ball Lightning resolves, attacks, and deals combat damage.
- **Expected Result:** The combat damage from Ball Lightning (the permanent) is prevented because the prevention effect from COP:Red targeted the spell and persists to the permanent per 400.7c.
- **Phase:** Phase 6 (Replacement/Prevention Effects)
- **Ticket:** NEW — prevention effect continuity across stack→battlefield transition

**ATOM-400.7c-002**
- **Rule:** 400.7c (Non-example from article) — Prevention effects on a source don't transfer to *other* objects that source creates.
- **Mechanism:** Hallow on Chandra, Flamecaller prevents damage from Chandra herself, but NOT from tokens Chandra creates (tokens are separate objects/sources).
- **Minimal Board:** Hallow cast targeting Chandra, Flamecaller spell on the stack. Chandra resolves. Chandra's +1 creates two 3/1 Elemental tokens.
- **Action:** The tokens attack and deal damage.
- **Expected Result:** Token damage is NOT prevented — the tokens are separate sources from Chandra. Chandra's own -X ability damage WOULD be prevented (same source).
- **Phase:** Phase 6 (Prevention Effects — source tracking)
- **Ticket:** NEW — prevention effect source identity (tokens ≠ source permanent)

---

**400.7d** — TESTABLE. Permanent can reference spell cast information.

**ATOM-400.7d-001**
- **Rule:** 400.7d — An ability of a permanent can reference information about the spell that became that permanent as it resolved, including what costs were paid.
- **Mechanism:** `CastInfo` on `BattlefieldEntity` carries mana spent, X value, additional costs paid from the stack entry.
- **Minimal Board:** A creature with kicker, cast with kicker paid.
- **Action:** Query the permanent's CastInfo.
- **Expected Result:** `cast_info.additional_costs_paid` includes the kicker cost. An "if kicked" ability can check this.
- **Phase:** Phase 5 Pre-Work (T21a)
- **Ticket:** T21a — CastInfo carried to permanent

---

**400.7e** — TESTABLE. Zone-change triggers can find the new object in a public zone.

**Audit note:** The article distinguishes this sharply. Rancor's LTB trigger needs this rule because it's a leaves-the-battlefield trigger (checks game state *before* the event). Without 400.7e, Rancor-on-the-battlefield (the trigger source) couldn't find Rancor-in-the-graveyard (a new object). Contrast with Vigor's "from anywhere" trigger which checks game state *after* the event — Vigor is already in the graveyard when the trigger resolves, so no special rule needed. Progenitus doesn't apply at all (replacement effect, not triggered ability).

**ATOM-400.7e-001**
- **Rule:** 400.7e — Abilities that trigger when an object moves from one zone to another can find the new object in the destination zone, if that zone is public.
- **Mechanism:** Triggered abilities referencing "this card" after a zone change can locate the new object in the public destination zone.
- **Minimal Board:** Rancor (Enchantment — Aura, "When this Aura is put into a graveyard from the battlefield, return it to its owner's hand") is attached to a creature.
- **Action:** The creature is destroyed; Rancor goes to graveyard via SBA (unattached aura).
- **Expected Result:** Rancor's LTB trigger fires, finds Rancor-the-new-object in the graveyard (public zone), and returns it to hand.
- **Phase:** Phase 7 (Triggered Abilities) — LTB triggers with public zone lookup
- **Ticket:** NEW — triggered ability zone-change object tracking

**ATOM-400.7e-002**
- **Rule:** 400.7e (Non-example: Vigor) — "From anywhere" triggers don't need this rule.
- **Mechanism:** Vigor's "When Vigor is put into a graveyard from anywhere, shuffle it into its owner's library" is NOT a leaves-the-battlefield trigger (it says "from anywhere"). It checks game state *after* the event, so Vigor is already in the graveyard — no special lookup needed.
- **Minimal Board:** Vigor is on the battlefield. It is destroyed.
- **Action:** Vigor goes to graveyard. Trigger fires.
- **Expected Result:** Trigger fires and Vigor is shuffled into library. This works WITHOUT 400.7e because the trigger checks post-event state (Vigor is already the current object in graveyard).
- **Phase:** Phase 7 (Triggered Abilities — "from anywhere" trigger distinction)
- **Ticket:** NEW — "from anywhere" triggers don't require 400.7e lookup

---

**400.7f** — TESTABLE. Aura LTB triggers find the Aura in graveyard when enchanted permanent leaves.

**ATOM-400.7f-001**
- **Rule:** 400.7f — Abilities that trigger when an enchanted permanent leaves the battlefield can find the Aura in its owner's graveyard if put there simultaneously.
- **Mechanism:** When an enchanted permanent and its Aura both go to graveyard at the same time (e.g., SBA for unattached Aura), the LTB trigger can reference the Aura's new object in the graveyard.
- **Minimal Board:** A creature enchanted by an Aura with "When enchanted creature leaves the battlefield, return this card to your hand."
- **Action:** Destroy the creature. SBA puts unattached Aura into graveyard.
- **Expected Result:** The triggered ability finds the Aura in the graveyard and returns it to hand.
- **Phase:** Phase 7 (Triggered Abilities) + Phase 5 Pre-Work (T15 Aura SBAs)
- **Ticket:** NEW — Aura LTB trigger with simultaneous graveyard movement

---

**400.7g–400.7k, 400.7m** — These sub-rules all deal with effects granting cast/play permissions and finding the new object after casting/playing. They are all **Phase 8+** concerns (zone-casting permissions, madness, stickers).

- **400.7g:** DEFERRED (cast-permission continuity — Phase 8 when cast-from-exile cards arrive)
- **400.7h:** DEFERRED (effect-grants-cast finding new object — Phase 8)
- **400.7i:** DEFERRED (effect-grants-land-play finding new object — Phase 8)
- **400.7j:** TESTABLE but low priority — effects moving objects to public zones can find them. Partially implemented (cost objects like sacrifice move to graveyard and effects can reference them).

**ATOM-400.7j-001**
- **Rule:** 400.7j — If an effect causes an object to move to a public zone, other parts of that effect can find that object.
- **Mechanism:** When an effect moves an object to a public zone (e.g., exile), subsequent parts of the same effect can reference the new object.
- **Minimal Board:** A spell that says "Exile target creature, then put a +1/+1 counter on that card."
- **Action:** Resolve the spell.
- **Expected Result:** The creature is exiled, then the effect finds the exiled card and places a counter on it.
- **Phase:** Phase 8 (Exile zone metadata, primitives)
- **Ticket:** NEW — effect self-referencing after zone change to public zone

- **400.7k:** DEFERRED (madness — Phase 8+)
- **400.7m:** OUT-OF-SCOPE (stickers — not in scope)

---

**400.8** — TESTABLE. Re-exiling an exiled object creates a new object identity.

**Audit note:** No known card currently exiles cards *in* exile. However, a hypothetical "Exile all cards in exile" could deny "return at end of turn" effects. The engine should structurally allow this (same-zone re-exile bumps the epoch) even if no card currently exercises it. This is a structural correctness test, not a card-specific test.

**ATOM-400.8-001**
- **Rule:** 400.8 — If an object in the exile zone is exiled, it doesn't change zones, but it becomes a new object that has just been exiled.
- **Mechanism:** When an exiled card is "exiled again," `move_object` (or a dedicated `re_exile` path) increments the global `zone_change_epoch` and updates the object's `last_zone_change_epoch` even though the physical zone doesn't change. Any `ObjectRef` holding a stale epoch will fail validation.
- **Minimal Board:** A card in exile, referenced by an effect via `ObjectRef { id, zone_epoch: 50 }` ("exile target card, return it at end of turn").
- **Action:** Another effect exiles the same card again (same-zone re-exile). Object's `last_zone_change_epoch` becomes 51.
- **Expected Result:** The original "return" effect checks `obj.last_zone_change_epoch (51) != stored_epoch (50)` → stale → effect loses track. The card stays in exile.
- **Phase:** Phase 8 (Exile zone metadata)
- **Ticket:** NEW — re-exile epoch bump for same-zone identity break (extends epoch-stamp infrastructure from 400.7)

---

**400.9** — DEFERRED. Face-down objects in command zone becoming new objects — Phase 9 (face-down mechanics).

**400.10** — DEFERRED. Command zone re-entry identity break — Phase 9 (Commander).

**400.11** — PURE-DEF. "Outside the game" is not a zone.

**400.11a** — PURE-DEF. Sideboard cards are outside the game.

**Audit note:** 400.11a + 400.11b together necessitate a "wish effect" test: cards that say "you may choose a card you own from outside the game" (e.g., Burning Wish) let you grab a card from the sideboard. This isn't doable until the match harness exists (need a sideboard concept). Flagging for Phase 8+ when wish effects come online.

**400.11b** — DEFERRED. Bringing cards from outside the game — Phase 8+ (Wish effects). See audit note above.

**400.11c** — DEFERRED. Cards outside the game can't be affected except by CDAs and specific effects — Phase 8+.

**400.12** — TESTABLE (reclassified from PURE-DEF).

**Audit note:** On reflection, this IS testable. "Shuffle your graveyard into your library" is an effect that acts on an entire zone, moving all cards from one zone into another. This is mechanically distinct from individual card moves and should be tested as a batch zone operation.

**ATOM-400.12-001**
- **Rule:** 400.12 — If an effect refers to doing something to a player's zone, it does so to all cards in that zone.
- **Mechanism:** An effect like "Shuffle your graveyard into your library" must move ALL cards from graveyard into library, then shuffle the library. This is a batch zone operation, not individual moves.
- **Minimal Board:** Player has 5 cards in graveyard and 20 cards in library.
- **Action:** Resolve "Shuffle your graveyard into your library."
- **Expected Result:** Graveyard is empty (0 cards). Library has 25 cards. Library is shuffled (randomized order). Each card that was in the graveyard is now in the library.
- **Phase:** Phase 8 (batch zone operations)
- **Ticket:** NEW — batch zone-to-zone move ("shuffle graveyard into library" pattern)

---

### 401. Library

**401.1** — PURE-DEF. Each player's deck becomes their library at game start. Implicit in `Game::setup`.

---

**401.2** — TESTABLE. Library is face-down; players can't look at or change card order.

**Audit note — Implementation thoughts:** The engine currently stores library as `Vec<ObjectId>` which is inherently ordered and opaque from the player's perspective. The question is about the *oracle/UI layer*: should there be a convenience feature (like Arena does) that tracks positional knowledge? Example: "Unexpectedly Absent" puts a nonland permanent into its owner's library just beneath the top X cards. Arena lets you visually trace that card as it moves toward the top. This is a **UI convenience, not a rules requirement** — the engine just needs to maintain positional correctness. The oracle layer could expose "cards whose position in library is known to player X" as a separate query, but this is firmly Phase 8+ polish and should NOT be confused with 401.5 (effects that explicitly reveal cards). Decision: library position tracking is a UI/oracle concern, not an engine concern. Defer to Phase 8+ if desired.

**ATOM-401.2-001**
- **Rule:** 401.2 — Each library must be kept in a single face-down pile. Players can't look at or change the order of cards in a library.
- **Mechanism:** Library contents must not be visible to any player by default. The engine must not expose library card identities through normal queries (only top-of-library when an effect permits).
- **Minimal Board:** A player with cards in their library.
- **Action:** Query library contents.
- **Expected Result:** The engine provides card count but not card identities (unless a "reveal top" or "look at" effect is active). Library order is immutable except through effects (draw, search, shuffle, scry, etc.).
- **Phase:** Phase 1 (zone fundamentals)
- **Ticket:** ALREADY-IMPLEMENTED — library is a `Vec<ObjectId>` with draw from end; no public visibility API

---

**401.3** — TESTABLE. Any player may count cards in any library.

**ATOM-401.3-001**
- **Rule:** 401.3 — Any player may count the number of cards remaining in any player's library at any time.
- **Mechanism:** Library card count must be publicly queryable for all players.
- **Minimal Board:** Two players with different library sizes.
- **Action:** Query each player's library size.
- **Expected Result:** Both queries succeed, returning the correct count.
- **Phase:** Phase 1 (zone fundamentals)
- **Ticket:** ALREADY-IMPLEMENTED — `player.library.len()` is accessible

---

**401.4** — TESTABLE. When multiple cards are put into a library at a specific position simultaneously, the owner arranges them.

**Audit note — DecisionProvider pattern:** This is another instance of the "give DP a set of objects, ask for ordering" pattern (also seen in 404.3 simultaneous graveyard ordering, 405.3 simultaneous trigger ordering). When we finalize the DecisionProvider trait, we should have a single `choose_ordering(&[ObjectId], context: OrderingContext) -> Vec<ObjectId>` method rather than separate methods for library ordering, graveyard ordering, etc. Flag for DP consolidation pass.

**Audit note — Event anonymization question:** Should the event log for this action be anonymized? i.e., "Player A ordered 3 cards on top of their library" vs "Player A put Card C, then Card B, then Card A on top of their library." Since the library is a hidden zone, the event log MUST NOT reveal card identities unless the cards were revealed as part of the effect (e.g., scry shows them to the player but not the opponent). The event should say "Player A ordered 3 cards on top of their library", and only the acting player should get an additional message to see identities. This is an EventLog formatting concern but worth flagging — the engine should emit a structured event with the ObjectIds, and the UI layer filters visibility per player.

**ATOM-401.4-001**
- **Rule:** 401.4 — If an effect puts two or more cards in a specific position in a library at the same time, the owner may arrange them in any order.
- **Mechanism:** When a scry-like or "put on top" effect places multiple cards, the DecisionProvider must be consulted for ordering.
- **Minimal Board:** Player has 3 cards to place on top of library (e.g., scry 3 effect).
- **Action:** Resolve the scry/put-on-top effect.
- **Expected Result:** DecisionProvider is called with the cards and the player chooses their order on top of the library.
- **Phase:** Phase 8 (Scry/Surveil primitives)
- **Ticket:** NEW — multi-card library placement ordering via DecisionProvider

---

**401.5** — TESTABLE (multi-clause). Top-of-library reveal rules during casting/activation.

**Audit note:** This IS multi-clause. The rule has two distinct parts: (1) the general rule that some effects let you look at the top card, and (2) the specific exception that during casting, a newly-revealed top card isn't visible until casting completes. These should be separate tests.

**ATOM-401.5-001**
- **Rule:** 401.5 (clause 1) — Some effects allow a player to play with the top card of their library revealed, or to look at the top card.
- **Mechanism:** A continuous effect like Future Sight ("Play with the top card of your library revealed") makes the top card of the player's library public information. This persists as long as the effect is active.
- **Minimal Board:** Player controls Future Sight ("Play with the top card of your library revealed"). Library has at least 2 cards.
- **Action:** Query the top card of the library.
- **Expected Result:** The top card's identity is visible to all players. When the top card changes (e.g., after drawing), the new top card is immediately revealed. Identity of other cards in the library is not revealed.
- **Phase:** Phase 8 (play-with-top-revealed effects)
- **Ticket:** NEW — "play with top card revealed" continuous effect

**ATOM-401.5-002**
- **Rule:** 401.5 (clause 2) — If the top card of the library changes while a spell is being cast, the new top card won't be revealed until the spell becomes cast.
- **Mechanism:** During the casting sequence (601.2a–i), if the top card changes (e.g., due to a mana ability that draws), the newly-revealed top card must not be visible until casting completes. This is a "freeze" on reveal updates during the casting process.
- **Minimal Board:** Player plays with top card revealed (e.g., Future Sight). Player casts a spell whose cost requires activating a mana ability that draws a card.
- **Action:** Cast the spell, activating draw-mana-ability mid-cast.
- **Expected Result:** The new top card is not revealed until the spell is fully cast (601.2i completes). During the casting window, the previous top card (now drawn) is no longer on top, but the new top card is not yet revealed.
- **Phase:** Phase 8 (play-with-top-revealed effects)
- **Ticket:** NEW — top-of-library reveal freeze during casting

**ATOM-401.5-003**
- **Rule:** 401.5 (clause 2, activation variant) — Same freeze applies when activating an ability.
- **Mechanism:** The reveal freeze also applies during ability activation, not just spell casting.
- **Minimal Board:** Same as 401.5-002 but with an activated ability instead of a spell.
- **Action:** Activate an ability whose cost involves drawing (or otherwise changing the top card).
- **Expected Result:** New top card not revealed until activation completes.
- **Phase:** Phase 8
- **Ticket:** Same as ATOM-401.5-002

---

**401.6** — DEFERRED. "Play with top revealed" card becoming unrevealed then re-revealed creates a new object. Requires Phase 8+ reveal tracking.

**401.7** — TESTABLE. "Put Nth from top" with fewer than N cards puts on bottom.

**ATOM-401.7-001**
- **Rule:** 401.7 — If an effect causes a player to put a card into a library "Nth from the top," and that library has fewer than N cards, the player puts that card on the bottom.
- **Mechanism:** Library insertion at position N must fall back to bottom insertion when library size < N.
- **Minimal Board:** Player with 1 card in library.
- **Action:** An effect instructs "put this card third from the top."
- **Expected Result:** The card is placed on the bottom of the library (since library has fewer than 2 cards).
- **Phase:** Phase 8 (library manipulation primitives)
- **Ticket:** NEW — library positional insertion with fallback

---

### 402. Hand

**402.1** — PURE-DEF. The hand is where drawn cards go. Starting hand size is normally seven. Implicit in `Game::setup`.

---

**402.2** — TESTABLE. Maximum hand size and cleanup discard.

**ATOM-402.2-001**
- **Rule:** 402.2 — Each player has a maximum hand size, normally seven. During cleanup, the player must discard excess cards.
- **Mechanism:** `Game::run_turn` cleanup step checks hand size against `max_hand_size` and calls `choose_discard` until hand size is at or below the maximum.
- **Minimal Board:** Player with 9 cards in hand, max hand size 7.
- **Action:** Enter cleanup step.
- **Expected Result:** Player is prompted to discard 2 cards. After discarding, hand size is 7.
- **Phase:** Phase 1 (Pre-Phase 3 work item 3.4)
- **Ticket:** ALREADY-IMPLEMENTED — design_doc §3.4, `Game::run_turn` cleanup discard

**ATOM-402.2-002**
- **Rule:** 402.2 — A player may have any number of cards in their hand (no maximum during other steps).
- **Mechanism:** No hand-size enforcement occurs outside the cleanup step.
- **Minimal Board:** Player draws cards during upkeep triggers, ending up with 10 cards in hand.
- **Action:** Main phase begins.
- **Expected Result:** No discard is forced. Player can hold 10 cards until cleanup.
- **Phase:** Phase 1
- **Ticket:** ALREADY-IMPLEMENTED — discard only happens in cleanup

---

**402.3** — TESTABLE (reclassified from PURE-DEF). Players can arrange and look at their own hand; can't look at opponent's.

**ATOM-402.3-001**
- **Rule:** 402.3 — A player can see and rearrange cards in their own hand but can't look at the cards in another player's hand.
- **Mechanism:** An oracle visibility query for another player's hand must return only the card count, not card identities. Own hand returns full identities.
- **Minimal Board:** Two players, each with 3 cards in hand. No reveal effects active.
- **Action:** Player A queries Player B's hand contents. Player A queries their own hand contents.
- **Expected Result:** Player B's hand query returns `{ count: 3, cards: None }`. Player A's own hand query returns `{ count: 3, cards: [CardId1, CardId2, CardId3] }` with full card data.
- **Phase:** Backlog — information model (was Phase 1)
- **Ticket:** NEW — hand visibility enforcement in oracle layer. Currently all `GameState` access is unrestricted; this needs a per-player visibility filter.

**ATOM-402.3-002**
- **Rule:** 402.3 + CR 701.16 (“Reveal”) — A temporary reveal effect makes specific cards in a hidden zone visible to ALL players.
- **Mechanism:** When an effect says "reveal" (e.g., Thoughtseize: "Target player reveals their hand"), ALL players in the game can see the revealed cards for the duration of the effect. The engine sets `revealed_to: AllPlayers` on the targeted cards (or zone-level reveal flag) for the effect's duration. After the effect finishes, visibility reverts to default.
- **Minimal Board:** Three players (A, B, C). Player B has 4 cards in hand. Thoughtseize resolves targeting Player B.
- **Action:** Player B's hand is revealed. Player A and Player C both query Player B's hand.
- **Expected Result:** Both Player A and Player C see all 4 card identities in Player B's hand. After the Thoughtseize effect completes (card chosen and discarded), Player B's remaining hand reverts to hidden — subsequent queries return count only.
- **Phase:** Phase 8 (reveal effects)
- **Ticket:** NEW — "reveal" keyword implementation with all-player visibility

**ATOM-402.3-003**
- **Rule:** 402.3 + CR 701.16a (“Look at”) — A "look at" effect makes cards visible ONLY to the effect's controller at resolution, not to all players.
- **Mechanism:** When an effect says "look at" (e.g., Telepathy: "You may look at target player's hand" or Gitaxian Probe: "Look at target player's hand"), only the controller of the resolving spell/ability sees the cards. Other players do NOT gain visibility. The engine sets `revealed_to: HashSet { controller_id }` on the targeted cards.
- **Minimal Board:** Three players (A, B, C). Player A resolves Gitaxian Probe targeting Player B.
- **Action:** Player C queries Player B's hand.
- **Expected Result:** Player A sees all card identities in Player B's hand. Player C sees only card count. The distinction between "reveal" (ATOM-402.3-002) and "look at" is critical for multiplayer correctness.
- **Phase:** Phase 8 (look-at effects)
- **Ticket:** NEW — "look at" keyword implementation with controller-only visibility

**ATOM-402.3-004**
- **Rule:** 402.3 — Persistent reveal effects (e.g., Telepathy: "Your opponents play with their hands revealed").
- **Mechanism:** A continuous effect that says "opponents play with their hands revealed" grants permanent visibility of all opponents' hand contents to all players (it's a "reveal," not a "look at"). This persists as long as the enchantment is on the battlefield.
- **Minimal Board:** Player A controls Telepathy. Player B has 3 cards in hand.
- **Action:** Player A and Player C both query Player B's hand.
- **Expected Result:** Both see all 3 card identities. If Telepathy is destroyed, visibility immediately reverts — subsequent queries return count only.
- **Phase:** Phase 8 (persistent reveal effects)
- **Ticket:** NEW — persistent "play with hand revealed" continuous effect

---

### 403. Battlefield

**403.1** — PURE-DEF. Battlefield starts empty. Permanents are kept in front of their controller. Definitional.

---

**403.2** — TESTABLE. Spells/abilities affect only the battlefield unless they mention another zone.

**ATOM-403.2-001**
- **Rule:** 403.2 — A spell or ability affects and checks only the battlefield unless it specifically mentions a player or another zone.
- **Mechanism:** Default target/effect scope is battlefield-only. "Destroy target creature" only considers creatures on the battlefield, not in other zones.
- **Minimal Board:** A creature on the battlefield and a creature card in the graveyard.
- **Action:** Cast "Destroy target creature."
- **Expected Result:** Only the battlefield creature is a legal target. The graveyard creature card is not targetable.
- **Phase:** Phase 2 (targeting)
- **Ticket:** ALREADY-IMPLEMENTED — targeting validation checks zone (battlefield) by default

**ATOM-403.2-002**
- **Rule:** 403.2 (positive case: player-targeting) — A spell that targets a player can't target a permanent.
- **Mechanism:** "Target player draws a card" has target type Player, not Permanent. Targeting validation rejects permanents.
- **Minimal Board:** A player and a creature on the battlefield.
- **Action:** Cast a spell targeting a player (e.g., Ancestral Recall targeting player). Attempt to target the creature instead.
- **Expected Result:** The creature is not a legal target — the spell targets players, not permanents.
- **Phase:** Phase 2 (targeting)
- **Ticket:** ALREADY-IMPLEMENTED — target type validation separates Player from Permanent

**ATOM-403.2-003**
- **Rule:** 403.2 (positive case: graveyard-targeting) — A spell that targets a card in a graveyard can't target a permanent on the battlefield.
- **Mechanism:** "Return target creature card from your graveyard to your hand" has zone filter Graveyard. Targeting validation rejects battlefield permanents.
- **Minimal Board:** A creature on the battlefield and a creature card in the graveyard.
- **Action:** Cast a graveyard-targeting spell. Attempt to target the battlefield creature.
- **Expected Result:** The battlefield creature is not a legal target — the spell only targets cards in graveyards.
- **Phase:** Phase 2 (targeting)
- **Ticket:** ALREADY-IMPLEMENTED — target zone filter in targeting validation

---

**403.3** — PURE-DEF. Permanents exist only on the battlefield. Every object on the battlefield is a permanent.

---

**403.4** — TESTABLE. A permanent entering the battlefield is a new object (overlaps with 400.7).

**ATOM-403.4-001**
- **Rule:** 403.4 — Whenever a permanent enters the battlefield, it becomes a new object and has no relationship to any previous permanent represented by the same card.
- **Mechanism:** A creature that was previously on the battlefield, died, and is returned to the battlefield has no memory of its previous existence (no damage marked, no counters, no attachments).
- **Minimal Board:** A creature on the battlefield with 2 damage marked on it. It dies and is returned to the battlefield by an effect.
- **Action:** Return the creature to the battlefield.
- **Expected Result:** The returned creature has 0 damage, no counters, fresh summoning sickness, no attachments. It is a new permanent.
- **Phase:** Phase 1 (zone transitions) — identity reset is implicit in `init_zone_state`
- **Ticket:** ALREADY-IMPLEMENTED — `init_zone_state` creates fresh `BattlefieldEntity`

---

**403.5** — PURE-DEF. Historical note about "in-play zone" terminology. Not testable.

---

### 404. Graveyard

**404.1** — TESTABLE. Graveyard is the discard pile; countered/destroyed/sacrificed/resolved objects go on top.

**ATOM-404.1-001**
- **Rule:** 404.1 — Any object that's countered, discarded, destroyed, or sacrificed is put on top of its owner's graveyard, as is any instant or sorcery spell that's finished resolving.
- **Mechanism:** `move_object(id, Zone::Graveyard)` must place the object on top (end/front of the graveyard Vec). Resolved instants/sorceries go to graveyard automatically.
- **Minimal Board:** Player casts Lightning Bolt targeting opponent.
- **Action:** Bolt resolves.
- **Expected Result:** Lightning Bolt is on top of its owner's graveyard. Any previously graveyard'd cards are below it.
- **Phase:** Phase 2 (stack resolution)
- **Ticket:** ALREADY-IMPLEMENTED — `stack.rs` moves resolved instants/sorceries to graveyard

**ATOM-404.1-002**
- **Rule:** 404.1 — Each player's graveyard starts out empty.
- **Mechanism:** At game start, all graveyards are empty.
- **Minimal Board:** Game just set up.
- **Action:** Query graveyard size for both players.
- **Expected Result:** Both graveyards have 0 cards.
- **Phase:** Phase 1
- **Ticket:** ALREADY-IMPLEMENTED — `PlayerState::new()` initializes empty graveyard

---

**404.2** — TESTABLE. Graveyard is face-up; any player can examine any graveyard.

**ATOM-404.2-001**
- **Rule:** 404.2 — Each graveyard is kept in a single face-up pile. A player can examine the cards in any graveyard at any time but normally can't change their order.
- **Mechanism:** Graveyard contents (card identities) are publicly visible. Order is preserved and immutable except by effects.
- **Minimal Board:** Player A has 3 cards in graveyard in a specific order.
- **Action:** Player B queries Player A's graveyard.
- **Expected Result:** All 3 cards and their order are visible to Player B.
- **Phase:** Phase 1
- **Ticket:** ALREADY-IMPLEMENTED — graveyard is a `Vec<ObjectId>` with public access

---

**404.3** — TESTABLE. Multiple cards entering graveyard simultaneously — owner arranges order.

**Audit note — Auto-ordering toggle:** The CR says the player "*may*" choose to order the cards. In practice, many players don't care about graveyard order (it only matters for a few old cards with "bottom of graveyard" text). The DecisionProvider should receive a **top-level preference bool** (that could itself be auto-completed by a UI toggle: "Do you want to manually order simultaneous graveyard entries?"). If the player opts out (returns false / auto-order), the engine falls back to the natural ordering from the source zone. This avoids forcing every DP implementation to handle a tedious ordering prompt for a rarely-relevant mechanic. Same `choose_ordering` consolidation as flagged in 401.4.

**ATOM-404.3-001**
- **Rule:** 404.3 — If an effect or rule puts two or more cards into the same graveyard at the same time, the owner may arrange them in any order (auto-order fallback).
- **Mechanism:** When multiple objects move to graveyard simultaneously (e.g., mass destruction) and the owner declines manual ordering (auto-order preference set), the cards enter in source-zone order.
- **Minimal Board:** Player controls 3 creatures (A, B, C). A "destroy all creatures" effect resolves.
- **Action:** All 3 creatures are destroyed simultaneously.
- **Expected Result:** DecisionProvider is prompted with boolean `choose_order`. Returns false. Cards enter graveyard in source-zone iteration order (e.g., A on bottom, C on top).
- **Phase:** Phase 8 (simultaneous graveyard ordering)
- **Ticket:** NEW — simultaneous graveyard insertion ordering via DecisionProvider with auto-order fallback. Note: current `move_object` processes one at a time; simultaneous moves need batching.

**ATOM-404.3-002**
- **Rule:** 404.3 — If an effect or rule puts two or more cards into the same graveyard at the same time, the owner may arrange them in any order (manual ordering).
- **Mechanism:** When multiple objects move to graveyard simultaneously and the owner chooses manual ordering, the DecisionProvider is prompted to choose the order.
- **Minimal Board:** Player controls 3 creatures (A, B, C). A "destroy all creatures" effect resolves.
- **Action:** All 3 creatures are destroyed simultaneously. DecisionProvider is prompted with boolean `choose_order`. Returns true. DecisionProvider is prompted for ordering of A, B, and C. DecisionProvider returns ordering [C, A, B].
- **Expected Result:** Cards enter graveyard in the player-chosen order: C on bottom, A in middle, B on top. The chosen order is respected exactly.
- **Phase:** Phase 8 (simultaneous graveyard ordering)
- **Ticket:** Same as ATOM-404.3-001

---

### 405. Stack

**405.1** — PURE-DEF. Describes how spells and abilities are put on the stack. Definitional — casting/activation rules (601, 602, 603) govern the mechanics.

---

**405.2** — TESTABLE. Stack is ordered — each new object goes on top.

**ATOM-405.2-001**
- **Rule:** 405.2 — The stack keeps track of the order that spells and/or abilities were added to it. Each time an object is put on the stack, it's put on top of all objects already there.
- **Mechanism:** Stack is a LIFO ordered collection. New entries go on top; resolution takes from top.
- **Minimal Board:** Player A casts Spell X, then Player B casts Spell Y in response.
- **Action:** Check stack order.
- **Expected Result:** Spell Y is on top (resolves first), Spell X is below.
- **Phase:** Phase 2 (stack implementation)
- **Ticket:** ALREADY-IMPLEMENTED — stack is a `Vec<StackEntry>` with push/pop semantics

---

**405.3** — TESTABLE. Simultaneous stack placement uses APNAP ordering.

**ATOM-405.3-001**
- **Rule:** 405.3 — If an effect puts two or more objects on the stack at the same time, those controlled by the active player are put on lowest, followed by each other player's objects in APNAP order.
- **Mechanism:** When multiple triggered abilities or effects go on the stack simultaneously, APNAP ordering determines placement order (active player's on bottom, non-active player's on top — so non-active player's resolve first).
- **Minimal Board:** Both players have "at the beginning of each upkeep" triggered abilities. It's Player A's turn.
- **Action:** Upkeep begins, both triggers go on stack.
- **Expected Result:** Player A's trigger is placed first (bottom), Player B's trigger is placed second (top). Player B's trigger resolves first.
- **Phase:** Phase 7 (Triggered Abilities — APNAP ordering, rule 603.3b)
- **Ticket:** NEW — APNAP stack placement for simultaneous triggered abilities

**ATOM-405.3-002**
- **Rule:** 405.3 — If a player controls more than one simultaneous object, that player chooses their relative order on the stack.
- **Mechanism:** When a player has multiple triggers going on the stack simultaneously, the DecisionProvider chooses the order.
- **Minimal Board:** Player A controls two permanents that each trigger "at the beginning of upkeep."
- **Action:** Upkeep begins.
- **Expected Result:** Player A is prompted to choose which trigger goes on the stack first (bottom) and which second (top, resolves first).
- **Phase:** Phase 7 (Triggered Abilities)
- **Ticket:** NEW — player-controlled simultaneous trigger ordering via DecisionProvider

---

**405.4** — TESTABLE (multiple clauses).

**Audit note — ALREADY-IMPLEMENTED vs effective_characteristics:** ATOM-405.4-001 is marked ALREADY-IMPLEMENTED because `StackEntry` references `card_data` and can answer characteristic queries today. Once Phase 5 (continuous effects / layers) lands, `effective_characteristics()` will be the canonical query path, which will layer-resolve characteristics on the stack too. The test itself is still atomic and correct — the ALREADY-IMPLEMENTED classification describes the *test's observable behavior*, not the final implementation path. When Phase 5 lands, the test should still pass (same inputs/outputs), just routed through layers.

**ATOM-405.4-001**
- **Rule:** 405.4 — Each spell has all the characteristics of the card associated with it.
- **Mechanism:** A spell on the stack retains its card's name, mana cost, types, etc. Queries about a spell's characteristics should return the card's characteristics (modified by any effects on the stack).
- **Minimal Board:** Lightning Bolt on the stack.
- **Action:** Query the spell's mana cost and type.
- **Expected Result:** Mana cost is {R}, type is Instant.
- **Phase:** Phase 2
- **Ticket:** ALREADY-IMPLEMENTED — `StackEntry` references `card_data`

**ATOM-405.4-002**
- **Rule:** 405.4 — Each activated or triggered ability on the stack has the text of the ability that created it and no other characteristics.
- **Mechanism:** Abilities on the stack don't have a mana cost, type line, power/toughness, etc. — only the ability text.
- **Minimal Board:** An activated ability on the stack.
- **Action:** Query the ability's mana cost.
- **Expected Result:** No mana cost (None/empty). The ability has only its text and source.
- **Phase:** Phase 2 (stack entries for abilities)
- **Ticket:** ALREADY-IMPLEMENTED — `StackEntry` for abilities carries ability source and effect but not card characteristics

**ATOM-405.4-003**
- **Rule:** 405.4 — The controller of a spell is the player who cast it. The controller of an activated ability is the player who activated it.
- **Mechanism:** `StackEntry.controller` is set to the casting/activating player.
- **Minimal Board:** Player A casts a spell.
- **Action:** Query the spell's controller on the stack.
- **Expected Result:** Controller is Player A.
- **Phase:** Phase 2
- **Ticket:** ALREADY-IMPLEMENTED — `StackEntry` has `controller` field

**ATOM-405.4-004**
- **Rule:** 405.4 — The controller of an activated ability is the player who activated it (not necessarily the permanent's controller).
- **Mechanism:** If Player A controls a permanent owned by Player B, and Player A activates an ability of that permanent, `StackEntry.controller` is Player A (the activator).
- **Minimal Board:** Player A controls Player B's permanent (via control-change effect). The permanent has an activated ability.
- **Action:** Player A activates the ability. Query the ability's controller on the stack.
- **Expected Result:** Controller is Player A (the activator), not Player B (the owner).
- **Phase:** Phase 2 (already correct via `StackEntry.controller = activating_player`) + Phase 5 (control-change effects)
- **Ticket:** ALREADY-IMPLEMENTED — `activate_ability` sets controller to the activating player

**ATOM-405.4-005**
- **Rule:** 405.4 — The controller of an activated ability is the player who activated it, even for "any player may activate" abilities.
- **Mechanism:** Some permanents have abilities that any player may activate (e.g., Howling Mine's draw trigger is passive, but consider activated examples like "Any player may: Pay 2 life: Draw a card"). When Player B activates an ability on Player A's permanent, the resulting stack entry's controller is Player B.
- **Minimal Board:** Player A controls a permanent with an activated ability that reads "Any player may activate this ability." Player B activates it.
- **Action:** Player B activates the ability. Query the ability's controller on the stack.
- **Expected Result:** Controller is Player B (the activator). The ability resolves under Player B's control — "you" in the ability text refers to Player B, effects target from Player B's perspective, and Player B makes all choices during resolution.
- **Phase:** Phase 8 ("any player may activate" abilities)
- **Ticket:** NEW — "any player may activate" ability controller assignment. Note: the engine's `activate_ability` already sets controller to the activating player, so the mechanical correctness follows from existing architecture. The test validates that ability activation legality checking permits non-controllers to activate these abilities.

---

**405.5** — TESTABLE. Resolution rule: all pass → top resolves. Empty stack + all pass → phase/step ends.

**ATOM-405.5-001**
- **Rule:** 405.5 — When all players pass in succession, the top (last-added) spell or ability on the stack resolves.
- **Mechanism:** Priority system: when all players pass consecutively, pop and resolve the top stack entry.
- **Minimal Board:** One spell on the stack. Both players pass priority.
- **Action:** Priority round completes with all passes.
- **Expected Result:** The top spell resolves.
- **Phase:** Phase 2 (priority system)
- **Ticket:** ALREADY-IMPLEMENTED — `priority.rs` handles consecutive passes → resolution

**ATOM-405.5-002**
- **Rule:** 405.5 — If the stack is empty when all players pass, the current step or phase ends and the next begins.
- **Mechanism:** When stack is empty and all players pass, advance to the next step/phase.
- **Minimal Board:** Empty stack, main phase.
- **Action:** Both players pass priority.
- **Expected Result:** Main phase ends, next phase begins.
- **Phase:** Phase 1 (turn structure)
- **Ticket:** ALREADY-IMPLEMENTED — `priority.rs` / `turns.rs` advance on empty stack + all pass

---

**405.6** — PURE-DEF (parent). Some things don't use the stack.

**405.6a** — PURE-DEF. Effects don't go on the stack (they're results of resolution). Delayed triggered abilities may go on stack later.

**405.6b** — PURE-DEF. Static abilities don't go on the stack.

**405.6c** — TESTABLE. Mana abilities resolve immediately.

**ATOM-405.6c-001**
- **Rule:** 405.6c — Mana abilities resolve immediately. If a mana ability both produces mana and has another effect, the mana is produced and the other effect happens immediately.
- **Mechanism:** When a mana ability is activated, it does not go on the stack — it resolves immediately, producing mana (and any side effects) before the player resumes the action they were taking.
- **Minimal Board:** Player controls a land with a mana ability.
- **Action:** Player activates the mana ability during spell casting.
- **Expected Result:** Mana is added to pool immediately. No stack entry is created. Priority is returned to the player who had it.
- **Phase:** Phase 1 (mana abilities) — verified in Phase 4.5 fuzz fix
- **Ticket:** ALREADY-IMPLEMENTED — `priority.rs` routes mana abilities to `activate_mana_ability()` which resolves immediately (Phase 4.5f bug fix)

**ATOM-405.6c-002**
- **Rule:** 405.6c — Mana abilities do NOT use the stack (negative test).
- **Mechanism:** After a mana ability is activated, the stack must not contain any entry for that ability. This is the pure "no stack" test, complementing the "resolves immediately" test above.
- **Minimal Board:** Player controls a land. Stack has one spell on it.
- **Action:** Player activates the land's mana ability.
- **Expected Result:** Stack still has exactly one entry (the original spell). No new entry was added for the mana ability. Mana pool increased.
- **Phase:** Phase 1 (mana abilities)
- **Ticket:** ALREADY-IMPLEMENTED — mana abilities bypass stack entirely

---

**405.6d** — PURE-DEF. Special actions don't use the stack. Reference to rule 116.

**405.6e** — TESTABLE. Turn-based actions don't use the stack.

**ATOM-405.6e-001**
- **Rule:** 405.6e — Turn-based actions don't use the stack; they happen automatically when certain steps or phases begin. They're dealt with before a player would receive priority.
- **Mechanism:** TBAs (draw during draw step, untap during untap step, etc.) execute before priority is granted.
- **Minimal Board:** Active player's draw step begins.
- **Action:** Observe draw step processing.
- **Expected Result:** The draw happens as a TBA before any player receives priority. No stack entry for the draw.
- **Phase:** Phase 1 (turn structure)
- **Ticket:** ALREADY-IMPLEMENTED — `process_draw_step` draws before priority, `process_untap_step` untaps without priority

---

**405.6f** — TESTABLE. SBAs don't use the stack; happen before priority.

**ATOM-405.6f-001**
- **Rule:** 405.6f — State-based actions don't use the stack; they happen automatically when certain conditions are met, before a player would receive priority.
- **Mechanism:** SBA check loop runs before granting priority. SBAs fire without using the stack.
- **Minimal Board:** A creature with lethal damage.
- **Action:** Priority is about to be granted.
- **Expected Result:** SBA check destroys the creature before any player receives priority. No stack entry for the destruction.
- **Phase:** Phase 1 (SBA system) — wired into priority in Pre-Phase 3
- **Ticket:** ALREADY-IMPLEMENTED — `perform_sba_and_triggers` stub in `priority.rs`

---

**405.6g** — PURE-DEF. Player may concede at any time. Concession is immediate, no stack.

**405.6h** — DEFERRED. Multiplayer player-leaves-game cleanup — Phase 9.

---

### 406. Exile

**406.1** — PURE-DEF. Exile is a holding area for objects. Definitional.

**406.2** — PURE-DEF. "To exile" means to put into the exile zone. Definitional.

---

**406.3** — TESTABLE. Exiled cards are face-up by default. Face-down exile has special rules.

**Audit note — Face-down viewing plan:** The plan for face-down exiled cards is to handle "viewing" permissions when the effects that create them come online. E.g., Spell Queller exiles a spell face-down but lets only its controller look at it. The `face_down: bool` metadata on the exiled card needs a companion `visible_to: HashSet<PlayerId>` (or the effect tracks it). For the base case, face-down = hidden from everyone. Specific card effects will override this per-player. Same pattern as hand reveal (402.3 audit note). This all belongs to Phase 8 when exile-interactive cards arrive.

**ATOM-406.3-001**
- **Rule:** 406.3 — Exiled cards are, by default, kept face up and may be examined by any player at any time.
- **Mechanism:** Cards in the exile zone are publicly visible by default.
- **Minimal Board:** A card in exile (e.g., exiled by a spell).
- **Action:** Query exile zone contents.
- **Expected Result:** The card's identity is visible to all players.
- **Phase:** Phase 8 (Exile primitive)
- **Ticket:** NEW — exile zone visibility (face-up default). Note: exile zone exists as a zone enum but cards are rarely exiled in current implementation.

**ATOM-406.3-002**
- **Rule:** 406.3 — Cards "exiled face down" can't be examined by any player except when instructions allow it.
- **Mechanism:** Face-down exiled cards must not be visible. A metadata flag (`face_down: bool`) plus `visible_to: HashSet<PlayerId>` tracks visibility per player.
- **Minimal Board:** A card exiled face down.
- **Action:** Query the card's identity.
- **Expected Result:** The card's identity is hidden from all players (returns no characteristics). A player specifically granted visibility by an effect CAN see it.
- **Phase:** Phase 8 (Exile zone metadata — D21 in roadmap)
- **Ticket:** NEW — face-down exile tracking with per-player visibility overrides

---

**406.3a** — DEFERRED. Face-down exiled cards have no characteristics but may be played from exile. Phase 8+ (face-down exile, morph/disguise).

**406.3b** — DEFERRED. Casting from face-down exile piles. Phase 8+.

**406.4** — DEFERRED. Face-down exile piles management. Phase 8+.

**406.5** — PURE-DEF. Exile pile management guidance for cards that may return. Organizational, not mechanical.

**406.6** — PURE-DEF. Linked abilities between exile-creating and exile-referencing abilities. Reference to rule 607 (linked abilities). Infrastructure in T20.

**406.7** — TESTABLE. Re-exiling from exile creates a new object (same as 400.8).

This is a restatement of 400.8. Test already generated as ATOM-400.8-001.

---

**406.8** — PURE-DEF. Historical note about "removed-from-the-game zone" terminology.

---

### 407. Ante

**407.1–407.4** — OUT-OF-SCOPE. Ante is explicitly not supported per roadmap.md scope.

---

### 408. Command

**408.1** — PURE-DEF. Command zone is reserved for specialized objects. Definitional.

**408.2** — TESTABLE (Phase 8+). Emblems in command zone.

**ATOM-408.2-001**
- **Rule:** 408.2 — Emblems may be created in the command zone.
- **Mechanism:** When a planeswalker ultimate creates an emblem, the emblem object is placed in the command zone. It has no characteristics except its ability text.
- **Minimal Board:** A planeswalker that can create an emblem.
- **Action:** Activate the emblem-creating ability.
- **Expected Result:** An emblem object is created in the command zone with the specified ability.
- **Phase:** Phase 8 (Planeswalker abilities, emblems)
- **Ticket:** NEW — emblem creation in command zone

---

**408.3** — DEFERRED. Format-specific command zone cards (Commander Phase 9; Planechase/Archenemy/Conspiracy not in scope but Commander is).

---

## Chapter 5: Turn Structure (500–514)

### 500. General

**500.1** — TESTABLE. A turn has five phases in fixed order.

**ATOM-500.1-001**
- **Rule:** 500.1 — A turn consists of five phases, in this order: beginning, precombat main, combat, postcombat main, and ending. Each takes place every turn, even if nothing happens.
- **Mechanism:** `advance_turn` / `run_turn` must iterate through all five phases in order every turn. No phase is skipped by default (combat sub-steps may be skipped per 508.8, but the combat *phase* itself always occurs).
- **Minimal Board:** A turn with no creatures and no spells to cast.
- **Action:** Run a complete turn, recording phase transitions.
- **Expected Result:** Phase sequence is: Beginning → Precombat Main → Combat → Postcombat Main → Ending. All five phases occur.
- **Phase:** Phase 1 (turn structure)
- **Ticket:** ALREADY-IMPLEMENTED — `turns.rs` iterates all phases

---

**500.2** — TESTABLE. Phase/step ends only when stack is empty AND all players pass in succession.

**ATOM-500.2-001**
- **Rule:** 500.2 — A phase or step in which players receive priority ends when the stack is empty and all players pass in succession. Simply having the stack become empty doesn't end it.
- **Mechanism:** After a spell resolves and empties the stack, the active player gets priority again before the phase/step ends. The phase/step only ends when all players pass with the stack already empty.
- **Minimal Board:** Player A casts a spell during main phase. Player B passes. Spell resolves (stack empty).
- **Action:** After resolution, active player gets priority again.
- **Expected Result:** The phase does NOT immediately end after the stack empties. Active player gets priority. Only when both players pass with the stack empty does the phase end.
- **Phase:** Phase 2 (priority system)
- **Ticket:** ALREADY-IMPLEMENTED — `priority.rs` handles this: resolution → SBA check → re-grant priority

---

**500.3** — TESTABLE. Steps without priority end when all specified actions complete.

**ATOM-500.3-001**
- **Rule:** 500.3 — A step in which no players receive priority ends when all specified actions that take place during that step are completed. The only such steps are the untap step and certain cleanup steps.
- **Mechanism:** Untap step: untap all → step ends immediately, no priority. Cleanup step (when no SBAs/triggers): discard + remove damage → step ends, no priority.
- **Minimal Board:** Active player's untap step.
- **Action:** Untap step processes.
- **Expected Result:** Permanents untap, step ends. No player receives priority during the untap step.
- **Phase:** Phase 1 (turn structure)
- **Ticket:** ALREADY-IMPLEMENTED — `process_untap_step` untaps without granting priority

---

**500.4** — TESTABLE. Effects that last "until" a step/phase expire as that step/phase begins.

**ATOM-500.4-001**
- **Rule:** 500.4 — As a step or phase begins, if there are effects that last until that step or phase, those effects expire.
- **Mechanism:** Duration expiry hook fires at the *beginning* of the specified step/phase. E.g., "until your next upkeep" expires as the upkeep step begins.
- **Minimal Board:** A continuous effect with duration "until your next upkeep."
- **Action:** The player's next upkeep step begins.
- **Expected Result:** The effect expires at the beginning of that upkeep step, before any triggers or priority.
- **Phase:** Phase 5 Pre-Work (T22 — duration expiry hooks)
- **Ticket:** T22 — Duration + turn structure fixes (E42: duration expiry hooks)

---

**500.5** — TESTABLE (multiple clauses). Effects expiring at end of step/phase, plus mana pool emptying.

**ATOM-500.5-001**
- **Rule:** 500.5 — As a step or phase ends, effects that last until the end of that step or phase expire. Then any unspent mana left in a player's mana pool empties.
- **Mechanism:** At end of each step/phase: (1) expire matching-duration effects, (2) empty all players' mana pools. This is a TBA (doesn't use the stack).
- **Minimal Board:** Player has 3 mana in pool at end of main phase.
- **Action:** Main phase ends.
- **Expected Result:** Mana pool is emptied. Any "until end of this phase" effects expire.
- **Phase:** Phase 1 (mana pool emptying) + Phase 5 Pre-Work (T22 for duration expiry)
- **Ticket:** ALREADY-IMPLEMENTED (mana emptying) + T22 (duration expiry hooks)

---

**500.5a** — TESTABLE. "Until end of combat" expires at end of combat phase, not beginning of end-of-combat step.

**ATOM-500.5a-001**
- **Rule:** 500.5a — Effects that last "until end of combat" expire at the end of the combat phase, not at the beginning of the end of combat step.
- **Mechanism:** `UntilEndOfCombat` duration must expire at the *end* of the combat phase (after the end of combat step finishes), not at the *beginning* of the end of combat step.
- **Minimal Board:** A creature with +2/+2 "until end of combat."
- **Action:** End of combat step begins, then ends.
- **Expected Result:** The creature still has +2/+2 during the end of combat step. The buff expires only after the combat phase fully ends.
- **Phase:** Phase 5 Pre-Work (T22 — `UntilEndOfCombat` duration variant + expiry hook)
- **Ticket:** T22 — Duration variants (E43: UntilEndOfCombat)

---

**500.5b** — PURE-DEF. "Until end of turn" has special rules — refers to rule 514.2. Covered by cleanup step tests.

---

**500.6** — TESTABLE. "At the beginning of" triggers fire when a phase/step begins, placed on stack before priority.

**ATOM-500.6-001**
- **Rule:** 500.6 — When a phase or step begins, any abilities that trigger "at the beginning of" that phase or step trigger. They are put on the stack the next time a player would receive priority.
- **Mechanism:** At the start of each step/phase, the engine must check for "at the beginning of [X]" triggers, place them on the stack, then grant priority.
- **Minimal Board:** A permanent with "At the beginning of your upkeep, gain 1 life."
- **Action:** Upkeep step begins.
- **Expected Result:** The trigger goes on the stack before any player gets priority.
- **Phase:** Phase 7 (Triggered Abilities)
- **Ticket:** NEW — step/phase beginning trigger checking. Depends on Phase 7 trigger infrastructure.

---

**500.7** — DEFERRED. Extra turns — Phase 9 (mutable TurnPlan, D6 in roadmap).

**500.8** — DEFERRED. Extra phases — Phase 9.

**500.9** — DEFERRED. Extra steps — Phase 9.

**500.10** — DEFERRED. Adding a step after a phase (creates the containing phase) — Phase 9. Has an Example (Obeka) but the mechanism is Phase 9 mutable TurnPlan.

**500.10a** — DEFERRED. "You get" additional step limited to controller's turn — Phase 9.

**500.11** — DEFERRED. Skip step/phase/turn — Phase 9 (mutable TurnPlan) + Phase 6 (replacement effects, rule 614.10).

**500.12** — PURE-DEF. No game events between steps/phases/turns. Definitional constraint on engine event ordering.

---

### 501. Beginning Phase

**501.1** — PURE-DEF. Beginning phase has three steps: untap, upkeep, draw. Structural definition.

---

### 502. Untap Step

**502.1** — DEFERRED. Phasing during untap step — Phase 9 (D1 in roadmap).

---

**502.2** — DEFERRED. Day/Night transition during untap step — Phase 9 (D14 in roadmap).

**502.2a** — DEFERRED. Multiplayer shared team turns day/night — Phase 9.

---

**502.3** — TESTABLE. Active player untaps all their permanents simultaneously.

**ATOM-502.3-001**
- **Rule:** 502.3 — The active player determines which permanents they control will untap. Then they untap them all simultaneously. Normally, all of a player's permanents untap.
- **Mechanism:** During the untap step, all permanents controlled by the active player are untapped simultaneously. This is a TBA (no stack).
- **Minimal Board:** Active player controls 3 tapped permanents.
- **Action:** Untap step begins.
- **Expected Result:** All 3 permanents are untapped simultaneously.
- **Phase:** Phase 1 (turn structure)
- **Ticket:** ALREADY-IMPLEMENTED — `process_untap_step` in `turns.rs` untaps all controlled permanents

**ATOM-502.3-002**
- **Rule:** 502.3 — Effects can keep one or more of a player's permanents from untapping.
- **Mechanism:** "Doesn't untap during your untap step" effects must prevent specific permanents from untapping during the untap step TBA.
- **Minimal Board:** Active player controls a permanent with "this permanent doesn't untap during your untap step" (e.g., tapped by a Frost Titan effect).
- **Action:** Untap step begins.
- **Expected Result:** The affected permanent stays tapped. Other permanents untap normally.
- **Phase:** Phase 5 (Continuous Effects) — the "doesn't untap" is a continuous effect modifying the untap TBA
- **Ticket:** NEW — "doesn't untap" continuous effect filtering in untap step. Requires L15 (post-layer pass for player action restrictions) or equivalent.

**ATOM-502.3-003**
- **Rule:** 502.3 — Effects that make players choose which permanents to untap (e.g., Winter Orb: "As long as Winter Orb is untapped, players can't untap more than one land during their untap steps", or Stasis: "Players skip their untap steps").
- **Mechanism:** The untap TBA must consult continuous effects that restrict *how many* or *which* permanents a player may untap. When such an effect is active, the engine must prompt the DecisionProvider to choose which permanents to untap within the constraint. This is distinct from ATOM-502.3-002 (binary "doesn't untap") — here the player has a constrained choice.
- **Minimal Board:** Active player controls Winter Orb (untapped) and 4 tapped lands.
- **Action:** Untap step begins.
- **Expected Result:** Player is prompted (via DecisionProvider) to choose exactly 1 land to untap. The other 3 lands remain tapped. Non-land permanents untap normally.
- **Phase:** Phase 5 (Continuous Effects) + Phase 8 (Winter Orb card)
- **Ticket:** NEW — constrained untap choice via DecisionProvider. Extends the untap TBA to support "untap up to N" restrictions. Separate from the binary "doesn't untap" mechanism in ATOM-502.3-002.

---

**502.4** — TESTABLE. No priority during untap step.

**ATOM-502.4-001**
- **Rule:** 502.4 — No player receives priority during the untap step, so no spells can be cast or resolve and no abilities can be activated or resolve. Any ability that triggers during this step will be held until the next time a player would receive priority (upkeep).
- **Mechanism:** The untap step must not grant priority to any player. Triggers that fire during untap (e.g., from untapping) are held until upkeep.
- **Minimal Board:** A permanent that triggers "whenever this permanent becomes untapped." Active player's untap step.
- **Action:** Untap step untaps the permanent.
- **Expected Result:** The trigger is created but not placed on the stack during the untap step. It waits until upkeep, when it goes on the stack before priority is granted (per 503.1a).
- **Phase:** Phase 7 (Triggered Abilities — trigger queueing from no-priority steps)
- **Ticket:** NEW — trigger hold-and-release for no-priority steps. Dependency: Phase 7 trigger infrastructure.

---

### 503. Upkeep Step

**503.1** — TESTABLE. Upkeep has no TBA; active player gets priority.

**ATOM-503.1-001**
- **Rule:** 503.1 — The upkeep step has no turn-based actions. Once it begins, the active player gets priority.
- **Mechanism:** Upkeep step starts, no TBA fires, active player immediately gets priority (after SBA check).
- **Minimal Board:** Start of upkeep step.
- **Action:** Upkeep begins.
- **Expected Result:** Active player receives priority. No automatic game actions occur.
- **Phase:** Phase 1 (turn structure)
- **Ticket:** ALREADY-IMPLEMENTED — upkeep step grants priority directly

---

**503.1a** — TESTABLE. Untap-step triggers and upkeep-beginning triggers go on stack before priority.

**ATOM-503.1a-001**
- **Rule:** 503.1a — Any abilities that triggered during the untap step and any abilities that triggered at the beginning of the upkeep are put onto the stack before the active player gets priority; the order in which they triggered doesn't matter.
- **Mechanism:** All pending triggers (from untap step + "at beginning of upkeep") are placed on the stack in APNAP order before the active player gets priority.
- **Minimal Board:** A permanent that triggers on untap, and a permanent that triggers "at beginning of upkeep."
- **Action:** Upkeep step begins.
- **Expected Result:** Both triggers are on the stack before active player gets priority. Order determined by APNAP rules.
- **Phase:** Phase 7 (Triggered Abilities)
- **Ticket:** NEW — trigger batch placement at upkeep start (untap-step held triggers + upkeep triggers)

---

**503.2** — DEFERRED. "After your upkeep step" casting timing with multiple upkeep steps — Phase 9 (extra steps).

---

### 504. Draw Step

**504.1** — TESTABLE. Active player draws a card as a TBA.

**ATOM-504.1-001**
- **Rule:** 504.1 — First, the active player draws a card. This turn-based action doesn't use the stack.
- **Mechanism:** At the start of the draw step, the active player draws one card before any player receives priority. This is a TBA.
- **Minimal Board:** Active player with cards in library. Draw step begins.
- **Action:** Draw step processing.
- **Expected Result:** Active player's hand size increases by 1. The drawn card is the top card of the library. No stack entry.
- **Phase:** Phase 1 (turn structure)
- **Ticket:** ALREADY-IMPLEMENTED — `process_draw_step` in `turns.rs` draws before priority

**Audit note — No priority before draw:** The CR states the draw is a TBA that happens "first" before any player receives priority. In our architecture this is implicit: `process_draw_step` performs the draw before calling the priority loop. No separate test is needed — if the draw happened after priority, players could cast instants before the draw, which would be caught by any draw-step interaction test.

**ATOM-504.1-002**
- **Rule:** 504.1 (+ rule 103.8a) — The starting player skips their first draw step.
- **Mechanism:** `skip_first_draw` flag prevents the draw TBA on the first turn.
- **Minimal Board:** First turn of the game. Starting player's draw step.
- **Action:** Draw step processing.
- **Expected Result:** No card is drawn. The flag is consumed (subsequent turns draw normally).
- **Phase:** Phase 1 (Pre-Phase 3 work item 3.5)
- **Ticket:** ALREADY-IMPLEMENTED — `skip_first_draw` flag on `GameState`, design_doc §3.5
- **Note:** This rule applies only to 2-player games. In multiplayer (rule 103.8, Commander variant), each player draws on their first turn. The multiplayer variant will be tested in the Phase 9 Commander session.

---

**504.2** — PURE-DEF. After the draw TBA, the active player gets priority. Implicit in the draw step implementation.

---

### 505. Main Phase

**505.1** — TESTABLE. Two main phases separated by combat.

**ATOM-505.1-001**
- **Rule:** 505.1 — There are two main phases in a turn. The first (precombat) and second (postcombat) main phases are separated by the combat phase.
- **Mechanism:** Turn structure must include two distinct main phases, one before combat and one after.
- **Minimal Board:** A full turn.
- **Action:** Track phase progression.
- **Expected Result:** Phase order includes: ... → Precombat Main → Combat → Postcombat Main → ...
- **Phase:** Phase 1 (turn structure)
- **Ticket:** ALREADY-IMPLEMENTED — `turns.rs` has both main phases

---

**505.1a** — TESTABLE. Only the first main phase is precombat; all others (including after skipped combat) are postcombat.

**ATOM-505.1a-001**
- **Rule:** 505.1a — Only the first main phase of the turn is a precombat main phase. All other main phases are postcombat main phases, including the second main phase of a turn in which the combat phase has been skipped.
- **Mechanism:** If an effect skips the combat phase, the second main phase is still "postcombat." Effects that trigger "during your postcombat main phase" still apply. Effects that care about "precombat main phase" only apply to the first one.
- **Minimal Board:** A turn where combat is skipped (no creatures, combat phase occurs but all sub-steps auto-pass).
- **Action:** Check phase identity of the second main phase.
- **Expected Result:** The second main phase is tagged as postcombat, not precombat. Saga lore counters (505.4) are NOT added during the postcombat main phase.
- **Phase:** Phase 1 (turn structure) — phase identity tracking
- **Ticket:** ALREADY-IMPLEMENTED — engine tracks precombat vs postcombat main phase identity

**ATOM-505.1a-002**
- **Rule:** 505.1a — Multiple postcombat main phases each trigger "at the beginning of your postcombat main phase" effects.
- **Mechanism:** If an effect grants additional combat phases (e.g., Aggravated Assault), each subsequent main phase is a postcombat main phase. An ability that triggers "at the beginning of your postcombat main phase" fires for EACH postcombat main phase, not just the first one.
- **Minimal Board:** A permanent with "At the beginning of your postcombat main phase, draw a card." Player takes an extra combat phase (via an effect), resulting in two postcombat main phases.
- **Action:** Both postcombat main phases begin.
- **Expected Result:** The trigger fires twice — once for each postcombat main phase.
- **Phase:** Phase 9 (extra phases/steps) + Phase 7 (triggered abilities)
- **Ticket:** NEW — multiple postcombat main phase trigger firing. Stubbed for Phase 9; the trigger infrastructure (Phase 7) should already handle this if phase identity is correctly tagged.

---

**505.1b** — PURE-DEF. "First main phase" / "second main phase" card text counting rule. Definitional for card text parsing.

---

**505.2** — PURE-DEF. Main phase has no steps; ends when all pass with empty stack. Covered by 500.2.

---

**505.3** — OUT-OF-SCOPE. Archenemy scheme TBA during precombat main. Not in scope.

**505.4** — DEFERRED. Saga lore counter TBA during precombat main — Phase 8 (Sagas).

**505.5** — OUT-OF-SCOPE. Attractions roll TBA during precombat main — not in scope.

---

**505.6** — PURE-DEF. Active player gets priority during main phase. Covered by general priority rules.

**505.6a** — TESTABLE. Main phase is the only phase for sorcery-speed spells.

**ATOM-505.6a-001**
- **Rule:** 505.6a — The main phase is the only phase in which a player can normally cast artifact, creature, enchantment, planeswalker, and sorcery spells. The active player may cast these spells.
- **Mechanism:** `check_cast_legality` must reject sorcery-speed spells outside of the main phase (or when the player doesn't have priority with an empty stack).
- **Minimal Board:** Active player has a creature card in hand. It's the combat phase.
- **Action:** Attempt to cast the creature spell.
- **Expected Result:** Cast is rejected — sorcery-speed spells can only be cast during main phase when stack is empty and you have priority.
- **Phase:** Phase 2 (casting pipeline)
- **Ticket:** ALREADY-IMPLEMENTED — `check_cast_legality` in `cast.rs` enforces sorcery-speed timing

---

**505.6b** — TESTABLE (multiple clauses). Land play timing rules.

**ATOM-505.6b-001**
- **Rule:** 505.6b — During either main phase, the active player may play one land card from their hand if the stack is empty, if the player has priority, and if they haven't played a land this turn.
- **Mechanism:** `play_land` must enforce: (1) main phase, (2) active player, (3) stack empty, (4) has priority, (5) haven't exceeded lands-per-turn count.
- **Minimal Board:** Active player has a land in hand. Precombat main phase, stack empty.
- **Action:** Play the land.
- **Expected Result:** Land enters the battlefield. `lands_played_this_turn` incremented.
- **Phase:** Phase 1 (zone transitions)
- **Ticket:** ALREADY-IMPLEMENTED — `play_land` in `zones.rs` with timing guards (Phase 1 verification)

**ATOM-505.6b-002**
- **Rule:** 505.6b — Playing a land doesn't use the stack; it can't be countered and players can't respond.
- **Mechanism:** `play_land` is a special action, not a spell. No stack entry. No response window.
- **Minimal Board:** Active player plays a land.
- **Action:** Play the land.
- **Expected Result:** Land immediately enters the battlefield. No stack entry created. Opponent gets no chance to respond to the land play itself.
- **Phase:** Phase 1
- **Ticket:** ALREADY-IMPLEMENTED — `play_land` bypasses stack

**ATOM-505.6b-003**
- **Rule:** 505.6b — A player who has already played a land this turn cannot play another (unless an effect allows additional land plays).
- **Mechanism:** `play_land` checks `lands_played_this_turn` against `lands_per_turn` (normally 1). Rejects if limit reached.
- **Minimal Board:** Active player has already played a land this turn. Has another land in hand.
- **Action:** Attempt to play the second land.
- **Expected Result:** Play is rejected — already played maximum lands this turn.
- **Phase:** Phase 1
- **Ticket:** ALREADY-IMPLEMENTED — `play_land` checks land count

**ATOM-505.6b-004**
- **Rule:** 505.6b — An effect may state the player may play additional lands (e.g., Exploration).
- **Mechanism:** `get_effective_lands_per_turn` returns a value > 1 when effects grant additional land plays.
- **Minimal Board:** Active player controls Exploration ("You may play an additional land on each of your turns"). Has played 1 land.
- **Action:** Attempt to play a second land.
- **Expected Result:** Play succeeds — Exploration allows 2 lands per turn.
- **Phase:** Phase 5 Pre-Work (T22 step 5: lands_per_turn dynamic) + Phase 5 (continuous effect populating the value)
- **Ticket:** T22 — `get_effective_lands_per_turn` oracle function (E45)

---

### 506. Combat Phase

**506.1** — TESTABLE. Combat phase has five steps; declare blockers and combat damage are skipped if no attackers.

**ATOM-506.1-001**
- **Rule:** 506.1 — The combat phase has five steps: beginning of combat, declare attackers, declare blockers, combat damage, and end of combat. The declare blockers and combat damage steps are skipped if no creatures are declared as attackers or put onto the battlefield attacking.
- **Mechanism:** When no attackers are declared, the declare blockers and combat damage steps must be entirely skipped (no TBAs, no priority).
- **Minimal Board:** Active player has no creatures (or chooses not to attack).
- **Action:** Declare attackers step — no attackers declared.
- **Expected Result:** Declare blockers step and combat damage step are skipped entirely. Phase proceeds directly from declare attackers to end of combat.
- **Phase:** Phase 3 (combat wiring)
- **Ticket:** ALREADY-IMPLEMENTED — `skipped_by_508_8` flag in `Game::run_turn` (Phase 3 post-audit)

**ATOM-506.1-002**
- **Rule:** 506.1 — There are two combat damage steps if any attacking or blocking creature has first strike or double strike.
- **Mechanism:** When a first strike or double strike creature is in combat, the combat damage step splits into two: first-strike damage step and normal damage step.
- **Minimal Board:** An attacking creature with first strike. A blocking creature without first strike.
- **Action:** Combat damage processing.
- **Expected Result:** Two damage steps occur: first-strike creatures deal damage first, then remaining creatures deal damage.
- **Phase:** Phase 4 (Keywords — first strike / double strike)
- **Ticket:** ALREADY-IMPLEMENTED — `should_deal_damage_this_step` in `combat/keywords.rs`, two-step processing in `process_combat_damage`

---

**506.2** — TESTABLE. Active player is attacking player; nonactive player is defending player.

**ATOM-506.2-001**
- **Rule:** 506.2 — During the combat phase, the active player is the attacking player; creatures that player controls may attack. The nonactive player is the defending player in a two-player game.
- **Mechanism:** Combat roles are assigned by turn structure: active player attacks, nonactive player defends.
- **Minimal Board:** Two-player game, Player A's turn.
- **Action:** Combat phase begins.
- **Expected Result:** Player A is the attacking player. Player B is the defending player.
- **Phase:** Phase 3 (combat)
- **Ticket:** ALREADY-IMPLEMENTED — combat uses `active_player` for attacking, opponent for defending

---

**506.2a** — DEFERRED. Multiplayer defending player choice — Phase 9.

**506.2b** — OUT-OF-SCOPE. Shared team turns — Phase 9.

---

**506.3** — BOUNDARY-DEF. Only creatures can attack or block. Only players, planeswalkers, or battles can be attacked.

**ATOM-506.3-001**
- **Rule:** 506.3 — Only a creature can attack or block. Only a player, a planeswalker, or a battle can be attacked.
- **Mechanism:** Attack/block validation must reject non-creature permanents. Attack targets must be players (or planeswalkers/battles when implemented).
- **Minimal Board:** Player controls a non-creature artifact.
- **Action:** Attempt to declare the artifact as an attacker.
- **Expected Result:** Declaration is illegal — only creatures can attack.
- **Phase:** Phase 3 (combat validation)
- **Ticket:** ALREADY-IMPLEMENTED — `validate_attackers` checks `is_creature()`

---

**506.3a** — TESTABLE. Noncreature permanent put onto battlefield attacking — enters but isn't attacking.

**ATOM-506.3a-001**
- **Rule:** 506.3a — If an effect would put a noncreature permanent onto the battlefield attacking or blocking, the permanent does enter the battlefield but it's never considered to be an attacking or blocking permanent.
- **Mechanism:** When an effect tries to put a noncreature onto the battlefield "attacking," the permanent enters but its `AttackingInfo` is not set.
- **Minimal Board:** An effect that would put an artifact token onto the battlefield attacking.
- **Action:** The effect resolves.
- **Expected Result:** The artifact enters the battlefield but is NOT attacking.
- **Phase:** Phase 8 (token creation + "put onto battlefield attacking" effects)
- **Ticket:** NEW — noncreature "enters attacking" guard

---

**506.3b** — TESTABLE. Creature put onto battlefield attacking under control of non-attacking player — enters but doesn't attack.

**ATOM-506.3b-001**
- **Rule:** 506.3b — If an effect would put a creature onto the battlefield attacking under the control of any player except an attacking player, that creature enters but is never an attacking creature.
- **Mechanism:** A creature entering the battlefield "attacking" must be controlled by the attacking player to actually be attacking.
- **Minimal Board:** An effect controlled by the defending player puts a creature onto the battlefield "attacking."
- **Action:** The effect resolves.
- **Expected Result:** The creature enters the battlefield but is NOT attacking.
- **Phase:** Phase 8 (ETB-attacking effects)
- **Ticket:** NEW — ETB-attacking controller validation

---

**506.3c** — TESTABLE. Creature enters attacking a player/permanent that's no longer valid — enters but doesn't attack.

**ATOM-506.3c-001**
- **Rule:** 506.3c — If an effect would put a creature onto the battlefield attacking a player not in the game or a permanent no longer on the battlefield, that creature enters but is never an attacking creature.
- **Mechanism:** When the specified attack target is invalid (player left, planeswalker destroyed), the creature enters but does not attack.
- **Minimal Board:** An effect puts a creature onto the battlefield attacking a planeswalker that was destroyed earlier on the stack.
- **Action:** The effect resolves.
- **Expected Result:** The creature enters the battlefield but is NOT attacking.
- **Phase:** Phase 8
- **Ticket:** NEW — ETB-attacking target validation

---

**506.3d** — TESTABLE. Creature put onto battlefield attacking during declare blockers/combat damage/end of combat enters unblocked.

**Audit note:** The rule explicitly names three steps. The earliest testable moment is the declare blockers step (during priority after blocks are declared), since that's the first step where the "guaranteed unblocked" semantics apply. We test all three steps for completeness.

**ATOM-506.3d-001**
- **Rule:** 506.3d — Creature enters attacking during the **declare blockers step** — enters unblocked.
- **Mechanism:** A creature that enters attacking during the declare blockers step (after blocks have been declared for existing attackers) is automatically unblocked — no blocker can be assigned to it because the blocking declaration has already happened.
- **Minimal Board:** During the declare blockers step (priority window after blocks declared), an effect puts a creature onto the battlefield attacking.
- **Action:** The effect resolves.
- **Expected Result:** The creature is attacking and unblocked. If it survives to the combat damage step, it deals combat damage to the defending player.
- **Phase:** Phase 8 (ETB-attacking during late combat steps)
- **Ticket:** NEW — late ETB-attacking creature enters unblocked

**ATOM-506.3d-002**
- **Rule:** 506.3d — Creature enters attacking during the **combat damage step** — enters unblocked.
- **Mechanism:** Same as 506.3d-001 but during the combat damage step's priority window (between damage assignment rounds or before/after first-strike damage).
- **Minimal Board:** During the combat damage step, an effect puts a creature onto the battlefield attacking.
- **Action:** The effect resolves.
- **Expected Result:** The creature is attacking and unblocked. Whether it deals damage depends on timing relative to damage assignment (it missed this round's assignment but may participate in a subsequent first-strike/normal damage step if one exists).
- **Phase:** Phase 8
- **Ticket:** Same as ATOM-506.3d-001

**ATOM-506.3d-003**
- **Rule:** 506.3d — Creature enters attacking during the **end of combat step** — enters unblocked.
- **Mechanism:** A creature entering attacking during the end of combat step is unblocked but won't deal combat damage (the combat damage step has already passed). It will be removed from combat when the end of combat step ends (511.3).
- **Minimal Board:** During the end of combat step, an effect puts a creature onto the battlefield attacking.
- **Action:** The effect resolves.
- **Expected Result:** The creature enters attacking and unblocked, but deals no combat damage (damage step already passed). It is removed from combat when the step ends.
- **Phase:** Phase 8
- **Ticket:** Same as ATOM-506.3d-001

---

**506.3e** — TESTABLE. Creature enters blocking but the creature it would block isn't attacking its controller — enters but doesn't block.

**Audit note — No current card uses this mechanic.** There is no card in Magic that puts a creature onto the battlefield blocking. Given the awkward rules implications (this rule, plus 509.4/509.4a/509.4b), it's unlikely one will be printed. The test is included for structural completeness but is extremely low priority. If resources are constrained, this test can be skipped without risk — the validation logic is simple and would be covered naturally if an ETB-blocking effect is ever implemented.

**ATOM-506.3e-001**
- **Rule:** 506.3e — If an effect would put a creature onto the battlefield blocking but the creature it would block isn't attacking the entering creature's controller (or their PW/battle), that creature enters but is never a blocking creature.
- **Mechanism:** ETB-blocking must validate that the would-be-blocked attacker is actually attacking the entering creature's controller.
- **Minimal Board:** Multiplayer game. An effect puts a creature onto Player C's battlefield blocking an attacker that's attacking Player B.
- **Action:** The effect resolves.
- **Expected Result:** The creature enters but is NOT blocking (the attacker isn't attacking Player C).
- **Phase:** Phase 9 (multiplayer combat)
- **Ticket:** NEW — ETB-blocking controller validation. **Low priority** — no existing card exercises this rule.

---

**506.3f** — DEFERRED. Creature that's also a battle entering attacking or blocking — deferred until battles are implemented (Phase 8+).

**506.3g** — DEFERRED. Battle becoming attacking/blocking creature — deferred until battles are implemented (Phase 8+).

---

**506.4** — TESTABLE. Removal from combat on various conditions.

**ATOM-506.4-001**
- **Rule:** 506.4 — A permanent is removed from combat if it leaves the battlefield, if its controller changes, if it phases out, if an effect specifically removes it, if it stops being a creature, etc.
- **Mechanism:** `remove_from_combat` helper must clear attacking/blocking state when any of these conditions occur.
- **Minimal Board:** An attacking creature.
- **Action:** The creature is bounced to hand (leaves the battlefield).
- **Expected Result:** The creature is no longer in combat. It's removed from the attacker list.
- **Phase:** Phase 5 Pre-Work (T21b — combat removal helper)
- **Ticket:** T21b — combat removal on control/type change (E36)

**ATOM-506.4-002**
- **Rule:** 506.4 — A permanent is removed from combat if its controller changes.
- **Mechanism:** When a control-change effect resolves on an attacking/blocking creature, the creature must be removed from combat.
- **Minimal Board:** An attacking creature. An effect changes its controller to the defending player.
- **Action:** Control change resolves.
- **Expected Result:** The creature is removed from combat.
- **Phase:** Phase 5 (Layer 2: control change) + T21b
- **Ticket:** T21b — combat removal on control change (E36)

**ATOM-506.4-003**
- **Rule:** 506.4 — A permanent is removed from combat if it stops being a creature.
- **Mechanism:** If a type-changing effect removes the creature type from an attacking/blocking permanent, it's removed from combat.
- **Minimal Board:** An attacking creature. An effect makes it a non-creature artifact.
- **Action:** Type-change resolves.
- **Expected Result:** The permanent is removed from combat.
- **Phase:** Phase 5 (Layer 4: type change) + T21b
- **Ticket:** T21b — combat removal on type change (E36)

**ATOM-506.4-004**
- **Rule:** 506.4 — A permanent is removed from combat if it phases out.
- **Mechanism:** When a permanent phases out during combat, it leaves the battlefield (rule 702.26d) and is therefore removed from combat.
- **Minimal Board:** An attacking creature. An effect causes it to phase out during the declare blockers step.
- **Action:** Creature phases out.
- **Expected Result:** The creature is removed from combat. It is treated as though it doesn't exist for the remainder of the combat phase.
- **Phase:** Phase 9 (phasing implementation)
- **Ticket:** NEW — phasing removes from combat. Depends on phasing infrastructure (Phase 9, D1).

**ATOM-506.4-005**
- **Rule:** 506.4 — A permanent is removed from combat if an effect specifically removes it from combat.
- **Mechanism:** Some effects say "remove [creature] from combat" (e.g., Maze of Ith: "Untap target attacking creature. Prevent all combat damage that would be dealt to and dealt by that creature. Remove it from combat."). The engine needs a `remove_from_combat(object_id)` helper that clears attacking/blocking state.
- **Minimal Board:** An attacking creature. Maze of Ith targets it.
- **Action:** Maze of Ith resolves.
- **Expected Result:** The creature is untapped, removed from combat (no longer attacking), and will deal/receive no combat damage.
- **Phase:** Phase 8 (Maze of Ith / explicit removal effects)
- **Ticket:** T21b — `remove_from_combat` helper (E36). The helper is generic; Maze of Ith is the first card to exercise it.

---

**506.4a** — TESTABLE. Spells/abilities that would have prevented attacking/blocking don't retroactively remove from combat.

**ATOM-506.4a-001**
- **Rule:** 506.4a — Once a creature has been declared as an attacking or blocking creature, spells or abilities that would have kept it from attacking or blocking don't remove it from combat.
- **Mechanism:** After attackers/blockers are legally declared, subsequent effects that would prevent attacking (e.g., giving it defender) don't undo the declaration.
- **Minimal Board:** An attacking creature. After declaration, an effect gives it Defender.
- **Action:** Defender is granted after attack declaration.
- **Expected Result:** The creature remains attacking. Defender doesn't retroactively remove it.
- **Phase:** Phase 4 (keywords) + Phase 3 (combat)
- **Ticket:** ALREADY-IMPLEMENTED — combat state is locked after declaration; keyword changes don't re-validate

---

**506.4b** — TESTABLE. Tapping/untapping doesn't remove from combat.

**ATOM-506.4b-001**
- **Rule:** 506.4b — Tapping or untapping a creature that's already been declared as an attacker or blocker doesn't remove it from combat and doesn't prevent its combat damage.
- **Mechanism:** An attacking creature that gets tapped by an effect mid-combat remains attacking and deals combat damage normally.
- **Minimal Board:** An attacking creature. An effect taps it during the declare blockers step.
- **Action:** Creature is tapped.
- **Expected Result:** Creature remains attacking and deals combat damage. Tapping doesn't remove from combat.
- **Phase:** Phase 3 (combat)
- **Ticket:** ALREADY-IMPLEMENTED — tapped state is irrelevant after attack declaration

---

**506.4c** — TESTABLE. Attacking creature's PW/battle target is removed — creature stays attacking but attacks nothing.

**ATOM-506.4c-001**
- **Rule:** 506.4c — If a creature is attacking a planeswalker or battle, removing that planeswalker or battle from combat doesn't remove the creature from combat. It continues attacking but is not attacking any player, PW, or battle. It may be blocked. If unblocked, it deals no combat damage.
- **Mechanism:** When a PW being attacked leaves the battlefield, the attacker stays attacking but its attack target becomes None. If unblocked, it assigns no damage.
- **Minimal Board:** A creature attacking a planeswalker. The planeswalker is destroyed during combat.
- **Action:** Planeswalker leaves the battlefield.
- **Expected Result:** The creature is still attacking (can be blocked), but if unblocked, it deals no combat damage.
- **Phase:** Phase 8 (planeswalker combat targets)
- **Ticket:** NEW — orphaned attacker (PW/battle removed) deals no damage if unblocked

---

**506.4d** — DEFERRED. Permanent that's both blocking creature and attacked planeswalker — complex type interaction, Phase 8+.

**506.4e** — DEFERRED. Permanent that's both attacked planeswalker and battle — Phase 8+.

---

**506.5** — PURE-DEF. "Attacks alone" / "blocking alone" definitions. Definitional — tested via keyword cards that use these terms.

---

**506.6** — TESTABLE. "Had to attack" check.

**Audit note — Tight coupling with goad:** As of 2026, there is only a single card that checks whether a creature "had to attack": Firkraag, Cunning Instigator (a Commander-only card that goads opponents' creatures, then triggers "Whenever a creature deals combat damage to one of your opponents, if that creature had to attack this combat..."). This mechanic is tightly coupled to goad and forced-attack effects. If WotC expands this design space, it will almost certainly remain tied to rules text that forces attacks (goad, Disrupt Decorum, etc.). The `had_to_attack` query is a natural byproduct of the combat requirements solver (T21d) — it just checks whether an attack requirement was active for that creature at declaration time. Low implementation cost once the requirements solver exists.

**ATOM-506.6-001**
- **Rule:** 506.6 — A creature had to attack if one or more effects were requiring it to attack at the time attackers were declared. A creature did not "have to attack" even if there were no other legal attacks.
- **Mechanism:** When checking "had to attack" for effects like Delirium ("if that creature had to attack"), check whether an attack requirement (must-attack-if-able, goad) was active for that creature at the time of declaration.
- **Minimal Board:** A goaded creature that attacked. An effect checks if it "had to attack."
- **Action:** Query whether the creature "had to attack."
- **Expected Result:** Returns true — the creature was goaded (a requirement was active).
- **Phase:** Phase 5 Pre-Work (T21d — combat requirements solver) + Phase 8 (cards using "had to attack")
- **Ticket:** T21d — combat requirements solver. The "had to attack" query is a natural extension.

---

**506.7** — PURE-DEF (parent). Spells with combat timing restrictions. Reference only.

**506.7a–506.7g** — These sub-rules define when "only before/after attackers/blockers declared" timing restrictions apply. They are all TESTABLE but belong to the **casting pipeline session (Session 5)** where timing restrictions on spells are tested.

- **506.7a:** "Before/after attackers declared" = before/after declare attackers step. → Session 5.
- **506.7b:** "Before/after blockers declared" = before/after declare blockers step. → Session 5.
- **506.7c:** "During combat" with multiple combat phases. → Session 5 + Phase 9.
- **506.7d:** "Before [point]" without "during combat" and multiple combat phases. → Session 5 + Phase 9.
- **506.7e:** "Before [point]" where point doesn't exist (508.8 skip). → Session 5.
- **506.7f:** "During combat after blockers declared" with blockers step skipped. → Session 5.
- **506.7g:** Same rules apply to abilities with combat timing. → Session 5.

---

### 507. Beginning of Combat Step

**507.1** — DEFERRED. In multiplayer, the active player chooses a defending player as a TBA. However, the only multiplayer variant we are explicitly supporting is Commander (4-player free-for-all), where all opponents automatically become defending players (rule 810.7). This rule only applies to multiplayer variants where a single defender must be chosen (e.g., non-Commander multi-player). Since Commander is the only planned multiplayer format, this TBA is never exercised.

**Audit note:** If a non-Commander multiplayer variant is ever added, this rule would need to be revisited. For now, the implicit 2-player behavior (nonactive player is the sole defender) and Commander's all-opponents-are-defenders are both handled without this TBA.

---

**507.2** — PURE-DEF. Active player gets priority after the TBA. Standard priority granting.

---

### 508. Declare Attackers Step

**508.1** — TESTABLE (parent + procedure). The active player declares attackers as a TBA.

**ATOM-508.1-001**
- **Rule:** 508.1 — The active player declares attackers. This turn-based action doesn't use the stack. If the declaration is illegal at any point, the game returns to the moment before the declaration.
- **Mechanism:** `process_declare_attackers` + `validate_attackers` must atomically validate the entire attack declaration. If any sub-step is illegal, the entire declaration is rolled back.
- **Minimal Board:** Active player controls 2 creatures, one of which is tapped.
- **Action:** Attempt to declare both as attackers.
- **Expected Result:** Declaration is illegal (tapped creature can't attack). The entire declaration is rejected — no creature attacks.
- **Phase:** Phase 3 (combat validation)
- **Ticket:** ALREADY-IMPLEMENTED — `validate_attackers` rejects illegal declarations

---

**508.1a** — TESTABLE (multiple clauses). Chosen attackers must be untapped, can't be battles, must have haste or have been controlled since turn began.

**ATOM-508.1a-001**
- **Rule:** 508.1a — The chosen creatures must be untapped.
- **Mechanism:** `validate_attackers` checks each proposed attacker is untapped.
- **Minimal Board:** A tapped creature.
- **Action:** Declare it as an attacker.
- **Expected Result:** Rejected — creature is tapped.
- **Phase:** Phase 3
- **Ticket:** ALREADY-IMPLEMENTED — `validate_attackers` checks untapped

**ATOM-508.1a-002**
- **Rule:** 508.1a — Each chosen creature must either have haste or have been controlled by the active player continuously since the turn began (summoning sickness).
- **Mechanism:** `validate_attackers` checks summoning sickness via `has_summoning_sickness` (or `controller_since_turn` after T09).
- **Minimal Board:** A creature that entered the battlefield this turn without haste.
- **Action:** Declare it as an attacker.
- **Expected Result:** Rejected — creature has summoning sickness.
- **Phase:** Phase 3 (combat) + Phase 4 (haste bypass)
- **Ticket:** ALREADY-IMPLEMENTED — `can_attack` checks summoning sickness; haste bypasses it

---

**508.1b** — TESTABLE. If defending player controls planeswalkers or battles, active player announces attack targets.

**ATOM-508.1b-001**
- **Rule:** 508.1b — If the defending player controls any planeswalkers, is the protector of any battles, or the game allows attacking multiple players, the active player announces which player, planeswalker, or battle each creature is attacking.
- **Mechanism:** When planeswalkers/battles exist, each attacker must have its attack target specified.
- **Minimal Board:** Defending player controls a planeswalker. Active player declares an attacker.
- **Action:** Active player must specify whether each attacker is attacking the player or the planeswalker.
- **Expected Result:** Each attacker has a specific attack target. DecisionProvider is consulted for the choice.
- **Phase:** Phase 8 (planeswalker combat targeting)
- **Ticket:** NEW — per-attacker target selection when PW/battles exist. Note: currently all attacks target the defending player directly.

---

**508.1c** — TESTABLE. Attack restrictions ("can't attack" / "can't attack unless") must be obeyed.

**ATOM-508.1c-001**
- **Rule:** 508.1c — The active player checks each creature for restrictions. If any restriction is disobeyed, the declaration is illegal.
- **Mechanism:** `validate_attackers` checks `AttackConstraints.restrictions` for each creature. If a restriction applies and is violated, the declaration is rejected.
- **Minimal Board:** A creature with Defender.
- **Action:** Declare it as an attacker.
- **Expected Result:** Rejected — Defender restriction prevents attacking.
- **Phase:** Phase 4 (Defender keyword)
- **Ticket:** ALREADY-IMPLEMENTED — `HasDefender` error in `validate_attackers`

**ATOM-508.1c-002**
- **Rule:** 508.1c (Example) — Two creatures that each "can't attack alone." Both declared as attackers is legal.
- **Mechanism:** "Can't attack alone" is a restriction that checks the total number of attackers, not a per-creature check. Both attacking satisfies the restriction.
- **Minimal Board:** Two creatures with "can't attack alone."
- **Action:** Declare both as attackers.
- **Expected Result:** Legal — neither is attacking alone.
- **Phase:** Phase 5 Pre-Work (T21d — combat requirements solver handles aggregate constraints)
- **Ticket:** T21d — combat requirements solver (aggregate restriction validation)

---

**508.1d** — TESTABLE. Attack requirements must be maximally obeyed.

**Audit note — Algorithm complexity:** The rule says the number of obeyed requirements must equal the "maximum possible number... that could be obeyed without disobeying any restrictions." This is a **constraint satisfaction / set maximization problem**. In the general case:

- Requirements are per-creature predicates ("this creature attacks if able", "all creatures attack if able", goad)
- Restrictions are per-creature or global predicates ("can't attack", "can't attack alone", "no more than N creatures can attack")
- The solver must find the **maximum cardinality subset** of requirements that can be simultaneously satisfied without violating any restriction

This is potentially NP-hard in the general case (it reduces to maximum independent set when restrictions create pairwise conflicts). In practice, the number of creatures + requirements + restrictions per combat is small enough (<20 in even extreme cases) that brute-force or backtracking is viable. However:

1. **New restriction types can be printed** — the solver must be extensible to new `AttackRestriction` variants without requiring algorithmic redesign.
2. **The solver must be provably correct** — it must find the TRUE maximum, not a greedy approximation. Greedy can fail: e.g., if satisfying requirement A for creature X prevents satisfying requirements B and C for creatures Y and Z.
3. **Recommended approach:** Model as integer linear programming over boolean variables (creature_i_attacks ∈ {0,1}), or use backtracking with pruning. The constraint set is small enough that exponential worst-case is acceptable.

This needs dedicated design work before implementation. The tests below define correctness criteria; the algorithm choice is an implementation concern.

**ATOM-508.1d-001**
- **Rule:** 508.1d — If the number of obeyed requirements is fewer than the maximum possible without disobeying any restriction, the declaration is illegal.
- **Mechanism:** The requirements solver must find the maximum set of satisfiable requirements and reject any declaration that doesn't achieve that maximum.
- **Minimal Board:** A creature with "attacks if able" and another creature with no abilities. A restriction says "no more than one creature can attack each turn."
- **Action:** Declare only the no-ability creature as attacker.
- **Expected Result:** Illegal — the "attacks if able" creature must attack (it's the only legal single-attacker choice that satisfies the maximum requirements).
- **Phase:** Phase 5 Pre-Work (T21d)
- **Ticket:** T21d — combat requirements solver (requirement maximization, 508.1d)

**ATOM-508.1d-002**
- **Rule:** 508.1d — A creature that "must attack if able" but can't attack unless a cost is paid — the player is NOT required to pay that cost.
- **Mechanism:** If attacking requires a cost (e.g., pay 2 life), the "must attack if able" requirement is vacuously satisfied when the player chooses not to pay.
- **Minimal Board:** A creature with "attacks if able" that also has "this creature can't attack unless you pay {2}."
- **Action:** Player chooses not to pay {2}. Creature doesn't attack.
- **Expected Result:** Legal — the player isn't required to pay costs to satisfy requirements.
- **Phase:** Phase 5 Pre-Work (T21d)
- **Ticket:** T21d — combat requirements solver (cost-gated requirements are optional)

---

**508.1e** — OUT-OF-SCOPE. Banding declarations — not in scope.

---

**508.1f** — TESTABLE. Attackers are tapped (not a cost; attacking causes tapping).

**ATOM-508.1f-001**
- **Rule:** 508.1f — The active player taps the chosen creatures. Tapping when declared as an attacker isn't a cost; attacking simply causes creatures to become tapped.
- **Mechanism:** `process_declare_attackers` taps each declared attacker (except those with vigilance).
- **Minimal Board:** An untapped creature declared as attacker.
- **Action:** Declare attackers.
- **Expected Result:** The creature becomes tapped. This is NOT a cost — it's a consequence of attacking.
- **Phase:** Phase 3 (combat steps)
- **Ticket:** ALREADY-IMPLEMENTED — `process_declare_attackers` in `steps.rs` taps attackers (vigilance skip per Phase 4)

---

**508.1g** — TESTABLE. Optional attack costs ("as this creature attacks, you may pay...").

**ATOM-508.1g-001**
- **Rule:** 508.1g — If there are optional costs to attack (expressed as costs a player may pay "as" a creature attacks), the active player chooses which to pay.
- **Mechanism:** DecisionProvider must be consulted for optional attack costs before the total cost is locked.
- **Minimal Board:** A creature with an optional attack cost (e.g., "As this creature attacks, you may pay {1}. If you do, it gets +1/+1 until end of turn.").
- **Action:** Declare the creature as an attacker.
- **Expected Result:** Player is prompted whether to pay the optional cost.
- **Phase:** Phase 8 (attack cost framework)
- **Ticket:** NEW — optional attack cost framework in declare attackers step

---

**508.1h** — TESTABLE. Total cost to attack is determined and locked.

**ATOM-508.1h-001**
- **Rule:** 508.1h — Once the total cost to attack is determined, it becomes "locked in." If effects would change the total cost after this time, ignore the change.
- **Mechanism:** After assembling all attack costs (mandatory + chosen optional), the total cost is frozen. Late effects don't modify it.
- **Minimal Board:** A creature with an attack cost. The cost is determined.
- **Action:** After cost determination, an effect would increase the attack cost.
- **Expected Result:** The cost change is ignored — the cost was locked.
- **Phase:** Phase 8 (attack cost framework)
- **Ticket:** NEW — attack cost lock-in mechanism

---

**508.1i** — TESTABLE (cross-cutting concern). Mana ability activation window before paying attack costs.

**Audit note — Cross-cutting mana ability windows:** This is one of several places in the CR where mana abilities get a special activation window during a structured process (others include: 601.2g during spell casting, 509.1e during block cost payment, 605.3a during resolution of mana abilities that produce less than needed). Mana abilities have unusual interactions across the rulebook:

- They don't use the stack (405.6c)
- They can be activated during the casting/activation process before costs are paid
- They can be activated during the resolution of other mana abilities (605.3a)
- They can trigger other mana abilities that also resolve immediately

The mana ability window during attack cost payment is not separately tested here because it follows the same pattern as the casting mana window (601.2g). However, the **structural requirement** is: wherever the engine has a "pay costs" step, it must open a mana ability window first. This should be verified as a cross-cutting concern when the attack/block cost framework (Phase 8) is implemented. Flagging for the Session 5 casting pipeline audit where the primary mana window tests live.

**508.1j** — PURE-DEF. Pay all attack costs; partial payments not allowed. Standard cost payment rule.

---

**508.1k** — TESTABLE. Each chosen creature becomes an attacking creature.

**ATOM-508.1k-001**
- **Rule:** 508.1k — Each chosen creature still controlled by the active player becomes an attacking creature. It remains attacking until removed from combat or combat phase ends.
- **Mechanism:** After all costs are paid, each declared attacker gets `AttackingInfo` set. If a creature changed controller between declaration and cost payment, it doesn't become attacking.
- **Minimal Board:** Two creatures declared as attackers. One changes controller before costs are paid (mid-declaration).
- **Action:** Complete the declare attackers process.
- **Expected Result:** Only the creature still controlled by the active player becomes attacking. The stolen creature does not.
- **Phase:** Phase 3 (combat steps) — basic case already implemented; control-change edge case is Phase 5
- **Ticket:** ALREADY-IMPLEMENTED (basic) + NEW — mid-declaration control change guard (Phase 5)

---

**508.1m** — TESTABLE. Trigger on attackers being declared.

**ATOM-508.1m-001**
- **Rule:** 508.1m — Any abilities that trigger on attackers being declared trigger.
- **Mechanism:** After all attackers are declared, the engine fires "whenever a creature attacks" triggers and other declare-attacker triggers.
- **Minimal Board:** A permanent with "Whenever a creature attacks, gain 1 life." A creature is declared as attacker.
- **Action:** Attackers are declared.
- **Expected Result:** The trigger fires and is placed on the stack.
- **Phase:** Phase 7 (Triggered Abilities)
- **Ticket:** NEW — declare-attacker trigger firing

---

**508.2** — PURE-DEF. Active player gets priority after TBA + triggers placed. Standard priority.

---

**508.2a** — TESTABLE. Attack trigger timing — triggers only at declaration point, not on later characteristic changes.

**ATOM-508.2a-001**
- **Rule:** 508.2a — Abilities that trigger on a creature attacking trigger only at the point the creature is declared as an attacker. They will not trigger if a creature's characteristics change to match the trigger condition after declaration.
- **Mechanism:** "Whenever a green creature attacks" only checks color at the moment of declaration. If a blue creature attacks and is later turned green, the trigger does NOT fire.
- **Minimal Board:** A permanent with "Whenever a green creature attacks, destroy that creature at end of combat." A blue creature is declared as attacker. Later, the creature is turned green.
- **Action:** Blue creature attacks, then is turned green.
- **Expected Result:** The trigger does NOT fire — the creature was blue when declared.
- **Phase:** Phase 7 (Triggered Abilities — trigger condition snapshot at declaration time)
- **Ticket:** NEW — attack trigger characteristic snapshot. Tags: META-trigger-timing

---

**508.2b** — PURE-DEF. Triggered abilities from 508.1 process go on stack before priority in APNAP order. Standard trigger placement rule.

---

**508.3** — PURE-DEF (parent). Different attack trigger conditions. The sub-rules define trigger semantics.

**508.3a–508.3f** — These are all Phase 7 (Triggered Abilities) trigger condition definitions. They define *when* specific trigger wordings fire. Each is TESTABLE within the triggered abilities system but belongs to Phase 7 implementation.

- **508.3a:** "Whenever [creature] attacks" = declared as attacker. Won't trigger for "put onto battlefield attacking." → Phase 7.
- **508.3b:** "Whenever [player/PW/battle] is attacked" = one or more attackers declared against it. → Phase 7.
- **508.3c:** "Whenever [player] attacks with [creature]" = player controls creature declared as attacker. → Phase 7.
- **508.3d:** "Whenever [player] attacks" = one or more creatures that player controls declared as attackers. → Phase 7.
- **508.3e:** "Whenever [player] attacks [another player]" = creatures declared attacking that player specifically. → Phase 7.
- **508.3f:** "Whenever [creature] attacks and isn't blocked" triggers during declare blockers step, not declare attackers. → Phase 7 + see 509.3g.

---

**508.4** — TESTABLE. Creature put onto battlefield attacking — controller chooses target; never "attacked."

**ATOM-508.4-001**
- **Rule:** 508.4 — If a creature is put onto the battlefield attacking, its controller chooses which defending player/PW/battle it's attacking. Such creatures are "attacking" but never "attacked" for trigger purposes.
- **Mechanism:** ETB-attacking creatures don't trigger "whenever [creature] attacks" abilities but ARE attacking creatures for all other purposes.
- **Minimal Board:** An effect puts a creature onto the battlefield attacking. A permanent has "whenever a creature attacks, draw a card."
- **Action:** Creature enters attacking.
- **Expected Result:** The creature is attacking, but the "whenever a creature attacks" trigger does NOT fire.
- **Phase:** Phase 8 (ETB-attacking effects)
- **Ticket:** NEW — ETB-attacking creatures don't trigger "attacks" triggers

---

**508.4a–508.4d** — Already covered by 506.3a–506.3d tests. Redundant with those earlier rules.

- **508.4a:** Same as 506.3c — invalid attack target, creature enters but doesn't attack.
- **508.4b:** Same concept for "is stated to be attacking."
- **508.4c:** ETB-attacking creatures aren't affected by attack requirements/restrictions.
- **508.4d:** Same as 506.3d — late ETB-attacking enters unblocked.

---

**508.5** — PURE-DEF. "Defending player" for an attacking creature refers to the player/PW-controller/battle-protector being attacked. Definitional for effect resolution.

**508.5a** — DEFERRED. Multiplayer "defending player" disambiguation — Phase 9.

---

**508.6** — PURE-DEF. "[Player] is attacking [player]" = first player controls creature attacking second player. Definitional.

---

**508.7** — DEFERRED. Reselecting attack targets — Phase 8+ (niche mechanic).

**508.7a–508.7e** — DEFERRED. Details of attack reselection — Phase 8+/Phase 9.

---

**508.8** — TESTABLE. Skip declare blockers and combat damage steps if no attackers.

**ATOM-508.8-001**
- **Rule:** 508.8 — If no creatures are declared as attackers or put onto the battlefield attacking, skip the declare blockers and combat damage steps.
- **Mechanism:** `Game::run_turn` must skip declare blockers, first-strike damage, and combat damage steps entirely when `!attacks_declared`.
- **Minimal Board:** Active player has creatures but declares no attackers.
- **Action:** Declare attackers step completes with zero attackers.
- **Expected Result:** Declare blockers step, first-strike damage step, and combat damage step are all skipped. Phase proceeds to end of combat step.
- **Phase:** Phase 3 (combat wiring, post-audit)
- **Ticket:** ALREADY-IMPLEMENTED — `skipped_by_508_8` flag in `Game::run_turn`

---

### 509. Declare Blockers Step

**509.1** — TESTABLE (parent + procedure). The defending player declares blockers as a TBA.

**ATOM-509.1-001**
- **Rule:** 509.1 — The defending player declares blockers. This TBA doesn't use the stack. If the declaration is illegal, the game returns to before the declaration.
- **Mechanism:** `process_declare_blockers` + `validate_blockers` must atomically validate the entire block declaration.
- **Minimal Board:** Defending player controls 2 creatures. An attacker is attacking.
- **Action:** Defend with both creatures blocking the same attacker.
- **Expected Result:** Legal — multiple blockers on one attacker is permitted (unless attacker has menace requiring ≥2, which is satisfied).
- **Phase:** Phase 3 (combat validation)
- **Ticket:** ALREADY-IMPLEMENTED — `validate_blockers` validates block declarations

---

**509.1a** — TESTABLE (multiple clauses). Chosen blockers must be untapped, can't be battles, and each blocks one attacking creature.

**ATOM-509.1a-001**
- **Rule:** 509.1a — The chosen creatures must be untapped.
- **Mechanism:** `validate_blockers` checks each blocker is untapped.
- **Minimal Board:** Defending player controls a tapped creature.
- **Action:** Attempt to declare the tapped creature as a blocker.
- **Expected Result:** Rejected — tapped creatures can't block.
- **Phase:** Phase 3 (combat validation)
- **Ticket:** ALREADY-IMPLEMENTED — `validate_blockers` checks untapped status

**ATOM-509.1a-002**
- **Rule:** 509.1a — Each chosen creature blocks exactly one attacking creature that is attacking the blocker's controller (or their PW/battle).
- **Mechanism:** Each blocker must be assigned to exactly one attacker. A blocker can't be assigned to an attacker that is attacking a different player (in multiplayer) or to no attacker.
- **Minimal Board:** Two-player game. Defending player controls a creature. Two attackers are attacking the defending player.
- **Action:** Declare the defender's creature as blocking both attackers simultaneously.
- **Expected Result:** Rejected — each blocker blocks exactly one attacker (unless an effect like "this creature can block an additional creature" applies).
- **Phase:** Phase 3 (combat validation)
- **Ticket:** ALREADY-IMPLEMENTED — `validate_blockers` enforces one-attacker-per-blocker

**ATOM-509.1a-003**
- **Rule:** 509.1a — In multiplayer, a blocker can only block an attacker that is attacking its controller (or their PW/battle).
- **Mechanism:** In a Commander game, Player B's creatures can only block attackers that are attacking Player B (or Player B's planeswalkers/battles). They cannot block attackers aimed at Player C.
- **Minimal Board:** 3-player Commander game. Player A attacks Player C. Player B attempts to declare a blocker against Player A's creature.
- **Action:** Player B declares a blocker.
- **Expected Result:** Rejected — Player B can only block attackers targeting Player B.
- **Phase:** Phase 9 (multiplayer combat)
- **Ticket:** NEW — multiplayer blocker controller validation

---

**509.1b** — TESTABLE. Blocking restrictions (evasion abilities) must be obeyed.

**ATOM-509.1b-001**
- **Rule:** 509.1b — The defending player checks each creature for blocking restrictions. If any restriction is disobeyed, the declaration is illegal. Evasion abilities are cumulative.
- **Mechanism:** Per-pair evasion checks (flying, shadow, fear, etc.) and aggregate checks (menace) must all pass.
- **Minimal Board:** An attacking creature with flying. A blocker without flying or reach.
- **Action:** Declare the ground creature as blocking the flyer.
- **Expected Result:** Rejected — ground creature can't block a flyer.
- **Phase:** Phase 4 (Flying/Reach evasion)
- **Ticket:** ALREADY-IMPLEMENTED — flying check in `validate_blockers`

**ATOM-509.1b-002**
- **Rule:** 509.1b — Evasion abilities are cumulative.
- **Mechanism:** An attacker with both flying AND shadow requires a blocker that has BOTH flying (or reach) AND shadow.
- **Minimal Board:** An attacker with flying and shadow. A blocker with flying but no shadow.
- **Action:** Declare the flying-only creature as blocker.
- **Expected Result:** Rejected — blocker needs shadow too (evasion abilities stack).
- **Phase:** Phase 5 Pre-Work (T21b — evasion framework expansion, shadow keyword)
- **Ticket:** T21b — evasion framework (E37: cumulative evasion check)

**ATOM-509.1b-003**
- **Rule:** 509.1b — If an attacking creature gains or loses an evasion ability after a legal block has been declared, it doesn't affect that block.
- **Mechanism:** After blockers are legally declared, changing the attacker's evasion abilities (e.g., granting flying) doesn't invalidate existing blocks.
- **Minimal Board:** A ground creature blocks another ground creature. After block declaration, the attacker gains flying.
- **Action:** Attacker gains flying after blocks are declared.
- **Expected Result:** The block remains valid. The blocker still blocks the now-flying attacker.
- **Phase:** Phase 3 (combat) — block state is locked after declaration
- **Ticket:** ALREADY-IMPLEMENTED — block state locked after declaration; no re-validation

---

**509.1c** — TESTABLE. Blocking requirements must be maximally obeyed.

**Audit note — Algorithm complexity (same as 508.1d):** The blocking requirements solver faces the same constraint satisfaction problem as the attack requirements solver. The solver must find the maximum set of blocking requirements that can be obeyed without violating any blocking restriction (evasion, menace, "can't block" effects). The same design considerations apply: backtracking or ILP over boolean variables, extensibility to new restriction types, provably correct maximization. The blocking solver is likely even more complex because it must also respect per-pair evasion checks (flying, shadow, fear, etc.) as constraints that interact with aggregate requirements (menace). The attack and blocking solvers should share infrastructure where possible.

**ATOM-509.1c-001**
- **Rule:** 509.1c — If the number of blocking requirements obeyed is fewer than the maximum possible without disobeying any restriction, the declaration is illegal.
- **Mechanism:** Requirements solver for blocking — same pattern as 508.1d for attacking.
- **Minimal Board:** A creature with "must block if able." One attacker is attacking. Defender has the "must block" creature and one other. The "must block" creature is not declared as a blocker.
- **Action:** Declare only the other creature as a blocker.
- **Expected Result:** Illegal — the "must block if able" creature should also block (it can, and blocking it satisfies one more requirement).
- **Phase:** Phase 5 Pre-Work (T21d)
- **Ticket:** T21d — combat requirements solver (block requirement maximization, 509.1c)

**ATOM-509.1c-002**
- **Rule:** 509.1c (Example) — A creature that "blocks if able" and a creature with no abilities. Attacker with menace. Player must block with both.
- **Mechanism:** Menace requires ≥2 blockers. The "blocks if able" creature must block. Blocking alone violates menace. Blocking with both satisfies menace AND the blocking requirement.
- **Minimal Board:** Attacker with menace. Defender has "blocks if able" creature + no-abilities creature.
- **Action:** Declare only the "blocks if able" creature.
- **Expected Result:** Illegal — menace requires 2+ blockers. Must declare both (satisfies both the menace restriction and the blocking requirement).
- **Phase:** Phase 5 Pre-Work (T21d + T21b menace)
- **Ticket:** T21d — combat requirements solver + T21b menace enforcement

---

**509.1d** — TESTABLE. Blocking costs are determined and locked.

**ATOM-509.1d-001**
- **Rule:** 509.1d — If any chosen creatures require paying costs to block, the total cost is determined and locked in.
- **Mechanism:** Same pattern as 508.1h but for blocking costs.
- **Minimal Board:** A creature with a blocking cost (e.g., "This creature can't block unless you pay {1}").
- **Action:** Declare it as a blocker.
- **Expected Result:** The blocking cost is determined and must be paid. After locking, cost changes are ignored.
- **Phase:** Phase 8 (blocking cost framework)
- **Ticket:** NEW — blocking cost determination and lock-in

---

**509.1e** — PURE-DEF. Mana ability window before paying block costs. Standard mana ability window.

**509.1f** — PURE-DEF. Pay all blocking costs; partial payments not allowed. Standard cost payment.

---

**509.1g** — TESTABLE. Each chosen creature becomes a blocking creature.

**ATOM-509.1g-001**
- **Rule:** 509.1g — Each chosen creature still controlled by the defending player becomes a blocking creature. It remains blocking until removed from combat or combat phase ends.
- **Mechanism:** After costs are paid, each declared blocker gets `BlockingInfo` set, linking it to the attacker it's blocking.
- **Minimal Board:** Defender declares a creature as blocker.
- **Action:** Block declaration completes.
- **Expected Result:** The creature has `BlockingInfo` set, referencing the attacker.
- **Phase:** Phase 3 (combat steps)
- **Ticket:** ALREADY-IMPLEMENTED — `process_declare_blockers` in `steps.rs` sets blocking state

---

**509.1h** — TESTABLE (multiple clauses). Attackers become blocked or unblocked.

**ATOM-509.1h-001**
- **Rule:** 509.1h — An attacking creature with one or more blockers declared for it becomes a blocked creature; one with no blockers becomes unblocked. A creature remains blocked even if all blockers are removed from combat.
- **Mechanism:** After block declaration, each attacker is marked as blocked or unblocked. A blocked creature stays blocked even if its blockers are later destroyed.
- **Minimal Board:** An attacker blocked by a single creature. The blocker is then destroyed before combat damage.
- **Action:** Blocker is destroyed after block declaration.
- **Expected Result:** The attacker remains "blocked." It assigns no combat damage (it's blocked with zero remaining blockers per 510.1c).
- **Phase:** Phase 3 (combat)
- **Ticket:** ALREADY-IMPLEMENTED — `blocked` flag persists after blocker removal

---

**509.1i** — PURE-DEF. Triggers on blockers being declared fire at this point. Covered by Phase 7.

---

**509.2** — PURE-DEF. Active player gets priority after block declaration. Standard priority.

**509.2a** — PURE-DEF. Block triggers go on stack before priority in APNAP order. Standard trigger placement.

---

**509.3** — PURE-DEF (parent). Different block trigger conditions.

**509.3a–509.3g** — All Phase 7 (Triggered Abilities) trigger condition definitions for blocking events.

- **509.3a:** "Whenever [creature] blocks" — triggers once per combat, at block declaration or when becoming a blocker via effect. → Phase 7.
- **509.3b:** "Whenever [creature] blocks a creature" — triggers once per blocked attacker. → Phase 7.
- **509.3c:** "Whenever [creature] becomes blocked" — triggers once when creature transitions from unblocked to blocked. → Phase 7.
- **509.3d:** "Whenever [creature] becomes blocked by a creature" — triggers per blocker. → Phase 7.
- **509.3e:** "Whenever [creature] blocks/becomes blocked by N creatures" — count-based trigger. → Phase 7.
- **509.3f:** Trigger checks characteristics at the moment of blocking, not later. Has Example. → Phase 7.
- **509.3g:** "Whenever [creature] attacks and isn't blocked" — triggers during declare blockers step when no blockers declared for it. → Phase 7.

---

**509.4** — TESTABLE. Creature put onto battlefield blocking — controller chooses which attacker; never "blocked."

**ATOM-509.4-001**
- **Rule:** 509.4 — If a creature is put onto the battlefield blocking, its controller chooses which attacking creature it's blocking. The creature is "blocking" but never "blocked" for trigger purposes.
- **Mechanism:** ETB-blocking creatures don't trigger "whenever [creature] blocks" but ARE blocking creatures.
- **Minimal Board:** An effect puts a creature onto the battlefield blocking.
- **Action:** Creature enters blocking.
- **Expected Result:** The creature is blocking the chosen attacker, but "whenever a creature blocks" triggers do NOT fire.
- **Phase:** Phase 8 (ETB-blocking effects)
- **Ticket:** NEW — ETB-blocking creature target selection and trigger suppression

---

**509.4a** — TESTABLE. ETB-blocking creature targeting invalid attacker — enters but doesn't block.

**ATOM-509.4a-001**
- **Rule:** 509.4a — If the specified attacker is no longer attacking, the creature enters but is never a blocking creature. Same if the entering creature's controller isn't a defending player for that attacker.
- **Mechanism:** ETB-blocking must validate the target attacker is still attacking and the entering creature's controller is the defending player.
- **Minimal Board:** An effect puts a creature onto the battlefield blocking an attacker that was already removed from combat.
- **Action:** The effect resolves.
- **Expected Result:** The creature enters the battlefield but is NOT blocking.
- **Phase:** Phase 8
- **Ticket:** NEW — ETB-blocking target validation

---

**509.4b** — TESTABLE. ETB-blocking creatures aren't affected by blocking requirements/restrictions and ignore evasion.

**Audit note — Evasion bypass:** This rule means a creature put onto the battlefield blocking via an effect ignores ALL blocking restrictions, including evasion. A ground creature can be put onto the battlefield blocking a flyer. A creature without shadow can be put onto the battlefield blocking a creature with shadow. This is correct — the restrictions in 509.1b only apply to the blocking *declaration* process, not to creatures that enter already blocking. Since no current card exercises this mechanic (see 506.3e audit note), this is academic but structurally important for correctness if such effects are ever printed.

**ATOM-509.4b-001**
- **Rule:** 509.4b — A creature put onto the battlefield blocking isn't subject to blocking requirements or restrictions.
- **Mechanism:** ETB-blocking bypasses all blocking restrictions (evasion, "can't block" effects, menace, etc.). The creature is simply placed as blocking.
- **Minimal Board:** An attacking creature with flying. An effect puts a ground creature (no flying, no reach) onto the battlefield blocking the flyer.
- **Action:** The effect resolves.
- **Expected Result:** The ground creature is blocking the flyer. The flying evasion check does NOT apply — the creature was never "declared" as a blocker.
- **Phase:** Phase 8 (ETB-blocking effects)
- **Ticket:** NEW — ETB-blocking evasion bypass. **Low priority** — no existing card exercises this rule (see 506.3e audit note).

---

### 510. Combat Damage Step

**510.1** — TESTABLE (parent + procedure). Active player announces attacking damage assignment, then defending player announces blocking damage assignment. This is a TBA.

**ATOM-510.1-001**
- **Rule:** 510.1 — First, the active player announces how each attacking creature assigns its combat damage, then the defending player announces how each blocking creature assigns its combat damage. This turn-based action doesn't use the stack.
- **Mechanism:** `assign_combat_damage` is called for active player (attackers) then defending player (blockers). Assignments are made via DecisionProvider and validated before being applied.
- **Minimal Board:** One attacker blocked by two creatures.
- **Action:** Combat damage step begins.
- **Expected Result:** Active player is prompted to divide the attacker's damage among the two blockers. Then defending player's blockers each assign their damage to the attacker they're blocking.
- **Phase:** Phase 3 (combat damage)
- **Ticket:** ALREADY-IMPLEMENTED — `assign_combat_damage` in `resolution.rs` with DecisionProvider

---

**510.1a** — TESTABLE. Creatures assign combat damage equal to their power; 0 or less power assigns no damage.

**ATOM-510.1a-001**
- **Rule:** 510.1a — Each attacking creature and each blocking creature assigns combat damage equal to its power. Creatures that would assign 0 or less damage this way don't assign combat damage at all.
- **Mechanism:** Combat damage equals `get_effective_power`. If power ≤ 0, no damage is assigned.
- **Minimal Board:** An attacking creature with 0 power (e.g., a 0/4 wall that somehow attacks).
- **Action:** Combat damage step.
- **Expected Result:** The 0-power creature assigns no combat damage.
- **Phase:** Phase 3 (combat damage)
- **Ticket:** ALREADY-IMPLEMENTED — damage assignment uses effective power; zero/negative power assigns nothing

---

**510.1b** — TESTABLE (multiple clauses). Unblocked creature assigns to what it's attacking; orphaned attacker assigns nothing.

**ATOM-510.1b-001**
- **Rule:** 510.1b — An unblocked creature assigns its combat damage to the player, planeswalker, or battle it's attacking.
- **Mechanism:** Unblocked attackers deal damage to their attack target (defending player in 2-player games).
- **Minimal Board:** An unblocked attacking creature.
- **Action:** Combat damage step.
- **Expected Result:** The creature's power is dealt as damage to the defending player.
- **Phase:** Phase 3 (combat damage)
- **Ticket:** ALREADY-IMPLEMENTED — unblocked attackers deal damage to defending player

**ATOM-510.1b-002**
- **Rule:** 510.1b — If an unblocked attacker isn't currently attacking anything (e.g., its PW target left the battlefield), it assigns no combat damage.
- **Mechanism:** An orphaned attacker (attack target gone) deals zero damage. See also 506.4c.
- **Minimal Board:** A creature that was attacking a planeswalker. The PW was destroyed during combat.
- **Action:** Combat damage step.
- **Expected Result:** The creature assigns no combat damage (target no longer exists).
- **Phase:** Phase 8 (planeswalker combat)
- **Ticket:** NEW — orphaned attacker assigns no damage (same ticket as ATOM-506.4c-001)

---

**510.1c** — TESTABLE (multiple clauses, Example). Blocked creature damage division rules (2025 rules — no ordering).

**ATOM-510.1c-001**
- **Rule:** 510.1c — A blocked creature assigns its combat damage to the creatures blocking it. If no blockers remain, it assigns no damage.
- **Mechanism:** A blocked creature with zero remaining blockers (all removed from combat) assigns no combat damage.
- **Minimal Board:** An attacker blocked by one creature. The blocker is destroyed before combat damage.
- **Action:** Combat damage step.
- **Expected Result:** The attacker is still blocked but has no valid targets — it assigns no combat damage.
- **Phase:** Phase 3 (combat)
- **Ticket:** ALREADY-IMPLEMENTED — blocked creature with no remaining blockers assigns nothing

**ATOM-510.1c-002**
- **Rule:** 510.1c — If exactly one creature is blocking, all damage goes to that creature.
- **Mechanism:** Single-blocker case: all attacker damage assigned to the sole blocker.
- **Minimal Board:** A 3/3 attacker blocked by a single 2/2.
- **Action:** Combat damage step.
- **Expected Result:** All 3 damage assigned to the 2/2 blocker.
- **Phase:** Phase 3
- **Ticket:** ALREADY-IMPLEMENTED — single blocker gets all damage

**ATOM-510.1c-003**
- **Rule:** 510.1c — If two or more creatures are blocking it, the attacking player divides damage freely among them (2025 rules — no ordering, no lethal-first constraint).
- **Mechanism:** Under 2025 rules, the active player freely divides damage among blockers via `choose_attacker_damage_assignment`. No damage ordering, no lethal-first requirement.
- **Minimal Board:** A 4/3 attacker blocked by a 2/3 and a 1/1. (Per the CR Example: Elvish Regrower blocked by Vampire Spawn and Helpful Hunter.)
- **Action:** Active player divides 4 damage among the two blockers.
- **Expected Result:** Any division is legal: 4-0, 3-1, 2-2, 1-3, 0-4. DecisionProvider chooses.
- **Phase:** Phase 3 (post-audit: 2025 rules removed damage ordering)
- **Ticket:** ALREADY-IMPLEMENTED — `choose_attacker_damage_assignment` freely divides damage (Phase 3 post-audit removed ordering)

---

**510.1d** — TESTABLE. Blocking creature damage division (2025 rules).

**ATOM-510.1d-001**
- **Rule:** 510.1d — A blocking creature assigns combat damage to the creatures it's blocking. If blocking exactly one, all damage to that creature. If blocking two or more, freely divided.
- **Mechanism:** Blocking creatures that block multiple attackers (rare but possible via effects) divide their damage freely.
- **Minimal Board:** A blocker that is blocking two attackers (via an effect like "this creature can block an additional creature"). The blocker is a 4/4.
- **Action:** Combat damage step — defending player divides 4 damage.
- **Expected Result:** Any division among the two attackers is legal.
- **Phase:** Phase 8 (multi-block effects — currently creatures block exactly one attacker)
- **Ticket:** NEW — blocker blocking multiple attackers damage division. Note: current engine only supports one blocked attacker per blocker.

---

**510.1e** — TESTABLE. Total damage assignment legality check.

**ATOM-510.1e-001**
- **Rule:** 510.1e — Once a player has assigned combat damage from each creature, the total assignment is checked for legality. If illegal, the assignment is rolled back (rule 732).
- **Mechanism:** After all damage assignments, the engine validates the total. For example, a creature can't assign more damage than its power. If the assignment is illegal, the game returns to before the assignment.
- **Minimal Board:** A 3/3 attacker blocked by two creatures. Active player tries to assign 2+2=4 total damage.
- **Action:** Validate total assignment.
- **Expected Result:** Illegal — total exceeds power (3). Assignment is rolled back.
- **Phase:** Phase 3 (combat damage validation)
- **Ticket:** ALREADY-IMPLEMENTED — damage assignment validation in `assign_combat_damage`

---

**510.2** — TESTABLE. All combat damage dealt simultaneously.

**ATOM-510.2-001**
- **Rule:** 510.2 — Second, all combat damage that's been assigned is dealt simultaneously. This TBA doesn't use the stack. No player can respond between assignment and dealing.
- **Mechanism:** All damage from all creatures (both attacking and blocking) is dealt at the same instant. No priority between assignment and dealing.
- **Minimal Board:** A 3/3 attacker blocked by a 2/2. Both assigned damage to each other.
- **Action:** Combat damage is dealt.
- **Expected Result:** Both creatures receive damage simultaneously. The 2/2 has 3 damage marked (lethal). The 3/3 has 2 damage marked (not lethal). SBAs will destroy the 2/2 before priority.
- **Phase:** Phase 3 (combat damage)
- **Ticket:** ALREADY-IMPLEMENTED — `deal_combat_damage` applies all damage simultaneously

---

**510.3** — TESTABLE. Active player gets priority after combat damage is dealt.

**ATOM-510.3-001**
- **Rule:** 510.3 — After combat damage is dealt, the active player gets priority. SBAs are checked before priority is granted.
- **Mechanism:** After `deal_combat_damage`, the engine must check SBAs (which will destroy creatures with lethal damage) and then grant priority to the active player. This priority window is important — it's where players can respond to combat results (e.g., cast a spell before the end of combat step).
- **Minimal Board:** A 3/3 attacker blocked by a 2/2. Combat damage dealt (2/2 dies from SBA). Active player has an instant in hand.
- **Action:** Combat damage is dealt. SBAs destroy the 2/2.
- **Expected Result:** Active player receives priority after SBAs. The 2/2 is in the graveyard. Active player can cast the instant before the end of combat step.
- **Phase:** Phase 3 (combat)
- **Ticket:** ALREADY-IMPLEMENTED — priority is granted after combat damage step processing

**510.3a** — TESTABLE. Damage-triggered abilities go on stack before priority. Phase 7 (triggered abilities).

---

**510.4** — TESTABLE. First strike / double strike two-step combat damage.

**ATOM-510.4-001**
- **Rule:** 510.4 — If at least one attacker or blocker has first strike or double strike as the combat damage step begins, only those creatures assign damage in the first step. A second combat damage step follows for remaining creatures + double strikers.
- **Mechanism:** First combat damage step: only first-strike and double-strike creatures deal damage. Second step: non-first-strike creatures + double-strike creatures deal damage.
- **Minimal Board:** A 2/2 first-strike attacker blocked by a 3/3 without first strike.
- **Action:** First combat damage step.
- **Expected Result:** First step: 2/2 first-striker deals 2 damage to 3/3. Second step: 3/3 deals 3 damage to 2/2. Both survive (2 damage on 3/3, 3 damage on 2/2 — only 2/2 dies from SBA after second step).
- **Phase:** Phase 4 (First Strike / Double Strike keywords)
- **Ticket:** ALREADY-IMPLEMENTED — `should_deal_damage_this_step` in `combat/keywords.rs`

**ATOM-510.4-002**
- **Rule:** 510.4 — If no creature has first strike or double strike, there is only one combat damage step.
- **Mechanism:** When no first strike / double strike creatures are present, only one damage step occurs.
- **Minimal Board:** A 3/3 attacker and a 2/2 blocker, neither with first strike.
- **Action:** Combat damage step.
- **Expected Result:** Only one damage step occurs. Both creatures deal damage simultaneously.
- **Phase:** Phase 3 (combat)
- **Ticket:** ALREADY-IMPLEMENTED — first-strike split only occurs when at least one creature has first/double strike

**ATOM-510.4-003**
- **Rule:** 510.4 — Double-strike creature deals damage in BOTH steps.
- **Mechanism:** A creature with double strike assigns/deals damage in both the first-strike step and the normal step.
- **Minimal Board:** A 3/3 double-strike attacker blocked by a 5/5.
- **Action:** Both combat damage steps.
- **Expected Result:** First step: 3 damage to blocker (3/5 marked). Second step: 3 more damage to blocker (6/5 marked, dies to SBA). Blocker deals 5 damage in second step.
- **Phase:** Phase 4 (Double Strike)
- **Ticket:** ALREADY-IMPLEMENTED — `should_deal_damage_this_step` returns true for both steps when creature has double strike

---

### 511. End of Combat Step

**511.1** — PURE-DEF. No TBA; active player gets priority. Standard priority grant.

---

**511.2** — TESTABLE. "At end of combat" triggers fire; "until end of combat" effects expire at END of combat phase.

**ATOM-511.2-001**
- **Rule:** 511.2 — Abilities that trigger "at end of combat" trigger as the end of combat step begins.
- **Mechanism:** "At end of combat" triggered abilities fire when the end of combat step begins, before priority.
- **Minimal Board:** A permanent with "At end of combat, exile target attacking creature."
- **Action:** End of combat step begins.
- **Expected Result:** The trigger fires and goes on the stack.
- **Phase:** Phase 7 (Triggered Abilities)
- **Ticket:** NEW — end-of-combat step beginning triggers

**ATOM-511.2-002**
- **Rule:** 511.2 — Effects that last "until end of combat" expire at the end of the combat phase.
- **Mechanism:** `UntilEndOfCombat` duration effects expire when the combat phase ends (after the end of combat step completes), not when the end of combat step begins. See also 500.5a.
- **Minimal Board:** A creature with +2/+2 "until end of combat." End of combat step.
- **Action:** End of combat step completes.
- **Expected Result:** The creature still has +2/+2 during the end of combat step. Effect expires after the step finishes.
- **Phase:** Phase 5 Pre-Work (T22)
- **Ticket:** T22 — `UntilEndOfCombat` duration expiry (same as ATOM-500.5a-001)

---

**511.3** — TESTABLE. All creatures/battles/planeswalkers removed from combat at end of combat step.

**ATOM-511.3-001**
- **Rule:** 511.3 — As soon as the end of combat step ends, all creatures, battles, and planeswalkers are removed from combat.
- **Mechanism:** At the end of the combat phase, all `AttackingInfo` and `BlockingInfo` is cleared from all permanents.
- **Minimal Board:** After combat, creatures still have attacking/blocking state.
- **Action:** End of combat step ends.
- **Expected Result:** All creatures' `AttackingInfo` and `BlockingInfo` are cleared. No creature is "in combat" during the postcombat main phase.
- **Phase:** Phase 3 (combat cleanup)
- **Ticket:** ALREADY-IMPLEMENTED — combat state cleared at end of combat phase in `turns.rs`

---

### 512. Ending Phase

**512.1** — PURE-DEF. Ending phase has two steps: end and cleanup. Structural definition.

---

### 513. End Step

**513.1** — PURE-DEF. No TBA; active player gets priority. Standard.

**513.1a** — PURE-DEF. Historical errata note: "at end of turn" → "at the beginning of the end step." Not testable.

---

**513.2** — TESTABLE. End-step trigger timing — entering permanent or created delayed trigger during end step waits until next turn.

**ATOM-513.2-001**
- **Rule:** 513.2 — If a permanent with "at the beginning of the end step" enters the battlefield during the end step, that ability won't trigger until the next turn's end step.
- **Mechanism:** A permanent that ETBs during the end step with an "at the beginning of the end step" trigger does NOT get that trigger this turn — it waits until next turn's end step. The end step doesn't "back up."
- **Minimal Board:** During the end step, an effect creates a token with "At the beginning of the end step, sacrifice this creature."
- **Action:** Token enters during the end step.
- **Expected Result:** The sacrifice trigger does NOT fire this end step. It fires during the next turn's end step.
- **Phase:** Phase 7 (Triggered Abilities — trigger timing, "already happened this turn" check)
- **Ticket:** NEW — end-step trigger "no back up" rule. The trigger system must track whether a step has already begun.

**ATOM-513.2-002**
- **Rule:** 513.2 — A delayed triggered ability created during this step that triggers "at the beginning of the next end step" won't trigger until the next turn's end step.
- **Mechanism:** "At the beginning of the next end step" delayed triggers created during an end step don't trigger during the same end step — they wait for the next turn.
- **Minimal Board:** During the end step, an effect creates a delayed trigger: "At the beginning of the next end step, return exiled card."
- **Action:** Delayed trigger is created during end step.
- **Expected Result:** The delayed trigger waits until the next turn's end step to fire.
- **Phase:** Phase 7 (Triggered Abilities — delayed trigger timing)
- **Ticket:** NEW — delayed trigger "at the beginning of the next end step" timing during end step

**ATOM-513.2-003**
- **Rule:** 513.2 — This no-back-up rule does NOT apply to continuous effects with "until end of turn" or "this turn" durations.
- **Mechanism:** A continuous effect created during the end step with "until end of turn" still expires during the cleanup step this turn — it doesn't carry over.
- **Minimal Board:** During the end step, an effect grants a creature +2/+2 "until end of turn."
- **Action:** Cleanup step begins.
- **Expected Result:** The +2/+2 effect expires this turn during cleanup (514.2). It does NOT carry over.
- **Phase:** Phase 5 Pre-Work (T22 — duration expiry)
- **Ticket:** T22 — UntilEndOfTurn duration expires in cleanup (rule 514.2)

---

### 514. Cleanup Step

**514.1** — TESTABLE. Active player discards to hand size as TBA.

**ATOM-514.1-001**
- **Rule:** 514.1 — First, if the active player's hand contains more cards than their maximum hand size (normally seven), they discard enough cards to reduce their hand size to that number. This TBA doesn't use the stack.
- **Mechanism:** `Game::run_turn` cleanup step checks `hand.len() > max_hand_size` and calls `choose_discard` via DecisionProvider.
- **Minimal Board:** Active player has 9 cards in hand, max hand size 7.
- **Action:** Cleanup step begins.
- **Expected Result:** Player is prompted to discard 2 cards. After discarding, hand has 7 cards. This doesn't use the stack.
- **Phase:** Phase 1 (turn structure)
- **Ticket:** ALREADY-IMPLEMENTED — cleanup discard in `turns.rs` (verified Phase 1 audit)

---

**514.2** — TESTABLE. Damage removed + "until end of turn" / "this turn" effects end simultaneously.

**ATOM-514.2-001**
- **Rule:** 514.2 — Second, all damage marked on permanents (including phased-out permanents) is removed and all "until end of turn" and "this turn" effects end. These happen simultaneously. This TBA doesn't use the stack.
- **Mechanism:** At cleanup, `remove_damage_from_all_permanents()` and `expire_until_end_of_turn_effects()` both execute simultaneously.
- **Minimal Board:** A creature with 2 damage marked and a +2/+2 "until end of turn" effect.
- **Action:** Cleanup step, after discard.
- **Expected Result:** Damage is removed AND the +2/+2 effect expires simultaneously. If the creature is a 1/1 with the +2/+2 (making it 3/3) and has 2 damage, both changes happen at once: it becomes a 1/1 with 0 damage (not a 1/1 with 2 damage that would die).
- **Phase:** Phase 1 (damage removal) + Phase 5 Pre-Work (T22 — duration expiry)
- **Ticket:** ALREADY-IMPLEMENTED (damage removal in `turns.rs`) + T22 (duration expiry hooks — "until end of turn" effects end simultaneously with damage removal)

---

**514.3** — TESTABLE. Normally no priority during cleanup.

**ATOM-514.3-001**
- **Rule:** 514.3 — Normally, no player receives priority during the cleanup step, so no spells can be cast and no abilities can be activated.
- **Mechanism:** After discard and damage/effects cleanup, the step ends without priority (unless 514.3a applies).
- **Minimal Board:** Active player has 7 or fewer cards. No SBAs or triggers pending.
- **Action:** Cleanup step processes.
- **Expected Result:** Step ends immediately after TBAs. No priority window.
- **Phase:** Phase 1 (turn structure)
- **Ticket:** ALREADY-IMPLEMENTED — cleanup step skips priority when no SBAs/triggers

---

**514.3a** — TESTABLE. Cleanup re-loop: if SBAs or triggers pending after cleanup TBAs, perform SBAs → put triggers on stack → grant priority → new cleanup step.

**ATOM-514.3a-001**
- **Rule:** 514.3a — If any state-based actions would be performed and/or any triggered abilities are waiting to be put onto the stack after cleanup TBAs, perform those SBAs, put triggers on the stack, then the active player gets priority. Once the stack is empty and all players pass, another cleanup step begins.
- **Mechanism:** After 514.1 and 514.2, the engine checks for SBAs and pending triggers. If any exist, it performs SBAs, places triggers on stack, grants priority, and when the stack empties + all pass, it starts a NEW cleanup step (re-loop).
- **Minimal Board:** A creature with 3 damage and a "until end of turn" +3/+3 effect. Base stats 1/1. After cleanup (514.2), the +3/+3 expires and damage is removed simultaneously. The creature is now 1/1 with 0 damage (fine). BUT: consider instead a creature that is a 2/2 with 3 damage and a "until end of turn" indestructible effect. After cleanup, indestructible expires and damage is removed simultaneously. If the creature also had a separate source of 3 damage from a new effect created during the cleanup priority window... this causes a re-loop.
- **Action (simpler scenario):** A creature's "until end of turn" effect expires, revealing a pending trigger (e.g., "when this creature loses indestructible, sacrifice it"). SBAs detect the trigger.
- **Expected Result:** SBAs are performed, triggers placed on stack, priority granted. After resolution, a new cleanup step begins.
- **Phase:** Phase 5 Pre-Work (T16 — cleanup re-loop)
- **Ticket:** T16 — cleanup re-loop (514.3a)

**ATOM-514.3a-002**
- **Rule:** 514.3a — The re-looped cleanup step performs ANOTHER discard check and damage removal.
- **Mechanism:** Each new cleanup step from the re-loop performs all cleanup TBAs again (514.1 discard + 514.2 damage removal/effect expiry).
- **Minimal Board:** During the first cleanup's priority window, a player draws cards (e.g., from a trigger), ending up with 9 cards in hand.
- **Action:** First cleanup re-loops into second cleanup step.
- **Expected Result:** Second cleanup step performs discard check (player must discard back to 7). Then damage removal occurs again.
- **Phase:** Phase 5 Pre-Work (T16)
- **Ticket:** T16 — cleanup re-loop (each iteration runs full cleanup TBAs)

---

## Composition Tests

These tests require 2+ atomic mechanisms working together.

---

**COMP-ZONE-TRANSITION-001**
- **Rule:** 400.3 + 400.7 — Object owned by Player B, controlled by Player A, destroyed → goes to Player B's graveyard as a new object.
- **Mechanism:** Ownership routing (400.3) + new object identity (400.7) must both work.
- **Composes:** ATOM-400.3-001, ATOM-400.7-001
- **Minimal Board:** Player A controls Player B's creature (via Control Magic). A targeting effect references the creature by ObjectId.
- **Action:** Destroy the creature.
- **Expected Result:** Creature goes to Player B's graveyard (400.3). The targeting effect loses track of the creature (400.7 — new object identity). The returned card in the graveyard has no damage, no counters.
- **Phase:** Phase 1 (zone transitions) + Phase 5 (control change)
- **Ticket:** NEW — cross-rule zone transition test

---

**COMP-COMBAT-FULL-001**
- **Rule:** 508.1 + 509.1 + 510.1 + 510.2 + 511.3 — Full combat sequence from declaration through cleanup.
- **Mechanism:** Declare attackers → declare blockers → damage assignment → simultaneous damage → combat state cleanup.
- **Composes:** ATOM-508.1-001, ATOM-509.1-001, ATOM-510.1-001, ATOM-510.2-001, ATOM-511.3-001
- **Minimal Board:** Player A: 3/3 creature. Player B: 2/2 creature.
- **Action:** Player A attacks with 3/3. Player B blocks with 2/2. Combat damage dealt.
- **Expected Result:** 3/3 takes 2 damage (survives), 2/2 takes 3 damage (dies to SBA). After end of combat, 3/3 is no longer attacking.
- **Phase:** Phase 3 (combat)
- **Ticket:** ALREADY-IMPLEMENTED — covered by integration tests

---

**COMP-FIRST-STRIKE-KILL-001**
- **Rule:** 510.4 + 510.1a + 510.1c — First-strike creature kills blocker before normal damage step.
- **Mechanism:** First-strike two-step damage → blocker dies in SBA between steps → attacker takes no damage.
- **Composes:** ATOM-510.4-001, ATOM-510.1a-001, ATOM-510.1c-001
- **Minimal Board:** A 3/1 first-strike attacker blocked by a 2/2.
- **Action:** First-strike damage step deals 3 to 2/2. SBA destroys 2/2. Normal damage step: 2/2 is gone, no damage dealt to attacker.
- **Expected Result:** Attacker survives with 0 damage. Blocker dies.
- **Phase:** Phase 4 (First Strike)
- **Ticket:** ALREADY-IMPLEMENTED — covered by phase4_integration_test.rs

---

**COMP-CLEANUP-RELOOP-001**
- **Rule:** 514.1 + 514.2 + 514.3a — Cleanup step triggers re-loop.
- **Mechanism:** Discard → damage removal + effect expiry → SBA finds lethal damage on a creature that lost its pump → trigger fires → priority → new cleanup step.
- **Composes:** ATOM-514.1-001, ATOM-514.2-001, ATOM-514.3a-001, ATOM-514.3a-002
- **Minimal Board:** A 1/1 creature with "until end of turn" +2/+2 and 2 damage marked. Active player has 8 cards in hand.
- **Action:** Cleanup step: (1) discard 1 card to reach 7. (2) Remove damage (0 now) and expire +2/+2 simultaneously → creature is 1/1 with 0 damage (survives). No SBAs needed → no re-loop.
- **Action (re-loop variant):** Instead, a 1/1 creature with "until end of turn" indestructible and 3 damage. (1) discard. (2) Remove damage AND expire indestructible simultaneously → creature is 1/1 with 0 damage (survives because damage removal is simultaneous).
- **Action (re-loop triggers):** A permanent has "when a creature you control dies, draw a card." During cleanup, a creature dies from SBA. Trigger goes on stack → priority → resolution → new cleanup step.
- **Expected Result:** New cleanup step occurs, running 514.1 (discard if needed) and 514.2 again.
- **Phase:** Phase 5 Pre-Work (T16) + Phase 7 (triggers during cleanup)
- **Ticket:** T16 — cleanup re-loop

---

**COMP-UNTAP-TRIGGER-UPKEEP-001**
- **Rule:** 502.3 + 502.4 + 503.1a — Untap triggers held until upkeep.
- **Mechanism:** Creature untaps during untap step → trigger fires but is held (no priority in untap) → placed on stack at start of upkeep step before priority.
- **Composes:** ATOM-502.3-001, ATOM-502.4-001, ATOM-503.1a-001
- **Minimal Board:** A tapped permanent with "Whenever this permanent becomes untapped, draw a card."
- **Action:** Untap step untaps it → trigger created but held → upkeep begins → trigger placed on stack → priority.
- **Expected Result:** Draw trigger resolves during upkeep, not during untap step.
- **Phase:** Phase 7 (Triggered Abilities — trigger holding from no-priority steps)
- **Ticket:** NEW — composition: untap trigger held until upkeep

---

**COMP-508.8-SKIP-001**
- **Rule:** 508.8 + 506.1 — No attackers → skip declare blockers and combat damage, straight to end of combat.
- **Mechanism:** Declare attackers finds no attackers → sets skip flag → blockers and damage steps skipped → end of combat runs normally (511.3 clears combat state).
- **Composes:** ATOM-508.8-001, ATOM-506.1-001, ATOM-511.3-001
- **Minimal Board:** Active player has creatures but chooses zero attackers.
- **Action:** Run combat phase.
- **Expected Result:** Beginning of combat → declare attackers (0 declared) → end of combat. No blockers step, no damage step. Combat state cleared.
- **Phase:** Phase 3
- **Ticket:** ALREADY-IMPLEMENTED — integration tests cover this

---

## Classification Summary

### Chapter 4: Zones (400–408)

| Rule | Classification | Notes |
|------|---------------|-------|
| 400.1 | PURE-DEF | Names the seven zones |
| 400.2 | BOUNDARY-DEF | Public vs hidden zones |
| 400.3 | TESTABLE | Owner routing for library/graveyard/hand |
| 400.4 | PURE-DEF | Parent rule |
| 400.4a | TESTABLE | Instant/sorcery can't enter battlefield |
| 400.4b | OUT-OF-SCOPE | Conspiracy/phenomenon/plane/scheme/vanguard |
| 400.5 | TESTABLE | Zone ordering preserved |
| 400.6 | TESTABLE | Zone-change replacement effects |
| 400.7 | TESTABLE | New object identity on zone change |
| 400.7a | TESTABLE | Spell-to-permanent effect continuity |
| 400.7b | TESTABLE | Static ability grants continue |
| 400.7c | TESTABLE | Prevention effects continue |
| 400.7d | TESTABLE | CastInfo on permanent |
| 400.7e | TESTABLE | Zone-change triggers find new object |
| 400.7f | TESTABLE | Aura LTB trigger finds aura in graveyard |
| 400.7g | DEFERRED | Cast-permission continuity (Phase 8+) |
| 400.7h | DEFERRED | Effect-grants-cast finding new object (Phase 8+) |
| 400.7i | DEFERRED | Effect-grants-land-play finding new object (Phase 8+) |
| 400.7j | TESTABLE | Effect moves object to public zone, finds it |
| 400.7k | DEFERRED | Madness (Phase 8+) |
| 400.7m | OUT-OF-SCOPE | Stickers (not in scope) |
| 400.8 | TESTABLE | Re-exile creates new object identity |
| 400.9 | DEFERRED | Face-down command zone (Phase 9) |
| 400.10 | DEFERRED | Command zone re-entry (Phase 9) |
| 400.11 | PURE-DEF | "Outside the game" not a zone |
| 400.11a | PURE-DEF | Sideboard cards |
| 400.11b | DEFERRED | Bring from outside (Phase 8+ Wish) |
| 400.11c | DEFERRED | Affecting outside cards (Phase 8+) |
| 400.12 | PURE-DEF | "Do something to a zone" |
| 401.1 | PURE-DEF | Library at game start |
| 401.2 | TESTABLE | Library face-down |
| 401.3 | TESTABLE | Count any library |
| 401.4 | TESTABLE | Multi-card library placement ordering |
| 401.5 | TESTABLE | Top-of-library reveal during casting |
| 401.6 | DEFERRED | Revealed top card identity (Phase 8+) |
| 401.7 | TESTABLE | "Nth from top" fallback |
| 402.1 | PURE-DEF | Hand definition |
| 402.2 | TESTABLE | Max hand size + cleanup discard |
| 402.3 | PURE-DEF | Hand visibility |
| 403.1 | PURE-DEF | Battlefield starts empty |
| 403.2 | TESTABLE | Default battlefield scope |
| 403.3 | PURE-DEF | Permanents on battlefield |
| 403.4 | TESTABLE | New object on ETB |
| 403.5 | PURE-DEF | Historical note |
| 404.1 | TESTABLE | Graveyard placement |
| 404.2 | TESTABLE | Graveyard face-up, public |
| 404.3 | TESTABLE | Simultaneous graveyard ordering |
| 405.1 | PURE-DEF | Stack intro |
| 405.2 | TESTABLE | Stack LIFO ordering |
| 405.3 | TESTABLE | APNAP simultaneous stack placement |
| 405.4 | TESTABLE | Spell/ability characteristics on stack |
| 405.5 | TESTABLE | Resolution rule (all pass → resolve) |
| 405.6 | PURE-DEF | Parent: things that don't use stack |
| 405.6a | PURE-DEF | Effects don't go on stack |
| 405.6b | PURE-DEF | Static abilities don't use stack |
| 405.6c | TESTABLE | Mana abilities resolve immediately |
| 405.6d | PURE-DEF | Special actions don't use stack |
| 405.6e | TESTABLE | TBAs don't use stack |
| 405.6f | TESTABLE | SBAs don't use stack |
| 405.6g | PURE-DEF | Concession is immediate |
| 405.6h | DEFERRED | Multiplayer leaves-game (Phase 9) |
| 406.1 | PURE-DEF | Exile definition |
| 406.2 | PURE-DEF | "To exile" definition |
| 406.3 | TESTABLE | Exile face-up default + face-down |
| 406.3a | DEFERRED | Face-down exile characteristics (Phase 8+) |
| 406.3b | DEFERRED | Casting from face-down exile (Phase 8+) |
| 406.4 | DEFERRED | Face-down exile piles (Phase 8+) |
| 406.5 | PURE-DEF | Exile pile organization |
| 406.6 | PURE-DEF | Linked abilities reference |
| 406.7 | TESTABLE | Re-exile identity (duplicate of 400.8) |
| 406.8 | PURE-DEF | Historical note |
| 407.1–407.4 | OUT-OF-SCOPE | Ante |
| 408.1 | PURE-DEF | Command zone definition |
| 408.2 | TESTABLE | Emblems in command zone |
| 408.3 | DEFERRED | Format-specific command zone (Phase 9) |

### Chapter 5: Turn Structure (500–514)

| Rule | Classification | Notes |
|------|---------------|-------|
| 500.1 | TESTABLE | Five phases in order |
| 500.2 | TESTABLE | Phase/step end condition |
| 500.3 | TESTABLE | No-priority steps end after TBAs |
| 500.4 | TESTABLE | "Until" duration expiry at step/phase begin |
| 500.5 | TESTABLE | End-of-step effect expiry + mana emptying |
| 500.5a | TESTABLE | "Until end of combat" timing |
| 500.5b | PURE-DEF | "Until end of turn" reference |
| 500.6 | TESTABLE | "At the beginning of" triggers |
| 500.7 | DEFERRED | Extra turns (Phase 9) |
| 500.8 | DEFERRED | Extra phases (Phase 9) |
| 500.9 | DEFERRED | Extra steps (Phase 9) |
| 500.10 | DEFERRED | Adding step after phase (Phase 9) |
| 500.10a | DEFERRED | "You get" step (Phase 9) |
| 500.11 | DEFERRED | Skip step/phase/turn (Phase 9) |
| 500.12 | PURE-DEF | No events between steps |
| 501.1 | PURE-DEF | Beginning phase structure |
| 502.1 | DEFERRED | Phasing (Phase 9) |
| 502.2 | DEFERRED | Day/Night (Phase 9) |
| 502.2a | DEFERRED | Multiplayer day/night (Phase 9) |
| 502.3 | TESTABLE | Untap all permanents |
| 502.4 | TESTABLE | No priority during untap |
| 503.1 | TESTABLE | Upkeep: no TBA, priority |
| 503.1a | TESTABLE | Untap + upkeep triggers on stack |
| 503.2 | DEFERRED | Multiple upkeep steps (Phase 9) |
| 504.1 | TESTABLE | Draw step TBA |
| 504.2 | PURE-DEF | Priority after draw TBA |
| 505.1 | TESTABLE | Two main phases |
| 505.1a | TESTABLE | Precombat vs postcombat identity |
| 505.1b | PURE-DEF | "First/second main phase" text |
| 505.2 | PURE-DEF | Main phase end condition |
| 505.3 | OUT-OF-SCOPE | Archenemy scheme (not in scope) |
| 505.4 | DEFERRED | Saga lore counter (Phase 8) |
| 505.5 | OUT-OF-SCOPE | Attractions (not in scope) |
| 505.6 | PURE-DEF | Priority during main phase |
| 505.6a | TESTABLE | Sorcery-speed timing |
| 505.6b | TESTABLE | Land play timing |
| 506.1 | TESTABLE | Combat phase five steps + skip rule |
| 506.2 | TESTABLE | Attacking/defending player roles |
| 506.2a | DEFERRED | Multiplayer defending (Phase 9) |
| 506.2b | OUT-OF-SCOPE | Shared team turns (not in scope) |
| 506.3 | BOUNDARY-DEF | Only creatures attack/block |
| 506.3a | TESTABLE | Noncreature ETB-attacking |
| 506.3b | TESTABLE | Non-attacking-player ETB-attacking |
| 506.3c | TESTABLE | Invalid target ETB-attacking |
| 506.3d | TESTABLE | Late ETB-attacking enters unblocked |
| 506.3e | TESTABLE | ETB-blocking controller mismatch |
| 506.3f | DEFERRED | Battle + creature (Phase 8+) |
| 506.3g | DEFERRED | Battle becomes creature (Phase 8+) |
| 506.4 | TESTABLE | Removal from combat conditions |
| 506.4a | TESTABLE | Post-declaration restrictions don't remove |
| 506.4b | TESTABLE | Tap/untap doesn't remove from combat |
| 506.4c | TESTABLE | Orphaned attacker (PW removed) |
| 506.4d | DEFERRED | Blocking creature + attacked PW (Phase 8+) |
| 506.4e | DEFERRED | Attacked PW + battle (Phase 8+) |
| 506.5 | PURE-DEF | "Attacks alone" / "blocking alone" |
| 506.6 | TESTABLE | "Had to attack" check |
| 506.7 | PURE-DEF | Combat timing restrictions parent |
| 506.7a–g | TESTABLE | Deferred to Session 5 (casting pipeline) |
| 507.1 | DEFERRED | Multiplayer defending player TBA (Phase 9) |
| 507.2 | PURE-DEF | Priority after beginning of combat |
| 508.1 | TESTABLE | Declare attackers TBA |
| 508.1a | TESTABLE | Untapped + summoning sickness checks |
| 508.1b | TESTABLE | PW/battle attack target selection |
| 508.1c | TESTABLE | Attack restrictions (Defender, etc.) |
| 508.1d | TESTABLE | Requirement maximization |
| 508.1e | OUT-OF-SCOPE | Banding (not in scope) |
| 508.1f | TESTABLE | Attackers tapped (not a cost) |
| 508.1g | TESTABLE | Optional attack costs |
| 508.1h | TESTABLE | Attack cost lock-in |
| 508.1i | TESTABLE | Mana ability window (cross-cutting concern, see audit note) |
| 508.1j | PURE-DEF | Pay all costs |
| 508.1k | TESTABLE | Creatures become attacking |
| 508.1m | TESTABLE | Declare-attacker triggers |
| 508.2 | PURE-DEF | Priority after declaration |
| 508.2a | TESTABLE | Attack trigger characteristic snapshot |
| 508.2b | PURE-DEF | Trigger APNAP placement |
| 508.3 | PURE-DEF | Trigger conditions parent |
| 508.3a–f | TESTABLE | Deferred to Phase 7 (trigger conditions) |
| 508.4 | TESTABLE | ETB-attacking never "attacked" |
| 508.4a–d | TESTABLE | Covered by 506.3a–d tests |
| 508.5 | PURE-DEF | "Defending player" definition |
| 508.5a | DEFERRED | Multiplayer disambiguation (Phase 9) |
| 508.6 | PURE-DEF | "[Player] is attacking [player]" |
| 508.7 | DEFERRED | Attack target reselection (Phase 8+) |
| 508.7a–e | DEFERRED | Reselection details (Phase 8+) |
| 508.8 | TESTABLE | Skip blockers/damage if no attackers |
| 509.1 | TESTABLE | Declare blockers TBA |
| 509.1a | TESTABLE | Blocker untapped + assignment |
| 509.1b | TESTABLE | Blocking restrictions (evasion) |
| 509.1c | TESTABLE | Blocking requirement maximization |
| 509.1d | TESTABLE | Blocking cost lock-in |
| 509.1e | PURE-DEF | Mana ability window |
| 509.1f | PURE-DEF | Pay all costs |
| 509.1g | TESTABLE | Creatures become blocking |
| 509.1h | TESTABLE | Blocked/unblocked status |
| 509.1i | PURE-DEF | Block triggers fire point |
| 509.2 | PURE-DEF | Priority after block declaration |
| 509.2a | PURE-DEF | Block triggers APNAP |
| 509.3 | PURE-DEF | Block trigger conditions parent |
| 509.3a–g | TESTABLE | Deferred to Phase 7 (trigger conditions) |
| 509.4 | TESTABLE | ETB-blocking never "blocked" |
| 509.4a | TESTABLE | ETB-blocking invalid target |
| 509.4b | TESTABLE | ETB-blocking ignores evasion/restrictions (low priority) |
| 510.1 | TESTABLE | Damage assignment TBA |
| 510.1a | TESTABLE | Power = damage; 0 power = no damage |
| 510.1b | TESTABLE | Unblocked → target; orphaned → nothing |
| 510.1c | TESTABLE | Blocked creature damage division (2025) |
| 510.1d | TESTABLE | Blocking creature damage division |
| 510.1e | TESTABLE | Total assignment legality check |
| 510.2 | TESTABLE | Simultaneous damage dealing |
| 510.3 | TESTABLE | Priority after damage (SBA check first) |
| 510.3a | TESTABLE | Damage triggers (Phase 7) |
| 510.4 | TESTABLE | First strike / double strike two steps |
| 511.1 | PURE-DEF | End of combat: no TBA, priority |
| 511.2 | TESTABLE | End-of-combat triggers + duration expiry |
| 511.3 | TESTABLE | Remove all from combat |
| 512.1 | PURE-DEF | Ending phase structure |
| 513.1 | PURE-DEF | End step: no TBA, priority |
| 513.1a | PURE-DEF | Historical errata note |
| 513.2 | TESTABLE | End-step trigger "no back up" |
| 514.1 | TESTABLE | Cleanup discard to hand size |
| 514.2 | TESTABLE | Damage removal + effect expiry |
| 514.3 | TESTABLE | No priority during cleanup (default) |
| 514.3a | TESTABLE | Cleanup re-loop |

---

## Gap Report

### Mechanisms in roadmap/implementation-plan with tests in this session

| Ticket | Mechanism | Session 4 Tests |
|--------|-----------|-----------------|
| T16 | Cleanup re-loop (514.3a) | ATOM-514.3a-001, ATOM-514.3a-002 |
| T21a | Zone guards + CastInfo | ATOM-400.4a-001, ATOM-400.7d-001 |
| T21b | Combat removal on control/type change | ATOM-506.4-001/002/003, ATOM-509.1b-002 |
| T21d | Combat requirements solver | ATOM-508.1c-002, ATOM-508.1d-001/002, ATOM-509.1c-001/002, ATOM-506.6-001 |
| T22 | Duration + turn structure fixes | ATOM-500.4-001, ATOM-500.5-001, ATOM-500.5a-001, ATOM-505.6b-004, ATOM-511.2-002, ATOM-513.2-003, ATOM-514.2-001 |

### Mechanisms in roadmap NOT covered by CR Chapter 4/5

These are referenced in `implementation-plan-final.md` or `roadmap.md` but don't have corresponding CR rules in Chapters 4–5. They belong to other sessions.

- **T01–T08:** Engine type system refactors — no CR rules (internal architecture)
- **T09:** `controller_since_turn` field — implements 302.6 (summoning sickness definition, Session for Ch. 3)
- **T10–T14:** State-based action refactors — CR Chapter 7 (SBA rules 704.x)
- **T15:** Aura SBA (CR 704.5m) — Session for Ch. 7
- **T17–T20:** Targeting, linked abilities — CR Chapter 6 (spells/abilities)
- **L01–L21:** Layer system — CR Chapter 6 (rule 613.x, Session 6)

### CR rules in Chapters 4–5 with no matching implementation ticket

These are NEW mechanisms discovered through this analysis:

1. **Zone visibility query (400.2)** — No ticket. Needs a public/hidden zone predicate.
2. **Replacement effect on zone changes (400.6)** — Phase 6 scope, no specific ticket yet.
3. **Object identity break verification (400.7)** — Implicit in `move_object` but no explicit test that effect references break.
4. **Stack-to-permanent effect continuity (400.7a)** — No ticket. Phase 5 scope.
5. **Prevention effect continuity (400.7c)** — No ticket. Phase 6 scope.
6. **Zone-change trigger object tracking (400.7e, 400.7f)** — No ticket. Phase 7 scope.
7. **Simultaneous graveyard ordering (404.3)** — No ticket. Needs batch zone-move.
8. **APNAP trigger ordering (405.3)** — No ticket. Phase 7 scope.
9. **"Doesn't untap" continuous effect (502.3)** — No ticket. Phase 5 scope.
10. **Trigger hold-and-release from no-priority steps (502.4)** — No ticket. Phase 7 scope.
11. **Step/phase beginning trigger checking (500.6)** — No ticket. Phase 7 infrastructure.
12. **End-step "no back up" rule (513.2)** — No ticket. Phase 7 scope.
13. **ETB-attacking/blocking effects (506.3a–d, 508.4, 509.4)** — No tickets. Phase 8 scope.
14. **Attack/blocking cost framework (508.1g–h, 509.1d)** — No tickets. Phase 8 scope.
15. **Multi-block damage division (510.1d)** — No ticket. Phase 8 scope.
16. **Orphaned attacker (506.4c, 510.1b)** — No ticket. Phase 8 (PW combat).
17. **Constrained untap choice (502.3)** — No ticket. Winter Orb / Stasis pattern. Phase 5 + Phase 8.
18. **Multiplayer blocker controller validation (509.1a)** — No ticket. Phase 9 (Commander combat).
19. **Phasing removes from combat (506.4)** — No ticket. Phase 9 (phasing, D1).
20. **ETB-blocking evasion bypass (509.4b)** — No ticket. Phase 8. Low priority (no current card).

### Session 4 Statistics

- **Total sub-rules analyzed:** ~190
- **ATOM tests generated:** 159
- **COMP tests generated:** 6
- **Total tests:** 165
- **Classifications:**
  - TESTABLE: 100
  - BOUNDARY-DEF: 2
  - PURE-DEF: 46
  - DEFERRED: 37 (Phase 8+ and Phase 9 mechanics)
  - OUT-OF-SCOPE: 7 (400.4b supplemental types, 400.7m stickers, 407.x ante, 505.3 Archenemy, 505.5 Attractions, 506.2b shared team turns, 508.1e banding)
  - Deferred to other sessions: ~20 (506.7a–g, 508.3a–f, 509.3a–g)
- **ALREADY-IMPLEMENTED:** 42 rules (covering ~48 ATOM tests)
- **NEW tickets identified:** 20

---

*End of Session 4.*
