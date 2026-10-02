//! Phase CV-2b integration tests: CR 707.9's exceptions on an entry copy,
//! with Spark Double, `copy-effects-architecture.md` §4.1a (the phase is §7c).
//!
//! What this file proves, in the order of the section:
//!
//! 1. Spark Double's three exceptions: two CR 707.9f conditions over CR
//!    707.9e counters, checked against the CR 614.12 frame of the copy, and a
//!    CR 707.9b edit that a copy of the copy keeps.
//! 2. Planeswalkers, where the loyalty exception meets CR 306.5b's gathered
//!    ability, a doubler's order, and the Kaito board (§4.1a, "The Kaito
//!    board"; the register rows `lookahead-entry-counters` and
//!    `copy-exception-conditions`).
//! 3. Spark Double's other rulings.
//! 4. The other arms, each on a fixture shaped after the printed card that
//!    uses it: 707.9a's gained ability and keyword, 707.9b's types, P/T and
//!    name, 707.9c's kept color, and 707.9d's derived drop.
//! 5. "Except it enters untapped" in both wordings, and a
//!    later copy taking back a 707.9e status.
//! 6. What the applier refuses.
//!
//! Fixtures are built inline, named for the printed card whose board they
//! stand in for, and never registered. The printed cards were verified on
//! Scryfall on 2026-09-29.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;

use mtgsim::cards::artifacts::{darksteel_myr, sol_ring};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase5_pre_cards::{glorious_anthem, isamaru_hound_of_konda};
use mtgsim::cards::phase_cv_cards::{self, spark_double};
use mtgsim::cards::phase_ld_cards::march_of_the_machines;
use mtgsim::cards::phase_le_cards::{culling_drone, tarmogoyf};
use mtgsim::cards::phase_ll_cards::grist_insect_clause;
use mtgsim::cards::phase_rc_cards::{adaptive_shimmerer, master_biomancer};
use mtgsim::cards::phase_rd_cards::loyalty_probe;
use mtgsim::cards::phase_re_cards::doubling_season;
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::engine::layers::compute_characteristics;
use mtgsim::engine::layers::types::{EffectModification, Layer, PtValue};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::objects::card_data::{AbilityDef, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{
    get_effective_abilities, get_effective_name, get_effective_power, get_effective_toughness,
    get_effective_types, has_keyword, has_subtype,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    fill_library, put_in_graveyard, put_in_hand, put_on_battlefield, registered, setup_two_player_game,
    static_ability, vanilla_creature,
};
use mtgsim::types::card_types::{CardType, CardTypes, CreatureType, PlaneswalkerType, Subtype, Supertype};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AmountExpr, CharacteristicEdit, Characteristic, Condition, CopyException, CounterType, Duration, Effect,
    EffectRecipient, ObjectFilter, ObjectSet, PlayerRef, PlayerSet, Primitive, SelectionFilter, TokenDef,
    TypeChange,
};
use mtgsim::types::ids::{AbilityId, ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::{ManaCost, ManaSymbol, ManaType};
use mtgsim::types::replacement::{
    CopyDonor, EnterMods, EnterModsTemplate, EntryCopyTemplate, EventPattern, ReplacementDef, Rewrite,
};
use mtgsim::types::restriction::{Restriction, RestrictionDef};
use mtgsim::types::zones::{Zone, ZoneChangeCause};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::DecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// The provider: every prompt scripted, with who is asked
// ---------------------------------------------------------------------------

/// What a scripted prompt answers: the objects to pick, found by id, or one
/// option by its place, for a CR 616.1 prompt between two effects of one
/// source.
enum Pick {
    Ids(Vec<ObjectId>),
    Index(usize),
}

/// Answers each prompt from a script, in order: who must be asked, the kind's
/// name, and the pick. An unscripted prompt panics, and so does a script left
/// over.
struct Scripted {
    script: RefCell<VecDeque<(PlayerId, &'static str, Pick)>>,
    /// Every prompt asked: who, the kind's name, and the objects offered.
    asked: RefCell<Vec<(PlayerId, String, Vec<ObjectId>)>>,
}

impl Scripted {
    fn new() -> Self {
        Scripted { script: RefCell::new(VecDeque::new()), asked: RefCell::new(Vec::new()) }
    }

    /// Expect `player` to be asked a `kind` prompt next, and pick `ids`.
    fn then(self, player: PlayerId, kind: &'static str, ids: &[ObjectId]) -> Self {
        self.script.borrow_mut().push_back((player, kind, Pick::Ids(ids.to_vec())));
        self
    }

    /// Expect `player` to be asked a `kind` prompt next, and pick option
    /// `index`.
    fn then_index(self, player: PlayerId, kind: &'static str, index: usize) -> Self {
        self.script.borrow_mut().push_back((player, kind, Pick::Index(index)));
        self
    }

    fn asked(&self) -> Vec<(PlayerId, String, Vec<ObjectId>)> {
        self.asked.borrow().clone()
    }
}

impl Drop for Scripted {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(self.script.borrow().is_empty(), "scripted prompts never asked");
        }
    }
}

fn variant_name(kind: &ChoiceKind) -> String {
    format!("{kind:?}").split([' ', '{', '(']).next().unwrap_or("").to_string()
}

impl DecisionProvider for Scripted {
    fn pick_n(
        &self,
        _game: &GameState,
        player: PlayerId,
        ctx: &ChoiceContext,
        options: &[ChoiceOption],
        _bounds: (usize, usize),
    ) -> Vec<usize> {
        let kind = variant_name(&ctx.kind);
        let offered: Vec<ObjectId> = options
            .iter()
            .filter_map(|o| match o {
                ChoiceOption::Object(id) => Some(*id),
                _ => None,
            })
            .collect();
        self.asked.borrow_mut().push((player, kind.clone(), offered.clone()));
        let (who, expected, pick) = self
            .script
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| panic!("unscripted prompt: {:?} to player {player}", ctx.kind));
        assert_eq!((who, expected), (player, kind.as_str()), "the wrong prompt, or the wrong player asked");
        match pick {
            Pick::Index(index) => vec![index],
            Pick::Ids(ids) => ids
                .iter()
                .map(|id| offered.iter().position(|o| o == id).unwrap_or_else(|| panic!("{id} not offered: {offered:?}")))
                .collect(),
        }
    }

    fn pick_number(&self, _: &GameState, _: PlayerId, ctx: &ChoiceContext, _: u64, _: u64) -> u64 {
        panic!("unscripted pick_number: {:?}", ctx.kind)
    }

    fn allocate(
        &self,
        _: &GameState,
        _: PlayerId,
        ctx: &ChoiceContext,
        _: u64,
        _: &[ChoiceOption],
        _: &[u64],
        _: Option<&[u64]>,
    ) -> Vec<u64> {
        panic!("unscripted allocate: {:?}", ctx.kind)
    }

    fn choose_ordering(&self, _: &GameState, _: PlayerId, ctx: &ChoiceContext, _: &[ChoiceOption]) -> Vec<usize> {
        panic!("unscripted ordering: {:?}", ctx.kind)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return `id`, a card in a graveyard, to the battlefield through the
/// replacement pipeline, as "return target creature card" does.
fn try_enter(game: &mut GameState, id: ObjectId, dp: &dyn DecisionProvider) -> Result<(), String> {
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(dp))
}

