//! Dragon's Maze (`dgm`), released 2013-05-03.

use std::sync::Arc;

use crate::cards::authoring::static_replacement;
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{PlayerRef, PlayerSet};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite, TemplateAmount};
use crate::types::zones::DrawFilter;

/// Notion Thief — {2}{U}{B}
/// Creature — Human Rogue 3/1
///
/// > Flash
/// > If an opponent would draw a card except the first one they draw in each of
/// > their draw steps, instead that player skips that draw and you draw a card.
///
/// **The same event with a new subject**, and its own ruling is the only reason
/// to know that. Read as English it is Alms Collector's shape — a skip and a
/// draw joined by "and" — and §3.2d filed it as `Prevent` plus a rider until
/// the rulings pass. The ruling walks two Thieves, says each is *"applied to
/// the card draw only once"*, and concludes that in a two-player game *"it
/// really will be that player who draws a card"*. A rider's draw is a fresh
/// proposal with a fresh applied set, so two Thieves as riders would hand the
/// draw back and forth forever. As `Instead(DrawCards { n: 1, player: You })`
/// the draw keeps its lineage: each Thief applies once, and the draw comes home.
///
/// `n: 1` rather than a `DrawCard`, because the substitute for a draw is always
/// the instruction (CR 121.2a) — which also means the Thief's own draw is a
/// draw proposed like any other, and a second Thief can take it.
/// # The rulings, and where each is tested
///
/// - *"If an opponent is instructed to draw a card then discard a card, and
///   [this] causes you to draw a card instead, that opponent still discards a
///   card. The same is true of any other actions that opponent is instructed to
///   do."* → tested on the second sentence: `Primitive::Discard` is
///   `NotImplemented` until RE-8, so the board is a draw-then-lose-life
///   resolution and the assertion is that the opponent still loses the life.
///   → `the_opponents_other_instructions_still_happen`
/// - *"If two or more players each control [one] ... that player chooses one ...
///   Then the player whose [effect] was chosen repeats this process among the
///   remaining ... Each effect can be applied to the card draw only once."* →
///   the three-player test, and every sentence falls out rather than being
///   coded. → `three_notion_thieves_pass_the_draw_once_each_in_the_rulings_order`
/// - *"[So] if each player in a two-player game controls [one] and one would
///   draw a card, it really will be that player who draws a card."* → test, and
///   the same mechanism observed from outside.
///   → `two_notion_thieves_hand_the_draw_across_the_table_and_back`
pub fn notion_thief() -> Arc<CardData> {
    CardDataBuilder::new("Notion Thief")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Black], 2))
        .color(Color::Blue)
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Rogue))
        .power_toughness(3, 1)
        .rules_text(
            "Flash\nIf an opponent would draw a card except the first one they draw in each of their draw steps, instead that player skips that draw and you draw a card.",
        )
        .ability(static_replacement("If an opponent would draw a card except the first one they draw in each of their draw steps, instead that player skips that draw and you draw a card.",
            ReplacementDef::new(
                EventPattern::DrawCard { place: Some(DrawFilter::NotFirstInDrawStep) },
                PlayerSet::Opponents,
                Rewrite::Instead(GameActionTemplate::DrawCards {
                    n: TemplateAmount::Fixed(1),
                    player: Some(PlayerRef::You),
                }),
            ),
        ))
        .build()
}
