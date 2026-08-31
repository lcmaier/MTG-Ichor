# Session 8: Keyword Abilities — 702.81–702.190

**CR Source:** `MTG-Rules/LLM-Chapter-Splits/ch7-pt-3.txt`
**Scope:** 110 keywords (Retrace through Sneak)
**Prior session:** Session 7B covered 702.1–702.80

---

## Chunk Plan

| Chunk | Rules | Keywords (count) |
|-------|-------|-----------------|
| 1 | 702.81–702.95 | Retrace, Devour, Exalted, Unearth, Cascade, Annihilator, Level Up, Rebound, Umbra Armor, Infect, Battle Cry, Living Weapon, Undying, Miracle, Soulbond (15) |
| 2 | 702.96–702.110 | Overload, Scavenge, Unleash, Cipher, Evolve, Extort, Fuse, Bestow, Tribute, Dethrone, Hidden Agenda, Outlast, Prowess, Dash, Exploit (15) |
| 3 | 702.111–702.126 | Menace, Renown, Awaken, Devoid, Ingest, Myriad, Surge, Skulk, Emerge, Escalate, Melee, Crew, Fabricate, Partner, Undaunted, Improvise (16) |
| 4 | 702.127–702.140 | Aftermath, Embalm, Eternalize, Afflict, Ascend, Assist, Jump-Start, Mentor, Afterlife, Riot, Spectacle, Escape, Companion, Mutate (14) |
| 5 | 702.141–702.155 | Encore, Boast, Foretell, Demonstrate, Daybound/Nightbound, Disturb, Decayed, Cleave, Training, Compleated, Reconfigure, Blitz, Casualty, Enlist, Read Ahead (15) |
| 6 | 702.156–702.170 | Ravenous, Squad, Space Sculptor, Visit, Prototype, Living Metal, More Than Meets the Eye, For Mirrodin!, Toxic, Backup, Bargain, Craft, Disguise, Solved, Plot (15) |
| 7 | 702.171–702.190 | Saddle, Spree, Freerunning, Gift, Offspring, Impending, Exhaust, Max Speed, Start Your Engines!, Harmonize, Mobilize, Job Select, Tiered, Station, Warp, ∞, Mayhem, Web-slinging, Firebending, Sneak (20) |
| Final | — | Classification Summary Table, Composition Tests, Gap Report |

---

## Chunk 1: 702.81–702.95 (Retrace through Soulbond)

### 702.81 Retrace

**Classification: DEFERRED — Phase 8 (alternative cost, zone-cast from graveyard)**

702.81a — Retrace is a static ability allowing cast from graveyard with "discard a land" additional cost. Requires T17 alt/additional cost framework + zone-casting (T19 activation_zone pattern extended to spells).

**ATOM-702.81a-001**
- **Rule:** 702.81a — Retrace allows casting from graveyard by discarding a land as additional cost
- **Mechanism:** Zone-casting (graveyard) + additional cost (discard land)
- **Minimal Board:** Player has a sorcery with retrace in graveyard, a land card in hand, sufficient mana
- **Action:** Player casts the retrace spell from graveyard, discarding a land
- **Expected Result:** Spell is cast from graveyard, land is discarded, spell resolves normally. Spell goes to graveyard again after resolution (can be retraced again).
- **Phase:** Phase 8
- **Ticket:** NEW — Retrace keyword (zone-cast from GY + discard-land additional cost)
- **Dependencies:** T17 (additional cost framework), T19 (zone-activated abilities pattern)

---

### 702.82 Devour

**Classification: DEFERRED — Phase 6 (replacement effect on ETB)**

702.82a — "As this object enters, you may sacrifice any number of creatures." This is a replacement effect modifying how the permanent enters.

**ATOM-702.82a-001**
- **Rule:** 702.82a — Devour N: sacrifice creatures as permanent enters, enters with N × sacrificed +1/+1 counters
- **Mechanism:** ETB replacement effect (sacrifice choice) + counter placement
- **Minimal Board:** Player casts a creature with Devour 2, controls two other creatures
- **Action:** Creature with Devour enters; player chooses to sacrifice both creatures
- **Expected Result:** Devour creature enters with 4 (2×2) +1/+1 counters. Sacrificed creatures go to graveyard.
- **Phase:** Phase 6 (replacement effects) + Phase 8 (keyword)
- **Ticket:** NEW — Devour keyword (ETB replacement, sacrifice N creatures, add N×devour counters)
- **Dependencies:** Phase 6 (replacement effects), T01 (counters)

702.82b — "It devoured" refers to creatures sacrificed via devour. TESTABLE (linked ability reference).

**ATOM-702.82b-001**
- **Rule:** 702.82b — "It devoured" = creatures sacrificed via its devour ability
- **Mechanism:** Linked ability referencing devour count
- **Minimal Board:** Player casts a creature with "Devour 2" and "When this creature enters, draw a card for each creature it devoured." Player sacrifices 3 creatures.
- **Action:** Creature enters with 6 +1/+1 counters (3 × 2). ETB trigger references devour count.
- **Expected Result:** Player draws 3 cards (number of creatures devoured, not counters).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.82a)
- **Dependencies:** Phase 7 (triggered), Devour count tracking (linked ability)

**Tag: LINKED-ABILITY-PATTERN** — Devour count, Exploit target, Tribute paid/not-paid all share a pattern where a replacement/ETB mechanic stores a value that a linked triggered ability references. Engine needs a "linked ability context" for this.

702.82c — Devour [quality] variant. Same mechanism, different sacrifice target. Covered by 702.82a test structure.

---

### 702.83 Exalted

**Classification: DEFERRED — Phase 7 (triggered ability)**

702.83a — "Whenever a creature you control attacks alone, that creature gets +1/+1 until end of turn."

**ATOM-702.83a-001**
- **Rule:** 702.83a — Exalted triggers when a creature you control attacks alone, giving +1/+1 until EOT
- **Mechanism:** Triggered ability on attack-alone condition + P/T modification
- **Minimal Board:** Player controls a 2/2 creature and a permanent with exalted. Only the 2/2 attacks.
- **Action:** Declare the 2/2 as the sole attacker
- **Expected Result:** Exalted triggers, resolves, creature becomes 3/3 until end of turn
- **Phase:** Phase 7 (triggered abilities) + Phase 8
- **Ticket:** NEW — Exalted keyword (triggered ability on solo attack)
- **Dependencies:** Phase 7 (triggered abilities), Phase 5 (continuous effect for +1/+1 until EOT)

702.83b — Defines "attacks alone" = only creature declared as attacker. PURE-DEF (prerequisite for 702.83a).

**ATOM-702.83a-002**
- **Rule:** 702.83a + 506.5 — Exalted triggers even when tokens enter attacking
- **Mechanism:** "Attacks alone" checks declaration, not total attacking creatures
- **Minimal Board:** Player controls creature A with exalted and creature B with "Whenever this creature attacks, create a 1/1 Warrior token tapped and attacking." Only B is declared as attacking.
- **Action:** Declare attackers with only B. B's trigger creates a Warrior token tapped and attacking.
- **Expected Result:** Exalted triggers (B attacked alone — it was the only creature *declared* as attacking). B gets +1/+1. The Warrior token does NOT prevent exalted.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.83a)
- **Dependencies:** Phase 7 (triggered), "declared as attacking" vs "enters attacking" distinction

**Tag: DECLARED-VS-ENTERS-ATTACKING** — Affects Exalted, Melee (702.121), and any "attacks alone" / "whenever a creature attacks" trigger. Engine must track how a creature became attacking (declared vs entered).

---

### 702.84 Unearth

**Classification: DEFERRED — Phase 8 (activated ability from graveyard + delayed triggers)**

702.84a — Activated ability from graveyard. Returns card to battlefield with haste, exile at next end step, exile if it would leave battlefield.

**ATOM-702.84a-001**
- **Rule:** 702.84a — Unearth returns creature from graveyard to battlefield with haste; exile at next end step; exile instead of leaving battlefield
- **Mechanism:** Zone-activated ability (graveyard), haste grant, delayed trigger (exile at end step), replacement effect (exile instead of other zone change)
- **Minimal Board:** Player has a creature with "Unearth {B}" in graveyard, {B} available
- **Action:** Activate unearth ability
- **Expected Result:** Creature returns to battlefield with haste. At beginning of next end step, it is exiled. If it would die before that, it is exiled instead.
- **Phase:** Phase 8
- **Ticket:** NEW — Unearth keyword (GY activated ability, haste, delayed exile trigger, replacement exile)
- **Dependencies:** T19 (activation_zone: Graveyard), Phase 7 (delayed triggers), Phase 6 (replacement effect)

**ATOM-702.84a-002**
- **Rule:** 702.84a — Unearth: "If it would leave the battlefield, exile it instead"
- **Mechanism:** Replacement effect on zone change (any non-exile destination → exile)
- **Minimal Board:** Player unearthed a creature from graveyard. Opponent casts "Return target creature to its owner's hand."
- **Action:** Bounce spell resolves targeting the unearthed creature
- **Expected Result:** Creature is exiled instead of returned to hand. The replacement applies to ANY zone change off battlefield (bounce, destroy, shuffle into library, etc.).
- **Phase:** Phase 6 (replacement) + Phase 8
- **Ticket:** (same as 702.84a)
- **Dependencies:** Phase 6 (replacement effects), Unearth zone-change replacement

---

### 702.85 Cascade

**Classification: DEFERRED — Phase 7 (triggered ability) + Phase 8. PRIORITY keyword.**

702.85a — "When you cast this spell, exile cards from top of library until you exile a nonland card whose mana value is less than this spell's mana value. You may cast that card without paying its mana cost..."

**ATOM-702.85a-001**
- **Rule:** 702.85a — Cascade exiles from library top until finding a cheaper nonland card; may cast it free; rest go to bottom in random order
- **Mechanism:** Triggered ability on cast + exile-from-library loop + free cast + zone management
- **Minimal Board:** Player casts a spell with cascade (mana value 4). Library has: Land, Land, 2-MV sorcery, 5-MV creature (top to bottom)
- **Action:** Cascade trigger resolves
- **Expected Result:** Exile Land, Land, then the 2-MV sorcery (nonland, MV < 4). Cascade skips cards with equal or greater MV. Player may cast the sorcery without paying its mana cost. If cast, the two lands go to bottom in random order. If not cast, all three exiled cards go to bottom in random order.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Cascade keyword (triggered on cast, exile loop, free cast, bottom-of-library)
- **Dependencies:** Phase 7 (triggered abilities), T17 (alternative cost: "without paying mana cost")

**ATOM-702.85a-002**
- **Rule:** 702.85a — Cascade checks resulting spell's mana value, not just exiled card's
- **Mechanism:** MV check applies to the resulting spell (relevant for split cards)
- **Minimal Board:** Player casts cascade spell (MV 4). Exiled card is a split card where one half has MV 2 and the other MV 5
- **Action:** Cascade finds the split card
- **Expected Result:** If casting the half with MV 2, it is legal (MV < 4). If the combined MV of a fused spell would be ≥ 4, that cast is illegal via cascade.
- **Phase:** Phase 9 (split cards)
- **Ticket:** NEW — Cascade + split card MV interaction

**ATOM-702.85a-003**
- **Rule:** 702.85a — Cascade: player chooses not to cast the found spell
- **Mechanism:** Optional cast from exile; decline → all exiled cards go to bottom in random order
- **Minimal Board:** Player casts a spell with cascade (MV 4). Cascade exiles 3 cards, finds a nonland with MV 2.
- **Action:** Player declines to cast the found spell
- **Expected Result:** All exiled cards (including the found spell) are put on the bottom of library in a random order. No spell is cast.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.85a)

702.85b — "As you cascade" action timing. DEFERRED — Phase 8 (niche timing window).

702.85c — Multiple instances trigger separately.

**ATOM-702.85c-001**
- **Rule:** 702.85c — Multiple cascade instances each trigger separately
- **Mechanism:** Multiple triggered ability instances
- **Minimal Board:** Player casts a spell with two instances of cascade
- **Action:** Spell is cast
- **Expected Result:** Two separate cascade triggers go on the stack. Each resolves independently.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Cascade multiple instances

---

### 702.86 Annihilator

**Classification: DEFERRED — Phase 7 (triggered ability)**

702.86a — "Whenever this creature attacks, defending player sacrifices N permanents."

**ATOM-702.86a-001**
- **Rule:** 702.86a — Annihilator N triggers on attack, forcing defending player to sacrifice N permanents
- **Mechanism:** Triggered ability on attack + forced sacrifice
- **Minimal Board:** Player controls a creature with Annihilator 2. Opponent controls 3 permanents.
- **Action:** Creature attacks
- **Expected Result:** Trigger goes on stack. On resolution, defending player sacrifices 2 permanents of their choice.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Annihilator keyword (attack trigger, forced sacrifice)
- **Dependencies:** Phase 7 (triggered abilities), Sacrifice primitive in resolve.rs

702.86b — Multiple instances trigger separately. DEFERRED — Phase 7.

---

### 702.87 Level Up

**Classification: DEFERRED — Phase 8 (activated ability + leveler card layout)**

702.87a — "Level up [cost]" = "[Cost]: Put a level counter on this permanent. Activate only as a sorcery."

**ATOM-702.87a-001**
- **Rule:** 702.87a — Level up puts a level counter on the permanent, sorcery-speed only
- **Mechanism:** Activated ability with sorcery-speed restriction + counter placement
- **Minimal Board:** Player controls a leveler creature with "Level up {2}", has {2} available, main phase
- **Action:** Activate level up ability
- **Expected Result:** A level counter is placed on the creature. Activation is rejected if not at sorcery speed.
- **Phase:** Phase 8
- **Ticket:** NEW — Level Up keyword (sorcery-speed activated, level counters)
- **Dependencies:** T01 (counters), T19 (ActivationRestriction::SorcerySpeed)

702.87b — Leveler card layout / level symbols. PURE-DEF.

702.87c — Class cards vs level up. PURE-DEF (disambiguation).

**Tag: CROSS-REF rule 711** — Level Up (702.87) uses the leveler card layout defined in rule 711. Level symbols define ability bands based on level counter count. Session 9B should cover rule 711 and generate tests for the level-symbol-to-ability mapping. Session-8 test for 702.87a covers the counter-placing activation; rule 711 tests will cover the "which abilities are active at which level" logic.

---

### 702.88 Rebound

**Classification: DEFERRED — Phase 6 (replacement) + Phase 7 (delayed trigger)**

702.88a — "If this spell was cast from your hand, instead of putting it into your graveyard as it resolves, exile it and, at the beginning of your next upkeep, you may cast this card from exile without paying its mana cost."

**ATOM-702.88a-001**
- **Rule:** 702.88a — Rebound exiles spell instead of going to GY on resolution; casts free from exile at next upkeep
- **Mechanism:** Replacement effect (exile instead of GY) + delayed triggered ability
- **Minimal Board:** Player casts an instant with rebound from hand
- **Action:** Spell resolves
- **Expected Result:** Spell is exiled instead of going to graveyard. At beginning of player's next upkeep, they may cast it from exile without paying its mana cost.
- **Phase:** Phase 6 + Phase 7 + Phase 8
- **Ticket:** NEW — Rebound keyword (replacement exile + delayed free cast)

**ATOM-702.88a-002**
- **Rule:** 702.88a — Rebound only works if cast from hand
- **Mechanism:** Zone-check condition on replacement
- **Minimal Board:** Player casts a rebound spell from graveyard (via flashback)
- **Action:** Spell resolves
- **Expected Result:** Spell goes to graveyard normally (rebound does not apply)
- **Phase:** Phase 8
- **Ticket:** (same as above)

702.88b — Casting via rebound follows alt cost rules. PURE-DEF.

702.88c — Multiple instances redundant. PURE-DEF.

---

### 702.89 Umbra Armor

**Classification: DEFERRED — Phase 6 (replacement effect)**

702.89a — "If enchanted permanent would be destroyed, instead remove all damage marked on it and destroy this Aura."

**ATOM-702.89a-001**
- **Rule:** 702.89a — Umbra armor replaces destruction of enchanted permanent
- **Mechanism:** Replacement effect on destruction
- **Minimal Board:** A creature enchanted by an Aura with umbra armor. A Destroy effect targets the creature.
- **Action:** Destroy effect resolves
- **Expected Result:** Creature is NOT destroyed. All damage is removed. The Aura is destroyed instead.
- **Phase:** Phase 6 + Phase 8
- **Ticket:** NEW — Umbra Armor keyword (replacement: prevent destroy, remove damage, destroy Aura)
- **Dependencies:** Phase 6 (replacement effects), T04 (attachment tracking)

**ATOM-702.89a-002**
- **Rule:** 702.89a — Umbra Armor prevents destruction from lethal damage
- **Mechanism:** Replacement effect intercepts SBA destruction from lethal damage
- **Minimal Board:** 3/3 creature enchanted by Aura with umbra armor. Creature takes 3 damage.
- **Action:** SBAs check — creature has lethal damage, would be destroyed
- **Expected Result:** Instead, all damage is removed from the creature and the Aura with umbra armor is destroyed. Creature survives.
- **Phase:** Phase 6 (replacement) + Phase 8
- **Ticket:** (same as 702.89a)

702.89b — Old "totem armor" renamed. PURE-DEF.

---

### 702.90 Infect

702.90a — PURE-DEF (static ability).

702.90b — ALREADY-IMPLEMENTED (T21c: infect to player → poison).

702.90c — ALREADY-IMPLEMENTED (T21c: infect to creature → -1/-1 counters).

702.90d — DEFERRED — Phase 5 Layers (LKI system).

**ATOM-702.90d-001**
- **Rule:** 702.90d — If source changes zones before dealing damage, LKI determines if it had infect
- **Mechanism:** Last Known Information for damage source
- **Minimal Board:** A creature with infect leaves the battlefield, then a delayed effect causes it to deal damage
- **Action:** Delayed damage effect resolves
- **Expected Result:** LKI shows creature had infect; damage dealt as poison/counters
- **Phase:** Phase 5 Layers (LKI — L18/T20b)
- **Ticket:** L18 (LKI system)

702.90e — Infect functions from any zone.

**ATOM-702.90e-001**
- **Rule:** 702.90e — Infect functions regardless of zone
- **Mechanism:** Zone-independent keyword behavior
- **Minimal Board:** A source with infect deals damage from a non-battlefield zone
- **Action:** Damage is dealt
- **Expected Result:** Damage results in poison/counters, not life loss/damage marked
- **Phase:** Phase 8
- **Ticket:** T21c (verify any-zone behavior)

702.90f — Multiple instances redundant. PURE-DEF.

**Tag: SHARED-BEHAVIOR wither** — Infect (702.90) = Wither (702.80, covered in Session 7B) + "damage to players = poison counters." For the dependency graph, infect's creature-damage behavior should reuse the wither implementation (deal damage as -1/-1 counters). The poison-counter-to-player logic is the only new piece.

---

### 702.91 Battle Cry

**Classification: DEFERRED — Phase 7 (triggered ability)**

**ATOM-702.91a-001**
- **Rule:** 702.91a — Battle cry triggers on attack, gives other attackers +1/+0 until EOT
- **Mechanism:** Triggered ability on attack + P/T modification
- **Minimal Board:** Creature A (2/2, battle cry) and creature B (3/3). Both attack.
- **Action:** Both declared as attackers
- **Expected Result:** Battle cry triggers. Creature B becomes 4/3. Creature A does NOT get the bonus.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Battle Cry keyword
- **Dependencies:** Phase 7 (triggered abilities), Phase 5 (continuous effect until EOT)

702.91b — Multiple instances trigger separately. DEFERRED — Phase 7.

---

### 702.92 Living Weapon

**Classification: DEFERRED — Phase 7 (triggered ability) + Phase 8**

**ATOM-702.92a-001**
- **Rule:** 702.92a — Living weapon creates a 0/0 Germ token on ETB and attaches Equipment to it
- **Mechanism:** ETB triggered ability + token creation + equipment attachment
- **Minimal Board:** Player casts an Equipment with living weapon
- **Action:** Equipment enters the battlefield
- **Expected Result:** A 0/0 black Phyrexian Germ creature token is created, Equipment attached to it.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Living Weapon keyword (ETB trigger, token, auto-attach)
- **Dependencies:** Phase 7, CreateToken primitive, T04 (attachment tracking)

---

### 702.93 Undying

**Classification: DEFERRED — Phase 7 (triggered ability)**

**ATOM-702.93a-001**
- **Rule:** 702.93a — Undying returns creature from GY to battlefield with a +1/+1 counter if it had none when it died
- **Mechanism:** Dies trigger + intervening-if condition + return to battlefield + counter placement
- **Minimal Board:** A 2/2 creature with undying, no +1/+1 counters. Destroy effect targets it.
- **Action:** Creature is destroyed
- **Expected Result:** Undying triggers. Creature returns with one +1/+1 counter (3/3). Second death: does NOT trigger (has counter).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Undying keyword (dies trigger with counter check)
- **Dependencies:** Phase 7 (triggered abilities, intervening-if), T01 (counters)

**Tag: SHARED-BEHAVIOR persist** — Undying (702.93) and Persist (702.79, Session 7B) are structural duals: Undying checks no +1/+1 counters → returns with +1/+1. Persist checks no -1/-1 counters → returns with -1/-1. Implementation should share a `dies_return_with_counter(counter_type, absence_check)` helper.

---

### 702.94 Miracle

**Classification: DEFERRED — Phase 7 (linked triggered ability) + Phase 8**

**ATOM-702.94a-001**
- **Rule:** 702.94a — Miracle: reveal on first draw of turn, triggered ability allows casting for miracle cost
- **Mechanism:** Static ability linked to triggered ability + alternative cost
- **Minimal Board:** Player's first draw this turn draws a card with "Miracle {W}"
- **Action:** Player draws the card, reveals it
- **Expected Result:** Miracle triggers. Player may cast for {W}. If not the first draw, miracle cannot be used.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Miracle keyword (first-draw reveal + triggered alt-cost cast)
- **Dependencies:** Phase 7 (linked triggered abilities), D16 (per-turn tracker)

702.94b — Revealed card stays revealed. PURE-DEF (display rule).

---

### 702.95 Soulbond

**Classification: DEFERRED — Phase 7 (triggered ability) + Phase 8**

**ATOM-702.95a-001**
- **Rule:** 702.95a — Soulbond: pair this creature with another unpaired creature on ETB
- **Mechanism:** ETB triggered ability + pairing designation
- **Minimal Board:** Player controls unpaired creature A. Casts creature B with soulbond.
- **Action:** Creature B enters the battlefield
- **Expected Result:** Soulbond triggers. Player may pair B with A.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Soulbond keyword (dual ETB triggers, pairing)
- **Dependencies:** Phase 7 (triggered abilities), D17 (per-permanent designations)

702.95b — PURE-DEF.