/// Put `card` in `owner`'s graveyard and return it to the battlefield.
fn enter(game: &mut GameState, card: Arc<CardData>, owner: PlayerId, dp: &dyn DecisionProvider) -> ObjectId {
    let id = put_in_graveyard(game, card, owner);
    try_enter(game, id, dp).expect("the entry is proposed");
    id
}

/// `card` entering under `player` as a copy of `donor`, the only prompt.
fn copy_of(game: &mut GameState, card: Arc<CardData>, player: PlayerId, donor: ObjectId) -> ObjectId {
    let dp = Scripted::new().then(player, "ChooseCopySource", &[donor]);
    enter(game, card, player, &dp)
}

fn spark_of(game: &mut GameState, player: PlayerId, donor: ObjectId) -> ObjectId {
    copy_of(game, spark_double(), player, donor)
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

/// Spark Double's exact cost, `{3}{U}`.
const SPARK_DOUBLE_COST: [(ManaType, u64); 2] = [(ManaType::Blue, 1), (ManaType::Colorless, 3)];

/// State-based actions and triggers, then the stack, until both are quiet.
fn settle(game: &mut GameState, dp: &dyn DecisionProvider) {
    for _ in 0..20 {
        game.perform_sba_and_triggers(dp).expect("SBAs and triggers");
        if game.stack.is_empty() {
            return;
        }
        game.resolve_top_of_stack(dp).expect("resolving");
    }
    panic!("the stack never emptied");
}

fn counters(game: &GameState, id: ObjectId, kind: CounterType) -> u32 {
    game.battlefield.get(&id).map_or(0, |e| e.counter_count(kind))
}

fn zone_of(game: &GameState, id: ObjectId) -> Zone {
    game.get_object(id).expect("in the store").zone
}

fn pt(game: &GameState, id: ObjectId) -> (Option<i32>, Option<i32>) {
    (get_effective_power(game, id), get_effective_toughness(game, id))
}

fn is_legendary(game: &GameState, id: ObjectId) -> bool {
    compute_characteristics(game, id).expect("in the store").supertypes.contains(&Supertype::Legendary)
}

/// A row on `id` alone, registered as a resolution would: an animation, a
/// pump, a color change, none of them copiable.
fn add_row(game: &mut GameState, id: ObjectId, layer: Layer, modification: EffectModification) {
    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(registered(id, layer, timestamp, modification));
}

fn and(a: ObjectFilter, b: ObjectFilter) -> ObjectFilter {
    ObjectFilter::And(Box::new(a), Box::new(b))
}

fn creatures_you_control() -> ObjectFilter {
    and(ObjectFilter::ByType(CardType::Creature), ObjectFilter::ByController(PlayerRef::You))
}

fn enters_with(template: EnterModsTemplate) -> AbilityDef {
    static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
        EventPattern::EnterBattlefield { cast: None },
        ObjectSet::SourceOnly,
        Rewrite::EnterWith(template),
    ))))
}

/// A creature that "may enter as a copy of any creature on the battlefield,
/// except ...": Clone's replacement with `except`.
fn copier(name: &str, color: Color, except: Vec<CopyException>) -> Arc<CardData> {
    copier_of(name, color, SelectionFilter::Creature, except)
}

fn copier_of(name: &str, color: Color, donor: SelectionFilter, except: Vec<CopyException>) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Shapeshifter))
        .color(color)
        .mana_cost(ManaCost::build(&[ManaType::Blue], 3))
        .power_toughness(0, 0)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef {
            optional: true,
            ..ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                ObjectSet::SourceOnly,
                Rewrite::EnterAsCopy(EntryCopyTemplate { donor: CopyDonor::Chosen(donor), except }),
            )
        }))))
        .build()
}

fn adding(change: TypeChange) -> CopyException {
    CopyException::Modifies(CharacteristicEdit::Types(change))
}

fn with_subtypes(subtypes: &[CreatureType]) -> TypeChange {
    TypeChange { add_subtypes: subtypes.iter().map(|t| Subtype::Creature(*t)).collect(), ..TypeChange::NONE }
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// Kaito, Bane of Nightmares' clause that decides the board: "During your
/// turn, as long as Kaito has one or more loyalty counters on him, he's a 3/4
/// Ninja creature and has hexproof." The fixture drops "during your turn",
/// which holds on a board cast in its controller's main phase, and the
/// hexproof: RG's fixture, whose seventh ruling makes him no planeswalker
/// while he is a creature.
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
    CardDataBuilder::new("Oath of Gideon-shaped")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .supertype(Supertype::Legendary)
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::battlefield_filter(and(
                ObjectFilter::ByType(CardType::Planeswalker),
                ObjectFilter::ByController(PlayerRef::You),
            )),
            Rewrite::EnterWith(EnterModsTemplate::with_counters(CounterType::Loyalty, 1)),
        )))))
        .build()
}

/// A Gideon planeswalker, loyalty 6, whose "becomes a 6/6 creature that's
/// still a planeswalker" is a resolution's, added by the test: Gideon Jura's
/// zero ability.
fn gideon_jura_shaped() -> Arc<CardData> {
    CardDataBuilder::new("Gideon Jura-shaped")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 3))
        .color(Color::White)
        .supertype(Supertype::Legendary)
        .card_type(CardType::Planeswalker)
        .subtype(Subtype::Planeswalker(PlaneswalkerType::Gideon))
        .loyalty(6)
        .build()
}

/// Gideon Blackblade's first ability without "during your turn": "Gideon
/// Blackblade is a 4/4 Human Soldier creature with indestructible that's
/// still a planeswalker." A static of his own, so a copy has it.
fn gideon_blackblade_shaped() -> Arc<CardData> {
    let creature = TypeChange {
        add_types: vec![CardType::Creature],
        add_subtypes: vec![Subtype::Creature(CreatureType::Human), Subtype::Creature(CreatureType::Soldier)],
        ..TypeChange::NONE
    };
    CardDataBuilder::new("Gideon Blackblade-shaped")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 1))
        .color(Color::White)
        .supertype(Supertype::Legendary)
        .card_type(CardType::Planeswalker)
        .subtype(Subtype::Planeswalker(PlaneswalkerType::Gideon))
        .loyalty(4)
        .ability(static_ability(Effect::Sequence(vec![
            Effect::Atom(Primitive::ChangeType(creature, Duration::WhileSourceOnBattlefield), EffectRecipient::ThisObject),
            Effect::Atom(
                Primitive::SetPowerToughness(AmountExpr::Fixed(4), AmountExpr::Fixed(4), Duration::WhileSourceOnBattlefield),
                EffectRecipient::ThisObject,
            ),
        ])))
        .build()
}

/// Wall of Omens — {1}{W} 0/4 Wall, defender, "When this creature enters,
/// draw a card."
fn wall_of_omens() -> Arc<CardData> {
    use mtgsim::cards::authoring::{enters, triggered_ability, whenever};
    use mtgsim::types::triggers::TriggerSubject;
    let draw = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller);
    CardDataBuilder::new("Wall of Omens")
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Wall))
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .power_toughness(0, 4)
        .keyword_flag(KeywordFlag::Defender)
        .ability(triggered_ability("", whenever(enters(TriggerSubject::ThisObject), draw)))
        .build()
}

