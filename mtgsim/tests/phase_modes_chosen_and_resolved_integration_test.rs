//! `modes-chosen-and-resolved`: CR 700.2's modes, chosen as a spell is cast,
//! an ability activated or a trigger put on the stack, each chosen mode's
//! targets announced after it, and only the chosen modes resolved
//! (`choices-architecture.md`, decisions 2 and 3).
//!
//! **Every board is cast, activated or triggered, never staged**: the modes
//! are a question asked on that path, and a staged entry cannot check it.

use std::cell::RefCell;

use mtgsim::cards::authoring::{
    activated_ability, choose, choose_one, creature, destroy, draw_cards, enchantment, gain_life, gets, spell_ability, tap, target_creature,
    target_permanent, you,
};
use mtgsim::cards::fixtures::{branching_beacon, forked_omen, hesitant_omen, kindled_option, mode_lever, tally_of_choices};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::objects::card_data::CardDataBuilder;
use mtgsim::oracle::characteristics::get_effective_abilities;
use mtgsim::oracle::mana_helpers::{can_activate, CannotActivate};
use mtgsim::types::card_types::CardType;
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::{Condition, Effect, ModalCount, ObjectFilter};
use mtgsim::types::mana::ManaCost;
use mtgsim::cards::printings::{abrade, casualties_of_war, cryptic_command, forest, grizzly_bears, hill_giant, paladin_of_atonement};
use mtgsim::oracle::mana_helpers::{can_cast, CannotCast};
use mtgsim::oracle::x_values::LegalXValues;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{fill_library, put_in_hand, put_on_battlefield, setup_two_player_game, test_ctx, test_dp};
use mtgsim::types::effects::ModeIndex;
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::{Zone, ZoneChangeAct};
use mtgsim::ui::choice_types::{position_named, ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider, SeatMode};
use mtgsim::ui::display::option_label;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

/// A provider that keeps each `ChooseModes` offer it is shown and answers
/// through `inner`. An offer of one set is answered unasked and never
/// reaches it.
struct ModesOffered<D> {
    inner: D,
    offers: RefCell<Vec<Vec<ChoiceOption>>>,
}

impl<D> ModesOffered<D> {
    fn new(inner: D) -> Self {
        ModesOffered { inner, offers: RefCell::new(Vec::new()) }
    }

    fn offers(&self) -> Vec<Vec<ChoiceOption>> {
        self.offers.borrow().clone()
    }
}

impl<D: DecisionProvider> DecisionProvider for ModesOffered<D> {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        if matches!(context.kind, ChoiceKind::ChooseModes { .. }) {
            self.offers.borrow_mut().push(options.to_vec());
        }
        self.inner.pick_n(game, player, context, options, bounds)
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
        self.inner.pick_number(game, player, context, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.inner.allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        self.inner.choose_ordering(game, player, context, items)
    }

    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        self.inner.seat_mode(player)
    }
}

/// The script's next answer: the modes printed at `places`.
fn chooses_modes(answers: &ScriptedDecisionProvider, places: &[u8]) {
    answers.expect_choice(ChoiceKind::ChooseModes { source: ObjectId::UNASSIGNED }, vec![ChoiceOption::modes(places)]);
}

/// The script's next answer: `chosen` for an instance of "target".
fn names_targets(answers: &ScriptedDecisionProvider, chosen: Vec<ChoiceOption>) {
    let kind = ChoiceKind::SelectRecipients { recipient: mtgsim::types::effects::EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED };
    answers.expect_choice(kind, chosen);
}

/// The X question, as a script matches it by kind.
fn choose_x() -> ChoiceKind {
    ChoiceKind::ChooseXValue { spell_id: ObjectId::UNASSIGNED, values: Box::new(LegalXValues::zero_through(0)) }
}

fn modes(places: &[u8]) -> Vec<ModeIndex> {
    places.iter().map(|&place| ModeIndex::printed(place)).collect()
}

