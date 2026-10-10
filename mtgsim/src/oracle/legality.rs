// Read-only legality queries — can a creature attack, block, etc.
//
// All functions here are pure reads against `&GameState`. They never mutate.
// The priority question's candidates are the actions each check allows: what
// it cannot read statically (a target that leaves, a payment the window
// fails) the engine rejects and reverses (CR 732.1). See
// `plans/atomic-tests/supplemental-docs/dp-middleware-and-candidate-enumeration.md`.

use std::cell::OnceCell;

use crate::oracle::characteristics::{controls, has_keyword, has_summoning_sickness, is_creature};
use crate::oracle::mana_helpers::{activatable_abilities_with, castable_spells_with};
use crate::engine::combat::validation::CombatError;
use crate::engine::put_on_stack::SorceryTiming;
use crate::state::game_state::GameState;
use crate::types::card_types::CardType;
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::zones::Zone;
use crate::types::keywords::KeywordFlag;
use crate::ui::decision::PriorityAction;

/// Can `player_id` declare `id` as an attacker (CR 508.1a)? One check for the
/// enumeration and the declaration: the declare-attackers question offers what
/// it allows, and `validate_attackers` refuses what it refuses, so attacking
/// has one road, as blocking has `can_block`.
///
/// The creature check is part of the answer, not the caller's job: CR 508.1a
/// lets only creatures be declared as attackers, and `has_summoning_sickness`
/// is false for a noncreature permanent, so without it an untapped Sol Ring
/// could attack.
pub fn can_attack(game: &GameState, player_id: PlayerId, id: ObjectId) -> Result<(), CombatError> {
    let entry = game.battlefield.get(&id).ok_or(CombatError::NotOnBattlefield(id))?;
    // Effective controller (CR 613.1b): the creature you stole this turn
    // attacks for you, which is what the haste clause is buying.
    if !controls(game, id, player_id) {
        return Err(CombatError::NotControlledByPlayer(id, player_id));
    }
    if !is_creature(game, id) {
        return Err(CombatError::NotACreature(id));
    }
    if entry.tapped {
        return Err(CombatError::CreatureIsTapped(id));
    }
    // CR 302.6, which haste lifts (CR 702.10b).
    if has_summoning_sickness(game, id) {
        return Err(CombatError::CreatureHasSummoningSickness(id));
    }
    if has_keyword(game, id, KeywordFlag::Defender) {
        return Err(CombatError::HasDefender(id));
    }
    Ok(())
}

/// Why a card is not offered to play as a land at a priority question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CannotPlayLand {
    /// Not in the player's hand, the one zone the engine plays a land from
    /// (CR 305.1).
    NotInHand,
    /// Not a land card (CR 305.1).
    NotALand,
    /// Outside its owner's main phase with the stack empty (CR 305.1).
    Timing(SorceryTiming),
    /// The player has played as many lands this turn as they may
    /// (CR 305.2a).
    NoLandDropLeft { played: u32, allowed: u32 },
}

/// The engine's own words, for an error a caller returns as text.
impl std::fmt::Display for CannotPlayLand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CannotPlayLand::NotInHand => f.write_str("Card is not in its owner's hand"),
            CannotPlayLand::NotALand => f.write_str("This card is not a land"),
            CannotPlayLand::Timing(timing) => timing.fmt(f),
            CannotPlayLand::NoLandDropLeft { .. } => f.write_str("Already played maximum lands this turn"),
        }
    }
}