/// Rusted Sentinel — {4}, Artifact Creature — Golem 3/4, "This creature
/// enters tapped."
fn rusted_sentinel() -> Arc<CardData> {
    CardDataBuilder::new("Rusted Sentinel")
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Golem))
        .mana_cost(ManaCost::build(&[], 4))
        .power_toughness(3, 4)
        .ability(enters_with(EnterModsTemplate::tapped()))
        .build()
}

/// A creature whose subtypes a characteristic-defining ability sets: "This
/// creature is a Sliver." The stand-in for changeling, which is unbuilt.
fn sliver_by_cda() -> Arc<CardData> {
    CardDataBuilder::new("Sliver by CDA")
        .card_type(CardType::Creature)
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .power_toughness(2, 2)
        .ability(AbilityDef {
            is_characteristic_defining: true,
            ..static_ability(Effect::Atom(
                Primitive::ChangeType(with_subtypes(&[CreatureType::Sliver]), Duration::WhileSourceOnBattlefield),
                EffectRecipient::ThisObject,
            ))
        })
        .build()
}

/// "Creatures you control can't have +1/+1 counters put on them": the shape
/// of Melira's "-1/-1", on the other kind.
fn no_plus_counters_on_your_creatures() -> Arc<CardData> {
    CardDataBuilder::new("No +1/+1 counters")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Restriction(Box::new(RestrictionDef::new(Restriction::Event {
            pattern: EventPattern::AddCounters { counter: Some(CounterType::PlusOnePlusOne), by: None },
            affected_objects: ObjectSet::battlefield_filter(creatures_you_control()),
            affected_players: PlayerSet::Nobody,
            by: None,
        })))))
        .build()
}

/// A resolution's "until end of turn" animation: `id` becomes a `p`/`t`
/// creature in addition to its other types.
fn animate(game: &mut GameState, id: ObjectId, p: i32, t: i32) {
    add_row(game, id, Layer::Layer4Type, EffectModification::AddType(CardType::Creature));
    add_row(game, id, Layer::Layer7bSetPT, EffectModification::SetPowerToughness {
        power: PtValue::Fixed(p),
        toughness: PtValue::Fixed(t),
    });
}

// ---------------------------------------------------------------------------
// 1. Spark Double's exceptions
// ---------------------------------------------------------------------------

/// Spark Double cast from hand, from exactly `{3}{U}`, copies a legendary
/// creature its controller controls. It is a creature, so its 707.9f
/// condition holds and it enters with the additional +1/+1 counter; it is not
/// legendary (707.9b), so the legend rule leaves both. The donor is the only
/// question.
// COVERS: ATOM-707.9f-002
#[test]
fn spark_double_cast_from_hand_copies_a_legend_with_a_counter_and_no_legend_rule() {
    let mut game = setup_two_player_game();
    let isamaru = put_on_battlefield(&mut game, isamaru_hound_of_konda(), 0);
    let dp = ManaWindowStop::new(Scripted::new());
    let spark = cast_from_pool(&mut game, 0, spark_double(), &SPARK_DOUBLE_COST, &dp).expect("castable from exactly {3}{U}");
    assert_eq!(game.players[0].mana_pool.total(), 0, "the whole pool was the cost");

    let dp = ManaWindowStop::new(Scripted::new().then(0, "ChooseCopySource", &[isamaru]));
    game.resolve_top_of_stack(&dp).expect("Spark Double resolves");
    assert_eq!(dp.inner().asked(), vec![(0, "ChooseCopySource".to_string(), vec![isamaru])]);

    assert_eq!(get_effective_name(&game, spark), "Isamaru, Hound of Konda");
    assert_eq!(counters(&game, spark, CounterType::PlusOnePlusOne), 1);
    assert_eq!(counters(&game, spark, CounterType::Loyalty), 0);
    assert_eq!(pt(&game, spark), (Some(3), Some(3)));
    assert!(!is_legendary(&game, spark));
    assert!(is_legendary(&game, isamaru));
    settle(&mut game, &Scripted::new());
    assert_eq!((zone_of(&game, spark), zone_of(&game, isamaru)), (Zone::Battlefield, Zone::Battlefield));
}

// RULING: Spark Double #2 - "Spark Double isn't legendary if it copies a legendary
//   permanent, and this exception is copiable."
/// The 707.9b edit is part of the copiable values, so a Clone of the Spark
/// Double copy is not legendary either, and three permanents named Isamaru,
/// one of them legendary, meet no legend rule.
#[test]
fn a_clone_of_spark_double_is_not_legendary_either() {
    let mut game = setup_two_player_game();
    let isamaru = put_on_battlefield(&mut game, isamaru_hound_of_konda(), 0);
    let spark = spark_of(&mut game, 0, isamaru);
    let clone = copy_of(&mut game, phase_cv_cards::clone(), 0, spark);

    assert_eq!(get_effective_name(&game, clone), "Isamaru, Hound of Konda");
    assert!(!is_legendary(&game, clone));
    assert_eq!(counters(&game, clone, CounterType::PlusOnePlusOne), 0, "the counter was an entry's, not a value");
    settle(&mut game, &Scripted::new());
    for id in [isamaru, spark, clone] {
        assert_eq!(zone_of(&game, id), Zone::Battlefield);
    }
}

/// CR 707.9f's negative case: an artifact a resolution animated is a
/// creature Spark Double may choose, and the copy is the noncreature artifact
/// its values say. So the condition fails and it gets no counter.
// COVERS: ATOM-707.9f-001
#[test]
fn spark_double_copying_an_animated_artifact_gets_no_counter() {
    let mut game = setup_two_player_game();
    let myr = put_on_battlefield(&mut game, sol_ring(), 0);
    animate(&mut game, myr, 3, 3);
    assert!(get_effective_types(&game, myr).contains(&CardType::Creature));

    let spark = spark_of(&mut game, 0, myr);
    assert_eq!(get_effective_types(&game, spark), CardTypes::from([CardType::Artifact]));
    assert_eq!(counters(&game, spark, CounterType::PlusOnePlusOne), 0);
    assert_eq!(get_effective_name(&game, spark), "Sol Ring");
}

/// CR 707.9e: Spark Double copies a Clone that copied nothing (Glorious
/// Anthem keeps it alive). As a creature it gets the +1/+1 counter, and then
/// the Clone's copied ability, applied as the same entry goes on, copies a
/// Bear: "the exception's effect doesn't happen", and it enters as a Bear
/// with no counter.
// COVERS: ATOM-707.9e-001
#[test]
fn a_later_copy_takes_back_spark_doubles_counter() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, glorious_anthem(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let empty_clone = enter(&mut game, phase_cv_cards::clone(), 0, &Scripted::new().then(0, "ChooseCopySource", &[]));
    assert_eq!(get_effective_name(&game, empty_clone), "Clone");

    let dp = Scripted::new().then(0, "ChooseCopySource", &[empty_clone]).then(0, "ChooseCopySource", &[bears]);
    let spark = enter(&mut game, spark_double(), 0, &dp);
    assert_eq!(get_effective_name(&game, spark), "Grizzly Bears");
    assert_eq!(counters(&game, spark, CounterType::PlusOnePlusOne), 0);
    assert_eq!(pt(&game, spark), (Some(3), Some(3)), "2/2 and the Anthem");

    // Declining the Clone's ability keeps the first copy and its counter.
    let dp = Scripted::new().then(0, "ChooseCopySource", &[empty_clone]).then(0, "ChooseCopySource", &[]);
    let spark = enter(&mut game, spark_double(), 0, &dp);
    assert_eq!(get_effective_name(&game, spark), "Clone");
    assert_eq!(counters(&game, spark, CounterType::PlusOnePlusOne), 1);
}

