//! Phase RE-4 — tokens.
//!
//! CR 614.16's token half, CR 111.5, CR 616.1g and CR 111.4, against the four
//! printed cards in `cards::phase_re_cards` and `Primitive::CreateToken`'s new
//! shape: one proposal, whose performer proposes every token's entry as one
//! batch.
//!
//! **Every board here is about which of two events an effect meets** — the
//! creation (the outer event, Parallel Lives') or an entry (a contained one,
//! Hallowed Moonlight's and Master Biomancer's) — and about what the log says
//! a token that never entered did. Item 52's line is asserted *absent*: a
//! token created in exile has no `ZoneChange { from: Battlefield }` for a
//! leaves-the-battlefield trigger to read.

use std::collections::HashSet;
use std::sync::Arc;

use mtgsim::cards::phase_rb_cards::kalitas_traitor_of_ghet;
use mtgsim::cards::phase_rc_cards::{master_biomancer, root_maze, thunder_thrash_elder};
use mtgsim::cards::phase_re_cards::{
    alms_collector, bard_king_of_dale, divine_visitation, goblin_token,
    hallowed_moonlight, hordeling_outburst, parallel_lives, raise_the_alarm, soldier_token,
    thought_reflection,
};
use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeCause};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::events::event::{BatchId, GameEvent};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{
    get_effective_colors, get_effective_name, get_effective_power, get_effective_subtypes,
    get_effective_toughness, get_effective_types, has_supertype,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    fill_library, place_bare, put_in_graveyard, put_in_hand, put_on_battlefield,
    setup_two_player_game, static_ability, test_ctx, test_dp, vanilla_creature,
    RecordingDecisionProvider,
};
use mtgsim::types::card_types::{
    CardType, CreatureType, EnchantmentType, Subtype, Supertype,
};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    ObjectSet, AmountExpr, CounterType, Effect, EffectRecipient, ObjectFilter, PlayerRef,
    PlayerSet, Primitive, SelectionFilter, TokenDef,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::replacement::{
    AmountRewrite, EventPattern, GameActionTemplate, ReplacementDef, Rewrite, TemplateAmount,
    TokenKind, TokenSubstitution,
};
use mtgsim::types::restriction::{Restriction, RestrictionDef};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A nameless card with no abilities, to be the source of a fixture
/// resolution.
fn fixture_object() -> Arc<CardData> {
    CardDataBuilder::new("Fixture").build()
}

/// Resolve an effect for `player`, the way a spell would. Puts one fixture
/// card into `player`'s hand as the source, which every `objects.len()`
/// assertion below counts.
fn resolve_for(game: &mut GameState, player: PlayerId, effect: &Effect, dp: &dyn DecisionProvider) {
    let source = put_in_hand(game, fixture_object(), player);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: player,
        targets: vec![],
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(effect, &ctx, dp).expect("resolving");
}

/// Resolve a registered instant or sorcery's spell ability for `player`, as
/// its text is written.
fn resolve_card(game: &mut GameState, player: PlayerId, card: Arc<CardData>, dp: &dyn DecisionProvider) {
    let effect = card.abilities[0].effect.clone();
    resolve_for(game, player, &effect, dp);
}

/// "Create `n` of these", as a resolving spell says it.
fn create(game: &mut GameState, player: PlayerId, def: TokenDef, n: u64, dp: &dyn DecisionProvider) {
    let effect = Effect::Atom(
        Primitive::CreateToken(def, AmountExpr::Fixed(n)),
        EffectRecipient::Controller,
    );
    resolve_for(game, player, &effect, dp);
}

/// Every token on the battlefield, in timestamp order.
fn tokens(game: &GameState) -> Vec<ObjectId> {
    game.battlefield_ids_ordered()
        .into_iter()
        .filter(|id| game.objects.get(id).is_some_and(|o| o.is_token))
        .collect()
}

/// Every `TokenCreated` since `start`, as `(token, zone)`.
fn creations(game: &GameState, start: usize) -> Vec<(ObjectId, Zone)> {
    game.events
        .records_from(start)
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::TokenCreated { object_id, zone, .. } => Some((*object_id, *zone)),
            _ => None,
        })
        .collect()
}

/// Every `PermanentEnteredBattlefield` since `start`.
fn entered(game: &GameState, start: usize) -> Vec<ObjectId> {
    game.events
        .records_from(start)
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::PermanentEnteredBattlefield { object_id, .. } => Some(*object_id),
            _ => None,
        })
        .collect()
}

/// Every `TokenCeasedToExist` since `start`.
fn ceased(game: &GameState, start: usize) -> Vec<ObjectId> {
    game.events
        .records_from(start)
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::TokenCeasedToExist { object_id } => Some(*object_id),
            _ => None,
        })
        .collect()
}

/// Every `ZoneChange` of `id` since `start`, as `(from, to)`.
fn moves_of(game: &GameState, start: usize, id: ObjectId) -> Vec<(Zone, Zone)> {
    game.events
        .records_from(start)
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::ZoneChange { object_id, from, to, .. } if *object_id == id => {
                Some((*from, *to))
            }
            _ => None,
        })
        .collect()
}

/// The batch id of every token line since `start` — creations and entries.
fn token_batches(game: &GameState, start: usize) -> Vec<Option<BatchId>> {
    game.events
        .records_from(start)
        .iter()
        .filter(|r| {
            matches!(
                r.event,
                GameEvent::TokenCreated { .. } | GameEvent::PermanentEnteredBattlefield { .. }
            )
        })
        .map(|r| r.batch())
        .collect()
}

fn counters(game: &GameState, id: ObjectId, kind: CounterType) -> u32 {
    game.battlefield.get(&id).map(|e| e.counter_count(kind)).unwrap_or(0)
}

