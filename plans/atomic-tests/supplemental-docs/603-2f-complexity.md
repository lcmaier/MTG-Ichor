The Cards:

"""

Library of Leng
{1}
Artifact

You have no maximum hand size.

If an effect causes you to discard a card, discard it, but you may put it on top of your library instead of into your graveyard.

Guerrilla Tactics
{1}{R}
Instant

Guerrilla Tactics deals 2 damage to any target.

When a spell or ability an opponent controls causes you to discard this card, it deals 4 damage to any target.

Mind Peel
{B}
Sorcery

Buyback {2}{B}{B} [NOTE: Buyback is irrelevant in this scenario]

Target player discards a card.

Coercion
{2}{B}
Sorcery

Target opponent reveals their hand. You choose a card from it. That player discards that card.

Telepathy
{U}
Enchantment

Your opponents play with their hands revealed.

Future Sight
{2}{U}{U}{U}
Enchantment

Play with the top card of your library revealed.

You may play lands and cast spells from the top of your library.

"""

The Scenario

"""

Player A controls a Library of Leng and has Guerrilla Tactics in hand. Player B casts Mind Peel targeting Player A. Player A discards the Guerrilla Tactics.

"""

Rules Questions and Answers

"""

Does Guerrilla Tactics triggered ability trigger?

Answer: No. It goes directly from your hand to your library, one hidden zone to another, and is never revealed, so it cannot trigger.

If Player B cast Coercsion instead of Mind Peel, would the Tactics trigger?
Answer: No, the card is never in a public zone after it is discarded, so the triggered ability has no way to trigger (since its source isn't in an appropriate zone (I think this is the reason? The ruling is right but the reasoning is my best attempt at interpreting why)). This is also true if Mind Peel was cast while a Telepathy was affecting Player A

Do any of these answers change if Player A controls a Future Sight?
Answer: Yes, (presumably) all of them. From the ruling thread: "Immediately after the discard event, it is revealed to all players and has a triggered ability that triggers on that discard."

"""

On this last point, further consider this ruling specific to Library of Leng from the Gatherer rulings: "If more than one card is discarded due to a single effect, the Library allows you to decide whether or not to use it on each of the cards. You get to decide the order the cards are placed on the library if more than one goes there," If an effect caused Player A to discard multiple cards, and Player A controls a Future Sight, does the order they decide to put the cards on top of their library affect if Tactics damage ability triggers? Does the Leng's "one at a time" ruling mean it happens no matter what? I'm unsure. This entire scenario is pulled from an MTG judge forum thread, it might not even be the right ruling since this is just one judge's opinion.

This is emblematic of a broader design concern that must be tackled before implementation can happen in earnest: MtG is arbitrarily complex. Even the comprehensive rules cannot and do not cover every single possible edge case. How do we handle this complexity? Let our system compose as architected, trusting the unit and composition tests to give us correct results where "correct results" can be well-defined?