/// Declined, Spark Double enters as the 0/0 Illusion its card is, and CR
/// 704.5f takes it.
#[test]
fn a_spark_double_that_copies_nothing_dies() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    let spark = enter(&mut game, spark_double(), 0, &Scripted::new().then(0, "ChooseCopySource", &[]));
    assert!(has_subtype(&game, spark, &Subtype::Creature(CreatureType::Illusion)));
    assert_eq!(counters(&game, spark, CounterType::PlusOnePlusOne), 0);
    settle(&mut game, &Scripted::new());
    assert_eq!(zone_of(&game, spark), Zone::Graveyard);
}

/// "A creature or planeswalker you control": an opponent's creature is not a
/// candidate, and an enchantment is not either.
#[test]
fn spark_double_offers_only_creatures_and_planeswalkers_you_control() {
    let mut game = setup_two_player_game();
    let mine = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let walker = put_on_battlefield(&mut game, loyalty_probe(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    put_on_battlefield(&mut game, glorious_anthem(), 0);
    let dp = Scripted::new().then(0, "ChooseCopySource", &[mine]);
    enter(&mut game, spark_double(), 0, &dp);
    let offered = dp.asked()[0].2.clone();
    assert_eq!(offered.len(), 2);
    assert!(offered.contains(&mine) && offered.contains(&walker));
}

// ---------------------------------------------------------------------------
// 2. Planeswalkers
// ---------------------------------------------------------------------------

/// A copied planeswalker gets its printed loyalty and one more: the copy's
/// loyalty exception at CR 616.1c, then CR 306.5b's ability off the copy's
/// frame. Not a creature, so no +1/+1; Grist's clause is legendary and the
/// copy is not.
#[test]
fn spark_double_copying_a_planeswalker_gets_printed_loyalty_plus_one() {
    for card in [loyalty_probe(), grist_insect_clause()] {
        let mut game = setup_two_player_game();
        let walker = put_on_battlefield(&mut game, card, 0);
        assert_eq!(counters(&game, walker, CounterType::Loyalty), 3);
        let spark = spark_of(&mut game, 0, walker);
        assert_eq!(counters(&game, spark, CounterType::Loyalty), 4);
        assert_eq!(counters(&game, spark, CounterType::PlusOnePlusOne), 0);
        assert_eq!(get_effective_types(&game, spark), CardTypes::from([CardType::Planeswalker]));
        assert!(!is_legendary(&game, spark));
    }
}

/// Beside Doubling Season the loyalty is an order. Once the copy's counter is
/// on the entry the doubler applies (CR 616.2), beside CR 306.5b's three:
/// 306.5b first reaches (1 + 3) × 2 = 8, Doubling Season first 1 × 2 + 3 = 5.
#[test]
fn beside_doubling_season_a_copied_planeswalkers_loyalty_is_an_order() {
    for (loyalty_first, expected) in [(true, 8), (false, 5)] {
        let mut game = setup_two_player_game();
        let season = put_on_battlefield(&mut game, doubling_season(), 0);
        let probe = put_on_battlefield(&mut game, loyalty_probe(), 0);
        let spark = put_in_graveyard(&mut game, spark_double(), 0);
        let first = if loyalty_first { spark } else { season };
        let dp = Scripted::new()
            .then(0, "ChooseCopySource", &[probe])
            .then(0, "ChooseReplacementEffect", &[first]);
        try_enter(&mut game, spark, &dp).expect("the entry is proposed");
        assert_eq!(counters(&game, spark, CounterType::Loyalty), expected);
    }
}

// RULING: Spark Double #8 - "Spark Double enters as a noncreature planeswalker and
//   doesn't get a +1/+1 counter."
/// Spark Double's eighth ruling: a Gideon that a resolution made a creature
/// is copied as the noncreature planeswalker its values say, with no +1/+1
/// counter and its printed 6 plus one.
#[test]
fn spark_double_copying_an_animated_gideon_is_a_noncreature_planeswalker() {
    let mut game = setup_two_player_game();
    let gideon = put_on_battlefield(&mut game, gideon_jura_shaped(), 0);
    animate(&mut game, gideon, 6, 6);
    assert!(get_effective_types(&game, gideon).contains(&CardType::Creature));

    let spark = spark_of(&mut game, 0, gideon);
    assert_eq!(get_effective_types(&game, spark), CardTypes::from([CardType::Planeswalker]));
    assert_eq!(counters(&game, spark, CounterType::PlusOnePlusOne), 0);
    assert_eq!(counters(&game, spark, CounterType::Loyalty), 7);
}

// RULING: Spark Double #8 - "if Spark Double copies Gideon Blackblade during your turn,
//   Spark Double enters as a planeswalker creature and gets both kinds of counters."
/// The eighth ruling's other half: Gideon Blackblade on your turn is a
/// planeswalker creature by his own static, which the copy has, so Spark
/// Double gets both counters and CR 306.5b's 4: a 5/5 with 5 loyalty.
#[test]
fn spark_double_copying_a_planeswalker_creature_gets_both_counters() {
    let mut game = setup_two_player_game();
    let gideon = put_on_battlefield(&mut game, gideon_blackblade_shaped(), 0);
    let spark = spark_of(&mut game, 0, gideon);
    let types = get_effective_types(&game, spark);
    assert!(types.contains(&CardType::Creature) && types.contains(&CardType::Planeswalker));
    assert_eq!(counters(&game, spark, CounterType::PlusOnePlusOne), 1);
    assert_eq!(counters(&game, spark, CounterType::Loyalty), 5);
    assert_eq!(pt(&game, spark), (Some(5), Some(5)));
}

// RULING-DEVIATION: Spark Double #7 (lookahead-entry-counters) - "printed on the card
//   plus one": 5 on this board by Arixmethes's look-ahead, 1 by CR 614.12's text.
/// The Kaito board (`copy-effects-architecture.md` §4.1a). Spark Double, cast
/// on its controller's turn, copies Kaito, a creature because he has
/// counters. With no counters the copy is a planeswalker, so the loyalty
/// exception, checked without itself, applies. Checked without itself, the
/// +1/+1 exception sees that loyalty counter and a creature, so it applies
/// too (`copy-exception-conditions`). With its counters on it, the frame is a
/// creature and no planeswalker, so CR 306.5b's ability is not there to
/// gather and Oath of Gideon does not match: 1 loyalty and one +1/+1, a 4/5,
/// beside Oath or not, and no question but the donor. Spark Double's seventh
/// ruling reads 5 here, by Arixmethes's look-ahead
/// (`lookahead-entry-counters`).
#[test]
fn spark_double_copying_kaito_gets_one_loyalty_counter_and_a_plus_one() {
    for beside_oath in [false, true] {
        let mut game = setup_two_player_game();
        if beside_oath {
            put_on_battlefield(&mut game, oath_of_gideon_shaped(), 0);
        }
        let kaito = put_on_battlefield(&mut game, kaito_shaped(), 0);
        assert_eq!(get_effective_types(&game, kaito), CardTypes::from([CardType::Creature]));

        let dp = ManaWindowStop::new(Scripted::new());
        let spark = cast_from_pool(&mut game, 0, spark_double(), &SPARK_DOUBLE_COST, &dp).expect("castable from exactly {3}{U}");
        let dp = ManaWindowStop::new(Scripted::new().then(0, "ChooseCopySource", &[kaito]));
        game.resolve_top_of_stack(&dp).expect("Spark Double resolves");

        assert_eq!(dp.inner().asked(), vec![(0, "ChooseCopySource".to_string(), vec![kaito])]);
        assert_eq!(counters(&game, spark, CounterType::Loyalty), 1);
        assert_eq!(counters(&game, spark, CounterType::PlusOnePlusOne), 1);
        assert_eq!(get_effective_types(&game, spark), CardTypes::from([CardType::Creature]));
        assert_eq!(pt(&game, spark), (Some(4), Some(5)));
        assert!(!is_legendary(&game, spark));
    }
}

// ---------------------------------------------------------------------------
// 3. Spark Double's other rulings
// ---------------------------------------------------------------------------

// RULING: Spark Double #1 - "It doesn't copy whether that permanent is tapped or untapped,
//   whether it has any counters on it ..., or any non-copy effects."
/// The donor's three +1/+1 counters, its tapped status, a pump and a color
/// change are not copiable: Spark Double is an untapped 2/2 green creature
/// with its own one counter.
#[test]
fn spark_double_copies_only_what_was_printed() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    game.add_counters(bears, CounterType::PlusOnePlusOne, 3);
    game.battlefield.get_mut(&bears).unwrap().tapped = true;
    add_row(&mut game, bears, Layer::Layer7cModifyPT, EffectModification::ModifyPowerToughness {
        power: PtValue::Fixed(2),
        toughness: PtValue::Fixed(2),
    });
    add_row(&mut game, bears, Layer::Layer5Color, EffectModification::SetColors([Color::Black].into_iter().collect()));

    let spark = spark_of(&mut game, 0, bears);
    assert_eq!(counters(&game, spark, CounterType::PlusOnePlusOne), 1);
    assert_eq!(pt(&game, spark), (Some(3), Some(3)));
    assert!(!game.battlefield[&spark].tapped);
    assert_eq!(compute_characteristics(&game, spark).unwrap().colors, [Color::Green].into_iter().collect());
}

// RULING: Spark Double #3 - "If the copied permanent has {X} in its mana cost, X is
//   considered to be 0."
/// The copy's cost keeps its {X}, and X is 0 in its mana value.
#[test]
fn spark_double_copying_an_x_creature_has_x_as_zero() {
    let mut game = setup_two_player_game();
    let x_cost = ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Green)]);
    let donor = put_on_battlefield(
        &mut game,
        CardDataBuilder::new("X Creature").card_type(CardType::Creature).mana_cost(x_cost.clone()).power_toughness(2, 2).build(),
        0,
    );
    let spark = spark_of(&mut game, 0, donor);
    let cost = compute_characteristics(&game, spark).unwrap().mana_cost.clone().expect("the copied cost");
    assert_eq!(cost, x_cost);
    assert_eq!(cost.mana_value(), 1);
}