/// A castable 2/2 for `{G}`: one colored symbol and no generic, so
/// `cast_spell` asks nothing about mana.
fn castable_bear() -> Arc<CardData> {
    CardDataBuilder::new("Grizzly Bears")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Bear))
        .power_toughness(2, 2)
        .build()
}

/// An enchantment with one static ability, to hold a fixture effect.
fn enchantment_with(name: &str, ability: mtgsim::objects::card_data::AbilityDef) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Enchantment)
        .ability(ability)
        .build()
}

// ---------------------------------------------------------------------------
// A plural creation is one event, and its entries are one batch
// ---------------------------------------------------------------------------

/// CR 111.2's two sentences, per token: "the player who creates a token is
/// its owner" — `TokenCreated` — and "the token enters the battlefield under
/// that player's control" — the entry. CR 111.4 names them.
#[test]
fn raise_the_alarm_creates_two_soldiers_named_by_their_subtype() {
    let mut game = setup_two_player_game();
    let start = game.events.records().len();

    resolve_card(&mut game, 0, raise_the_alarm(), &test_dp());

    let soldiers = tokens(&game);
    assert_eq!(soldiers.len(), 2);
    for id in &soldiers {
        assert_eq!(
            get_effective_name(&game, *id),
            "Soldier Token",
            "CR 111.4: the card gives no name, so the name is the subtypes plus the word Token"
        );
        assert!(get_effective_types(&game, *id).contains(&CardType::Creature));
        assert!(get_effective_subtypes(&game, *id).contains(&Subtype::Creature(CreatureType::Soldier)));
        assert_eq!(get_effective_colors(&game, *id), HashSet::from([Color::White]));
        assert_eq!(get_effective_power(&game, *id), Some(1));
        assert_eq!(get_effective_toughness(&game, *id), Some(1));
        let obj = game.get_object(*id).unwrap();
        assert!(obj.is_token, "CR 111.1");
        assert_eq!(obj.owner, 0, "CR 111.2: the player who creates a token is its owner");
    }
    assert_eq!(
        creations(&game, start),
        soldiers.iter().map(|id| (*id, Zone::Battlefield)).collect::<Vec<_>>(),
        "each token is announced as created on the battlefield"
    );
    assert_eq!(entered(&game, start), soldiers, "and then as having entered");
}

/// "Create three tokens" is one event (CR 111, 614.16), and the three entries
/// it contains are part of it: one batch id across every token line.
///
/// The order is the creation's — CR 613.7m's APNAP choice is not asked for a
/// homogeneous batch (`replacement-architecture.md` §9, RE decision 3) — so
/// the timestamp order `tokens` reads is the log order `creations` reads.
#[test]
fn a_plural_creation_is_one_event_and_its_entries_join_it() {
    let mut game = setup_two_player_game();
    let start = game.events.records().len();

    resolve_card(&mut game, 0, hordeling_outburst(), &test_dp());

    let goblins = tokens(&game);
    assert_eq!(goblins.len(), 3);
    assert!(goblins.iter().all(|id| get_effective_name(&game, *id) == "Goblin Token"));
    let batches: HashSet<Option<BatchId>> = token_batches(&game, start).into_iter().collect();
    assert_eq!(batches.len(), 1, "six token lines, one batch: the creation's");
    assert!(batches.iter().all(|b| b.is_some()));
    assert_eq!(
        creations(&game, start).iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        goblins,
        "the batch order is the creation's, and so are the timestamps"
    );
}

// ---------------------------------------------------------------------------
// CR 614.16 — Parallel Lives replaces the creation, not the entries
// ---------------------------------------------------------------------------

#[test]
fn parallel_lives_doubles_a_plural_creation_without_asking() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, parallel_lives(), 0);
    let dp = RecordingDecisionProvider::picking(0);

    resolve_card(&mut game, 0, raise_the_alarm(), &dp);

    assert_eq!(tokens(&game).len(), 4, "twice that many");
    assert_eq!(dp.prompts(), 0, "one candidate is no choice (CR 616.1)");
}

/// Parallel Lives' first ruling: *"If you control two Parallel Lives, then the
/// number of tokens created is four times the original number."* The Furnace
/// pair's third kind — two commuting multipliers on one event — so CR 616.1's
/// order prompt has one outcome and is not asked
/// (`pipeline::ordering_cannot_change_outcome`).
#[test]
fn two_parallel_lives_create_four_times_as_many_and_ask_nothing() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, parallel_lives(), 0);
    put_on_battlefield(&mut game, parallel_lives(), 0);
    let dp = RecordingDecisionProvider::picking(0);

    resolve_card(&mut game, 0, raise_the_alarm(), &dp);

    assert_eq!(tokens(&game).len(), 8, "four times the original number");
    assert_eq!(dp.prompts(), 0, "two multipliers commute, so the order is not a choice");
}

#[test]
fn parallel_lives_leaves_an_opponents_creation_alone() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, parallel_lives(), 0);

    resolve_card(&mut game, 1, raise_the_alarm(), &test_dp());

    let soldiers = tokens(&game);
    assert_eq!(soldiers.len(), 2, "\"under your control\" is CR 109.5's you");
    assert!(soldiers.iter().all(|id| game.get_object(*id).unwrap().owner == 1));
}

/// Parallel Lives' second ruling: *"Everything that is specified by the effect
/// creating the original token or tokens will also be true about the
/// additional token or tokens."* Structurally true of a repeated def, asserted
/// on every token: a doubled Hordeling Outburst is six Goblins, and no two of
/// them differ in anything the effect said.
#[test]
fn the_extra_tokens_are_the_same_tokens() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, parallel_lives(), 0);

    resolve_card(&mut game, 0, hordeling_outburst(), &test_dp());

    let goblins = tokens(&game);
    assert_eq!(goblins.len(), 6);
    let described = |id: ObjectId| {
        (
            get_effective_name(&game, id),
            get_effective_colors(&game, id),
            get_effective_types(&game, id),
            get_effective_subtypes(&game, id),
            get_effective_power(&game, id),
            get_effective_toughness(&game, id),
            game.get_object(id).unwrap().owner,
        )
    };
    let first = described(goblins[0]);
    assert_eq!(first.0, "Goblin Token");
    assert_eq!(first.1, HashSet::from([Color::Red]));
    for id in &goblins[1..] {
        assert_eq!(described(*id), first, "the extra tokens are the tokens the effect described");
    }
}