/// Each offer's options as the log writes them.
fn offered(game: &GameState, offers: &[Vec<ChoiceOption>]) -> Vec<Vec<String>> {
    offers.iter().map(|offer| offer.iter().map(|option| option.as_logged(game)).collect()).collect()
}

fn damage_on(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield[&id].damage_marked
}

/// `card` cast by player 0 from `mana` floating, and resolved: it enters,
/// and what it triggered waits to be placed.
fn cast_and_resolve(game: &mut GameState, card: std::sync::Arc<mtgsim::objects::card_data::CardData>, mana: &[(ManaType, u64)]) -> ObjectId {
    let id = put_in_hand(game, card, 0);
    for &(mana_type, n) in mana {
        game.players[0].mana_pool.add(mana_type, n);
    }
    game.cast_spell(0, id, &ManaWindowStop::new(test_dp())).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    id
}

// ---------------------------------------------------------------------------
// CR 603.3c: a modal trigger's modes, chosen as it is put on the stack
// ---------------------------------------------------------------------------

/// ATOM-603.3c-001's board: no creature anywhere, so the damage mode can't be
/// chosen and the life mode is the one legal set, announced without asking.
// COVERS: ATOM-603.3c-001
#[test]
fn a_modal_trigger_with_one_legal_mode_takes_it_unasked() {
    let mut game = setup_two_player_game();
    cast_and_resolve(&mut game, branching_beacon(), &[(ManaType::Green, 2)]);

    // `test_dp` answers nothing, so a question would panic.
    assert!(game.place_pending_triggers(&test_dp()).expect("the trigger is placed"));
    let trigger = *game.stack.last().expect("on the stack");
    assert_eq!(game.stack_entries[&trigger].chosen_modes, modes(&[2]));
    assert!(game.stack_entries[&trigger].chosen_targets.is_empty(), "the unchosen mode announces nothing");
    game.resolve_top_of_stack(&test_dp()).expect("the trigger resolves");
    assert_eq!(game.players[0].life_total, 23);
}

/// ATOM-603.3c-002's board: every mode targets an artifact or a creature, and
/// there is neither, so no mode can be chosen and the ability is removed.
// COVERS: ATOM-603.3c-002, ATOM-700.2b-001
#[test]
fn a_modal_trigger_with_no_legal_mode_is_removed() {
    let mut game = setup_two_player_game();
    cast_and_resolve(&mut game, forked_omen(), &[(ManaType::Black, 2)]);

    assert_eq!(game.pending_triggers.len(), 1, "it triggered");
    assert!(!game.place_pending_triggers(&test_dp()).expect("the placement runs"), "nothing was placed");
    assert!(game.stack.is_empty() && game.pending_triggers.is_empty());
}

/// The same trigger with a creature to destroy: its second mode is the one
/// legal set, and it is placed.
#[test]
fn a_modal_trigger_with_a_legal_mode_is_placed() {
    let mut game = setup_two_player_game();
    let giant = put_on_battlefield(&mut game, hill_giant(), 1);
    cast_and_resolve(&mut game, forked_omen(), &[(ManaType::Black, 2)]);

    assert!(game.place_pending_triggers(&test_dp()).expect("the trigger is placed"));
    let trigger = *game.stack.last().expect("on the stack");
    assert_eq!(game.stack_entries[&trigger].chosen_modes, modes(&[2]));
    game.resolve_top_of_stack(&test_dp()).expect("the trigger resolves");
    assert_eq!(game.get_object(giant).unwrap().zone, Zone::Graveyard);
}

