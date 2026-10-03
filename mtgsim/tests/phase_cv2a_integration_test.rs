//! Phase CV-2a integration tests: a permanent that enters as a copy (CR 707.5,
//! 616.1c), `copy-effects-architecture.md` §7b.
//!
//! Five things this file proves, in the order the phase built them:
//!
//! 1. The copy is what the permanent **enters as**, so CR 614.12's frame, the
//!    layer walk and every later replacement see the copy (613.2a, 616.1f).
//! 2. The CR 707.6 choice: made by the entering object's controller, among
//!    what is on the battlefield before the entry, and "may" is a pick of none.
//! 3. What the copy brings: its "enters with" and "as enters" abilities apply
//!    during the same entry, and its "enters" triggers fire for it (707.5).
//! 4. Every gate sees an entry copy's abilities, because registration files
//!    the list the permanent arrived with (D5): one test per gate.
//! 5. The carrier is state, so the copy leaves with the permanent (400.7),
//!    and a later copy effect applies over it and gives it back (707.4).
//!
//! Fixtures are built inline, named for the printed card whose board they
//! stand in for, and never registered. The printed cards were verified on
//! Scryfall on 2026-09-28.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;

use mtgsim::cards::authoring::{enters, triggered_ability, whenever};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_cm_cards::thalia_guardian_of_thraben;
use mtgsim::cards::phase_cv_cards::{self, cytoshape};
use mtgsim::cards::phase_lf_cards::citanul_hierophants;
use mtgsim::cards::phase_rc_cards::{chainbreaker, dryad_arbor, master_biomancer, root_maze, thunder_thrash_elder};
use mtgsim::cards::phase_re_cards::doubling_season;
use mtgsim::cards::phase_rs_cards::{diabolic_edict, sigarda_host_of_herons};
use mtgsim::cards::phase_tr1_cards::soul_warden;
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::engine::layers::compute_characteristics;
use mtgsim::engine::layers::types::{EffectModification, Layer, PtValue};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::event::DamageTarget;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::objects::object::GameObject;
use mtgsim::oracle::characteristics::{
    get_effective_abilities, get_effective_controller, get_effective_name, get_effective_power,
    get_effective_toughness, get_effective_types, has_keyword, has_subtype,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    fill_library, place_bare, put_in_graveyard, put_in_hand, put_on_battlefield, registered, setup_two_player_game,
    static_ability, vanilla_creature,
};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AmountExpr, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerRef, PlayerSet,
    Primitive, TokenDef,
};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::{ManaCost, ManaSymbol, ManaType};
use mtgsim::types::replacement::{
    CopyDonor, EnterMods, EnterModsTemplate, EntryCopyTemplate, EventPattern, ReplacementDef, Rewrite,
};
use mtgsim::types::restriction::{Restriction, RestrictionDef};
use mtgsim::types::triggers::TriggerSubject;
use mtgsim::types::zones::{Zone, ZoneChangeCause};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceOption, position_of};
use mtgsim::ui::decision::DecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// The provider: every prompt scripted, by object id, with who is asked
// ---------------------------------------------------------------------------

/// Answers each prompt from a script, in order: who must be asked, the kind's
/// name, and the objects to pick. Picking by id keeps a test off the order of
/// the candidates, which is the battlefield's timestamps, and naming the
/// player is CR 707.6's "the copy's controller" asserted at every prompt. An
/// unscripted prompt panics, and so does a script left over.
struct ById {
    script: RefCell<VecDeque<(PlayerId, &'static str, Vec<ObjectId>)>>,
    /// Every prompt asked: who, the kind's name, and the objects offered.
    asked: RefCell<Vec<(PlayerId, String, Vec<ObjectId>)>>,
}

impl ById {
    fn new() -> Self {
        ById { script: RefCell::new(VecDeque::new()), asked: RefCell::new(Vec::new()) }
    }

    /// Expect `player` to be asked a `kind` prompt next, and pick `ids`.
    fn then(self, player: PlayerId, kind: &'static str, ids: &[ObjectId]) -> Self {
        self.script.borrow_mut().push_back((player, kind, ids.to_vec()));
        self
    }

    fn asked(&self) -> Vec<(PlayerId, String, Vec<ObjectId>)> {
        self.asked.borrow().clone()
    }
}

impl Drop for ById {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(self.script.borrow().is_empty(), "scripted prompts never asked: {:?}", self.script.borrow());
        }
    }
}

impl DecisionProvider for ById {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        ctx: &ChoiceContext,
        options: &[ChoiceOption],
        _bounds: (usize, usize),
    ) -> Vec<usize> {
        let kind = ctx.kind.as_str();
        let offered: Vec<ObjectId> = options
            .iter()
            .filter_map(|o| match o {
                ChoiceOption::Object(id) => Some(*id),
                _ => None,
            })
            .collect();
        self.asked.borrow_mut().push((player, kind.to_string(), offered.clone()));
        let (who, expected, ids) = self
            .script
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| panic!("unscripted prompt: {:?} to player {player}", ctx.kind));
        assert_eq!((who, expected), (player, kind), "the wrong prompt, or the wrong player asked");
        let mut picked = Vec::new();
        for id in ids {
            let found = position_of(options, game, &ChoiceOption::Object(id).as_logged(game), &picked);
            picked.push(found.unwrap_or_else(|| panic!("{id} not offered: {offered:?}")));
        }
        picked
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

/// Put `card` in `owner`'s graveyard and return it to the battlefield through
/// the replacement pipeline, as "return target creature card" does.
fn enter(game: &mut GameState, card: Arc<CardData>, owner: PlayerId, dp: &dyn DecisionProvider) -> ObjectId {
    let id = put_in_graveyard(game, card, owner);
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(dp))
        .expect("the entry is proposed");
    id
}