// COVERS-PARTIAL: ATOM-614.16-001
//
// The atom's board is a death replaced by "instead create a 1/1 Spirit" under
// Doubling Season. Two things differ, and neither is the rule: the doubler is
// Parallel Lives, whose text is Doubling Season's token half word for word
// (the Season is registered in RE-5, when its counter half has an event to
// watch); and Kalitas creates its token as the replacement's CR 615.5 rider
// rather than as the substitute itself — there is no
// `GameActionTemplate::CreateTokens`, so "instead create a token" is not yet
// a rewrite this engine can write. What the rule says — token replacements
// "also apply if another replacement or prevention effect does so" — is what
// this proves: the token a replacement effect makes is doubled.
#[test]
fn parallel_lives_doubles_the_zombie_a_replacement_effect_makes() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, kalitas_traitor_of_ghet(), 0);
    put_on_battlefield(&mut game, parallel_lives(), 0);
    let victim = place_bare(&mut game, vanilla_creature(2, 2, &[]), 1);

    game.change_zone(victim, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx())
        .unwrap();

    assert_eq!(game.get_object(victim).unwrap().zone, Zone::Exile);
    let zombies = tokens(&game);
    assert_eq!(
        zombies.len(),
        2,
        "CR 614.16: the doubler applies to a token another replacement effect creates"
    );
    assert!(zombies.iter().all(|id| get_effective_name(&game, *id) == "Zombie Token"));
}

// ---------------------------------------------------------------------------
// CR 614.12 — every entry in the batch is decided against the board none of
// them has entered
// ---------------------------------------------------------------------------

/// Master Biomancer on the board, two Soldiers created: each entry reads the
/// Biomancer's power off the real board, and each gets its counters.
#[test]
fn two_soldiers_under_master_biomancer_each_get_its_counters() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, master_biomancer(), 0);

    resolve_card(&mut game, 0, raise_the_alarm(), &test_dp());

    let soldiers = tokens(&game);
    assert_eq!(soldiers.len(), 2);
    for id in &soldiers {
        assert_eq!(counters(&game, *id, CounterType::PlusOnePlusOne), 2, "Biomancer's power is 2");
    }
}

/// RC-5's `test_two_biomancers_entering_together_give_each_other_nothing`,
/// reached from a producer rather than a hand-built batch — which is what
/// `codebase-state.md` item 46 was waiting for. The tokens carry Master
/// Biomancer's ability (the first use of `TokenDef::abilities`), enter as one
/// batch, and each is decided against a board the other is not on (CR 614.12,
/// §5b), so neither gets the other's counters.
///
/// The control is the second half: two Soldiers created afterwards get both
/// Biomancer tokens' counters, which is the tokens' abilities functioning on
/// the battlefield — the field the def wrote is a field the layer walk reads.
#[test]
fn two_biomancer_tokens_entering_together_give_each_other_nothing() {
    let mut game = setup_two_player_game();
    let biomancer_token = TokenDef {
        name: Some("Biomancer Token".to_string()),
        colors: vec![Color::Green, Color::Blue],
        types: vec![CardType::Creature],
        subtypes: vec![
            Subtype::Creature(CreatureType::Elf),
            Subtype::Creature(CreatureType::Wizard),
        ],
        supertypes: Vec::new(),
        power: Some(2),
        toughness: Some(4),
        keyword_flags: Vec::new(),
        abilities: master_biomancer().abilities.clone(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
    };

    create(&mut game, 0, biomancer_token, 2, &test_dp());

    let biomancers = tokens(&game);
    assert_eq!(biomancers.len(), 2);
    for id in &biomancers {
        assert_eq!(
            counters(&game, *id, CounterType::PlusOnePlusOne),
            0,
            "CR 614.12 / §5b: neither token was on the battlefield when the other's entry was decided"
        );
    }

    resolve_card(&mut game, 0, raise_the_alarm(), &test_dp());

    let soldiers: Vec<ObjectId> =
        tokens(&game).into_iter().filter(|id| !biomancers.contains(id)).collect();
    assert_eq!(soldiers.len(), 2);
    for id in &soldiers {
        assert_eq!(
            counters(&game, *id, CounterType::PlusOnePlusOne),
            4,
            "two Biomancer tokens of power 2 on the board, and the def's abilities function"
        );
    }
}

// ---------------------------------------------------------------------------
// CR 616.1g — the creation is decided first, then each entry, once per token
// ---------------------------------------------------------------------------

/// Parallel Lives on the creation, then Master Biomancer and Hallowed
/// Moonlight on each entry. The creation's loop has one candidate and asks
/// nothing; each of the four entries then has two — counters or exile — and
/// **nothing is asked there either**: the exile's substitute carries no mods,
/// so it is the same event whichever applied first, and a token that ceases
/// to exist in exile has no counters to have had. That is the fifth shape of
/// `pipeline::ordering_cannot_change_outcome`, asked for at RE-4's review
/// (`plans/handoffs/re-4-review.md`, R15). All four are created in exile.
#[test]
fn hallowed_moonlight_beside_master_biomancer_asks_nothing_because_the_exile_wins_either_way() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 3);
    put_on_battlefield(&mut game, parallel_lives(), 0);
    put_on_battlefield(&mut game, master_biomancer(), 0);
    resolve_card(&mut game, 0, hallowed_moonlight(), &test_dp());
    let dp = RecordingDecisionProvider::picking(0);
    let start = game.events.records().len();

    resolve_card(&mut game, 0, raise_the_alarm(), &dp);

    assert_eq!(dp.prompts(), 0, "one outcome per token, so no question — and none about the creation");
    let created = creations(&game, start);
    assert_eq!(created.len(), 4, "Parallel Lives applied at the creation");
    assert!(created.iter().all(|(_, zone)| *zone == Zone::Exile));
    assert!(tokens(&game).is_empty());
}