/// May `player_id` play `card_id` as a land now (CR 305.1, 305.2a)? One check
/// for the enumeration and for `GameState::play_land`.
pub fn can_play_land(game: &GameState, player_id: PlayerId, card_id: ObjectId) -> Result<(), CannotPlayLand> {
    let Some(obj) = game.objects.get(&card_id) else {
        return Err(CannotPlayLand::NotInHand);
    };
    // A hand holds only its owner's cards (CR 400.3).
    if obj.zone != Zone::Hand || obj.owner != player_id {
        return Err(CannotPlayLand::NotInHand);
    }
    // PRE-LAYER ZONE: reads printed types on purpose. This is cast-zone /
    // play-from-hand legality, which happens before the object is a permanent,
    // so the layer system has nothing to contribute. Same exemption as
    // engine/cast.rs -- see "Before Layers" in plans/codebase-state.md.
    if !obj.card_data.types.contains(&CardType::Land) {
        return Err(CannotPlayLand::NotALand);
    }
    game.check_sorcery_timing(player_id).map_err(CannotPlayLand::Timing)?;
    if let Some(player) = game.players.get(player_id)
        && !player.can_play_land()
    {
        return Err(CannotPlayLand::NoLandDropLeft {
            played: player.lands_played_this_turn,
            allowed: player.lands_per_turn,
        });
    }
    Ok(())
}

/// Get all lands in a player's hand that they can legally play this turn:
/// each one [`can_play_land`] allows.
pub fn playable_lands(game: &GameState, player_id: PlayerId) -> Vec<ObjectId> {
    let Some(player) = game.players.get(player_id) else {
        return Vec::new();
    };
    player.hand.iter().copied().filter(|&id| can_play_land(game, player_id, id).is_ok()).collect()
}

/// Get all creatures controlled by a player that can legally be declared as
/// attackers: each one [`can_attack`] allows.
///
/// Ordered by `battlefield_ordered` — a `DecisionProvider` picks by index, so
/// the order this returns in is part of the decision, not a presentation
/// detail.
pub fn legal_attackers(game: &GameState, player_id: PlayerId) -> Vec<ObjectId> {
    game.battlefield_ordered()
        .into_iter()
        .filter_map(|(id, _)| can_attack(game, player_id, id).is_ok().then_some(id))
        .collect()
}

/// Get all creatures controlled by a player that can legally block.
///
/// A creature can block if it's on the battlefield, is a creature, untapped,
/// and controlled by the defending player. Specific attacker legality
/// (flying/reach checks) is handled during actual block declarations.
pub fn legal_blockers(game: &GameState, player_id: PlayerId) -> Vec<ObjectId> {
    game.battlefield_ordered().into_iter()
        .filter_map(|(id, entry)| {
            if !controls(game, id, player_id) {
                return None;
            }
            if !is_creature(game, id) {
                return None;
            }
            if entry.tapped {
                return None;
            }
            Some(id)
        })
        .collect()
}

/// Build the candidate list of priority actions for a player: `Pass` first,
/// then each land [`can_play_land`] allows, each card `can_cast` allows and
/// each ability `can_activate` allows.
///
/// One inventory of what the player can pay with serves every card and
/// ability that reaches the mana check, read at the first
/// (`mana-architecture.md` §3.1).
pub fn candidate_priority_actions(game: &GameState, player_id: PlayerId) -> Vec<PriorityAction> {
    let mut actions = vec![PriorityAction::Pass];
    let supply = OnceCell::new();
    actions.extend(playable_lands(game, player_id).into_iter().map(PriorityAction::PlayLand));
    actions.extend(castable_spells_with(game, player_id, &supply).into_iter().map(PriorityAction::CastSpell));
    for (source_id, _ability_index, ability_id) in activatable_abilities_with(game, player_id, &supply) {
        actions.push(PriorityAction::ActivateAbility(source_id, ability_id));
    }
    actions
}

/// Enumerate all legal selections for an `EffectRecipient`.
///
/// Returns every `ResolvedTarget` that passes `validate_selection` for the
/// given filter. Used by `ask_select_recipients` to build the options list.
///
/// `exclude_id`: CR 115.5's "a spell or ability on the stack is an illegal
/// target for itself" — the object being cast or activated, which the
/// `Spell` and `DamageSource` filters would otherwise offer back to it.
/// `you` is CR 109.5's "you" for the filter — the player the selection is being
/// made for, which is who a `ByController(PlayerRef::You)` node names.
pub fn enumerate_legal_selections(
    game: &GameState,
    filter: &crate::types::effects::SelectionFilter,
    exclude_id: Option<ObjectId>,
    you: PlayerId,
) -> Vec<crate::engine::resolve::ResolvedTarget> {
    enumerate_legal_selections_excluding(
        game,
        filter,
        exclude_id,
        you,
        crate::engine::targeting::FilterIdentity::NONE,
    )
}

