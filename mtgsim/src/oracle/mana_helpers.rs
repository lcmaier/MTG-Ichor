// The checks the priority question builds its options by: whether a card can
// be cast and an ability activated now (CR 601.3, 602.2), each refusal a
// typed reason the why panel words. The mana half asks `oracle::mana_supply`.
// All functions are read-only queries over &GameState.

use std::cell::OnceCell;

use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction};
use crate::oracle::mana_supply::{ManaSupply, NonManaCosts};
use crate::state::game_state::GameState;
use crate::types::card_types::CardType;
use crate::types::costs::Cost;
use crate::types::effects::{EffectRecipient, TargetCount};
use crate::types::ids::{AbilityId, ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};
use crate::types::zones::Zone;
use crate::engine::costs::CannotPay;
use crate::engine::put_on_stack::SorceryTiming;

/// Whether `player_id` can pay the mana `cost` asks, for `payment`: from the
/// pool alone, or from all they can make, the inventory `supply` holds once
/// a check needs it. The priority question's every check shares one
/// (`mana-architecture.md` §3.1).
fn mana_payable(
    game: &GameState,
    player_id: PlayerId,
    cost: &ManaCost,
    non_mana: &NonManaCosts<'_>,
    supply: &OnceCell<ManaSupply>,
) -> bool {
    game.players.get(player_id).is_some_and(|player| player.mana_pool.can_pay(cost))
        || supply.get_or_init(|| ManaSupply::read(game, player_id)).covers(game, cost, non_mana)
}

/// Why a card is not offered to cast at a priority question.
///
/// The first four are CR 601.3's "can begin to cast", which
/// [`can_begin_to_cast`] asks for the enumeration and for the cast alike. The
/// last three predict what CR 601.2c and 601.2h would refuse: the enumeration
/// leaves out a cast that could not be completed, and the cast itself finds
/// them by trying, and reverses what it began (CR 732.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CannotCast {
    /// Not in the caster's hand, the one zone the engine casts from
    /// (`backlog.md` §2.3).
    NotInHand,
    /// A land is played, never cast (CR 305.9).
    Land,
    /// Neither a permanent card nor one with a spell ability, so there is
    /// nothing to resolve.
    NoSpellAbility,
    /// Neither an instant nor with flash, and outside sorcery timing
    /// (CR 117.1a).
    Timing(SorceryTiming),
    /// An instance of "target" with no legal choice (CR 601.2c).
    NoLegalTarget,
    /// A mandatory additional cost that can't be paid (CR 601.2h).
    AdditionalCost(CannotPay),
    /// The mana its caster's pool and untapped sources can make does not
    /// cover the total cost CR 601.2f would lock in.
    ManaShort,
}

/// The engine's own words, for an error a caller returns as text.
impl std::fmt::Display for CannotCast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CannotCast::NotInHand => f.write_str("Card is not in its caster's hand"),
            CannotCast::Land => f.write_str("A land is played, never cast"),
            CannotCast::NoSpellAbility => f.write_str("Card has no spell ability"),
            CannotCast::Timing(timing) => timing.fmt(f),
            CannotCast::NoLegalTarget => f.write_str("No legal choice for one of its targets"),
            CannotCast::AdditionalCost(cost) => cost.fmt(f),
            CannotCast::ManaShort => f.write_str("Not enough mana"),
        }
    }
}

/// CR 601.3: may `player_id` begin to cast `card_id`? The enumeration asks it,
/// and the cast asks it again before it moves the card, so the two agree.
pub fn can_begin_to_cast(game: &GameState, player_id: PlayerId, card_id: ObjectId) -> Result<(), CannotCast> {
    let Some(obj) = game.objects.get(&card_id) else {
        return Err(CannotCast::NotInHand);
    };
    // A hand holds only its owner's cards (CR 400.3), so this is "in the
    // caster's hand".
    if obj.zone != Zone::Hand || obj.owner != player_id {
        return Err(CannotCast::NotInHand);
    }
    // PRE-LAYER ZONE: reads printed types on purpose. This is cast-zone /
    // play-from-hand legality, which happens before the object is a permanent,
    // so the layer system has nothing to contribute. Same exemption as
    // engine/cast.rs -- see "Before Layers" in plans/codebase-state.md.
    if obj.card_data.types.contains(&CardType::Land) {
        return Err(CannotCast::Land);
    }
    let has_spell_ability = obj.card_data.abilities.iter().any(|a| a.ability_type == AbilityType::Spell);
    if !has_spell_ability && !obj.card_data.types.iter().any(|t| t.is_permanent()) {
        return Err(CannotCast::NoSpellAbility);
    }
    // Through the layers: a row can give a card in hand flash.
    if !crate::oracle::characteristics::is_instant_or_has_flash(game, card_id) {
        game.check_sorcery_timing(player_id).map_err(CannotCast::Timing)?;
    }
    Ok(())
}

