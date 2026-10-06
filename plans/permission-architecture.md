# Permission — who may cast, play or activate what, from where, and when

> **Status:** a slot, not a design. Created 2026-10-05 by `codebase-state.md`
> item 212's census (`plans/references/cast-census.md`, #223). At its review the
> owner chose a document of its own, designed beside RS-2 and built before B2.
> No code written. **Authority:** whether a player may begin to cast, play or
> activate an object, from the zone it is in, now: CR 601.3's "allows" half,
> 602.2 (who may activate), 602.5 (when), 113.6b–m (from which zone), 305.1–2
> (land play), 606.3 (loyalty abilities), and 903.8's permission to cast a
> commander. On what exists, `codebase-state.md` wins; on what is being built,
> this file does. `CLAUDE.md` → "Critical path to v1" owns the ordering.
> **Graduates:** `backlog.md` §2.3 (casting from a non-hand zone), §2.8
> (functioning zones and activation restrictions), §2.11 (loyalty abilities),
> `codebase-state.md` "Before card breadth" item 2 (who may activate). It is
> also the first consumer of §2.15's effective-value query, through the land
> count `can_play_land` reads (CR 305.2). The PR that builds the land count
> builds that query, and §2.15's other values reuse it (`cost-architecture.md`
> §3.9: one surface for three values).
> **Companions:**
> - `cant-effects-architecture.md`: RS-2 builds the prohibition half at the
>   same three checks, and its §4.3 already argues that permission and
>   prohibition want one query (Conduit of Worlds grants what Aggressive
>   Mining removes).
> - `cost-architecture.md`: what a permission's cost pays, through CP-1 and
>   CP-2.
> - `backlog.md` §2.24: CR 609.4's "as though", whose permission half holds
>   "as though it had flash" (`roadmap-v2.md` B8).
> - The copy track: face-down casting (CV-6) and alternative characteristics
>   (`backlog.md` §2.41).
>
> Phase codes are `PM-*`; branches are `permission/pm-<n>-…`.

---

## 0. Why a document, and what it owns

The census counted the families this surface would express (Commander-legal
cards, `legal:commander -t:token game:paper -is:funny`; each query is in the
census beside its number):

| family | cards | where in the census |
|---|---:|---|
| a commander, cast from the command zone (CR 903.8) | 3,497 | §7, `is:commander` |
| an ability that works in a hand: cycling, channel, ninjutsu, … | 516 keyword + 95 printed | §7, §4 |
| cast or play a card from exile | 450 + 160 after a special action (foretell, plot, suspend) | §6, §7 |
| an activation limit: once each turn, during a step, if a condition holds, once | 489 abilities + 92 keyword | §4, §7 |
| cast from a graveyard: flashback, escape, … | 355 keyword + 241 + 39 printed | §7, §6 |
| an ability that works in a graveyard: unearth, embalm, … | 123 keyword + 200 printed | §7, §4 |
| cast without paying its mana cost | 315 | §6 |
| a planeswalker's loyalty abilities (CR 606) | 311 | §7 |
| cast while something resolves: cascade, discover, madness, … | 183 | §7 |
| cast or play from the top of a library | 88 | §6 |
| who may activate: any player, or only an opponent | 39 + 5 | §4 |
| play lands from another zone, or more lands a turn | 52 + 36 | §2.1, §6 |
| cast a card its caster does not own | 29 | §6 |

**One surface.** SU-7 made each option one check that the enumeration and
the enforcement both ask: `can_begin_to_cast`, `can_play_land`, and
`can_activate_its_abilities` with `can_begin_to_activate`. Every family above
is a question one of those checks answers. RS-2 adds the prohibitions at the
same checks, so the two are designed side by side: each check's reasons are
designed once, and neither half reshapes the other's enum.

## 1. The rules, verbatim

> **601.3.** A player can begin to cast a spell only if a rule or effect
> allows that player to cast it and no rule or effect prohibits that player
> from casting it.