// RULING: Spark Double #4 - "If the chosen permanent is copying something else ...,
//   then Spark Double enters the battlefield as whatever the chosen permanent copied."
/// Spark Double copying a Clone that copied Grizzly Bears enters as Grizzly
/// Bears, with its counter.
#[test]
fn spark_double_copying_a_clone_enters_as_what_the_clone_copied() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let clone = copy_of(&mut game, phase_cv_cards::clone(), 0, bears);
    let spark = spark_of(&mut game, 0, clone);
    assert_eq!(get_effective_name(&game, spark), "Grizzly Bears");
    assert!(!has_subtype(&game, spark, &Subtype::Creature(CreatureType::Shapeshifter)));
    assert_eq!(pt(&game, spark), (Some(3), Some(3)));
}

// RULING: Spark Double #5 - "Spark Double copies the original characteristics of that
//   token ... Spark Double doesn't become a token in this case."
/// A token's copiable values are what its creating effect stated, and Spark
/// Double stays a card.
#[test]
fn spark_double_copying_a_token_is_not_a_token() {
    let mut game = setup_two_player_game();
    let soldier = TokenDef {
        name: None,
        colors: vec![Color::White],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(CreatureType::Soldier)],
        supertypes: Vec::new(),
        power: Some(1),
        toughness: Some(1),
        keyword_flags: Vec::new(),
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
    };
    let source = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let create = Effect::Atom(Primitive::CreateToken(soldier, AmountExpr::Fixed(1)), EffectRecipient::Controller);
    game.resolve_effect(&create, &ResolutionContext::untargeted(source, 0), &Scripted::new()).expect("created");
    let token = game.battlefield_ids_ordered().into_iter().find(|id| game.get_object(*id).unwrap().is_token).unwrap();

    let spark = spark_of(&mut game, 0, token);
    assert_eq!(get_effective_name(&game, spark), "Soldier Token");
    assert_eq!(pt(&game, spark), (Some(2), Some(2)));
    assert!(!game.get_object(spark).unwrap().is_token);
}

// RULING: Spark Double #6 - "Any enters-the-battlefield abilities of the copied permanent
//   will trigger ... 'enters the battlefield with' abilities ... will also work."
/// CR 707.5: the copied "enters with three +1/+1 counters" applies to the
/// same entry, after the copy and its own counter, and a copied "when this
/// enters" trigger fires for Spark Double.
#[test]
fn spark_double_brings_the_copied_enters_abilities() {
    let mut game = setup_two_player_game();
    let shimmerer = enter(&mut game, adaptive_shimmerer(), 0, &Scripted::new());
    assert_eq!(counters(&game, shimmerer, CounterType::PlusOnePlusOne), 3);
    let spark = spark_of(&mut game, 0, shimmerer);
    assert_eq!(counters(&game, spark, CounterType::PlusOnePlusOne), 4);

    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    fill_library(&mut game, 1, 5);
    let wall = put_on_battlefield(&mut game, wall_of_omens(), 0);
    // The Wall's own entry triggered for the same player; resolve it first.
    settle(&mut game, &Scripted::new());
    let hand = game.players[0].hand.len();
    spark_of(&mut game, 0, wall);
    settle(&mut game, &Scripted::new());
    assert_eq!(game.players[0].hand.len(), hand + 1, "the copied trigger drew a card");
}

/// Ruling #7's creature half: Spark Double's one counter, "plus any counters
/// that will be put on it from abilities it copied and other abilities of
/// other objects": Adaptive Shimmerer's three and Master Biomancer's two.
#[test]
fn spark_double_gets_its_counter_beside_copied_and_other_entry_counters() {
    let mut game = setup_two_player_game();
    let shimmerer = enter(&mut game, adaptive_shimmerer(), 0, &Scripted::new());
    put_on_battlefield(&mut game, master_biomancer(), 0);
    let spark = spark_of(&mut game, 0, shimmerer);
    assert_eq!(counters(&game, spark, CounterType::PlusOnePlusOne), 6);
    assert!(has_subtype(&game, spark, &Subtype::Creature(CreatureType::Mutant)));
}