/// Clone entering under `player` as a copy of `donor`, the only prompt.
fn clone_of(game: &mut GameState, player: PlayerId, donor: ObjectId) -> ObjectId {
    let dp = ById::new().then(player, "ChooseCopySource", &[donor]);
    enter(game, phase_cv_cards::clone(), player, &dp)
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

/// A row on `id` alone, registered as a resolution would.
fn add_row(game: &mut GameState, id: ObjectId, layer: Layer, modification: EffectModification) {
    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(registered(id, layer, timestamp, modification));
}

/// Resolve `card`'s one spell ability under `controller` with `targets`.
fn resolve_spell(
    game: &mut GameState,
    card: Arc<CardData>,
    controller: PlayerId,
    targets: Vec<ResolvedTarget>,
    dp: &dyn DecisionProvider,
) {
    let source = game.add_object(GameObject::new(card.clone(), controller, Zone::Stack));
    let ctx = ResolutionContext { targets: ChosenTargets::one(targets), ..ResolutionContext::untargeted(source, controller) };
    game.resolve_effect(&card.abilities[0].effect, &ctx, dp).expect("resolution");
}

fn creatures_you_control() -> ObjectFilter {
    ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    )
}

fn replacement(pattern: EventPattern, affected: ObjectSet, rewrite: Rewrite) -> AbilityDef {
    static_ability(Effect::Replacement(Box::new(ReplacementDef::new(pattern, affected, rewrite))))
}

/// "This creature enters with ..." or "enters tapped", on the creature itself.
fn enters_with(template: EnterModsTemplate) -> AbilityDef {
    replacement(EventPattern::EnterBattlefield { cast: None }, ObjectSet::SourceOnly, Rewrite::EnterWith(template))
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// A 5/5 white flier, so a copy has a name, a color, a keyword, a cost and a
/// P/T to take.
fn colossus() -> Arc<CardData> {
    CardDataBuilder::new("Colossus")
        .card_type(CardType::Creature)
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White], 4))
        .power_toughness(5, 5)
        .keyword_flag(KeywordFlag::Flying)
        .build()
}

/// Essence of the Wild — "Creatures you control enter as a copy of this
/// creature." Named by the caller, so two can be told apart.
fn essence_of_the_wild(name: &str, power: i32, toughness: i32) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Avatar))
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green, ManaType::Green], 3))
        .power_toughness(power, toughness)
        .ability(replacement(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::battlefield_filter(creatures_you_control()),
            Rewrite::EnterAsCopy(EntryCopyTemplate { donor: CopyDonor::ThisObject, except: Vec::new() }),
        ))
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

/// Skyshroud Behemoth — a 10/10 that enters tapped and with two counters.
/// The counters are charge counters: fading's are its upkeep's, which is
/// unbuilt, and to CR 707.5 a counter's kind is data.
fn skyshroud_behemoth() -> Arc<CardData> {
    CardDataBuilder::new("Skyshroud Behemoth")
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Beast))
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 5))
        .power_toughness(10, 10)
        .ability(enters_with(EnterModsTemplate::with_counters(CounterType::Charge, 2)))
        .ability(enters_with(EnterModsTemplate::tapped()))
        .build()
}

/// Wall of Omens — {1}{W} 0/4 Wall, defender, "When this creature enters,
/// draw a card."
fn wall_of_omens() -> Arc<CardData> {
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

/// Worms of the Earth's second sentence, "Lands can't enter the
/// battlefield", in `phase_rc4b_integration_test`'s shape.
fn worms_of_the_earth() -> Arc<CardData> {
    CardDataBuilder::new("Worms of the Earth")
        .card_type(CardType::Enchantment)
        .color(Color::Black)
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black, ManaType::Black], 2))
        .ability(static_ability(Effect::Restriction(Box::new(RestrictionDef::new(Restriction::Event {
            pattern: EventPattern::ZoneChange { from: None, to: Some(Zone::Battlefield), cause: None, object: None },
            affected_objects: ObjectSet::battlefield_filter(ObjectFilter::ByType(CardType::Land)),
            affected_players: PlayerSet::Nobody,
            by: None,
        })))))
        .build()
}

/// A creature carrying Glorious Anthem's ability, so a Clone can choose it.
fn anthem_bear() -> Arc<CardData> {
    CardDataBuilder::new("Anthem Bear")
        .card_type(CardType::Creature)
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White], 2))
        .power_toughness(2, 2)
        .ability(static_ability(Effect::Atom(
            Primitive::ModifyPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(1), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(creatures_you_control()),
        )))
        .build()
}

/// A creature that is also a planeswalker, which no printed card is on the
/// battlefield: the board D7's copiable loyalty is for.
fn planeswalker_creature() -> Arc<CardData> {
    CardDataBuilder::new("Planeswalker Creature")
        .card_type(CardType::Creature)
        .card_type(CardType::Planeswalker)
        .color(Color::Blue)
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .power_toughness(2, 2)
        .loyalty(3)
        .build()
}

/// A {0} instant that does nothing: a noncreature spell to tax.
fn idle_thought() -> Arc<CardData> {
    CardDataBuilder::new("Idle Thought")
        .card_type(CardType::Instant)
        .mana_cost(ManaCost::build(&[], 0))
        .ability(AbilityDef {
            rules_text: "".into(),
            id: new_ability_id(),
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(Vec::new()),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
        })
        .build()
}

// ---------------------------------------------------------------------------
// 1. The copy is what the permanent enters as
// ---------------------------------------------------------------------------

