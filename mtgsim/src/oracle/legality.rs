// Read-only legality queries — can a creature attack, block, etc.
//
// All functions here are pure reads against `&GameState`. They never mutate.
// For priority actions, the candidate list is an **overapproximation** —
// false positives are harmless (engine rejects via rollback), false negatives
// are bugs. See `plans/atomic-tests/supplemental-docs/dp-middleware-and-candidate-enumeration.md`.

use crate::oracle::characteristics::{controls, has_keyword, has_summoning_sickness, is_creature};
use crate::oracle::mana_helpers::{activatable_abilities, castable_spells};
use crate::state::game_state::{GameState, PhaseType};
use crate::types::card_types::CardType;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::keywords::KeywordFlag;
use crate::ui::decision::PriorityAction;

/// Check if a creature can attack (not summoning-sick, or has haste).
/// Rule 702.10b: Haste bypasses summoning sickness for attacking.
///
/// The creature check is part of the answer, not the caller's job: CR 508.1a
/// lets only creatures be declared as attackers, and `has_summoning_sickness`
/// is false for a noncreature permanent — so without it this would report that
/// an untapped Sol Ring can attack.
pub fn can_attack(game: &GameState, id: ObjectId) -> bool {
    if game.battlefield.contains_key(&id) {
        is_creature(game, id) && !has_summoning_sickness(game, id)
    } else {
        false
    }
}

/// Get all lands in a player's hand that they can legally play this turn.
///
/// Checks:
/// - Card is a land
/// - Player hasn't exceeded their lands-per-turn limit
/// - It's a main phase and the stack is empty (sorcery-speed timing)
/// - Player is the active player
pub fn playable_lands(game: &GameState, player_id: PlayerId) -> Vec<ObjectId> {
    let player = match game.players.get(player_id) {
        Some(p) => p,
        None => return Vec::new(),
    };

    // Timing: active player, main phase, empty stack
    if player_id != game.active_player {
        return Vec::new();
    }
    let is_main = matches!(
        game.phase.phase_type,
        PhaseType::Precombat | PhaseType::Postcombat
    );
    if !is_main || !game.stack.is_empty() {
        return Vec::new();
    }

    if !player.can_play_land() {
        return Vec::new();
    }

    player.hand.iter()
        .copied()
        .filter(|&id| {
            game.objects.get(&id)
                // PRE-LAYER ZONE: reads printed types on purpose. This is cast-zone /
                // play-from-hand legality, which happens before the object is a permanent,
                // so the layer system has nothing to contribute. Same exemption as
                // engine/cast.rs -- see "Before Layers" in plans/codebase-state.md.
                .map(|obj| obj.card_data.types.contains(&CardType::Land))
                .unwrap_or(false)
        })
        .collect()
}