/// The per-entry loop, on a board where it is observable: two tokens carrying
/// Thunder-Thrash Elder's devour, created together, with two Bears on the
/// board. Each token's entry is decided in its own CR 616.1 loop — so each
/// is asked what it devours — and CR 614.13a's second clause, "nor any other
/// object entering the battlefield at the same time", keeps each token off
/// the other's list. A provider that takes everything offered is the one
/// that would sacrifice the sibling if it were there (RC-5's lesson); both
/// tokens survive, so it was not.
///
/// RC-5 proved this clause at the `execute_actions` boundary with a
/// hand-built batch, and `codebase-state.md` item 46 said a producer would
/// make it reachable. This is that producer.
#[test]
fn two_devour_tokens_created_together_are_each_asked_and_never_offered_each_other() {
    let mut game = setup_two_player_game();
    let bear_a = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let bear_b = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let devourer = TokenDef {
        name: Some("Devourer Token".to_string()),
        colors: vec![Color::Red],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(CreatureType::Warrior)],
        supertypes: Vec::new(),
        power: Some(3),
        toughness: Some(3),
        keyword_flags: Vec::new(),
        abilities: thunder_thrash_elder().abilities.clone(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
    };
    let dp = RecordingDecisionProvider::picking_all();

    create(&mut game, 0, devourer, 2, &dp);

    let devourers = tokens(&game);
    assert_eq!(devourers.len(), 2, "neither token was offered as the other's meal");
    assert!(dp.kinds().iter().any(|k| k.starts_with("ChooseAuxiliaryZoneChange")), "{:?}", dp.kinds());
    assert!(!game.battlefield.contains_key(&bear_a) && !game.battlefield.contains_key(&bear_b));
    let counters_on: Vec<u32> =
        devourers.iter().map(|id| counters(&game, *id, CounterType::PlusOnePlusOne)).collect();
    assert_eq!(
        counters_on,
        vec![6, 0],
        "the first token devoured both Bears (devour 3 each) and the second found none: \
         two loops, in batch order, each against the board the last one left"
    );
}

// ---------------------------------------------------------------------------
// Item 52 — a token exiled instead was created in exile
// ---------------------------------------------------------------------------

/// Hallowed Moonlight's first ruling: *"if a creature token would be put onto
/// the battlefield, it's put into exile instead and then ceases to exist."*
///
/// The log holds `TokenCreated { Exile }` and CR 704.5d's `TokenCeasedToExist`,
/// and **no `ZoneChange` for the token at all** — the `from: Battlefield` line
/// Dour Port-Mage and Aang would have read is the line this asserts absent.
/// The Moonlight is P0's and the tokens P1's: "a creature" names no controller.
#[test]
fn hallowed_moonlight_creates_the_token_in_exile_and_it_ceases_to_exist() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 3);
    let drawn_before = game.events.events().filter(|e| matches!(e, GameEvent::CardDrawn { .. })).count();
    resolve_card(&mut game, 0, hallowed_moonlight(), &test_dp());
    let drawn_after = game.events.events().filter(|e| matches!(e, GameEvent::CardDrawn { .. })).count();
    assert_eq!(drawn_after, drawn_before + 1, "Draw a card");
    let start = game.events.records().len();
    let objects_before = game.objects.len();

    resolve_card(&mut game, 1, raise_the_alarm(), &test_dp());

    let created = creations(&game, start);
    assert_eq!(created.len(), 2);
    for (id, zone) in &created {
        assert_eq!(*zone, Zone::Exile, "created in exile, from nowhere");
        assert_eq!(game.get_object(*id).unwrap().zone, Zone::Exile);
        assert!(game.exile.contains(id));
        assert!(!game.battlefield.contains_key(id));
        assert!(
            moves_of(&game, start, *id).is_empty(),
            "a token that was never on the battlefield did not leave it: no ZoneChange at all"
        );
    }
    assert!(entered(&game, start).is_empty(), "it never entered");

    // CR 704.5d takes it from there.
    game.check_state_based_actions(&test_dp()).unwrap();
    let gone = ceased(&game, start);
    assert_eq!(gone.len(), 2);
    assert_eq!(gone, created.iter().map(|(id, _)| *id).collect::<Vec<_>>());
    assert!(game.exile.is_empty());
    assert_eq!(
        game.objects.len(),
        objects_before + 1,
        "the two tokens are gone and the fixture spell is what was added"
    );
}

/// Hallowed Moonlight's second ruling: *"won't affect any creature that was
/// cast, no matter which zone it was cast from and whether or not its mana
/// cost was paid."*
///
/// Cast from the hand here, which is the only zone the gate admits; "no
/// matter which zone" owes a second board — a creature cast from a graveyard
/// or from exile under the Moonlight — to the PR that opens `backlog.md`
/// §2.3, and §2.3 says so.
#[test]
fn hallowed_moonlight_does_not_affect_a_creature_that_was_cast() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 3);
    resolve_card(&mut game, 0, hallowed_moonlight(), &test_dp());
    let bear = put_in_hand(&mut game, castable_bear(), 0);
    game.players[0].mana_pool.add(ManaType::Green, 1);

    game.cast_spell(0, bear, &test_dp()).expect("it is castable");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    assert!(game.battlefield.contains_key(&bear), "a creature that was cast enters");
    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Battlefield);
}

