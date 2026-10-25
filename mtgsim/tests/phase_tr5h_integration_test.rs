//! Phase TR-5h — one history of what happened (`triggers-architecture.md`
//! §12, TR-5h).
//!
//! Every count a card reads is a sum over the history log's rows, and each row
//! carries its object as the event left it (CR 603.10). The fixtures are
//! invented and carry their own names
//! (`engineering-practices.md#cards-and-pools`).

use std::sync::Arc;

use mtgsim::cards::phase_lg_cards::act_of_treason;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::layers::condition::{settled_holds, ConditionReferents};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::event::DamageTarget;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::get_effective_power;
use mtgsim::scenario::Scenario;
use mtgsim::engine::layers::compute_characteristics;
use mtgsim::state::battlefield::CostChoices;
use mtgsim::state::game_state::GameState;
use mtgsim::state::history_log::RowFact;
use mtgsim::test_support::{
    put_in_hand, put_on_battlefield, setup_two_player_game, static_ability, test_ctx, test_dp, vanilla_creature,
    RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, Condition, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, PlayerSet, Primitive, SelectionFilter,
    Selector, TargetCount,
};
use mtgsim::types::history::{CountIs, HistoryCount, HistorySpan, ObjectFact, TurnFact};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::triggers::Lki;
use mtgsim::types::mana::ManaCost;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

/// A {0} fixture of `card_type`; an instant or sorcery does `effect`.
fn free_spell(name: &str, card_type: CardType, effect: Effect) -> Arc<CardData> {
    let builder = CardDataBuilder::new(name).card_type(card_type).mana_cost(ManaCost::build(&[], 0));
    match card_type {
        CardType::Instant | CardType::Sorcery => builder
            .ability(AbilityDef {
                rules_text: "".into(),
                id: new_ability_id(),
                instances: Vec::new(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect,
                is_characteristic_defining: false,
                activation_restriction: ActivationRestriction::None,
            })
            .build(),
        _ => builder.build(),
    }
}

/// A sorcery that does nothing.
fn quiet_ritual() -> Arc<CardData> {
    free_spell("Quiet Ritual", CardType::Sorcery, Effect::Sequence(Vec::new()))
}

/// Cast `card` from `player`'s hand for free, then resolve it and whatever
/// its casting triggered.
fn cast_and_resolve(game: &mut GameState, player: PlayerId, card: Arc<CardData>) {
    let id = put_in_hand(game, card, player);
    game.cast_spell(player, id, &ManaWindowStop::new(RecordingDecisionProvider::picking(0))).expect("castable");
    let dp = RecordingDecisionProvider::picking(0);
    game.perform_sba_and_triggers(&dp).expect("placing");
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&dp).expect("resolving");
        game.perform_sba_and_triggers(&dp).expect("placing");
    }
}

/// "Spells you've cast", narrowed by `of`.
fn spells_you_cast(of: Option<ObjectFilter>) -> HistoryCount {
    HistoryCount { players: PlayerSet::You, fact: TurnFact::SpellsCast, of: of.map(Box::new) }
}

/// +`n`/+0 until end of turn on `id`, the way a resolving pump spell gives it.
fn pump(game: &mut GameState, id: ObjectId, n: u64) {
    let pump = Effect::Atom(
        Primitive::ModifyPowerToughness(AmountExpr::Fixed(n), AmountExpr::Fixed(0), Duration::UntilEndOfTurn),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    let source = put_in_hand(game, quiet_ritual(), 0);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: 0,
        generated_by: mtgsim::events::cause::GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(id)]),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&pump, &ctx, &test_dp()).expect("the pump");
}

/// A lifelinker's damage and its controller's gain are one event (CR
/// 702.15b). "Creatures get +1/+0 as long as you've gained life this turn" is
/// on once that event has ended, so the damage row, which carries the source
/// as the event left it, has the bonus: the window's own rows are part of
/// what it left.
#[test]
fn a_rows_frame_sees_the_rows_of_its_own_window() {
    let mut game = setup_two_player_game();
    let banner = CardDataBuilder::new("Lifted Banner")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Conditional(
            Condition::ThisTurn(HistoryCount { players: PlayerSet::You, fact: TurnFact::LifeGainEvents, of: None }, CountIs::AtLeast(1)),
            Box::new(Effect::Atom(
                Primitive::ModifyPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(0), Duration::WhileSourceOnBattlefield),
                EffectRecipient::FilteredPermanents(ObjectFilter::ByType(CardType::Creature)),
            )),
        )))
        .build();
    put_on_battlefield(&mut game, banner, 0);
    let linker = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Lifelink]), 0);
    assert_eq!(get_effective_power(&game, linker), Some(2), "no life gained yet");

    let damage = GameAction::DealDamage {
        source: linker,
        target: DamageTarget::Player(1),
        amount: 2,
        is_combat: false,
        unpreventable: false,
        source_lki: None,
    };
    game.execute_action(damage, &test_ctx()).unwrap();
    assert_eq!(get_effective_power(&game, linker), Some(3), "the gain turned the banner on");
    let row = game.history.log.rows_newest_first().find(|row| matches!(row.fact, RowFact::DealtDamage { .. }));
    let power = row.and_then(|row| row.frame.characteristics()).and_then(|chars| chars.power);
    assert_eq!(power, Some(3), "the lifelinker as the event left it");
}