/// [`enumerate_legal_selections`] inside CR 601.2c's loop, where the instances
/// of "target" announced so far are known.
///
/// `earlier_targets` is what an `ObjectFilter::OtherThanInstance` leaf reads — the
/// difference between Incremental Growth offering three creatures for its
/// second clause and offering the two it has not already taken. Outside the
/// loop the list is empty and that leaf is refused, which is why the plain
/// spelling above stays the one to call.
pub fn enumerate_legal_selections_excluding(
    game: &GameState,
    filter: &crate::types::effects::SelectionFilter,
    exclude_id: Option<ObjectId>,
    you: PlayerId,
    identity: crate::engine::targeting::FilterIdentity<'_>,
) -> Vec<crate::engine::resolve::ResolvedTarget> {
    enumerate_legal_selections_upto(game, filter, exclude_id, you, identity, usize::MAX)
}

/// [`enumerate_legal_selections_excluding`] that stops once it has `limit`.
///
/// **For a caller that wants candidates rather than the candidate list.**
/// CR 601.2c's castability check needs to know that `n` legal choices exist and,
/// when a later clause excludes them, *which* `n` — and nothing about the rest.
/// Enumerating the whole battlefield to keep the first three is a
/// `validate_selection` per extra permanent, and each of those is a layer walk.
///
/// The bound is what lets the check and the feed-forward be one pass: a caller
/// that asks for `n` and gets fewer than `n` back has its answer, and a caller
/// that gets `n` back has the list it needed. `usize::MAX` is the unbounded
/// spelling, which is what a `DecisionProvider`'s option list wants.
pub fn enumerate_legal_selections_upto(
    game: &GameState,
    filter: &crate::types::effects::SelectionFilter,
    exclude_id: Option<ObjectId>,
    you: PlayerId,
    identity: crate::engine::targeting::FilterIdentity<'_>,
    limit: usize,
) -> Vec<crate::engine::resolve::ResolvedTarget> {
    use crate::engine::resolve::ResolvedTarget as RT;
    use crate::types::effects::SelectionFilter;

    // **Each arm is an iterator and the cap is `take`**: arms yield in their
    // documented order, so `take` keeps the *first* `limit`, process-independent,
    // and laziness stops the layer walks there. CR 800.4a: a departed player is
    // no candidate, and `num_players()` is the seat count the game began with,
    // so `Player` and `Any` read this set rather than the range.
    let players = || {
        (0..game.num_players())
            .filter(move |&p| game.in_game(p))
            .map(RT::Player)
    };
    let battlefield = || {
        game.battlefield_ids_ordered()
            .into_iter()
            .filter(move |&id| Some(id) != exclude_id)
    };
    // CR 112.1 — a *spell* on the stack, which the activated ability sharing
    // the zone with it is not. Both arms that read the stack want exactly this
    // set, so the predicate sits in the closure rather than on each of them.
    let spells_on_stack = || {
        game.stack
            .iter()
            .copied()
            .filter(move |&id| Some(id) != exclude_id && game.is_spell_on_stack(id))
    };
    let passes = |id: ObjectId| {
        let candidate = RT::Object(id);
        game.validate_selection(filter, &candidate, you, identity)
            .is_ok()
            .then_some(candidate)
    };

    match filter {
        SelectionFilter::Player => players().take(limit).collect(),

        // "Any target" — players, then creatures and planeswalkers.
        SelectionFilter::Any => players()
            .chain(battlefield().filter_map(passes))
            .take(limit)
            .collect(),

        SelectionFilter::Spell => spells_on_stack().map(RT::Object).take(limit).collect(),

        // CR 609.7a, as an id names each: the source choice itself reads
        // `damage_sources`, whose existences an id cannot tell apart.
        SelectionFilter::DamageSource => {
            let mut ids: Vec<ObjectId> = Vec::new();
            for source in damage_sources(game, exclude_id) {
                if !ids.contains(&source.id) {
                    ids.push(source.id);
                }
            }
            ids.into_iter().map(RT::Object).take(limit).collect()
        }

        // Creature, Permanent(_), or other battlefield-based filters
        _ => battlefield().filter_map(passes).take(limit).collect(),
    }
}

