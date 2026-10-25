//! A6m (`roadmap-v2.md` A6m). Its first PR: the set a resolution affects
//! (`codebase-state.md` item 244) and the board's queries (item 242). Its
//! second: one `ObjectFilter` leaf table (item 245).

use std::sync::Arc;

use mtgsim::engine::actions::ActionContext;
use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, EffectOrigin, Layer};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{
    get_effective_abilities, get_effective_colors, get_effective_controller, get_effective_name, get_effective_power,
    get_effective_toughness, has_keyword, has_type,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    card_of_type, creature_with_ability, forest, place_bare, put_in_graveyard, put_on_battlefield,
    put_on_battlefield_under, set_attacking, setup_two_player_game, static_ability, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    Affected, AmountExpr, ColorChange, Condition, CopyException, CounterType, Duration, Effect, EffectRecipient, ObjectFilter,
    ObjectSet, PlayerRef, Primitive, SelectionFilter, TargetCount, TypeChange,
};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::replacement::{
    CopyDonor, EnterModsTemplate, EntryCopyTemplate, EventPattern, GameActionTemplate, ReplacementDef, Rewrite,
};
use mtgsim::types::restriction::{Restriction, RestrictionDef, SourceFilter};
use mtgsim::types::zones::{Zone, ZoneChangeAct, ZoneChangePattern};
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// The boards
// ---------------------------------------------------------------------------

/// "Creatures target player controls" in a two-player game, where the target
/// is the one opponent: a filter has no leaf for a targeted player, so the
/// fixture names the opponent.
fn opponents_creatures() -> EffectRecipient {
    EffectRecipient::FilteredPermanents(ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerRef::Opponent)),
    ))
}

/// Resolve `effect` as an instant `controller` cast, with `targets` chosen.
fn resolve(game: &mut GameState, controller: PlayerId, effect: &Effect, targets: Vec<ObjectOrPlayerId>) {
    let source = place_bare(game, card_of_type("A6m Instant", CardType::Instant), controller);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(targets),
        replaced_event: None,
        trigger: None,
    };
    game.resolve_effect(effect, &ctx, &ScriptedDecisionProvider::new()).expect("resolving");
}

/// "Destroy target creature", resolved by player 0.
fn destroy(game: &mut GameState, creature: ObjectId) {
    let effect = Effect::Atom(
        Primitive::Destroy,
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    resolve(game, 0, &effect, vec![ObjectOrPlayerId::Object(creature)]);
}

fn power_toughness(game: &GameState, id: ObjectId) -> (Option<i32>, Option<i32>) {
    (get_effective_power(game, id), get_effective_toughness(game, id))
}

// ---------------------------------------------------------------------------
// CR 611.2c — a resolution's characteristic change, fixed as it begins
// ---------------------------------------------------------------------------

/// Partial for the atom, whose filter is a color and whose late arrival
/// becomes white; here the filter is a controller and the creature comes
/// under that player's control. The claim is the same: the set is fixed as
/// the effect begins.
// COVERS-PARTIAL: ATOM-611.2c-001
#[test]
fn test_a_resolving_set_power_toughness_over_a_filter_affects_each_creature_it_names() {
    // Sudden Spoiling's 0/2. Before the fix the primitive read only its
    // targets, of which a filtered recipient has none, so nothing changed.
    let mut game = setup_two_player_game();
    let theirs = [
        put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1),
        put_on_battlefield(&mut game, vanilla_creature(4, 5, &[]), 1),
    ];
    let mine = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    let effect = Effect::Atom(
        Primitive::SetPowerToughness(AmountExpr::Fixed(0), AmountExpr::Fixed(2), Duration::UntilEndOfTurn),
        opponents_creatures(),
    );
    resolve(&mut game, 0, &effect, Vec::new());

    for id in theirs {
        assert_eq!(power_toughness(&game, id), (Some(0), Some(2)), "each creature the filter names is 0/2");
    }
    assert_eq!(power_toughness(&game, mine), (Some(2), Some(2)), "the caster's own creature is not named");

    // Sudden Spoiling's ruling: a creature the player controls later in the
    // turn is not in the set, whether it arrives or changes hands.
    let later = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    assert_eq!(power_toughness(&game, later), (Some(3), Some(3)));
    let gain_control = Effect::Atom(
        Primitive::GainControl(Duration::Indefinite),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    resolve(&mut game, 1, &gain_control, vec![ObjectOrPlayerId::Object(mine)]);
    assert_eq!(get_effective_controller(&game, mine), Some(1));
    assert_eq!(power_toughness(&game, mine), (Some(2), Some(2)));
}

/// A characteristic-changing primitive, the creatures it is resolved over, and
/// whether a creature shows its effect.
struct Case {
    name: &'static str,
    primitive: Primitive,
    creature: fn() -> std::sync::Arc<mtgsim::objects::card_data::CardData>,
    shows: fn(&GameState, ObjectId) -> bool,
}

/// The one ability both creatures in `LoseAbility`'s case carry, by one id.
fn shared_ability() -> mtgsim::objects::card_data::AbilityDef {
    use std::sync::OnceLock;
    static ABILITY: OnceLock<mtgsim::objects::card_data::AbilityDef> = OnceLock::new();
    ABILITY
        .get_or_init(|| {
            static_ability(Effect::Atom(
                Primitive::ModifyPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(0), Duration::Indefinite),
                EffectRecipient::ThisObject,
            ))
        })
        .clone()
}