/// Is `card_id` offered to `player_id` to cast: CR 601.3's start, then what
/// CR 601.2c and 601.2h would refuse, the mana last.
pub fn can_cast(game: &GameState, player_id: PlayerId, card_id: ObjectId) -> Result<(), CannotCast> {
    can_cast_with(game, player_id, card_id, &OnceCell::new())
}

/// [`can_cast`], paying from `supply`, the inventory a priority point's
/// checks share.
fn can_cast_with(
    game: &GameState,
    player_id: PlayerId,
    card_id: ObjectId,
    supply: &OnceCell<ManaSupply>,
) -> Result<(), CannotCast> {
    can_begin_to_cast(game, player_id, card_id)?;
    let Some(obj) = game.objects.get(&card_id) else {
        return Err(CannotCast::NotInHand);
    };

    // Target legality check (rule 601.2c): can't cast a spell that
    // requires targets if no legal target exists. Asked of the card, not
    // the spell ability — an Aura's target is its enchant ability
    // (CR 303.4a) and it has no spell ability to ask.
    if !every_instance_has_a_choice(game, &obj.card_data.spell_instances, player_id, card_id) {
        return Err(CannotCast::NoLegalTarget);
    }

    // A mandatory additional cost is part of what casting takes, so a
    // spell whose mandatory cost is unpayable is not castable (CR 601.2h, "unpayable
    // costs can't be paid"). Enumeration and enforcement must agree
    // (`cost-architecture.md` §3.6): without this, Altar's Reap is offered
    // with no creature on the board and the cast rolls back. Optional
    // costs are not checked — declining one is always available.
    //
    // Only the non-mana part: a mandatory cost's own mana is inside the
    // total `preview_mana_cost` returns below, and asking `can_pay_costs`
    // about it here would test it against a pool that has not been filled
    // by 601.2g yet.
    let mandatory_non_mana: Vec<Cost> = obj.card_data.additional_costs
        .iter()
        .filter(|c| !c.is_optional())
        .flat_map(|c| c.costs().iter())
        .filter(|c| !matches!(c, Cost::Mana(_)))
        .cloned()
        .collect();
    if !mandatory_non_mana.is_empty() {
        game.can_pay_costs(&mandatory_non_mana, player_id, card_id).map_err(CannotCast::AdditionalCost)?;
    }

    // Check mana affordability — against the cost CR 601.2f would lock
    // in, not the printed one. Enumeration and enforcement must agree
    // (`cost-architecture.md` §3.6): a Thalia on the board would
    // otherwise offer spells the cast then rolls back, and an
    // Electromancer would withhold ones the player can afford.
    let Some(printed) = &obj.card_data.mana_cost else {
        return Ok(());
    };
    let mana_cost = crate::engine::cost_determination::preview_mana_cost(game, card_id, printed);
    // The mandatory sacrifices are paid at 601.2h, after the window, so what
    // they take is not there to make mana with.
    let non_mana = NonManaCosts { source: None, costs: &mandatory_non_mana };
    if !mana_payable(game, player_id, &mana_cost, &non_mana, supply) {
        return Err(CannotCast::ManaShort);
    }
    Ok(())
}

/// The cards in `player_id`'s hand that [`can_cast`] offers.
pub fn castable_spells(game: &GameState, player_id: PlayerId) -> Vec<ObjectId> {
    castable_spells_with(game, player_id, &OnceCell::new())
}

/// [`castable_spells`], paying from `supply`.
pub(crate) fn castable_spells_with(
    game: &GameState,
    player_id: PlayerId,
    supply: &OnceCell<ManaSupply>,
) -> Vec<ObjectId> {
    let Some(player) = game.players.get(player_id) else {
        return Vec::new();
    };
    player.hand.iter().copied().filter(|&card_id| can_cast_with(game, player_id, card_id, supply).is_ok()).collect()
}