/// CR 609.7a — every source of damage a player may choose, each existence
/// once, in the order offered: the permanents (CR 613.7 order), the spells on
/// the stack (its order, a spell resolving now among them, CR 608.2), then
/// each object referred to by an object on the stack, by a replacement or
/// prevention effect waiting to apply, or by a delayed triggered ability
/// waiting to trigger, "even if that object is no longer in the zone it used
/// to be in" ([`referred_damage_sources`]). Enumerated,
/// not validated: "a source doesn't need to be capable of dealing damage"
/// leaves only membership to test. The rule's fourth category, a face-up
/// object in the command zone, waits for that zone (`codebase-state.md` item
/// 99). `exclude_id` is CR 115.5's object.
pub fn damage_sources(game: &GameState, exclude_id: Option<ObjectId>) -> Vec<ObjectRef> {
    // A spell's resolution takes its entry, and it stays on the stack.
    let resolving_spell = game.resolving.as_ref().filter(|r| r.identity.is_none()).map(|r| r.id);
    let present = game
        .battlefield_ids_ordered()
        .into_iter()
        .chain(game.stack.iter().copied().filter(|&id| game.is_spell_on_stack(id) || Some(id) == resolving_spell))
        .filter_map(|id| game.object_ref(id));
    let mut sources: Vec<ObjectRef> = Vec::new();
    for source in present.chain(referred_damage_sources(game)) {
        if Some(source.id) != exclude_id && !sources.contains(&source) {
            sources.push(source);
        }
    }
    sources
}

/// CR 609.7a's referred-to sources of damage: the objects a player choosing
/// a source may choose because something waiting refers to them, each by
/// the existence it was (CR 400.7), in this order: by each object on the stack, bottom first,
/// then the one resolving now — an ability's source, what its trigger names,
/// its targets, and each it named that has since left; by each replacement
/// or prevention effect a resolution created, in the order they were — its
/// chosen source of damage, its targets and the objects it is around; and by
/// each delayed triggered ability, in the order they were — its source and
/// what it refers to. A trigger on its subject's departure names the object
/// that left (CR 603.10a), the one that deals its damage, and not the one it
/// became. Not a CR 610.3 return, which is no triggered ability.
fn referred_damage_sources(game: &GameState) -> Vec<ObjectRef> {
    use crate::engine::targeting::TargetRef;
    let objects_of = |targets: &[TargetRef]| -> Vec<ObjectRef> {
        targets
            .iter()
            .filter_map(|t| match t {
                TargetRef::Object(object) => Some(*object),
                TargetRef::Player(_) => None,
            })
            .collect()
    };
    let mut referred: Vec<ObjectRef> = Vec::new();
    for entry in game.stack.iter().filter_map(|id| game.stack_entries.get(id)) {
        let as_it_left = |object: ObjectRef| entry.lki_of(object).map_or(object, |lki| lki.object);
        let binding = entry.trigger.as_ref();
        referred.extend(entry.ability_identity.map(|identity| as_it_left(identity.source)));
        referred.extend(binding.and_then(|b| b.subject).map(as_it_left));
        if let Some(binding) = binding {
            referred.extend(binding.referred.objects.iter().map(|r| r.object));
        }
        for instance in &entry.chosen_targets {
            referred.extend(objects_of(&instance.chosen));
        }
        referred.extend(entry.departed.iter().map(|d| d.object));
    }
    if let Some(resolving) = &game.resolving {
        let as_it_left = |object: ObjectRef| resolving.lki_of(object).map_or(object, |lki| lki.object);
        referred.extend(resolving.identity.map(|identity| as_it_left(identity.source)));
        referred.extend(resolving.subject.map(as_it_left));
        referred.extend(resolving.departed.iter().map(|d| d.object));
    }
    for row in game.replacement_effects.iter() {
        referred.extend(row.def.pattern.chosen_damage_source());
        referred.extend(objects_of(&row.targets));
        if let crate::types::effects::ObjectSet::Fixed(ids) = &row.def.affected_objects {
            referred.extend(ids.iter().filter_map(|&id| game.object_ref(id)));
        }
    }
    for delayed in &game.delayed_triggers {
        referred.push(delayed.source);
        referred.extend(delayed.referred.objects.iter().map(|r| r.object));
    }
    referred
}