// RULING: Spark Double #8 - "Use the characteristics of Spark Double as it enters the
//   battlefield, not of the copied permanent."
/// March of the Machines makes a copied Sol Ring an artifact creature as it
/// enters, so the CR 614.12 frame the condition reads is a creature, and
/// Spark Double gets the counter: a 2/2.
#[test]
fn spark_double_copying_sol_ring_under_march_of_the_machines_gets_the_counter() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, march_of_the_machines(), 1);
    let ring = put_on_battlefield(&mut game, sol_ring(), 0);
    assert!(get_effective_types(&game, ring).contains(&CardType::Creature));
    let spark = spark_of(&mut game, 0, ring);
    assert_eq!(counters(&game, spark, CounterType::PlusOnePlusOne), 1);
    assert_eq!(pt(&game, spark), (Some(2), Some(2)));
}

// RULING: Spark Double #9 - "You may choose only a creature or planeswalker that's
//   already on the battlefield."
/// Spark Double and a Bear entering as one event: the Bear is not offered,
/// since each entry is decided against the board before the batch performs.
#[test]
fn a_creature_entering_beside_spark_double_is_not_a_candidate() {
    let mut game = setup_two_player_game();
    let donor = put_on_battlefield(&mut game, vanilla_creature(4, 4, &[]), 0);
    let spark = put_in_graveyard(&mut game, spark_double(), 0);
    let bears = put_in_graveyard(&mut game, grizzly_bears(), 0);
    let entry = |id| GameAction::EnterBattlefield {
        object: id,
        from: Some(Zone::Graveyard),
        controller: 0,
        mods: EnterMods::NONE,
        cause: Some(ZoneChangeCause::Returned),
    };
    let dp = Scripted::new().then(0, "ChooseCopySource", &[donor]);
    game.execute_actions(vec![entry(spark), entry(bears)], &ActionContext::new(&dp)).expect("both enter");
    assert_eq!(dp.asked()[0].2, vec![donor]);
    assert!(game.battlefield.contains_key(&bears));
    assert_eq!(pt(&game, spark), (Some(5), Some(5)));
}

// ---------------------------------------------------------------------------
// 4. The other arms
// ---------------------------------------------------------------------------

/// Copy Artifact — "You may have this enchantment enter as a copy of any
/// artifact on the battlefield, except it's an enchantment in addition to
/// its other types."
fn copy_artifact_shaped() -> Arc<CardData> {
    CardDataBuilder::new("Copy Artifact-shaped")
        .card_type(CardType::Enchantment)
        .color(Color::Blue)
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef {
            optional: true,
            ..ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                ObjectSet::SourceOnly,
                Rewrite::EnterAsCopy(EntryCopyTemplate {
                    donor: CopyDonor::Chosen(SelectionFilter::Permanent(ObjectFilter::ByType(CardType::Artifact))),
                    except: vec![adding(TypeChange { add_types: vec![CardType::Enchantment], ..TypeChange::NONE })],
                }),
            )
        }))))
        .build()
}

/// CR 707.9b's example: the copy of Darksteel Myr is an artifact creature
/// enchantment, and those are its copiable types, so a Clone of it is all
/// three as well.
// COVERS: ATOM-707.9b-001
#[test]
fn a_copy_artifact_of_a_myr_is_an_enchantment_and_so_is_a_clone_of_it() {
    let mut game = setup_two_player_game();
    let myr = put_on_battlefield(&mut game, darksteel_myr(), 1);
    let copy = copy_of(&mut game, copy_artifact_shaped(), 0, myr);
    let all_three = CardTypes::from([CardType::Artifact, CardType::Creature, CardType::Enchantment]);
    assert_eq!(get_effective_types(&game, copy), all_three);
    assert!(has_keyword(&game, copy, KeywordFlag::Indestructible));

    let clone = copy_of(&mut game, phase_cv_cards::clone(), 0, copy);
    assert_eq!(get_effective_types(&game, clone), all_three);
}

/// Quicksilver Gargantuan — "... except it's 7/7." CR 707.9d's example: the
/// copy of Tarmogoyf is 7/7 whatever the graveyards hold, because the P/T it
/// provides drops Tarmogoyf's characteristic-defining ability.
// COVERS: ATOM-707.9d-001
// COVERS-PARTIAL: COMP-9A-006
#[test]
fn a_gargantuan_of_tarmogoyf_is_a_seven_seven_with_no_cda() {
    let mut game = setup_two_player_game();
    let goyf = put_on_battlefield(&mut game, tarmogoyf(), 1);
    put_in_graveyard(&mut game, grizzly_bears(), 1);
    assert_eq!(pt(&game, goyf), (Some(1), Some(2)));

    let except = vec![CopyException::Modifies(CharacteristicEdit::PowerToughness(7, 7))];
    let gargantuan = copy_of(&mut game, copier("Quicksilver Gargantuan-shaped", Color::Blue, except), 0, goyf);
    assert_eq!(get_effective_name(&game, gargantuan), "Tarmogoyf");
    assert_eq!(pt(&game, gargantuan), (Some(7), Some(7)));
    assert!(get_effective_abilities(&game, gargantuan).iter().all(|a| !a.is_characteristic_defining));
    put_in_graveyard(&mut game, grizzly_bears(), 0);
    assert_eq!(pt(&game, gargantuan), (Some(7), Some(7)));
}

/// Glasspool Mimic — "... except it's a Shapeshifter Rogue in addition to its
/// other types." CR 707.9d's carve-out: an "in addition" exception keeps the
/// donor's subtype CDA, so the copy is a Sliver, a Shapeshifter and a Rogue.
/// An exception that sets the subtypes instead drops it.
// COVERS-PARTIAL: ATOM-707.9d-002, COMP-9A-006
#[test]
fn an_in_addition_exception_keeps_a_subtype_cda_and_a_setting_one_drops_it() {
    let mut game = setup_two_player_game();
    let sliver = put_on_battlefield(&mut game, sliver_by_cda(), 0);
    assert!(has_subtype(&game, sliver, &Subtype::Creature(CreatureType::Sliver)));

    let mimic_except = vec![adding(with_subtypes(&[CreatureType::Shapeshifter, CreatureType::Rogue]))];
    let mimic = copy_of(&mut game, copier("Glasspool Mimic-shaped", Color::Blue, mimic_except), 0, sliver);
    for subtype in [CreatureType::Sliver, CreatureType::Shapeshifter, CreatureType::Rogue] {
        assert!(has_subtype(&game, mimic, &Subtype::Creature(subtype)), "{subtype:?}");
    }
    assert!(get_effective_abilities(&game, mimic).iter().any(|a| a.is_characteristic_defining));

    let set_except = vec![adding(TypeChange {
        set_subtypes: Some([Subtype::Creature(CreatureType::Construct)].into_iter().collect()),
        ..TypeChange::NONE
    })];
    let construct = copy_of(&mut game, copier("Construct copier", Color::Blue, set_except), 0, sliver);
    assert!(has_subtype(&game, construct, &Subtype::Creature(CreatureType::Construct)));
    assert!(!has_subtype(&game, construct, &Subtype::Creature(CreatureType::Sliver)));
    assert!(get_effective_abilities(&game, construct).iter().all(|a| !a.is_characteristic_defining));

    // A card type is not a subtype, and no CDA defines one: "an artifact in
    // addition to its other types" leaves the subtype CDA alone.
    let artifact_except = vec![adding(TypeChange { add_types: vec![CardType::Artifact], ..TypeChange::NONE })];
    let artifact = copy_of(&mut game, copier("Artifact copier", Color::Blue, artifact_except), 0, sliver);
    assert_eq!(get_effective_types(&game, artifact), CardTypes::from([CardType::Artifact, CardType::Creature]));
    assert!(has_subtype(&game, artifact, &Subtype::Creature(CreatureType::Sliver)));
    assert!(get_effective_abilities(&game, artifact).iter().any(|a| a.is_characteristic_defining));
}