/// CR 603.3c's other removal: "choose up to one" lets its controller choose
/// none, and an ability with no mode chosen is removed from the stack.
#[test]
fn a_modal_trigger_whose_controller_chooses_none_is_removed() {
    let mut game = setup_two_player_game();
    let lever = put_on_battlefield(&mut game, mode_lever(), 1);
    cast_and_resolve(&mut game, hesitant_omen(), &[(ManaType::White, 2)]);

    let answers = ModesOffered::new(ScriptedDecisionProvider::new());
    chooses_modes(&answers.inner, &[]);
    assert!(!game.place_pending_triggers(&answers).expect("the placement runs"));
    assert_eq!(offered(&game, &answers.offers()), vec![vec!["no modes", "mode 1", "mode 2"]]);
    assert!(game.stack.is_empty());
    assert_eq!(game.get_object(lever).unwrap().zone, Zone::Battlefield);
    assert_eq!(game.players[0].life_total, 20);
}

// ---------------------------------------------------------------------------
// CR 700.2a: a modal activation, and a mode that can't be chosen
// ---------------------------------------------------------------------------

/// CR 602.2b runs 601.2b–c: the modes first, then the chosen mode's target.
#[test]
fn a_modal_activation_announces_its_modes_before_its_targets() {
    let mut game = setup_two_player_game();
    let lever = put_on_battlefield(&mut game, mode_lever(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    put_on_battlefield(&mut game, hill_giant(), 1);
    game.players[0].mana_pool.add(ManaType::Green, 1);

    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[1]);
    names_targets(&answers, vec![ChoiceOption::Object(bears)]);
    let stop = ManaWindowStop::new(answers);
    game.activate_ability(0, lever, 0, &stop).expect("{1} and the tap pay it");
    let kinds: Vec<&str> = stop.inner().asked().iter().map(ChoiceKind::as_str).collect();
    assert_eq!(kinds, ["ChooseModes", "SelectRecipients"]);

    let ability = *game.stack.last().expect("on the stack");
    assert_eq!(game.stack_entries[&ability].chosen_modes, modes(&[1]));
    game.resolve_top_of_stack(&test_dp()).expect("the ability resolves");
    assert_eq!(mtgsim::oracle::characteristics::get_effective_power(&game, bears), Some(3));
    assert_eq!(game.players[0].life_total, 20, "the other mode did nothing");
}

/// ATOM-700.2a-001: a mode whose target has no legal choice is not offered.
/// Kindled Option's caster controls an artifact, so either mode or both may
/// be chosen; with no creature, only the second mode is a legal set.
// COVERS: ATOM-700.2a-001
#[test]
fn a_mode_whose_target_has_no_legal_choice_is_not_offered() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, mode_lever(), 0);
    fill_library(&mut game, 1, 3);
    let option = put_in_hand(&mut game, kindled_option(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 2);

    // The one legal set is announced unasked, and the player is its target.
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Player(1)]);
    let stop = ManaWindowStop::new(answers);
    game.cast_spell(0, option, &stop).expect("the second mode is legal");
    assert_eq!(game.stack_entries[&option].chosen_modes, modes(&[2]));
    assert!(!stop.inner().asked().iter().any(|kind| matches!(kind, ChoiceKind::ChooseModes { .. })));
}

// ---------------------------------------------------------------------------
// The count: a condition raises it, X sets it, and no mode is chosen twice
// ---------------------------------------------------------------------------

/// "If you control an artifact as you cast this spell, you may choose both
/// instead", read as the modes are chosen: one mode at a time without the
/// artifact, and either or both with it.
#[test]
fn a_count_raised_by_a_condition_reads_it_as_the_modes_are_chosen() {
    for with_an_artifact in [false, true] {
        let mut game = setup_two_player_game();
        if with_an_artifact {
            put_on_battlefield(&mut game, mode_lever(), 0);
        }
        put_on_battlefield(&mut game, hill_giant(), 1);
        let option = put_in_hand(&mut game, kindled_option(), 0);
        game.players[0].mana_pool.add(ManaType::Red, 2);

        // The giant is the one creature, so mode 1's target is not asked.
        let answers = ModesOffered::new(ScriptedDecisionProvider::new());
        chooses_modes(&answers.inner, &[1]);
        let stop = ManaWindowStop::new(answers);
        game.cast_spell(0, option, &stop).expect("castable");
        let expected: Vec<&str> = if with_an_artifact { vec!["mode 1", "mode 2", "modes 1, 2"] } else { vec!["mode 1", "mode 2"] };
        assert_eq!(offered(&game, &stop.inner().offers()), vec![expected]);
    }
}

