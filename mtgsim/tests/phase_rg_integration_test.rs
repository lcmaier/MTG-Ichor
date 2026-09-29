//! Phase RG integration tests: the entry state, CR 614.1c's two halves as one
//! shape (`replacement-architecture.md` §3.5).
//!
//! What this file proves, in the order the phase built it:
//!
//! 1. **The shape.** What a permanent enters *as* is applied at its own
//!    timestamp at each edit's layer, in the CR 614.12 frame and on the
//!    permanent alike; what it enters *with* is state, and the last applied
//!    status wins (D1, D2, D5). That the frame and the performer build one
//!    permanent is a unit test beside `Lookahead`.
//! 2. **The feeds table** (D3). CR 616.1's order is asked exactly where one
//!    member's write can make another stop applying, or where two members set
//!    opposite statuses, and nowhere else.
//! 3. **Master Biomancer's Mutant** (D4). The first `CharacteristicEdit`: at
//!    layer 4 at the creature's timestamp, not copied, and kept after the
//!    Biomancer leaves.
//! 4. **Enters untapped** (D5, D6), on Archelos, Lagoon Mystic: one test per
//!    ruling, and one through `cast_spell` from hand, from an exact pool,
//!    under `ManaWindowStop`.
//! 5. **CR 306.5b gathered** (D7): an ability on a planeswalker's frame, so
//!    CR 616.1 orders it (the Kaito board, cast from hand) and Layer 6 removes
//!    it (the Humility board); and the feeds table's premise (d), the order
//!    two counter writers make observable to a multiplier.
//!
//! Fixtures are built inline, named for the printed card whose board they
//! stand in for, and never registered.

use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::keyword_creatures::wall_of_stone;
use mtgsim::cards::phase_lf_cards::humility;
use mtgsim::cards::phase_rc_cards::{adaptive_shimmerer, idyllic_beachfront, master_biomancer};
use mtgsim::cards::phase_rd_cards::loyalty_probe;
use mtgsim::cards::phase_re_cards::{doubling_season, soldier_token};
use mtgsim::cards::phase_rg_cards::archelos_lagoon_mystic;
use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeCause};
use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, Layer};
use mtgsim::engine::layers::intrinsic::is_intrinsic_entry_ability;
use mtgsim::engine::layers::{compute_as_entering, copiable_values};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::objects::object::GameObject;
use mtgsim::oracle::characteristics::{
    get_effective_abilities, get_effective_name, get_effective_power, get_effective_toughness,
    get_effective_types, has_subtype,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    creature_with_ability, put_in_graveyard, put_in_hand, put_on_battlefield, registered, setup_game,
    setup_two_player_game, static_ability, test_ctx,
};
use mtgsim::types::card_types::{CardType, CardTypes, CreatureType, Subtype, Supertype};
use mtgsim::types::effects::{
    AmountExpr, CharacteristicEdit, Condition, CounterType, Duration, Effect, EffectRecipient, ObjectFilter,
    ObjectSet, PlayerRef, Primitive, TokenDef, TypeChange,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::replacement::{EnterMods, EnterModsTemplate, EventPattern, ReplacementDef, Rewrite, TapStatus};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use mtgsim::ui::mana_window_stop::ManaWindowStop;

fn mutant() -> Subtype {
    Subtype::Creature(CreatureType::Mutant)
}

/// Master Biomancer's edit, "as a Mutant in addition to its other types".
fn as_a_mutant() -> CharacteristicEdit {
    CharacteristicEdit::Types(TypeChange { add_subtypes: vec![mutant()], ..TypeChange::NONE })
}

fn entering_as(edits: Vec<CharacteristicEdit>) -> EnterMods {
    EnterMods { edits: Some(Arc::from(edits)), ..EnterMods::NONE }
}

fn creatures() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Creature)
}

fn and(a: ObjectFilter, b: ObjectFilter) -> ObjectFilter {
    ObjectFilter::And(Box::new(a), Box::new(b))
}

/// An enchantment whose one ability modifies how the permanents `filter`
/// matches enter the battlefield.
fn entry_effect(name: &str, filter: ObjectFilter, template: EnterModsTemplate) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::battlefield_filter(filter),
            Rewrite::EnterWith(template),
        )))))
        .build()
}