**ATOM-702.95c-001**
- **Rule:** 702.95c — Soulbond pairing fails if either is no longer creature/on battlefield/same controller
- **Mechanism:** Resolution validity check
- **Minimal Board:** Creature A with soulbond and creature B. B is destroyed before trigger resolves.
- **Action:** Soulbond trigger resolves
- **Expected Result:** Pairing does not occur
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.95a)

702.95d — PURE-DEF (one pair only).

**ATOM-702.95e-001**
- **Rule:** 702.95e — Paired creature becomes unpaired if partner leaves battlefield
- **Mechanism:** Pairing dissolution on zone change
- **Minimal Board:** A and B are paired. B is destroyed.
- **Action:** B goes to graveyard
- **Expected Result:** A becomes unpaired
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.95a)

**ATOM-702.95a-002**
- **Rule:** 702.95a (second trigger) — Soulbond: another creature enters, unpaired soulbond creature may pair with it
- **Mechanism:** ETB trigger on OTHER creatures entering (soulbond creature's perspective)
- **Minimal Board:** Player controls unpaired creature A with soulbond (no other creatures). Player casts creature B.
- **Action:** Creature B enters; soulbond's second trigger fires on creature A
- **Expected Result:** Controller may pair A and B. If they do, both become paired.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.95a)

**ATOM-702.95e-002**
- **Rule:** 702.95e — Soulbond: creature stops being a creature → unpaired
- **Mechanism:** Pairing dissolution on type loss
- **Minimal Board:** A and B are paired. Effect turns B into a noncreature artifact.
- **Action:** B loses creature type
- **Expected Result:** A and B become unpaired.
- **Phase:** Phase 5 + Phase 7 + Phase 8
- **Ticket:** (same as 702.95a)

**ATOM-702.95e-003**
- **Rule:** 702.95e — Soulbond: creature changes controller → unpaired
- **Mechanism:** Pairing dissolution on controller change
- **Minimal Board:** A and B are paired under player 1. Opponent gains control of B.
- **Action:** B's controller changes to player 2
- **Expected Result:** A and B become unpaired.
- **Phase:** Phase 5 + Phase 7 + Phase 8
- **Ticket:** (same as 702.95a)

--- End of Chunk 1 ---

## Chunk 2: 702.96–702.110 (Overload through Exploit)

### 702.96 Overload

**Classification: DEFERRED — Phase 5 (text-changing effect) + Phase 8. PRIORITY keyword.**

702.96a — Two static abilities: alt cost + text change ("target" → "each"). Text-changing effect per rule 612.

**ATOM-702.96a-001**
- **Rule:** 702.96a — Overload replaces "target" with "each" in spell text when overload cost is paid
- **Mechanism:** Alternative cost + text-changing effect
- **Minimal Board:** Player casts "Destroy target artifact" with overload. Opponent controls 3 artifacts.
- **Action:** Player pays overload cost
- **Expected Result:** Spell text becomes "Destroy each artifact." All 3 artifacts are destroyed. Spell has no targets.
- **Phase:** Phase 5 (text-changing effects, rule 612) + Phase 8
- **Ticket:** NEW — Overload keyword (alt cost + text-changing effect)
- **Dependencies:** T17 (alternative cost), Phase 5 (text-changing effects)

702.96b — Overloaded spell has no targets; may affect objects that couldn't be legal targets.

**ATOM-702.96b-001**
- **Rule:** 702.96b — Overloaded spell won't require targets and can affect untargetable objects
- **Mechanism:** Targetless spell affecting all qualifying objects
- **Minimal Board:** Player casts overloaded "Destroy target artifact." Opponent has an artifact with hexproof.
- **Action:** Overloaded spell resolves
- **Expected Result:** Hexproof artifact IS destroyed (spell has no targets, hexproof irrelevant)
- **Phase:** Phase 5 + Phase 8
- **Ticket:** (same as 702.96a)

702.96c — Overload's second ability is a text-changing effect per rule 612. PURE-DEF (cross-ref).

**Tag: TEXT-CHANGING-EFFECT** — Overload (702.96), Splice (702.47 from Session 7B), and Cleave (702.148) all involve text-changing effects. The engine's text-changing infrastructure must handle all three. Consider a shared `TextModification` layer that these keywords feed into. Cross-ref rule 612 (text-changing effects).

---

### 702.97 Scavenge

**Classification: DEFERRED — Phase 8 (GY activated ability)**

702.97a — "[Cost], Exile this card from your graveyard: Put +1/+1 counters equal to this card's power on target creature. Sorcery speed."

**ATOM-702.97a-001**
- **Rule:** 702.97a — Scavenge exiles card from GY, puts +1/+1 counters equal to its power on target creature
- **Mechanism:** GY activated ability + exile self + counter placement on target
- **Minimal Board:** Player has a 4/4 creature with "Scavenge {3}{G}" in graveyard, controls a 2/2 creature
- **Action:** Activate scavenge targeting the 2/2
- **Expected Result:** Card is exiled from graveyard. 4 +1/+1 counters placed on the 2/2 (becomes 6/6).
- **Phase:** Phase 8
- **Ticket:** NEW — Scavenge keyword (GY activated, exile self, counters on target)
- **Dependencies:** T19 (activation_zone: Graveyard), T01 (counters)

---

### 702.98 Unleash

**Classification: DEFERRED — Phase 6 (ETB replacement) + Phase 5 (static blocking restriction)**

702.98a — Two static abilities: "enter with additional +1/+1 counter" (optional) + "can't block while has +1/+1 counter."

**ATOM-702.98a-001**
- **Rule:** 702.98a — Unleash: may enter with +1/+1 counter; can't block if has +1/+1 counter
- **Mechanism:** ETB replacement (optional counter) + static blocking restriction
- **Minimal Board:** Player casts a 2/2 creature with unleash
- **Action:** Creature enters; player chooses to add the counter
- **Expected Result:** Creature enters as 3/3 (with +1/+1 counter). It cannot be declared as a blocker.
- **Phase:** Phase 5 (continuous blocking restriction) + Phase 6 (ETB replacement) + Phase 8
- **Ticket:** NEW — Unleash keyword (optional ETB counter + blocking restriction)
- **Dependencies:** Phase 6 (replacement), T01 (counters)

**ATOM-702.98a-002**
- **Rule:** 702.98a — Unleash creature without +1/+1 counter CAN block
- **Mechanism:** Blocking restriction conditional on counter presence
- **Minimal Board:** Player casts a 2/2 with unleash, chooses NOT to add counter
- **Action:** Opponent attacks; player declares the unleash creature as blocker
- **Expected Result:** Block is legal (no +1/+1 counter present)
- **Phase:** Phase 5 + Phase 8
- **Ticket:** (same as above)

**ATOM-702.98a-003**
- **Rule:** 702.98a (second static) — Unleash: can't block with +1/+1 counter from ANY source
- **Mechanism:** Static blocking restriction conditional on counter presence
- **Minimal Board:** Player casts creature with unleash, chooses NOT to put the +1/+1 counter on it. Later, another effect puts a +1/+1 counter on it.
- **Action:** Creature now has a +1/+1 counter (from external source). Player tries to block with it.
- **Expected Result:** Block is illegal. The "can't block" restriction checks for ANY +1/+1 counter, not just the unleash one.
- **Phase:** Phase 5 + Phase 8
- **Ticket:** (same as above)

---

### 702.99 Cipher

**Classification: DEFERRED — Phase 8 (exile encoding + combat damage trigger)**

702.99a — Exile card encoded on a creature; creature gains "deal combat damage → copy and cast free."

**ATOM-702.99a-001**
- **Rule:** 702.99a — Cipher encodes spell on creature; creature deals combat damage → copy encoded card and may cast free
- **Mechanism:** Exile zone encoding + triggered ability on combat damage + copy + free cast
- **Minimal Board:** Player casts an instant with cipher. Chooses to encode it on a 2/2 creature.
- **Action:** Creature deals combat damage to opponent
- **Expected Result:** Trigger fires. Player may copy the encoded card and cast the copy without paying its mana cost.
- **Phase:** Phase 8
- **Ticket:** NEW — Cipher keyword (exile encoding, combat damage trigger, copy + free cast)
- **Dependencies:** Phase 7 (triggered abilities), CopySpell primitive

**ATOM-702.99a-003**
- **Rule:** 702.99a — Cipher: a copy of a cipher spell does not get encoded
- **Mechanism:** Copy exclusion from cipher encoding
- **Minimal Board:** Player casts a spell with cipher, it resolves and is encoded on creature A. Later, creature A deals combat damage, creating a copy of the encoded spell.
- **Action:** The copy resolves
- **Expected Result:** The copy is NOT encoded on any creature after resolving. Only the original card (already encoded) remains encoded. Copies cease to exist after resolving.
- **Phase:** Phase 8
- **Ticket:** (same as 702.99a)

702.99b — "Encoded" definition. PURE-DEF.

702.99c — Encoding persists while card is exiled and creature is on battlefield; survives control change and losing creature type.

**ATOM-702.99c-001**
- **Rule:** 702.99c — Encoded card remains encoded even if creature changes controller or stops being a creature
- **Mechanism:** Encoding persistence across state changes
- **Minimal Board:** A cipher card is encoded on creature A. Opponent gains control of A.
- **Action:** A deals combat damage to a player
- **Expected Result:** The encoded ability triggers for A's current controller (the opponent). Encoding persists.
- **Phase:** Phase 8
- **Ticket:** (same as 702.99a)

---

### 702.100 Evolve

**Classification: DEFERRED — Phase 7 (triggered ability)**

702.100a — "Whenever a creature you control enters, if that creature's power is greater than this creature's power and/or toughness greater than this creature's toughness, put a +1/+1 counter on this creature."

**ATOM-702.100a-001**
- **Rule:** 702.100a — Evolve triggers when a creature with greater P or T enters, placing a +1/+1 counter
- **Mechanism:** ETB triggered ability with P/T comparison + counter placement
- **Minimal Board:** Player controls a 1/1 with evolve. Player casts a 2/3.
- **Action:** The 2/3 enters the battlefield
- **Expected Result:** Evolve triggers (2 > 1 power AND 3 > 1 toughness). +1/+1 counter placed on the evolve creature (becomes 2/2).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Evolve keyword (ETB trigger, P/T comparison, +1/+1 counter)
- **Dependencies:** Phase 7 (triggered abilities, intervening-if), T01 (counters)

**ATOM-702.100a-002**
- **Rule:** 702.100a — Evolve triggers on toughness-only being greater
- **Mechanism:** OR condition (power OR toughness)
- **Minimal Board:** Player controls a 2/1 with evolve. A 1/3 enters.
- **Action:** The 1/3 enters
- **Expected Result:** Evolve triggers (1 < 2 power but 3 > 1 toughness — toughness condition met). +1/+1 counter placed.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as above)

702.100b — "Evolves" = counters placed by evolve ability resolving. TESTABLE (linked trigger definition).

**ATOM-702.100b-001**
- **Rule:** 702.100b — "A creature evolves" = its evolve trigger resolves and a counter is placed
- **Mechanism:** Linked trigger definition (evolve event)
- **Minimal Board:** Creature A with evolve and "Whenever this creature evolves, draw a card." A is 1/1. Player casts a 3/3.
- **Action:** Evolve triggers (3/3 has greater power). Counter placed on A. "Evolves" event fires.
- **Expected Result:** Player draws a card (the "whenever evolves" trigger fires because the counter was actually placed).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.100a)
- **Dependencies:** Phase 7 (triggered), LINKED-ABILITY-PATTERN

**ATOM-702.100b-002**
- **Rule:** 702.100b — Evolve trigger resolves but counter NOT placed (entering creature left) → doesn't "evolve"
- **Mechanism:** Evolve re-checks on resolution
- **Minimal Board:** Creature A with evolve and "Whenever evolves, draw a card." A is 1/1. A 3/3 enters, evolve triggers, but 3/3 is destroyed before trigger resolves.
- **Action:** Evolve trigger resolves; entering creature no longer on battlefield, comparison can't be made
- **Expected Result:** No counter placed → creature did NOT "evolve" → no card drawn.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.100a)

702.100c — Noncreature permanent can't have greater P/T than creature. PURE-DEF.

702.100d — Multiple instances trigger separately. DEFERRED — Phase 7.

---

### 702.101 Extort

**Classification: DEFERRED — Phase 7 (triggered ability)**

702.101a — "Whenever you cast a spell, you may pay {W/B}. If you do, each opponent loses 1 life and you gain life equal to total life lost."

**ATOM-702.101a-001**
- **Rule:** 702.101a — Extort triggers on spell cast; optional {W/B} payment drains opponents
- **Mechanism:** Cast trigger + optional mana payment + life drain
- **Minimal Board:** Player controls a permanent with extort and has {W} available. Opponent at 20 life.
- **Action:** Player casts any spell
- **Expected Result:** Extort triggers. Player pays {W/B}. Opponent loses 1 life (→19). Player gains 1 life.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Extort keyword (cast trigger, optional payment, life drain)
- **Dependencies:** Phase 7 (triggered abilities)

702.101b — Multiple instances trigger separately. DEFERRED — Phase 7.

---

### 702.102 Fuse

**Classification: DEFERRED — Phase 9 (split cards)**

702.102a — Cast both halves of a split card from hand.

**ATOM-702.102a-001**
- **Rule:** 702.102a — Fuse allows casting both halves of a split card from hand
- **Mechanism:** Split card casting with combined halves
- **Minimal Board:** Player has a split card with fuse in hand
- **Action:** Player casts both halves as a fused split spell
- **Expected Result:** Spell goes on stack as fused. Both halves' targets chosen. Combined characteristics per 702.102b.
- **Phase:** Phase 9 (split cards)
- **Ticket:** NEW — Fuse keyword (split card dual-cast)

702.102b — Fused spell has combined characteristics. PURE-DEF (cross-ref to 709.4).

702.102c — Total cost includes both halves' mana costs.

**ATOM-702.102c-001**
- **Rule:** 702.102c — Fused split spell's total cost includes mana cost of each half
- **Mechanism:** Combined mana cost calculation
- **Minimal Board:** Split card has halves costing {1}{R} and {2}{U}. Player has {3}{R}{U}.
- **Action:** Player casts fused
- **Expected Result:** Total mana cost is {3}{R}{U}. Player can afford it.
- **Phase:** Phase 9
- **Ticket:** (same as 702.102a)

702.102d — Resolve left half first, then right half. TESTABLE.

**ATOM-702.102d-001**
- **Rule:** 702.102d — Fused spell resolves left half first, then right half
- **Mechanism:** Resolution ordering
- **Minimal Board:** Left half: "Target creature gets +2/+0 until EOT." Right half: "Target creature deals damage equal to its power to target creature." Cast fused targeting same creature for left and attacker/target for right.
- **Action:** Fused spell resolves
- **Expected Result:** Left half applies first (creature gets +2/+0), then right half uses updated power for damage.
- **Phase:** Phase 9
- **Ticket:** (same as 702.102a)

---

### 702.103 Bestow

**Classification: DEFERRED — Phase 5 (type changing) + Phase 8. PRIORITY keyword.**

702.103a — Cast as Aura enchantment with enchant creature for bestow cost.

**ATOM-702.103a-001**
- **Rule:** 702.103a — Bestow casts creature card as Aura with enchant creature for alternative cost
- **Mechanism:** Alternative cost + type change (creature → Aura) + enchant creature grant
- **Minimal Board:** Player has a creature card with "Bestow {3}{W}" in hand. Controls a 2/2 creature.
- **Action:** Player casts it bestowed targeting the 2/2
- **Expected Result:** Spell is on stack as an Aura enchantment with enchant creature. Requires legal target.
- **Phase:** Phase 5 (type changing) + Phase 8
- **Ticket:** NEW — Bestow keyword (alt cost, type change, Aura mode)
- **Dependencies:** T17 (alternative cost), Phase 5 (type-changing CDA)

702.103b — On stack: becomes Aura enchantment, gains enchant creature. TESTABLE (covered by 702.103a test).

702.103c — Copy of bestowed spell is also bestowed. DEFERRED — Phase 8 (copy).

702.103d — Only bestow characteristics evaluated for castability.

**ATOM-702.103d-001**
- **Rule:** 702.103d — When casting bestowed, only modified characteristics are evaluated
- **Mechanism:** Castability check uses Aura characteristics, not creature characteristics
- **Minimal Board:** "Creature spells can't be cast" effect is active. Player casts a creature card bestowed.
- **Action:** Player attempts to cast bestowed
- **Expected Result:** Cast succeeds — the spell is an Aura spell, not a creature spell, so the restriction doesn't apply.
- **Phase:** Phase 5 + Phase 8
- **Ticket:** (same as 702.103a)

702.103e — If bestowed Aura's target is illegal on resolution, it ceases to be bestowed and enters as creature.

**ATOM-702.103e-001**
- **Rule:** 702.103e — Bestowed Aura with illegal target reverts to creature on resolution
- **Mechanism:** Target legality check → type reversion
- **Minimal Board:** Player casts a creature bestowed targeting a 2/2. Before resolution, the 2/2 is destroyed.
- **Action:** Bestowed spell begins resolving; target is illegal
- **Expected Result:** Spell ceases to be bestowed, enters the battlefield as a creature (not an Aura).
- **Phase:** Phase 8
- **Ticket:** (same as 702.103a)

702.103f — Bestowed Aura unattaches → ceases to be bestowed. TESTABLE.

**ATOM-702.103f-001**
- **Rule:** 702.103f — Bestowed Aura becomes unattached → ceases to be bestowed (becomes creature)
- **Mechanism:** Unattach triggers type reversion
- **Minimal Board:** A bestowed Aura is attached to a creature. The enchanted creature is destroyed.
- **Action:** Enchanted creature leaves battlefield
- **Expected Result:** Bestowed Aura becomes unattached, ceases to be bestowed, becomes a creature on the battlefield.
- **Phase:** Phase 8
- **Ticket:** (same as 702.103a)

702.103g — Bestowed Aura phases in unattached → ceases to be bestowed. DEFERRED — Phase 8 (phasing).

---

### 702.104 Tribute

**Classification: DEFERRED — Phase 6 (ETB replacement) + Phase 7 (triggered ability)**

702.104a — "As this creature enters, choose an opponent. That player may put an additional N +1/+1 counters on it."

**ATOM-702.104a-001**
- **Rule:** 702.104a — Tribute N: opponent chooses whether creature enters with N +1/+1 counters
- **Mechanism:** ETB replacement (opponent's choice) + counter placement
- **Minimal Board:** Player casts a creature with Tribute 3.
- **Action:** Creature enters; chosen opponent decides NOT to pay tribute
- **Expected Result:** Creature enters without extra counters. "Tribute wasn't paid" condition is true for associated triggers.
- **Phase:** Phase 6 + Phase 7 + Phase 8
- **Ticket:** NEW — Tribute keyword (ETB opponent choice + conditional trigger)
- **Dependencies:** Phase 6 (replacement), Phase 7 (triggered), T01 (counters)

702.104b — "If tribute wasn't paid" condition definition. TESTABLE (linked trigger).

**ATOM-702.104b-001**
- **Rule:** 702.104b — Tribute not paid → "if tribute wasn't paid" triggered ability fires
- **Mechanism:** Conditional triggered ability linked to tribute payment
- **Minimal Board:** Player casts a 2/2 creature with "Tribute 3" and "When this creature enters, if tribute wasn't paid, it deals 3 damage to target player."
- **Action:** Chosen opponent declines to pay tribute (creature enters as 2/2, no extra counters)
- **Expected Result:** "If tribute wasn't paid" trigger fires → 3 damage to target player.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.104a)

**ATOM-702.104b-002**
- **Rule:** 702.104b — Tribute paid → creature enters with counters, "wasn't paid" trigger does NOT fire
- **Mechanism:** Positive tribute path
- **Minimal Board:** Same setup as above.
- **Action:** Chosen opponent PAYS tribute → creature enters as 5/5 (2/2 + 3 counters)
- **Expected Result:** "If tribute wasn't paid" trigger does NOT fire. No damage dealt.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.104a)

**Tag: LINKED-ABILITY-PATTERN** — Same as Devour/Exploit. Tribute stores a "was paid" boolean that linked abilities reference.

---

### 702.105 Dethrone

**Classification: DEFERRED — Phase 7 (triggered ability)**

702.105a — "Whenever this creature attacks the player with the most life or tied for most life, put a +1/+1 counter on this creature."

**ATOM-702.105a-001**
- **Rule:** 702.105a — Dethrone triggers on attacking the player with most life
- **Mechanism:** Attack trigger with life-total comparison
- **Minimal Board:** Player (18 life) attacks opponent (20 life, highest) with a creature with dethrone
- **Action:** Creature attacks the opponent with most life
- **Expected Result:** Dethrone triggers. +1/+1 counter placed on creature.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Dethrone keyword (attack trigger, life comparison)
- **Dependencies:** Phase 7 (triggered abilities), T01 (counters)

702.105b — Multiple instances trigger separately. DEFERRED — Phase 7.

---

### 702.106 Hidden Agenda

**Classification: OUT-OF-SCOPE (Conspiracy format)**

702.106a–f — All sub-rules relate to conspiracy cards in the command zone. Conspiracy is permanently out of scope.

---

### 702.107 Outlast

**Classification: DEFERRED — Phase 8 (activated ability)**

702.107a — "[Cost], {T}: Put a +1/+1 counter on this creature. Activate only as a sorcery."

**ATOM-702.107a-001**
- **Rule:** 702.107a — Outlast: pay cost + tap to put +1/+1 counter, sorcery speed
- **Mechanism:** Activated ability (mana + tap) with sorcery restriction + counter placement
- **Minimal Board:** Player controls a creature with "Outlast {W}", has {W}, main phase, stack empty
- **Action:** Activate outlast
- **Expected Result:** Creature taps, {W} paid, +1/+1 counter placed. Rejected if not sorcery speed.
- **Phase:** Phase 8
- **Ticket:** NEW — Outlast keyword (sorcery-speed tap activated, +1/+1 counter)
- **Dependencies:** T01 (counters), T19 (ActivationRestriction::SorcerySpeed)

---

### 702.108 Prowess

**Classification: DEFERRED — Phase 7 (triggered ability). PRIORITY keyword.**

702.108a — "Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn."

**ATOM-702.108a-001**
- **Rule:** 702.108a — Prowess triggers on noncreature spell cast, giving +1/+1 until EOT
- **Mechanism:** Cast trigger (noncreature filter) + P/T modification
- **Minimal Board:** Player controls a 2/2 with prowess. Player casts an instant.
- **Action:** Instant is cast
- **Expected Result:** Prowess triggers. Creature becomes 3/3 until end of turn.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Prowess keyword (noncreature cast trigger, +1/+1 until EOT)
- **Dependencies:** Phase 7 (triggered abilities), Phase 5 (continuous effect until EOT)

**ATOM-702.108a-002**
- **Rule:** 702.108a — Prowess does NOT trigger on creature spell
- **Mechanism:** Noncreature filter
- **Minimal Board:** Player controls a 2/2 with prowess. Player casts a creature spell.
- **Action:** Creature spell is cast
- **Expected Result:** Prowess does NOT trigger. Creature stays 2/2.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as above)