/// CR 613.2a — Clone cast from hand, from exactly `{3}{U}`, enters as the
/// 5/5: its name, colors, cost, types, keyword and P/T are the donor's. One
/// candidate is two outcomes under "you may", so it is asked, once, of the
/// Clone's controller.
// COVERS: ATOM-613.2a-001
#[test]
fn clone_cast_from_hand_enters_as_a_copy_of_a_five_five() {
    let mut game = setup_two_player_game();
    let donor = put_on_battlefield(&mut game, colossus(), 1);
    let dp = ManaWindowStop::new(ById::new());
    let clone = cast_from_pool(&mut game, 0, phase_cv_cards::clone(), &[(ManaType::Blue, 1), (ManaType::Colorless, 3)], &dp)
        .expect("Clone is castable from exactly {3}{U}");
    assert_eq!(game.players[0].mana_pool.total(), 0, "the whole pool was the cost");

    let dp = ManaWindowStop::new(ById::new().then(0, "ChooseCopySource", &[donor]));
    game.resolve_top_of_stack(&dp).expect("Clone resolves");

    let chars = compute_characteristics(&game, clone).expect("on the battlefield");
    assert_eq!(chars.name, "Colossus");
    assert_eq!((chars.power, chars.toughness), (Some(5), Some(5)));
    assert_eq!(chars.types, [CardType::Creature].into_iter().collect());
    assert!(chars.subtypes.is_empty(), "the Shapeshifter subtype is the card's, not the copy's");
    assert_eq!(chars.colors, [Color::White].into_iter().collect());
    assert_eq!(chars.mana_cost, Some(ManaCost::build(&[ManaType::White], 4)));
    assert!(has_keyword(&game, clone, KeywordFlag::Flying));
    assert_eq!(get_effective_controller(&game, clone), Some(0), "control is not copiable");
    assert_eq!(dp.inner().asked(), vec![(0, "ChooseCopySource".to_string(), vec![donor])]);
}

// RULING: Clone #7 - "If the copied creature is copying something else ...,
//   then Clone enters as whatever that creature copied."
/// CR 613.2c — a copy's copiable values are its state after layer 1, so Clone
/// B, copying Clone A, which copied Grizzly Bears, is Grizzly Bears.
// COVERS: ATOM-613.2c-001
#[test]
fn a_clone_of_a_clone_enters_as_what_the_first_copied() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let a = clone_of(&mut game, 0, bears);
    let b = clone_of(&mut game, 0, a);

    assert_eq!(get_effective_name(&game, b), "Grizzly Bears");
    assert_eq!((get_effective_power(&game, b), get_effective_toughness(&game, b)), (Some(2), Some(2)));
    assert!(!has_subtype(&game, b, &Subtype::Creature(CreatureType::Shapeshifter)));
}

// RULING: Clone #8 - "It doesn't copy whether that creature is tapped or
//   untapped, whether it has any counters on it or Auras and Equipment
//   attached to it, or any non-copy effects that have changed its power,
//   toughness, types, color, or so on."
/// CR 707.2 — the donor's three +1/+1 counters, its tapped status, a pump and
/// a color change are not copiable: the Clone is an untapped 2/2 green Bear
/// with no counters.
// COVERS: ATOM-707.2-003
#[test]
fn a_clone_copies_no_counters_status_or_noncopy_effects() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.add_counters(bears, CounterType::PlusOnePlusOne, 3);
    game.battlefield.get_mut(&bears).unwrap().tapped = true;
    add_row(&mut game, bears, Layer::Layer7cModifyPT, EffectModification::ModifyPowerToughness {
        power: PtValue::Fixed(2),
        toughness: PtValue::Fixed(2),
    });
    add_row(&mut game, bears, Layer::Layer5Color, EffectModification::SetColors([Color::Black].into_iter().collect()));
    assert_eq!(get_effective_power(&game, bears), Some(7), "the donor really is a 7/7 now");

    let clone = clone_of(&mut game, 0, bears);
    assert_eq!((get_effective_power(&game, clone), get_effective_toughness(&game, clone)), (Some(2), Some(2)));
    assert_eq!(counters(&game, clone, CounterType::PlusOnePlusOne), 0);
    assert!(!game.battlefield[&clone].tapped);
    assert_eq!(compute_characteristics(&game, clone).unwrap().colors, [Color::Green].into_iter().collect());
}

/// CR 707.2 — Chimeric Staff, animated into a 5/5 creature by its own
/// ability, is a creature Clone may choose, and the animation is not
/// copiable: the Clone enters as a noncreature artifact with the Staff's
/// activated ability, and does not die a 0/0.
// COVERS: ATOM-707.2-001
#[test]
fn a_clone_of_an_animated_artifact_is_that_artifact() {
    let mut game = setup_two_player_game();
    let staff_ability = AbilityDef {
        rules_text: "".into(),
        id: new_ability_id(),
        instances: Vec::new(),
        ability_type: AbilityType::Activated,
        costs: vec![mtgsim::types::costs::Cost::Mana(ManaCost::from_symbols(vec![ManaSymbol::X]))],
        effect: Effect::Sequence(Vec::new()),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    };
    let staff_card = CardDataBuilder::new("Chimeric Staff")
        .card_type(CardType::Artifact)
        .mana_cost(ManaCost::build(&[], 4))
        .ability(staff_ability.clone())
        .build();
    let staff = put_on_battlefield(&mut game, staff_card, 1);
    // "Becomes an X/X Construct artifact creature until end of turn", X = 5.
    add_row(&mut game, staff, Layer::Layer4Type, EffectModification::AddType(CardType::Creature));
    add_row(&mut game, staff, Layer::Layer7bSetPT, EffectModification::SetPowerToughness {
        power: PtValue::Fixed(5),
        toughness: PtValue::Fixed(5),
    });
    assert!(get_effective_types(&game, staff).contains(&CardType::Creature));

    let clone = clone_of(&mut game, 0, staff);
    settle(&mut game, &ById::new());
    assert_eq!(zone_of(&game, clone), Zone::Battlefield, "a noncreature has no toughness to die of");
    assert_eq!(get_effective_name(&game, clone), "Chimeric Staff");
    assert_eq!(get_effective_types(&game, clone), [CardType::Artifact].into_iter().collect());
    assert_eq!(get_effective_power(&game, clone), None);
    let abilities = get_effective_abilities(&game, clone);
    assert_eq!(abilities.iter().map(|a| a.id).collect::<Vec<_>>(), vec![staff_ability.id]);
}