/// A *card* that was not cast still takes RC-4b's shape under the same row:
/// one move from where it is, `Graveyard → Exile`, and no creation — a card is
/// created nowhere. The token arm did not take the card arm with it.
#[test]
fn hallowed_moonlight_exiles_a_returned_card_in_one_move() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 3);
    resolve_card(&mut game, 0, hallowed_moonlight(), &test_dp());
    let start = game.events.records().len();
    let bear = put_in_graveyard(&mut game, vanilla_creature(2, 2, &[]), 1);

    game.change_zone(bear, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx())
        .expect("the entry is replaced, and that is not an error");

    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Exile);
    assert_eq!(moves_of(&game, start, bear), vec![(Zone::Graveyard, Zone::Exile)]);
    assert!(creations(&game, start).is_empty(), "a card is not created anywhere");
}

// ---------------------------------------------------------------------------
// CR 111.5 — a token that can't enter is not created
// ---------------------------------------------------------------------------

// COVERS: ATOM-111.5-002
//
// "Creatures can't enter the battlefield" is active; "create two 1/1 creature
// tokens" resolves. CR 111.5: the token is not created — no object, no
// announcement — and CR 614.17d is the door: the "can't" refuses each entry
// ahead of the pipeline, and the performer un-creates what it had made.
#[test]
fn a_token_that_cannot_enter_is_not_created() {
    let mut game = setup_two_player_game();
    let no_creatures = static_ability(Effect::Restriction(Box::new(RestrictionDef::new(
        Restriction::Event {
            pattern: EventPattern::ZoneChange {
                from: None,
                to: Some(Zone::Battlefield),
                cause: None,
                object: None,
            },
            affected_objects: ObjectSet::battlefield_filter(ObjectFilter::ByType(CardType::Creature)),
            affected_players: PlayerSet::Nobody,
            by: None,
        },
    ))));
    put_on_battlefield(&mut game, enchantment_with("No Creatures May Enter", no_creatures), 1);
    let start = game.events.records().len();
    let objects_before = game.objects.len();

    resolve_card(&mut game, 0, raise_the_alarm(), &test_dp());

    assert!(tokens(&game).is_empty());
    assert!(creations(&game, start).is_empty(), "the token is not created, and nothing announces it");
    assert_eq!(
        game.objects.len(),
        objects_before + 1,
        "the fixture spell was added and no token object survives"
    );
}

// ---------------------------------------------------------------------------
// The precedent: the outer event is reported as decided; the log counts what
// was created
// ---------------------------------------------------------------------------

/// Two tokens proposed, one refused. `execute_actions` reports the creation as
/// CR 616.1 left it — `defs` of two — and the log holds one `TokenCreated`:
/// CR 111.5's un-creation is no more an event than `add_object` was, and
/// `DrawCards { n }` against a library that runs out is the same shape.
///
/// The refusal is a fixture — an optional `Prevent` on entries, "you may have
/// a creature entering under your control not enter" — answered yes for the
/// first token and no for the second.
#[test]
fn a_creation_is_reported_as_decided_and_the_log_counts_what_was_created() {
    let mut game = setup_two_player_game();
    let may_refuse = static_ability(Effect::Replacement(Box::new(
        ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::battlefield_filter(ObjectFilter::And(
                    Box::new(ObjectFilter::ByType(CardType::Creature)),
                    Box::new(ObjectFilter::ByController(PlayerRef::You)),
                )),
            Rewrite::Prevent,
        )
        .optional(),
    )));
    let gate = put_on_battlefield(&mut game, enchantment_with("Gate of Refusal", may_refuse), 0);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(
        ChoiceKind::ApplyOptionalReplacement { affected_object: None, source: gate },
        vec![0],
    );
    dp.expect_pick_n(
        ChoiceKind::ApplyOptionalReplacement { affected_object: None, source: gate },
        vec![],
    );
    let start = game.events.records().len();
    let objects_before = game.objects.len();
    let proposed = GameAction::CreateTokens { defs: vec![soldier_token(); 2], controller: 0 };

    let performed = game
        .execute_actions(vec![proposed.clone()], &ActionContext::new(&dp))
        .expect("the creation performs");

    assert_eq!(
        performed,
        vec![proposed],
        "the outer event is reported as decided — two, the count CR 616.1 left it with"
    );
    assert_eq!(creations(&game, start).len(), 1, "and the log says one was created");
    assert_eq!(tokens(&game).len(), 1);
    assert_eq!(
        game.objects.len(),
        objects_before + 1,
        "CR 111.5: the refused token is not created"
    );
}

/// `Rewrite::Amount` over a creation admits a multiplier and refuses the
/// rest as the authoring error it would be: a Xorn-shaped `Plus(1)` is the
/// customer that adds the next arm, and until it does the pipeline says so
/// rather than guessing which def "plus one" repeats.
#[test]
fn amount_over_a_creation_admits_a_multiplier_and_refuses_the_rest() {
    let mut game = setup_two_player_game();
    let plus_one = static_ability(Effect::Replacement(Box::new(
        ReplacementDef::new(
            EventPattern::CreateTokens { kind: None },
            ObjectSet::NO_OBJECTS,
            Rewrite::Amount(AmountRewrite::Plus(1)),
        )
        .affecting_players(PlayerSet::You),
    )));
    put_on_battlefield(&mut game, enchantment_with("That Many Plus One", plus_one), 0);
    let source = put_in_hand(&mut game, fixture_object(), 0);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: 0,
        targets: vec![],
        replaced_amount: None,
        damage_prevented: None,
    };
    let effect = raise_the_alarm().abilities[0].effect.clone();

    let err = game.resolve_effect(&effect, &ctx, &test_dp()).unwrap_err();

    assert!(err.contains("multiplier"), "{err}");
}

// ---------------------------------------------------------------------------
// The type half — what a token can now be
// ---------------------------------------------------------------------------

