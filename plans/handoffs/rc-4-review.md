# RC-4 review — answers and changes (2026-09-02)

The owner's review of PR #85, one entry per comment, in file order. A
**Changed** line means the same PR carries the change. Questions for
understanding are answered here and nowhere in source comments. Rulings quoted
were fetched from Scryfall on 2026-09-02.

## `engine/layers/lookahead.rs`

**The preamble.** Moved to `replacement-architecture.md` §5d. The module doc is
now what the overlay is, the two accessors, and a pointer.

**Why two bool flags (`any_multi_row_group`, `any_control_changing`)?** They are
the two gates `compute.rs` already keys off the registry's
`RegistryScopeSummary`: CR 613.6's "started applying" bookkeeping runs only
when some effect has more than one row, and `effective_controller` walks only
when some row can change control. The would-be rows are not in the registry,
so a frame whose own rows would flip a gate — an entering permanent with a
two-row static, or one that changes control of itself — would be answered
wrongly by the registry's summary alone. They were bespoke only in that they
duplicated the registry's computation. **Changed:** `Lookahead::summary` is a
`RegistryScopeSummary` computed by `RegistryScopeSummary::of`, the same
function the registry now uses for itself; both gates read it.

**Re-timestamping for Equipment and Auras (CR 613.7e).** CR 613.7e gives an
Aura, Equipment or Fortification a new timestamp when it *becomes attached*,
and never re-timestamps the host. For the frame that cannot matter: the only
property the would-be timestamps need is "later than every registered row",
and a re-timestamp on attachment is later still. The order among the object's
own rows and its own counters could in principle move, but that order only
decides conflicts between the object's own effects, which an entry cannot
create. The engine does not model 613.7e's re-timestamp anywhere yet;
attachment as a layers input is critical-path item 6b.

## `types/replacement.rs`