/// "Choose X": X is announced first, then the sets of that many modes. A mode
/// is offered at most once because the fixture does not say it may repeat
/// (CR 700.2d); Doomsday Confluence, the one printed "choose X", does, and its
/// repeats are `mode-rules-and-costs`'.
// COVERS: ATOM-700.2d-001
#[test]
fn a_count_of_x_asks_x_then_the_sets_of_that_many() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 3);
    fill_library(&mut game, 1, 3);
    let tally = put_in_hand(&mut game, tally_of_choices(), 0);
    game.players[0].mana_pool.add(ManaType::Blue, 3);

    let answers = ModesOffered::new(ScriptedDecisionProvider::new());
    answers.inner.expect_number(choose_x(), 2);
    chooses_modes(&answers.inner, &[1, 3]);
    names_targets(&answers.inner, vec![ChoiceOption::Player(1)]);
    let stop = ManaWindowStop::new(answers);
    game.cast_spell(0, tally, &stop).expect("{2}{U} from three blue");
    let kinds: Vec<&str> = stop.inner().inner.asked().iter().map(ChoiceKind::as_str).collect();
    assert_eq!(kinds, ["ChooseXValue", "ChooseModes", "SelectRecipients"]);

    // CR 700.2d: two different modes, never one mode twice.
    let offers = stop.inner().offers();
    assert_eq!(offered(&game, &offers), vec![vec!["modes 1, 2", "modes 1, 3", "modes 2, 3"]]);
    assert_eq!(position_named(&offers[0], &game, &ChoiceOption::modes(&[1, 1]), &[]), None, "a mode twice is no choice");

    let hand = game.players[0].hand.len();
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.players[0].hand.len(), hand + 1, "mode 1 drew");
    assert_eq!(game.players[0].life_total, 20, "mode 2 was not chosen");
    assert_eq!(game.players[1].graveyard.len(), 2, "mode 3 milled");
}

/// A count of X past the modes printed has no set of that size, so the
/// announcement can't be completed and the cast is rewound (CR 601.2e).
#[test]
fn a_count_of_x_past_the_modes_rewinds_the_cast() {
    let mut game = setup_two_player_game();
    let tally = put_in_hand(&mut game, tally_of_choices(), 0);
    game.players[0].mana_pool.add(ManaType::Blue, 5);

    let answers = ScriptedDecisionProvider::new();
    answers.expect_number(choose_x(), 4);
    assert!(game.cast_spell(0, tally, &ManaWindowStop::new(answers)).is_err());
    assert_eq!(game.get_object(tally).unwrap().zone, Zone::Hand);
    assert!(game.stack.is_empty());
}

// ---------------------------------------------------------------------------
// Decision 3: the chosen modes resolve, each its own instance scope
// ---------------------------------------------------------------------------

/// A mode not chosen announces no target, so the creature its target would
/// have been is never asked about and is untouched. The chosen mode targets a
/// player here; Cryptic Command's untargeted pair is the atom's own board.
// COVERS-PARTIAL: ATOM-700.2c-001
#[test]
fn an_unchosen_modes_target_is_never_announced() {
    let mut game = setup_two_player_game();
    let giant = put_on_battlefield(&mut game, hill_giant(), 1);
    fill_library(&mut game, 1, 3);
    let option = put_in_hand(&mut game, kindled_option(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 2);

    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[2]);
    names_targets(&answers, vec![ChoiceOption::Player(1)]);
    game.cast_spell(0, option, &ManaWindowStop::new(answers)).expect("castable");
    let entry = &game.stack_entries[&option];
    assert_eq!(entry.chosen_targets.len(), 1, "the second mode's player alone");

    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(damage_on(&game, giant), 0);
    assert_eq!(game.players[1].graveyard.len(), 1, "drew a card, then discarded it");
}