702.108b — Multiple instances trigger separately. DEFERRED — Phase 7.

---

### 702.109 Dash

**Classification: DEFERRED — Phase 8. PRIORITY keyword.**

702.109a — Three abilities: alt cost, delayed trigger (return to hand at end step), haste while dash cost was paid.

**ATOM-702.109a-001**
- **Rule:** 702.109a — Dash: cast for alt cost, gains haste, returns to hand at end of next end step
- **Mechanism:** Alternative cost + haste grant + delayed trigger (return to hand)
- **Minimal Board:** Player casts a 3/2 creature for its dash cost
- **Action:** Creature enters the battlefield
- **Expected Result:** Creature has haste. At beginning of next end step, it returns to owner's hand.
- **Phase:** Phase 8
- **Ticket:** NEW — Dash keyword (alt cost, haste, delayed return-to-hand)
- **Dependencies:** T17 (alternative cost), Phase 7 (delayed triggers)

**ATOM-702.109a-002**
- **Rule:** 702.109a — Dash creature cast normally does NOT have haste or return-to-hand
- **Mechanism:** Conditional abilities based on dash cost payment
- **Minimal Board:** Player casts the same creature for its normal mana cost (not dash)
- **Action:** Creature enters the battlefield
- **Expected Result:** Creature does NOT have haste (summoning sickness applies). No delayed return trigger.
- **Phase:** Phase 8
- **Ticket:** (same as above)

---

### 702.110 Exploit

**Classification: DEFERRED — Phase 7 (triggered ability)**

702.110a — "When this creature enters, you may sacrifice a creature."

**ATOM-702.110a-001**
- **Rule:** 702.110a — Exploit: ETB trigger, may sacrifice a creature
- **Mechanism:** ETB triggered ability + optional sacrifice
- **Minimal Board:** Player casts a creature with exploit, controls another creature
- **Action:** Creature enters; exploit triggers
- **Expected Result:** Player may sacrifice a creature (can sacrifice the exploit creature itself or another). If they do, the creature "exploits" per 702.110b.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Exploit keyword (ETB trigger, optional sacrifice)
- **Dependencies:** Phase 7 (triggered abilities), Sacrifice primitive

702.110b — "Exploits a creature" = sacrificed a creature as exploit resolves. TESTABLE (linked ability reference).