/// CR 111.9's "create Boo, a legendary 1/1 red Hamster creature token with
/// trample and haste", created twice: the supertype reaches the token, and so
/// does CR 704.5j.
#[test]
fn a_legendary_token_created_twice_meets_the_legend_rule() {
    let mut game = setup_two_player_game();
    let boo = TokenDef {
        name: Some("Boo".to_string()),
        colors: vec![Color::Red],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(CreatureType::Hamster)],
        supertypes: vec![Supertype::Legendary],
        power: Some(1),
        toughness: Some(1),
        keyword_flags: vec![KeywordFlag::Trample, KeywordFlag::Haste],
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
    };

    create(&mut game, 0, boo, 2, &test_dp());

    let boos = tokens(&game);
    assert_eq!(boos.len(), 2);
    for id in &boos {
        assert_eq!(get_effective_name(&game, *id), "Boo", "CR 111.9: the name the effect gave");
        assert!(has_supertype(&game, *id, Supertype::Legendary));
    }

    let dp = RecordingDecisionProvider::picking(0);
    game.check_state_based_actions(&dp).unwrap();

    assert_eq!(tokens(&game).len(), 1, "CR 704.5j");
    assert!(dp.kinds().iter().any(|k| k.starts_with("LegendRule")), "{:?}", dp.kinds());
}

/// A noncreature token has no power or toughness (CR 208.3), and Root Maze's
/// "artifacts and lands enter tapped" reaches it through the same entry batch
/// a creature token takes.
#[test]
fn an_artifact_token_enters_tapped_under_root_maze() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, root_maze(), 1);
    let trinket = TokenDef {
        name: Some("Trinket".to_string()),
        colors: Vec::new(),
        types: vec![CardType::Artifact],
        subtypes: Vec::new(),
        supertypes: Vec::new(),
        power: None,
        toughness: None,
        keyword_flags: Vec::new(),
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
    };

    create(&mut game, 0, trinket, 1, &test_dp());

    let trinkets = tokens(&game);
    assert_eq!(trinkets.len(), 1);
    assert!(game.battlefield.get(&trinkets[0]).unwrap().tapped, "Root Maze applied to its entry");
    assert_eq!(get_effective_power(&game, trinkets[0]), None, "a noncreature has no power");
}

/// The lowering, field by field: CR 111.4's own example for the default name,
/// a given name kept, no power on a noncreature, and the four fields RE-4
/// added reaching the `CardData` — a Role-shaped Aura token with an enchant
/// filter, a static ability and rules text.
#[test]
fn a_token_def_lowers_every_field_it_carries() {
    let dwarves = TokenDef {
        name: None,
        colors: vec![Color::Red],
        types: vec![CardType::Creature],
        subtypes: vec![
            Subtype::Creature(CreatureType::Dwarf),
            Subtype::Creature(CreatureType::Berserker),
        ],
        supertypes: Vec::new(),
        power: Some(2),
        toughness: Some(1),
        keyword_flags: Vec::new(),
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
    };
    assert_eq!(dwarves.effective_name(), "Dwarf Berserker Token", "CR 111.4's example");
    let data = dwarves.card_data();
    assert_eq!(data.name, "Dwarf Berserker Token");
    assert_eq!((data.power, data.toughness), (Some(2), Some(1)));
    assert!(data.mana_cost.is_none(), "CR 111.6");

    let role = TokenDef {
        name: Some("Cursed".to_string()),
        colors: Vec::new(),
        types: vec![CardType::Enchantment],
        subtypes: vec![
            Subtype::Enchantment(EnchantmentType::Aura),
            Subtype::Enchantment(EnchantmentType::Role),
        ],
        supertypes: vec![Supertype::Legendary],
        power: None,
        toughness: None,
        keyword_flags: Vec::new(),
        abilities: vec![static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::Untap,
            ObjectSet::Host,
            Rewrite::Prevent,
        ))))],
        rules_text: "Enchanted creature has base power and toughness 1/1.".to_string(),
        enchant_filter: Some(SelectionFilter::Creature),
        enters_tapped: false,
    };
    let data = role.card_data();
    assert_eq!(data.name, "Cursed", "CR 111.9 / 111.10j: the name the rule gives");
    assert_eq!((data.power, data.toughness), (None, None), "CR 208.3");
    assert!(data.supertypes.contains(&Supertype::Legendary));
    assert_eq!(data.abilities.len(), 1);
    assert_eq!(data.rules_text, "Enchanted creature has base power and toughness 1/1.");
    assert!(data.enchant_filter.is_some(), "CR 303.4");
    assert!(data.subtypes.contains(&Subtype::Enchantment(EnchantmentType::Role)));
}

/// The two RE-4 defs, as registered: what the cards say, lowered.
#[test]
fn the_registered_token_defs_say_what_the_cards_say() {
    let soldier = soldier_token().card_data();
    assert_eq!(soldier.name, "Soldier Token");
    assert!(soldier.colors.contains(&Color::White));
    assert_eq!((soldier.power, soldier.toughness), (Some(1), Some(1)));
    let goblin = goblin_token().card_data();
    assert_eq!(goblin.name, "Goblin Token");
    assert!(goblin.colors.contains(&Color::Red));
    assert!(goblin.subtypes.contains(&Subtype::Creature(CreatureType::Goblin)));
}

// ---------------------------------------------------------------------------
// Found by this phase's A/B — a rider carries the replaced event's applied set
// ---------------------------------------------------------------------------