/// Color-sensitive subtraction of pool mana from a mana cost.
///
/// For each colored symbol in the cost, if the pool has that color available
/// (beyond what earlier symbols already consumed), skip the symbol. For generic
/// symbols, subtract any excess pool mana. Returns a new ManaCost representing
/// only the portion that must still be covered by tapping sources, its symbols
/// in the cost's own order, which is how a payment prompt prints it.
pub(crate) fn remaining_cost_after_pool(
    cost: &ManaCost,
    pool: &crate::types::mana::ManaPool,
) -> ManaCost {
    // Snapshot pool amounts so we can "spend" conceptually without mutating
    let mut available: std::collections::HashMap<ManaType, u64> = pool.available().clone();
    let mut covered = vec![false; cost.symbols.len()];

    // First pass: colored/colorless symbols, which only their own mana pays.
    // Hybrid/Phyrexian/X can't auto-subtract and are kept as-is.
    for (sym, covered) in cost.symbols.iter().zip(covered.iter_mut()) {
        let mana_type = match sym {
            ManaSymbol::Colored(mt) => *mt,
            ManaSymbol::Colorless => ManaType::Colorless,
            _ => continue,
        };
        let avail = available.entry(mana_type).or_insert(0);
        if *avail > 0 {
            *avail -= 1; // pool covers this symbol
            *covered = true;
        }
    }

    // Second pass: generic symbols can be paid by any remaining pool mana
    let mut excess: u64 = available.values().sum();
    for (sym, covered) in cost.symbols.iter().zip(covered.iter_mut()) {
        if *sym == ManaSymbol::Generic && excess > 0 {
            excess -= 1; // pool covers this generic
            *covered = true;
        }
    }

    ManaCost::from_symbols(cost.symbols.iter().zip(covered).filter(|(_, covered)| !covered).map(|(sym, _)| *sym).collect())
}

/// Why an ability is not offered to activate at a priority question.
///
/// The first five are CR 602.2's and CR 602.5's "can begin to activate",
/// which [`can_activate_its_abilities`] and [`can_begin_to_activate`] ask for
/// the enumeration and for the activation alike. The last three predict what
/// CR 602.2b's run of 601.2c and 601.2h would refuse, which the activation
/// finds by trying, and reverses (CR 732.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CannotActivate {
    /// Not on the battlefield, the one zone the engine activates from
    /// (`backlog.md` §2.8).
    NotOnBattlefield,
    /// Only its controller activates it (CR 602.2).
    NotYours,
    /// A mana ability: the engine offers one when a cost asks for mana
    /// (CR 601.2g), and never at a priority question, though CR 605.3a
    /// allows it there.
    ManaAbility,
    /// Not an activated ability at all.
    NotActivated,
    /// "Activate only as a sorcery", outside sorcery timing (CR 602.5d).
    Timing(SorceryTiming),
    /// A cost other than mana that can't be paid (CR 118.3).
    Cost(CannotPay),
    /// The mana its controller's pool and untapped sources can make does not
    /// cover its mana cost.
    ManaShort,
    /// An instance of "target" with no legal choice (CR 602.2b, 601.2c).
    NoLegalTarget,
}

/// The engine's own words, for an error a caller returns as text.
impl std::fmt::Display for CannotActivate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CannotActivate::NotOnBattlefield => f.write_str("Permanent not on battlefield"),
            CannotActivate::NotYours => f.write_str(
                "Only this permanent's controller can activate its abilities (CR 602.1a; \"any player may activate\" is not yet modeled)",
            ),
            CannotActivate::ManaAbility => f.write_str("Use activate_mana_ability for mana abilities"),
            CannotActivate::NotActivated => f.write_str("Not an activated ability"),
            CannotActivate::Timing(timing) => timing.fmt(f),
            CannotActivate::Cost(cost) => cost.fmt(f),
            CannotActivate::ManaShort => f.write_str("Not enough mana"),
            CannotActivate::NoLegalTarget => f.write_str("No legal choice for one of its targets"),
        }
    }
}