/// A mode is its own instance scope: the second mode's "then discards a
/// card" names the mode's own first instance, which is the spell's second
/// when both modes are chosen. Read from the spell's first, it would name the
/// creature. The artifact that raised the count leaves before the spell
/// resolves, and both modes still do: the count was read as they were chosen.
#[test]
fn a_modes_back_reference_names_its_own_first_instance() {
    let mut game = setup_two_player_game();
    let lever = put_on_battlefield(&mut game, mode_lever(), 0);
    let giant = put_on_battlefield(&mut game, hill_giant(), 1);
    fill_library(&mut game, 1, 3);
    let option = put_in_hand(&mut game, kindled_option(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 2);

    // The giant is the one creature, so only the player is asked.
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[1, 2]);
    names_targets(&answers, vec![ChoiceOption::Player(1)]);
    game.cast_spell(0, option, &ManaWindowStop::new(answers)).expect("castable");
    assert_eq!(game.stack_entries[&option].chosen_modes, modes(&[1, 2]));
    let targets: Vec<Vec<ObjectOrPlayerId>> =
        game.stack_entries[&option].chosen_targets.iter().map(|instance| instance.as_resolved_targets().collect()).collect();
    assert_eq!(targets, vec![vec![ObjectOrPlayerId::Object(giant)], vec![ObjectOrPlayerId::Player(1)]], "mode 1's first");

    game.change_zone(lever, Zone::Exile, ZoneChangeAct::Put, &test_ctx()).expect("the artifact leaves");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(damage_on(&game, giant), 2);
    assert!(game.players[1].hand.is_empty(), "the card drawn was discarded");
    assert_eq!(game.players[1].graveyard.len(), 1);
}

/// CR 608.2b over modes: a chosen mode whose target went illegal does
/// nothing, and the spell does as much as it can (Prismari Command's second
/// ruling, on this shape).
#[test]
fn a_chosen_mode_whose_target_left_does_nothing_and_the_rest_resolves() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, mode_lever(), 0);
    let giant = put_on_battlefield(&mut game, hill_giant(), 1);
    fill_library(&mut game, 1, 3);
    let option = put_in_hand(&mut game, kindled_option(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 2);

    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[1, 2]);
    names_targets(&answers, vec![ChoiceOption::Player(1)]);
    game.cast_spell(0, option, &ManaWindowStop::new(answers)).expect("castable");
    game.change_zone(giant, Zone::Exile, ZoneChangeAct::Put, &test_ctx()).expect("the giant leaves");

    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.players[1].graveyard.len(), 1, "the second mode still drew and discarded");
}

/// "Another target" inside a mode excludes the mode's own earlier target, at
/// CR 608.2b's re-check as at the announcement. The first mode targets the
/// second creature; the second mode taps the first creature and then
/// "another target creature", the second. Read against the spell's first
/// instance instead of the mode's, the second creature would be excluded at
/// the re-check and left untapped.
#[test]
fn another_target_inside_a_mode_excludes_only_that_modes_targets() {
    let mut game = setup_two_player_game();
    let first = put_on_battlefield(&mut game, hill_giant(), 1);
    let second = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let another = creature().and(ObjectFilter::OtherThanInstance(0));
    let card = invented_instant(
        "Choose one or both —\n• Target creature gets +1/+1 until end of turn.\n• Tap target creature. Tap another target creature.",
        choose(
            ModalCount::Range { fewest: 1, most: 2 },
            [
                gets(target_creature(), 1, 1).until_end_of_turn(),
                Effect::Sequence(vec![tap(target_permanent(creature())), tap(target_permanent(another))]),
            ],
        ),
    );
    let spell = put_in_hand(&mut game, card, 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    // The second mode's "another" has one creature left, so it is not asked.
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[1, 2]);
    names_targets(&answers, vec![ChoiceOption::Object(second)]);
    names_targets(&answers, vec![ChoiceOption::Object(first)]);
    game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable");
    assert_eq!(game.stack_entries[&spell].mode_scope_starts(), Ok(vec![0, 1]));

    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert!(game.battlefield[&first].tapped);
    assert!(game.battlefield[&second].tapped, "its other target, the mode's own second");
}