/// "This creature gets +1/+1", granted.
fn granted_anthem() -> mtgsim::objects::card_data::AbilityDef {
    use std::sync::OnceLock;
    static ABILITY: OnceLock<mtgsim::objects::card_data::AbilityDef> = OnceLock::new();
    ABILITY
        .get_or_init(|| {
            static_ability(Effect::Atom(
                Primitive::ModifyPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(1), Duration::Indefinite),
                EffectRecipient::ThisObject,
            ))
        })
        .clone()
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            name: "SwitchPowerToughness",
            primitive: Primitive::SwitchPowerToughness(Duration::UntilEndOfTurn),
            creature: || vanilla_creature(1, 3, &[]),
            shows: |game, id| power_toughness(game, id) == (Some(3), Some(1)),
        },
        Case {
            name: "ChangeColor",
            primitive: Primitive::ChangeColor(ColorChange::Add(Color::Blue), Duration::UntilEndOfTurn),
            creature: || vanilla_creature(2, 2, &[]),
            shows: |game, id| get_effective_colors(game, id).contains(&Color::Blue),
        },
        Case {
            name: "ChangeType",
            primitive: Primitive::ChangeType(
                TypeChange { add_types: vec![CardType::Artifact], ..TypeChange::NONE },
                Duration::UntilEndOfTurn,
            ),
            creature: || vanilla_creature(2, 2, &[]),
            shows: |game, id| has_type(game, id, CardType::Artifact),
        },
        // The grant and the rows its static ability generates on each grantee
        // (CR 613.7a), so the +1/+1 is the check.
        Case {
            name: "GrantAbility",
            primitive: Primitive::GrantAbility(std::sync::Arc::new(granted_anthem()), Duration::UntilEndOfTurn),
            creature: || vanilla_creature(2, 2, &[]),
            shows: |game, id| {
                get_effective_abilities(game, id).iter().any(|a| a.id.definition() == granted_anthem().id.definition())
                    && power_toughness(game, id) == (Some(3), Some(3))
            },
        },
        Case {
            name: "RemoveKeywordFlag",
            primitive: Primitive::RemoveKeywordFlag(KeywordFlag::Flying, Duration::UntilEndOfTurn),
            creature: || vanilla_creature(2, 2, &[KeywordFlag::Flying]),
            shows: |game, id| !has_keyword(game, id, KeywordFlag::Flying),
        },
        Case {
            name: "LoseAbility",
            primitive: Primitive::LoseAbility(shared_ability().id, Duration::UntilEndOfTurn),
            creature: || creature_with_ability("A6m Bear", 2, 2, shared_ability()),
            shows: |game, id| !get_effective_abilities(game, id).iter().any(|a| a.id == shared_ability().id),
        },
        Case {
            name: "LoseAllAbilities",
            primitive: Primitive::LoseAllAbilities(Duration::UntilEndOfTurn),
            creature: || vanilla_creature(2, 2, &[KeywordFlag::Flying]),
            shows: |game, id| !has_keyword(game, id, KeywordFlag::Flying),
        },
    ]
}