/// Master Biomancer's Mutant clause alone: "Each other creature you control
/// enters ... as a Mutant in addition to its other types."
fn creatures_enter_as_mutants() -> Arc<CardData> {
    entry_effect(
        "Creatures enter as Mutants",
        and(creatures(), ObjectFilter::ByController(PlayerRef::You)),
        EnterModsTemplate { status: None, counters: Vec::new(), edits: vec![as_a_mutant()] },
    )
}

fn with_a_charge_counter() -> EnterModsTemplate {
    EnterModsTemplate::with_counters(CounterType::Charge, 1)
}

/// Return `card` from `owner`'s graveyard to the battlefield, answering each
/// CR 616.1 prompt about it from `picks` in order. An unscripted prompt, or a
/// scripted one left over, fails the test.
fn return_to_battlefield(game: &mut GameState, card: Arc<CardData>, owner: PlayerId, picks: &[usize]) -> ObjectId {
    let id = put_in_graveyard(game, card, owner);
    let dp = ScriptedDecisionProvider::new();
    for &pick in picks {
        dp.expect_pick_n(ChoiceKind::ChooseReplacementEffect { affected_object: Some(id) }, vec![pick]);
    }
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("the entry is proposed");
    assert!(dp.is_empty(), "every scripted CR 616.1 prompt was asked");
    id
}

// ---------------------------------------------------------------------------
// 1. The shape
// ---------------------------------------------------------------------------

/// An edit is what the permanent enters as, so the CR 614.12 frame reads it
/// before the entry and the permanent carries it after, at layer 4 both
/// times, with no registry row.
#[test]
fn an_entry_edit_is_in_the_frame_and_on_the_permanent() {
    let mut game = setup_two_player_game();
    let mods = entering_as(vec![as_a_mutant()]);

    let bears = put_in_graveyard(&mut game, grizzly_bears(), 0);
    let frame = compute_as_entering(&game, bears, 0, &mods).expect("the bears exist");
    assert!(frame.subtypes.contains(&mutant()), "the frame is the permanent as it would exist");
    assert!(!has_subtype(&game, bears, &mutant()), "the card in the graveyard is not a Mutant");

    let rows = game.continuous_effects.iter().count();
    let placed = game.add_object(GameObject::new(grizzly_bears(), 0, Zone::Battlefield));
    game.place_on_battlefield(placed, 0, &mods);
    assert!(has_subtype(&game, placed, &mutant()));
    assert_eq!(game.continuous_effects.iter().count(), rows, "state on the permanent, not a row");
}

/// CR 110.5b gives a permanent one tapped/untapped status, and the last
/// effect applied sets it (Spelunking's first ruling). An effect that names no
/// status leaves it, and counters still add.
#[test]
fn the_last_applied_status_is_the_one_the_permanent_enters_with() {
    let untapped = EnterMods { status: Some(TapStatus::Untapped), ..EnterMods::NONE };

    let mut mods = EnterMods::tapped();
    mods.merge(&untapped);
    assert!(!mods.enters_tapped(), "untapped applied last");
    mods.merge(&EnterMods::tapped());
    assert!(mods.enters_tapped(), "tapped applied last");
    mods.merge(&EnterMods::with_counters(CounterType::PlusOnePlusOne, 1));
    assert!(mods.enters_tapped(), "an effect that names no status leaves it");
    assert_eq!(mods.counters.len(), 1);
    assert!(!EnterMods::NONE.enters_tapped(), "CR 110.5b's default");
}

// ---------------------------------------------------------------------------
// 2. The feeds table
// ---------------------------------------------------------------------------

/// Premise (c): two statuses that differ are an order, since the last one
/// applied is the one the permanent enters with, and the entering permanent's
/// controller chooses it. Two that agree, or a status beside counters, are not.
#[test]
fn opposite_statuses_are_an_order_and_equal_ones_are_not() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, entry_effect("Permanents enter tapped", ObjectFilter::All, EnterModsTemplate::tapped()), 1);
    put_on_battlefield(&mut game, entry_effect("Permanents enter untapped", ObjectFilter::All, EnterModsTemplate::untapped()), 1);
    let tapped_first = return_to_battlefield(&mut game, grizzly_bears(), 0, &[0]);
    assert!(!game.battlefield[&tapped_first].tapped, "untapped applied last");
    let untapped_first = return_to_battlefield(&mut game, grizzly_bears(), 0, &[1]);
    assert!(game.battlefield[&untapped_first].tapped, "tapped applied last");

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, entry_effect("Permanents enter tapped", ObjectFilter::All, EnterModsTemplate::tapped()), 1);
    put_on_battlefield(&mut game, entry_effect("Creatures enter tapped", creatures(), EnterModsTemplate::tapped()), 1);
    put_on_battlefield(&mut game, entry_effect("Creatures enter charged", creatures(), with_a_charge_counter()), 1);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    let entry = &game.battlefield[&bears];
    assert!(entry.tapped);
    assert_eq!(entry.counter_count(CounterType::Charge), 1);
}