/// "If you chose [that mode]" reads the record the announcement wrote.
#[test]
fn a_condition_on_a_chosen_mode_reads_the_modes_chosen() {
    for (chosen, drew) in [(&[1u8, 2][..], 1), (&[2u8][..], 0)] {
        let mut game = setup_two_player_game();
        fill_library(&mut game, 0, 3);
        let first = ModeIndex::printed(1);
        let card = invented_instant(
            "Choose one or both —\n• You gain 2 life.\n• If you chose the first mode, draw a card.",
            choose(
                ModalCount::Range { fewest: 1, most: 2 },
                [gain_life(you(), 2), Effect::Conditional(Condition::ModeChosen(first), Box::new(draw_cards(you(), 1)))],
            ),
        );
        let spell = put_in_hand(&mut game, card, 0);
        game.players[0].mana_pool.add(ManaType::Red, 1);
        let answers = ScriptedDecisionProvider::new();
        chooses_modes(&answers, chosen);
        game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable");
        let hand = game.players[0].hand.len();
        game.resolve_top_of_stack(&test_dp()).expect("it resolves");
        assert_eq!(game.players[0].hand.len(), hand + drew, "modes {chosen:?}");
    }
}

/// A modal node anywhere but its ability's root was announced for by
/// nothing, so it is refused rather than resolved as a choice of none.
#[test]
fn a_modal_node_below_the_root_is_refused_at_resolution() {
    let mut game = setup_two_player_game();
    let source = put_in_hand(&mut game, hill_giant(), 0);
    let effect = Effect::Sequence(vec![choose_one([gain_life(you(), 1), gain_life(you(), 2)])]);
    let ctx = ResolutionContext::untargeted(source, 0, GeneratedBy::Spell);
    let refused = game.resolve_effect_with_announced_targets(&effect, &[], &[], &ctx, &test_dp());
    assert!(refused.is_err_and(|why| why.contains("no modes announced")));
    assert_eq!(game.players[0].life_total, 20);
}

/// A modal activation every mode of which targets is not offered on a board
/// with nothing to target (CR 700.2a, 602.2b).
#[test]
fn a_modal_activation_with_no_legal_mode_is_not_offered() {
    let mut game = setup_two_player_game();
    let text = "{T}: Choose one —\n• Destroy target creature.\n• Destroy target enchantment.";
    let card = CardDataBuilder::new("Two-Way Lever")
        .card_type(CardType::Artifact)
        .rules_text(text)
        .ability(activated_ability(
            text,
            vec![Cost::TapSelf],
            choose_one([destroy(target_creature()), destroy(target_permanent(enchantment()))]),
        ))
        .build();
    let lever = put_on_battlefield(&mut game, card, 0);
    let ability = |game: &GameState| get_effective_abilities(game, lever)[0].clone();
    assert_eq!(can_activate(&game, 0, lever, &ability(&game)), Err(CannotActivate::NoLegalModes));

    put_on_battlefield(&mut game, hill_giant(), 1);
    assert_eq!(can_activate(&game, 0, lever, &ability(&game)), Ok(()));
}

/// An instant of `text` doing `effect` for {R}, a test's own.
fn invented_instant(text: &'static str, effect: Effect) -> std::sync::Arc<mtgsim::objects::card_data::CardData> {
    CardDataBuilder::new("Invented Modal Instant")
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, effect))
        .build()
}

// ---------------------------------------------------------------------------
// The option's words
// ---------------------------------------------------------------------------