/// CR 707.9a: an exception that gives the copy "Creatures you control get
/// +1/+1" makes it part of the copiable values. Registration files it like
/// the copy's other abilities, a Clone of the copy has it too, and the
/// builder stamped it an id like any printed ability.
// COVERS-PARTIAL: ATOM-707.9a-001
#[test]
fn a_gained_ability_is_copiable_and_works() {
    let mut anthem = glorious_anthem().abilities[0].clone();
    anthem.id = AbilityId::UNASSIGNED;
    let except = vec![CopyException::Modifies(CharacteristicEdit::GainsAbility(anthem))];
    let card = copier("Anthem copier", Color::Blue, except);
    let CopyException::Modifies(CharacteristicEdit::GainsAbility(stamped)) = gained_of(&card) else {
        panic!("the fixture's one exception")
    };
    assert_ne!(stamped.id, AbilityId::UNASSIGNED);

    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let bystander = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let copy = copy_of(&mut game, card, 0, bears);
    assert_eq!(pt(&game, bystander), (Some(2), Some(2)));
    assert_eq!(pt(&game, copy), (Some(3), Some(3)));

    let clone = copy_of(&mut game, phase_cv_cards::clone(), 0, copy);
    assert!(get_effective_abilities(&game, clone).iter().any(|a| a.id == stamped.id));
    assert_eq!(pt(&game, bystander), (Some(3), Some(3)), "two anthems now");
}

/// The first exception of a copier built by [`copier`].
fn gained_of(card: &CardData) -> CopyException {
    let Effect::Replacement(def) = &card.abilities[0].effect else { panic!("a replacement") };
    let Rewrite::EnterAsCopy(template) = &def.rewrite else { panic!("an entry copy") };
    template.except[0].clone()
}

/// Vesuvan Doppelganger — "... except it doesn't copy that creature's
/// color". CR 707.9c keeps its own blue, and 707.9d drops Culling Drone's
/// devoid, the CDA that defines the color not copied. A Clone of it copies
/// the new values: blue, with no devoid.
// COVERS-PARTIAL: ATOM-707.3-001
#[test]
fn a_vesuvan_of_culling_drone_keeps_its_blue_without_devoid() {
    let mut game = setup_two_player_game();
    let drone = put_on_battlefield(&mut game, culling_drone(), 1);
    assert!(compute_characteristics(&game, drone).unwrap().colors.is_empty(), "devoid");

    let except = vec![CopyException::DoesNotCopy(Characteristic::Color)];
    let vesuvan = copy_of(&mut game, copier("Vesuvan-shaped", Color::Blue, except), 0, drone);
    let blue: std::collections::HashSet<Color> = [Color::Blue].into_iter().collect();
    assert_eq!(get_effective_name(&game, vesuvan), "Culling Drone");
    assert_eq!(compute_characteristics(&game, vesuvan).unwrap().colors, blue);
    assert!(get_effective_abilities(&game, vesuvan).iter().all(|a| !a.is_characteristic_defining));

    let clone = copy_of(&mut game, phase_cv_cards::clone(), 0, vesuvan);
    assert_eq!(compute_characteristics(&game, clone).unwrap().colors, blue);
}

/// Sakashima the Impostor — "... except its name is Sakashima the Impostor,
/// it's legendary in addition to its other types". Both are copiable values.
#[test]
fn a_name_exception_is_the_copys_name_and_a_clone_of_it_takes_it() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let except = vec![
        CopyException::Modifies(CharacteristicEdit::Name("Sakashima the Impostor".to_string())),
        adding(TypeChange { add_supertypes: vec![Supertype::Legendary], ..TypeChange::NONE }),
    ];
    let sakashima = copy_of(&mut game, copier("Sakashima-shaped", Color::Blue, except), 0, bears);
    assert_eq!(get_effective_name(&game, sakashima), "Sakashima the Impostor");
    assert!(is_legendary(&game, sakashima));
    assert_eq!(pt(&game, sakashima), (Some(2), Some(2)));

    let clone = copy_of(&mut game, phase_cv_cards::clone(), 1, sakashima);
    assert_eq!(get_effective_name(&game, clone), "Sakashima the Impostor");
}

/// Mockingbird — "... except it's a Bird in addition to its other types and
/// it has flying." Its candidate filter on the mana spent is unbuilt.
#[test]
fn a_keyword_exception_gives_the_copy_flying() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let except = vec![
        adding(with_subtypes(&[CreatureType::Bird])),
        CopyException::Modifies(CharacteristicEdit::GainsKeyword(KeywordFlag::Flying)),
    ];
    let bird = copy_of(&mut game, copier("Mockingbird-shaped", Color::Blue, except), 0, bears);
    assert!(has_keyword(&game, bird, KeywordFlag::Flying));
    assert!(has_subtype(&game, bird, &Subtype::Creature(CreatureType::Bird)));
    assert_eq!(get_effective_name(&game, bird), "Grizzly Bears");
}

/// CR 614.17d at the door: under "creatures you control can't have +1/+1
/// counters put on them", Spark Double's counter is refused and the copy
/// still enters.
#[test]
fn a_cant_have_counters_refuses_spark_doubles_counter() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, no_plus_counters_on_your_creatures(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let spark = spark_of(&mut game, 0, bears);
    assert_eq!(get_effective_name(&game, spark), "Grizzly Bears");
    assert_eq!(counters(&game, spark, CounterType::PlusOnePlusOne), 0);
}

// ---------------------------------------------------------------------------
// 5. "Except it enters untapped": two wordings, and a status taken back
// ---------------------------------------------------------------------------

fn enters_untapped_as_an_addition() -> Arc<CardData> {
    copier("Untapped Addition copier", Color::Blue, vec![CopyException::Additionally(EnterModsTemplate::untapped())])
}

/// Read as CR 707.9e's additional effect, "except it enters untapped" applies
/// with the copy, at 616.1c. The copied "enters tapped" applies after it, at
/// 616.1e, and the copy of Rusted Sentinel enters tapped. Nothing is asked.
#[test]
fn an_untapped_addition_is_overridden_by_the_copied_enters_tapped() {
    let mut game = setup_two_player_game();
    let sentinel = put_on_battlefield(&mut game, rusted_sentinel(), 1);
    let copy = copy_of(&mut game, enters_untapped_as_an_addition(), 0, sentinel);
    assert_eq!(get_effective_name(&game, copy), "Rusted Sentinel");
    assert!(game.battlefield[&copy].tapped);
}