/// Get all creatures controlled by a player that can legally be declared as attackers.
///
/// Ordered by `battlefield_ordered` — a `DecisionProvider` picks by index, so
/// the order this returns in is part of the decision, not a presentation
/// detail.
///
/// Checks per-creature legality (rule 508.1a): on battlefield, is a creature,
/// controlled by player, untapped, not summoning-sick (or has haste), no defender.
pub fn legal_attackers(game: &GameState, player_id: PlayerId) -> Vec<ObjectId> {
    game.battlefield_ordered().into_iter()
        .filter_map(|(id, entry)| {
            // Effective controller (CR 613.1b): the creature you stole this turn
            // attacks for you, which is what the haste clause is buying.
            if !controls(game, id, player_id) {
                return None;
            }
            if !is_creature(game, id) {
                return None;
            }
            if entry.tapped {
                return None;
            }
            if !can_attack(game, id) {
                return None;
            }
            if has_keyword(game, id, KeywordFlag::Defender) {
                return None;
            }
            Some(id)
        })
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

/// Build the candidate list of priority actions for a player.
///
/// This is an **overapproximation**: every action returned passes static
/// legality checks (timing, zone, tap-state) but may fail dynamic checks
/// (mana affordability, complex cost payability). The engine's execution +
/// rollback handles false positives.
///
/// Always includes `Pass` as the first option.
pub fn candidate_priority_actions(game: &GameState, player_id: PlayerId) -> Vec<PriorityAction> {
    let mut actions = vec![PriorityAction::Pass];

    // Playable lands — exact (land drop count is static state)
    for land_id in playable_lands(game, player_id) {
        actions.push(PriorityAction::PlayLand(land_id));
    }

    // Castable spells — overapproximation (affordability is heuristic)
    for (spell_id, _sources) in castable_spells(game, player_id) {
        actions.push(PriorityAction::CastSpell(spell_id));
    }

    // Activatable abilities — overapproximation (affordability is heuristic)
    for (_source_id, _ability_index, ability_id) in activatable_abilities(game, player_id) {
        actions.push(PriorityAction::ActivateAbility(_source_id, ability_id));
    }

    actions
}

/// Enumerate all legal selections for an `EffectRecipient`.
///
/// Returns every `ResolvedTarget` that passes `validate_selection` for the
/// given filter. Used by `ask_select_recipients` to build the options list.
///
/// `exclude_id`: optionally exclude an object (e.g. the Aura itself for
/// enchant-selection, or the spell being cast for "target spell" effects).
/// `you` is CR 109.5's "you" for the filter — the player the selection is being
/// made for, which is who a `ByController(PlayerRef::You)` node names.
pub fn enumerate_legal_selections(
    game: &GameState,
    filter: &crate::types::effects::SelectionFilter,
    exclude_id: Option<ObjectId>,
    you: PlayerId,
) -> Vec<crate::engine::resolve::ResolvedTarget> {
    use crate::engine::resolve::ResolvedTarget;
    use crate::types::effects::SelectionFilter;

    let mut selections = Vec::new();

    match filter {
        SelectionFilter::Player => {
            for pid in 0..game.num_players() {
                selections.push(ResolvedTarget::Player(pid));
            }
        }
        SelectionFilter::Any => {
            // Players
            for pid in 0..game.num_players() {
                selections.push(ResolvedTarget::Player(pid));
            }
            // Creatures and planeswalkers on battlefield
            for id in game.battlefield_ids_ordered() {
                if Some(id) == exclude_id {
                    continue;
                }
                let candidate = ResolvedTarget::Object(id);
                if game.validate_selection(filter, &candidate, you).is_ok() {
                    selections.push(candidate);
                }
            }
        }
        SelectionFilter::Spell => {
            for &id in &game.stack {
                if Some(id) == exclude_id {
                    continue;
                }
                selections.push(ResolvedTarget::Object(id));
            }
        }
        // CR 609.7a — permanents first, then spells on the stack. Both halves
        // are enumerated rather than validated one by one, because
        // `validate_damage_source` asks the same two membership questions and
        // nothing else: the rule's "a source doesn't need to be capable of
        // dealing damage" means there is no property to test.
        //
        // Battlefield order is CR 613.7's timestamp order and stack order is
        // the stack's, so the list a `DecisionProvider` picks from by index is
        // process-independent.
        SelectionFilter::DamageSource => {
            for id in game.battlefield_ids_ordered() {
                if Some(id) == exclude_id {
                    continue;
                }
                selections.push(ResolvedTarget::Object(id));
            }
            for &id in &game.stack {
                if Some(id) == exclude_id {
                    continue;
                }
                if game.stack_entries.get(&id).is_some_and(|e| e.is_spell) {
                    selections.push(ResolvedTarget::Object(id));
                }
            }
        }
        // Creature, Permanent(_), or other battlefield-based filters
        _ => {
            for id in game.battlefield_ids_ordered() {
                if Some(id) == exclude_id {
                    continue;
                }
                let candidate = ResolvedTarget::Object(id);
                if game.validate_selection(filter, &candidate, you).is_ok() {
                    selections.push(candidate);
                }
            }
        }
    }

    selections
}

#[cfg(test)]
mod tests {
    use crate::types::replacement::EnterMods;
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::PermanentState;
    use crate::types::card_types::CardType;
    use crate::types::zones::Zone;

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
        assert!(!can_attack(&game, sol_ring));
    }

    #[test]
    fn test_can_attack_not_summoning_sick() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = PermanentState::new(id, 0, ts, 0);
        game.battlefield.insert(id, entry);

        assert!(can_attack(&game, id));
    }

    #[test]
    fn test_cannot_attack_summoning_sick() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE); // entered this turn = summoning sick

        assert!(!can_attack(&game, id));
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
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE); // entered this turn = summoning sick

        assert!(can_attack(&game, id));
    }

    #[test]
    fn test_can_attack_not_on_battlefield() {
        let game = GameState::new(2, 20);
        let fake_id = crate::types::ids::new_object_id();
        assert!(!can_attack(&game, fake_id));
    }

    // --- playable_lands tests ---

    #[test]
    fn test_playable_lands_main_phase() {
        let mut game = GameState::new(2, 20);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let obj = GameObject::new(forest, 0, Zone::Hand);
        let id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(id);

        let lands = playable_lands(&game, 0);
        assert_eq!(lands.len(), 1);
        assert_eq!(lands[0], id);
    }

    #[test]
    fn test_playable_lands_wrong_phase() {
        let mut game = GameState::new(2, 20);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Combat);
        game.active_player = 0;

        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let obj = GameObject::new(forest, 0, Zone::Hand);
        let id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(id);

        assert!(playable_lands(&game, 0).is_empty());
    }

    #[test]
    fn test_playable_lands_already_played() {
        let mut game = GameState::new(2, 20);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;
        game.players[0].lands_played_this_turn = 1;

        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let obj = GameObject::new(forest, 0, Zone::Hand);
        let id = obj.id;
        game.add_object(obj);
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
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = PermanentState::new(id, 0, ts, 0);
        game.battlefield.insert(id, entry);

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
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = PermanentState::new(id, 0, ts, 0);
        game.battlefield.insert(id, entry);

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
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let mut entry = PermanentState::new(id, 0, ts, 0);
        entry.tapped = true;
        game.battlefield.insert(id, entry);

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
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = PermanentState::new(id, 0, ts, 0);
        game.battlefield.insert(id, entry);

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
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let mut entry = PermanentState::new(id, 0, ts, 0);
        entry.tapped = true;
        game.battlefield.insert(id, entry);

        assert!(legal_blockers(&game, 0).is_empty());
    }

    // CR 609.7a — "they may choose a permanent; a spell on the stack
    // (including a permanent spell); ... A source doesn't need to be capable
    // of dealing damage to be a legal choice." Both reachable categories are
    // offered and neither is filtered by what the object can do: a Plains is
    // on the list.
    //
    // COVERS-PARTIAL: ATOM-609.7a-001 -- the atom asks for all four of the
    // rule's categories. Two are unreachable and are RD-3's recorded
    // decision: "an object referred to by an object on the stack, by a
    // replacement or prevention effect that's waiting to apply, or by a
    // delayed triggered ability" has no referred-to relation to read (the
    // atom's own example is an emblem referring to a card in exile, and
    // CR 603.7's delayed triggers do not exist yet), and "a face-up object in
    // the command zone" needs the command zone populated, which is the
    // Commander track's. The permanent and stack-spell legs are built whole.
    //
    // COVERS-PARTIAL: BOUNDARY-DEF-609.7a-001 -- in-set (a creature permanent)
    // and out-of-set (a card in hand referred to by nothing) are both built;
    // the boundary's middle -- an object in a hidden zone that *is* referred
    // to -- is the same unreachable category.
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
            assert!(game.validate_selection(&SelectionFilter::DamageSource, choice, 0).is_ok());
        }
        assert!(game
            .validate_selection(
                &SelectionFilter::DamageSource,
                &ResolvedTarget::Object(in_hand),
                0
            )
            .is_err());
        assert!(game
            .validate_selection(&SelectionFilter::DamageSource, &ResolvedTarget::Player(0), 0)
            .is_err());
        assert!(game.has_any_legal_choice(&SelectionFilter::DamageSource, None, 0));
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
        assert!(!game.has_any_legal_choice(&SelectionFilter::DamageSource, None, 0));
    }
}