/// What the log and a client write for a choice of modes: by printed place,
/// never by where the set sat in the offer.
#[test]
fn a_choice_of_modes_is_written_by_its_printed_places() {
    let game = setup_two_player_game();
    assert_eq!(ChoiceOption::modes(&[1, 3]).as_logged(&game), "modes 1, 3");
    assert_eq!(ChoiceOption::modes(&[2]).as_logged(&game), "mode 2");
    assert_eq!(ChoiceOption::modes(&[]).as_logged(&game), "no modes");
    assert_eq!(option_label(&game, &ChoiceOption::modes(&[1, 3])), "Modes 1 and 3");
    assert_eq!(option_label(&game, &ChoiceOption::modes(&[])), "No modes");
}

// ---------------------------------------------------------------------------
// The cards, cast from hand
// ---------------------------------------------------------------------------

/// Abrade with neither a creature nor an artifact to target: no mode can be
/// chosen, so it is not offered, and a cast is refused before anything is
/// paid (CR 700.2a). With an artifact, its second mode is the one legal set.
#[test]
fn abrade_is_offered_only_while_one_of_its_modes_can_be_chosen() {
    let mut game = setup_two_player_game();
    let abrade = put_in_hand(&mut game, abrade(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 2);
    assert_eq!(can_cast(&game, 0, abrade), Err(CannotCast::NoLegalModes));
    assert!(game.cast_spell(0, abrade, &ManaWindowStop::new(test_dp())).is_err());
    assert_eq!(game.get_object(abrade).unwrap().zone, Zone::Hand);
    assert_eq!(game.players[0].mana_pool.unspent(ManaType::Red), 2, "nothing was paid");

    let lever = put_on_battlefield(&mut game, mode_lever(), 1);
    assert_eq!(can_cast(&game, 0, abrade), Ok(()));
    game.cast_spell(0, abrade, &ManaWindowStop::new(test_dp())).expect("the second mode, unasked");
    assert_eq!(game.stack_entries[&abrade].chosen_modes, modes(&[2]));
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.get_object(lever).unwrap().zone, Zone::Graveyard);
}

/// Cryptic Command with no spell on the stack: its first mode can't be
/// chosen, so each pair of the other three is offered, two different modes
/// each, and a pair that targets nothing announces nothing though a creature
/// it could have bounced is there (ATOM-700.2c-001's board).
// RULING: Cryptic Command #1 - "You choose both modes as you cast Cryptic Command. You must choose two different modes."
// COVERS: ATOM-700.2c-001
#[test]
fn cryptic_command_offers_each_pair_of_different_modes() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 3);
    let giant = put_on_battlefield(&mut game, hill_giant(), 1);
    let command = put_in_hand(&mut game, cryptic_command(), 0);
    game.players[0].mana_pool.add(ManaType::Blue, 4);

    let answers = ModesOffered::new(ScriptedDecisionProvider::new());
    chooses_modes(&answers.inner, &[3, 4]);
    let stop = ManaWindowStop::new(answers);
    game.cast_spell(0, command, &stop).expect("castable");
    assert_eq!(offered(&game, &stop.inner().offers()), vec![vec!["modes 2, 3", "modes 2, 4", "modes 3, 4"]]);
    assert!(game.stack_entries[&command].chosen_targets.is_empty());

    let hand = game.players[0].hand.len();
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert!(game.battlefield[&giant].tapped, "the opponent's creatures are tapped");
    assert_eq!(game.players[0].hand.len(), hand + 1, "and a card drawn");
}