**Will `EnterBattlefield { cast }` grow?** Yes, along one axis, and the doc now
names it: CR 400.7d lets a permanent's abilities reference facts about the
spell it was. Gnarlid Pack's "if it was kicked, it enters with a +1/+1
counter" is a CR 614.1c effect reading such a fact; "cast from" and mana-spent
conditions are the same shape. Those land as fields beside `cast`. A fact
about the card where it is (Grafdigger's Cage's "creature cards in
graveyards") stays on `ZoneChange::object`. One field is what one customer
needed; the shape is not a one-off.

**`ReplacementClass::of`.** **Changed:** `from_rewrite`.

## `cards/phase_rc_cards.rs`

**Why does the Priest's target reach the battlefield at all?** Because RC-2
split entering into two chokepoint events: the `ZoneChange` performer moves
the card into the zone and *then* proposes `EnterBattlefield`, which is the
event `EnterMods` and the CR 614.12 frame hang off. The Priest watches the
second, so by the time it applies the card is in the zone with no entity, and
"instead" has to be a move back out. That is a defect, not a modelling
choice; it is sized under `pipeline.rs` below.

**Dryad Arbor's rules text.** Correct — the parenthetical is reminder text.
**Changed:** the `rules_text` call is gone; the builder writes the intrinsic
"{T}: Add {G}." from the mana ability, which is the only text the card has.

**Keldon Warlord entering beside two creatures.** Verified against the
rulings, and the comment was inviting the misreading. There are two different
checks, and the CR gives them different answers:

- A *replacement* check (CR 614.12) happens before the object is on the
  battlefield. Thassa's ruling: "Because replacement effects are considered
  before the God is on the battlefield, the mana symbols in its mana cost
  won't be counted when determining this." The Warlord's count has the same
  shape — "creatures you control" is a count over permanents, and it is not
  one yet — so it is 2/2 to "creatures with power 2 or less enter tapped".
- A *trigger* check (CR 603.10) happens against the board after the event.
  Thassa's other ruling: devotion "including the mana symbols in the mana
  cost of the God itself" decides whether a creature entered. Welcoming
  Vampire's ruling is the same rule from the trigger side: counters and
  continuous effects the creature enters with "apply when checking to see if
  Welcoming Vampire's ability will trigger." The Warlord counts itself there
  and is 3/3, so the Vampire does not trigger.

There is no moment at which the Warlord is 2/2 *on the battlefield*. The
comment said "a moment later", which read as a battlefield state; it now
names the two checks. No Warlord-specific ruling exists; the Thassa pair is
the authority, applied by analogy, and §5a records the reasoning.

**Are all CDAs closures?** No. The closure was a construction-time helper to
build one `AmountExpr` twice, because `SetPowerToughness` takes power and
toughness by value. It ran when the card registry was built, never in a game,
so it had no performance implication. **Changed:** a value and a clone.

## `engine/layers/compute.rs`

**`EntityFacts`.** It existed because `Lookahead` held the controller, clock
and counters as three loose fields, so the accessor needed a type to return
either those or a `BattlefieldEntity`'s. **Changed:** `Lookahead` now holds a
`BattlefieldEntity` built the way `place_on_battlefield` would build it, and
`FrameCache::entity` returns `Option<&BattlefieldEntity>` for both cases. The
struct and the word "facts" are gone.

**"The look-ahead wins over a real entity…"** Rewritten. It means: if a real
entity exists for the same id, the look-ahead still answers, because the
caller asked what the object would be under the proposal. In a game the two
never coexist for an entering object; the rule is what makes
`compute_as_entering` well-defined on any id.

**`where 'l: 'a`.** The accessor returns a reference with lifetime `'a`, and
that reference comes from one of two places: `game: &'a GameState` or the
look-ahead the cache holds as `&'l Lookahead`. A `&'l T` can only be handed out
as a `&'a T` if `'l` lives at least as long as `'a`. The bound says so; without
it the compiler cannot shorten the look-ahead borrow to `'a` and the
`return Some(&l.entity)` arm does not type-check.

**`in_battlefield_zone`.** **Changed:** `in_battlefield_zone_or_entering`.

**`seed_controller`.** **Changed:** gone. `base_controller` takes the look-ahead
and has the arm as its first, so there is one definition of the pre-Layer-2
controller with one extra case rather than a wrapper around it. Not
`determine_controller`: the function that *determines* the controller is
`effective_controller`, which runs Layer 2; this is the value Layer 2 starts
from.

**`facts`.** Gone with `EntityFacts`; the binding is `entity`.

**"Bump `count` to 1 if the Warlord should count itself?"** No. It would be
wrong in both directions: in a *real* walk the object is already in
`battlefield_ids_ordered`, so it would count twice; and in a frame the +1
would count the entering object whether or not it matches the filter (a
Warlord's own filter matches it, but a "non-Wall creatures you control" count
on some other entering CDA card need not). The correct change, if the rulings
said so, is four lines in the `CountOf` arm: when the walk has a look-ahead
for an object not in the enumeration, test the filter against its frame and
add one. The rulings say not to (above).

## `engine/replacement/gather.rs`

**`.unwrap_or(false)` after `permanent_matches_filter`.** The filter returns
`Err` in exactly two cases, and both mean "does not match": the object is not
in the store (nothing can be affected by an effect on a thing that does not
exist), or `PowerLE` was asked of an object with no power (a noncreature is
not "a creature with power 2 or less"). A panic on the second would be wrong.
The collapse is safe because the leaf table is closed; a new leaf that can
`Err` for a third reason has to say what its `Err` means, and the `Or` arm's
comment records the one place the collapse is order-sensitive.

**`wanted` / `unwrap_or(true)`.** `cast` is `Option<bool>`: `Some(b)` requires
the event's cast-ness to equal `b`, and `None` means the pattern does not care.
The `unwrap_or(true)` is the `None` arm, by the field's definition.
**Changed:** the binding is `required`.

## `engine/replacement/lookahead.rs`

**`Subject` collides with `EventSubject`.** **Changed:** `FrameBasis`, held in
the field `basis`.

**When is `frame_of(id)` asked about the wrong id?** In a game, never for an
event that has a frame: `set_affects` asks about the event's subject, which is
the object the frame is for. The guard makes the function total rather than a
precondition. `is_prohibited` derives an `EntryFrame` from *any* action and
hands it to every `Filter` check, so for a `DealDamage` or an `AddCounters` on
some other object the basis is `None` or another id, and the answer is "no
frame, use the board". Without the guard every caller would have to re-derive
whether the frame applies before asking.

## `engine/replacement/pipeline.rs`

**"Mandatory" — unless all but one option are impossible?** Agreed it is thin,
and the predicate excludes optionals wholesale on purpose. The theorem needs
each member's *application* to be a fixed function of the event, and an
optional's application includes an answer the player gives. CR 616.1 asks
"may" even when only one answer is performable, so the engine cannot treat
that answer as forced. A clause for the impossible-alternatives case would add
a premise to prove and buy nothing today.

**Why does `order_invariant_entry_bucket` exist if it is fragile, and what
does it cost?** It exists because the brief asked for §11 item 19: every land
drop under Root Maze beside Idyllic Beachfront was a decision round-trip with
one outcome, and for the AI harness the prompt is the expensive thing. Cost:
it runs only when a bucket has two or more members, which is rare, and each
check is a handful of enum matches over those members. The debug re-gather
runs only in debug builds. "Fragile" here means "has expiry conditions", and
each is caught somewhere: `filter_is_mods_invariant` is matched exhaustively,
so a new leaf is a compile error; the `EnterMods` and `EventPattern` growth
cases are checked by `check_order_invariance` on every board a test or a debug
fuzz run reaches; and Deferred Migrations item 47 names them for whoever adds
one.

**"Debug-build check".** Literally `cfg!(debug_assertions)`: on for
`cargo test` and the default `cargo run`, off for `--release`, which is what
the fuzz measurement binaries are built as. The release binary trusts the leaf
table; the debug binary re-gathers and asserts.

**The `ZoneChange` into the zone is in the log — isn't that incorrect?** Yes,
and it is worse than the comment said; the deferred item now says so
(`codebase-state.md`, "Before Triggered abilities" item 4). The log holds a
zone change *into* the battlefield, a zone change *out of* it with
`from: Battlefield` and an LKI frame, and two `zone_change_epoch` bumps — for a
card the CR says was exiled from its graveyard and never entered. Keying ETB
triggers on the performer's event covers the first; nothing covers the second
or the third. No trigger matcher reads the log yet, so it is unreachable
today; it is a bug-in-waiting for item 6. The fix is to reverse the nesting so
the entry event carries `from` and its performer does the move, which keeps
the Priest in Root Maze's CR 616.1 bucket (moving the Priest to the zone change
would split it and force Priest-first). Sized at ~300–500 additions in the
item; not done in this PR because it restates a CLAUDE.md invariant and
shares its seam with RC-5 part 2.