/// CR 602.2: "Only an object's controller ... can activate its activated
/// ability". The source's half of the check, asked once for all its
/// abilities.
///
/// CR 602.2's *default*, not a universal rule: 41 printed cards say "Any
/// player may activate this ability" (Aether Storm, Excavation, Feral Hydra).
/// That permission is unmodeled — `AbilityDef` has nowhere to record it — so
/// this refuses an activation those cards would allow. Deferred Migrations,
/// "Before card breadth".
///
/// Effective controller, not the battlefield field, so a stolen permanent
/// answers to whoever stole it (CR 613.1b).
pub fn can_activate_its_abilities(game: &GameState, player_id: PlayerId, source_id: ObjectId) -> Result<(), CannotActivate> {
    if !crate::oracle::characteristics::controls(game, source_id, player_id) {
        return Err(CannotActivate::NotYours);
    }
    Ok(())
}

/// CR 602.5's "can begin to activate", for one ability: an activated ability,
/// and, if it may be activated only as a sorcery, at sorcery timing
/// (CR 602.5d).
pub fn can_begin_to_activate(game: &GameState, player_id: PlayerId, ability: &AbilityDef) -> Result<(), CannotActivate> {
    match ability.ability_type {
        AbilityType::Activated => {}
        AbilityType::Mana => return Err(CannotActivate::ManaAbility),
        AbilityType::Spell | AbilityType::Triggered | AbilityType::Static => {
            return Err(CannotActivate::NotActivated);
        }
    }
    if ability.activation_restriction == ActivationRestriction::OnlyAsSorcery {
        game.check_sorcery_timing(player_id).map_err(CannotActivate::Timing)?;
    }
    Ok(())
}

/// Is `ability`, on `source_id`, offered to `player_id` to activate: CR 602.2
/// and 602.5's start, then what CR 602.2b's costs and targets would refuse.
pub fn can_activate(
    game: &GameState,
    player_id: PlayerId,
    source_id: ObjectId,
    ability: &AbilityDef,
) -> Result<(), CannotActivate> {
    can_activate_its_abilities(game, player_id, source_id)?;
    can_activate_as_its_controller(game, player_id, source_id, ability, &OnceCell::new())
}

/// [`can_activate`] past the controller's check, which the enumeration asks
/// once a source rather than once an ability.
///
/// The mana is asked last, as a card's is: a tapped source is refused for
/// being tapped before anything reads an inventory (`mana-architecture.md`
/// §3.1).
fn can_activate_as_its_controller(
    game: &GameState,
    player_id: PlayerId,
    source_id: ObjectId,
    ability: &AbilityDef,
    supply: &OnceCell<ManaSupply>,
) -> Result<(), CannotActivate> {
    can_begin_to_activate(game, player_id, ability)?;

    let other_costs: Vec<Cost> = ability.costs.iter().filter(|c| !matches!(c, Cost::Mana(_))).cloned().collect();
    for cost in &other_costs {
        game.can_pay_costs(std::slice::from_ref(cost), player_id, source_id).map_err(CannotActivate::Cost)?;
    }

    // CR 602.2b routes an activation through 601.2c, so an ability
    // that *requires* a target and has none is no more activatable
    // than such a spell is castable — the same check `can_cast`
    // makes, and the one the enumeration was missing. `UpTo` is left
    // in: choosing zero targets is legal, so an empty board does not
    // make it illegal. Provably illegal from a static read, which is
    // what the oracle may filter on
    // (`dp-middleware-and-candidate-enumeration.md` §2).
    if !every_instance_has_a_choice(game, &ability.instances, player_id, source_id) {
        return Err(CannotActivate::NoLegalTarget);
    }

    let symbols: Vec<ManaSymbol> = ability
        .costs
        .iter()
        .filter_map(|c| if let Cost::Mana(mana) = c { Some(mana.symbols.iter().copied()) } else { None })
        .flatten()
        .collect();
    let non_mana = NonManaCosts { source: Some(source_id), costs: &other_costs };
    if !symbols.is_empty() && !mana_payable(game, player_id, &ManaCost::from_symbols(symbols), &non_mana, supply) {
        return Err(CannotActivate::ManaShort);
    }
    Ok(())
}

/// Non-mana activated abilities the player can currently pay for: each
/// ability on a permanent they control that [`can_activate`] offers, as
/// `(source_permanent_id, ability_index, ability_id)`.
pub fn activatable_abilities(game: &GameState, player_id: PlayerId) -> Vec<(ObjectId, usize, AbilityId)> {
    activatable_abilities_with(game, player_id, &OnceCell::new())
}