#[test]
fn test_each_characteristic_changing_primitive_takes_a_filtered_set_at_resolution() {
    // Every case is run before any is judged, so a failure names them all.
    let mut wrong = Vec::new();
    for case in cases() {
        let mut game = setup_two_player_game();
        let theirs = [
            put_on_battlefield(&mut game, (case.creature)(), 1),
            put_on_battlefield(&mut game, (case.creature)(), 1),
        ];
        let mine = put_on_battlefield(&mut game, (case.creature)(), 0);
        assert!(!(case.shows)(&game, theirs[0]), "{}: the board starts unaffected", case.name);

        resolve(&mut game, 0, &Effect::Atom(case.primitive.clone(), opponents_creatures()), Vec::new());
        let later = put_on_battlefield(&mut game, (case.creature)(), 1);

        if !theirs.iter().all(|&id| (case.shows)(&game, id)) {
            wrong.push(format!("{}: misses a creature the filter names", case.name));
        }
        if (case.shows)(&game, mine) {
            wrong.push(format!("{}: reaches a creature the filter does not name", case.name));
        }
        if (case.shows)(&game, later) {
            wrong.push(format!("{}: reaches a creature that arrived after (CR 611.2c)", case.name));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn test_gaining_control_of_a_filtered_set_takes_the_permanents_it_names_then() {
    // Mob Rule's "gain control of all creatures with power 4 or greater":
    // CR 611.2c names a control change beside a characteristic change.
    let mut game = setup_two_player_game();
    let theirs = [
        put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1),
        put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1),
    ];
    let effect = Effect::Atom(Primitive::GainControl(Duration::UntilEndOfTurn), opponents_creatures());
    resolve(&mut game, 0, &effect, Vec::new());
    let later = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    for id in theirs {
        assert_eq!(get_effective_controller(&game, id), Some(0), "each creature the filter named");
    }
    assert_eq!(get_effective_controller(&game, later), Some(1), "fixed as the effect began");
}

// ---------------------------------------------------------------------------
// One-shot instructions over a filtered set (CR 608.2h)
// ---------------------------------------------------------------------------

#[test]
fn test_regenerate_each_creature_you_control_shields_each_one() {
    // Golgari Charm's third mode. A shield per creature, for the creatures
    // there are as it resolves.
    let mut game = setup_two_player_game();
    let mine = [
        put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0),
        put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0),
    ];
    let effect = Effect::Atom(
        Primitive::Regenerate,
        EffectRecipient::FilteredPermanents(ObjectFilter::And(
            Box::new(ObjectFilter::ByType(CardType::Creature)),
            Box::new(ObjectFilter::ByController(PlayerRef::You)),
        )),
    );
    resolve(&mut game, 0, &effect, Vec::new());
    let later = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    for id in mine {
        destroy(&mut game, id);
        let entry = game.battlefield.get(&id).expect("regenerated, so still on the battlefield");
        assert!(entry.tapped, "CR 701.19a: regeneration taps it");
    }
    destroy(&mut game, later);
    assert!(!game.battlefield.contains_key(&later), "a creature arriving after has no shield");
}

#[test]
fn test_remove_from_combat_and_remove_all_damage_take_a_filtered_set() {
    let mut game = setup_two_player_game();
    let theirs = [
        put_on_battlefield(&mut game, vanilla_creature(2, 4, &[]), 1),
        put_on_battlefield(&mut game, vanilla_creature(2, 4, &[]), 1),
    ];
    for id in theirs {
        set_attacking(&mut game, id, 0);
        game.battlefield.get_mut(&id).expect("on the battlefield").damage_marked = 2;
    }

    resolve(&mut game, 0, &Effect::Atom(Primitive::RemoveFromCombat, opponents_creatures()), Vec::new());
    resolve(&mut game, 0, &Effect::Atom(Primitive::RemoveAllDamage, opponents_creatures()), Vec::new());

    for id in theirs {
        let entry = game.battlefield.get(&id).expect("on the battlefield");
        assert!(entry.attacking.is_none(), "removed from combat");
        assert_eq!(entry.damage_marked, 0, "its damage removed");
    }
}