/// Its targets are its chosen modes': with the second and fourth modes, the
/// permanent is its one target, and once that is illegal the spell does not
/// resolve, so no card is drawn either (CR 608.2b).
// RULING: Cryptic Command #2 - "Look at both chosen modes to determine how many targets Cryptic Command has, if any. If it has at least one target, and all its targets are illegal when it tries to resolve, then it won't resolve and none of its effects will happen."
#[test]
fn cryptic_command_whose_one_target_left_draws_nothing() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 3);
    let giant = put_on_battlefield(&mut game, hill_giant(), 1);
    let command = put_in_hand(&mut game, cryptic_command(), 0);
    game.players[0].mana_pool.add(ManaType::Blue, 4);

    // The giant is the one permanent, so its target is not asked.
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[2, 4]);
    game.cast_spell(0, command, &ManaWindowStop::new(answers)).expect("castable");
    game.change_zone(giant, Zone::Exile, ZoneChangeAct::Put, &test_ctx()).expect("the giant leaves");

    let hand = game.players[0].hand.len();
    game.resolve_top_of_stack(&test_dp()).expect("it fails to resolve");
    assert_eq!(game.players[0].hand.len(), hand, "no card drawn");
    assert_eq!(game.get_object(command).unwrap().zone, Zone::Graveyard);
}

/// Casualties of War over an artifact, a creature and a land, with no
/// enchantment or planeswalker: every nonempty set of the first, second and
/// fourth modes, none twice, smallest first.
// RULING: Casualties of War #2 - "You can choose any number of the modes for Casualties of War, but you can't choose a mode more than once."
#[test]
fn casualties_of_war_offers_every_set_of_its_legal_modes() {
    let (game, offers) = cast_casualties_of_war(&[1, 2, 4]);
    let expected = ["mode 1", "mode 2", "mode 4", "modes 1, 2", "modes 1, 4", "modes 2, 4", "modes 1, 2, 4"];
    assert_eq!(offered(&game, &offers), vec![expected.to_vec()]);
}

/// The three are destroyed in the order the modes are printed, and the
/// creature's dies trigger waits for the spell to finish (CR 608.2m, 603.3).
// RULING: Casualties of War #1 - "Each target permanent is destroyed in the order specified. Abilities that trigger when one is destroyed won't be put onto the stack until Casualties of War has finished resolving."
#[test]
fn casualties_of_war_destroys_in_printed_order_and_its_triggers_wait() {
    let (mut game, _) = cast_casualties_of_war(&[1, 2, 4]);
    game.record_events();
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    let destroyed: Vec<String> = mtgsim::ui::display::format_event_log(&game)
        .into_iter()
        .filter(|line| line.contains("[Destroyed]"))
        .map(|line| line.split(" (").next().unwrap_or_default().trim_start_matches("ZoneChange: ").to_string())
        .collect();
    assert_eq!(destroyed, ["Mode Lever", "Paladin of Atonement", "Forest"]);
    assert!(game.stack.is_empty(), "nothing went on the stack while it resolved");
    assert_eq!(game.pending_triggers.len(), 1, "the paladin's dies trigger waits to be placed");
    assert_eq!(game.players[1].life_total, 20);
}

/// Casualties of War cast by player 0 at an opponent's artifact, creature
/// (Paladin of Atonement, which triggers as it dies) and land, the modes at
/// `places` chosen: the game, and the offer it was shown.
fn cast_casualties_of_war(places: &[u8]) -> (GameState, Vec<Vec<ChoiceOption>>) {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, mode_lever(), 1);
    put_on_battlefield(&mut game, paladin_of_atonement(), 1);
    put_on_battlefield(&mut game, forest(), 1);
    let casualties = put_in_hand(&mut game, casualties_of_war(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 3);
    game.players[0].mana_pool.add(ManaType::Green, 3);

    // One of each kind of permanent, so every target is announced unasked.
    let answers = ModesOffered::new(ScriptedDecisionProvider::new());
    chooses_modes(&answers.inner, places);
    let stop = ManaWindowStop::new(answers);
    game.cast_spell(0, casualties, &stop).expect("castable");
    assert_eq!(game.stack_entries[&casualties].chosen_modes, modes(places));
    let offers = stop.inner().offers();
    (game, offers)
}