/// [`activatable_abilities`], paying from `supply`.
pub(crate) fn activatable_abilities_with(
    game: &GameState,
    player_id: PlayerId,
    supply: &OnceCell<ManaSupply>,
) -> Vec<(ObjectId, usize, AbilityId)> {
    let mut result = Vec::new();

    // Ordered, not raw `battlefield.iter()`: this is the activatable half of the
    // priority action list, which the DP picks from by index.
    for (id, _entry) in game.battlefield_ordered() {
        if can_activate_its_abilities(game, player_id, id).is_err() {
            continue;
        }

        // `idx` indexes the EFFECTIVE ability list. `priority.rs` re-derives it
        // by id and `cast.rs::activate_ability` consumes it — all three must
        // index the same list or activation silently targets the wrong ability.
        let abilities = crate::oracle::characteristics::get_effective_abilities(game, id);

        for (idx, ability) in abilities.iter().enumerate() {
            if can_activate_as_its_controller(game, player_id, id, ability, supply).is_ok() {
                result.push((id, idx, ability.id));
            }
        }
    }

    result
}

/// CR 601.2c — can a legal choice be announced for **every** instance of the
/// word "target"?
///
/// The rule Decimate's reminder text spells out: "you can't cast this spell
/// unless you have legal choices for all its targets." One instance and one
/// legal creature is the shape every card in the pool has, and at that shape
/// this is the single `has_any_legal_choice` it replaced.
///
/// Three things it asks that the single check could not:
/// - **A count.** Jagged Lightning's "each of two target creatures" needs two
///   distinct creatures, because CR 601.2c forbids choosing one twice for one
///   instance.
/// - **Each instance in turn.** Decimate needs an artifact *and* a creature
///   *and* an enchantment *and* a land.
/// - **What the earlier instances took.** Incremental Growth's "a third target
///   creature" needs a third.
///
/// `UpTo` is left alone: choosing zero targets is legal (CR 115.6), so an empty
/// board does not make such a spell uncastable. An over-approximation here is
/// `codebase-state.md` item 139's class — the engine offers a cast it then
/// rewinds — so the checks that can be made statically are made.
fn every_instance_has_a_choice(
    game: &GameState,
    instances: &[EffectRecipient],
    player_id: PlayerId,
    this_object: ObjectId,
) -> bool {
    // The last clause whose criteria read an earlier instance. Everything after
    // it has nothing to feed forward to, and for every spell but the "another
    // target" family there is no such clause at all — see
    // `clause_reads_earlier_instances` for what that costs when it is not asked.
    let feed_until = instances
        .iter()
        .rposition(crate::engine::targeting::clause_reads_earlier_instances);
    // Where the fold below may look back from. Every clause past `feed_until`
    // is past the last one that reads an earlier instance, so `earlier_targets`
    // is not among the things its answer depends on — which is what makes two
    // equal clauses in this range the same question. A clause *at* `feed_until`
    // is counted rather than enumerated too, but reads the announcement to do
    // it, so it neither reuses an earlier answer nor offers its own.
    let reusable_from = feed_until.map_or(0, |last| last + 1);
    let mut earlier_targets = crate::engine::targeting::ChosenTargets::NONE;
    for (ix, recipient) in instances.iter().enumerate() {
        // **Every instance pushes, in order, whether or not it is checked.**
        // `OtherThanInstance(k)` reads position `k`, so a skipped push would
        // renumber every instance after it and an "another target" clause would
        // exclude the wrong one. Pushing nothing is the honest content: an
        // instance that announces nothing excludes nothing.
        let mut feed = Vec::new();
        if let EffectRecipient::Target(f, TargetCount::Exactly(n))
            | EffectRecipient::Choose(f, TargetCount::Exactly(n)) = recipient
        {
            let n = *n as usize;
            let view = crate::engine::targeting::FilterIdentity::announcing(
                this_object,
                crate::engine::targeting::EarlierTargets::Chosen(&earlier_targets),
            );
            // **One pass, not two.** When a later clause reads this one, the
            // check and the feed-forward want the same scan: `n` candidates, or
            // the knowledge that there are not `n`. A bounded enumeration
            // answers both, and stops where `has_legal_choices` would have.
            //
            // What it feeds forward is a static over-approximation, like the
            // check itself: what matters to the next clause is *how many* this
            // one will take, and any `n` distinct legal choices exclude the same
            // number. Feeding nothing would let an "another target" chain claim
            // it can always be satisfied.
            //
            // `UpTo` never reaches here: choosing zero targets is legal
            // (CR 115.6), so it neither fails the cast nor constrains what
            // follows.
            if feed_until.is_some_and(|last| ix < last) {
                feed = crate::oracle::legality::enumerate_legal_selections_upto(
                    game, f, None, player_id, view, n,
                );
                if feed.len() < n {
                    return false;
                }
            } else {
                // **Equal clauses in that range are one question, asked once.**
                // Seeds of Strength prints "target creature" three times, and
                // each ask is a `validate_selection` per candidate, which is a
                // layer query — three battlefield scans per copy in hand per
                // priority check for an answer that cannot differ between them.
                // The whole recipient is compared rather than the filter alone:
                // `Exactly(1)` and `Exactly(2)` over one filter are different
                // questions. An earlier equal clause was answered `true`,
                // because a `false` returns below rather than reaching here.
                let already_answered =
                    ix >= reusable_from && instances[reusable_from..ix].contains(recipient);
                if !already_answered && !game.has_legal_choices(f, None, player_id, n, view) {
                    return false;
                }
            }
        }
        earlier_targets.push(feed);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::{AbilityDef, CardDataBuilder};
    use crate::objects::object::GameObject;
    use crate::state::battlefield::PermanentState;
    use crate::state::game_state::{GameState, Phase, PhaseType};
    use crate::types::card_types::*;
    use crate::types::effects::{AmountExpr, Effect, Primitive, EffectRecipient, SelectionFilter, TargetCount};
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;

    fn place_mountain(game: &mut GameState, player_id: PlayerId) -> (ObjectId, AbilityId) {
        let mountain = CardDataBuilder::new("Mountain")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .subtype(Subtype::Land(LandType::Mountain))
            .mana_ability_single(ManaType::Red)
            .build();
        let ability_id = mountain.abilities[0].id;
        let obj = GameObject::new(mountain, player_id, Zone::Battlefield);
        let id = game.add_object(obj);
        let entry = PermanentState::new(id, player_id, 0);
        game.insert_battlefield_entity(id, entry);
        (id, ability_id)
    }

    #[test]
    fn test_castable_spells_finds_affordable() {
        let mut game = GameState::new(2, 20);
        game.set_turn_position(Phase::new(PhaseType::Precombat));
        game.active_player = 0;

        place_mountain(&mut game, 0);

        // Put a bolt in hand
        let bolt = CardDataBuilder::new("Lightning Bolt")
            .card_type(CardType::Instant)
            .color(crate::types::colors::Color::Red)
            .mana_cost(ManaCost::build(&[ManaType::Red], 0))
            .ability(AbilityDef {
                rules_text: "".into(),
                is_characteristic_defining: false,
                activation_restriction: crate::objects::card_data::ActivationRestriction::None,
                id: crate::types::ids::new_ability_id(),
                instances: Vec::new(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage { amount: AmountExpr::Fixed(3), unpreventable: false },
                    EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
                ),
            })
            .build();
        let obj = GameObject::new(bolt, 0, Zone::Hand);
        let card_id = game.add_object(obj);
        game.players[0].hand.push(card_id);

        let castable = castable_spells(&game, 0);
        assert_eq!(castable.len(), 1);
        assert_eq!(castable[0], card_id);
    }

    #[test]
    fn test_castable_spells_empty_when_unaffordable() {
        let mut game = GameState::new(2, 20);
        game.set_turn_position(Phase::new(PhaseType::Precombat));
        game.active_player = 0;
        // No lands

        let bolt = CardDataBuilder::new("Lightning Bolt")
            .card_type(CardType::Instant)
            .color(crate::types::colors::Color::Red)
            .mana_cost(ManaCost::build(&[ManaType::Red], 0))
            .ability(AbilityDef {
                rules_text: "".into(),
                is_characteristic_defining: false,
                activation_restriction: crate::objects::card_data::ActivationRestriction::None,
                id: crate::types::ids::new_ability_id(),
                instances: Vec::new(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage { amount: AmountExpr::Fixed(3), unpreventable: false },
                    EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
                ),
            })
            .build();
        let obj = GameObject::new(bolt, 0, Zone::Hand);
        let card_id = game.add_object(obj);
        game.players[0].hand.push(card_id);

        let castable = castable_spells(&game, 0);
        assert!(castable.is_empty());
    }

    #[test]
    fn test_remaining_cost_after_pool_partial_colored() {
        use crate::types::mana::ManaPool;

        // Cost: {1}{G}{G}, pool has 1G → remaining should be {1}{G}
        let cost = ManaCost::from_symbols(vec![
            ManaSymbol::Generic,
            ManaSymbol::Colored(ManaType::Green),
            ManaSymbol::Colored(ManaType::Green),
        ]);
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 1);

        let remaining = remaining_cost_after_pool(&cost, &pool);
        assert_eq!(remaining.generic_count(), 1);
        assert_eq!(remaining.colored_count(ManaType::Green), 1);
    }

    #[test]
    fn test_remaining_cost_after_pool_generic_covered() {
        use crate::types::mana::ManaPool;

        // Cost: {2}{R}, pool has 1R 1G → remaining should be {1}
        // Pool covers {R} (specific) and {1} of the generic with {G}
        let cost = ManaCost::from_symbols(vec![
            ManaSymbol::Generic,
            ManaSymbol::Generic,
            ManaSymbol::Colored(ManaType::Red),
        ]);
        let mut pool = ManaPool::new();
        pool.add(ManaType::Red, 1);
        pool.add(ManaType::Green, 1);

        let remaining = remaining_cost_after_pool(&cost, &pool);
        assert_eq!(remaining.colored_count(ManaType::Red), 0);
        assert_eq!(remaining.generic_count(), 1);
    }

    #[test]
    fn what_a_pool_leaves_owing_keeps_the_costs_order() {
        use crate::types::mana::ManaPool;

        let anthem = ManaCost::build(&[ManaType::White, ManaType::White], 1);
        let mut pool = ManaPool::new();
        assert_eq!(remaining_cost_after_pool(&anthem, &pool).to_string(), "{1}{W}{W}");
        pool.add(ManaType::White, 1);
        assert_eq!(remaining_cost_after_pool(&anthem, &pool).to_string(), "{1}{W}");
        pool.add(ManaType::Green, 1);
        assert_eq!(remaining_cost_after_pool(&anthem, &pool).to_string(), "{W}");
    }

    #[test]
    fn test_castable_spells_with_partial_pool_mana() {
        // {1}{R} bolt with 1G in pool + 1 Mountain on battlefield
        // Pool covers the {1} generic, Mountain covers {R}
        let mut game = GameState::new(2, 20);
        game.set_turn_position(Phase::new(PhaseType::Precombat));
        game.active_player = 0;

        place_mountain(&mut game, 0);
        game.players[0].mana_pool.add(ManaType::Green, 1);

        // A spell costing {1}{R}
        let spell = CardDataBuilder::new("Shock Plus")
            .card_type(CardType::Instant)
            .mana_cost(ManaCost::build(&[ManaType::Red], 1))
            .ability(AbilityDef {
                rules_text: "".into(),
                is_characteristic_defining: false,
                activation_restriction: crate::objects::card_data::ActivationRestriction::None,
                id: crate::types::ids::new_ability_id(),
                instances: Vec::new(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage { amount: AmountExpr::Fixed(2), unpreventable: false },
                    EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
                ),
            })
            .build();
        let obj = GameObject::new(spell, 0, Zone::Hand);
        let card_id = game.add_object(obj);
        game.players[0].hand.push(card_id);

        let castable = castable_spells(&game, 0);
        assert_eq!(castable.len(), 1, "Should be castable with pool + tap");
    }

    #[test]
    fn test_castable_spells_respects_timing() {
        let mut game = GameState::new(2, 20);
        // Combat phase — sorceries can't be cast
        game.set_turn_position(Phase::new(PhaseType::Combat));
        game.active_player = 0;

        place_mountain(&mut game, 0);

        let sorcery = CardDataBuilder::new("Lava Axe")
            .card_type(CardType::Sorcery)
            .mana_cost(ManaCost::build(&[ManaType::Red], 4))
            .ability(AbilityDef {
                rules_text: "".into(),
                is_characteristic_defining: false,
                activation_restriction: crate::objects::card_data::ActivationRestriction::None,
                id: crate::types::ids::new_ability_id(),
                instances: Vec::new(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage { amount: AmountExpr::Fixed(5), unpreventable: false },
                    EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
                ),
            })
            .build();
        let obj = GameObject::new(sorcery, 0, Zone::Hand);
        let card_id = game.add_object(obj);
        game.players[0].hand.push(card_id);

        let castable = castable_spells(&game, 0);
        assert!(castable.is_empty());
    }
}