// ---------------------------------------------------------------------------
// CR 611.2c's second sentence — a resolution's "can't" over a filter is read
// live, so it reaches objects that were not there when it began
// ---------------------------------------------------------------------------

/// Partial for the atom, whose rules-modifying effect is a prevention; this
/// one is a "can't", and the claim is the same.
// COVERS-PARTIAL: ATOM-611.2c-002
#[test]
fn test_a_resolving_restriction_over_a_filter_reaches_creatures_that_arrive_later() {
    // Magmatic Chasm's ruling: "Creatures without flying that enter the
    // battlefield later in the turn won't be able to block." Here the
    // restriction is "can't be destroyed", over the opponent's creatures.
    let mut game = setup_two_player_game();
    let theirs = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let mine = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let effect = Effect::Atom(
        Primitive::Restrict(
            RestrictionDef::new(Restriction::Event {
                pattern: EventPattern::Destroy { source: None },
                affected: Affected::NOTHING,
                by: None,
            }),
            Duration::UntilEndOfTurn,
        ),
        opponents_creatures(),
    );
    resolve(&mut game, 0, &effect, Vec::new());
    let later = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    for id in [theirs, later] {
        destroy(&mut game, id);
        assert!(game.battlefield.contains_key(&id), "the filter names it, now as then");
    }
    destroy(&mut game, mine);
    assert!(!game.battlefield.contains_key(&mine), "the filter never named the caster's own");
}

// ---------------------------------------------------------------------------
// The prohibition gate's granted leg sees through "as long as"
// ---------------------------------------------------------------------------

#[test]
fn test_a_granted_conditional_restriction_is_the_boards_only_restriction_and_applies() {
    // "As long as this creature is untapped, it can't be destroyed", granted
    // by a resolution: no printed source, no registry row, so the gate opens
    // on the summary's granted leg or not at all.
    let cant_be_destroyed_while_untapped = static_ability(Effect::Conditional(
        Condition::SourceUntapped,
        Box::new(Effect::Restriction(Box::new(RestrictionDef::new(Restriction::Event {
            pattern: EventPattern::Destroy { source: None },
            affected: Affected::objects(ObjectSet::SourceOnly),
            by: None,
        })))),
    ));
    let mut game = setup_two_player_game();
    let untapped = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let tapped = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let grant = Effect::Atom(
        Primitive::GrantAbility(std::sync::Arc::new(cant_be_destroyed_while_untapped), Duration::UntilEndOfTurn),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(2)),
    );
    resolve(&mut game, 1, &grant, vec![ObjectOrPlayerId::Object(untapped), ObjectOrPlayerId::Object(tapped)]);
    game.battlefield.get_mut(&tapped).expect("on the battlefield").tapped = true;
    assert!(game.restriction_ability_sources.is_empty() && game.restrictions.is_empty());

    destroy(&mut game, untapped);
    assert!(game.battlefield.contains_key(&untapped), "the granted \"can't\" holds while its condition does");
    destroy(&mut game, tapped);
    assert!(!game.battlefield.contains_key(&tapped), "and not once it fails");
}

// ---------------------------------------------------------------------------
// One `ObjectFilter` leaf table (`layers-architecture.md` §3.4)
// ---------------------------------------------------------------------------

/// "If a permanent `affected` names would be put into a graveyard from the
/// battlefield, exile it instead", with the pattern's own object clause and a
/// cause clause when given.
fn exiler(affected: ObjectFilter, object: Option<ObjectFilter>, by: Option<SourceFilter>) -> Arc<CardData> {
    let mut def = ReplacementDef::new(
        EventPattern::ZoneChange {
            change: ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Graveyard), act: None },
            object,
        },
        ObjectSet::battlefield_filter(affected),
        Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Exile }),
    );
    if let Some(by) = by {
        def = def.caused_by(by);
    }
    CardDataBuilder::new("A6m Exiler")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Replacement(Box::new(def))))
        .build()
}