/// `PowerLE` can be unmatched only by something that raises power. Beside a
/// status and a charge counter nothing does, so nothing is asked; it was
/// asked before the table, whatever stood beside it.
#[test]
fn a_power_filter_beside_members_that_raise_no_power_is_not_asked() {
    let mut game = setup_two_player_game();
    let small = and(creatures(), ObjectFilter::PowerLE(2));
    put_on_battlefield(&mut game, entry_effect("Small creatures enter tapped", small, EnterModsTemplate::tapped()), 1);
    put_on_battlefield(&mut game, entry_effect("Creatures enter charged", creatures(), with_a_charge_counter()), 1);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    let entry = &game.battlefield[&bears];
    assert!(entry.tapped);
    assert_eq!(entry.counter_count(CounterType::Charge), 1);
}

/// Premise (a) with an edit. Adding Mutant can turn a leaf only on, so it can
/// unmatch "non-Mutant creatures" and nothing positive: a Mutant filter is
/// either not a candidate yet, and becomes one after the edit applies
/// (CR 616.2), or matches already; a type filter is untouched.
#[test]
fn an_added_subtype_asks_only_where_a_not_reads_it() {
    let non_mutants = and(creatures(), ObjectFilter::Not(Box::new(ObjectFilter::BySubtype(mutant()))));
    for (pick, tapped) in [(0, false), (1, true)] {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, creatures_enter_as_mutants(), 0);
        put_on_battlefield(&mut game, entry_effect("Non-Mutants enter tapped", non_mutants.clone(), EnterModsTemplate::tapped()), 1);
        let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[pick]);
        assert!(has_subtype(&game, bears, &mutant()));
        assert_eq!(game.battlefield[&bears].tapped, tapped, "the Mutant edit first unmatches the tapper");
    }

    let mutants = and(creatures(), ObjectFilter::BySubtype(mutant()));
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, creatures_enter_as_mutants(), 0);
    put_on_battlefield(&mut game, entry_effect("Mutants enter tapped", mutants, EnterModsTemplate::tapped()), 1);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    assert!(game.battlefield[&bears].tapped, "a candidate once the edit applied (CR 616.2)");
    let born_mutant = CardDataBuilder::new("Born Mutant")
        .card_type(CardType::Creature)
        .subtype(mutant())
        .power_toughness(2, 2)
        .build();
    let mutant_id = return_to_battlefield(&mut game, born_mutant, 0, &[]);
    assert!(game.battlefield[&mutant_id].tapped, "both applied, in either order");

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, creatures_enter_as_mutants(), 0);
    put_on_battlefield(&mut game, entry_effect("Creatures enter tapped", creatures(), EnterModsTemplate::tapped()), 1);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    assert!(game.battlefield[&bears].tapped && has_subtype(&game, bears, &mutant()));
}

/// "As long as this creature is untapped, it's an artifact in addition to its
/// other types." No printed card says this; it is the one condition leaf that
/// reads entry state today, on the object the entry is deciding.
fn artifact_while_untapped() -> Arc<CardData> {
    creature_with_ability(
        "Artifact While Untapped",
        2,
        2,
        static_ability(Effect::Conditional(
            Condition::SourceUntapped,
            Box::new(Effect::Atom(
                Primitive::ChangeType(
                    TypeChange { add_types: vec![CardType::Artifact], ..TypeChange::NONE },
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::ThisObject,
            )),
        )),
    )
}

/// The table's last row. The entering creature is an artifact only while
/// untapped, so "enters tapped" applied first unmatches "artifacts enter with
/// a charge counter", though no member writes a type: the two orders give a
/// counter or none, and the controller is asked.
#[test]
fn a_status_write_is_an_order_where_the_entering_permanents_type_hangs_on_its_status() {
    for (pick, charged) in [(0, 0), (1, 1)] {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, entry_effect("Permanents enter tapped", ObjectFilter::All, EnterModsTemplate::tapped()), 1);
        let artifacts = ObjectFilter::ByType(CardType::Artifact);
        put_on_battlefield(&mut game, entry_effect("Artifacts enter charged", artifacts, with_a_charge_counter()), 1);
        let id = return_to_battlefield(&mut game, artifact_while_untapped(), 0, &[pick]);
        let entry = &game.battlefield[&id];
        assert!(entry.tapped);
        assert_eq!(entry.counter_count(CounterType::Charge), charged);
    }
}