> **602.2.** … Only an object's controller (or its owner, if it doesn't have a
> controller) can activate its activated ability unless the object
> specifically says otherwise. …

> **113.6.** Abilities of an instant or sorcery spell usually function only
> while that object is on the stack. Abilities of all other objects usually
> function only while that object is on the battlefield. The exceptions are as
> follows: … (113.6a–p)

> **305.1.** A player who has priority may play a land card from their hand
> during a main phase of their turn when the stack is empty. …

> **606.3.** A player may activate a loyalty ability of a permanent they
> control any time they have priority and the stack is empty during a main
> phase of their turn, but only if no player has previously activated a
> loyalty ability of that permanent that turn.

> **903.8.** A player may cast a commander they own from the command zone. …

## 2. What the engine has (2026-10-05, at `0481f00`)

- `oracle::mana_helpers::can_begin_to_cast` admits the hand alone, and only
  the owner's card: `CannotCast::NotInHand`. `StackEntry.cast_from` records
  the zone a spell was cast from (CR 903.8's tax reads it).
- `oracle::legality::can_play_land` admits the hand alone: `CannotPlayLand::
  NotInHand`. Its count, `PlayerState::lands_per_turn`, is set to 1 and
  nothing changes it: `CannotPlayLand::NoLandDropLeft`.
- `oracle::mana_helpers::can_activate_its_abilities` asks the controller once
  per source: `CannotActivate::NotYours`. `can_begin_to_activate` reads
  `ActivationRestriction`, which has `None` and `OnlyAsSorcery`. A source off
  the battlefield is `CannotActivate::NotOnBattlefield`.
- `AlternativeCost::Flashback` and `Escape` exist, and no graveyard is ever a
  zone a card is cast from.
- No loyalty ability exists. Loyalty Probe has none, and the counters its
  costs move are CP-2's arms.

## 3. What it must express at v1

From the census's rules pass (`cast-census.md` §2):

- **A permission per player, object and zone**, granted by a rule or an
  effect:
  - the hand (601.2's "usually the hand");
  - the command zone, for a commander its player owns (903.8);
  - a graveyard: flashback (702.34a) and its siblings, and "you may cast …
    from your graveyard";
  - exile, after a special action (116.2f, h, k) or during a resolution
    (608.2g);
  - a library's top, with 601.3f's "can look at" for a face-down card.
- **Ownership inside the permission, not in front of it.** Hostage Taker
  casts a card it does not own; 903.8's commander is "a commander they own".
- **A permission's cost and consequence.** Flashback's permission comes with
  its alternative cost (118.9) and an exile in place of any other zone. Which
  permission a cast used is a choice already made, and 601.2b says it may
  restrict the choices after it.
- **The reason, typed.** `NotPermitted { zone }` replaces `CannotCast::
  NotInHand`, `CannotPlayLand::NotInHand` and `CannotActivate::
  NotOnBattlefield`, in CR 601.3's and 113.6's words. RS-2's reason names
  the prohibiting source, beside it.
- **Who may activate, per ability** (602.2): its controller, any player, or
  only an opponent. A field on `AbilityDef`, read by `can_begin_to_activate`
  with the player. The enumeration walks every permanent the player could
  activate an ability of, not only theirs (`codebase-state.md` "Before card
  breadth" item 2, graduated here).
- **When, beyond priority** (602.5): once or twice each turn, during a step
  or a turn, if a condition holds (`Condition` exists), once. 602.5b keeps a
  limit across a change of control, and 602.5c keeps an acquired ability's
  limit to that instance. 606.3's loyalty limit is per permanent and counts
  every player's activations.
- **From which zone** (113.6b, j, m): an ability that names its zone, or
  whose cost or effect moves its own object out of one: cycling from a hand,
  unearth from a graveyard.
- **The land count** (305.2): additional land drops, which `lands_per_turn`
  holds and nothing sets. It is the first consumer of `backlog.md` §2.15's
  effective-value query, which its PR builds; max hand size and player
  hexproof reuse that query. 305.2b and 305.3 deny every override.

## 4. Seams

- **RS-2.** The prohibition half at the same checks. RS-2's §4.3 contract,
  over-approximate at the gate and exact after the announcement, is this
  document's too: a permission that a choice could satisfy is offered, and
  CR 601.2's rewind refuses a cast that does not satisfy it.
- **B8, "as though".** "Cast as though it had flash" (CR 601.3b, 609.4) stays
  `backlog.md` §2.24's. This document owns a zone, an owner and an activation
  limit, not a fiction.
- **CP-1 and CP-2.** A permission's alternative cost is an `AlternativeCost`.
  A loyalty ability's cost is CP-2's counter arms; its timing is 606.3's,
  here.
- **Item 162.** Every zone a permission names adds candidates to the priority
  question, whose candidate list is 18.6% of a close-out run's instructions
  (SU-6's engine arm). The design prices the gather. Item 211's mana
  abilities at a priority question (CR 605.3a) are item 162's offer, not
  this document's.
- **B2.** CR 903.8 is the permission; the designation and the tax are B2's.
- **`backlog.md` §2.9.** A permission to cast from a library's top or from
  face-down exile reads what its player can see (601.3f).

## 5. Open questions for the design

Named here so the design session starts from them; none is answered.

1. **Where a permission lives.** There are three kinds:
   - a static ability that functions in the zone it permits from (113.6e/f:
     flashback works in the graveyard);
   - a resolution's permission with a duration (an impulse draw's "until end
     of turn");
   - a permission tied to one object ("for as long as it remains exiled").

   Is RS-1's `RestrictionRegistry` the shape for the second and third, as
   CR 609.4's permission half (B8) already asks?
2. **The enumeration's cost, priced at four seats.** PM-0 measures the
   gather on SU-6's engine arm: the candidate list is 18.6% of a close-out
   run's instructions, and 39–40 µs a question on the large board in release
   (`setup-architecture.md` §7c). `castable_spells` walks the hand today, and
   a permission adds every zone it names, whose size grows through a game.
   **If a scan of those zones costs measurably, the plan is an index**,
   maintained where an object changes zones, as LK registers the abilities
   that function off the battlefield. "Any player may activate" (44 cards)
   must not cost every player a read of every opponent's abilities.
   `activatable_abilities` already walks every permanent and stops at SU-7's
   per-source control check. A per-card bit for an ability someone else may
   activate keeps that early exit, and the rest is a Layer 6 grant's
   question.
3. **A consequence.** Flashback's exile is a replacement keyed on the
   permission a cast used. Where is that recorded, and does a copy of the
   spell keep it (CR 707.10)?
4. **Two permissions for one card.** Flashback and Snapcaster Mage's grant
   are an example: does the player choose (601.2b), and when?
5. **A limit's state.** Per ability instance (602.5c), per permanent (606.3),
   kept across a change of control (602.5b), held on `GameState`
   (`CLAUDE.md`: no decision-site state off it).

## 6. Sizing and the phase plan

| PR | Shape | Size | Risk |
|---|---|---|---|
| **PM-0, who may cast, play or activate: the design** | §5 answered, and the build split and sized against the census's families. Reviewed before any build, with RS-2's reasons designed beside it | docs | low |
| **PM-1 onward** | sized by PM-0. The first carries CR 903.8's command zone, since B2 needs it | — | — |

**Slot** (the owner, 2026-10-05): PM-0 beside RS-2, which is "beside A,
after A6c's item 10" (`roadmap-v2.md` §3a), so RS-2 builds against reasons
designed for both halves. The first build lands before B2, which comes first
after triggers (`roadmap-v2.md` B11). The rest is pulled by its card families
and lands before C.