/// Sacrifice `id` as a rule would, and say where it went.
fn sacrifice(game: &mut GameState, id: ObjectId) -> Zone {
    let dp = ScriptedDecisionProvider::new();
    game.change_zone(id, Zone::Graveyard, ZoneChangeAct::Sacrificed, &ActionContext::new(&dp)).unwrap();
    game.get_object(id).unwrap().zone
}

fn creature_and(filter: ObjectFilter) -> ObjectFilter {
    ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(filter))
}

/// CR 208.3: a land has no power, so it is not "power 4 or greater" either.
/// The layer walk answered `PowerLE` of a land with `false`, and so its
/// negation with `true`.
// COVERS-PARTIAL: ATOM-208.3-001
#[test]
fn test_power_four_or_greater_does_not_name_a_land_in_the_layer_walk() {
    let mut game = setup_two_player_game();
    let land = put_on_battlefield(&mut game, forest(), 0);
    let big = put_on_battlefield(&mut game, vanilla_creature(5, 5, &[]), 0);
    let small = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(ContinuousEffect {
        id: 0,
        source: big,
        origin: EffectOrigin::Resolution,
        layer: Layer::Layer5Color,
        duration: Duration::UntilEndOfTurn,
        controller: 0,
        created_on_turn: 1,
        timestamp,
        affected_objects: ObjectSet::battlefield_filter(ObjectFilter::Not(Box::new(ObjectFilter::PowerLE(3)))),
        modification: EffectModification::AddColor(Color::Red),
    });

    assert!(get_effective_colors(&game, big).contains(&Color::Red));
    assert!(!get_effective_colors(&game, small).contains(&Color::Red));
    assert!(!get_effective_colors(&game, land).contains(&Color::Red), "a land has no power to compare");
}

/// The selection side refused `PowerLE` of a land as a malformed question, so
/// a `matched` asker panicked in a debug build: a filter meeting a board, not
/// a card asking what its site cannot answer.
// COVERS-PARTIAL: ATOM-208.3-001
#[test]
fn test_a_land_is_not_power_two_or_less_and_asking_is_not_malformed() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, exiler(ObjectFilter::PowerLE(2), None, None), 0);
    let land = put_on_battlefield(&mut game, forest(), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    assert_eq!(sacrifice(&mut game, land), Zone::Graveyard);
    assert_eq!(sacrifice(&mut game, bear), Zone::Exile);
}

/// An arm that decides decides on either side (Kleene's "or"): a refusal on
/// the left no longer hides a token on the right.
#[test]
fn test_an_or_with_a_powerless_arm_beside_a_true_one_is_true() {
    let mut game = setup_two_player_game();
    let either = ObjectFilter::Or(Box::new(ObjectFilter::PowerLE(2)), Box::new(ObjectFilter::Token));
    put_on_battlefield(&mut game, exiler(either, None, None), 0);
    let treasure = put_on_battlefield(&mut game, card_of_type("A6m Treasure", CardType::Artifact), 0);
    game.objects.get_mut(&treasure).unwrap().is_token = true;

    assert_eq!(sacrifice(&mut game, treasure), Zone::Exile);
}

/// Xantcha's "planeswalkers its owner controls": in a filter, `Owner` is the
/// source's owner (`PlayerRef::Owner`'s own doc). The selection side read it
/// as the tested object's owner, and `ByOwner(Owner)` as always true.
#[test]
fn test_owner_in_an_affected_set_is_the_sources_owner() {
    // Player 0 owns each effect's source; player 1 controls it.
    let controlled_by_its_owner = creature_and(ObjectFilter::ByController(PlayerRef::Owner));
    let mut game = setup_two_player_game();
    put_on_battlefield_under(&mut game, exiler(controlled_by_its_owner, None, None), 0, 1);
    let stolen = put_on_battlefield_under(&mut game, vanilla_creature(2, 2, &[]), 1, 0);
    let theirs = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    assert_eq!(sacrifice(&mut game, stolen), Zone::Exile, "player 0 controls it");
    assert_eq!(sacrifice(&mut game, theirs), Zone::Graveyard, "its own owner controls it, and that is not player 0");

    let owned_by_its_owner = creature_and(ObjectFilter::ByOwner(PlayerRef::Owner));
    let mut game = setup_two_player_game();
    put_on_battlefield_under(&mut game, exiler(owned_by_its_owner, None, None), 0, 1);
    let mine = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let theirs = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    assert_eq!(sacrifice(&mut game, mine), Zone::Exile);
    assert_eq!(sacrifice(&mut game, theirs), Zone::Graveyard, "player 1 owns it");
}

