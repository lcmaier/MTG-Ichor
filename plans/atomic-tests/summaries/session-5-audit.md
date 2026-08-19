General Notes

Chapter 6 is a "difficulty spike" as far as planning a rules engine implementation reading the CR front to back. The nuances in rules like 601.2 and 601.3 (e.g., 601.3b's example of a creature with an ability that allows you to cast it as an aura (Bestow) interacting with an effect that lets you cast Auras at instant speed; this is a complex interaction between multiple parts of the rules) require careful thought during implementation, and might necessitate architectural changes now to accomodate.

Design note: Cards sometimes have card-specific rulings on their `gatherer.wizards.com` entry. These provide additional context and examples for rules interactions but should not conflict with the CR. Will be useful when implementing specific cards (e.g. all rules examples listed on the gatherer page must be correctly implemented for the card to be accepted)

Rule-Specific Notes

601.2a

Should we test casting spells from zones that aren't the hand (specifically graveyard and exile)?

601.2b

Missing one clause, the case where there is an X cost that must be chosen but it's not in the mana cost. Example:

"""
Devastating Summons

{R}

Sorcery

As an additional cost to cast this spell, sacrifice X lands.

Create two X/X red Elemental creature tokens.

"""

Even though the total cost determination and payment happens later in the casting process (601.2f and h), the X value must be chosen during this step (within spell casting proposal)

Are we worried about the last line at all? "Previously made choices (such as choosing to cast a spell with flashback from a graveyard or choosing to cast a creature with morph face down) may restrict the player’s options when making these choices." Or is the strategy to build the engine such that we get these "for free"?

601.2c

I think we need one more atomic test? These two lines are very subtle

"""

A spell may require some targets only if an alternative or additional cost (such as a
kicker cost) or a particular mode was chosen for it; otherwise, the spell is cast as though it did not require those targets. Similarly, a spell may require alternative targets only if an alternative or additional cost was chosen for it.

"""

Consider the spell Probe:
"""
Probe

{2}{U}

Kicker {1}{B}

Draw three cards, then discard two cards. If this spell was kicked, target player discards two cards.

"""
This specifically tests the line "A spell may require some targets only if an alternative or additional cost (such as a kicker cost) or a particular mode was chosen for it; otherwise, the spell is cast as though it did not require those targets." directly. To test the second line, consider the spell Bloodchief's Thirst:

"""

Bloodchief's Thirst

{B}

Sorcery

Kicker {2}{B}

Destroy target creature or planeswalker with mana value 2 or less. If this spell was kicked, instead destroy target creature or planeswalker.

"""

**Evaluate the tests against each clause in this rule, it's both important and dense so a second pass is worth it.**

In general, do we need to distinguish between "no targets" and "untargeted" at the architectural level? Since if a spell/ability has a target, that info must be included in the stack entry (even if it's later altered/invalidated)

601.2d

Should we mention prompting the DecisionProvider here?

601.2e

We should add an implementation note about snapshotting gamestate for rollbacks (which might dovetail with loop detection (see rule 731) a tricky problem I've been grappling with in previous planning sessions)

601.2f

Need a "If mutliple cost reduction effects apply, player chooses order" test. Can't think of a situation where this would matter but we can test deferral to the DecisionProvider

The 003 test is basically retested more thorougly in 601.2h right? Can we defer to that set of tests?

601.2h

There's a canonical test for "pays costs in any order", probably a good atomic/integration test. The interaction is between Omnath, Locus of Mana and Momentous Fall. First, the two cards:

"""

Omnath, Locus of Mana

{2}{G}

Legendary Creature — Elemental

You don’t lose unspent green mana as steps and phases end.

Omnath gets +1/+1 for each unspent green mana you have.

1/1

Momentous Fall

{2}{G}{G}

Instant

As an additional cost to cast this spell, sacrifice a creature.

You draw cards equal to the sacrificed creature’s power, then you gain life equal to its toughness.

"""

If you choose Omnath, how you order payment affects the card draw and life gain from Momentous Fall. If you sacrifice Omnath before paying the 4 mana (let's assume it's 4 green mana), its last known information power/toughness will be 4 greater than if you pay the mana first (because the stat boost is a characteristic-defining ability (I think this is the reason anyway))

601.2i

Should test effects that change characteristics. The Mycosynth Lattice ability is a good candidate. Has the ability "All cards that aren’t on the battlefield, spells, and permanents are colorless."

601.3

This feels meta to me. There are multiple rules and multiple categories of abilities that influence casting

601.3a

Not necessarily an issue with the test spec but this is a tricky interaction, we should think about the implementation plan with a little more scrutiny

601.3c

Should also test alternative cost. From Primal Prayers, an Enchantment: "You may cast creature spells with mana value 3 or less by paying {E} rather than paying their mana costs. If you cast a spell this way, you may cast it as though it had flash." Doesn't have to be this exact text but something like this

601.3e

The adventure example should be two integration tests (for the case where you can cast each half with flash), that's an excellent intersection of multiple parts of the rules

601.3f

Should we also test the negative case? There's also a card in exile face down from a different effect, can't cast that one (or see any information about it? This info leak check might be worth it on its own)

601.4

Is this meta? Not sure how many ways this can happen but it's worded vague enough to permit a whole class of effect interactions

602.1e

This test will incorrectly pass if neither effect applies, we should have an explicit test for just one of the conditions

602.2a

Should we also test the lack of other characteristics for ability objects (most prominently no card name or mana cost)

602.4

For reference, I think the only way to actually invoke this rule with the current card pool is a contrived scenario involving the following card (worth noting this came out in the last 12 months so another similar design could be printed in the future):

"""
Urianger Augurelt

{W}{U}

Legendary Creature — Elf Advisor

Whenever you play a land from exile or cast a spell from exile, you gain 2 life.

Draw Arcanum — {T}: Look at the top card of your library. You may exile it face down.

Play Arcanum — {T}: Until end of turn, you may play cards exiled with Urianger Augurelt. Spells you cast this way cost {2} less to cast.

1/3

"""

The scenario I'm imagining is one where you've exiled multiple cards to the first tap effect. You've already tapped him this turn and cast a spell from his exile pool. You then cast an instant speed untap effect to untap him and tap him again. The spell you already cast doesn't get an additional {2} discount (because you already paid all the costs--does this implicitly handle all these cost-changing rules?). Might be worth an integration test

602.5c

Good integration test candidate: Necrotic Ooze with 2 Skinshifters in the graveyard. Card text for reference:

"""

Skinshifter

{1}{G}

Creature — Human Shaman

{G}: Choose one. Activate only once each turn.

• Until end of turn, this creature becomes a Rhino with base power and toughness 4/4 and gains trample.

• Until end of turn, this creature becomes a Bird with base power and toughness 2/2 and gains flying.

• Until end of turn, this creature becomes a Plant with base power and toughness 0/8.

1/1

Necrotic Ooze

{2}{B}{B}

Creature — Ooze

As long as this creature is on the battlefield, it has all activated abilities of all creature cards in all graveyards.

4/3

"""

The Ooze should have two abilities, each of which can be activated once each turn (integrates with CDAs).

602.5d

More of a design note but the sorcery-speed timing restriction check can be one chunk of code we can reuse everywhere

602.5e

Explanation isn't right, this is a niche effect that's designed to prevent mana abilities from causing weird side effects (primary card: Lion's Eye Diamond)

603.1b

This rule is setting off "we need to make architectural decisions to accomodate this now" alarms in my head. What do you think?

603.2b

Combination/Integration test idea: multiple "at the beginning of each upkeep" triggers for all players, test APNAP and player-chosen ordering within that and multiple simultaneous triggers.

603.2d

Similar alarm bells to 603.1b. Can/Should we just treat this as a game-engine level replacement effect? Is it too early to tell?

603.2e

Another good test (not sure if atomic though): activate the Equip ability of an equipment targeting the creature it's already attached to. Equipment has an ability that  triggers when it "becomes attached" *doesn't* trigger in this instance.

 603.2f

This rule has revealed a really nasty rules scenario (also overlaps with 603.10) that should maybe be a stress test but is so gnarly I want to confirm this architecture and plan can even conceputally handle it. See `plans/atomic-tests/603-2f-complexity.md` for details

603.2h

Should also test "May do this once each turn"  with multiple triggers on the stack. Example:

Relevant Card:

"""

Nykthos Paragon
{4}{W}{W}
Enchantment Creature — Human Soldier

Whenever you gain life, you may put that many +1/+1 counters on each creature you control. Do this only once each turn.

4/6

"""

Interaction:

"""

Two lifelink creatures deal combat damage simultaneously while you control Nykthos Paragon. Two Paragon triggers go on the stack (ordered as you choose, but this isn't directly testing that). You choose to take the action on the first one resolving. The second trigger resolves without effect (DecisionProvider not prompted for any choices)

603.3

Similar to activated abilities, should we have a test that checks a triggered ability object's characteristics

603.3a

There are few (none that I can think of) effects that would change control of a permanent between priority rounds. Only idea I had is multiplayer and a player with a control-changing effect concedes the game? Will likely be more clear once we reach this in the implementation

603.3b

To clarify the class of triggered abilities that are excluded from the first phase of putting triggers on the stack, consider these abilities:

"""

Whenever a permanent entering causes a triggered ability to trigger, counter that ability unless its controller pays {2}.

*Probing Telepathy* — Whenever a creature entering under an opponent’s control causes a triggered ability of that creature to trigger, you may copy that ability. You may choose new targets for the copy.

"""

(from Strict Proctor and Aboleth Spawn respectively)

We'll need some way to segregate these abilities from other triggered abilities for this two-tier process. We should probably also test the two-tier system here

603.3c

Should also test "if no mode is chosen ability is removed from stack" (fizzles). Triggered ability that says "When \[condition\], choose one: Destroy target artifact or destroy target enchantment" with neither permanent type on the battlefield

603.3d

Is this testable? Or are we folding it in with the 601.2 tests? Feel like if we're saying it's testable we should split out and test each step (but idk if that's necessary).

603.4

001 seems like it should be split into 2 tests (one where the condition isn't still true at resolution and one where it is)

603.6

Overlaps with proposed Last Known Information (LKI) system

603.6a

We should have the entering creature also have an ETB effect (to test the fact that *all* permanents get checked for triggers, including newcomers)

603.6b

Should think about how we're implementing this rule carefully. This is subtley different from replacement effects, it's more like "continuous effects 'get there first'", before triggered abilities check

603.6c

The rule notes an edge case that can come up in multiplayer: "Leaves-the-battlefield abilities trigger...when a phased-in permanent leaves the game because its owner leaves the game." Consider this triggered ability from Extractor Demon: "Whenever another creature leaves the battlefield, you may have target player mill two cards." If Player A controls Extractor Demon and Player B loses the game with 10 creatures, Player A should now get 10 Extractor Demon triggers, which they can point at either themselves or any remaining player (each one will require a choice from Player A given the Demon's ability's wording).

Should also test the line "An ability that attempts to do something to the card that left the battlefield checks for it only in the first zone that it went to." The enchantment "Enduring Renewal" has an ability "Whenever a creature is put into your graveyard from the battlefield, return it to your hand." If the object that ability is acting on gets exiled by an instant speed "Exile target card from a graveyard" ability, triggered ability will resolve without effect.

Do we also want to do the negative case mentioned about a "from anywhere" ability not being an LTB even if it goes from the battlefield (though admittedly I'm not sure what an effect that would care about that distinction would even look like)

603.6e

Should also have a test for finding the object the Aura was attached to. Consider the ability of Abduction, an aura with "When enchanted creature dies, return that card to the battlefield under its owner’s control."

603.7a

The rule might say the trigger gets created but if the trigger event is impossible (e.g. a trigger that was going to affect a creature that left the battlefield before the effect that created the trigger resolved) wouldn't registering the trigger be registering orphaned data that will never be accessed and just takes up overhead? Is there a reasonable way to do this or would it just be a list of hooks that would catch certain scenarios and not register triggers?

603.7b

We should also test the "If its trigger event occurs more than once simultaneously and the ability doesn’t have a stated duration, the controller of the delayed triggered ability chooses which event causes the ability to trigger." line. Consider the interaction between any generic creature token doubler (like Anointed Procession) and this card:

"""

Tatsumasa, the Dragon's Fang
{6}
Legendary Artifact — Equipment

Equipped creature gets +5/+5.

{6}, Exile Tatsumasa: Create a 5/5 blue Dragon Spirit creature token with flying. Return Tatsumasa to the battlefield under its owner’s control when that token dies.

Equip {3}

"""

If both tokens die simultaneously, controller of the Tatsumasa ability decides which one is the source for the trigger (I think) (this is also very niche). 

603.7c

Isn't this explicitly different than how targeting works (characteristic change can make target invalid on resolution). I guess it's to get around the complexity of "on resolution"? Regardless, what are the options for architecturally handling this difference? Just a separate system for delayed triggers?

603.7f

Atomic is a little vague, though I can't find good examples elsewhere. Maybe we make a note to specify the test more later.

603.8

The example with the "discard your hand, then draw that many cards" triggering it is probably an issue. Need to think about how to handle that architecturally

603.10

Document structure issue--the top level rule is purely definitional I think. The example you cite is nested in 603.10a

603.10c

Should also test attaching to a different creature still triggers "becomes unattached" effects

604.5, 604.6

Should we test the separate conditions individually? 

604.7

Example is wrong. This rule is actually much nicher than that. Example I was given was regarding this card:
"""

Saproling Burst
{4}{G}
Enchantment

Fading 7 *(This enchantment enters with seven fade counters on it. At the beginning of your upkeep, remove a fade counter from it. If you can’t, sacrifice it.)*

Remove a fade counter from this enchantment: Create a green Saproling creature token. It has “This token’s power and toughness are each equal to the number of fade counters on Saproling Burst.”

When this enchantment leaves the battlefield, destroy all tokens created with this enchantment. They can’t be regenerated.

"""

The idea is if you put the activated ability from Saproling Burst on the stack, but before it resolved the Saproling Burst was destroyed, when the token entered it would enter as a 0/0 since it can't reference SB's last known information

605.1a, b

Should have tests for each individual qualifier (i.e. examples where the other two conditions are met but the third fails, and thus it isn't a mana ability and uses the stack). The trivial two are "doesn't add mana" (extra trivial, a mana ability has to add mana), plus a loyalty ability that just says "Add {R}{R}" or something (a Chandra actually has this exact ability iirc). Explosive Welcome (below) handles the one nontrivial case: satisfying (2) and (3), but it requires a target. 
"""

Explosive Welcome
{7}{R}
Instant

Explosive Welcome deals 5 damage to any target and 3 damage to any other target. Add {R}{R}{R}.

"""

And should we test triggered vs activated individually? Or will the handling be similar enough that one set of tests can cover both? Or is all of this subsumed into 605.5 tests?

605.3a

Multi-clause. Need to test them all (probably)

605.3c

After paying the tap cost the cost checker would deny a reactivation anyway right? We need a separate check for mana abilities that could theoretically be activated before resolution if this rule didn't exist (probably mana filtering abilities, stuff like "{1}: Add one mana of any color"?)

606.4

Integration/composition test idea: Counter doubling effect ("If you would put one or more counters on a permanent you control, put twice that many instead) and Planeswalker +1 providing +2 loyalty to the permanent

606.6

Edge case interaction with 606.5 (cost that normally wouldn't be able to be paid is paid by an additional +1 cost to activate a loyalty ability)

607.1c

The example is wrong. Both criteria must be satisfied by *one* ability for this rule to apply. Consider the following card as a guiding example:

"""
Tyrant's Choice
{1}{B}
Sorcery

*Will of the council* — Starting with you, each player votes for death or torture. If death gets more votes, each opponent sacrifices a creature of their choice. If torture gets more votes or the vote is tied, each opponent loses 4 life.

"""

607.1d

I think your test works but it might be worth holding off as I cannot find any examples of this rule anywhere and I'm not 100% sure how wide its purview is. A retrofit cost should be low since it's just carving out an exception that "two objects can have linked abilities between them under certain circumstances" which is by its nature an exception change, not something fundamental we have to do to the engine. Does that plan seem reasonable?

607.2

Got some specific card examples for each of these just to bolster the tests, they can be found in `plans/atomic-tests/607-2-examples.md`. We don't have to use those particular cards (plus we don't even have examples for all the subrules) but they're useful for informing our tests.



608.2

Are we sure this doesn't need atomic testing?



608.2b

Should also check the case where a spell has multiple targets and some (but not all) become illegal--spell should still resolve. Example:
"""
Jagged Lightning
{3}{R}{R}
Sorcery

Jagged Lightning deals 3 damage to each of two target creatures.

"""



608.2c

Not testable in our engine I'm pretty sure (this is a text formatting concern--our engine deals with pre-tokenized structures representing these text discrepancies)



608.2d

I don't think 002 is capturing all the nuance of this clause. Maybe another test to show Player A can choose just one creature (in which case 3/0 distribution would be legal), but if they choose two then they have to distribute at least one counter among each chosen.



608.2f

We also need a test of the 2nd example (if there's one in the comp or integration section I missed disregard this), it's critical to how the rule works



608.2g

Important to note this doesn't give the spell you cast functional uncounterability. Once the spell that's resolving is done, players will get a chance to respond to the one or more spells you cast during the resolution of the previous effect, just not before that.



608.2i

Note that the Fight mechanic is an exception to this (which we'll get a better idea of when we reach that rule in chapter 7)



608.2m

Might not be testable, could be a catachall. Don't know for sure though.



608.2p

Test is wrong, currently tests cast triggers, not resolution triggers (much narrower set of cards). Example:
"""
Maelstrom Muse
{1}{U}{U/R}{R}
Creature — Djinn Wizard

Flying

Whenever this creature attacks, the next instant or sorcery spell you cast this turn costs {X} less to cast, where X is this creature’s power as this ability resolves.

2/4

"""



608.3a

Note interaction with other rules that explicitly prevent instants and sorceries from entering the battlefield (essentially making this rule "safe")



608.3b

Should also test the mutate case



608.3d

Not out of scope, mutating permanents are in popular formats