## `engine/restriction/predicate.rs`

**`lookahead: Option<&'a EntryFrame<'a>>`.** Read inside-out: `EntryFrame<'a>`
is a frame that borrows the game for `'a`; `&'a EntryFrame<'a>` is a borrow of
one for the same `'a`; `Option` because most queries have none. `Query<'a>` is
`Copy` and carries only borrows, so the frame is owned by whoever built it —
the pipeline's per-iteration frame, or `is_prohibited`'s own — and the query
just points at it. One lifetime rather than two keeps the signature readable;
nothing needs the frame to outlive the game borrow it wraps.

**Why the `derived` block in `is_prohibited`.** CR 614.17d: a "can't" about an
entering permanent is judged against the characteristics it would have on the
battlefield. `set_affects` does that when handed a frame, so `is_prohibited`
needs one for an entry or the zone change ahead of one. When the caller
already holds a frame (the pipeline, or the synthetic `AddCounters` case) that
one is used; otherwise one is derived from the action. `let derived;`
declared before the `match` is how Rust lets one arm create an owned value
that the reference outlives: the value has to live in the enclosing scope, and
the arm assigns it exactly once. The frame computes lazily inside, so a query
with no `Filter` never pays for a layer walk, and it is built once here rather
than per sweep leg so the sweep and the registry share it.

## `engine/targeting.rs`

**Why `permanent_matches_filter_with` beside the other two?** Three names, one
leaf table. `permanent_matches_filter` computes the object's frame lazily and
once — `All`, `Token` and `ByOwner` never trigger a walk, which is what keeps
Rest in Peace free on every graveyard-bound zone change.
`permanent_matches_filter_in_frame` is the same table over a frame the caller
already holds (CR 614.12's look-ahead). `_with` is the table itself,
parameterised by "how do I get the frame", so the leaves are written once.
Merging the public two would either lose the laziness or duplicate the leaves.

## `tests/phase_rc4_integration_test.rs`

**Worms of the Earth.** It is the edge guard for one path, not a template:
"can't enter" asked at the *zone change* with the frame's `Pending` basis,
which is the only way to refuse an entry without stranding the card. The
printed family is small — Worms of the Earth, and Grafdigger's Cage's
"Creature cards in graveyards and libraries can't enter the battlefield" —
and Cage is the one that matters in play. Cage's filter is about the card
where it is, which is `ZoneChange::object` and the source zone, so it does not
exercise the frame; the test uses the Worms shape because it is the one that
does.

**The Warlord test's count.** Not a bug: the fourth creature is the opponent's.
Warlord plus your two bears is three; the Wall is excluded by name and the
opponent's bear by controller. **Changed:** the test names `you` and
`opponent`, and its message says whose creatures are counted.