**ATOM-702.110b-001**
- **Rule:** 702.110b — Exploit: "the creature it exploited" linked ability
- **Mechanism:** Linked ability referencing the sacrificed creature
- **Minimal Board:** Player controls a 2/2 with exploit and "When this creature exploits a creature, draw cards equal to the exploited creature's toughness." Also controls a 1/4.
- **Action:** Exploit triggers, player sacrifices the 1/4.
- **Expected Result:** "Exploits a creature" trigger fires. Player draws 4 cards (exploited creature's toughness was 4).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.110a)
- **Dependencies:** LINKED-ABILITY-PATTERN (exploit stores reference to sacrificed creature)

--- End of Chunk 2 ---

## Chunk 3: 702.111–702.126 (Menace through Improvise)

### 702.111 Menace

**Classification: ALREADY-IMPLEMENTED (Phase 4 combat keywords). PRIORITY keyword.**

702.111a — Menace is an evasion ability. PURE-DEF.

702.111b — "A creature with menace can't be blocked except by two or more creatures."

**ATOM-702.111b-001**
- **Rule:** 702.111b — Menace requires two or more blockers
- **Mechanism:** Blocking restriction (minimum blocker count)
- **Minimal Board:** Attacking creature has menace. Defender controls one creature.
- **Action:** Defender attempts to block with a single creature
- **Expected Result:** Block is illegal. Menace creature can only be blocked by 2+ creatures.
- **Phase:** Phase 4 (already implemented)
- **Ticket:** T21 (combat keywords) — verify menace blocking validation

**ATOM-702.111b-002**
- **Rule:** 702.111b — Menace creature CAN be blocked by two or more creatures
- **Mechanism:** Legal block with sufficient blockers
- **Minimal Board:** Attacking creature has menace. Defender controls two creatures.
- **Action:** Defender blocks with both creatures
- **Expected Result:** Block is legal.
- **Phase:** Phase 4
- **Ticket:** T21

702.111c — Multiple instances redundant. PURE-DEF.

**IMPLEMENTATION NOTE:** Menace is in the `KeywordAbility` enum but **NOT enforced in blocker validation**. `validate_blockers()` checks flying/reach and block count limits but has no menace logic. Needs a post-assignment check: for each attacker with menace, if blocked at all, must be blocked by ≥ 2 creatures. This is a set-level constraint (can't check per-blocker). **Action:** Phase 4 fix ticket.

---

### 702.112 Renown

**Classification: DEFERRED — Phase 7 (triggered ability)**

702.112a — "When this creature deals combat damage to a player, if it isn't renowned, put N +1/+1 counters on it and it becomes renowned."

**ATOM-702.112a-001**
- **Rule:** 702.112a — Renown N triggers on combat damage to player, places N counters, becomes renowned
- **Mechanism:** Combat damage trigger + intervening-if (not renowned) + counter placement + designation
- **Minimal Board:** A 2/2 creature with Renown 1, not yet renowned. Opponent at 20 life.
- **Action:** Creature deals combat damage to opponent
- **Expected Result:** Renown triggers. 1 +1/+1 counter placed (becomes 3/3). Creature becomes renowned. Future combat damage does NOT re-trigger.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Renown keyword (combat damage trigger, renowned designation)
- **Dependencies:** Phase 7 (triggered abilities), T01 (counters), D17 (per-permanent designations)

702.112b — "Renowned" is a designation, not an ability or copiable value. PURE-DEF.

702.112c — Multiple instances: first to resolve makes creature renowned, rest do nothing. TESTABLE.

**ATOM-702.112c-001**
- **Rule:** 702.112c — Multiple renown instances: first resolves sets renowned, subsequent have no effect
- **Mechanism:** Intervening-if check on resolution
- **Minimal Board:** Creature with two instances of Renown 1 deals combat damage
- **Action:** Both triggers go on stack, first resolves
- **Expected Result:** First trigger: +1/+1 counter, becomes renowned. Second trigger: no effect (already renowned).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.112a)

---

### 702.113 Awaken

**Classification: DEFERRED — Phase 5 (type changing + continuous effect) + Phase 8**

702.113a — Alt cost + spell ability: "put N +1/+1 counters on target land you control. That land becomes a 0/0 Elemental creature with haste. It's still a land."

**ATOM-702.113a-001**
- **Rule:** 702.113a — Awaken animates a land into a 0/0 Elemental creature with haste + N counters
- **Mechanism:** Alternative cost + land animation (type change) + counter placement + haste
- **Minimal Board:** Player casts an instant with "Awaken 4—{5}{U}" for the awaken cost, targeting a land
- **Action:** Spell resolves
- **Expected Result:** 4 +1/+1 counters placed on the land. Land becomes a 0/0 Elemental creature with haste (still a land). Effectively a 4/4.
- **Phase:** Phase 5 (type changing, continuous effect) + Phase 8
- **Ticket:** NEW — Awaken keyword (alt cost, land animation, counters)
- **Dependencies:** T17 (alternative cost), Phase 5 (type change), T01 (counters)

**ATOM-702.113a-002**
- **Rule:** 702.113a — Awaken: spell's original effects still apply when awaken cost is paid
- **Mechanism:** Additional cost does not replace spell effects
- **Minimal Board:** Player casts "Sheer Drop" ({2}{W} destroy target tapped creature, Awaken 3 — {5}{W}) for awaken cost, targeting tapped creature and own land.
- **Action:** Spell resolves
- **Expected Result:** Tapped creature is destroyed (original effect). Land gets 3 +1/+1 counters and becomes a 0/0 Elemental creature (awaken effect). Both effects apply.
- **Phase:** Phase 8
- **Ticket:** (same as 702.113a)
- **Note:** Awaken is an additional cost + additional effect, not a replacement. Handled by T17 additional cost infrastructure.

702.113b — Awaken target only chosen if awaken cost was paid. TESTABLE.

**ATOM-702.113b-001**
- **Rule:** 702.113b — Awaken target is only chosen if awaken cost was paid
- **Mechanism:** Conditional target selection
- **Minimal Board:** Player casts an awaken spell for its normal cost (not awaken cost)
- **Action:** Spell is cast normally
- **Expected Result:** No land target is chosen. Spell resolves with only its normal effect.
- **Phase:** Phase 8
- **Ticket:** (same as 702.113a)

---

### 702.114 Devoid

**Classification: DEFERRED — Phase 5 (CDA)**

702.114a — "Devoid" means "This object is colorless." Characteristic-defining ability, functions everywhere.

**ATOM-702.114a-001**
- **Rule:** 702.114a — Devoid makes the object colorless everywhere (CDA)
- **Mechanism:** Characteristic-defining ability overriding color
- **Minimal Board:** A card with devoid and mana cost {2}{R} is in a player's hand
- **Action:** Check the card's color
- **Expected Result:** Card is colorless (not red), despite having {R} in its mana cost. This applies in all zones.
- **Phase:** Phase 5 (CDAs, Layer 5 color)
- **Ticket:** L05 (Layer 5 color-changing effects)

**ATOM-702.114a-002**
- **Rule:** 702.114a — Devoid: card is colorless despite colored mana symbols
- **Mechanism:** Characteristic-defining effect (color) vs mana cost color indicators
- **Minimal Board:** Player's library contains a card with "Devoid" and mana cost {1}{R}{G}. An effect says "Search your library for a colorless card."
- **Action:** Player searches library
- **Expected Result:** The devoid card is a legal find (it's colorless). Despite having {R}{G} in its mana cost, devoid overrides the color-from-mana-cost rule.
- **Phase:** Phase 5 (layer 5 — color CDA)
- **Ticket:** (same as 702.114a)

**BOUNDARY-702.114a-001**
- **Rule:** 702.114a — Devoid only removes color, not mana cost
- **Mechanism:** CDA scope boundary
- **In-set:** A devoid card with {2}{R} mana cost → colorless
- **Out-of-set:** The card's mana cost is still {2}{R}, mana value is still 3
- **Phase:** Phase 5
- **Ticket:** L05

---

### 702.115 Ingest

**Classification: DEFERRED — Phase 7 (triggered ability)**

702.115a — "Whenever this creature deals combat damage to a player, that player exiles the top card of their library."

**ATOM-702.115a-001**
- **Rule:** 702.115a — Ingest exiles top card of damaged player's library on combat damage
- **Mechanism:** Combat damage trigger + exile from library top
- **Minimal Board:** A creature with ingest deals combat damage to opponent
- **Action:** Combat damage is dealt
- **Expected Result:** Trigger fires. Top card of opponent's library is exiled.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Ingest keyword (combat damage trigger, exile top card)
- **Dependencies:** Phase 7 (triggered abilities)

702.115b — Multiple instances trigger separately. DEFERRED — Phase 7.

---

### 702.116 Myriad

**Classification: DEFERRED — Phase 7 (triggered ability) + Phase 9 (multiplayer)**

702.116a — "Whenever this creature attacks, for each opponent other than defending player, create a token copy tapped and attacking that player. Exile tokens at end of combat."

**ATOM-702.116a-001**
- **Rule:** 702.116a — Myriad creates token copies attacking each other opponent
- **Mechanism:** Attack trigger + token creation (copies) + tapped and attacking + delayed exile
- **Minimal Board:** 3-player game. Player attacks opponent A with a creature with myriad.
- **Action:** Creature attacks opponent A
- **Expected Result:** A token copy of the creature is created tapped and attacking opponent B. At end of combat, the token is exiled.
- **Phase:** Phase 7 + Phase 9 (multiplayer)
- **Ticket:** NEW — Myriad keyword (attack trigger, token copies per opponent)
- **Dependencies:** Phase 7 (triggered abilities), Phase 9 (multiplayer), CreateToken + Copy primitive

702.116b — Multiple instances trigger separately. DEFERRED — Phase 7.

---

### 702.117 Surge

**Classification: DEFERRED — Phase 8 (alternative cost with condition)**

702.117a — "You may pay [cost] rather than pay this spell's mana cost if you or a teammate cast another spell this turn."

**ATOM-702.117a-001**
- **Rule:** 702.117a — Surge: alt cost available if you or teammate cast another spell this turn
- **Mechanism:** Conditional alternative cost (per-turn spell tracking)
- **Minimal Board:** Player casts a spell, then attempts to cast a second spell with surge
- **Action:** Player pays surge cost
- **Expected Result:** Surge cost is legal (another spell was cast this turn). Spell is cast for surge cost.
- **Phase:** Phase 8
- **Ticket:** NEW — Surge keyword (conditional alt cost, spell-this-turn tracker)
- **Dependencies:** T17 (alternative cost), D16 (per-turn tracker)

---

### 702.118 Skulk

**Classification: DEFERRED — Phase 8 (evasion ability)**

702.118a — Skulk is an evasion ability. PURE-DEF.

702.118b — "A creature with skulk can't be blocked by creatures with greater power."

**ATOM-702.118b-001**
- **Rule:** 702.118b — Skulk: can't be blocked by creatures with greater power
- **Mechanism:** Blocking restriction based on power comparison
- **Minimal Board:** A 1/1 creature with skulk attacks. Defender controls a 3/3.
- **Action:** Defender attempts to block with the 3/3
- **Expected Result:** Block is illegal (3 > 1, blocker has greater power).
- **Phase:** Phase 8
- **Ticket:** NEW — Skulk keyword (power-based blocking restriction)

**ATOM-702.118b-002**
- **Rule:** 702.118b — Skulk: CAN be blocked by creature with equal or lesser power
- **Mechanism:** Legal block with ≤ power
- **Minimal Board:** A 2/1 with skulk attacks. Defender controls a 1/1.
- **Action:** Defender blocks with the 1/1
- **Expected Result:** Block is legal (1 ≤ 2).
- **Phase:** Phase 8
- **Ticket:** (same as above)

702.118c — Multiple instances redundant. PURE-DEF.

---

### 702.119 Emerge

**Classification: DEFERRED — Phase 5-Pre T17 (cost modification)**

702.119a — "You may cast this spell by paying [cost] and sacrificing a creature rather than paying its mana cost" + "total cost reduced by sacrificed creature's mana value in generic mana."

**ATOM-702.119a-001**
- **Rule:** 702.119a — Emerge: alt cost + sacrifice creature + cost reduction by creature's MV
- **Mechanism:** Alternative cost + sacrifice as cost + generic mana reduction
- **Minimal Board:** Player casts a spell with "Emerge {5}{U}{U}" by sacrificing a creature with MV 3
- **Action:** Player pays emerge cost, sacrifices the creature
- **Expected Result:** Total cost is {5}{U}{U} minus {3} generic = {2}{U}{U}. Creature is sacrificed.
- **Phase:** Phase 5-Pre T17 (cost modification) + Phase 8
- **Ticket:** T17 (cost modification pipeline) + NEW — Emerge keyword
- **Dependencies:** T17 (cost modification, sacrifice-as-cost)

702.119b — Emerge from [quality] variant. Same mechanism, broader sacrifice targets. DEFERRED — Phase 8.

702.119c — Choose creature to sacrifice at 601.2b, sacrifice at 601.2h. PURE-DEF (casting procedure ref).

---

### 702.120 Escalate

**Classification: DEFERRED — Phase 8 (additional cost for extra modes)**

702.120a — "For each mode you choose beyond the first, you pay an additional [cost]."

**ATOM-702.120a-001**
- **Rule:** 702.120a — Escalate: additional cost per extra mode chosen
- **Mechanism:** Modal spell + additional cost scaling with mode count
- **Minimal Board:** Player casts a modal spell with escalate {1} and 3 modes. Chooses all 3 modes.
- **Action:** Player pays base cost + {2} (escalate for 2 extra modes)
- **Expected Result:** All 3 modes execute. Total additional cost for escalate is {2} ({1} × 2 extra modes).
- **Phase:** Phase 8
- **Ticket:** NEW — Escalate keyword (per-mode additional cost)
- **Dependencies:** T17 (additional cost), Modal spell infrastructure

---

### 702.121 Melee

**Classification: DEFERRED — Phase 7 (triggered ability) + Phase 9 (multiplayer)**

702.121a — "Whenever this creature attacks, it gets +1/+1 until end of turn for each opponent you attacked with a creature this combat."

**ATOM-702.121a-001**
- **Rule:** 702.121a — Melee: +1/+1 per opponent attacked this combat
- **Mechanism:** Attack trigger + multiplayer opponent counting + P/T modification
- **Minimal Board:** 3-player game. Player attacks opponent A with melee creature and opponent B with another creature.
- **Action:** Melee creature attacks
- **Expected Result:** Melee triggers. +2/+2 until EOT (attacked 2 opponents). In 2-player: always +1/+1.
- **Phase:** Phase 7 + Phase 9 (multiplayer)
- **Ticket:** NEW — Melee keyword (attack trigger, per-opponent-attacked bonus)
- **Dependencies:** Phase 7 (triggered abilities), Phase 9 (multiplayer attack tracking)

702.121b — Multiple instances trigger separately. DEFERRED — Phase 7.

---

### 702.122 Crew

**Classification: DEFERRED — Phase 8 (activated ability). PRIORITY keyword.**

702.122a — "Tap any number of other untapped creatures you control with total power N or greater: This permanent becomes an artifact creature until end of turn."

**ATOM-702.122a-001**
- **Rule:** 702.122a — Crew N: tap creatures with total power ≥ N to animate Vehicle
- **Mechanism:** Activated ability (tap creatures as cost) + type change (Vehicle → artifact creature until EOT)
- **Minimal Board:** Player controls a Vehicle with Crew 3 and two creatures (2/2 and 2/2)
- **Action:** Tap both creatures to crew the Vehicle
- **Expected Result:** Vehicle becomes an artifact creature until end of turn. Total power tapped = 4 ≥ 3.
- **Phase:** Phase 8
- **Ticket:** NEW — Crew keyword (tap-creatures-as-cost, Vehicle animation)
- **Dependencies:** Phase 5 (type changing continuous effect until EOT), T19 (activation cost: tap other creatures)

**ATOM-702.122a-002**
- **Rule:** 702.122a — Crew fails if total power of tapped creatures < N
- **Mechanism:** Cost validation (minimum power threshold)
- **Minimal Board:** Player controls Vehicle with Crew 4 and one creature (2/2)
- **Action:** Attempt to crew with the 2/2 alone
- **Expected Result:** Crew cost cannot be paid (2 < 4). Ability activation fails.
- **Phase:** Phase 8
- **Ticket:** (same as above)

702.122b — "Crews a Vehicle" = tapped to pay crew cost. PURE-DEF.

**ATOM-702.122-003**
- **Rule:** 702.122a — Crew on a non-Vehicle permanent (via ability copying)
- **Mechanism:** Crew ability functions on any permanent, not just Vehicles
- **Minimal Board:** A non-Vehicle artifact has gained "Crew 3" through an ability-copying effect. Player controls a 3/3 creature.
- **Action:** Tap the 3/3 to crew the artifact
- **Expected Result:** The artifact becomes an artifact creature until end of turn (even though it's not a Vehicle).
- **Phase:** Phase 8
- **Ticket:** (same as 702.122a)

**ATOM-702.122b-001**
- **Rule:** 702.122b — "Whenever this creature crews a Vehicle" trigger
- **Mechanism:** Crew event triggers linked abilities
- **Minimal Board:** Player controls creature A with "This creature crews Vehicles as though its power were 2 greater" (power 1 → effective 3 for crewing) and creature B with "Whenever this creature crews a Vehicle, that Vehicle gains flying until end of turn." Vehicle with Crew 3.
- **Action:** Tap creature A and creature B to crew the Vehicle (effective total power: 3+1 = 4 ≥ 3)
- **Expected Result:** Vehicle becomes artifact creature. B's trigger fires → Vehicle gains flying until EOT. A's power-boost is accounted for in the crew cost check.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.122a)
- **Tag: CREW-EVENT** — Engine needs a "creature crewed a Vehicle" event for triggers.

702.122c — "Can't crew Vehicles" = can't be tapped for crew cost. TESTABLE.

**ATOM-702.122c-001**
- **Rule:** 702.122c — Creature that "can't crew Vehicles" cannot be tapped for crew cost
- **Mechanism:** Crew restriction
- **Minimal Board:** Player controls a Vehicle with Crew 1, a 3/3 creature with "can't crew Vehicles"
- **Action:** Attempt to tap the restricted creature to crew
- **Expected Result:** Illegal — creature cannot be used to pay crew cost.
- **Phase:** Phase 8
- **Ticket:** (same as 702.122a)

702.122d — "Whenever this Vehicle becomes crewed" trigger definition. DEFERRED — Phase 7.

---

### 702.123 Fabricate

**Classification: DEFERRED — Phase 7 (triggered ability)**

702.123a — "When this permanent enters, you may put N +1/+1 counters on it. If you don't, create N 1/1 Servo tokens."

**ATOM-702.123a-001**
- **Rule:** 702.123a — Fabricate N: choose counters or Servo tokens on ETB
- **Mechanism:** ETB trigger + modal choice (counters vs tokens)
- **Minimal Board:** Player casts a creature with Fabricate 2
- **Action:** Creature enters; player chooses tokens
- **Expected Result:** 2 1/1 colorless Servo artifact creature tokens created. No counters on the creature.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Fabricate keyword (ETB trigger, counters-or-tokens choice)
- **Dependencies:** Phase 7 (triggered abilities), T01 (counters), CreateToken primitive

**ATOM-702.123a-002**
- **Rule:** 702.123a — Fabricate: choosing counters path
- **Mechanism:** Counter placement path
- **Minimal Board:** Player casts a 2/3 creature with Fabricate 2
- **Action:** Creature enters; player chooses counters
- **Expected Result:** 2 +1/+1 counters placed on creature (becomes 4/5). No tokens created.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as above)

702.123b — Multiple instances trigger separately. DEFERRED — Phase 7.

---

### 702.124 Partner

**Classification: DEFERRED — Phase 9 (Commander format)**

702.124a–n — All sub-rules relate to Commander variant deck construction and gameplay.

702.124a — Partner abilities modify Commander deck construction rules. DEFERRED — Phase 9.
702.124b — Deck must contain exactly 100 cards including two commanders. DEFERRED — Phase 9.
702.124c — Combined color identity of two commanders. DEFERRED — Phase 9.
702.124d — Two commanders function independently. DEFERRED — Phase 9.
702.124e — Effect referring to "your commander" with two commanders. DEFERRED — Phase 9.
702.124f — Different partner abilities are distinct, cannot combine. DEFERRED — Phase 9.
702.124g — If a card has multiple partner abilities, choose one. DEFERRED — Phase 9.
702.124h — "Partner" basic variant. DEFERRED — Phase 9.
702.124i — "Partner—[text]" variant. DEFERRED — Phase 9.
702.124j — "Partner with [name]" variant + search triggered ability. DEFERRED — Phase 9.
702.124k — "Choose a Background" variant. DEFERRED — Phase 9.
702.124m — "Doctor's companion" variant. DEFERRED — Phase 9.
702.124n — Effect referring to partner by name. DEFERRED — Phase 9.

---

### 702.125 Undaunted

**Classification: DEFERRED — Phase 5-Pre T17 (cost modification) + Phase 9 (multiplayer)**

702.125a — "This spell costs {1} less to cast for each opponent you have."

**ATOM-702.125a-001**
- **Rule:** 702.125a — Undaunted reduces cost by {1} per opponent
- **Mechanism:** Cost reduction based on opponent count
- **Minimal Board:** 4-player Commander game. Player has 3 opponents. Spell costs {4}{W} with undaunted.
- **Action:** Player casts the spell
- **Expected Result:** Cost reduced by {3} (3 opponents). Total cost is {1}{W}.
- **Phase:** Phase 5-Pre T17 + Phase 9
- **Ticket:** T17 (cost modification) + NEW — Undaunted keyword
- **Dependencies:** T17 (cost reduction pipeline), Phase 9 (multiplayer opponent count)

702.125b — Players who left the game not counted. DEFERRED — Phase 9.

702.125c — Multiple instances each apply. TESTABLE.

**ATOM-702.125c-001**
- **Rule:** 702.125c — Multiple undaunted instances each reduce cost separately
- **Mechanism:** Stacking cost reduction
- **Minimal Board:** Spell with two instances of undaunted. 2-player game (1 opponent).
- **Action:** Player casts the spell
- **Expected Result:** Cost reduced by {2} total ({1} per instance × 1 opponent × 2 instances).
- **Phase:** Phase 5-Pre T17 + Phase 8
- **Ticket:** (same as 702.125a)

---

### 702.126 Improvise

**Classification: DEFERRED — Phase 5-Pre T17 (cost modification)**

702.126a — "For each generic mana in this spell's total cost, you may tap an untapped artifact you control rather than pay that mana."

**ATOM-702.126a-001**
- **Rule:** 702.126a — Improvise: tap artifacts to pay generic mana costs
- **Mechanism:** Alternative mana payment (tap artifacts for generic)
- **Minimal Board:** Player casts a spell costing {4}{U} with improvise. Controls 3 untapped artifacts.
- **Action:** Player taps 3 artifacts and pays {1}{U} in mana
- **Expected Result:** Total cost satisfied: 3 generic from artifacts + {1}{U} in mana = {4}{U}.
- **Phase:** Phase 5-Pre T17
- **Ticket:** T17 (cost modification) + NEW — Improvise keyword
- **Dependencies:** T17 (mana payment alternatives)

**ATOM-702.126a-002**
- **Rule:** 702.126a — Improvise: tapping artifacts only pays generic mana, not colored
- **Mechanism:** Cost reduction restriction (generic only)
- **Minimal Board:** Player casts a spell with "Improvise" costing {2}{U}{U}. Controls 4 untapped artifacts.
- **Action:** Player taps all 4 artifacts
- **Expected Result:** Only {2} generic is paid by tapping artifacts (maximum generic portion). Player must still pay {U}{U} from mana pool. Tapping extra artifacts beyond the generic portion has no effect.
- **Phase:** Phase 5-Pre T17 + Phase 8
- **Ticket:** (same as 702.126a)
- **Tag: SHARED-BEHAVIOR convoke** — Same "tap to pay generic" pattern.

702.126b — Improvise isn't additional or alternative cost; applies after total cost determined. PURE-DEF.

702.126c — Multiple instances redundant. PURE-DEF.

**Tag: SHARED-BEHAVIOR convoke-improvise** — Convoke (702.50, Session 7B) and Improvise (702.126) share a "tap permanents to reduce cost" pattern. Key differences: Convoke: tap creatures → pay {1} OR one mana of creature's color per creature. Improvise: tap artifacts → pay {1} per artifact (generic only). Implementation should share a `tap_to_reduce_cost(permanent_filter, reduction_type)` helper. `reduction_type` is either `GenericOnly` (Improvise) or `GenericOrColor` (Convoke).

--- End of Chunk 3 ---

## Chunk 4: 702.127–702.140 (Aftermath through Mutate)

### 702.127 Aftermath

**Classification: DEFERRED — Phase 9 (split cards)**

702.127a — Three static abilities: cast from GY, can't cast from other zones, exile instead of leaving stack if cast from GY.

**ATOM-702.127a-001**
- **Rule:** 702.127a — Aftermath: cast this half from graveyard; can't cast from other zones; exile on leaving stack
- **Mechanism:** Zone-restricted casting + exile replacement on stack departure
- **Minimal Board:** Player has a split card with aftermath in graveyard
- **Action:** Player casts the aftermath half from graveyard
- **Expected Result:** Spell resolves. Instead of going to graveyard, it is exiled.
- **Phase:** Phase 9 (split cards)
- **Ticket:** NEW — Aftermath keyword (GY-only cast, exile on leave stack)
- **Dependencies:** Phase 9 (split cards), T19 (zone-casting)

**ATOM-702.127a-002**
- **Rule:** 702.127a — Aftermath half cannot be cast from hand
- **Mechanism:** Zone restriction on casting
- **Minimal Board:** Player has a split card with aftermath in hand
- **Action:** Player attempts to cast the aftermath half from hand
- **Expected Result:** Cast is illegal. Aftermath half can only be cast from graveyard.
- **Phase:** Phase 9
- **Ticket:** (same as above)

---

### 702.128 Embalm

**Classification: DEFERRED — Phase 8 (GY activated ability + token creation)**

702.128a — "[Cost], Exile this card from your graveyard: Create a token that's a copy of this card, except it's white, has no mana cost, and is a Zombie in addition to its other types. Sorcery speed."

**ATOM-702.128a-001**
- **Rule:** 702.128a — Embalm creates a modified token copy from graveyard
- **Mechanism:** GY activated ability + exile self + token creation (modified copy)
- **Minimal Board:** Player has a 2/2 blue creature with "Embalm {3}{W}" in graveyard
- **Action:** Activate embalm
- **Expected Result:** Card is exiled. A token is created: 2/2, white (not blue), no mana cost, Zombie in addition to its types.
- **Phase:** Phase 8
- **Ticket:** NEW — Embalm keyword (GY activated, exile self, modified token copy)
- **Dependencies:** T19 (activation_zone: Graveyard), CreateToken + Copy primitive

702.128b — "Embalmed" token definition. PURE-DEF.

---

### 702.129 Eternalize

**Classification: DEFERRED — Phase 8 (GY activated ability + token creation)**

702.129a — "[Cost], Exile from GY: Create a 4/4 black Zombie token copy with no mana cost. Sorcery speed."

**ATOM-702.129a-001**
- **Rule:** 702.129a — Eternalize creates a 4/4 black Zombie token copy from graveyard
- **Mechanism:** GY activated ability + exile self + token creation (modified copy, forced 4/4)
- **Minimal Board:** Player has a 2/2 white creature with "Eternalize {4}{B}{B}" in graveyard
- **Action:** Activate eternalize
- **Expected Result:** Card is exiled. Token created: 4/4, black (not white), no mana cost, Zombie in addition to types.
- **Phase:** Phase 8
- **Ticket:** NEW — Eternalize keyword (GY activated, exile self, 4/4 modified token)
- **Dependencies:** T19 (activation_zone: Graveyard), CreateToken + Copy primitive

---

### 702.130 Afflict

**Classification: DEFERRED — Phase 7 (triggered ability)**

702.130a — "Whenever this creature becomes blocked, defending player loses N life."

**ATOM-702.130a-001**
- **Rule:** 702.130a — Afflict N: defending player loses N life when creature becomes blocked
- **Mechanism:** Becomes-blocked trigger + life loss
- **Minimal Board:** A creature with Afflict 3 attacks. Defender blocks with a creature.
- **Action:** Creature becomes blocked
- **Expected Result:** Afflict triggers. Defending player loses 3 life.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Afflict keyword (becomes-blocked trigger, life loss)
- **Dependencies:** Phase 7 (triggered abilities)

702.130b — Multiple instances trigger separately. DEFERRED — Phase 7.

---

### 702.131 Ascend

**Classification: DEFERRED — Phase 8 (designation)**

702.131a — On instant/sorcery: "If you control ten or more permanents and you don't have the city's blessing, you get it for the rest of the game."

**ATOM-702.131a-001**
- **Rule:** 702.131a — Ascend on spell: get city's blessing if you control 10+ permanents
- **Mechanism:** Spell ability checking permanent count + granting designation
- **Minimal Board:** Player controls 10 permanents, does not have city's blessing. Casts a spell with ascend.
- **Action:** Spell resolves (ascend is checked as spell ability)
- **Expected Result:** Player gets the city's blessing for the rest of the game.
- **Phase:** Phase 8
- **Ticket:** NEW — Ascend keyword (city's blessing designation)
- **Dependencies:** D17 (per-player designations)

702.131b — On permanent: static ability checks continuously. TESTABLE.

**ATOM-702.131b-001**
- **Rule:** 702.131b — Ascend on permanent: get city's blessing immediately when controlling 10+ permanents
- **Mechanism:** Static ability, no stack, immediate designation
- **Minimal Board:** Player controls 9 permanents including one with ascend. Player plays a land (10th permanent).
- **Action:** Land enters the battlefield
- **Expected Result:** Player gets city's blessing immediately. No trigger, no stack. Opponents cannot respond between land entry and blessing acquisition.
- **Phase:** Phase 8
- **Ticket:** NEW — Ascend keyword (static designation, permanence)
- **Dependencies:** D17 (designations), SBA-adjacent timing

**ATOM-702.131b-002**
- **Rule:** 702.131b — Ascend: mid-resolution acquisition
- **Mechanism:** Static ability checked during spell resolution
- **Minimal Board:** Player controls 9 permanents (one with ascend). Player casts a spell that says "Each player creates a 1/1 white Human creature token. Then destroy all creatures." Player also has a creature with "As long as you have the city's blessing, this creature has indestructible."
- **Action:** Spell resolves: token created (player now controls 10 permanents) → city's blessing acquired → "destroy all creatures" effect happens
- **Expected Result:** Player's creature with indestructible-if-blessed survives the destroy effect because city's blessing was acquired mid-resolution before the destroy clause.
- **Phase:** Phase 8
- **ARCHITECTURAL NOTE:** Requires mid-resolution static ability checks.

702.131c — City's blessing is a designation, no rules meaning beyond marker. TESTABLE (permanence).

**ATOM-702.131c-001**
- **Rule:** 702.131c — City's blessing persists even below 10 permanents
- **Mechanism:** Permanent designation
- **Minimal Board:** Player has city's blessing, controls 10 permanents. 5 permanents are destroyed.
- **Expected Result:** Player still has city's blessing (5 permanents remaining).
- **Phase:** Phase 8
- **Ticket:** (same as 702.131a)

702.131d — After getting city's blessing, continuous effects reapplied before trigger check. PURE-DEF (process ordering).

**Tag: ARCHITECTURE-CONCERN mid-resolution-static-checks** — Ascend requires the engine to check static abilities (specifically ascend) between sequential effects within a single resolving spell. This is NOT the same as SBAs (which run between spells/abilities resolving). Needs design discussion.

---

### 702.132 Assist

**Classification: DEFERRED — Phase 8 + Phase 9 (multiplayer cost sharing)**

702.132a — Another player may pay for generic mana in the spell's total cost.

**ATOM-702.132a-001**
- **Rule:** 702.132a — Assist: another player may pay generic mana in spell's total cost
- **Mechanism:** Cost-sharing with another player (mana ability activation window)
- **Minimal Board:** Player casts a spell with assist costing {6}{W}. Chooses any other player.
- **Action:** Chosen player activates mana abilities and pays {4} of the generic cost. Caster pays {2}{W}.
- **Expected Result:** Spell is cast. Total cost satisfied by combined payment.
- **Phase:** Phase 8 + Phase 9 (multiplayer)
- **Ticket:** NEW — Assist keyword (multiplayer cost sharing)
- **Dependencies:** T17 (cost payment), Phase 9 (multiplayer)

**ATOM-702.132a-002**
- **Rule:** 702.132a — Assist: chosen player declines to pay → caster must pay full cost
- **Mechanism:** Optional cost sharing; chosen player may pay zero
- **Minimal Board:** Player casts a spell with assist costing {6}{W}. Chooses any other player. Chosen player declines to pay any generic mana.
- **Action:** Chosen player pays {0}. Caster must pay {6}{W}.
- **Expected Result:** Caster pays the full {6}{W}. Spell is cast normally. The chosen player's contribution is voluntary — declining is legal and simply means the caster covers everything.
- **Phase:** Phase 8 + Phase 9 (multiplayer)
- **Ticket:** (same as 702.132a)

---

### 702.133 Jump-Start

**Classification: DEFERRED — Phase 8 (GY casting + additional cost)**

702.133a — Cast from GY by discarding a card as additional cost. If cast from GY, exile instead of going anywhere else on leaving stack.

**ATOM-702.133a-001**
- **Rule:** 702.133a — Jump-start: cast from GY by discarding a card; exile on leaving stack
- **Mechanism:** Zone-casting (graveyard) + additional cost (discard) + exile replacement
- **Minimal Board:** Player has an instant with jump-start in graveyard, a card in hand to discard
- **Action:** Player casts the spell from graveyard, discarding a card
- **Expected Result:** Spell resolves. It is exiled instead of going to graveyard (or anywhere else).
- **Phase:** Phase 8
- **Ticket:** NEW — Jump-Start keyword (GY cast, discard additional cost, exile replacement)
- **Dependencies:** T17 (additional cost), T19 (zone-casting)

**ATOM-702.133a-002**
- **Rule:** 702.133a — Jump-Start: cast from hand → goes to graveyard normally (no exile)
- **Mechanism:** Exile replacement only applies when cast via jump-start
- **Minimal Board:** Player casts a spell with jump-start from hand normally (not from GY)
- **Action:** Spell resolves
- **Expected Result:** Spell goes to graveyard as normal. NOT exiled (the exile clause only applies to jump-start casts).
- **Phase:** Phase 8
- **Ticket:** (same as 702.133a)

---

### 702.134 Mentor

**Classification: DEFERRED — Phase 7 (triggered ability)**

702.134a — "Whenever this creature attacks, put a +1/+1 counter on target attacking creature with power less than this creature's power."

**ATOM-702.134a-001**
- **Rule:** 702.134a — Mentor: attack trigger puts +1/+1 counter on weaker attacking creature
- **Mechanism:** Attack trigger + targeted counter placement with power comparison
- **Minimal Board:** Creature A (3/3, mentor) and creature B (2/2). Both attack.
- **Action:** Both declared as attackers; mentor targets creature B
- **Expected Result:** Mentor triggers. +1/+1 counter placed on B (becomes 3/3). B must have power < A's power at time of targeting.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Mentor keyword (attack trigger, power-comparison targeting)
- **Dependencies:** Phase 7 (triggered abilities), T01 (counters)

702.134b — Multiple instances trigger separately. DEFERRED — Phase 7.

702.134c — "Whenever a creature mentors another creature" trigger condition. PURE-DEF.

---

### 702.135 Afterlife

**Classification: DEFERRED — Phase 7 (triggered ability)**

702.135a — "When this permanent is put into a graveyard from the battlefield, create N 1/1 white and black Spirit creature tokens with flying."

**ATOM-702.135a-001**
- **Rule:** 702.135a — Afterlife N: create N Spirit tokens on dies
- **Mechanism:** Dies trigger + token creation
- **Minimal Board:** A creature with Afterlife 2 is on the battlefield. Destroy effect targets it.
- **Action:** Creature is destroyed, goes to graveyard
- **Expected Result:** Afterlife triggers. 2 1/1 white and black Spirit creature tokens with flying are created.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Afterlife keyword (dies trigger, Spirit token creation)
- **Dependencies:** Phase 7 (triggered abilities), CreateToken primitive

702.135b — Multiple instances trigger separately. DEFERRED — Phase 7.

---

### 702.136 Riot

**Classification: DEFERRED — Phase 6 (ETB replacement)**

702.136a — "You may have this permanent enter with an additional +1/+1 counter on it. If you don't, it gains haste."

**ATOM-702.136a-001**
- **Rule:** 702.136a — Riot: choose +1/+1 counter or haste on ETB
- **Mechanism:** ETB replacement (choice: counter or haste)
- **Minimal Board:** Player casts a 3/3 creature with riot
- **Action:** Creature enters; player chooses haste
- **Expected Result:** Creature enters as 3/3 with haste (no counter). Can attack immediately.
- **Phase:** Phase 6 (replacement) + Phase 8
- **Ticket:** NEW — Riot keyword (ETB choice: counter or haste)
- **Dependencies:** Phase 6 (replacement effects), T01 (counters)

**ATOM-702.136a-002**
- **Rule:** 702.136a — Riot: choosing counter path
- **Mechanism:** Counter placement path
- **Minimal Board:** Player casts a 3/3 with riot
- **Action:** Creature enters; player chooses +1/+1 counter
- **Expected Result:** Creature enters as 4/4 (with counter). Does NOT have haste.
- **Phase:** Phase 6 + Phase 8
- **Ticket:** (same as above)

702.136b — Multiple instances work separately (each gives a separate choice). TESTABLE.

**ATOM-702.136b-001**
- **Rule:** 702.136b — Multiple riot instances: each gives a separate choice
- **Mechanism:** Independent replacement per instance
- **Minimal Board:** Creature with two instances of riot enters
- **Action:** Player chooses counter for first riot, haste for second
- **Expected Result:** Creature enters with one +1/+1 counter AND haste.
- **Phase:** Phase 6 + Phase 8
- **Ticket:** (same as 702.136a)

---

### 702.137 Spectacle

**Classification: DEFERRED — Phase 5-Pre T17 (alternative cost with condition)**

702.137a — "You may pay [cost] rather than pay this spell's mana cost if an opponent lost life this turn."

**ATOM-702.137a-001**
- **Rule:** 702.137a — Spectacle: alt cost if opponent lost life this turn
- **Mechanism:** Conditional alternative cost (per-turn life loss tracking)
- **Minimal Board:** Opponent lost 2 life earlier this turn. Player casts a spell with "Spectacle {R}" (normal cost {2}{R}).
- **Action:** Player pays spectacle cost
- **Expected Result:** Spell cast for {R}. Legal because opponent lost life this turn.
- **Phase:** Phase 5-Pre T17 + Phase 8
- **Ticket:** T17 (alternative cost) + NEW — Spectacle keyword
- **Dependencies:** T17 (alternative cost), D16 (per-turn tracker: opponent life loss)

**ATOM-702.137a-002**
- **Rule:** 702.137a — Spectacle: can't use if no opponent lost life this turn
- **Mechanism:** Condition check failure
- **Minimal Board:** No opponent has lost life this turn. Player has a spell with "Spectacle {B}" (normal cost {2}{B}).
- **Action:** Player attempts to cast for spectacle cost
- **Expected Result:** Cast is illegal. Spectacle requires an opponent to have lost life this turn.
- **Phase:** Phase 8
- **Ticket:** (same as 702.137a)
- **Dependencies:** D16 (per-turn life-loss tracker)

---

### 702.138 Escape

**Classification: DEFERRED — Phase 5-Pre T17 (alternative cost + GY casting). PRIORITY keyword.**

702.138a — Cast from GY by paying escape cost (which includes exiling other cards from GY).

**ATOM-702.138a-001**
- **Rule:** 702.138a — Escape: cast from graveyard by paying escape cost
- **Mechanism:** Zone-casting (graveyard) + alternative cost (often includes exiling cards from GY)
- **Minimal Board:** Player has a creature with "Escape {3}{R}{R}, exile four other cards from your graveyard" in GY, plus 4 other cards in GY
- **Action:** Player casts the escape spell
- **Expected Result:** Spell is cast from graveyard. 4 other cards exiled from GY as part of cost. Spell resolves normally.
- **Phase:** Phase 5-Pre T17 + Phase 8
- **Ticket:** T17 (alternative cost) + NEW — Escape keyword
- **Dependencies:** T17 (alt cost), T19 (zone-casting from GY)

702.138b — "Escaped" definition. PURE-DEF.

702.138c — "Escapes with [counters]" = enters with counters if escaped. Linked trigger.

**ATOM-702.138c-001**
- **Rule:** 702.138c — Creature that escaped enters with additional counters
- **Mechanism:** ETB replacement linked to escape condition
- **Minimal Board:** Player casts a creature via escape that has "escapes with two +1/+1 counters"
- **Action:** Creature enters the battlefield
- **Expected Result:** Creature enters with two additional +1/+1 counters (beyond any base counters).
- **Phase:** Phase 6 (replacement) + Phase 8
- **Ticket:** (same as 702.138a)

702.138d — "Escapes with [ability]" = gains ability if escaped. TESTABLE.

**ATOM-702.138d-001**
- **Rule:** 702.138d — Permanent that escaped gains specified ability
- **Mechanism:** Conditional ability grant based on escape status
- **Minimal Board:** Player casts a creature via escape that has "escapes with flying"
- **Action:** Creature enters the battlefield
- **Expected Result:** Creature has flying (because it escaped).
- **Phase:** Phase 8
- **Ticket:** (same as 702.138a)

---

### 702.139 Companion

**Classification: DEFERRED — Phase 9 (outside-the-game, sideboard)**

702.139a — Reveal from outside game if condition met by starting deck. Pay {3} to put into hand (special action).

**ATOM-702.139a-001**
- **Rule:** 702.139a — Companion: reveal from outside game, pay {3} to put into hand as special action
- **Mechanism:** Outside-game zone + deck condition check + special action (pay {3}, add to hand)
- **Minimal Board:** Player has revealed a companion before game start. Main phase, stack empty, has {3}.
- **Action:** Player pays {3} and puts companion into hand
- **Expected Result:** Companion moves from outside the game into hand. Special action, doesn't use stack.
- **Phase:** Phase 9
- **Ticket:** NEW — Companion keyword (outside-game, special action)
- **Dependencies:** Phase 9 (sideboard/outside-game), Special action infrastructure

702.139b — "Starting deck" = deck after sideboard, before commander set aside. PURE-DEF.

702.139c — Once in hand, companion remains in game until game ends. PURE-DEF.

702.139d — Cards can enter Commander games via companion. PURE-DEF (Commander cross-ref).

---

### 702.140 Mutate

**Classification: DEFERRED — Phase 8. PRIORITY keyword.**

702.140a — Alt cost to target a non-Human creature with same owner; becomes a mutating creature spell.

**ATOM-702.140a-001**
- **Rule:** 702.140a — Mutate: cast as mutating creature spell targeting a non-Human creature you own
- **Mechanism:** Alternative cost + targeting restriction (non-Human, same owner)
- **Minimal Board:** Player controls a non-Human creature. Casts a creature with "Mutate {2}{G}" targeting it.
- **Action:** Spell is cast for mutate cost
- **Expected Result:** Spell is a mutating creature spell on the stack, targeting the non-Human creature.
- **Phase:** Phase 8
- **Ticket:** NEW — Mutate keyword (alt cost, mutating creature spell, merge)
- **Dependencies:** T17 (alternative cost), Rule 729 (merging permanents)

702.140b — If target is illegal on resolution, enters as creature normally.

**ATOM-702.140b-001**
- **Rule:** 702.140b — Mutating spell with illegal target enters as creature normally
- **Mechanism:** Target illegality fallback
- **Minimal Board:** Player casts a mutating creature spell. Target creature is destroyed before resolution.
- **Action:** Mutating spell resolves; target is illegal
- **Expected Result:** Spell ceases to be mutating, enters battlefield as a creature normally.
- **Phase:** Phase 8
- **Ticket:** (same as 702.140a)

702.140c — On legal resolution, merges with target. Controller chooses top or bottom.

**ATOM-702.140c-001**
- **Rule:** 702.140c — Mutating creature merges with target; controller chooses ordering
- **Mechanism:** Merge (rule 729) + top/bottom choice
- **Minimal Board:** Player casts mutate creature targeting a 3/3 non-Human. Mutate card is a 4/4.
- **Action:** Mutating spell resolves; player puts mutate card on top
- **Expected Result:** Resulting permanent is one object represented by two cards. Characteristics from topmost card: 4/4. Has abilities of both cards.
- **Phase:** Phase 8
- **Ticket:** (same as 702.140a)

702.140d — "Whenever a creature mutates" trigger. DEFERRED — Phase 7.

702.140e — Mutated permanent has all abilities of each component. Its other characteristics are from the topmost card/token.

**ATOM-702.140e-001**
- **Rule:** 702.140e — Mutated permanent has all abilities from all components; other characteristics from top
- **Mechanism:** Merged permanent characteristics
- **Minimal Board:** A 2/2 with flying is on bottom. A 4/4 with trample is on top.
- **Action:** Check the mutated permanent's characteristics
- **Expected Result:** P/T is 4/4 (from top), has both flying and trample (all abilities from both).
- **Phase:** Phase 8
- **Ticket:** (same as 702.140a)

702.140f — Effects referring to mutating creature spell refer to mutated permanent. PURE-DEF.

--- End of Chunk 4 ---

## Chunk 5: 702.141–702.155 (Encore through Read Ahead)

### 702.141 Encore

**Classification: DEFERRED — Phase 7 (triggered/activated) + Phase 9 (multiplayer)**

702.141a — "[Cost], Exile from GY: For each opponent, create a token copy that attacks that opponent if able. Tokens gain haste. Sacrifice at next end step. Sorcery speed."

**ATOM-702.141a-001**
- **Rule:** 702.141a — Encore creates token copies per opponent, each must attack that opponent
- **Mechanism:** GY activated ability + exile self + per-opponent token creation + forced attack + haste + delayed sacrifice
- **Minimal Board:** 3-player game. Player has a 3/3 creature with "Encore {4}{B}" in graveyard.
- **Action:** Activate encore
- **Expected Result:** Card exiled. 2 token copies created (one per opponent). Each has haste and must attack the designated opponent. Sacrificed at beginning of next end step.
- **Phase:** Phase 7 + Phase 8 + Phase 9 (multiplayer)
- **Ticket:** NEW — Encore keyword (GY activated, per-opponent tokens, forced attack, sacrifice)
- **Dependencies:** T19 (GY activation), Phase 9 (multiplayer), CreateToken + Copy primitive

---

### 702.142 Boast

**Classification: DEFERRED — Phase 8 (activated ability with attack condition)**

702.142a — "[Cost]: [Effect]. Activate only if this creature attacked this turn and only once each turn."

**ATOM-702.142a-001**
- **Rule:** 702.142a — Boast: activated ability usable only if creature attacked this turn, once per turn
- **Mechanism:** Activated ability with dual restriction (attacked this turn + once per turn)
- **Minimal Board:** Player controls a creature with boast that attacked this turn
- **Action:** Activate boast ability
- **Expected Result:** Ability resolves. Second activation this turn is illegal.
- **Phase:** Phase 8
- **Ticket:** NEW — Boast keyword (attack-conditioned, once-per-turn activated)
- **Dependencies:** T19 (ActivationRestriction), D16 (per-turn attack tracker)

**ATOM-702.142a-002**
- **Rule:** 702.142a — Boast cannot activate if creature did not attack this turn
- **Mechanism:** Attack condition check
- **Minimal Board:** Player controls a creature with boast that did NOT attack this turn
- **Action:** Attempt to activate boast
- **Expected Result:** Activation is illegal (creature didn't attack this turn).
- **Phase:** Phase 8
- **Ticket:** (same as above)

702.142b — "Creature boasting" = its boast ability being activated. PURE-DEF.

---

### 702.143 Foretell

**Classification: DEFERRED — Phase 5-Pre T17 (alternative cost) + Phase 8**

702.143a — Pay {2} and exile from hand face down (special action). Cast later for foretell cost.

**ATOM-702.143a-001**
- **Rule:** 702.143a — Foretell: pay {2} to exile from hand face down; cast later for foretell cost
- **Mechanism:** Special action (exile face down) + alternative cost on later turn
- **Minimal Board:** Player has a card with "Foretell {W}" in hand during their turn
- **Action:** Player pays {2}, exiles the card face down
- **Expected Result:** Card is exiled face down. On a subsequent turn, player may cast it for {W} instead of its normal mana cost.
- **Phase:** Phase 5-Pre T17 + Phase 8
- **Ticket:** T17 (alternative cost) + NEW — Foretell keyword
- **Dependencies:** T17 (alt cost), Special action infrastructure, Face-down exile zone

**ATOM-702.143a-002**
- **Rule:** 702.143a — Foretold card cannot be cast on the same turn it was foretold
- **Mechanism:** Turn restriction on foretold cast
- **Minimal Board:** Player foretells a card this turn
- **Action:** Player attempts to cast it for foretell cost this same turn
- **Expected Result:** Cast is illegal. Must wait until after the current turn ends.
- **Phase:** Phase 8
- **Ticket:** (same as above)

702.143b — Foretelling is a special action, doesn't use stack. PURE-DEF.

702.143c — "Foretelling" and "foretold" definitions. PURE-DEF.

702.143d — Card in exile "becomes foretold" by an effect. DEFERRED — Phase 8 (niche).

702.143e — Multiple foretold cards must be distinguishable. PURE-DEF (tracking requirement).

702.143f — Face-down foretold cards revealed when player leaves or game ends. PURE-DEF.

---

### 702.144 Demonstrate

**Classification: DEFERRED — Phase 7 (triggered ability)**

702.144a — "When you cast this spell, you may copy it. If you copy, choose an opponent who also copies it."

**ATOM-702.144a-001**
- **Rule:** 702.144a — Demonstrate: copy spell for you; if you do, an opponent also copies it
- **Mechanism:** Cast trigger + optional copy + forced opponent copy
- **Minimal Board:** Player casts a sorcery with demonstrate
- **Action:** Player chooses to copy. Chooses an opponent.
- **Expected Result:** Player gets a copy (may choose new targets). Opponent also gets a copy (may choose new targets). Both copies go on stack above original.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Demonstrate keyword (cast trigger, mutual copy)
- **Dependencies:** Phase 7 (triggered abilities), CopySpell primitive

---

### 702.145 Daybound and Nightbound

**Classification: DEFERRED — Phase 9 (DFC)**

702.145a — Found on opposite faces of DFCs. PURE-DEF.

702.145b — Daybound: three static abilities controlling DFC transformation with day/night.

702.145c — Front-face-up daybound permanent at night → transform immediately. DEFERRED — Phase 9.

702.145d — Controlling a daybound permanent when neither day nor night → becomes day. DEFERRED — Phase 9.

702.145e — Nightbound: two static abilities controlling back-face transformation. DEFERRED — Phase 9.

702.145f — Back-face-up nightbound permanent during day → transform immediately. DEFERRED — Phase 9.

702.145g — Nightbound with no daybound permanents → becomes night. DEFERRED — Phase 9.

All sub-rules DEFERRED — Phase 9 (DFC + day/night system, rule 730).

---

### 702.146 Disturb

**Classification: DEFERRED — Phase 9 (DFC)**

702.146a — "You may cast this card transformed from your graveyard by paying [cost]."

702.146b — Resolving disturb spell enters back face up.

All sub-rules DEFERRED — Phase 9 (DFC casting from graveyard).

---

### 702.147 Decayed

**Classification: DEFERRED — Phase 7 (triggered ability) + Phase 5 (static blocking restriction)**

702.147a — "This creature can't block" + "When this creature attacks, sacrifice it at end of combat."

**ATOM-702.147a-001**
- **Rule:** 702.147a — Decayed: creature can't block; when it attacks, sacrifice at end of combat
- **Mechanism:** Static blocking restriction + attack trigger (delayed sacrifice)
- **Minimal Board:** Player controls a 2/2 Zombie token with decayed
- **Action:** Creature attacks
- **Expected Result:** Attack is legal. Trigger: sacrifice at end of combat. Creature CANNOT be declared as a blocker.
- **Phase:** Phase 5 (blocking restriction) + Phase 7 (triggered) + Phase 8
- **Ticket:** NEW — Decayed keyword (can't block + attack → sacrifice at EOC)
- **Dependencies:** Phase 5 (static restriction), Phase 7 (triggered abilities)

---

### 702.148 Cleave

**Classification: DEFERRED — Phase 5 (text-changing effect) + Phase 8**

702.148a — Alt cost + "remove all text within square brackets." Text-changing effect per rule 612.

**ATOM-702.148a-001**
- **Rule:** 702.148a — Cleave: alt cost removes bracketed text from spell
- **Mechanism:** Alternative cost + text-changing effect (remove [] content)
- **Minimal Board:** Player casts "Destroy target [nonland] permanent" with cleave cost
- **Action:** Player pays cleave cost
- **Expected Result:** Spell text becomes "Destroy target permanent" (the word "nonland" removed). Can target any permanent.
- **Phase:** Phase 5 (text-changing effects) + Phase 8
- **Ticket:** NEW — Cleave keyword (alt cost + bracket text removal)
- **Dependencies:** T17 (alternative cost), Phase 5 (text-changing effects, rule 612)

702.148b — Cleave's second ability is text-changing effect. PURE-DEF (cross-ref).

**Tag: TEXT-CHANGING-EFFECT cleave** — Cleave (702.148) removes bracketed text when the cleave cost is paid. Same text-changing-effect infrastructure as Overload (702.96) and Splice (702.47). Cross-ref rule 612.

---

### 702.149 Training

**Classification: DEFERRED — Phase 7 (triggered ability)**

702.149a — "Whenever this creature and at least one other creature with power greater than this creature's power attack, put a +1/+1 counter on this creature."

**ATOM-702.149a-001**
- **Rule:** 702.149a — Training: triggers when attacking alongside a creature with greater power
- **Mechanism:** Attack trigger with co-attacker power comparison + counter placement
- **Minimal Board:** Creature A (1/1, training) and creature B (3/3). Both attack.
- **Action:** Both declared as attackers
- **Expected Result:** Training triggers (B's power 3 > A's power 1). +1/+1 counter placed on A (becomes 2/2).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Training keyword (co-attacker power trigger, +1/+1 counter)
- **Dependencies:** Phase 7 (triggered abilities), T01 (counters)

702.149b — Multiple instances trigger separately. DEFERRED — Phase 7.

702.149c — "When this creature trains" = training ability puts counters on it. PURE-DEF.

---

### 702.150 Compleated

**Classification: DEFERRED — Phase 8 (Phyrexian mana + loyalty modification)**

702.150a — "If this permanent would enter with loyalty counters and player paid life for Phyrexian mana symbols, it enters with that many counters minus two per Phyrexian symbol paid with life."

**ATOM-702.150a-001**
- **Rule:** 702.150a — Compleated reduces starting loyalty by 2 per Phyrexian mana paid with life
- **Mechanism:** ETB replacement modifying loyalty counter count based on casting cost payment
- **Minimal Board:** Player casts a planeswalker with compleated. Mana cost includes {G/P}{G/P}. Player pays 4 life (both Phyrexian). Base loyalty is 5.
- **Action:** Planeswalker enters
- **Expected Result:** Enters with 5 − 4 = 1 loyalty counter (2 per Phyrexian symbol paid with life, 2 symbols).
- **Phase:** Phase 8
- **Ticket:** NEW — Compleated keyword (Phyrexian mana loyalty reduction)
- **Dependencies:** Phase 6 (replacement), Phyrexian mana in ManaCost

---

### 702.151 Reconfigure

**Classification: DEFERRED — Phase 8. PRIORITY keyword.**

702.151a — Two activated abilities: "[Cost]: Attach to another target creature. Sorcery." and "[Cost]: Unattach. Sorcery."

**ATOM-702.151a-001**
- **Rule:** 702.151a — Reconfigure: attach to creature or unattach, sorcery speed
- **Mechanism:** Dual activated abilities (attach/unattach) with sorcery restriction
- **Minimal Board:** Player controls an Equipment creature with "Reconfigure {3}" and another creature
- **Action:** Activate reconfigure to attach to the other creature
- **Expected Result:** Equipment attaches to the creature. Equipment stops being a creature (per 702.151b).
- **Phase:** Phase 8
- **Ticket:** NEW — Reconfigure keyword (dual activate: attach/unattach, stops being creature)
- **Dependencies:** T04 (attachment tracking), T19 (sorcery-speed activation)

702.151b — Attached reconfigure Equipment stops being a creature until unattached.

**ATOM-702.151b-001**
- **Rule:** 702.151b — Reconfigure Equipment stops being a creature while attached
- **Mechanism:** Type-loss while attached
- **Minimal Board:** A 2/2 Equipment creature with reconfigure is attached to a 3/3
- **Action:** Check the Equipment's types
- **Expected Result:** Equipment is NOT a creature while attached. When unattached, it becomes a creature again.
- **Phase:** Phase 5 (type changing) + Phase 8
- **Ticket:** (same as 702.151a)

---

### 702.152 Blitz

**Classification: DEFERRED — Phase 8. PRIORITY keyword.**

702.152a — Three abilities: alt cost + delayed sacrifice at end step + "haste and 'When this permanent dies, draw a card'" while blitz cost was paid.

**ATOM-702.152a-001**
- **Rule:** 702.152a — Blitz: alt cost grants haste, dies-draw, sacrifice at end step
- **Mechanism:** Alternative cost + haste + dies trigger (draw) + delayed sacrifice
- **Minimal Board:** Player casts a 4/3 creature for its blitz cost
- **Action:** Creature enters
- **Expected Result:** Creature has haste and "When this dies, draw a card." At beginning of next end step, sacrifice it. If sacrificed, draw a card.
- **Phase:** Phase 8
- **Ticket:** NEW — Blitz keyword (alt cost, haste, dies-draw, delayed sacrifice)
- **Dependencies:** T17 (alternative cost), Phase 7 (delayed triggers, dies trigger)

**ATOM-702.152a-002**
- **Rule:** 702.152a — Creature cast normally (not for blitz) has no blitz effects
- **Mechanism:** Conditional abilities based on blitz payment
- **Minimal Board:** Player casts the same creature for its normal mana cost
- **Action:** Creature enters
- **Expected Result:** No haste, no dies-draw, no delayed sacrifice.
- **Phase:** Phase 8
- **Ticket:** (same as above)

702.152b — Multiple instances: only one may be used to cast; each instance tracks its own payment. PURE-DEF.

---

### 702.153 Casualty

**Classification: DEFERRED — Phase 7 (triggered ability) + Phase 8. PRIORITY keyword.**

702.153a — Additional cost: sacrifice creature with power ≥ N. Triggered: if casualty paid, copy the spell.

**ATOM-702.153a-001**
- **Rule:** 702.153a — Casualty N: sacrifice creature with power ≥ N as additional cost; if paid, copy spell
- **Mechanism:** Additional cost (sacrifice with power check) + cast trigger (copy if paid)
- **Minimal Board:** Player casts a sorcery with Casualty 2, controls a 3/3 creature
- **Action:** Player sacrifices the 3/3 (power 3 ≥ 2) as casualty cost
- **Expected Result:** Casualty trigger fires. Spell is copied. Player may choose new targets for the copy.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Casualty keyword (sacrifice additional cost, conditional copy)
- **Dependencies:** T17 (additional cost), Phase 7 (triggered abilities), CopySpell primitive

**ATOM-702.153a-002**
- **Rule:** 702.153a — Casualty: not paying the cost means no copy
- **Mechanism:** Optional additional cost check
- **Minimal Board:** Player casts a spell with Casualty 2, chooses NOT to sacrifice
- **Action:** Spell resolves
- **Expected Result:** No copy is made. Spell resolves normally once.
- **Phase:** Phase 8
- **Ticket:** (same as above)

702.153b — Multiple instances paid separately. PURE-DEF.

---

### 702.154 Enlist

**Classification: DEFERRED — Phase 7 (triggered ability) + Phase 8**

702.154a — "As this creature attacks, you may tap an untapped non-attacking creature with haste or continuously-controlled. When you do, this creature gets +X/+0 where X is tapped creature's power."

**ATOM-702.154a-001**
- **Rule:** 702.154a — Enlist: tap another creature on attack for +X/+0
- **Mechanism:** Optional attack cost (tap creature) + triggered ability (+X/+0)
- **Minimal Board:** Creature A (2/2, enlist) attacks. Creature B (3/3, has been controlled since turn start) doesn't attack.
- **Action:** Player taps B as enlist cost for A
- **Expected Result:** Enlist triggers. A gets +3/+0 until EOT (becomes 5/2).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Enlist keyword (tap-on-attack cost, power bonus trigger)
- **Dependencies:** Phase 7 (triggered abilities)

702.154b — Enlist static is optional attack cost; trigger is linked. PURE-DEF.

702.154c — "Enlists" definition. PURE-DEF.

702.154d — Multiple instances function independently. PURE-DEF.

---

### 702.155 Read Ahead

**Classification: DEFERRED — Phase 8 (Saga modification)**

702.155a — Saga chapter abilities can't trigger the turn it entered unless it has exactly that many lore counters.

702.155b — Choose starting lore counter count (1 to final chapter number) on ETB.

**ATOM-702.155b-001**
- **Rule:** 702.155b — Read ahead: choose starting lore counter count on Saga ETB
- **Mechanism:** ETB replacement (choose starting chapter)
- **Minimal Board:** Player casts a Saga with read ahead (final chapter III)
- **Action:** Player chooses to enter with 2 lore counters
- **Expected Result:** Saga enters with 2 lore counters. Chapter II ability doesn't trigger this turn (per 702.155a). On next turn's lore counter addition, Chapter III triggers.
- **Phase:** Phase 8
- **Ticket:** NEW — Read Ahead keyword (Saga starting chapter choice)
- **Dependencies:** Saga infrastructure (rule 714), T01 (counters)

**ATOM-702.155-002**
- **Rule:** 702.155 + ruling — Read Ahead: counter doubler modifies actual entry counters
- **Mechanism:** Replacement effect modifies lore counter count after chapter choice
- **Minimal Board:** Player controls a permanent with "If a permanent would enter with one or more counters, it enters with twice that many instead." Player casts a Read Ahead Saga, choosing chapter I (1 lore counter).
- **Action:** Saga enters
- **Expected Result:** Player chose 1 lore counter, but replacement doubles it to 2. Saga enters with 2 lore counters. ONLY the chapter II ability triggers (matching actual counter count), NOT chapters I and II. The "all chapters up to chosen" Read Ahead exception is demonstrated.
- **Phase:** Phase 6 (replacement) + Phase 8
- **Ticket:** (same as 702.155b)
- **Note:** This differs from normal saga ETB (which would trigger chapters I and II for 2 counters). Read Ahead specifically triggers only the chapter matching the actual counter count.

702.155c — Multiple instances redundant. PURE-DEF.

--- End of Chunk 5 ---

## Chunk 6: 702.156–702.170 (Ravenous through Plot)

### 702.156 Ravenous

**Classification: DEFERRED — Phase 7 (triggered ability) + Phase 8**

702.156a — "This creature enters with X +1/+1 counters. When it enters, if X is 5 or more, draw a card."

**ATOM-702.156a-001**
- **Rule:** 702.156a — Ravenous: enters with X counters; draw if X ≥ 5
- **Mechanism:** ETB counter placement (X-based) + conditional ETB trigger (X threshold)
- **Minimal Board:** Player casts a creature with ravenous where X=6
- **Action:** Creature enters
- **Expected Result:** Creature enters with 6 +1/+1 counters. Since X ≥ 5, draw-a-card trigger fires.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Ravenous keyword (X-based ETB counters + conditional draw)
- **Dependencies:** Phase 7 (triggered abilities), T01 (counters), X-cost infrastructure

**ATOM-702.156a-002**
- **Rule:** 702.156a — Ravenous: no draw if X < 5
- **Mechanism:** Threshold check fails
- **Minimal Board:** Player casts a ravenous creature where X=3
- **Action:** Creature enters with 3 counters
- **Expected Result:** 3 +1/+1 counters placed. Draw trigger does NOT fire (X < 5).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as above)

---

### 702.157 Squad

**Classification: DEFERRED — Phase 7 (triggered ability) + Phase 8**

702.157a — Optional additional cost (any number of times). "When this creature enters, create tokens equal to number of times squad cost was paid."

**ATOM-702.157a-001**
- **Rule:** 702.157a — Squad: pay additional cost N times; create N token copies on ETB
- **Mechanism:** Repeatable additional cost + ETB trigger (token copies)
- **Minimal Board:** Player casts a 2/2 creature with "Squad {2}" and pays squad cost twice
- **Action:** Creature enters
- **Expected Result:** ETB trigger creates 2 token copies of the creature.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Squad keyword (repeatable additional cost, ETB token copies)
- **Dependencies:** T17 (additional cost), Phase 7 (triggered abilities), CreateToken + Copy primitive

---

### 702.158 Space Sculptor

**Classification: OUT-OF-SCOPE (Un-sets / Unfinity attraction mechanic)**

702.158a–b — Relates to Unfinity space-themed mechanics. Permanently out of scope.

---

### 702.159 Visit

**Classification: OUT-OF-SCOPE (Un-sets / Unfinity attraction mechanic)**

702.159a–c — Relates to Unfinity attraction visits. Permanently out of scope.

---

### 702.160 Prototype

**Classification: DEFERRED — Phase 9 (DFC-adjacent / alternate characteristics)**

702.160a — "You may cast this spell with different mana cost, color, power, and toughness." Applies on stack and battlefield; other zones use normal characteristics.

**ATOM-702.160a-001**
- **Rule:** 702.160a — Prototype: cast with alternate characteristics (cost, color, P/T)
- **Mechanism:** Alternative casting mode with different characteristics on stack/battlefield
- **Minimal Board:** Player casts a creature with "Prototype {1}{R} — 2/2" (normal: {5} 4/4 colorless)
- **Action:** Player casts for prototype cost {1}{R}
- **Expected Result:** On stack and on battlefield: creature is red 2/2 with mana cost {1}{R}. In GY/hand/library: characteristics are normal (colorless, {5}, 4/4).
- **Phase:** Phase 9
- **Ticket:** NEW — Prototype keyword (alternate characteristics on stack/battlefield)

702.160b — Prototype permanent's copiable values use prototype characteristics. DEFERRED — Phase 9.

702.160c — Object not on stack or battlefield uses normal characteristics. PURE-DEF (zone boundary).

---

### 702.161 Living Metal

**Classification: DEFERRED — Phase 5 (continuous effect) + Phase 8**

702.161a — "As long as it's your turn, this permanent is also a creature."

**ATOM-702.161a-001**
- **Rule:** 702.161a — Living metal: permanent is also a creature during your turn
- **Mechanism:** Continuous effect (type addition, turn-conditional)
- **Minimal Board:** Player controls a Vehicle with living metal. It is the player's turn.
- **Action:** Check the permanent's types
- **Expected Result:** Permanent is an artifact creature (Vehicle) during controller's turn. On opponent's turn, it is just an artifact (Vehicle).
- **Phase:** Phase 5 (continuous effect, type change) + Phase 8
- **Ticket:** NEW — Living Metal keyword (turn-conditional creature type)
- **Dependencies:** Phase 5 (type-changing continuous effects)

---

### 702.162 More Than Meets the Eye

**Classification: DEFERRED — Phase 9 (DFC / Transformers set)**

702.162a — "You may cast this card converted for [cost]." Enters back face up (transformed).

All sub-rules DEFERRED — Phase 9 (DFC casting transformed).

---

### 702.163 For Mirrodin!

**Classification: DEFERRED — Phase 7 (triggered ability) + Phase 8**

702.163a — "When this Equipment enters, create a 2/2 red Rebel creature token, then attach this to it."

**ATOM-702.163a-001**
- **Rule:** 702.163a — For Mirrodin!: ETB creates 2/2 Rebel token, auto-attaches Equipment
- **Mechanism:** ETB triggered ability + token creation + equipment attachment
- **Minimal Board:** Player casts an Equipment with "For Mirrodin!"
- **Action:** Equipment enters
- **Expected Result:** A 2/2 red Rebel creature token is created. Equipment is attached to it.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — For Mirrodin! keyword (ETB trigger, token, auto-attach)
- **Dependencies:** Phase 7 (triggered abilities), CreateToken primitive, T04 (attachment)

---

### 702.164 Toxic

**Classification: DEFERRED — Phase 8 (combat damage modifier)**

702.164a — "Whenever this creature deals combat damage to a player, that player gets N poison counters."

**ATOM-702.164a-001**
- **Rule:** 702.164a — Toxic N: player gets N poison counters when dealt combat damage
- **Mechanism:** Combat damage trigger + poison counter placement (additive, not replacing damage)
- **Minimal Board:** A 2/2 creature with Toxic 1 deals combat damage to opponent
- **Action:** Combat damage resolves
- **Expected Result:** Opponent takes 2 damage (normal) AND gets 1 poison counter. (Unlike infect, damage is NOT replaced.)
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Toxic keyword (combat damage trigger, additive poison)
- **Dependencies:** Phase 7 (triggered abilities), T21c (poison counter tracking)

702.164b — Multiple instances: each triggers separately. Total poison = sum of all toxic values.

**ATOM-702.164b-001**
- **Rule:** 702.164b — Multiple toxic instances each trigger separately
- **Mechanism:** Stacking triggered abilities
- **Minimal Board:** A creature with Toxic 1 and Toxic 2 deals combat damage
- **Action:** Combat damage resolves
- **Expected Result:** Two triggers: opponent gets 1 + 2 = 3 poison counters total.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.164a)

---

### 702.165 Backup

**Classification: DEFERRED — Phase 7 (triggered ability). PRIORITY keyword.**

702.165a — "When this creature enters, put N +1/+1 counters on target creature. If it's another creature, it gains [abilities] until end of turn."

**ATOM-702.165a-001**
- **Rule:** 702.165a — Backup N: ETB puts N counters on target; if other creature, grants abilities until EOT
- **Mechanism:** ETB trigger + targeted counter placement + conditional ability grant
- **Minimal Board:** Creature A (2/2, Backup 1, has flying) enters. Controls creature B (3/3).
- **Action:** A enters; backup targets B
- **Expected Result:** 1 +1/+1 counter placed on B (becomes 4/4). B gains flying until end of turn (because target was another creature).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Backup keyword (ETB counters + conditional ability grant)
- **Dependencies:** Phase 7 (triggered abilities), T01 (counters), Phase 5 (continuous effect until EOT)

**ATOM-702.165a-002**
- **Rule:** 702.165a — Backup targeting self: counters placed but no ability grant
- **Mechanism:** Self-target path (no "another creature" clause met)
- **Minimal Board:** Creature A (2/2, Backup 1, has flying) enters. No other creatures.
- **Action:** A enters; backup targets A itself
- **Expected Result:** 1 +1/+1 counter placed on A (becomes 3/3). A already has flying; no additional ability grant text applies.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as above)

**ATOM-702.165c-001**
- **Rule:** 702.165c — Backup: only printed abilities are granted, not gained ones
- **Mechanism:** Granted abilities vs printed abilities distinction
- **Minimal Board:** Creature A with "Backup 1" and printed "Flying". A also has been granted "Trample" by a continuous effect. Creature B enters.
- **Action:** Backup triggers, targeting B
- **Expected Result:** B gets +1/+1 counter AND flying (printed after backup) until EOT. B does NOT get trample (granted, not printed).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.165a)