/// Two Thought Reflections and two Alms Collectors across two players, and P0
/// draws a card. P0's Reflection doubles it; P1's Collector makes it one and
/// has P1 draw — a draw that is the *rest of the same replacement*, so it
/// carries {Reflection P0, Collector P1}; P1's Reflection doubles that (its
/// first opportunity); P0's Collector halves it and has P0 draw; and that draw
/// meets four effects that have each had their one opportunity (CR 614.5, and
/// Alms Collector's ruling: an applied effect "can't be applied again to the
/// resulting events"). P0 draws two, P1 draws one, and nothing had to end it.
///
/// With a fresh applied set per rider — RE-2's design — this board handed one
/// draw back and forth until the stack overflowed, which is how RE-4's
/// four-player A/B found it (seed 12523, a Notion Thief in P0's seat).
#[test]
fn a_riders_draw_carries_the_replaced_events_applied_set() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 60);
    fill_library(&mut game, 1, 60);
    put_on_battlefield(&mut game, thought_reflection(), 0);
    put_on_battlefield(&mut game, alms_collector(), 0);
    put_on_battlefield(&mut game, thought_reflection(), 1);
    put_on_battlefield(&mut game, alms_collector(), 1);
    let effect = Effect::Atom(
        Primitive::DrawCards(AmountExpr::Fixed(1)),
        EffectRecipient::Controller,
    );

    resolve_for(&mut game, 0, &effect, &test_dp());

    assert_eq!(game.result, None, "no loop, and no rule was needed to end one");
    assert_eq!(game.players[0].hand.len(), 1 + 2, "the fixture spell and two drawn cards");
    assert_eq!(game.players[1].hand.len(), 1, "the Collector's rider draw, doubled and halved");
}

/// The control: one Thought Reflection beside one Alms Collector is the
/// board RE-2 walked, and it ends — the Collector's rider draw is doubled
/// once and stops, because nothing hands it back.
#[test]
fn one_reflection_beside_one_collector_ends() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 60);
    fill_library(&mut game, 1, 60);
    put_on_battlefield(&mut game, thought_reflection(), 0);
    put_on_battlefield(&mut game, alms_collector(), 1);
    let effect = Effect::Atom(
        Primitive::DrawCards(AmountExpr::Fixed(1)),
        EffectRecipient::Controller,
    );

    resolve_for(&mut game, 0, &effect, &test_dp());

    assert_eq!(game.result, None, "no loop: the draw ends");
    // P0 drew one (the Collector made two into one), P1 drew one (the rider).
    assert_eq!(game.players[0].hand.len(), 1 + 1, "the fixture spell and one drawn card");
    assert_eq!(game.players[1].hand.len(), 1);
}

// ---------------------------------------------------------------------------
// The review's arms — the kind, the template, and a def that enters tapped
// ---------------------------------------------------------------------------

/// An artifact token with no abilities, to sit beside creatures in one
/// creation. Its name is its own; no printed token is being imitated.
fn trinket() -> TokenDef {
    TokenDef {
        name: Some("Trinket".to_string()),
        colors: Vec::new(),
        types: vec![CardType::Artifact],
        subtypes: Vec::new(),
        supertypes: Vec::new(),
        power: None,
        toughness: None,
        keyword_flags: Vec::new(),
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
    }
}

/// A creation of several kinds at once, proposed directly: Academy
/// Manufactor's shape, which no registered card produces yet.
fn create_mixed(game: &mut GameState, player: PlayerId, defs: Vec<TokenDef>, dp: &dyn DecisionProvider) {
    game.execute_actions(
        vec![GameAction::CreateTokens { defs, controller: player }],
        &ActionContext::new(dp),
    )
    .expect("the creation performs");
}

/// Divine Visitation's first ruling: the characteristics are entirely
/// replaced — two Soldiers become two 4/4 Angels with flying and vigilance
/// and nothing of the Soldier — and "anything else specified in the effect
/// creating the token (such as tapped …) still applies": a creation that
/// enters tapped still does.
#[test]
fn divine_visitation_replaces_the_creatures_and_keeps_how_they_entered() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, divine_visitation(), 0);
    let dp = RecordingDecisionProvider::picking(0);

    resolve_card(&mut game, 0, raise_the_alarm(), &dp);

    let angels = tokens(&game);
    assert_eq!(angels.len(), 2, "that many: two Soldiers, two Angels");
    for id in &angels {
        assert_eq!(get_effective_name(&game, *id), "Angel Token");
        assert_eq!((get_effective_power(&game, *id), get_effective_toughness(&game, *id)), (Some(4), Some(4)));
        assert!(get_effective_subtypes(&game, *id).contains(&Subtype::Creature(CreatureType::Angel)));
        assert!(!get_effective_subtypes(&game, *id).contains(&Subtype::Creature(CreatureType::Soldier)));
        assert!(!game.battlefield.get(id).unwrap().tapped);
    }
    assert_eq!(dp.prompts(), 0);

    // The same effect, said to enter tapped: the Angels do.
    let tapped_soldier = TokenDef { enters_tapped: true, ..soldier_token() };
    create(&mut game, 0, tapped_soldier, 1, &test_dp());
    let all = tokens(&game);
    assert_eq!(all.len(), 3);
    let newest = *all.last().unwrap();
    assert_eq!(get_effective_name(&game, newest), "Angel Token");
    assert!(game.battlefield.get(&newest).unwrap().tapped, "how the effect said it enters carries over");
}

/// The kind is asked of the token's def, never of the entry's frame — Divine
/// Visitation's second ruling, about a noncreature token that would be a
/// creature on the battlefield. Here the plain case: a Trinket created beside
/// a Soldier is not a creature token, so the Soldier becomes an Angel and the
/// Trinket stays a Trinket, in its place.
#[test]
fn divine_visitation_reads_the_def_and_not_the_frame() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, divine_visitation(), 0);

    create_mixed(&mut game, 0, vec![trinket(), soldier_token(), trinket()], &test_dp());

    let names: Vec<String> = tokens(&game).iter().map(|id| get_effective_name(&game, *id)).collect();
    assert_eq!(names, vec!["Trinket", "Angel Token", "Trinket"], "only the creature def is replaced, in its place");
}

#[test]
fn divine_visitation_leaves_an_opponents_creation_alone() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, divine_visitation(), 0);

    resolve_card(&mut game, 1, raise_the_alarm(), &test_dp());

    assert!(tokens(&game).iter().all(|id| get_effective_name(&game, *id) == "Soldier Token"));
}