// ---------------------------------------------------------------------------
// 3. Master Biomancer's Mutant (D4)
// ---------------------------------------------------------------------------

/// "Each other creature you control enters with ... and as a Mutant in
/// addition to its other types." The Mutant is how the creature entered, so
/// it outlives the Biomancer, whose effect it never was. The counters are its
/// power as the creature enters (its one ruling).
// COVERS: ATOM-614.1c-001
#[test]
fn a_creature_entering_under_master_biomancer_stays_a_mutant_after_it_leaves() {
    let mut game = setup_two_player_game();
    let biomancer = put_on_battlefield(&mut game, master_biomancer(), 0);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    assert_eq!(game.battlefield[&bears].counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(has_subtype(&game, bears, &mutant()));

    game.change_zone(biomancer, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx()).unwrap();
    assert!(has_subtype(&game, bears, &mutant()), "how it entered, not the Biomancer's effect");
}

/// The Mutant applies at layer 4 at the creature's own timestamp (CR 613.7),
/// so an effect that sets creature types with an earlier timestamp applies
/// under it, and one with a later timestamp over it (CR 205.1a).
// COVERS-PARTIAL: ATOM-614.1c-001
#[test]
fn a_later_layer_4_effect_applies_over_the_mutant_and_an_earlier_one_under_it() {
    let slivers = |game: &mut GameState, source: ObjectId| {
        let timestamp = game.allocate_timestamp();
        let set = EffectModification::SetSubtypes(HashSet::from([Subtype::Creature(CreatureType::Sliver)]));
        game.continuous_effects.add(ContinuousEffect {
            affected_objects: ObjectSet::battlefield_filter(creatures()),
            ..registered(source, Layer::Layer4Type, timestamp, set)
        });
    };
    let sliver = Subtype::Creature(CreatureType::Sliver);

    let mut game = setup_two_player_game();
    let biomancer = put_on_battlefield(&mut game, master_biomancer(), 0);
    slivers(&mut game, biomancer);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    assert!(has_subtype(&game, bears, &sliver) && has_subtype(&game, bears, &mutant()), "earlier: under it");

    let mut game = setup_two_player_game();
    let biomancer = put_on_battlefield(&mut game, master_biomancer(), 0);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    slivers(&mut game, biomancer);
    assert!(has_subtype(&game, bears, &sliver) && !has_subtype(&game, bears, &mutant()), "later: over it");
}

/// The Mutant is not a copiable value (CR 707.2's last sentence), so a copy
/// of the creature is not a Mutant; and a copy effect over the creature
/// replaces its copiable values and leaves the Mutant, which is not one.
#[test]
fn the_mutant_is_not_copied_and_a_copy_effect_over_it_leaves_it() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, master_biomancer(), 0);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    let values = copiable_values(&game, bears).expect("the bears are on the battlefield");
    assert!(!values.subtypes.contains(&mutant()), "a Clone of it would not be a Mutant");

    let wall = put_on_battlefield(&mut game, wall_of_stone(), 1);
    let cytoshape_row = EffectModification::CopyFrom(Arc::new(copiable_values(&game, wall).unwrap()));
    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(registered(bears, Layer::Layer1Copy, timestamp, cytoshape_row));
    assert_eq!(get_effective_name(&game, bears), "Wall of Stone");
    assert!(has_subtype(&game, bears, &mutant()), "the copy replaced the copiable values, and the Mutant is not one");
}

/// D3's example on the printed card: beside "non-Mutant creatures enter
/// tapped", Master Biomancer applied first unmatches the tapper, so the
/// entering creature's controller is asked.
#[test]
fn master_biomancer_beside_a_non_mutant_filter_is_an_order() {
    let non_mutants = and(creatures(), ObjectFilter::Not(Box::new(ObjectFilter::BySubtype(mutant()))));
    for (pick, tapped) in [(0, false), (1, true)] {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, master_biomancer(), 0);
        put_on_battlefield(&mut game, entry_effect("Non-Mutants enter tapped", non_mutants.clone(), EnterModsTemplate::tapped()), 1);
        let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[pick]);
        assert_eq!(game.battlefield[&bears].counter_count(CounterType::PlusOnePlusOne), 2);
        assert_eq!(game.battlefield[&bears].tapped, tapped);
    }
}

