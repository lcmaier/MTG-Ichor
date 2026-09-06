# Blood Moon + Urborg + Ashaya + Opalescence — a judge's layer-4 walkthrough

**What this is.** A judge's answer on a Reddit rules thread, recovered from the
owner's browser history on 2026-09-06 and quoted below as posted (the thread's
URL was not recorded; the answer is the artifact). It is the worked example
that `codebase-state.md` "Before Layers" item 8 had been citing from memory as
"the judge walkthrough". **It is not a ruling.** The rulings for the pairs it
contains are on Scryfall — Urborg (2021-03-19) and Rootpath Purifier
(2022-10-14) for the Blood Moon dependencies, Humility / Opalescence
(2009-10-01, 2006-02-01) for the timestamp cases — and `layers-architecture.md`
§13b quotes them. What this answer adds is the *procedure*, applied to four
effects in one layer, which no ruling walks.

**What it settles for the engine** (`layers-architecture.md` §13b, LI-2):

- "You determine dependencies before applying any of the effects and
  re-evaluate after each effect is applied" — CR 613.8c is a loop, one
  application per iteration, dependencies recomputed over the remaining set.
- "The dependency has to be actual for the current game state being built,
  not theoretical" — the hypothetical check runs against the live board (§13b
  decision 3), and "Blood Moon is dependent on Ashaya *if there are nontoken
  creatures*" is the check finding a member whose match flips.
- The pairs are checked in both directions, and an effect with no unapplied
  dependency is applied first; when only one is independent, it is the next
  application. Four effects, six pairs, three rounds.
- The answer for the four-card board: **Opalescence, then Ashaya, then Blood
  Moon; Urborg never applies.** This is LI-2's CR 613.8c test once
  Opalescence (one new filter leaf, "each other") and Ashaya are registered
  beside Blood Moon and Urborg, with the sequence asserted step by step.

**A detail the walkthrough leaves implicit.** After Ashaya applies, Blood Moon
is itself a nontoken creature-land and so a nonbasic land; when Blood Moon
then applies, it applies to itself too — a Mountain that loses its abilities
under CR 305.7. That does not undo its effect: it has already applied, and
CR 613.6 keeps a started effect applying. The engine's locked set
(`Board::started`) and the "already applied" state of the loop are what carry
that.

---

## The answer, as posted

> **Poster:** I understand that normally, Blood Moon + Urborg is answered by
> the concept of dependencies. Urborg's status depends on Blood Moon's effect
> while Blood Moon isn't affected by Urborg, so you apply Blood Moon's effect
> first, turning Urborg into a mountain before it can make everything also a
> swamp.
>
> But what if Blood Moon is also a land due to Opalescence and Ashaya? Now
> Blood Moon's type depends on Urborg, and Urborg's type depends on Blood
> Moon.
>
> **Judge:** You determine dependencies before applying any of the effects
> and re-evaluate after each effect is applied. Additionally, the dependency
> has to be actual for the current game state being built, not theoretical.
>
> You determine dependencies by pairing up effects.
>
> All four apply in layer 4 and there are six combinations to check for
> dependencies.
>
> 1 - Ashaya + Blood Moon
> Blood Moon is dependent on Ashaya if there are nontoken creatures which
> Ashaya presumably is because Ashaya would increase the set of nonbasic
> lands that Blood Moon would apply to
> Ashaya is not dependent on Blood Moon
>
> 2 - Ashaya + Opalescence
> Ashaya is dependent on Opalescence because Opalescence makes more creatures
> for Ashaya to turn into Forests
> Opalescence isn't dependent on Ashaya.
>
> 3 - Ashaya + Urborg
> Urborg is dependent on Ashaya because Ashaya makes creatures lands for
> Urborg to apply to
> Ashaya isn't dependent on Urborg
>
> 4 - Blood Moon + Opalescence
> neither is dependent on the other
>
> 5 - Blood Moon + Urborg
> Urborg is dependent on Blood Moon because applying Blood Moon would remove
> Urborg's effect
> Blood Moon isn't dependent on Urborg
>
> 6 - Opalescence + Urborg
> neither is dependent on the other
>
> So we've identified four dependencies in the initial game state: Urborg is
> dependent on both Blood Moon and Ashaya. Blood Moon is dependent on Ashaya.
> Ashaya is dependent on Opalescence. Only one effect is independent -
> Opalescence so we apply it.
>
> Opalescence makes Blood Moon a creature.
>
> We re-evaluate dependencies with the now current game state and have 3
> effects so 3 dependencies to check for.
>
> 1 - Ashaya + Blood Moon
> Blood Moon is dependent on Ashaya if there are nontoken creatures which
> Ashaya and Blood Moon presumably are because Ashaya would increase the set
> of nonbasic lands that Blood Moon would apply to
> Ashaya is not dependent on Blood Moon.
>
> 2 - Ashaya + Urborg
> Urborg is dependent on Ashaya because Ashaya makes creatures lands for
> Urborg to apply to
> Ashaya isn't dependent on Urborg
>
> 3 - Blood Moon + Urborg
> Urborg is dependent on Blood Moon because applying Blood Moon would remove
> Urborg's effect
> Blood Moon isn't dependent on Urborg
>
> In this game state there are three dependencies: Urborg is dependent on
> both Ashaya and Blood Moon. Blood Moon is dependent on Ashaya. Ashaya is
> the only independent effect so we apply it.
>
> Ashaya adds Forest Land to herself, Blood Moon and any other nontoken
> creatures.
>
> We re-evaluate dependencies with the now current game state and have 2
> effects so 1 dependency to check for.
>
> 1 - Blood Moon + Urborg
> Urborg is dependent on Blood Moon because applying Blood Moon would remove
> Urborg's effect
> Blood Moon isn't dependent on Urborg
>
> There's one dependency, Urborg on Blood Moon so Blood Moon is the only
> independent effect and is applied.
>
> Blood Moon changes all nonbasic lands to type Mountain removing existing
> abilities and adding the ability to tap for red mana, affecting Ashaya and
> Urborg and any other nontoken creatures.
>
> Urborg no longer has an effect so we're done in layer 4 and move to
> layer 5.