/// "Caused by a spell its owner controls" names the source's owner, not its
/// controller, whom `SourceFilter` read it as.
#[test]
fn test_a_cause_its_owner_controls_is_the_sources_owner() {
    let mut game = setup_two_player_game();
    let by_its_owner = Some(SourceFilter::ControlledBy(PlayerRef::Owner));
    put_on_battlefield_under(&mut game, exiler(creature_and(ObjectFilter::All), None, by_its_owner), 0, 1);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    destroy(&mut game, bear);
    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Exile, "player 0's spell destroyed it");
}

/// A replacement's pattern names "another creature" against the effect's own
/// source, which the pattern's filter was not given: `NotSource` was refused,
/// and a debug build panicked.
#[test]
fn test_a_pattern_filter_knows_its_effects_source() {
    let mut game = setup_two_player_game();
    let another = ReplacementDef::new(
        EventPattern::ZoneChange {
            change: ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Graveyard), act: None },
            object: Some(creature_and(ObjectFilter::NotSource)),
        },
        ObjectSet::battlefield_filter(ObjectFilter::All),
        Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Exile }),
    );
    let ability = static_ability(Effect::Replacement(Box::new(another)));
    let watcher = put_on_battlefield(&mut game, creature_with_ability("A6m Watcher", 2, 2, ability), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    assert_eq!(sacrifice(&mut game, bear), Zone::Exile);
    assert_eq!(sacrifice(&mut game, watcher), Zone::Graveyard, "not another creature");
}

/// **Fixture.** "You may have this enter as a copy of an artifact you
/// control, except it enters with an additional +1/+1 counter on it if it has
/// power 2 or less": Spark Double's shape (CR 707.9f) with a condition no
/// printed copier has, which a noncreature copy has no power to answer (CR
/// 208.3).
fn small_artifact_copier() -> Arc<CardData> {
    let artifact_you_control = ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Artifact)),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    );
    let counter = CopyException::Additionally(EnterModsTemplate::with_counters(CounterType::PlusOnePlusOne, 1));
    let copy = Rewrite::EnterAsCopy(EntryCopyTemplate {
        donor: CopyDonor::Chosen(SelectionFilter::Permanent(artifact_you_control)),
        except: vec![CopyException::If(ObjectFilter::PowerLE(2), vec![counter])],
    });
    let def = ReplacementDef {
        optional: true,
        ..ReplacementDef::new(EventPattern::EnterBattlefield { cast: None }, ObjectSet::SourceOnly, copy)
    };
    CardDataBuilder::new("A6m Small Copier")
        .card_type(CardType::Artifact)
        .ability(static_ability(Effect::Replacement(Box::new(def))))
        .build()
}

/// A copy exception's condition is the rules' question about the copy as it
/// enters, so one it has no answer to is no, not an error: copying an
/// artifact, the exception's "power 2 or less" has no power to read, and the
/// entry failed.
// COVERS-PARTIAL: ATOM-208.3-001
#[test]
fn test_an_entry_copy_condition_a_noncreature_cannot_answer_is_no() {
    let mut game = setup_two_player_game();
    let relic = put_on_battlefield(&mut game, card_of_type("A6m Relic", CardType::Artifact), 0);
    let copier = put_in_graveyard(&mut game, small_artifact_copier(), 0);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(ChoiceKind::ChooseCopySource { source: copier }, vec![ChoiceOption::Object(relic)]);

    game.change_zone(copier, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&dp)).unwrap();

    assert!(dp.is_empty());
    assert_eq!(get_effective_name(&game, copier), "A6m Relic");
    assert_eq!(game.battlefield[&copier].counter_count(CounterType::PlusOnePlusOne), 0);
}