/// Beside Parallel Lives the answer is four Angels either way: doubling then
/// replacing "that many", or replacing then doubling, commute — and since
/// RE-5's review the commutation table says so, so the creating player is not
/// asked. This was the sixth shape `backlog.md` §2.29 named as the table's
/// trigger; until then both orders were asked for and the test stated the
/// commutation by hand.
#[test]
fn divine_visitation_beside_parallel_lives_is_four_angels_in_either_order() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, parallel_lives(), 0);
    put_on_battlefield(&mut game, divine_visitation(), 0);
    let dp = RecordingDecisionProvider::picking(0);

    resolve_card(&mut game, 0, raise_the_alarm(), &dp);

    assert_eq!(dp.prompts(), 0, "a multiplier and a replace-by-that-many commute");
    let angels = tokens(&game);
    assert_eq!(angels.len(), 4);
    assert!(angels.iter().all(|id| get_effective_name(&game, *id) == "Angel Token"));
}

/// The append mode, from a fixture in Xorn's shape — "those tokens plus an
/// additional one" — and the reason `AmountRewrite::Plus` is refused over a
/// creation: the additional token is a *named* def.
#[test]
fn an_append_template_keeps_the_creation_and_joins_its_own_def() {
    let mut game = setup_two_player_game();
    let one_more = static_ability(Effect::Replacement(Box::new(
        ReplacementDef::new(
            EventPattern::CreateTokens { kind: Some(TokenKind::of_type(CardType::Artifact)) },
            ObjectSet::NO_OBJECTS,
            Rewrite::Instead(GameActionTemplate::CreateTokens {
                def: trinket(),
                count: TemplateAmount::Fixed(1),
                mode: TokenSubstitution::Append,
            }),
        )
        .affecting_players(PlayerSet::You),
    )));
    put_on_battlefield(&mut game, enchantment_with("One More Trinket", one_more), 0);

    create_mixed(&mut game, 0, vec![trinket(), soldier_token()], &test_dp());

    let names: Vec<String> = tokens(&game).iter().map(|id| get_effective_name(&game, *id)).collect();
    assert_eq!(names, vec!["Trinket", "Soldier Token", "Trinket"], "the creation, then the extra");

    // And a creation with no artifact in it is not the pattern's business.
    let before = tokens(&game).len();
    resolve_card(&mut game, 0, raise_the_alarm(), &test_dp());
    assert_eq!(tokens(&game).len(), before + 2);
}

/// A def that says it enters tapped does, through the same entry seed a
/// printed "enters tapped" uses — and a replacement that also taps it is not
/// asked about, since the seed already says so.
#[test]
fn a_def_that_enters_tapped_does() {
    let mut game = setup_two_player_game();
    let tapped_trinket = TokenDef { enters_tapped: true, ..trinket() };

    create(&mut game, 0, tapped_trinket, 2, &test_dp());

    let trinkets = tokens(&game);
    assert_eq!(trinkets.len(), 2);
    assert!(trinkets.iter().all(|id| game.battlefield.get(id).unwrap().tapped));
}

/// Bard's two halves, and the ruling that two of him multiply both by four.
#[test]
fn two_bards_quadruple_both_halves() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 20);
    put_on_battlefield(&mut game, bard_king_of_dale(), 0);
    put_on_battlefield(&mut game, bard_king_of_dale(), 0);
    let dp = RecordingDecisionProvider::picking(0);

    resolve_card(&mut game, 0, raise_the_alarm(), &dp);
    assert_eq!(tokens(&game).len(), 8, "four times the number of tokens");

    let hand_before = game.players[0].hand.len();
    let draw = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller);
    resolve_for(&mut game, 0, &draw, &dp);
    assert_eq!(game.players[0].hand.len(), hand_before + 1 + 4, "the fixture spell, and one draw made four");
    assert_eq!(dp.prompts(), 0, "two multipliers, and two draw doublers, commute");
}

/// Bard's ruling: "if an effect creates more than one kind of token, it'll
/// create twice as many of each kind" — repeated in place, so the kinds stay
/// adjacent and the creation's order is the tokens'.
#[test]
fn bard_doubles_each_kind_of_a_mixed_creation() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, bard_king_of_dale(), 0);

    create_mixed(&mut game, 0, vec![soldier_token(), trinket()], &test_dp());

    let names: Vec<String> = tokens(&game).iter().map(|id| get_effective_name(&game, *id)).collect();
    assert_eq!(names, vec!["Soldier Token", "Soldier Token", "Trinket", "Trinket"]);
}

/// A kind the creation does not contain is not a match, and a multiplier on
/// a kind repeats only that kind — "creature tokens" doubled beside a Trinket
/// leaves the Trinket single.
#[test]
fn a_multiplier_on_a_kind_repeats_only_that_kind() {
    let mut game = setup_two_player_game();
    let creatures_twice = static_ability(Effect::Replacement(Box::new(
        ReplacementDef::new(
            EventPattern::CreateTokens { kind: Some(TokenKind::of_type(CardType::Creature)) },
            ObjectSet::NO_OBJECTS,
            Rewrite::Amount(AmountRewrite::Multiplier(2)),
        )
        .affecting_players(PlayerSet::You),
    )));
    put_on_battlefield(&mut game, enchantment_with("Creatures Twice", creatures_twice), 0);

    create_mixed(&mut game, 0, vec![trinket(), soldier_token()], &test_dp());
    let names: Vec<String> = tokens(&game).iter().map(|id| get_effective_name(&game, *id)).collect();
    assert_eq!(names, vec!["Trinket", "Soldier Token", "Soldier Token"]);

    let start = game.events.records().len();
    create(&mut game, 0, trinket(), 1, &test_dp());
    assert_eq!(creations(&game, start).len(), 1, "no creature in it: the pattern does not match at all");
}
