//! Item 356, built (`replacement-architecture.md`, "Where replacement effects
//! come from"): the replacement gather finds its candidates by the kind of
//! action a proposal is, the trigger dispatcher's shape.
//!
//! **The battlefield leg** asks only the permanents that could answer: the
//! printed sources filed under the proposal's kind, the objects a named grant
//! or copy row puts an ability on whose kinds hold it, and the holders of a
//! replacement counter, each once, oldest first. Each test is a board where a
//! leg it reads is the only one that finds the answer, or where two legs name
//! one permanent.

use std::sync::Arc;

use mtgsim::cards::phase_cv_cards::cytoshape;
use mtgsim::cards::phase_rb_cards::rest_in_peace;
use mtgsim::engine::actions::{GameAction, GameActionKind, GameActionKindMask};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::objects::card_data::CardData;
use mtgsim::objects::object::GameObject;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    creature_with_ability, effect_ctx, place_on_battlefield, put_in_hand, put_on_battlefield,
    setup_two_player_game, static_ability, test_ctx, vanilla_creature,
};
use mtgsim::types::effects::{CounterType, Effect, ObjectFilter, ObjectSet};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::types::replacement::{EnterMods, EntryCounters, EnterModsTemplate, EventPattern, ReplacementDef, Rewrite};
use mtgsim::types::zones::{Zone, ZoneChangeAct, ZoneChangePattern};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::DecisionProvider;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// "Prevent all damage that would be dealt to this creature": one
/// replacement, about itself, watching damage.
fn warded(name: &str) -> Arc<CardData> {
    creature_with_ability(
        name,
        2,
        2,
        static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            ObjectSet::SourceOnly,
            Rewrite::Prevent,
        )))),
    )
}

/// "This creature enters tapped": its one replacement is its own entry.
fn enters_tapped() -> Arc<CardData> {
    creature_with_ability(
        "Tapped Arrival",
        2,
        2,
        static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::SourceOnly,
            Rewrite::EnterWith(EnterModsTemplate::tapped()),
        )))),
    )
}

fn deal_damage(game: &mut GameState, source: ObjectId, target: ObjectId, amount: u64) {
    // `test_ctx`'s provider panics on any prompt, so a candidate offered twice
    // would ask CR 616.1's question of one effect.
    game.execute_action(
        GameAction::DealDamage {
            source,
            target: ObjectOrPlayerId::Object(target),
            amount,
            is_combat: false,
            unpreventable: false,
            source_lki: None,
        },
        &test_ctx(),
    )
    .unwrap();
}

/// Cytoshape resolving: `target` becomes a copy of `donor` until end of turn,
/// through a copy row that names `target`.
fn copy_onto(game: &mut GameState, target: ObjectId, donor: ObjectId) {
    struct PickById(ObjectId);
    impl DecisionProvider for PickById {
        fn pick_n(
            &self,
            _game: &GameState,
            _player: PlayerId,
            ctx: &ChoiceContext,
            options: &[ChoiceOption],
            _bounds: (usize, usize),
        ) -> Vec<usize> {
            assert!(matches!(ctx.kind, ChoiceKind::ChooseCopySource { .. }), "unexpected prompt: {:?}", ctx.kind);
            let idx = options
                .iter()
                .position(|o| matches!(o, ChoiceOption::Object(id) if *id == self.0))
                .unwrap_or_else(|| panic!("the donor was not offered"));
            vec![idx]
        }
        fn pick_number(&self, _g: &GameState, _p: PlayerId, _c: &ChoiceContext, min: u64, _max: u64) -> u64 {
            min
        }
        fn allocate(
            &self,
            _g: &GameState,
            _p: PlayerId,
            _c: &ChoiceContext,
            total: u64,
            buckets: &[ChoiceOption],
            _mins: &[u64],
            _maxs: Option<&[u64]>,
        ) -> Vec<u64> {
            let mut out = vec![0; buckets.len()];
            if let Some(first) = out.first_mut() {
                *first = total;
            }
            out
        }
        fn choose_ordering(&self, _g: &GameState, _p: PlayerId, _c: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
            (0..items.len()).collect()
        }
    }
    let card = cytoshape();
    let spell = game.add_object(GameObject::new(card.clone(), 0, Zone::Stack));
    let ctx = ResolutionContext {
        source: spell,
        ability_source: None,
        controller: 0,
        generated_by: mtgsim::events::cause::GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ObjectOrPlayerId::Object(target)]),
        replaced_event: None,
        trigger: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &PickById(donor)).unwrap();
}