/// CR 707.2b — the Clone's values were captured as it entered, so the Bears
/// later becoming a copy of something else does not change it.
// COVERS: ATOM-707.2b-001
#[test]
fn a_clone_is_unchanged_when_its_donor_becomes_a_copy_of_something_else() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let big = put_on_battlefield(&mut game, colossus(), 1);
    let clone = clone_of(&mut game, 0, bears);

    let dp = ById::new().then(0, "ChooseCopySource", &[big]);
    resolve_spell(&mut game, cytoshape(), 0, vec![ResolvedTarget::Object(bears)], &dp);
    assert_eq!(get_effective_name(&game, bears), "Colossus");

    assert_eq!(get_effective_name(&game, clone), "Grizzly Bears");
    assert_eq!(get_effective_power(&game, clone), Some(2));
}

/// CR 613.1a — the copy is the base the later layers apply to: -3/-3 on a
/// Clone of a vanilla 5/5 leaves a 2/2, which survives the state-based
/// actions.
// COVERS: ATOM-613.1a-001
#[test]
fn minus_three_on_a_clone_of_a_five_five_leaves_a_two_two() {
    let mut game = setup_two_player_game();
    let donor = put_on_battlefield(&mut game, vanilla_creature(5, 5, &[]), 1);
    let clone = clone_of(&mut game, 0, donor);
    add_row(&mut game, clone, Layer::Layer7cModifyPT, EffectModification::ModifyPowerToughness {
        power: PtValue::Fixed(-3),
        toughness: PtValue::Fixed(-3),
    });
    settle(&mut game, &ById::new());
    assert_eq!((get_effective_power(&game, clone), get_effective_toughness(&game, clone)), (Some(2), Some(2)));
    assert_eq!(zone_of(&game, clone), Zone::Battlefield);
}

// ---------------------------------------------------------------------------
// 2. The CR 707.6 choice
// ---------------------------------------------------------------------------

// RULING: Clone #4 - "You can choose not to copy anything. In that case,
//   Clone enters as a 0/0 Shapeshifter creature, and is probably put into
//   the graveyard immediately."
/// Declining is a pick of none, asked once: the Clone enters as itself and
/// CR 704.5f puts it into the graveyard.
#[test]
fn a_clone_that_copies_nothing_enters_as_a_zero_zero_and_dies() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    let dp = ById::new().then(0, "ChooseCopySource", &[]);
    let clone = enter(&mut game, phase_cv_cards::clone(), 0, &dp);

    assert_eq!(get_effective_name(&game, clone), "Clone");
    assert!(has_subtype(&game, clone, &Subtype::Creature(CreatureType::Shapeshifter)));
    assert_eq!(get_effective_toughness(&game, clone), Some(0));
    settle(&mut game, &ById::new());
    assert_eq!(zone_of(&game, clone), Zone::Graveyard);
}

/// With no creature on the battlefield there is nothing to choose, so
/// nothing is asked (CR 101.3), and the 0/0 dies.
#[test]
fn a_clone_with_no_creature_to_copy_asks_nothing() {
    let mut game = setup_two_player_game();
    let clone = enter(&mut game, phase_cv_cards::clone(), 0, &ById::new());
    assert_eq!(get_effective_name(&game, clone), "Clone");
    settle(&mut game, &ById::new());
    assert_eq!(zone_of(&game, clone), Zone::Graveyard);
}

// RULING: Clone #1 - "Clone's ability doesn't target the chosen creature."
/// An opponent's hexproof creature is a candidate, since the choice is not a
/// target, and the copy has hexproof.
#[test]
fn a_clone_may_copy_an_opponents_hexproof_creature() {
    let mut game = setup_two_player_game();
    let donor = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[KeywordFlag::Hexproof]), 1);
    let clone = clone_of(&mut game, 0, donor);
    assert!(has_keyword(&game, clone, KeywordFlag::Hexproof));
    assert_eq!(get_effective_power(&game, clone), Some(3));
}

// RULING: Clone #5 - "If Clone somehow enters at the same time as another
//   creature, Clone can't become a copy of that creature. You may choose only
//   a creature that's already on the battlefield."
/// Clone and a Bear entering as one event: the Bear is not offered, because
/// each entry is decided against the board before the batch performs. At the
/// `execute_actions` boundary, since no registered effect makes the batch.
#[test]
fn a_creature_entering_beside_clone_is_not_a_candidate() {
    let mut game = setup_two_player_game();
    let donor = put_on_battlefield(&mut game, colossus(), 1);
    let clone = put_in_graveyard(&mut game, phase_cv_cards::clone(), 0);
    let bears = put_in_graveyard(&mut game, grizzly_bears(), 0);
    let entry = |id| GameAction::EnterBattlefield {
        object: id,
        from: Some(Zone::Graveyard),
        controller: 0,
        mods: EnterMods::NONE,
        cause: Some(ZoneChangeCause::Returned),
    };
    let dp = ById::new().then(0, "ChooseCopySource", &[donor]);
    game.execute_actions(vec![entry(clone), entry(bears)], &ActionContext::new(&dp)).expect("both enter");

    assert_eq!(dp.asked()[0].2, vec![donor], "only what was already on the battlefield is offered");
    assert!(game.battlefield.contains_key(&bears));
    assert_eq!(get_effective_name(&game, clone), "Colossus");
}