// ---------------------------------------------------------------------------
// 4. Enters untapped: Archelos, Lagoon Mystic (D5, D6)
// ---------------------------------------------------------------------------

/// Answers each CR 616.1 prompt from a script of who must be asked and which
/// candidate they pick. Any other prompt panics, and so does a script left
/// over, so a test states every order it expects and who owns it.
struct Orders {
    script: RefCell<VecDeque<(PlayerId, usize)>>,
}

impl Orders {
    fn new(script: &[(PlayerId, usize)]) -> Self {
        Orders { script: RefCell::new(script.iter().copied().collect()) }
    }
}

impl Drop for Orders {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(self.script.borrow().is_empty(), "orders never asked: {:?}", self.script.borrow());
        }
    }
}

impl DecisionProvider for Orders {
    fn pick_n(&self, _: &GameState, player: PlayerId, ctx: &ChoiceContext, _: &[ChoiceOption], _: (usize, usize)) -> Vec<usize> {
        assert!(
            matches!(ctx.kind, ChoiceKind::ChooseReplacementEffect { .. }),
            "unscripted prompt: {:?}",
            ctx.kind
        );
        let (who, pick) = self.script.borrow_mut().pop_front().unwrap_or_else(|| panic!("unscripted order for {player}"));
        assert_eq!(who, player, "CR 616.1's chooser is the entering permanent's controller");
        vec![pick]
    }

    fn pick_number(&self, _: &GameState, _: PlayerId, ctx: &ChoiceContext, _: u64, _: u64) -> u64 {
        panic!("unscripted pick_number: {:?}", ctx.kind)
    }

    fn allocate(&self, _: &GameState, _: PlayerId, ctx: &ChoiceContext, _: u64, _: &[ChoiceOption], _: &[u64], _: Option<&[u64]>) -> Vec<u64> {
        panic!("unscripted allocate: {:?}", ctx.kind)
    }

    fn choose_ordering(&self, _: &GameState, _: PlayerId, ctx: &ChoiceContext, _: &[ChoiceOption]) -> Vec<usize> {
        panic!("unscripted ordering: {:?}", ctx.kind)
    }
}

/// `return_to_battlefield` with a provider that checks who is asked.
fn return_asking(game: &mut GameState, card: Arc<CardData>, owner: PlayerId, orders: &[(PlayerId, usize)]) -> ObjectId {
    let id = put_in_graveyard(game, card, owner);
    let dp = Orders::new(orders);
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("the entry is proposed");
    id
}

/// Empty `player`'s pool, fill it with exactly `pool`, and cast `card` from
/// hand under `ManaWindowStop`, as a shipped client does.
fn cast_from_pool(
    game: &mut GameState,
    player: PlayerId,
    card: Arc<CardData>,
    pool: &[(ManaType, u64)],
    dp: &dyn DecisionProvider,
) -> Result<ObjectId, String> {
    let id = put_in_hand(game, card, player);
    for t in [ManaType::White, ManaType::Blue, ManaType::Black, ManaType::Red, ManaType::Green, ManaType::Colorless] {
        let have = game.players[player].mana_pool.amount(t);
        if have > 0 {
            game.players[player].mana_pool.remove(t, have).unwrap();
        }
    }
    for &(t, n) in pool {
        game.players[player].mana_pool.add(t, n);
    }
    game.cast_spell(player, id, dp).map(|_| id)
}

// RULING: Archelos, Lagoon Mystic #2 - beside an "enters tapped" effect, the entering
//   permanent's controller chooses whether it enters tapped or untapped.
/// Archelos cast from hand, then a land that enters tapped by its own
/// ability. The two statuses are opposite, so the land's controller orders
/// them and the one applied last is the one it enters with: candidates in
/// sweep order, Archelos before the land's own ability (source 1a).
// COVERS-PARTIAL: ATOM-110.5b-002
#[test]
fn archelos_cast_from_hand_makes_a_tapland_its_controllers_order() {
    let mut game = setup_two_player_game();
    let dp = ManaWindowStop::new(Orders::new(&[]));
    let pool = [(ManaType::Black, 1), (ManaType::Green, 1), (ManaType::Blue, 1), (ManaType::Colorless, 1)];
    let archelos = cast_from_pool(&mut game, 0, archelos_lagoon_mystic(), &pool, &dp)
        .expect("Archelos is castable from exactly {1}{B}{G}{U}");
    assert_eq!(game.players[0].mana_pool.total(), 0, "the whole pool was the cost");
    game.resolve_top_of_stack(&dp).expect("Archelos resolves");
    assert!(!game.battlefield[&archelos].tapped);

    let land = put_in_hand(&mut game, idyllic_beachfront(), 0);
    game.play_land(0, land, Zone::Hand, &ActionContext::new(&Orders::new(&[(0, 1)]))).unwrap();
    assert!(!game.battlefield[&land].tapped, "its own ability first, then Archelos");
    let land = return_asking(&mut game, idyllic_beachfront(), 0, &[(0, 0)]);
    assert!(game.battlefield[&land].tapped, "Archelos first, then its own ability");
}