**ATOM-702.165d-001**
- **Rule:** 702.165d — Backup: granted abilities determined when trigger goes on stack, not on resolution
- **Mechanism:** Lock-in at trigger time
- **Minimal Board:** Creature A has "Backup 1" with printed "Flying" and "Deathtouch". A enters, backup triggers, goes on stack. In response, A loses deathtouch.
- **Action:** Backup trigger resolves targeting B
- **Expected Result:** B gets +1/+1 counter AND both flying and deathtouch until EOT. Deathtouch was locked in when the trigger went on the stack.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.165a)

702.165b — Multiple instances trigger separately. DEFERRED — Phase 7.

**Tag: TRIGGERED-ABILITY-INFRA 603.3c** — "No legal targets → trigger doesn't go on stack" is a general rule, not keyword-specific. Applies to Mentor, Backup, Fabricate, etc.

**Tag: ARCHITECTURE-NOTE printed-vs-granted-abilities** — Multiple rules require distinguishing "printed" (innate) abilities from "granted" abilities: (1) Backup (702.165c) grants only printed abilities after the backup keyword. (2) Copy effects (rule 707.2) copy only copiable values — granted abilities are NOT copiable values, so a Clone copying a creature that was granted flying by a continuous effect does NOT get flying. In the current engine design, this distinction is implicit: printed abilities live in `CardData.abilities`/`CardData.keywords` (immutable), while granted abilities are `ContinuousEffect` entries with `AddAbility` modifications (Layer 6). Both Backup and copy effects should read `CardData` directly for printed abilities, NOT `compute_characteristics()`. **Caveat:** If Alchemy "perpetually gains" support is ever added, perpetual effects modify the card itself (not via Layer 6), blurring the printed/granted line. That would require an `AbilityOrigin` enum (`Printed`, `Perpetual`, `Granted(EffectId)`).