/// "If you've cast a noncreature spell this turn" narrows the turn's casts by
/// what each spell was as it was cast, the frame its row keeps (CR 601.2i).
#[test]
fn a_count_narrowed_by_what_its_spells_were() {
    let mut game = setup_two_player_game();
    let watcher = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let noncreature = ObjectFilter::Not(Box::new(ObjectFilter::ByType(CardType::Creature)));
    let cast_one = Condition::ThisTurn(spells_you_cast(Some(noncreature)), CountIs::AtLeast(1));
    let holds = |game: &GameState| settled_holds(&cast_one, game, watcher, None, ConditionReferents::NONE);

    cast_and_resolve(&mut game, 0, free_spell("Clockwork Pup", CardType::Creature, Effect::Sequence(Vec::new())));
    assert!(!holds(&game), "a creature spell is not one");
    cast_and_resolve(&mut game, 0, quiet_ritual());
    assert!(holds(&game));
}

/// Two 2/2s die, one pumped to 4/2 first, and "creatures with power 3 or
/// greater that died this turn" reads each as it died, off the frame its row
/// kept: in the graveyard both are 2/2s again.
#[test]
fn a_narrowed_count_reads_the_object_as_its_row_kept_it() {
    let mut game = setup_two_player_game();
    let watcher = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let pumped = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let plain = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let power_three = ObjectFilter::Not(Box::new(ObjectFilter::PowerLE(2)));
    let died = HistoryCount { players: PlayerSet::You, fact: TurnFact::ControlledCreaturesDied, of: Some(Box::new(power_three)) };
    let one_big_one_died = Condition::ThisTurn(died, CountIs::Exactly(1));

    pump(&mut game, pumped, 2);
    for creature in [pumped, plain] {
        game.execute_action(GameAction::Destroy { object: creature }, &test_ctx()).unwrap();
    }
    assert_eq!(get_effective_power(&game, pumped), Some(2), "a 2/2 in the graveyard");
    assert!(settled_holds(&one_big_one_died, &game, watcher, None, ConditionReferents::NONE), "one died a 4/2");
}

/// "You gain life equal to the life you lost this turn" (Betor, Ancestor's
/// Voice; `codebase-state.md` item 280): a history count read as a number
/// as the spell resolves.
#[test]
fn an_amount_reads_a_history_count_as_a_spell_resolves() {
    let mut game = setup_two_player_game();
    game.execute_action(GameAction::LoseLife { player: 0, amount: 3, damage_source: None }, &test_ctx()).unwrap();
    let lost = HistoryCount { players: PlayerSet::You, fact: TurnFact::LifeLost, of: None };
    let second_wind = Effect::Atom(Primitive::GainLife(AmountExpr::History(HistorySpan::ThisTurn, Box::new(lost))), EffectRecipient::Controller);
    cast_and_resolve(&mut game, 0, free_spell("Second Wind", CardType::Sorcery, second_wind));
    assert_eq!(game.players[0].life_total, 20, "lost 3, gained 3");

    // Narrowed: "life equal to the number of creature spells you've cast
    // this turn", after a creature spell and with the sorcery itself cast.
    cast_and_resolve(&mut game, 0, free_spell("Clockwork Pup", CardType::Creature, Effect::Sequence(Vec::new())));
    let creature_spells = spells_you_cast(Some(ObjectFilter::ByType(CardType::Creature)));
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::History(HistorySpan::ThisTurn, Box::new(creature_spells))), EffectRecipient::Controller);
    cast_and_resolve(&mut game, 0, free_spell("Herd Census", CardType::Sorcery, gain));
    assert_eq!(game.players[0].life_total, 21, "one creature spell among three");
}