// RULING: Archelos, Lagoon Mystic #1 - its abilities do not apply to itself as it enters.
/// Archelos's first ruling, its own half: its abilities say "other
/// permanents", and an entering permanent's filter-scoped replacements never
/// reach its own entry (CR 614.12). So it enters tapped under an effect
/// that says so, with nothing asked.
#[test]
fn archelos_does_not_apply_to_its_own_entry() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, entry_effect("Permanents enter tapped", ObjectFilter::All, EnterModsTemplate::tapped()), 1);
    let archelos = return_asking(&mut game, archelos_lagoon_mystic(), 0, &[]);
    assert!(game.battlefield[&archelos].tapped);
}

// RULING: Archelos, Lagoon Mystic #1 - nor to permanents entering at the same time as it.
/// Archelos's first ruling, its other half: a permanent returned to the
/// battlefield at the same time as Archelos is decided against the board as
/// it stood, which Archelos is not on yet, so a tapland returned beside it
/// enters tapped by its own ability, with nothing asked.
#[test]
fn archelos_does_not_apply_to_what_enters_beside_it() {
    let mut game = setup_two_player_game();
    let archelos = put_in_graveyard(&mut game, archelos_lagoon_mystic(), 0);
    let land = put_in_graveyard(&mut game, idyllic_beachfront(), 0);
    let returned = |object| GameAction::EnterBattlefield {
        object,
        from: Some(Zone::Graveyard),
        controller: 0,
        mods: EnterMods::NONE,
        cause: Some(ZoneChangeCause::Returned),
    };
    let dp = Orders::new(&[]);
    game.execute_actions(vec![returned(archelos), returned(land)], &ActionContext::new(&dp))
        .expect("both enter");
    assert!(!game.battlefield[&archelos].tapped);
    assert!(game.battlefield[&land].tapped, "decided before Archelos was on the battlefield");
}

// RULING: Archelos, Lagoon Mystic #2 - a permanent simply put onto the battlefield tapped,
//   with no replacement effect, enters untapped while Archelos is untapped.
/// Archelos's second ruling, its other half: a permanent an instruction puts
/// onto the battlefield tapped, with no replacement effect, enters untapped
/// under an untapped Archelos, because the instruction's word is the
/// proposal's starting status and every status-setting effect applies over
/// it. And a tapped Archelos taps whatever enters, any player's.
// COVERS: ATOM-110.5b-003
#[test]
fn archelos_overrides_an_instructions_tapped_and_taps_what_enters_while_tapped() {
    let mut game = setup_two_player_game();
    let archelos = put_on_battlefield(&mut game, archelos_lagoon_mystic(), 0);
    let tapped_token = TokenDef { enters_tapped: true, ..soldier_token() };
    let dp = Orders::new(&[]);
    game.execute_actions(vec![GameAction::CreateTokens { defs: vec![tapped_token], controller: 1 }], &ActionContext::new(&dp))
        .expect("the creation performs");
    let token = *game.battlefield_ids_ordered().last().expect("the token entered");
    assert!(game.objects[&token].is_token);
    assert!(!game.battlefield[&token].tapped, "created tapped, and untapped under Archelos");

    game.execute_action(GameAction::Tap { object: archelos }, &test_ctx()).unwrap();
    let bears = return_asking(&mut game, grizzly_bears(), 1, &[]);
    assert!(game.battlefield[&bears].tapped);
}