---

### 702.166 Bargain

**Classification: DEFERRED — Phase 8. PRIORITY keyword.**

702.166a — Optional additional cost: sacrifice an artifact, enchantment, or token.

**ATOM-702.166a-001**
- **Rule:** 702.166a — Bargain: optional additional cost to sacrifice an artifact, enchantment, or token
- **Mechanism:** Optional additional cost (sacrifice with type restriction)
- **Minimal Board:** Player casts a spell with bargain, controls an artifact token
- **Action:** Player sacrifices the artifact token as bargain cost
- **Expected Result:** Bargain is paid. "If this spell was bargained" condition is true for spell's abilities.
- **Phase:** Phase 8
- **Ticket:** NEW — Bargain keyword (optional sacrifice additional cost)
- **Dependencies:** T17 (additional cost)

702.166b — "Bargained" definition. PURE-DEF.

---

### 702.167 Craft

**Classification: DEFERRED — Phase 8 (activated ability + exile transformation)**

702.167a — "[Cost], Exile this + [materials from your graveyard/battlefield]: Return this card transformed. Sorcery."

**ATOM-702.167a-001**
- **Rule:** 702.167a — Craft: exile self + materials; return transformed
- **Mechanism:** Activated ability (exile self + exile materials) + DFC transformation
- **Minimal Board:** Player controls an artifact with "Craft with artifact {2}" and another artifact on battlefield
- **Action:** Pay {2}, exile the craft artifact and another artifact
- **Expected Result:** Craft artifact returns to battlefield transformed (back face up).
- **Phase:** Phase 8 + Phase 9 (DFC)
- **Ticket:** NEW — Craft keyword (exile self + materials, return transformed)
- **Dependencies:** Phase 9 (DFC), T19 (activation)

---

### 702.168 Disguise

**Classification: DEFERRED — Phase 8 (face-down casting)**

702.168a — "You may cast this card face down as a 2/2 creature with ward {2} for {3}."

**ATOM-702.168a-001**
- **Rule:** 702.168a — Disguise: cast face down as 2/2 with ward {2} for {3}
- **Mechanism:** Face-down casting (morph variant) + ward {2} grant
- **Minimal Board:** Player has a creature with "Disguise {1}{W}" in hand, has {3} available
- **Action:** Player casts it face down for {3}
- **Expected Result:** A 2/2 face-down creature enters the battlefield with ward {2}. No characteristics from front face visible.
- **Phase:** Phase 8
- **Ticket:** NEW — Disguise keyword (face-down cast, ward {2}, turn face up for disguise cost)
- **Dependencies:** Face-down/morph infrastructure (rule 708), Ward keyword

702.168b — Turn face up for disguise cost (special action). TESTABLE.