// RULING: Clone #6 - "If the copied creature is a token, Clone copies the
//   original characteristics of that token as stated by the effect that
//   created the token."
/// A token's copiable values are what its creating effect stated (CR
/// 707.2), and the Clone is a card, not a token (CR 111.1).
#[test]
fn a_clone_of_a_token_is_not_a_token() {
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
    let source = place_bare(&mut game, vanilla_creature(1, 1, &[]), 1);
    let create = Effect::Atom(Primitive::CreateToken(soldier, AmountExpr::Fixed(1)), EffectRecipient::Controller);
    game.resolve_effect(&create, &ResolutionContext::untargeted(source, 1), &ById::new()).expect("created");
    let token = game.battlefield_ids_ordered().into_iter().find(|id| game.get_object(*id).unwrap().is_token).unwrap();

    let clone = clone_of(&mut game, 0, token);
    assert_eq!(get_effective_name(&game, clone), "Soldier Token");
    assert!(has_subtype(&game, clone, &Subtype::Creature(CreatureType::Soldier)));
    assert_eq!(get_effective_power(&game, clone), Some(1));
    assert!(!game.get_object(clone).unwrap().is_token, "the Clone is still a card");
}

// RULING: Clone #2 - "If the copied creature has {X} in its mana cost, X is
//   considered to be 0."
/// The copy's cost has the {X}, and its mana value counts it as 0 (CR
/// 202.3e), which the Clone was never cast with.
#[test]
fn a_clone_of_a_creature_with_x_in_its_cost_has_x_as_zero() {
    let mut game = setup_two_player_game();
    let x_cost = ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Green)]);
    let donor = put_on_battlefield(
        &mut game,
        CardDataBuilder::new("X Creature").card_type(CardType::Creature).mana_cost(x_cost.clone()).power_toughness(2, 2).build(),
        1,
    );
    let clone = clone_of(&mut game, 0, donor);
    let cost = compute_characteristics(&game, clone).unwrap().mana_cost.clone().expect("the copied cost");
    assert_eq!(cost, x_cost);
    assert_eq!(cost.mana_value(), 1);
}

/// Two Essence-shaped fixtures are two CR 616.1c effects, so the entering
/// Bear's controller orders them, and the one applied last wins: a second
/// copy replaces the first rather than merging with it. Essence of the Wild's
/// third ruling, which is unlinked because the card is a fixture here.
#[test]
fn two_essences_give_the_one_applied_last() {
    for (first, last) in [("Essence A", "Essence B"), ("Essence B", "Essence A")] {
        let mut game = setup_two_player_game();
        let a = put_on_battlefield(&mut game, essence_of_the_wild("Essence A", 6, 6), 0);
        let b = put_on_battlefield(&mut game, essence_of_the_wild("Essence B", 4, 4), 0);
        let chosen = if first == "Essence A" { a } else { b };
        let dp = ById::new().then(0, "ChooseReplacementEffect", &[chosen]);
        let bears = enter(&mut game, grizzly_bears(), 0, &dp);
        assert_eq!(get_effective_name(&game, bears), last, "{first} applied first");
    }
}

/// Clone beside Essence of the Wild, the card's seventh ruling, in both
/// orders. Essence first: the entering Clone becomes an Essence, and Clone's
/// own ability, which the would-be permanent no longer has, never applies.
/// Clone first: the entering Clone becomes a copy of the Colossus (the
/// Colossus itself does not change), and Essence's effect, which still
/// applies to a creature entering under its controller, makes that entering
/// copy an Essence. The Clone chooses the Colossus rather than the Essence so
/// the result can only come from Essence's effect applying second. CR 616.1
/// still asks the order, which `classify` cannot prove moot.
#[test]
fn a_clone_beside_essence_of_the_wild_enters_as_the_essence() {
    for clone_first in [false, true] {
        let mut game = setup_two_player_game();
        let essence = put_on_battlefield(&mut game, essence_of_the_wild("Essence of the Wild", 6, 6), 0);
        let big = put_on_battlefield(&mut game, colossus(), 1);
        let clone = put_in_graveyard(&mut game, phase_cv_cards::clone(), 0);
        let dp = if clone_first {
            ById::new().then(0, "ChooseReplacementEffect", &[clone]).then(0, "ChooseCopySource", &[big])
        } else {
            ById::new().then(0, "ChooseReplacementEffect", &[essence])
        };
        game.change_zone(clone, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp)).unwrap();
        assert_eq!(get_effective_name(&game, clone), "Essence of the Wild", "Clone first: {clone_first}");
        assert_eq!(get_effective_power(&game, clone), Some(6));
    }
}