// RULING: Archelos, Lagoon Mystic #3 - with more than one Archelos, the entering permanent's
//   controller orders their effects.
/// Archelos's third ruling, on a four-seat board: two of them, one tapped and
/// one untapped, set opposite statuses on a permanent a third player's
/// creature puts onto the battlefield, and that player orders them. Two in
/// the same state agree, and nobody is asked.
#[test]
fn two_archelos_in_opposite_states_ask_the_entering_permanents_controller() {
    for (pick, tapped) in [(0, true), (1, false)] {
        let mut game = setup_game(4);
        put_on_battlefield(&mut game, archelos_lagoon_mystic(), 0);
        let tapped_one = put_on_battlefield(&mut game, archelos_lagoon_mystic(), 1);
        game.execute_action(GameAction::Tap { object: tapped_one }, &test_ctx()).unwrap();
        let bears = return_asking(&mut game, grizzly_bears(), 2, &[(2, pick)]);
        assert_eq!(game.battlefield[&bears].tapped, tapped, "the one applied last");
    }

    let mut game = setup_game(4);
    put_on_battlefield(&mut game, archelos_lagoon_mystic(), 0);
    put_on_battlefield(&mut game, archelos_lagoon_mystic(), 1);
    let bears = return_asking(&mut game, grizzly_bears(), 3, &[]);
    assert!(!game.battlefield[&bears].tapped);
}

// ---------------------------------------------------------------------------
// 5. CR 306.5b gathered, and the order counters make observable (D7, D3 (d))
// ---------------------------------------------------------------------------

/// Kaito, Bane of Nightmares's clause that decides the board: "During your
/// turn, as long as Kaito has one or more loyalty counters on him, he's a 3/4
/// Ninja creature and has hexproof." The fixture drops "during your turn",
/// which holds on a board cast in its controller's main phase, and the
/// hexproof; its ninjutsu and loyalty abilities are not the board.
fn kaito_shaped() -> Arc<CardData> {
    let with_counters = Condition::SourceHasCounters { counter: CounterType::Loyalty, at_least: 1 };
    let ninja_creature = TypeChange {
        set_types: Some(CardTypes::from([CardType::Creature])),
        add_subtypes: vec![Subtype::Creature(CreatureType::Ninja)],
        ..TypeChange::NONE
    };
    CardDataBuilder::new("Kaito-shaped")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Black], 2))
        .supertype(Supertype::Legendary)
        .card_type(CardType::Planeswalker)
        .loyalty(4)
        .ability(static_ability(Effect::Conditional(
            with_counters,
            Box::new(Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::ChangeType(ninja_creature, Duration::WhileSourceOnBattlefield),
                    EffectRecipient::ThisObject,
                ),
                Effect::Atom(
                    Primitive::SetPowerToughness(AmountExpr::Fixed(3), AmountExpr::Fixed(4), Duration::WhileSourceOnBattlefield),
                    EffectRecipient::ThisObject,
                ),
            ])),
        )))
        .build()
}

/// Oath of Gideon's static: "Each planeswalker you control enters with an
/// additional loyalty counter on it." Its token trigger is not the board.
fn oath_of_gideon_shaped() -> Arc<CardData> {
    entry_effect(
        "Oath of Gideon-shaped",
        and(ObjectFilter::ByType(CardType::Planeswalker), ObjectFilter::ByController(PlayerRef::You)),
        EnterModsTemplate::with_counters(CounterType::Loyalty, 1),
    )
}

/// A planeswalker that is a creature before any counters, as Humility's
/// board needs.
fn planeswalker_creature() -> Arc<CardData> {
    CardDataBuilder::new("Planeswalker Creature")
        .card_type(CardType::Creature)
        .card_type(CardType::Planeswalker)
        .power_toughness(2, 2)
        .loyalty(4)
        .build()
}

fn has_intrinsic_entry_ability(game: &GameState, id: ObjectId) -> bool {
    get_effective_abilities(game, id).iter().any(|a| is_intrinsic_entry_ability(a, id))
}

/// A planeswalker has CR 306.5b's ability on its frame; a Kaito that is a
/// creature, with a counter on it, is no planeswalker and has none.
// COVERS-PARTIAL: ATOM-306.5b-001
#[test]
fn a_planeswalker_has_the_intrinsic_ability_and_a_creature_kaito_does_not() {
    let mut game = setup_two_player_game();
    let probe = put_on_battlefield(&mut game, loyalty_probe(), 0);
    assert!(has_intrinsic_entry_ability(&game, probe));
    assert_eq!(game.battlefield[&probe].counter_count(CounterType::Loyalty), 3);

    let kaito = put_on_battlefield(&mut game, kaito_shaped(), 0);
    assert_eq!(game.battlefield[&kaito].counter_count(CounterType::Loyalty), 4, "it entered a planeswalker");
    assert_eq!(get_effective_types(&game, kaito), CardTypes::from([CardType::Creature]));
    assert!(!has_intrinsic_entry_ability(&game, kaito));
}