**ATOM-702.168b-001**
- **Rule:** 702.168b — Disguised creature can be turned face up by paying disguise cost
- **Mechanism:** Special action (turn face up)
- **Minimal Board:** Player controls a face-down disguised creature, has mana for disguise cost
- **Action:** Player pays disguise cost and turns creature face up
- **Expected Result:** Creature is turned face up, revealing its front face characteristics. This is a special action (doesn't use stack).
- **Phase:** Phase 8
- **Ticket:** (same as 702.168a)

702.168c — See rule 708 (face-down spells/permanents). PURE-DEF.

**Tag: SHARED-BEHAVIOR morph-disguise** — Disguise (702.168) and Morph (702.36, Session 7B) share face-down casting infrastructure. Key differences: Morph grants no ward; Disguise grants ward {2}. Both use the same "cast face down for {3}, turn face up for [cost]" pattern. Implementation should share the face-down casting, 2/2 characteristics, and turn-face-up special action. Disguise adds ward {2} as a modifier.

---

### 702.169 Solved

**Classification: DEFERRED — Phase 8 (Case enchantment support)**

702.169a — Definition of "solved" for Cases: when all requirements of the case are met.

**ATOM-702.169a-001**
- **Rule:** 702.169a — A Case becomes solved when its "To solve" condition is met
- **Mechanism:** Designation based on condition check
- **Minimal Board:** Player controls a Case enchantment with "To solve — You control 3 or more creatures." Player controls 3 creatures.
- **Action:** Condition is checked
- **Expected Result:** Case becomes solved. Its "Solved — [effect]" ability becomes active.
- **Phase:** Phase 8
- **Ticket:** NEW — Solved/Case keyword (condition-based designation, unlocks ability)
- **Dependencies:** Case enchantment infrastructure

702.169b — "Solved" is a designation. PURE-DEF.

702.169c — Continuous effects reapplied after becoming solved. PURE-DEF (process ordering).

---

### 702.170 Plot

**Classification: DEFERRED — Phase 8. PRIORITY keyword.**

702.170a — Pay plot cost, exile from hand face up (sorcery speed special action). Cast on a future turn without paying mana cost.

**ATOM-702.170a-001**
- **Rule:** 702.170a — Plot: pay plot cost to exile face up; cast free on future turn
- **Mechanism:** Special action (exile face up) + future free cast (sorcery speed)
- **Minimal Board:** Player has a creature with "Plot {2}{R}" in hand during main phase
- **Action:** Player pays {2}{R}, exiles the card face up
- **Expected Result:** Card is exiled face up ("plotted"). On a future turn, player may cast it without paying its mana cost (sorcery speed).
- **Phase:** Phase 8
- **Ticket:** NEW — Plot keyword (special action exile, future free cast)
- **Dependencies:** T17 (alternative cost: free cast), Special action infrastructure

**ATOM-702.170a-002**
- **Rule:** 702.170a — Plotted card cannot be cast on the same turn it was plotted
- **Mechanism:** Turn restriction
- **Minimal Board:** Player plots a card this turn
- **Action:** Attempts to cast it free this same turn
- **Expected Result:** Cast is illegal. Must wait until a future turn.
- **Phase:** Phase 8
- **Ticket:** (same as above)

702.170b — Plotting is a special action. PURE-DEF.

702.170c — "Plotted" definition. PURE-DEF.

702.170d — A plotted card may be cast without paying its mana cost. TESTABLE (covered by ATOM-702.170a-001).

702.170e — A plotted card may only be cast at sorcery speed. TESTABLE (covered by ATOM-702.170a-001 expected result).

702.170f — If a plotted card leaves exile, it's a new object (loses "plotted" status). TESTABLE.

**ATOM-702.170f-001**
- **Rule:** 702.170f — Plotted card that leaves exile and returns is no longer plotted
- **Mechanism:** Zone-change epoch resets plotted designation
- **Minimal Board:** A card was plotted (exiled face up). An effect returns it to hand, then another effect exiles it again.
- **Action:** Player attempts to cast it free from exile
- **Expected Result:** Illegal. The re-exiled card is a new game object; it's not "plotted."
- **Phase:** Phase 8
- **Ticket:** (same as 702.170a)
- **Dependencies:** Rule 400.7 (zone-change epoch)

**Tag: ARCHITECTURE-CONCERN special-action-mod** — Plot and Foretell (702.143) both use special actions that involve cost payment + exile. The cost modification pipeline (T17) needs to extend to special actions, not just casting costs.

--- End of Chunk 6 ---

## Chunk 7: 702.171–702.190 (Saddle through Sneak)

### 702.171 Saddle

**Classification: DEFERRED — Phase 8 (activated ability, Vehicle variant)**

702.171a — "Tap any number of other untapped creatures you control with total power N or greater: This Mount becomes saddled until end of turn. Sorcery speed."

**ATOM-702.171a-001**
- **Rule:** 702.171a — Saddle N: tap creatures with total power ≥ N to saddle this Mount
- **Mechanism:** Activated ability (tap creatures as cost) + designation until EOT + sorcery restriction
- **Minimal Board:** Player controls a Mount creature with Saddle 2 and a 3/3 creature
- **Action:** Tap the 3/3 to saddle the Mount
- **Expected Result:** Mount becomes "saddled" until end of turn. Saddled status enables conditional abilities.
- **Phase:** Phase 8
- **Ticket:** NEW — Saddle keyword (tap-creatures cost, saddled designation)
- **Dependencies:** T19 (sorcery-speed activation, tap-other-creatures cost), D17 (designations)

702.171b — "Saddles a Mount" = tapped to pay saddle cost. PURE-DEF.

702.171c — "Can't saddle Mounts" restriction. TESTABLE (same pattern as 702.122c crew restriction).

**Tag: SHARED-BEHAVIOR crew-saddle** — Crew (702.122) and Saddle (702.171) share a "tap creatures with total power ≥ N" cost pattern. Key differences: Crew makes Vehicle an artifact creature until EOT. Saddle gives Mount the "saddled" designation until EOT. Implementation should share a `tap_creatures_for_power(n, permanent_filter)` helper for the cost, with different effects on the permanent.

---

### 702.172 Spree

**Classification: DEFERRED — Phase 8 (additional costs per mode)**

702.172a — "Choose one or more additional costs. You must pay those costs as you cast. If you choose a cost, the corresponding mode is added."

**ATOM-702.172a-001**
- **Rule:** 702.172a — Spree: choose additional costs; each adds a mode to the spell
- **Mechanism:** Additional cost per mode (cost-selects-mode pattern)
- **Minimal Board:** Player casts a spell with spree. Three modes available with costs +{R}, +{1}, +{W}.
- **Action:** Player chooses to pay +{R} and +{W}
- **Expected Result:** Spell has those two modes. Both effects execute on resolution. Mode associated with +{1} does not apply.
- **Phase:** Phase 8
- **Ticket:** NEW — Spree keyword (cost-selects-mode additional costs)
- **Dependencies:** T17 (additional cost), Modal spell infrastructure

**ATOM-702.172a-002**
- **Rule:** 702.172 — Spree: must choose at least one mode (and pay its cost)
- **Mechanism:** Modal casting constraint (minimum 1 mode)
- **Minimal Board:** Player casts a spell with spree modes.
- **Action:** Player attempts to cast without choosing any modes
- **Expected Result:** Illegal. At least one mode must be chosen.
- **Phase:** Phase 8
- **Ticket:** (same as 702.172a)

---

### 702.173 Freerunning

**Classification: DEFERRED — Phase 8 (conditional alternative cost)**

702.173a — "You may pay [cost] if you dealt combat damage to a player this turn with an Assassin or creature that entered this turn."

**ATOM-702.173a-001**
- **Rule:** 702.173a — Freerunning: alt cost if Assassin or new creature dealt combat damage this turn
- **Mechanism:** Conditional alternative cost (per-turn combat damage + creature type/ETB tracking)
- **Minimal Board:** Player dealt combat damage to opponent with an Assassin creature this turn. Casts a spell with "Freerunning {1}{B}" (normal cost {3}{B}{B}).
- **Action:** Player pays freerunning cost
- **Expected Result:** Spell cast for {1}{B}. Condition met (Assassin dealt combat damage).
- **Phase:** Phase 8
- **Ticket:** NEW — Freerunning keyword (conditional alt cost, Assassin/new-creature combat tracker)
- **Dependencies:** T17 (alternative cost), D16 (per-turn tracker)

**Tag: COMMANDER-INTERACTION freerunning** — Freerunning's condition includes "a commander you control dealt combat damage to a player this turn." Relevant for Phase 9 (Commander).

---

### 702.174 Gift

**Classification: DEFERRED — Phase 8 (two-ability keyword: opponent choice + benefit delivery)**

702.174a — Gift defines TWO abilities: (1) Static on stack: "As an additional cost, you may **choose an opponent**." (2) Either a static ability (instants/sorceries) or a triggered ability (permanents) that gives the chosen opponent a specific benefit. The additional cost is choosing an opponent, NOT the gift effect itself.

**ATOM-702.174a-001**
- **Rule:** 702.174a — Gift on instant/sorcery: choose opponent as additional cost; gift effect happens before other spell effects on resolution
- **Mechanism:** Static on stack (opponent choice) + static on resolution (benefit delivery, per 702.174j)
- **Minimal Board:** Player casts an instant with "Gift a card" (choose opponent → chosen opponent draws a card on resolution, before spell's other effects).
- **Action:** Player chooses opponent, spell resolves
- **Expected Result:** Chosen opponent draws a card (gift effect, happens FIRST per 702.174j). Then spell's other effects resolve.
- **Phase:** Phase 8
- **Ticket:** NEW — Gift keyword (opponent choice + benefit delivery)
- **Dependencies:** T17 (additional cost — opponent choice), Phase 7 (triggered for permanents)

**ATOM-702.174a-002**
- **Rule:** 702.174a — Gift on permanent: choose opponent as additional cost; gift effect is ETB trigger
- **Mechanism:** Static on stack (opponent choice) + ETB triggered ability (benefit delivery)
- **Minimal Board:** Player casts a creature with "Gift a Food" (choose opponent → when this creature enters, chosen opponent creates a Food token).
- **Action:** Player chooses opponent, creature enters
- **Expected Result:** ETB trigger fires. Chosen opponent creates a Food token.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.174a)

**ATOM-702.174a-003**
- **Rule:** 702.174a — Gift: declining to gift (no opponent chosen)
- **Mechanism:** Optional additional cost declined
- **Minimal Board:** Player casts a spell with gift. Chooses NOT to gift.
- **Action:** Player declines the gift cost
- **Expected Result:** No opponent chosen. Gift effect does not happen. "Gifted" condition is false for any conditional abilities on the spell.
- **Phase:** Phase 8
- **Ticket:** (same as 702.174a)

702.174b — "Gifted" definition: a spell is "gifted" if an opponent was chosen for its gift cost. PURE-DEF.

702.174c — "Whenever a player gives a gift" trigger condition. DEFERRED — Phase 7.

702.174d–i — Specific gift definitions (Gift a card, Gift a Food, Gift a tapped land, Gift N tokens, etc.). PURE-DEF (parameterized variants).

702.174j — On instants/sorceries, gift effect happens before other spell effects on resolution. TESTABLE (covered by ATOM-702.174a-001).

702.174k — "Gift promised" = an opponent was chosen but the gift hasn't been delivered yet (relevant for targeting). PURE-DEF.

702.174m — Target selection may be conditional on whether gift was promised. DEFERRED — Phase 8.

---

### 702.175 Offspring

**Classification: DEFERRED — Phase 7 (triggered ability) + Phase 8**

702.175a — "Offspring [cost]" means "You may pay an additional [cost] as you cast this spell" and "When this permanent enters, if its offspring cost was paid, create a token that's a copy of it, except it's 1/1."

**ATOM-702.175a-001**
- **Rule:** 702.175a — Offspring: pay additional [cost]; on ETB create a 1/1 token copy
- **Mechanism:** Additional cost + ETB trigger (conditional token creation, modified copy — P/T overridden to 1/1)
- **Minimal Board:** Player casts a 3/2 creature with "Offspring {2}", paying the additional {2}
- **Action:** Creature enters
- **Expected Result:** ETB trigger creates a token copy of the creature, except the token is 1/1 (retains all other copiable values including abilities).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Offspring keyword (additional cost, conditional 1/1 token copy)
- **Dependencies:** T17 (additional cost), Phase 7 (triggered), CreateToken + Copy primitive

702.175b — Multiple instances: each paid separately, each triggers based on its own payment. PURE-DEF.

---

### 702.176 Impending

**Classification: DEFERRED — Phase 5-Pre T17 + Phase 5 + Phase 7 + Phase 8**

702.176a — Impending represents FOUR abilities: (1) static on stack: "You may choose to pay [cost] rather than this spell's mana cost"; (2) static replacement: "If you chose to pay this permanent's impending cost, it enters with N time counters"; (3) static on battlefield: "As long as this permanent's impending cost was paid and it has a time counter on it, it's not a creature"; (4) triggered on battlefield: "At the beginning of your end step, if this permanent's impending cost was paid and it has a time counter on it, remove a time counter from it."

**ATOM-702.176a-001**
- **Rule:** 702.176a (ability 1+2) — Impending: alt cost → enters with N time counters
- **Mechanism:** Alternative cost + ETB replacement (counter placement)
- **Minimal Board:** Player casts a creature with "Impending 4—{1}{B}" for the impending cost
- **Action:** Permanent enters
- **Expected Result:** Permanent enters with 4 time counters on it.
- **Phase:** Phase 5-Pre T17 + Phase 6 (replacement)
- **Ticket:** NEW — Impending keyword (alt cost, time counters, not-creature, end-step removal)
- **Dependencies:** T17 (alt cost), T01 (counters), Phase 6 (replacement effect)

**ATOM-702.176a-002**
- **Rule:** 702.176a (ability 3) — Impending: not a creature while impending cost was paid and has time counters
- **Mechanism:** Static continuous effect (type restriction conditional on payment + counters)
- **Minimal Board:** Permanent entered via impending cost, currently has 2 time counters
- **Action:** Check permanent's types
- **Expected Result:** Permanent is NOT a creature. It retains all other types (e.g., enchantment).
- **Phase:** Phase 5 (continuous type-loss)
- **Ticket:** (same as above)

**ATOM-702.176a-003**
- **Rule:** 702.176a (ability 4) — Impending: at beginning of your end step, remove a time counter
- **Mechanism:** Triggered ability (end step, conditional on impending cost paid + has time counter)
- **Minimal Board:** Permanent entered via impending with 1 time counter remaining
- **Action:** Beginning of controller's end step
- **Expected Result:** Time counter removed. Permanent now has 0 time counters → becomes a creature (ability 3 no longer applies).
- **Phase:** Phase 7 (triggered)
- **Ticket:** (same as above)

**ATOM-702.176a-004**
- **Rule:** 702.176a — Impending: cast for normal cost → no time counters, is a creature immediately
- **Mechanism:** Normal cast path (impending cost not chosen)
- **Minimal Board:** Player casts the same card for its normal mana cost
- **Action:** Permanent enters
- **Expected Result:** No time counters. Permanent is a creature immediately. End step trigger does not fire (impending cost was not paid).
- **Phase:** Phase 8
- **Ticket:** (same as above)

---

### 702.177 Exhaust

**Classification: DEFERRED — Phase 8 (activate-only-once restriction)**

702.177a — An exhaust ability is a special kind of activated ability. "Exhaust — [Cost]: [Effect]" means "[Cost]: [Effect]. Activate only once."

**ATOM-702.177a-001**
- **Rule:** 702.177a — Exhaust: activated ability that can only be activated once (ever, not per turn)
- **Mechanism:** Activated ability with permanent "activate only once" restriction
- **Minimal Board:** Player controls a permanent with "Exhaust — {2}: Draw a card"
- **Action:** Activate the exhaust ability
- **Expected Result:** Ability resolves, player draws a card. The ability CANNOT be activated again for the rest of the game (not just this turn).
- **Phase:** Phase 8
- **Ticket:** NEW — Exhaust keyword (activate-only-once activated ability)
- **Dependencies:** T19 (activation restriction tracking)

**ATOM-702.177a-002**
- **Rule:** 702.177a — Exhaust: zone change resets the "activated once" restriction
- **Mechanism:** New object identity on zone change (rule 400.7)
- **Minimal Board:** Player controls a permanent with exhaust ability. Activates it. Permanent is bounced to hand. Player re-casts it.
- **Action:** Player activates the exhaust ability on the new permanent
- **Expected Result:** Activation is legal. The returned permanent is a new object; the exhaust restriction tracked on the old object does not carry over.
- **Phase:** Phase 8
- **Ticket:** (same as 702.177a)
- **Dependencies:** Rule 400.7 (zone-change epoch)

702.177b — An effect may allow actions "as long as you haven't activated an exhaust ability this turn." This means not even begun to activate one this turn. PURE-DEF (interaction clarification).

---

### 702.178 Max Speed

**Classification: DEFERRED — Phase 8 (speed system)**

702.178a — "Max speed — [Ability]" means "As long as your speed is 4, this object has '[Ability].'" See rule 702.179, "Start Your Engines!"

**ATOM-702.178a-001**
- **Rule:** 702.178a — Max speed grants ability conditional on player having speed 4
- **Mechanism:** Static ability (conditional on player speed value)
- **Minimal Board:** Player controls a permanent with "Max speed — This creature has flying." Player's speed is 4.
- **Action:** Check creature's abilities
- **Expected Result:** Creature has flying. If player's speed drops below 4, creature loses flying.
- **Phase:** Phase 8
- **Ticket:** NEW — Max Speed keyword (speed-conditional static ability)
- **Dependencies:** Speed system (702.179)

702.178b — If max speed grants an ability that states which zones it functions from, the max speed ability also functions from those zones. PURE-DEF (rule 113.6c cross-ref).

---

### 702.179 Start Your Engines!

**Classification: DEFERRED — Phase 8 (speed system — SBA + triggered)**

702.179a — Start your engines! is a static ability. If a player controls a permanent with start your engines! and that player has no speed, their speed becomes 1. This is a state-based action. See rule 704.

**ATOM-702.179a-001**
- **Rule:** 702.179a — Start your engines!: SBA sets player speed to 1 if they have no speed
- **Mechanism:** Static ability triggering a state-based action (player speed initialization)
- **Minimal Board:** Player controls a permanent with "Start your engines!" Player currently has no speed.
- **Action:** SBAs are checked
- **Expected Result:** Player's speed becomes 1.
- **Phase:** Phase 8
- **Ticket:** NEW — Start Your Engines! + Speed system (SBA, speed tracking, inherent trigger)
- **Dependencies:** SBA infrastructure (rule 704), Speed as player attribute

702.179b — Players do not have speed until a rule or effect sets it. PURE-DEF.

702.179c — If a player has no speed and is instructed to increase speed by N, their speed becomes N. BOUNDARY-DEF.

702.179d — Inherent triggered ability: "Whenever one or more opponents lose life during your turn, if your speed is less than 4, your speed increases by 1. This triggers only once each turn."

**ATOM-702.179d-001**
- **Rule:** 702.179d — Speed inherent trigger: opponents lose life during your turn → speed +1 (max 4, once/turn)
- **Mechanism:** Inherent triggered ability (no source, controlled by player with speed ≥ 1)
- **Minimal Board:** Player has speed 2. During player's turn, opponent loses 3 life.
- **Action:** Trigger fires
- **Expected Result:** Player's speed increases to 3. Even though opponent lost life multiple times, trigger fires only once this turn.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** (same as 702.179a)

702.179e — A player has "max speed" if their speed is 4. PURE-DEF.

702.179f — If a player has no speed, their speed is 0 for effects that refer to speed. PURE-DEF.

---

### 702.180 Harmonize

**Classification: DEFERRED — Phase 5-Pre T17 + Phase 8 (GY cast + creature-tap cost reduction)**

702.180a — Harmonize represents THREE static abilities: (1) "You may cast this card from your graveyard by paying [cost] and tapping up to one untapped creature you control rather than paying its mana cost"; (2) "If you cast this spell using its harmonize ability, its total cost is reduced by an amount of generic mana equal to the tapped creature's power"; (3) "If the harmonize cost was paid, exile this card instead of putting it anywhere else any time it would leave the stack."

**ATOM-702.180a-001**
- **Rule:** 702.180a (ability 1+2) — Harmonize: cast from GY for [cost], tap creature for generic reduction by its power
- **Mechanism:** Alt cost (zone-cast from GY) + optional tap-creature cost reduction
- **Minimal Board:** Player has a card with "Harmonize {1}{G}" in graveyard. Controls a 3/3 creature.
- **Action:** Player casts from GY paying harmonize cost, taps the 3/3 creature
- **Expected Result:** Total cost reduced by {3} (tapped creature's power). Player pays {1}{G} minus up to {3} generic reduction.
- **Phase:** Phase 5-Pre T17 + Phase 8
- **Ticket:** NEW — Harmonize keyword (GY cast, creature-tap cost reduction, exile on leave stack)
- **Dependencies:** T17 (alt cost + cost reduction), T19 (zone-cast: GY)

**ATOM-702.180a-002**
- **Rule:** 702.180a (ability 3) — Harmonize: exile instead of going anywhere else when leaving stack
- **Mechanism:** Replacement effect on zone change from stack
- **Minimal Board:** Player cast a spell using harmonize. Spell resolves.
- **Action:** Spell would go to graveyard
- **Expected Result:** Card is exiled instead of going to graveyard. Also exiled if countered.
- **Phase:** Phase 6 (replacement)
- **Ticket:** (same as above)

702.180b — Choose which creature to tap as you choose to pay harmonize cost (rule 601.2b), tap as you pay total cost. PURE-DEF (process ordering).

---

### 702.181 Mobilize

**Classification: DEFERRED — Phase 7 (triggered ability) + Phase 8**

702.181a — "Mobilize N" means "Whenever this creature attacks, create N 1/1 red Warrior creature tokens. Those tokens enter tapped and attacking. Sacrifice them at the beginning of the next end step."

**ATOM-702.181a-001**
- **Rule:** 702.181a — Mobilize N: attack trigger creates N 1/1 red Warrior tokens tapped and attacking, sacrifice at next end step
- **Mechanism:** Attack trigger + token creation (tapped, attacking) + delayed sacrifice
- **Minimal Board:** A creature with Mobilize 2 attacks
- **Action:** Creature attacks; mobilize triggers
- **Expected Result:** 2 1/1 red Warrior creature tokens are created tapped and attacking. At beginning of next end step, sacrifice them.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Mobilize keyword (attack trigger, Warrior tokens tapped+attacking, delayed sacrifice)
- **Dependencies:** Phase 7 (triggered), CreateToken primitive, "enters tapped and attacking" (rule 506.3a)

---

### 702.182 Job Select

**Classification: DEFERRED — Phase 7 (triggered ability) + Phase 8**

702.182a — "Job select" means "When this Equipment enters, create a 1/1 colorless Hero creature token, then attach this Equipment to it."

**ATOM-702.182a-001**
- **Rule:** 702.182a — Job select: ETB creates 1/1 Hero token + auto-attaches Equipment
- **Mechanism:** ETB triggered ability + token creation + equipment attachment
- **Minimal Board:** Player casts an Equipment with "Job select"
- **Action:** Equipment enters the battlefield
- **Expected Result:** A 1/1 colorless Hero creature token is created. Equipment is attached to it.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Job Select keyword (ETB trigger, Hero token, auto-attach)
- **Dependencies:** Phase 7 (triggered), CreateToken primitive, T04 (attachment)

**Tag: SHARED-BEHAVIOR for-mirrodin-job-select** — For Mirrodin! (702.163) and Job Select (702.182) share the "ETB creates a token + auto-attaches this Equipment to it" pattern. For Mirrodin! creates a 2/2 red Rebel token. Job Select creates a 1/1 colorless Hero token. Implementation should share a `create_token_and_attach(token_def, equipment_id)` helper.

---

### 702.183 Tiered

**Classification: DEFERRED — Phase 8 (modal + additional cost)**

702.183a — Tiered is a static ability on some modal spells. "Choose one. As an additional cost to cast this spell, pay the cost associated with that mode."

**ATOM-702.183a-001**
- **Rule:** 702.183a — Tiered: choose one mode; pay that mode's associated additional cost
- **Mechanism:** Modal spell where each mode has a different additional cost
- **Minimal Board:** Player casts a tiered spell with modes: Mode 1 (additional {1}: deal 2 damage), Mode 2 (additional {3}: deal 4 damage)
- **Action:** Player chooses Mode 2, pays the spell's mana cost + {3} additional
- **Expected Result:** Mode 2 resolves (deal 4 damage). Mode 1 does not apply.
- **Phase:** Phase 8
- **Ticket:** NEW — Tiered keyword (modal + per-mode additional cost)
- **Dependencies:** T17 (additional cost), Modal spell infrastructure (rule 700.2)

---

### 702.184 Station

**Classification: DEFERRED — Phase 8 (activated ability + nonstandard layout)**

702.184a — "Station" means "Tap another untapped creature you control: Put a number of charge counters on this permanent equal to the tapped creature's power. Activate only as a sorcery."

**ATOM-702.184a-001**
- **Rule:** 702.184a — Station: tap creature → charge counters equal to its power
- **Mechanism:** Activated ability (tap-creature cost, charge counters, sorcery restriction)
- **Minimal Board:** Player controls a station permanent and a 4/4 creature
- **Action:** Activate station, tapping the 4/4
- **Expected Result:** 4 charge counters placed on the station permanent (equal to tapped creature's power).
- **Phase:** Phase 8
- **Ticket:** NEW — Station keyword (tap-creature activated, charge counters, sorcery speed)
- **Dependencies:** T19 (sorcery-speed activation), T01 (counters)

702.184b — Station cards have nonstandard layout with station symbols that are keyword abilities. See rule 721. PURE-DEF.

702.184c — Static abilities may modify station to use a characteristic other than power (e.g., toughness). PURE-DEF (modifier hook).

---

### 702.185 Warp

**Classification: DEFERRED — Phase 5-Pre T17 + Phase 7 + Phase 8 (alt cost from hand + delayed exile + future cast)**

702.185a — Warp represents TWO static abilities on the stack: (1) "You may cast this card from your hand by paying [cost] rather than its mana cost"; (2) "If this spell's warp cost was paid, exile the permanent this spell becomes at the beginning of the next end step. Its owner may cast this card after the current turn has ended for as long as it remains exiled."

**ATOM-702.185a-001**
- **Rule:** 702.185a (ability 1) — Warp: alt cost from hand
- **Mechanism:** Alternative cost
- **Minimal Board:** Player has a creature with "Warp {1}{U}" (normal cost {3}{U}{U}) in hand
- **Action:** Player casts for warp cost {1}{U}
- **Expected Result:** Spell is cast for {1}{U}. Creature enters the battlefield normally.
- **Phase:** Backlog — CR 702 keywords (was Phase 5-Pre T17)
- **Ticket:** NEW — Warp keyword (alt cost, delayed exile, future free cast)
- **Dependencies:** T17 (alternative cost)

**ATOM-702.185a-002**
- **Rule:** 702.185a (ability 2) — Warp: permanent exiled at next end step, then castable for free from exile
- **Mechanism:** Delayed triggered ability (exile at end step) + future casting permission
- **Minimal Board:** Creature entered via warp cost this turn
- **Action:** Beginning of next end step
- **Expected Result:** Permanent is exiled. After the current turn has ended, owner may cast the card from exile for as long as it remains exiled (without paying mana cost).
- **Phase:** Phase 7 (delayed trigger) + Phase 8
- **Ticket:** (same as above)

702.185b — "Warped" cards in exile = cards exiled by the delayed triggered ability from warp. PURE-DEF.

702.185c — "A spell was warped this turn" = a spell was cast for its warp cost this turn. PURE-DEF.

---

### 702.186 ∞ (Infinity)

**Classification: DEFERRED — Phase 8 (harness-conditional static ability)**

702.186a — ∞ is a keyword found on Infinity cards. "∞" is followed by ability text. Together they represent a static ability. PURE-DEF.

702.186b — "∞ — [Ability]" means "As long as this permanent is harnessed, it has [ability]." See rule 701.64, "Harness."

**ATOM-702.186b-001**
- **Rule:** 702.186b — ∞: grants ability as long as permanent is harnessed
- **Mechanism:** Static ability conditional on "harnessed" designation (see rule 701.64)
- **Minimal Board:** Player controls an Infinity permanent that is harnessed. It has "∞ — This creature gets +2/+2."
- **Action:** Check creature's P/T
- **Expected Result:** Creature gets +2/+2 (harnessed). If it becomes unharnessed, it loses the bonus.
- **Phase:** Phase 8
- **Ticket:** NEW — ∞ keyword (harness-conditional static ability)
- **Dependencies:** Harness infrastructure (rule 701.64)

**ATOM-702.186b-002**
- **Rule:** 702.186b — ∞: harness acquisition triggers the ability
- **Mechanism:** Harness event → static ability becomes active
- **Minimal Board:** Player controls an unharnessed Infinity permanent with "∞ — This creature gets +2/+2." Player casts a spell that harnesses the permanent.
- **Action:** Permanent becomes harnessed
- **Expected Result:** Creature immediately gets +2/+2 (static ability becomes active). If later unharnessed, it loses the bonus.
- **Phase:** Phase 8
- **Ticket:** (same as 702.186b)
- **Dependencies:** Harness infrastructure (rule 701.64), Phase 5 (continuous effects)

---

### 702.187 Mayhem

**Classification: DEFERRED — Phase 5-Pre T17 + Phase 8 (GY cast after discard)**

702.187a — Mayhem is a static ability that functions while the card is in a player's graveyard. PURE-DEF.

702.187b — "Mayhem [cost]" means "As long as you discarded this card this turn, you may cast it from your graveyard by paying [cost] rather than paying its mana cost."

**ATOM-702.187b-001**
- **Rule:** 702.187b — Mayhem: cast from GY for [cost] if discarded this turn
- **Mechanism:** Conditional zone-casting (GY, alt cost, discarded-this-turn condition)
- **Minimal Board:** Player discards a card with "Mayhem {1}{R}" this turn. Card is now in graveyard.
- **Action:** Player casts it from graveyard for {1}{R}
- **Expected Result:** Spell is cast from graveyard for mayhem cost. Legal because it was discarded this turn.
- **Phase:** Phase 5-Pre T17 + Phase 8
- **Ticket:** NEW — Mayhem keyword (discard-conditional GY cast, alt cost)
- **Dependencies:** T17 (alternative cost), T19 (zone-cast: GY), D16 (per-turn discard tracker)

**ATOM-702.187b-002**
- **Rule:** 702.187b — Mayhem: cannot cast from GY if not discarded this turn
- **Mechanism:** Condition check failure
- **Minimal Board:** A card with mayhem is in graveyard but was milled (not discarded) this turn
- **Action:** Player attempts to cast from GY for mayhem cost
- **Expected Result:** Cast is illegal. Card was not discarded this turn.
- **Phase:** Phase 8
- **Ticket:** (same as above)

**ATOM-702.187b-003**
- **Rule:** 702.187b — Mayhem: card leaves GY and returns → new object, not "discarded this turn"
- **Mechanism:** Zone-change epoch resets discard tracking
- **Minimal Board:** Player discards a card with mayhem. Card goes to GY. An effect exiles the card from GY, then returns it to GY.
- **Action:** Player attempts to cast the card from GY via mayhem
- **Expected Result:** Cast is illegal. The card in the GY is a new game object (rule 400.7). It was not "discarded" this turn — it entered the GY from exile, not from hand.
- **Phase:** Phase 8
- **Ticket:** (same as 702.187b)
- **Dependencies:** Rule 400.7 (zone-change epoch)

702.187c — "Mayhem" without a cost means "You may play this card from your graveyard if you discarded it this turn." (Play, not cast — includes lands.)

**ATOM-702.187c-001**
- **Rule:** 702.187c — Mayhem with no cost: play (not just cast) from GY if discarded this turn
- **Mechanism:** Play permission from GY (includes lands) conditional on discard
- **Minimal Board:** Player discards a land with "Mayhem" this turn.
- **Action:** Player plays the land from graveyard
- **Expected Result:** Land is played from graveyard (uses land play for turn). Legal because it was discarded this turn.
- **Phase:** Phase 8
- **Ticket:** (same as above)

---

### 702.188 Web-slinging

**Classification: DEFERRED — Phase 5-Pre T17 + Phase 8 (alt cost + bounce tapped creature)**

702.188a — "Web-slinging [cost]" means "You may cast this spell by paying [cost] and returning a tapped creature you control to its owner's hand rather than paying its mana cost."

**ATOM-702.188a-001**
- **Rule:** 702.188a — Web-slinging: alt cost = [cost] + return a tapped creature you control to hand
- **Mechanism:** Alternative cost (mana + bounce own tapped creature as additional component)
- **Minimal Board:** Player has a spell with "Web-slinging {U}" (normal cost {3}{U}{U}). Controls a tapped creature.
- **Action:** Player pays {U} and returns the tapped creature to hand
- **Expected Result:** Spell is cast for web-slinging cost. Tapped creature returned to owner's hand as part of cost payment.
- **Phase:** Phase 5-Pre T17 + Phase 8
- **Ticket:** NEW — Web-slinging keyword (alt cost + bounce tapped creature)
- **Dependencies:** T17 (alternative cost)

---

### 702.189 Firebending

**Classification: DEFERRED — Phase 7 (triggered ability) + Phase 8**

702.189a — "Firebending N" means "Whenever this creature attacks, add N {R}. Until end of combat, you don't lose this mana as steps and phases end."

**ATOM-702.189a-001**
- **Rule:** 702.189a — Firebending N: attack trigger adds N red mana that persists until end of combat
- **Mechanism:** Attack trigger + mana generation + mana persistence (through combat steps)
- **Minimal Board:** A creature with Firebending 3 attacks
- **Action:** Creature attacks; firebending triggers
- **Expected Result:** Player adds {R}{R}{R} to mana pool. This mana does NOT empty as combat steps/phases end (persists until end of combat phase).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Firebending keyword (attack trigger, mana add, combat-duration persistence)
- **Dependencies:** Phase 7 (triggered), Mana pool persistence exception

702.189b — "Whenever a player firebends" triggers whenever a firebending ability they control resolves. PURE-DEF.

---

### 702.190 Sneak

**Classification: DEFERRED — Phase 5-Pre T17 + Phase 8 (declare-blockers-step alt cost + bounce unblocked creature)**

702.190a — "Sneak [cost]" means "Any time you could cast an instant during your declare blockers step, you may choose to pay [cost] and return an unblocked creature you control to its owner's hand rather than pay this spell's mana cost."

**ATOM-702.190a-001**
- **Rule:** 702.190a — Sneak: alt cost during declare blockers step, return unblocked creature to hand
- **Mechanism:** Alternative cost with timing restriction (declare blockers step only) + bounce own unblocked creature
- **Minimal Board:** Player is in declare blockers step. Controls an unblocked attacking creature. Has a creature spell with "Sneak {G}" in hand.
- **Action:** Player pays {G} and returns the unblocked creature to hand
- **Expected Result:** Spell is cast for sneak cost. Unblocked creature returned to hand.
- **Phase:** Phase 5-Pre T17 + Phase 8
- **Ticket:** NEW — Sneak keyword (declare-blockers alt cost, bounce unblocked creature, enters tapped+attacking)
- **Dependencies:** T17 (alternative cost), Combat step timing

702.190b — A permanent spell cast using sneak enters the battlefield tapped and attacking. It attacks the same target as the creature that was returned to hand.

**ATOM-702.190b-001**
- **Rule:** 702.190b — Sneak: permanent enters tapped and attacking the same target as bounced creature
- **Mechanism:** Modified ETB (tapped + attacking + inherited attack target)
- **Minimal Board:** Player cast a creature via sneak, returning a creature that was attacking opponent
- **Action:** Sneak creature enters
- **Expected Result:** Creature enters the battlefield tapped and attacking the same opponent the bounced creature was attacking.
- **Phase:** Phase 8
- **Ticket:** (same as 702.190a)
- **Dependencies:** Rule 506.3a (enters attacking)

**Tag: SHARED-BEHAVIOR ninjutsu-sneak** — Sneak (702.190) and Ninjutsu (702.49, Session 7B) share a "swap creature during combat" pattern. Key differences: Ninjutsu returns unblocked attacker + puts creature from hand onto battlefield attacking. Sneak returns unblocked attacker + casts spell for alt cost (enters tapped + attacking same target). Implementation should share the "return unblocked creature, put/cast replacement attacking same target" logic.

--- End of Chunk 7 (revised) ---

## Classification Summary Table

| Rule | Keyword | Classification | Phase | Notes |
|------|---------|---------------|-------|-------|
| 702.81 | Retrace | DEFERRED | Phase 8 | Zone-cast from GY + discard-land additional cost |
| 702.82 | Devour | DEFERRED | Phase 6 + 8 | ETB replacement, sacrifice creatures, counters |
| 702.83 | Exalted | DEFERRED | Phase 7 + 8 | Triggered on solo attack |
| 702.84 | Unearth | DEFERRED | Phase 8 | GY activated, haste, delayed exile |
| 702.85 | Cascade | DEFERRED | Phase 7 + 8 | **PRIORITY.** Triggered on cast, exile loop, free cast |
| 702.86 | Annihilator | DEFERRED | Phase 7 + 8 | Attack trigger, forced sacrifice |
| 702.87 | Level Up | DEFERRED | Phase 8 | Sorcery-speed activated, level counters |
| 702.88 | Rebound | DEFERRED | Phase 6 + 7 + 8 | Replacement exile + delayed free cast |
| 702.89 | Umbra Armor | DEFERRED | Phase 6 + 8 | Replacement: prevent destroy, destroy Aura |
| 702.90 | Infect | ALREADY-IMPL (partial) | T21c / Phase 5 | 702.90b/c implemented; 702.90d/e deferred |
| 702.91 | Battle Cry | DEFERRED | Phase 7 + 8 | Attack trigger, +1/+0 to others |
| 702.92 | Living Weapon | DEFERRED | Phase 7 + 8 | ETB trigger, Germ token, auto-attach |
| 702.93 | Undying | DEFERRED | Phase 7 + 8 | Dies trigger with counter check |
| 702.94 | Miracle | DEFERRED | Phase 7 + 8 | First-draw reveal + triggered alt-cost |
| 702.95 | Soulbond | DEFERRED | Phase 7 + 8 | Dual ETB triggers, pairing designation |
| 702.96 | Overload | DEFERRED | Phase 5 + 8 | **PRIORITY.** Alt cost + text-changing effect |
| 702.97 | Scavenge | DEFERRED | Phase 8 | GY activated, exile self, counters on target |
| 702.98 | Unleash | DEFERRED | Phase 5 + 6 + 8 | Optional ETB counter + blocking restriction |
| 702.99 | Cipher | DEFERRED | Phase 8 | Exile encoding, combat damage copy |
| 702.100 | Evolve | DEFERRED | Phase 7 + 8 | ETB trigger, P/T comparison, counter |
| 702.101 | Extort | DEFERRED | Phase 7 + 8 | Cast trigger, optional payment, drain |
| 702.102 | Fuse | DEFERRED | Phase 9 | Split card dual-cast |
| 702.103 | Bestow | DEFERRED | Phase 5 + 8 | **PRIORITY.** Alt cost, type change, Aura mode |
| 702.104 | Tribute | DEFERRED | Phase 6 + 7 + 8 | ETB opponent choice + conditional trigger |
| 702.105 | Dethrone | DEFERRED | Phase 7 + 8 | Attack trigger, life comparison |
| 702.106 | Hidden Agenda | OUT-OF-SCOPE | — | Conspiracy format |
| 702.107 | Outlast | DEFERRED | Phase 8 | Sorcery-speed tap activated, counter |
| 702.108 | Prowess | DEFERRED | Phase 7 + 8 | **PRIORITY.** Noncreature cast trigger |
| 702.109 | Dash | DEFERRED | Phase 8 | **PRIORITY.** Alt cost, haste, delayed return |
| 702.110 | Exploit | DEFERRED | Phase 7 + 8 | ETB trigger, optional sacrifice |
| 702.111 | Menace | ALREADY-IMPL | Phase 4 | **PRIORITY.** Blocking restriction (2+ blockers) |
| 702.112 | Renown | DEFERRED | Phase 7 + 8 | Combat damage trigger, renowned designation |
| 702.113 | Awaken | DEFERRED | Phase 5 + 8 | Alt cost, land animation, counters |
| 702.114 | Devoid | DEFERRED | Phase 5 | CDA, colorless override |
| 702.115 | Ingest | DEFERRED | Phase 7 + 8 | Combat damage trigger, exile top card |
| 702.116 | Myriad | DEFERRED | Phase 7 + 9 | Attack trigger, per-opponent token copies |
| 702.117 | Surge | DEFERRED | Phase 8 | Conditional alt cost |
| 702.118 | Skulk | DEFERRED | Phase 8 | Power-based blocking restriction |
| 702.119 | Emerge | DEFERRED | Phase 5-Pre T17 + 8 | Alt cost + sacrifice + MV reduction |
| 702.120 | Escalate | DEFERRED | Phase 8 | Per-mode additional cost |
| 702.121 | Melee | DEFERRED | Phase 7 + 9 | Attack trigger, per-opponent bonus |
| 702.122 | Crew | DEFERRED | Phase 8 | **PRIORITY.** Tap-creatures cost, Vehicle animation |
| 702.123 | Fabricate | DEFERRED | Phase 7 + 8 | ETB trigger, counters-or-tokens |
| 702.124 | Partner | DEFERRED | Phase 9 | Commander deck construction |
| 702.125 | Undaunted | DEFERRED | Phase 5-Pre T17 + 9 | Cost reduction per opponent |
| 702.126 | Improvise | DEFERRED | Phase 5-Pre T17 | Tap artifacts for generic mana |
| 702.127 | Aftermath | DEFERRED | Phase 9 | Split card, GY-only cast, exile on leave |
| 702.128 | Embalm | DEFERRED | Phase 8 | GY activated, modified token copy |
| 702.129 | Eternalize | DEFERRED | Phase 8 | GY activated, 4/4 modified token |
| 702.130 | Afflict | DEFERRED | Phase 7 + 8 | Becomes-blocked trigger, life loss |
| 702.131 | Ascend | DEFERRED | Phase 8 | City's blessing designation |
| 702.132 | Assist | DEFERRED | Phase 8 + 9 | Multiplayer cost sharing |
| 702.133 | Jump-Start | DEFERRED | Phase 8 | GY cast, discard additional cost, exile |
| 702.134 | Mentor | DEFERRED | Phase 7 + 8 | Attack trigger, power-comparison counters |
| 702.135 | Afterlife | DEFERRED | Phase 7 + 8 | Dies trigger, Spirit tokens |
| 702.136 | Riot | DEFERRED | Phase 6 + 8 | ETB choice: counter or haste |
| 702.137 | Spectacle | DEFERRED | Phase 5-Pre T17 + 8 | Conditional alt cost (opponent life loss) |
| 702.138 | Escape | DEFERRED | Phase 5-Pre T17 + 8 | **PRIORITY.** GY cast, exile cards as cost |
| 702.139 | Companion | DEFERRED | Phase 9 | Outside-game, special action |
| 702.140 | Mutate | DEFERRED | Phase 8 | **PRIORITY.** Alt cost, merge, rule 729 |
| 702.141 | Encore | DEFERRED | Phase 7 + 8 + 9 | GY activated, per-opponent tokens |
| 702.142 | Boast | DEFERRED | Phase 8 | Attack-conditioned, once-per-turn |
| 702.143 | Foretell | DEFERRED | Phase 5-Pre T17 + 8 | Special action exile, future alt cost |
| 702.144 | Demonstrate | DEFERRED | Phase 7 + 8 | Cast trigger, mutual copy |
| 702.145 | Daybound/Nightbound | DEFERRED | Phase 9 | DFC + day/night system |
| 702.146 | Disturb | DEFERRED | Phase 9 | DFC cast from GY transformed |
| 702.147 | Decayed | DEFERRED | Phase 5 + 7 + 8 | Can't block + attack → sacrifice |
| 702.148 | Cleave | DEFERRED | Phase 5 + 8 | Alt cost + bracket text removal |
| 702.149 | Training | DEFERRED | Phase 7 + 8 | Co-attacker power trigger, counter |
| 702.150 | Compleated | DEFERRED | Phase 8 | Phyrexian mana loyalty reduction |
| 702.151 | Reconfigure | DEFERRED | Phase 8 | **PRIORITY.** Dual activate, Equipment creature |
| 702.152 | Blitz | DEFERRED | Phase 8 | **PRIORITY.** Alt cost, haste, dies-draw |
| 702.153 | Casualty | DEFERRED | Phase 7 + 8 | **PRIORITY.** Sacrifice additional cost, copy |
| 702.154 | Enlist | DEFERRED | Phase 7 + 8 | Tap-on-attack, power bonus |
| 702.155 | Read Ahead | DEFERRED | Phase 8 | Saga starting chapter choice |
| 702.156 | Ravenous | DEFERRED | Phase 7 + 8 | X-based counters + conditional draw |
| 702.157 | Squad | DEFERRED | Phase 7 + 8 | Repeatable additional cost, token copies |
| 702.158 | Space Sculptor | OUT-OF-SCOPE | — | Un-sets / Unfinity |
| 702.159 | Visit | OUT-OF-SCOPE | — | Un-sets / Unfinity |
| 702.160 | Prototype | DEFERRED | Phase 9 | Alternate characteristics on stack/BF |
| 702.161 | Living Metal | DEFERRED | Phase 5 + 8 | Turn-conditional creature type |
| 702.162 | More Than Meets the Eye | DEFERRED | Phase 9 | DFC casting transformed |
| 702.163 | For Mirrodin! | DEFERRED | Phase 7 + 8 | ETB trigger, Rebel token, auto-attach |
| 702.164 | Toxic | DEFERRED | Phase 7 + 8 | Combat damage trigger, additive poison |
| 702.165 | Backup | DEFERRED | Phase 7 + 8 | **PRIORITY.** ETB counters + ability grant |
| 702.166 | Bargain | DEFERRED | Phase 8 | **PRIORITY.** Optional sacrifice additional cost |
| 702.167 | Craft | DEFERRED | Phase 8 + 9 | Exile self + materials, return transformed |
| 702.168 | Disguise | DEFERRED | Phase 8 | Face-down cast, ward {2} |
| 702.169 | Solved | DEFERRED | Phase 8 | Case condition-based designation |
| 702.170 | Plot | DEFERRED | Phase 8 | **PRIORITY.** Special action exile, future free cast |
| 702.171 | Saddle | DEFERRED | Phase 8 | Tap-creatures cost, saddled designation |
| 702.172 | Spree | DEFERRED | Phase 8 | Cost-selects-mode additional costs |
| 702.173 | Freerunning | DEFERRED | Phase 8 | Conditional alt cost |
| 702.174 | Gift | DEFERRED | Phase 7 + 8 | Two-ability keyword: opponent choice (static) + benefit delivery (static/triggered) |
| 702.175 | Offspring | DEFERRED | Phase 7 + 8 | Additional cost, conditional 1/1 token copy |
| 702.176 | Impending | DEFERRED | Phase 5-Pre T17 + 5 + 6 + 7 | 4 abilities: alt cost, ETB counters, not-creature static, end-step counter removal |
| 702.177 | Exhaust | DEFERRED | Phase 8 | Activate-only-once restriction |
| 702.178 | Max Speed | DEFERRED | Phase 8 | Speed-conditional static ability (speed = 4) |
| 702.179 | Start Your Engines! | DEFERRED | Phase 7 + 8 | SBA sets speed to 1; inherent trigger speed +1 on opp life loss |
| 702.180 | Harmonize | DEFERRED | Phase 5-Pre T17 + 6 + 8 | GY cast, tap-creature cost reduction by power, exile on leave stack |
| 702.181 | Mobilize | DEFERRED | Phase 7 + 8 | Attack trigger, N Warrior tokens tapped+attacking, delayed sacrifice |
| 702.182 | Job Select | DEFERRED | Phase 7 + 8 | ETB trigger, Hero token, auto-attach Equipment |
| 702.183 | Tiered | DEFERRED | Phase 8 | Modal + per-mode additional cost |
| 702.184 | Station | DEFERRED | Phase 8 | Activated: tap creature → charge counters = power, sorcery speed |
| 702.185 | Warp | DEFERRED | Phase 5-Pre T17 + 7 + 8 | Alt cost from hand, delayed exile at end step, future free cast |
| 702.186 | ∞ | DEFERRED | Phase 8 | Harness-conditional static ability |
| 702.187 | Mayhem | DEFERRED | Phase 5-Pre T17 + 8 | GY cast if discarded this turn, alt cost; costless variant = play |
| 702.188 | Web-slinging | DEFERRED | Phase 5-Pre T17 + 8 | Alt cost + bounce own tapped creature |
| 702.189 | Firebending | DEFERRED | Phase 7 + 8 | Attack trigger, add N {R}, mana persists through combat |
| 702.190 | Sneak | DEFERRED | Phase 5-Pre T17 + 8 | Declare-blockers alt cost, bounce unblocked creature, enters tapped+attacking |

**Totals:**
- **ALREADY-IMPLEMENTED:** 2 (Menace 702.111, Infect 702.90 partial)
- **DEFERRED:** 105 keywords
- **OUT-OF-SCOPE:** 3 (Hidden Agenda 702.106, Space Sculptor 702.158, Visit 702.159)
- **TESTABLE (with ATOM tests):** ~130 tests generated across all chunks (including ~30 added during first-round audit)

---

## Composition Tests

### COMP-INFECT-TRAMPLE-001
- **Composes:** ATOM-702.90b (infect to player), Trample (702.19b from Session 7B)
- **Rule:** Creature with infect and trample assigns lethal damage (-1/-1 counters) to blocker, remainder as poison counters to defending player
- **Minimal Board:** A 5/5 with infect and trample attacks. Blocked by a 2/2.
- **Action:** Combat damage assignment
- **Expected Result:** 2 -1/-1 counters to blocker (lethal), 3 poison counters to defending player.
- **Phase:** Phase 8
- **Ticket:** T21c (infect routing) + trample overflow

### COMP-CASCADE-STORM-001
- **Composes:** ATOM-702.85a (cascade), Storm (702.40 from Session 7B)
- **Rule:** Cascade spell with storm — storm copies don't trigger cascade (cascade triggers on cast, copies aren't cast)
- **Minimal Board:** Player casts a spell with cascade and storm (storm count = 2)
- **Action:** Storm creates 2 copies. Cascade triggers from the original cast.
- **Expected Result:** Only 1 cascade trigger (from original cast). Storm copies do NOT trigger cascade.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Cascade + Storm interaction

### COMP-BESTOW-UNDYING-001
- **Composes:** ATOM-702.103f (bestow unattach → creature), ATOM-702.93a (undying)
- **Rule:** Bestowed Aura enchanting a creature with undying — when creature dies and returns, Aura becomes unattached → becomes creature
- **Minimal Board:** Creature with undying enchanted by a bestowed Aura. Creature is destroyed.
- **Action:** Creature dies, undying triggers, creature returns with +1/+1 counter
- **Expected Result:** Bestowed Aura becomes unattached when enchanted creature leaves → Aura becomes a creature. Undying creature returns independently.
- **Phase:** Phase 7 + Phase 8

### COMP-TOXIC-INFECT-001
- **Composes:** ATOM-702.164a (toxic), ATOM-702.90b/c (infect)
- **Rule:** Creature with both toxic and infect — infect replaces damage (poison + -1/-1 counters), toxic ALSO triggers (additional poison)
- **Minimal Board:** A 2/2 creature with infect and Toxic 1 deals combat damage to a player
- **Action:** Combat damage resolves
- **Expected Result:** Infect: 2 poison counters (replaces damage). Toxic: 1 additional poison counter (triggered). Total: 3 poison counters to player.
- **Phase:** Phase 8

### COMP-DASH-BLITZ-001
- **Composes:** ATOM-702.109a (dash), ATOM-702.152a (blitz)
- **Rule:** A creature with both dash and blitz — only one alternative cost can be paid
- **Minimal Board:** Player casts a creature that has both "Dash {2}{R}" and "Blitz {1}{R}"
- **Action:** Player chooses to pay blitz cost
- **Expected Result:** Creature has blitz effects (haste, dies-draw, sacrifice at end step). Dash effects do NOT apply.
- **Phase:** Phase 8
- **Ticket:** T17 (only one alt cost at a time)

### COMP-CREW-TYPE-205.1b-001
- **Composes:** ATOM-702.122-003 (crew on non-Vehicle), Rule 205.1b (type addition is additive)
- **Rule:** When a non-artifact permanent "becomes an artifact creature" via crew, it gains those types additively — existing types (enchantment, etc.) are NOT overwritten. 205.1b carves out that "becomes" for types means "gains" unless it says "instead."
- **Minimal Board:** A non-artifact enchantment creature has gained "Crew 3" through an ability-copying effect. Player controls a 3/3 creature.
- **Action:** Tap the 3/3 to crew the enchantment creature
- **Expected Result:** The permanent becomes an enchantment artifact creature until end of turn. It retains its enchantment type AND creature type, and gains the artifact type. All three types coexist.
- **Phase:** Phase 5 (layer 4 type changes) + Phase 8
- **Ticket:** (same as 702.122a)
- **Dependencies:** Rule 205.1b (additive type changes), Layer 4 (type-changing effects)

---

## Gap Report

### 1. Cost Modification Pipeline (T17) — HIGH coverage gap
Multiple keywords in this session depend on T17 (alternative/additional cost framework): Retrace, Overload, Bestow, Dash, Emerge, Escalate, Escape, Foretell, Blitz, Casualty, Bargain, Plot, Spree, Gift, Offspring, Surge, Spectacle, Undaunted, Improvise, Freerunning, Harmonize, Squad, Impending, Tiered. T17 is the single most critical dependency for Phase 8 keyword implementation. No atomic test in this session directly tests T17 itself — that belongs to Session 5 (rule 601.2f).

### 2. Triggered Ability Infrastructure (Phase 7) — HIGH coverage gap
~40 keywords depend on Phase 7 triggered ability infrastructure. The delta-log scanner, trigger registration, and trigger resolution pipeline must be implemented before any of these keywords become testable. Session 9A (rules 703-712) will cover the trigger system rules.

### 3. Zone-Casting Framework (T19) — MEDIUM gap
Keywords requiring casting from non-hand zones: Retrace, Aftermath, Jump-Start, Escape, Unearth (activated from GY), Embalm, Eternalize, Scavenge, Harmonize (GY), Mayhem (GY), Disturb. T19 currently handles activation_zone for activated abilities but needs extension to spell casting permissions (CastPermission framework documented in cast.rs).

### 4. Face-Down Infrastructure (Rule 708) — MEDIUM gap
Disguise (702.168) and Foretell (702.143) depend on face-down casting/exile infrastructure. No tests for rule 708 exist yet. Session 9B will cover rule 708.

### 5. Vehicle / Mount Ecosystem — LOW priority gap
Crew (702.122), Saddle (702.171), Station (702.184), Living Metal (702.161) share a tap-creatures-as-cost pattern. Max Speed (702.178) and Start Your Engines! (702.179) form a new "speed" subsystem (player attribute + SBA + inherent trigger). Mobilize (702.181) is an attack-trigger token creator. Job Select (702.182) is Living Weapon's spiritual successor (ETB token + auto-attach). A shared Vehicle/speed infrastructure ticket may be more efficient than individual keyword tickets.

### 6. Merging Permanents (Rule 729) — LOW priority gap
Mutate (702.140) depends on rule 729 (merging permanents). This is a unique mechanic with no other keywords sharing the infrastructure. Session 9B will cover rule 729.

### 7. Per-Permanent Designations (D17) — MEDIUM gap
Multiple keywords introduce new designations: renowned (Renown), saddled (Saddle), exhausted (Exhaust), city's blessing (Ascend), paired (Soulbond), solved (Solved). D17 needs a flexible designation system. Currently not implemented.

### 8. DFC System (Phase 9) — Expected gap
Daybound/Nightbound, Disturb, More Than Meets the Eye, Prototype, Craft are all deferred to Phase 9 DFC implementation. Rule 712 (DFCs) covered in Session 9B.

### 9. Speed System (702.178 + 702.179) — NEW subsystem gap
Max Speed and Start Your Engines! introduce "speed" as a new player attribute (values 0–4). Requires: player-level numeric attribute storage, a new SBA (704) for speed initialization, an inherent triggered ability with no source (speed +1 on opponent life loss, once/turn). No existing infrastructure supports player-level numeric attributes beyond life total and poison counters.

### 10. Harness Infrastructure (Rule 701.64) — NEW subsystem gap
The ∞ keyword (702.186) depends on the "harness" action (rule 701.64). Harnessed is a designation on permanents. Session 7A should cover rule 701.64 if not already; verify during merge pass.

### 11. Linked Ability Pattern — MEDIUM shared-infra gap
Devour (702.82), Tribute (702.104), Exploit (702.110), and Evolve (702.100b) all follow a pattern where an ETB or replacement effect stores a value (creatures devoured, tribute paid, creature exploited, creature evolved), and a linked triggered ability references that stored value. Implementation needs a generic `LinkedAbilityData` mechanism that persists from the original event through to the linked trigger resolution.

### 12. Text-Changing Effects (Rule 612) — MEDIUM shared-infra gap
Overload (702.96), Splice (702.47 from Session 7B), and Cleave (702.148) all modify spell text. The engine needs a shared `TextModification` layer. Currently no infrastructure exists for runtime text manipulation.

### 13. Evasion Framework — Phase 4 fix gap
Menace (702.111) is classified as ALREADY-IMPLEMENTED but is **NOT enforced** in `validate_blockers()`. Additionally, Skulk (702.118) introduces a power-comparison blocking restriction. The current per-pair blocker check needs extension to handle both per-pair filters (flying/reach, skulk) and set-level constraints (menace: ≥ 2 blockers). Phase 4 fix ticket needed.

### 14. Crew/Saddle Shared Infrastructure — LOW priority gap
Crew (702.122) and Saddle (702.171) share a "tap creatures with total power ≥ N" pattern. Implementation should share a `tap_creatures_for_power(n, filter)` helper. Crew also needs a "creature crewed" event for trigger support.

--- End of Session 8 ---