#[cfg(test)]
mod tests {
    use crate::engine::targeting::FilterIdentity;
    use crate::types::replacement::EnterMods;
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::PermanentState;
    use crate::types::card_types::CardType;
    use crate::types::zones::Zone;
    use crate::state::game_state::PhaseType;

    /// CR 508.1a — only creatures attack, and `has_summoning_sickness` answers
    /// `false` for a noncreature permanent because CR 302.6 does not restrict
    /// one. Without the type half, an untapped mana rock would be reported as a
    /// legal attacker.
    #[test]
    fn test_a_noncreature_permanent_cannot_attack() {
        let mut game = GameState::new(2, 20);
        let sol_ring = crate::test_support::put_on_battlefield(
            &mut game,
            crate::cards::artifacts::sol_ring(),
            0,
        );

        assert!(!crate::oracle::characteristics::is_creature(&game, sol_ring));
        assert!(!has_summoning_sickness(&game, sol_ring), "not a creature, not sick");
        assert_eq!(can_attack(&game, 0, sol_ring), Err(CombatError::NotACreature(sol_ring)));
    }

    #[test]
    fn test_can_attack_not_summoning_sick() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        let entry = PermanentState::new(id, 0, 0);
        game.insert_battlefield_entity(id, entry);

        assert_eq!(can_attack(&game, 0, id), Ok(()));
    }

    #[test]
    fn test_cannot_attack_summoning_sick() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE); // entered this turn = summoning sick

        assert_eq!(can_attack(&game, 0, id), Err(CombatError::CreatureHasSummoningSickness(id)));
    }

    #[test]
    fn test_can_attack_with_haste_while_summoning_sick() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Raging Cougar")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .keyword_flag(KeywordFlag::Haste)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE); // entered this turn = summoning sick

        assert_eq!(can_attack(&game, 0, id), Ok(()));
    }

    #[test]
    fn test_can_attack_not_on_battlefield() {
        let game = GameState::new(2, 20);
        let fake_id = crate::types::ids::new_object_id();
        assert_eq!(can_attack(&game, 0, fake_id), Err(CombatError::NotOnBattlefield(fake_id)));
    }

    // --- playable_lands tests ---

    #[test]
    fn test_playable_lands_main_phase() {
        let mut game = GameState::new(2, 20);
        game.set_turn_position(crate::state::game_state::Phase::new(PhaseType::Precombat));
        game.active_player = 0;

        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let obj = GameObject::new(forest, 0, Zone::Hand);
        let id = game.add_object(obj);
        game.players[0].hand.push(id);

        let lands = playable_lands(&game, 0);
        assert_eq!(lands.len(), 1);
        assert_eq!(lands[0], id);
    }

    #[test]
    fn test_playable_lands_wrong_phase() {
        let mut game = GameState::new(2, 20);
        game.set_turn_position(crate::state::game_state::Phase::new(PhaseType::Combat));
        game.active_player = 0;

        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let obj = GameObject::new(forest, 0, Zone::Hand);
        let id = game.add_object(obj);
        game.players[0].hand.push(id);

        assert!(playable_lands(&game, 0).is_empty());
    }

    #[test]
    fn test_playable_lands_already_played() {
        let mut game = GameState::new(2, 20);
        game.set_turn_position(crate::state::game_state::Phase::new(PhaseType::Precombat));
        game.active_player = 0;
        game.players[0].lands_played_this_turn = 1;

        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let obj = GameObject::new(forest, 0, Zone::Hand);
        let id = game.add_object(obj);
        game.players[0].hand.push(id);

        assert!(playable_lands(&game, 0).is_empty());
    }

    // --- legal_attackers tests ---

    #[test]
    fn test_legal_attackers_basic() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        let entry = PermanentState::new(id, 0, 0);
        game.insert_battlefield_entity(id, entry);

        let attackers = legal_attackers(&game, 0);
        assert_eq!(attackers.len(), 1);
        assert_eq!(attackers[0], id);
    }

    #[test]
    fn test_legal_attackers_excludes_defender() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Wall of Stone")
            .card_type(CardType::Creature)
            .power_toughness(0, 8)
            .keyword_flag(KeywordFlag::Defender)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        let entry = PermanentState::new(id, 0, 0);
        game.insert_battlefield_entity(id, entry);

        assert!(legal_attackers(&game, 0).is_empty());
    }

    #[test]
    fn test_legal_attackers_excludes_tapped() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        let mut entry = PermanentState::new(id, 0, 0);
        entry.tapped = true;
        game.insert_battlefield_entity(id, entry);

        assert!(legal_attackers(&game, 0).is_empty());
    }

    // --- legal_blockers tests ---

    #[test]
    fn test_legal_blockers_basic() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        let entry = PermanentState::new(id, 0, 0);
        game.insert_battlefield_entity(id, entry);

        let blockers = legal_blockers(&game, 0);
        assert_eq!(blockers.len(), 1);
    }

    #[test]
    fn test_legal_blockers_excludes_tapped() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        let mut entry = PermanentState::new(id, 0, 0);
        entry.tapped = true;
        game.insert_battlefield_entity(id, entry);

        assert!(legal_blockers(&game, 0).is_empty());
    }

    // CR 609.7a — "they may choose a permanent; a spell on the stack
    // (including a permanent spell); ... A source doesn't need to be capable
    // of dealing damage to be a legal choice." Both categories are offered
    // and neither is filtered by what the object can do: a Plains is on the
    // list. The referred-to category is `phase_tr3c_integration_test`'s.
    //
    // COVERS-PARTIAL: ATOM-609.7a-001 -- the permanent and stack-spell legs,
    // built whole. The referred-to legs are `phase_tr3c_integration_test`'s,
    // and "a face-up object in the command zone" waits for the command zone
    // (`codebase-state.md` item 99, B2).
    //
    // COVERS-PARTIAL: BOUNDARY-DEF-609.7a-001 -- in-set (a creature permanent)
    // and out-of-set (a card in hand referred to by nothing); the boundary's
    // middle, an object that left and *is* referred to, is TR-3c's test.
    #[test]
    fn a_damage_source_is_a_permanent_or_a_spell_on_the_stack() {
        use crate::engine::resolve::ResolvedTarget;
        use crate::test_support::{
            place_vanilla_creature, put_in_hand, put_land_on_battlefield, put_spell_on_stack,
            setup_two_player_game,
        };
        use crate::types::effects::SelectionFilter;

        let mut game = setup_two_player_game();
        let creature = place_vanilla_creature(&mut game, 0, 2, 2, &[]);
        let land = put_land_on_battlefield(&mut game, crate::cards::basic_lands::plains, 1);
        let spell = put_spell_on_stack(&mut game, crate::test_support::lightning_bolt(), 1);
        let in_hand = put_in_hand(&mut game, crate::test_support::lightning_bolt(), 0);

        let legal = enumerate_legal_selections(&game, &SelectionFilter::DamageSource, None, 0);
        assert!(legal.contains(&ResolvedTarget::Object(creature)), "a permanent");
        assert!(
            legal.contains(&ResolvedTarget::Object(land)),
            "a land: 609.7a's source need not be capable of dealing damage"
        );
        assert!(legal.contains(&ResolvedTarget::Object(spell)), "a spell on the stack");
        assert!(!legal.contains(&ResolvedTarget::Object(in_hand)), "a card in hand is not");
        assert_eq!(legal.len(), 3, "and nothing else");

        // Enumeration and enforcement agree, which is the rule RS-2 fixed in
        // both directions: everything offered validates, and the card in hand
        // does not.
        for choice in &legal {
            assert!(game.validate_selection(&SelectionFilter::DamageSource, choice, 0, FilterIdentity::NONE).is_ok());
        }
        assert!(game
            .validate_selection(
                &SelectionFilter::DamageSource,
                &ResolvedTarget::Object(in_hand),
                0,
                FilterIdentity::NONE,
            )
            .is_err());
        assert!(game
            .validate_selection(&SelectionFilter::DamageSource, &ResolvedTarget::Player(0), 0, FilterIdentity::NONE)
            .is_err());
        assert!(game.has_legal_choices(&SelectionFilter::DamageSource, None, 0, 1, FilterIdentity::NONE));
    }

    // The board with nothing on it: no permanent, no spell, so CR 101.3's
    // impossible instruction and the caller's no-op path.
    #[test]
    fn an_empty_board_offers_no_damage_source() {
        use crate::test_support::setup_two_player_game;
        use crate::types::effects::SelectionFilter;

        let game = setup_two_player_game();
        assert!(enumerate_legal_selections(&game, &SelectionFilter::DamageSource, None, 0)
            .is_empty());
        assert!(!game.has_legal_choices(&SelectionFilter::DamageSource, None, 0, 1, FilterIdentity::NONE));
    }

    // CR 800.4a — a player who has left the game is not a player, so not a
    // legal target and not a candidate to offer. `num_players()` is the player
    // vector's length and nothing ever shrinks it, which is why both arms that
    // read it had to ask a second question.
    //
    // **Four seats, and the flag set directly.** The rule needs a game that
    // continues after a departure, which CR 104.2a denies a two-player game;
    // what the departure was *for* belongs to `engine::leaving`'s own tests,
    // and this one asks only what the selection arms do with the answer.
    #[test]
    fn a_seat_that_left_the_game_is_neither_offered_nor_counted() {
        use crate::engine::resolve::ResolvedTarget;
        use crate::test_support::setup_game;
        use crate::types::effects::SelectionFilter;

        let mut game = setup_game(4);
        let seats = |game: &GameState, filter| {
            enumerate_legal_selections(game, &filter, None, 0)
        };

        // The control: with everyone in the game, four seats and a fourth
        // choice. Without it the assertions below would pass on an arm that
        // offered nothing at all.
        for filter in [SelectionFilter::Player, SelectionFilter::Any] {
            assert_eq!(seats(&game, filter.clone()).len(), 4, "{:?}: four seats", filter);
            assert!(game.has_legal_choices(&filter, None, 0, 4, FilterIdentity::NONE));
        }

        game.player_lost[3] = true;

        let in_game = vec![
            ResolvedTarget::Player(0),
            ResolvedTarget::Player(1),
            ResolvedTarget::Player(2),
        ];
        for filter in [SelectionFilter::Player, SelectionFilter::Any] {
            // The battlefield is empty, so `Any` offers the seats and nothing
            // else and the two lists are the same list.
            assert_eq!(seats(&game, filter.clone()), in_game, "{:?}: the seat is not offered", filter);
            assert!(
                game.has_legal_choices(&filter, None, 0, 3, FilterIdentity::NONE),
                "{:?}: three seats are still three choices",
                filter,
            );
            assert!(
                !game.has_legal_choices(&filter, None, 0, 4, FilterIdentity::NONE),
                "{:?}: and the fourth is not a choice to count",
                filter,
            );
            // Enumeration and enforcement agree — the rule RS-2 fixed in both
            // directions, asked here of the seat that left.
            assert!(game
                .validate_selection(&filter, &ResolvedTarget::Player(3), 0, FilterIdentity::NONE)
                .is_err());
        }
    }
}