// ---------------------------------------------------------------------------
// The kinds a pattern watches, and what a source is filed under
// ---------------------------------------------------------------------------

/// A zone-change pattern watches an entry only when its destination admits
/// the battlefield and it does not start there; a counter pattern watches an
/// entry too (CR 122.6).
#[test]
fn a_pattern_watches_an_entry_only_where_the_move_could_be_one() {
    let zone_change = |from: Option<Zone>, to: Option<Zone>| EventPattern::ZoneChange {
        change: ZoneChangePattern { from, to, act: None },
        object: None,
    };
    let moves = GameActionKindMask::of(GameActionKind::ZoneChange);
    let moves_or_enters = moves.with(GameActionKind::EnterBattlefield);

    assert_eq!(zone_change(Some(Zone::Battlefield), Some(Zone::Graveyard)).action_kinds(), moves, "dies");
    assert_eq!(zone_change(Some(Zone::Battlefield), None).action_kinds(), moves, "leaves the battlefield");
    assert_eq!(zone_change(None, Some(Zone::Graveyard)).action_kinds(), moves, "put into a graveyard from anywhere");
    assert_eq!(zone_change(Some(Zone::Graveyard), None).action_kinds(), moves_or_enters, "leaves a graveyard, for anywhere");
    assert_eq!(zone_change(None, Some(Zone::Battlefield)).action_kinds(), moves_or_enters, "can't enter");
    assert_eq!(
        EventPattern::AddCounters { counter: None, by: None }.action_kinds(),
        GameActionKindMask::of(GameActionKind::AddCounters).with(GameActionKind::EnterBattlefield),
        "Hardened Scales: counters a permanent enters with are put on it"
    );
}

/// A def that affects only its own object answers nothing about its own entry
/// from where it sits (CR 614.12, 400.7); the same pattern over a filter does.
#[test]
fn an_entry_that_affects_only_its_own_object_is_never_watched_from_in_place() {
    let entry = EventPattern::EnterBattlefield { cast: None };
    let tapped = Rewrite::EnterWith(EnterModsTemplate::tapped());
    let own = ReplacementDef::new(entry.clone(), ObjectSet::SourceOnly, tapped.clone());
    let others = ReplacementDef::new(entry, ObjectSet::battlefield_filter(ObjectFilter::All), tapped);

    assert!(own.action_kinds_in_place().is_empty());
    assert_eq!(others.action_kinds_in_place(), GameActionKindMask::of(GameActionKind::EnterBattlefield));

    let counters = ReplacementDef::new(EventPattern::AddCounters { counter: None, by: None }, ObjectSet::SourceOnly, Rewrite::Prevent);
    assert_eq!(
        counters.action_kinds_in_place(),
        GameActionKindMask::of(GameActionKind::AddCounters),
        "Mowu's \"put on ~\" keeps the counters it is put later"
    );
}

/// Rest in Peace is filed under the move it watches and nothing else; a
/// creature whose one replacement is its own entry is filed nowhere, and
/// still enters tapped, which source 1a reads off it as it enters.
#[test]
fn a_printed_source_is_filed_under_the_kinds_it_answers_from_the_battlefield() {
    let mut game = setup_two_player_game();
    let peace = put_on_battlefield(&mut game, rest_in_peace(), 0);
    assert_eq!(
        game.replacement_ability_sources.get(&peace),
        Some(&GameActionKindMask::of(GameActionKind::ZoneChange)),
        "its pattern ends in a graveyard, so no entry is asked of it"
    );

    let arrival = put_in_hand(&mut game, enters_tapped(), 0);
    game.change_zone(arrival, Zone::Battlefield, ZoneChangeAct::Put, &test_ctx()).unwrap();
    assert!(game.battlefield[&arrival].tapped, "source 1a read it as it entered");
    assert!(!game.replacement_ability_sources.contains_key(&arrival), "and from the battlefield it answers nothing");
}

// ---------------------------------------------------------------------------
// The named rows, by kind
// ---------------------------------------------------------------------------