/// "Creatures get +X/+0, where X is the number of spells you've cast this
/// turn": the same amount in a layer, which walks again as a cast row lands.
#[test]
fn an_amount_reads_a_history_count_in_a_layer() {
    let mut game = setup_two_player_game();
    let spells = AmountExpr::History(HistorySpan::ThisTurn, Box::new(spells_you_cast(None)));
    let banner = CardDataBuilder::new("Rally Banner")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::ModifyPowerToughness(spells, AmountExpr::Fixed(0), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(ObjectFilter::ByType(CardType::Creature)),
        )))
        .build();
    put_on_battlefield(&mut game, banner, 0);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert_eq!(get_effective_power(&game, bears), Some(2));
    cast_and_resolve(&mut game, 0, quiet_ritual());
    cast_and_resolve(&mut game, 0, quiet_ritual());
    assert_eq!(get_effective_power(&game, bears), Some(4));
}

/// The board a scenario's text loads, and its one permanent: a source for
/// "you".
fn loaded(history: &str) -> (mtgsim::state::game::Game, ObjectId) {
    let text = format!("turn 3\n{history}\nbattlefield: Grizzly Bears | controller 0");
    let game = Scenario::parse(&text).and_then(|s| s.build(&CardRegistry::default_registry())).unwrap_or_else(|r| panic!("{r}")).game;
    let bears = game.state.battlefield_ids_ordered()[0];
    (game, bears)
}

/// A scenario names a cast by its card (`setup-architecture.md` §5.1): it
/// loads as its printed frame, so a narrowed count reads it, the unnamed
/// count adds to it, and a save writes both back.
#[test]
fn a_named_cast_loads_as_its_printed_card_and_saves_by_name() {
    let count_holds = |game: &GameState, source, count: HistoryCount, is| {
        settled_holds(&Condition::ThisTurn(count, is), game, source, None, ConditionReferents::NONE)
    };
    let (game, bears) = loaded("player 0 this turn: spells cast 2, cast Lightning Bolt");
    assert!(count_holds(&game.state, bears, spells_you_cast(None), CountIs::Exactly(3)), "two unnamed and one named");
    let written = Scenario::write(&game.state).to_string();
    assert!(written.contains("player 0 this turn: spells cast 2, cast Lightning Bolt"), "{written}");

    let (game, bears) = loaded("player 0 this turn: cast Lightning Bolt\nplayer 0 this turn: cast Grizzly Bears");
    let instants = spells_you_cast(Some(ObjectFilter::ByType(CardType::Instant)));
    assert!(count_holds(&game.state, bears, instants, CountIs::Exactly(1)));
}

/// A card that narrows a count a scenario stated without naming its spells
/// asks what the file did not say, which is loud in a debug build.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "without naming its objects")]
fn narrowing_an_unnamed_count_is_loud() {
    let (game, bears) = loaded("player 0 this turn: spells cast 2");
    let instants = Condition::ThisTurn(spells_you_cast(Some(ObjectFilter::ByType(CardType::Instant))), CountIs::AtLeast(1));
    settled_holds(&instants, &game.state, bears, None, ConditionReferents::NONE);
}

/// "Creatures [that did `fact`] this turn get +1/+0", from player 0's
/// enchantment.
fn banner_over(fact: ObjectFact) -> Arc<CardData> {
    CardDataBuilder::new("Veteran's Banner")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::ModifyPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(0), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(ObjectFilter::ThisTurn(fact)),
        )))
        .build()
}

fn deal_damage(game: &mut GameState, source: ObjectId, target: DamageTarget, is_combat: bool) {
    let damage = GameAction::DealDamage { source, target, amount: 1, is_combat, unpreventable: false, source_lki: None };
    game.execute_action(damage, &test_ctx()).unwrap();
}

/// "Was dealt damage this turn" reads the damage rows that name the object
/// as what the damage was dealt to (CR 120.3), so the source is not one.
#[test]
fn was_dealt_damage_this_turn_reads_the_rows_dealt_to_it() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, banner_over(ObjectFact::WasDealtDamage), 0);
    let wall = put_on_battlefield(&mut game, vanilla_creature(0, 4, &[]), 0);
    let shooter = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    deal_damage(&mut game, shooter, DamageTarget::Object(wall), false);
    assert_eq!(get_effective_power(&game, wall), Some(1), "the wall was dealt damage");
    assert_eq!(get_effective_power(&game, shooter), Some(1), "the shooter dealt it");
}