/// `codebase-state.md` main item 186's printed board: Kaito cast from hand in
/// its controller's main phase beside Oath of Gideon. It would enter with no
/// counters, so it is a planeswalker and both apply, and its type hangs on
/// its counters (the feeds table's last row), so its controller orders them.
/// CR 306.5b first gives 4 and makes it a creature, so Oath no longer
/// applies; Oath first gives 1, and CR 306.5b no longer exists.
// COVERS-PARTIAL: ATOM-616.1-001
#[test]
fn kaito_cast_beside_oath_of_gideon_is_its_controllers_order() {
    for (pick, loyalty) in [(0, 1), (1, 4)] {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, oath_of_gideon_shaped(), 0);
        let pool = [(ManaType::Blue, 1), (ManaType::Black, 1), (ManaType::Colorless, 2)];
        let kaito = cast_from_pool(&mut game, 0, kaito_shaped(), &pool, &ManaWindowStop::new(Orders::new(&[])))
            .expect("castable from exactly {2}{U}{B}");
        assert_eq!(game.players[0].mana_pool.total(), 0, "the whole pool was the cost");
        game.resolve_top_of_stack(&ManaWindowStop::new(Orders::new(&[(0, pick)]))).expect("it resolves");

        assert_eq!(game.battlefield[&kaito].counter_count(CounterType::Loyalty), loyalty);
        assert_eq!(get_effective_types(&game, kaito), CardTypes::from([CardType::Creature]));
        assert_eq!((get_effective_power(&game, kaito), get_effective_toughness(&game, kaito)), (Some(3), Some(4)));
    }
}

/// Item 186's ability-loss half: Humility strips a planeswalker creature's
/// abilities at layer 6, CR 306.5b's with the rest, so it enters with no
/// loyalty and CR 704.5i puts it into its owner's graveyard. Without Humility
/// it enters with its 4.
// COVERS: ATOM-704.5i-001
#[test]
fn humility_takes_the_loyalty_ability_from_a_planeswalker_creature() {
    let mut game = setup_two_player_game();
    let walker = return_asking(&mut game, planeswalker_creature(), 0, &[]);
    assert_eq!(game.battlefield[&walker].counter_count(CounterType::Loyalty), 4);

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, humility(), 1);
    let walker = return_asking(&mut game, planeswalker_creature(), 0, &[]);
    assert_eq!(game.battlefield[&walker].counter_count(CounterType::Loyalty), 0);
    game.check_state_based_actions(&Orders::new(&[])).expect("SBAs");
    assert_eq!(game.objects[&walker].zone, Zone::Graveyard);
}

/// With no multiplier on the board, CR 306.5b's count and Oath's are both
/// constants, so 3 + 1 is 4 in either order and nothing is asked.
#[test]
fn loyalty_beside_an_additional_counter_asks_nothing_without_a_multiplier() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, oath_of_gideon_shaped(), 0);
    let probe = return_asking(&mut game, loyalty_probe(), 0, &[]);
    assert_eq!(game.battlefield[&probe].counter_count(CounterType::Loyalty), 4);
}

/// Premise (d), on registered cards: Adaptive Shimmerer's 3 and Master
/// Biomancer's 2 under Doubling Season. Doubling Season applies only once one
/// of them has written (CR 616.2), so the first choice decides whose counters
/// it can double: 7, 8 or 10, each two questions away.
// COVERS-PARTIAL: ATOM-616.2-001
#[test]
fn shimmerer_under_biomancer_and_doubling_season_reaches_every_order() {
    let mut outcomes = Vec::new();
    for first in 0..2 {
        for second in 0..2 {
            let mut game = setup_two_player_game();
            put_on_battlefield(&mut game, master_biomancer(), 0);
            put_on_battlefield(&mut game, doubling_season(), 0);
            let shimmerer = return_asking(&mut game, adaptive_shimmerer(), 0, &[(0, first), (0, second)]);
            outcomes.push(game.battlefield[&shimmerer].counter_count(CounterType::PlusOnePlusOne));
        }
    }
    outcomes.sort_unstable();
    outcomes.dedup();
    assert_eq!(outcomes, vec![7, 8, 10]);
}