/// CR 616.1c ahead of 616.1e: Rusted Sentinel, cast under Essence of the
/// Wild, becomes an Essence before "enters tapped" could apply, and the copy
/// has no such ability, so it enters untapped (Essence's eighth ruling).
/// Nothing is asked.
// COVERS: ATOM-616.1c-001
#[test]
fn rusted_sentinel_under_essence_of_the_wild_enters_untapped() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, essence_of_the_wild("Essence of the Wild", 6, 6), 0);
    let dp = ManaWindowStop::new(ById::new());
    let sentinel = cast_from_pool(&mut game, 0, rusted_sentinel(), &[(ManaType::Colorless, 4)], &dp).expect("castable");
    game.resolve_top_of_stack(&dp).expect("it resolves");

    assert_eq!(get_effective_name(&game, sentinel), "Essence of the Wild");
    assert!(!game.battlefield[&sentinel].tapped, "the copy has no 'enters tapped' to apply");
    assert!(dp.inner().asked().is_empty());
}

/// Infinite Reflection's "enter as a copy of enchanted creature": the donor
/// is the host of the effect's source, and nothing is asked. Its other half,
/// "each other nontoken creature you control becomes a copy" as it enters, is
/// a triggered "becomes a copy" with no duration, which is CV-1b's.
#[test]
fn a_host_donor_is_the_enchanted_creature() {
    let mut game = setup_two_player_game();
    let big = put_on_battlefield(&mut game, colossus(), 1);
    let reflection = CardDataBuilder::new("Infinite Reflection")
        .card_type(CardType::Enchantment)
        .color(Color::Blue)
        .mana_cost(ManaCost::build(&[ManaType::Blue], 5))
        .ability(replacement(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::battlefield_filter(ObjectFilter::And(
                Box::new(creatures_you_control()),
                Box::new(ObjectFilter::Not(Box::new(ObjectFilter::Token))),
            )),
            Rewrite::EnterAsCopy(EntryCopyTemplate { donor: CopyDonor::Host, except: Vec::new() }),
        ))
        .build();
    let aura = put_on_battlefield(&mut game, reflection, 0);
    assert!(game.attach(aura, big));

    let bears = enter(&mut game, grizzly_bears(), 0, &ById::new());
    assert_eq!(get_effective_name(&game, bears), "Colossus");
    assert_eq!(get_effective_controller(&game, bears), Some(0));
}

// ---------------------------------------------------------------------------
// 3. What the copy brings to its own entry (CR 707.5's last sentence)
// ---------------------------------------------------------------------------

// RULING: Clone #3 - "Any "as [this creature] enters" or "[this creature]
//   enters with" abilities of the chosen creature will also work."
/// CR 707.5 — the copied "enters tapped" and "enters with" abilities apply to
/// the Clone's own entry: a Clone of Skyshroud Behemoth enters tapped with
/// two counters, and a Clone of Chainbreaker with two -1/-1 counters. The
/// donors are untapped and have none.
// COVERS: ATOM-707.5-001
#[test]
fn a_clone_enters_with_what_its_copy_enters_with() {
    let mut game = setup_two_player_game();
    let behemoth = put_on_battlefield(&mut game, skyshroud_behemoth(), 1);
    let clone = clone_of(&mut game, 0, behemoth);
    assert_eq!(get_effective_name(&game, clone), "Skyshroud Behemoth");
    assert!(game.battlefield[&clone].tapped);
    assert_eq!(counters(&game, clone, CounterType::Charge), 2);

    let scarecrow = put_on_battlefield(&mut game, chainbreaker(), 1);
    let clone = clone_of(&mut game, 0, scarecrow);
    assert_eq!(counters(&game, clone, CounterType::MinusOneMinusOne), 2);
    assert_eq!(get_effective_power(&game, clone), Some(1));
}

// RULING: Clone #3 - "Any enters abilities of the copied creature will
//   trigger when Clone enters."
/// CR 707.5 — the copied "when this creature enters" triggers for the Clone:
/// registration filed the copied trigger before the entry was dispatched, so
/// its controller draws. Both libraries are stocked, since the donor's own
/// entry draws for its controller too.
// COVERS: ATOM-707.5-002
#[test]
fn a_clone_of_wall_of_omens_draws_a_card() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 3);
    fill_library(&mut game, 1, 3);
    let wall = put_on_battlefield(&mut game, wall_of_omens(), 1);
    let hand = game.players[0].hand.len();
    let clone = clone_of(&mut game, 0, wall);
    settle(&mut game, &ById::new());

    assert_eq!(get_effective_name(&game, clone), "Wall of Omens");
    assert_eq!(game.players[0].hand.len(), hand + 1, "the Clone's controller drew");
}

/// CR 707.6 — the copied devour is the Clone's "as enters" choice, made by
/// its controller over its controller's creatures: one sacrificed is three
/// +1/+1 counters. The donor's own three counters are not copied (CR 707.2),
/// or the Clone would have six. Partial: the atoms' boards choose a creature
/// type and a color, which is `backlog.md` §2.2's.
// COVERS-PARTIAL: ATOM-707.6-001
// COVERS-PARTIAL: COMP-9A-002
#[test]
fn a_clone_of_thunder_thrash_elder_devours_for_its_own_controller() {
    let mut game = setup_two_player_game();
    let elder = put_on_battlefield(&mut game, thunder_thrash_elder(), 1);
    game.add_counters(elder, CounterType::PlusOnePlusOne, 3);
    let fodder = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let dp = ById::new()
        .then(0, "ChooseCopySource", &[elder])
        .then(0, "ChooseAuxiliaryZoneChange", &[fodder]);
    let clone = enter(&mut game, phase_cv_cards::clone(), 0, &dp);

    assert_eq!(get_effective_name(&game, clone), "Thunder-Thrash Elder");
    assert_eq!(counters(&game, clone, CounterType::PlusOnePlusOne), 3);
    assert_eq!(zone_of(&game, fodder), Zone::Graveyard);
    assert_eq!(counters(&game, elder, CounterType::PlusOnePlusOne), 3, "the donor is untouched");
}