/// A creature that printed a damage replacement becomes a copy of another
/// (Cytoshape's shape): it is filed as a printed source and named by its copy
/// row, both under damage. Asked once, its one effective replacement is one
/// candidate, so CR 616.1 asks nothing (`test_ctx` panics on a prompt).
#[test]
fn a_printed_source_that_becomes_a_copy_of_another_is_offered_once() {
    let mut game = setup_two_player_game();
    let copyist = put_on_battlefield(&mut game, warded("Warded Copyist"), 0);
    let donor = put_on_battlefield(&mut game, warded("Warded Donor"), 1);
    let source = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    copy_onto(&mut game, copyist, donor);
    assert!(
        game.replacement_ability_sources[&copyist].contains(GameActionKind::DealDamage),
        "filed as printed at its entry"
    );
    assert!(
        game.continuous_effects.summary().named_unattributed_replacement_kinds.contains(GameActionKind::DealDamage),
        "and named by its copy row"
    );

    deal_damage(&mut game, source, copyist, 3);

    assert_eq!(game.battlefield[&copyist].damage_marked, 0, "the copied ability prevented it");
}

/// A copied damage replacement on a creature that printed none is found
/// beside a printed source of another kind: only the named leg names the
/// copyist, and only for the kinds its copied ability watches.
#[test]
fn a_copied_replacement_is_found_beside_a_printed_source_of_another_kind() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, rest_in_peace(), 0);
    let copyist = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let donor = put_on_battlefield(&mut game, warded("Warded Donor"), 1);
    let source = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    copy_onto(&mut game, copyist, donor);

    assert_eq!(
        game.continuous_effects.summary().named_unattributed_replacement_kinds,
        GameActionKindMask::of(GameActionKind::DealDamage),
        "the copy row is read for damage and for nothing else"
    );
    assert!(!game.replacement_ability_sources.contains_key(&copyist), "it printed nothing");

    deal_damage(&mut game, source, copyist, 3);

    assert_eq!(game.battlefield[&copyist].damage_marked, 0, "the named leg found the copied ability");
}

// ---------------------------------------------------------------------------
// The replacement-counter holders
// ---------------------------------------------------------------------------

/// A permanent is a holder from its first shield, stun or finality counter to
/// its last, and a +1/+1 counter makes it none.
#[test]
fn a_permanent_holds_a_replacement_counter_from_the_first_to_the_last() {
    let mut game = setup_two_player_game();
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let holds = |game: &GameState| game.replacement_counter_holders.contains(&bear);

    game.add_counters(bear, CounterType::PlusOnePlusOne, 1);
    assert!(!holds(&game), "a +1/+1 counter makes no replacement");
    game.add_counters(bear, CounterType::Shield, 1);
    assert!(holds(&game));
    game.remove_counters(bear, CounterType::PlusOnePlusOne, 1);
    assert!(holds(&game), "the shield is still on it");
    game.remove_counters(bear, CounterType::Shield, 1);
    assert!(!holds(&game), "its last replacement counter is gone");
}

/// A permanent that arrives with a shield counter is a holder as it lands,
/// and the shield stops the destruction (CR 122.1c); leaving takes it out.
#[test]
fn a_shield_counter_a_permanent_arrives_with_shields_it() {
    let mut game = setup_two_player_game();
    let bear = game.add_object(GameObject::new(vanilla_creature(2, 2, &[]), 0, Zone::Battlefield));
    let mods = EnterMods { counters: vec![EntryCounters { counter: CounterType::Shield, n: 1, by: None }], ..EnterMods::default() };
    place_on_battlefield(&mut game, bear, 0, &mods);
    assert!(game.replacement_counter_holders.contains(&bear));

    let source = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    game.execute_action(GameAction::Destroy { object: bear }, &effect_ctx(&game, source)).unwrap();

    assert!(game.battlefield.contains_key(&bear), "the shield replaced the destruction");
    assert_eq!(game.battlefield[&bear].counter_count(CounterType::Shield), 0, "and was removed doing it");
    assert!(!game.replacement_counter_holders.contains(&bear));

    game.add_counters(bear, CounterType::Stun, 1);
    game.change_zone(bear, Zone::Graveyard, ZoneChangeAct::Put, &test_ctx()).unwrap();
    assert!(!game.replacement_counter_holders.contains(&bear), "its counters ceased to exist as it left (CR 122.2)");
}