/// Read as CR 707.9a's gained ability, "it has 'This creature enters
/// untapped'" joins the 616.1e bucket beside the copied "enters tapped", and
/// its controller orders the two, as with Spelunking: both statuses are
/// reachable, the last applied winning.
#[test]
fn an_untapped_grant_is_ordered_against_the_copied_enters_tapped() {
    let mut reached = Vec::new();
    for pick in [0, 1] {
        let mut game = setup_two_player_game();
        let sentinel = put_on_battlefield(&mut game, rusted_sentinel(), 1);
        let except =
            vec![CopyException::Modifies(CharacteristicEdit::GainsAbility(enters_with(EnterModsTemplate::untapped())))];
        let dp = Scripted::new()
            .then(0, "ChooseCopySource", &[sentinel])
            .then_index(0, "ChooseReplacementEffect", pick);
        let copy = enter(&mut game, copier("Untapped Grant copier", Color::Blue, except), 0, &dp);
        reached.push(game.battlefield[&copy].tapped);
    }
    reached.sort();
    assert_eq!(reached, vec![false, true], "each order reaches its own status");
}

/// CR 707.9e takes back a status as it takes back counters. An instruction
/// returns the copier tapped; its addition makes it untapped as it copies a
/// Clone that copied nothing, and the Clone's copied ability then copies a
/// Bear, so the addition does not happen and the instruction's "tapped" is
/// back. Declining the Clone's ability keeps the addition.
#[test]
fn a_later_copy_puts_back_the_status_an_addition_replaced() {
    for (bear, tapped) in [(true, true), (false, false)] {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, glorious_anthem(), 0);
        let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
        let empty_clone = enter(&mut game, phase_cv_cards::clone(), 0, &Scripted::new().then(0, "ChooseCopySource", &[]));
        let copier = put_in_graveyard(&mut game, enters_untapped_as_an_addition(), 0);
        let second: &[ObjectId] = if bear { &[bears] } else { &[] };
        let dp = Scripted::new().then(0, "ChooseCopySource", &[empty_clone]).then(0, "ChooseCopySource", second);
        let returned_tapped = GameAction::EnterBattlefield {
            object: copier,
            from: Some(Zone::Graveyard),
            controller: 0,
            mods: EnterMods::tapped(),
            cause: Some(ZoneChangeCause::Returned),
        };
        game.execute_actions(vec![returned_tapped], &ActionContext::new(&dp)).expect("it enters");
        assert_eq!(game.battlefield[&copier].tapped, tapped);
    }
}

/// Essence of the Wild — "Creatures you control enter as a copy of this
/// creature."
fn essence_of_the_wild() -> Arc<CardData> {
    CardDataBuilder::new("Essence of the Wild")
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Avatar))
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green, ManaType::Green], 3))
        .power_toughness(6, 6)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::battlefield_filter(creatures_you_control()),
            Rewrite::EnterAsCopy(EntryCopyTemplate { donor: CopyDonor::ThisObject, except: Vec::new() }),
        )))))
        .build()
}

/// An artifact that "enters tapped" and is an artifact creature "as long as
/// it's tapped": a status write that makes a copy effect applicable.
fn tapped_animus() -> Arc<CardData> {
    let creature = TypeChange { add_types: vec![CardType::Creature], ..TypeChange::NONE };
    CardDataBuilder::new("Tapped Animus")
        .card_type(CardType::Artifact)
        .mana_cost(ManaCost::build(&[], 2))
        .ability(enters_with(EnterModsTemplate::tapped()))
        .ability(static_ability(Effect::Conditional(
            Condition::SourceTapped,
            Box::new(Effect::Sequence(vec![
                Effect::Atom(Primitive::ChangeType(creature, Duration::WhileSourceOnBattlefield), EffectRecipient::ThisObject),
                Effect::Atom(
                    Primitive::SetPowerToughness(AmountExpr::Fixed(2), AmountExpr::Fixed(2), Duration::WhileSourceOnBattlefield),
                    EffectRecipient::ThisObject,
                ),
            ])),
        )))
        .build()
}

/// `EnterMods::merge` gives up a copy's claim to the status once a later
/// effect sets one. The copier, untapped by its addition, copies the Animus.
/// The Animus's copied "enters tapped" then taps it, which makes it a
/// creature, so Essence of the Wild's copy becomes applicable (CR 616.2).
/// That copy takes the addition back, and the "tapped" set after the
/// addition stands.
#[test]
fn a_later_copy_does_not_restore_over_a_status_set_after_the_addition() {
    let mut game = setup_two_player_game();
    let animus = put_on_battlefield(&mut game, tapped_animus(), 0);
    game.battlefield.get_mut(&animus).unwrap().tapped = true;
    game.bump_layer_epoch();
    put_on_battlefield(&mut game, essence_of_the_wild(), 0);
    assert!(get_effective_types(&game, animus).contains(&CardType::Creature));

    let copier = put_in_graveyard(&mut game, enters_untapped_as_an_addition(), 0);
    let dp = Scripted::new()
        .then(0, "ChooseReplacementEffect", &[copier])
        .then(0, "ChooseCopySource", &[animus]);
    try_enter(&mut game, copier, &dp).expect("it enters");
    assert_eq!(get_effective_name(&game, copier), "Essence of the Wild");
    assert!(game.battlefield[&copier].tapped);
}

// ---------------------------------------------------------------------------
// 6. What the applier refuses
// ---------------------------------------------------------------------------

/// An edit written as a CR 707.9e addition, which 707.9e defines as "not a
/// modification of the affected object's characteristics", and a condition
/// inside a condition, are authoring errors, refused loudly rather than
/// guessed at.
#[test]
fn an_edit_as_an_addition_and_a_nested_condition_are_refused() {
    let as_a_mutant = CharacteristicEdit::Types(with_subtypes(&[CreatureType::Mutant]));
    let edit_as_addition = CopyException::Additionally(EnterModsTemplate {
        status: None,
        counters: Vec::new(),
        edits: vec![as_a_mutant],
    });
    let creature = || ObjectFilter::ByType(CardType::Creature);
    let nested = CopyException::If(creature(), vec![CopyException::If(creature(), Vec::new())]);
    for (except, says) in [(edit_as_addition, "707.9e"), (nested, "nests")] {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
        let card = put_in_graveyard(&mut game, copier("Malformed copier", Color::Blue, vec![except]), 0);
        let dp = Scripted::new().then(0, "ChooseCopySource", &[bears]);
        let refusal = try_enter(&mut game, card, &dp).expect_err("refused");
        assert!(refusal.contains(says), "{refusal}");
    }
}

/// The copy placement's arms have no entry placement yet
/// (`replacement-architecture.md` §3.5), so an "enters with" that makes one
/// is refused at the entry door rather than dropped.
#[test]
fn an_entry_edit_with_no_entry_placement_is_refused() {
    let mut game = setup_two_player_game();
    let grants_flying = EnterModsTemplate {
        status: None,
        counters: Vec::new(),
        edits: vec![CharacteristicEdit::GainsKeyword(KeywordFlag::Flying)],
    };
    let card = put_in_graveyard(
        &mut game,
        CardDataBuilder::new("Flying entry")
            .card_type(CardType::Creature)
            .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
            .power_toughness(1, 1)
            .ability(enters_with(grants_flying))
            .build(),
        0,
    );
    let refusal = try_enter(&mut game, card, &Scripted::new()).expect_err("refused");
    assert!(refusal.contains("no entry placement"), "{refusal}");
}