/// CR 616.1f through the frame: Root Maze does not apply to a Clone, which
/// is no artifact, until the copy makes it Chainbreaker. The re-gather then
/// finds both "enters tapped" and the copied "enters with", which commute, so
/// nothing is asked.
#[test]
fn root_maze_taps_a_clone_of_chainbreaker() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, root_maze(), 1);
    let scarecrow = put_on_battlefield(&mut game, chainbreaker(), 1);
    let clone = clone_of(&mut game, 0, scarecrow);

    assert!(game.battlefield[&clone].tapped);
    assert_eq!(counters(&game, clone, CounterType::MinusOneMinusOne), 2);
}

/// CR 608.3e's own example: under Worms of the Earth, a Clone that chose
/// Dryad Arbor would be a land as it enters, so it can't, and it goes from
/// the stack to its owner's graveyard. Cast from hand, so the refusal is a
/// resolving spell's.
// COVERS: ATOM-608.3e-001
#[test]
fn worms_of_the_earth_refuses_a_clone_of_dryad_arbor() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, worms_of_the_earth(), 1);
    let arbor = put_on_battlefield(&mut game, dryad_arbor(), 1);
    let dp = ManaWindowStop::new(ById::new().then(0, "ChooseCopySource", &[arbor]));
    let clone = cast_from_pool(&mut game, 0, phase_cv_cards::clone(), &[(ManaType::Blue, 4)], &dp).expect("castable");
    game.resolve_top_of_stack(&dp).expect("it resolves; it cannot arrive");

    assert_eq!(zone_of(&game, clone), Zone::Graveyard);
    assert!(game.stack.is_empty());
    assert!(!game.battlefield.contains_key(&clone));
}

// ---------------------------------------------------------------------------
// 4. Every gate sees the copied abilities (D5)
// ---------------------------------------------------------------------------

/// The replacement gate: a Clone of an opponent's Master Biomancer gives the
/// next creature its controller's entering two +1/+1 counters, and makes it
/// a Mutant.
#[test]
fn a_clone_of_master_biomancer_modifies_the_next_entry() {
    let mut game = setup_two_player_game();
    let biomancer = put_on_battlefield(&mut game, master_biomancer(), 1);
    clone_of(&mut game, 0, biomancer);
    let bears = enter(&mut game, grizzly_bears(), 0, &ById::new());

    assert_eq!(counters(&game, bears, CounterType::PlusOnePlusOne), 2);
    assert!(has_subtype(&game, bears, &Subtype::Creature(CreatureType::Mutant)));
}

/// The restriction gate: a Clone of an opponent's Sigarda keeps that
/// opponent's edict from making its controller sacrifice. Two creatures, so
/// without the restriction the edict would ask which.
#[test]
fn a_clone_of_sigarda_keeps_its_controller_from_sacrificing() {
    let mut game = setup_two_player_game();
    let sigarda = put_on_battlefield(&mut game, sigarda_host_of_herons(), 1);
    let clone = clone_of(&mut game, 0, sigarda);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);

    resolve_spell(&mut game, diabolic_edict(), 1, vec![ResolvedTarget::Player(0)], &ById::new());
    assert_eq!(zone_of(&game, clone), Zone::Battlefield);
    assert_eq!(zone_of(&game, bears), Zone::Battlefield);
}

/// The cost gate: a Clone of an opponent's Thalia taxes noncreature spells
/// beside the original, so a {0} instant costs exactly {2}.
#[test]
fn a_clone_of_thalia_taxes_noncreature_spells() {
    let mut game = setup_two_player_game();
    let thalia = put_on_battlefield(&mut game, thalia_guardian_of_thraben(), 1);
    clone_of(&mut game, 0, thalia);

    let dp = ManaWindowStop::new(ById::new());
    cast_from_pool(&mut game, 0, idle_thought(), &[(ManaType::Colorless, 2)], &dp).expect("castable for {2}");
    assert_eq!(game.players[0].mana_pool.total(), 0, "two Thalias, two generic");
}

/// The trigger gate: a Clone of an opponent's Soul Warden gains its
/// controller life when the next creature enters, beside the original.
#[test]
fn a_clone_of_soul_warden_triggers_on_the_next_entry() {
    let mut game = setup_two_player_game();
    let warden = put_on_battlefield(&mut game, soul_warden(), 1);
    clone_of(&mut game, 0, warden);
    settle(&mut game, &ById::new());
    assert_eq!(game.players[1].life_total, 21, "the original saw the Clone enter");

    enter(&mut game, grizzly_bears(), 0, &ById::new());
    settle(&mut game, &ById::new());
    assert_eq!((game.players[0].life_total, game.players[1].life_total), (21, 22));
}