/// "Dealt combat damage to an opponent this turn": a combat damage row to a
/// player other than the banner's controller (CR 120.1, 510.2). Noncombat
/// damage is not one, nor is combat damage to "you".
#[test]
fn dealt_combat_damage_to_an_opponent_reads_whom_and_how() {
    let mut game = setup_two_player_game();
    let fact = ObjectFact::DealtDamageTo { player: Some(PlayerRef::Opponent), combat: true };
    put_on_battlefield(&mut game, banner_over(fact), 0);
    let raider = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let pinger = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let theirs = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    deal_damage(&mut game, raider, DamageTarget::Player(1), true);
    deal_damage(&mut game, pinger, DamageTarget::Player(1), false);
    deal_damage(&mut game, theirs, DamageTarget::Player(0), true);
    let powers = [raider, pinger, theirs].map(|id| get_effective_power(&game, id));
    assert_eq!(powers, [Some(3), Some(2), Some(2)]);
}

/// `player` gains control of `victim` the way Act of Treason's resolution
/// gives it (a Layer 2 row).
fn steal(game: &mut GameState, victim: ObjectId, player: PlayerId) {
    let treason = put_in_hand(game, act_of_treason(), player);
    let ctx = ResolutionContext {
        source: treason,
        ability_source: None,
        controller: player,
        generated_by: mtgsim::events::cause::GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(victim)]),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&act_of_treason().abilities[0].effect, &ctx, &test_dp()).expect("the steal");
}

/// "Creatures get +X/+0", an artifact's static, with X `amount`.
fn anthem_of(amount: AmountExpr) -> Arc<CardData> {
    CardDataBuilder::new("Measuring Totem")
        .card_type(CardType::Artifact)
        .ability(static_ability(Effect::Atom(
            Primitive::ModifyPowerToughness(amount, AmountExpr::Fixed(0), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(ObjectFilter::ByType(CardType::Creature)),
        )))
        .build()
}

/// A static ability's "you" is its source's controller now (CR 109.5), so
/// once its source is stolen an amount it reads counts the thief's: the
/// spells they cast this turn, the cards in their hand.
#[test]
fn a_static_amounts_you_follows_its_source_to_a_new_controller() {
    let quick = || free_spell("Quick Ritual", CardType::Instant, Effect::Sequence(Vec::new()));
    let spells = AmountExpr::History(HistorySpan::ThisTurn, Box::new(spells_you_cast(None)));
    let hand = AmountExpr::CountOf(Selector::CardsInHand(PlayerRef::You));
    for (amount, what) in [(spells, "spells cast"), (hand, "cards in hand")] {
        let mut game = setup_two_player_game();
        let totem = put_on_battlefield(&mut game, anthem_of(amount), 0);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        cast_and_resolve(&mut game, 1, quick());
        cast_and_resolve(&mut game, 1, quick());
        let before = get_effective_power(&game, bears);
        steal(&mut game, totem, 1);
        // Player 1 cast two spells, and holds the Act of Treason that stole it.
        let theirs = if what == "spells cast" { 2 } else { game.players[1].hand.len() as i32 };
        assert!(theirs > 0, "{what}: player 0 has none, so the two readings differ");
        assert_eq!((before, get_effective_power(&game, bears)), (Some(2), Some(2 + theirs)), "{what}: player 1's");
    }
}

/// A source the store has lost answers "its owner" off its last known
/// information (CR 108.3), in a history count's filter as in every other.
#[test]
fn a_gone_sources_history_filter_reads_its_owner_off_its_last_known_information() {
    let mut game = setup_two_player_game();
    let relic = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    cast_and_resolve(&mut game, 0, quiet_ritual());
    let lki = Lki::new(
        game.object_ref(relic).unwrap(),
        compute_characteristics(&game, relic).unwrap(),
        0,
        false,
        CostChoices::NONE,
        game.battlefield.get(&relic),
        None,
    );
    game.battlefield.remove(&relic);
    game.objects.remove(&relic);
    let owners_spells = spells_you_cast(Some(ObjectFilter::ByOwner(PlayerRef::Owner)));
    let cast_one = Condition::ThisTurn(owners_spells, CountIs::AtLeast(1));
    let referents = ConditionReferents { source_lki: Some(&lki), ..ConditionReferents::NONE };
    assert!(settled_holds(&cast_one, &game, relic, Some(0), referents), "player 0 owned both");
}

/// "Your last turn" is written as the turn begins, by the turn's own writer.
#[test]
fn your_last_turn_is_written_as_the_turn_begins() {
    let mut game = GameState::new(2, 20);
    game.begin_turn(2, 1);
    game.begin_turn(3, 0);
    let history = &game.history.players;
    assert_eq!((history[0].your_last_turn(3), history[1].your_last_turn(3)), (Some(1), Some(2)));
}