/// A copied static ability registers its rows: a Clone of an opponent's
/// Citanul Hierophants gives its controller's creatures "{T}: Add {G}", and
/// not the opponent's.
#[test]
fn a_clone_of_citanul_hierophants_grants_the_mana_ability() {
    let mut game = setup_two_player_game();
    let hierophants = put_on_battlefield(&mut game, citanul_hierophants(), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let theirs = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let mana_abilities = |game: &GameState, id| {
        get_effective_abilities(game, id).iter().filter(|a| a.ability_type == AbilityType::Mana).count()
    };
    assert_eq!(mana_abilities(&game, bears), 0);

    clone_of(&mut game, 0, hierophants);
    assert_eq!(mana_abilities(&game, bears), 1);
    assert_eq!(mana_abilities(&game, theirs), 1, "the original's one grant, and not the Clone's");
}

// ---------------------------------------------------------------------------
// 5. The carrier is state: it leaves with the permanent, and a row applies
//    over it
// ---------------------------------------------------------------------------

/// CR 400.7 — the Clone in the graveyard is a Clone card, and its copied
/// anthem's rows went with it.
#[test]
fn a_clone_in_the_graveyard_is_a_clone_and_its_rows_are_gone() {
    let mut game = setup_two_player_game();
    let anthem = put_on_battlefield(&mut game, anthem_bear(), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let clone = clone_of(&mut game, 0, anthem);
    assert_eq!(get_effective_power(&game, bears), Some(3));

    let dp = ById::new();
    let ctx = ActionContext::new(&dp);
    game.change_zone(clone, Zone::Graveyard, ZoneChangeCause::Destroyed, &ctx).unwrap();
    assert_eq!(get_effective_name(&game, clone), "Clone");
    assert_eq!(get_effective_power(&game, bears), Some(2));
    assert!(!game.continuous_effects.iter().any(|e| e.source == clone), "no row outlives its source");
}

/// CR 707.4 — Cytoshape over a Clone applies after the entry copy, by
/// timestamp, and when it expires at cleanup the Clone is its entry copy
/// again, not a 0/0 Clone.
#[test]
fn a_copy_effect_over_a_clone_expires_back_to_the_entry_copy() {
    let mut game = setup_two_player_game();
    let big = put_on_battlefield(&mut game, colossus(), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let clone = clone_of(&mut game, 0, big);

    let dp = ById::new().then(0, "ChooseCopySource", &[bears]);
    resolve_spell(&mut game, cytoshape(), 0, vec![ResolvedTarget::Object(clone)], &dp);
    assert_eq!(get_effective_name(&game, clone), "Grizzly Bears");

    game.continuous_effects.remove_expired_at_cleanup(0, game.turn_number);
    assert_eq!(get_effective_name(&game, clone), "Colossus");
    assert_eq!(get_effective_power(&game, clone), Some(5));
}

/// CR 704.5j — a Clone of its controller's own legend is a second legend of
/// that name, and the controller keeps one.
#[test]
fn a_clone_of_your_own_legend_meets_the_legend_rule() {
    let mut game = setup_two_player_game();
    let thalia = put_on_battlefield(&mut game, thalia_guardian_of_thraben(), 0);
    let clone = clone_of(&mut game, 0, thalia);
    assert!(compute_characteristics(&game, clone).unwrap().supertypes.contains(&Supertype::Legendary));

    settle(&mut game, &ById::new().then(0, "LegendRule", &[clone]));
    assert_eq!(zone_of(&game, thalia), Zone::Graveyard);
    assert_eq!(zone_of(&game, clone), Zone::Battlefield);
}

/// CR 903.3 — the commander designation is not a copiable value: a Clone of
/// a commander is no commander, and its combat damage is not commander damage.
// COVERS: ATOM-903.3-002
#[test]
fn a_clone_of_a_commander_is_not_a_commander() {
    let mut game = setup_two_player_game();
    let legend = CardDataBuilder::new("Isamaru, Hound of Konda")
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .power_toughness(2, 2)
        .build();
    let commander = put_on_battlefield(&mut game, legend, 0);
    game.objects.get_mut(&commander).unwrap().is_commander = true;
    let clone = clone_of(&mut game, 1, commander);

    assert_eq!(get_effective_name(&game, clone), "Isamaru, Hound of Konda");
    assert!(!game.get_object(clone).unwrap().is_commander);
    let dp = ById::new();
    let ctx = ActionContext::new(&dp);
    let hit = |source| GameAction::DealDamage {
        source,
        target: DamageTarget::Player(0),
        amount: 2,
        is_combat: true,
        unpreventable: false,
    };
    game.execute_action(hit(clone), &ctx).unwrap();
    assert_eq!(game.players[0].life_total, 18);
    assert!(game.players[0].commander_damage_taken.is_empty());
}

// ---------------------------------------------------------------------------
// Loyalty: a characteristic (CR 109.3) and a copiable value (CR 707.2)
// ---------------------------------------------------------------------------

/// CR 306.5a — off the battlefield, a planeswalker card's loyalty is its
/// printed number: Ajani Goldmane's 4, in a hand and in a graveyard.
// COVERS: ATOM-306.5a-001
#[test]
fn a_planeswalker_card_has_its_printed_loyalty() {
    let mut game = setup_two_player_game();
    let ajani = || {
        CardDataBuilder::new("Ajani Goldmane")
            .card_type(CardType::Planeswalker)
            .color(Color::White)
            .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 2))
            .loyalty(4)
            .build()
    };
    for id in [put_in_hand(&mut game, ajani(), 0), put_in_graveyard(&mut game, ajani(), 0)] {
        assert_eq!(compute_characteristics(&game, id).unwrap().loyalty, Some(4));
    }
}

/// CR 306.5b asked of what the Clone enters as: a copy of a creature that is
/// a planeswalker enters with the copy's loyalty, and Doubling Season doubles
/// it as it would the card's own.
#[test]
fn a_clone_of_a_planeswalker_creature_enters_with_its_loyalty() {
    for (doubled, expected) in [(false, 3), (true, 6)] {
        let mut game = setup_two_player_game();
        if doubled {
            put_on_battlefield(&mut game, doubling_season(), 0);
        }
        let walker = put_on_battlefield(&mut game, planeswalker_creature(), 1);
        let clone = clone_of(&mut game, 0, walker);
        assert_eq!(counters(&game, clone, CounterType::Loyalty), expected, "Doubling Season: {doubled}");
        assert_eq!(compute_characteristics(&game, clone).unwrap().loyalty, Some(3));
    }
}
