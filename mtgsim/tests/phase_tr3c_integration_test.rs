//! TR-3c — the reflexive trigger, and the sources of damage that have left
//! (`triggers-architecture.md`, TR-3c).
//!
//! 1. `codebase-state.md` item 225: a shield whose chosen source has left
//!    still prevents that source's damage, dealt as it last existed (CR
//!    608.2h, 609.7a), and no damage of the object it became (CR 400.7).
//! 2. Item 99's first half: CR 609.7a's three referred-to categories, each
//!    existence once and told apart from the object it became.
//! 3. Item 103's last-known-information half: a source the store has lost
//!    is matched as it last existed.
//! 4. CR 603.12: the reflexive trigger. CR 603.12's example on a
//!    Heart-Piercer Manticore fixture with its four trigger rulings and item
//!    229's "another"; Cornered Crook, its ruling, and the Crook killed in
//!    response; the "doesn't" form; CR 603.12a.

use std::cell::RefCell;
use std::sync::Arc;

use mtgsim::cards::authoring::{another, dies, enters, sacrificed, triggered_ability, whenever};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::keyword_creatures::wall_of_stone;
use mtgsim::cards::phase5_pre_cards::glorious_anthem;
use mtgsim::cards::phase_lh_cards::loxodon_warhammer;
use mtgsim::cards::phase_rd_cards::{circle_of_protection_red, reverse_damage};
use mtgsim::cards::phase_tr1_cards::blood_artist;
use mtgsim::cards::phase_tr3b_cards::flickerwisp;
use mtgsim::cards::phase_tr3c_cards::cornered_crook;
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::event::{DamageTarget, GameEvent, NamesAsAnnounced};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::oracle::legality::damage_sources;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    card_of_type, put_in_hand, put_on_battlefield, setup_game, setup_two_player_game, static_ability, test_ctx, test_dp,
    vanilla_creature, RecordingDecisionProvider,
};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::colors::Color;
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::{
    AmountExpr, Choice, ChoiceScope, ChoiceSide, Duration, Effect, EffectRecipient, ObjectFilter, Pick, PlayerRef,
    Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{AbilityId, ObjectId, ObjectRef, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::triggers::{
    DelayedDuration, DelayedTriggerTemplate, DelayedTurn, ReflexiveForm, TriggerEvent, TriggerSubject,
};
use mtgsim::types::zones::{DestructionSource, Zone, ZoneChangeCause};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use mtgsim::ui::display::format_event;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// Fixtures and helpers
// ---------------------------------------------------------------------------

/// "[This] deals `n` damage to any target."
fn deals(n: u64) -> Effect {
    Effect::Atom(
        Primitive::DealDamage { amount: AmountExpr::Fixed(n), unpreventable: false },
        EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
    )
}

/// Scorchfuse Myr — a fixture under no real card's name: Perilous Myr's
/// shape, red, so that Circle of Protection: Red may choose it.
///
/// > When this creature dies, it deals 2 damage to any target.
fn scorchfuse_myr() -> Arc<CardData> {
    let text = "When this creature dies, it deals 2 damage to any target.";
    CardDataBuilder::new("Scorchfuse Myr")
        .mana_cost(ManaCost::build(&[ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Myr))
        .power_toughness(1, 1)
        .rules_text(text)
        .ability(triggered_ability(text, whenever(dies(TriggerSubject::ThisObject), deals(2))))
        .build()
}

/// A red creature with no ability.
fn red_probe() -> Arc<CardData> {
    CardDataBuilder::new("Red Probe")
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .build()
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

fn destroy(game: &mut GameState, object: ObjectId) {
    let by = mtgsim::types::ids::new_object_id();
    game.execute_action(GameAction::Destroy { object, source: DestructionSource::Effect(by) }, &test_ctx())
        .unwrap();
}

/// Put every waiting trigger on the stack and resolve the stack, answering
/// every prompt with `dp`.
fn resolve_all(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(dp).unwrap();
        game.perform_sba_and_triggers(dp).unwrap();
    }
}

/// Answers CR 609.7a's choice of a source with the option naming `wanted`,
/// keeping the options it was offered as the log words them; every other
/// prompt takes its first option.
struct ChoosingSource {
    wanted: ChoiceOption,
    offered: RefCell<Vec<String>>,
    rest: RecordingDecisionProvider,
}

impl ChoosingSource {
    fn new(wanted: ChoiceOption) -> Self {
        ChoosingSource { wanted, offered: RefCell::new(Vec::new()), rest: RecordingDecisionProvider::picking(0) }
    }

    fn offered(&self) -> Vec<String> {
        self.offered.borrow().clone()
    }
}

impl DecisionProvider for ChoosingSource {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        ctx: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        if !matches!(ctx.kind, ChoiceKind::ChooseDamageSource { .. }) {
            return self.rest.pick_n(game, player, ctx, options, bounds);
        }
        let logged: Vec<String> = options.iter().map(|o| o.as_logged(game)).collect();
        let wanted = self.wanted.as_logged(game);
        let at = logged.iter().position(|o| *o == wanted).unwrap_or_else(|| panic!("{wanted} not offered: {logged:?}"));
        *self.offered.borrow_mut() = logged;
        vec![at]
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, ctx: &ChoiceContext, min: u64, max: u64) -> u64 {
        self.rest.pick_number(game, player, ctx, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        ctx: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        mins: &[u64],
        maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.rest.allocate(game, player, ctx, total, buckets, mins, maxs)
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, ctx: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        self.rest.choose_ordering(game, player, ctx, items)
    }
}

/// Circle of Protection: Red's effect, resolved for its controller P0 as its
/// ability resolves, choosing `wanted` as CR 609.7a's source. Returns the
/// options offered, as the log words them.
fn circle_chooses(game: &mut GameState, circle: ObjectId, wanted: ChoiceOption) -> Vec<String> {
    let ctx = ResolutionContext {
        source: circle,
        ability_source: game.object_ref(circle),
        controller: 0,
        targets: ChosenTargets::NONE,
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    let dp = ChoosingSource::new(wanted);
    game.resolve_effect(&circle_of_protection_red().abilities[0].effect, &ctx, &dp).unwrap();
    dp.offered()
}

/// Activate Circle of Protection: Red for P0 from a pool of exactly `{1}`
/// and resolve it, choosing `wanted`. Returns the options offered.
fn activate_circle(game: &mut GameState, circle: ObjectId, wanted: ChoiceOption) -> Vec<String> {
    game.players[0].mana_pool.add(ManaType::White, 1);
    let dp = ManaWindowStop::new(ChoosingSource::new(wanted));
    game.activate_ability(0, circle, 0, &dp).unwrap();
    game.resolve_top_of_stack(&dp).unwrap();
    dp.inner().offered()
}

/// A source as it is now.
fn now(id: ObjectId) -> ChoiceOption {
    ChoiceOption::Object(id)
}

/// A source as it last existed, once it has left.
fn as_it_was(object: ObjectRef) -> ChoiceOption {
    ChoiceOption::Departed(object)
}

fn logged(game: &GameState, options: &[ChoiceOption]) -> Vec<String> {
    options.iter().map(|o| o.as_logged(game)).collect()
}

// ---------------------------------------------------------------------------
// Item 225: a chosen source that has left
// ---------------------------------------------------------------------------

/// Item 225's board. Circle of Protection: Red chose the Myr while it was on
/// the battlefield; the Myr dies, and its trigger has it deal 2 damage to the
/// Circle's controller as it last existed (CR 608.2h). The shield watches
/// that existence, so the damage is prevented (CR 609.7a). It used to be
/// dropped as the Myr died.
#[test]
fn a_shield_prevents_the_damage_its_chosen_source_deals_after_it_died() {
    let mut game = setup_two_player_game();
    let circle = put_on_battlefield(&mut game, circle_of_protection_red(), 0);
    let myr = put_on_battlefield(&mut game, scorchfuse_myr(), 1);
    circle_chooses(&mut game, circle, now(myr));

    destroy(&mut game, myr);
    // The Myr's controller, P1, targets P0: players are offered first.
    resolve_all(&mut game, &RecordingDecisionProvider::picking(0));

    assert_eq!(life(&game, 0), 20, "the Myr that died is the source the Circle chose");
    assert!(game.replacement_effects.is_empty(), "and the shield is spent (CR 615.8)");
}

/// The other half of CR 400.7: the shield watches the existence it chose,
/// so the card the Myr became, put back onto the battlefield, is a new
/// object whose damage it does not prevent.
#[test]
fn a_shield_does_not_prevent_the_damage_of_the_object_its_source_became() {
    let mut game = setup_two_player_game();
    let circle = put_on_battlefield(&mut game, circle_of_protection_red(), 0);
    let probe = put_on_battlefield(&mut game, red_probe(), 1);
    circle_chooses(&mut game, circle, now(probe));

    game.change_zone(probe, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx()).unwrap();
    game.change_zone(probe, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx()).unwrap();
    game.execute_action(
        GameAction::DealDamage {
            source: probe,
            target: DamageTarget::Player(0),
            amount: 2,
            is_combat: false,
            unpreventable: false,
            source_frame: None,
        },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(life(&game, 0), 18, "a new object (CR 400.7)");
    assert_eq!(game.replacement_effects.len(), 1, "and the shield waits for the one it chose");
}

// ---------------------------------------------------------------------------
// Item 99: CR 609.7a's referred-to sources
// ---------------------------------------------------------------------------

/// CR 609.7a's first referred-to category: an object an object on the
/// stack refers to, "even if that object is no longer in the zone it used to
/// be in". With the Myr's dies trigger on the stack, the Circle may choose the
/// Myr as it died, which is the source that deals the trigger's damage; the
/// card in the graveyard is a new object, and is not offered.
#[test]
fn the_circle_may_choose_a_source_that_died_while_its_trigger_waits_on_the_stack() {
    let mut game = setup_two_player_game();
    let circle = put_on_battlefield(&mut game, circle_of_protection_red(), 0);
    let myr = put_on_battlefield(&mut game, scorchfuse_myr(), 1);
    let was = game.object_ref(myr).unwrap();
    destroy(&mut game, myr);
    game.perform_sba_and_triggers(&RecordingDecisionProvider::picking(0)).unwrap();
    assert_eq!(game.stack.len(), 1, "the Myr's trigger, targeting P0");

    let offered = activate_circle(&mut game, circle, as_it_was(was));
    assert_eq!(offered, logged(&game, &[now(circle), as_it_was(was)]));

    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), 20, "the damage the Myr dealt as it last existed is prevented");
}

/// CR 609.7a's second referred-to category: an object a prevention effect
/// waiting to apply refers to. The first shield chose the probe, which has
/// since died, and nothing on the stack names it; a second choice is
/// offered the probe as it died, because that shield names it.
#[test]
fn a_source_a_waiting_shield_names_is_offered_after_it_has_left() {
    let mut game = setup_two_player_game();
    let circle = put_on_battlefield(&mut game, circle_of_protection_red(), 0);
    let probe = put_on_battlefield(&mut game, red_probe(), 1);
    let was = game.object_ref(probe).unwrap();
    circle_chooses(&mut game, circle, now(probe));
    destroy(&mut game, probe);
    assert!(game.stack.is_empty());

    assert_eq!(damage_sources(&game, None), vec![game.object_ref(circle).unwrap(), was]);
}

/// CR 609.7a's third referred-to category: an object a delayed triggered
/// ability waiting to trigger refers to. Flickerwisp's return names the card
/// it exiled, which is offered where it is; Flickerwisp, the trigger's
/// source, is offered as it last existed once it has died. A card in a hand
/// that nothing names is no source (BOUNDARY-DEF-609.7a-001's out-of-set
/// member), and neither is the card the exiled one becomes in a hand, while
/// the existence the trigger names still is.
// COVERS: BOUNDARY-DEF-609.7a-001
// COVERS-PARTIAL: ATOM-609.7a-001 -- the permanent, the spell and the
// three referred-to legs; the fourth category, a face-up object in the
// command zone (the atom's emblem), waits for the command zone (item 99's
// second half, B2).
#[test]
fn what_a_waiting_delayed_trigger_names_is_a_source_wherever_it_has_gone() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let wisp = put_on_battlefield(&mut game, flickerwisp(), 0);
    let stray = put_in_hand(&mut game, grizzly_bears(), 0);
    game.change_zone(wisp, Zone::Exile, ZoneChangeCause::Exiled, &test_ctx()).unwrap();
    game.change_zone(wisp, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx()).unwrap();
    let wisp_was = game.object_ref(wisp).unwrap();
    resolve_all(&mut game, &RecordingDecisionProvider::picking(0));
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Exile);
    assert_eq!(game.delayed_triggers.len(), 1);
    let exiled = game.object_ref(bears).unwrap();

    let sources = damage_sources(&game, None);
    assert!(sources.contains(&exiled), "the card the return names, in exile");
    assert!(!sources.iter().any(|s| s.id == stray), "a card in hand that nothing names");

    destroy(&mut game, wisp);
    game.change_zone(bears, Zone::Hand, ZoneChangeCause::Returned, &test_ctx()).unwrap();
    let sources = damage_sources(&game, None);
    assert!(sources.contains(&wisp_was), "the trigger's source, as it last existed");
    assert!(sources.contains(&exiled), "the card it names, though it has left exile");
    assert!(!sources.contains(&game.object_ref(bears).unwrap()), "and not the card it became in a hand");
}

/// Two existences of one id are two sources, told apart. The Myr dies and
/// is returned to the battlefield while its trigger waits: the permanent is
/// the object as it is, and the trigger's source is the Myr as it died.
/// Choosing the second prevents the trigger's damage, and none of the new
/// permanent's.
#[test]
fn two_existences_of_one_object_are_two_sources_told_apart() {
    let mut game = setup_two_player_game();
    let circle = put_on_battlefield(&mut game, circle_of_protection_red(), 0);
    let myr = put_on_battlefield(&mut game, scorchfuse_myr(), 1);
    let was = game.object_ref(myr).unwrap();
    destroy(&mut game, myr);
    game.perform_sba_and_triggers(&RecordingDecisionProvider::picking(0)).unwrap();
    game.change_zone(myr, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx()).unwrap();

    let offered = activate_circle(&mut game, circle, as_it_was(was));
    assert_eq!(offered, logged(&game, &[now(circle), now(myr), as_it_was(was)]));
    assert_ne!(offered[1], offered[2], "the log tells them apart");

    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), 20, "the trigger's damage, dealt by the Myr that died");
    game.execute_action(
        GameAction::DealDamage {
            source: myr,
            target: DamageTarget::Player(0),
            amount: 1,
            is_combat: false,
            unpreventable: false,
            source_frame: None,
        },
        &test_ctx(),
    )
    .unwrap();
    assert_eq!(life(&game, 0), 19, "the Myr on the battlefield is not the source chosen");
}

/// CR 608.2 keeps a spell on the stack while it resolves, so Reverse Damage,
/// choosing its source as it resolves, may choose itself: item 99's last
/// gap, a spell whose stack entry its resolution had taken.
#[test]
fn a_spell_may_choose_itself_as_the_source_while_it_resolves() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let reverse = put_in_hand(&mut game, reverse_damage(), 0);
    game.players[0].mana_pool.add(ManaType::White, 3);
    game.cast_spell(0, reverse, &ManaWindowStop::new(RecordingDecisionProvider::picking(0))).unwrap();

    let dp = ChoosingSource::new(now(reverse));
    game.resolve_top_of_stack(&dp).unwrap();
    assert_eq!(dp.offered(), logged(&game, &[now(bears), now(reverse)]));
}

// ---------------------------------------------------------------------------
// Item 103: a source the store has lost
// ---------------------------------------------------------------------------

/// A token that died ceases to exist (CR 704.5d), and the store loses it
/// before its dies trigger resolves. Its damage is dealt as it last existed
/// (CR 608.2h), so the shield's "red" is asked of that frame, which answers
/// though no object is left to ask (`codebase-state.md` item 103).
#[test]
fn a_token_that_ceased_to_exist_is_red_as_it_last_existed() {
    let mut game = setup_two_player_game();
    let circle = put_on_battlefield(&mut game, circle_of_protection_red(), 0);
    let token = put_on_battlefield(&mut game, scorchfuse_myr(), 1);
    game.objects.get_mut(&token).unwrap().is_token = true;
    circle_chooses(&mut game, circle, now(token));

    destroy(&mut game, token);
    game.perform_sba_and_triggers(&RecordingDecisionProvider::picking(0)).unwrap();
    assert!(game.objects.get(&token).is_none(), "the token has ceased to exist");

    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), 20, "prevented: the token was red");
}

/// Item 103's last-known-information half, met by a trigger. A creature dies
/// in the state-based check that makes its owner lose, and leaves the game
/// with them (CR 800.4a) before that check's triggers are found. Blood Artist
/// sees it die as it last existed, a creature (CR 603.10a). The filter used
/// to ask the store for the creature and read its absence as no: two of 200
/// four-seat `performance` games met it at TR-3c's sitting.
#[test]
fn a_creature_whose_owner_lost_as_it_died_is_still_seen_to_die() {
    let mut game = setup_game(4);
    put_on_battlefield(&mut game, blood_artist(), 0);
    let doomed = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    game.battlefield.get_mut(&doomed).unwrap().damage_marked = 2;
    game.players[1].life_total = 0;

    game.perform_sba_and_triggers(&RecordingDecisionProvider::picking(0)).unwrap();
    assert!(!game.objects.contains_key(&doomed), "it died, then left the game with its owner");
    let blood_artist_triggered = game
        .recorded_events()
        .events()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { controller: 0, .. }))
        .count();
    assert_eq!(blood_artist_triggered, 1, "Blood Artist saw it die");
}

/// Marrowgnaw Ghoul — a fixture: "Sacrifice another creature: You gain 1
/// life." The shape 162 printed costs share (Scryfall, 2026-10-10).
fn marrowgnaw_ghoul() -> Arc<CardData> {
    let text = "Sacrifice another creature: You gain 1 life.";
    CardDataBuilder::new("Marrowgnaw Ghoul")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .rules_text(text)
        .ability(AbilityDef {
            rules_text: text.into(),
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Activated,
            costs: vec![Cost::Sacrifice(another(ObjectFilter::ByType(CardType::Creature)), 1)],
            effect: Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
        })
        .build()
}

/// "Another" in a cost is other than the object whose cost it is (CR
/// 113.7a), asked as a resolution's "another" is. The cost's filter used to
/// be asked with no source, which refused the leaf and read the refusal as
/// "no creature to sacrifice"; found by item 103's review, which routed every
/// filter refusal through one rule.
#[test]
fn a_cost_sacrifices_another_creature_and_never_its_own_source() {
    let mut game = setup_two_player_game();
    let ghoul = put_on_battlefield(&mut game, marrowgnaw_ghoul(), 0);
    assert!(game.activate_ability(0, ghoul, 0, &test_dp()).is_err(), "alone, it has nothing else to sacrifice");

    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.activate_ability(0, ghoul, 0, &test_dp()).unwrap();
    resolve_all(&mut game, &test_dp());
    assert_eq!((zone(&game, bear), zone(&game, ghoul)), (Zone::Graveyard, Zone::Battlefield));
    assert_eq!(life(&game, 0), 21);
}

// ---------------------------------------------------------------------------
// CR 603.12: the reflexive trigger
// ---------------------------------------------------------------------------

/// "You may sacrifice [a `filter`]", then `when`.
fn may_sacrifice_then(filter: ObjectFilter, when: DelayedTriggerTemplate) -> Effect {
    Effect::Sequence(vec![sacrifice(PlayerRef::You, Pick::exactly(1, filter)), create(when)])
}

/// "[The chooser] may sacrifice `pick`" — or must, for `PlayerRef::Player`'s
/// absence: [`sacrifice_now`].
fn sacrifice(chooser: PlayerRef, pick: Pick) -> Effect {
    Effect::Optional { chooser, effect: Box::new(sacrifice_now(pick)) }
}

/// "Sacrifice `pick`", of the controller's own permanents (CR 701.21a).
fn sacrifice_now(pick: Pick) -> Effect {
    Effect::Atom(
        Primitive::Sacrifice,
        EffectRecipient::ChosenBy(Box::new(Choice {
            chooser: EffectRecipient::Controller,
            among: ChoiceScope::ChoosersPermanents,
            picks: vec![pick],
            acts_on: ChoiceSide::Chosen,
        })),
    )
}

fn create(template: DelayedTriggerTemplate) -> Effect {
    Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(template)), EffectRecipient::Controller)
}

/// A reflexive trigger of `form` on "you sacrifice" (CR 603.12).
fn when_you(form: ReflexiveForm, effect: Effect, text: &'static str) -> DelayedTriggerTemplate {
    reflexive(form, sacrificed(ObjectFilter::ByController(PlayerRef::You)).into(), effect, text)
}

fn reflexive(form: ReflexiveForm, event: TriggerEvent, effect: Effect, text: &'static str) -> DelayedTriggerTemplate {
    DelayedTriggerTemplate {
        def: Arc::new(whenever(event, effect)),
        duration: DelayedDuration::Reflexive(form),
        turn: DelayedTurn::Any,
        rules_text: text.into(),
    }
}

/// Spitefang Manticore — a fixture under no real card's name: Heart-Piercer
/// Manticore without embalm, which keeps that card unregistered. It is CR
/// 603.12's own example.
///
/// > When this creature enters, you may sacrifice another creature. When you
/// > do, this creature deals damage equal to that creature's power to any
/// > target.
fn spitefang_manticore() -> Arc<CardData> {
    let text = "When this creature enters, you may sacrifice another creature. When you do, this creature deals damage equal to that creature's power to any target.";
    let that_creatures_power = Effect::Atom(
        Primitive::DealDamage { amount: AmountExpr::TriggeringPower, unpreventable: false },
        EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
    );
    let when_you_do = when_you(
        ReflexiveForm::Does,
        that_creatures_power,
        "When you do, this creature deals damage equal to that creature's power to any target.",
    );
    CardDataBuilder::new("Spitefang Manticore")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Manticore))
        .power_toughness(4, 3)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(
                enters(TriggerSubject::ThisObject),
                may_sacrifice_then(another(ObjectFilter::ByType(CardType::Creature)), when_you_do),
            ),
        ))
        .build()
}

/// Bonegrinder Altar — a fixture: a sacrifice that is no instruction of a
/// resolving ability.
///
/// > Sacrifice a creature: You gain 1 life.
fn bonegrinder_altar() -> Arc<CardData> {
    CardDataBuilder::new("Bonegrinder Altar")
        .card_type(CardType::Artifact)
        .ability(AbilityDef {
            rules_text: "Sacrifice a creature: You gain 1 life.".into(),
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Activated,
            costs: vec![Cost::Sacrifice(ObjectFilter::ByType(CardType::Creature), 1)],
            effect: Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
        })
        .build()
}

/// Venomweave Charm — a fixture: "Creatures you control have deathtouch."
fn venomweave_charm() -> Arc<CardData> {
    CardDataBuilder::new("Venomweave Charm")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::GrantKeywordFlag(KeywordFlag::Deathtouch, Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(ObjectFilter::And(
                Box::new(ObjectFilter::ByType(CardType::Creature)),
                Box::new(ObjectFilter::ByController(PlayerRef::You)),
            )),
        )))
        .build()
}

/// Withheld Tithe — a fixture for CR 603.12's "doesn't" form, which no
/// printed reflexive trigger uses (Scryfall, 2026-10-09).
///
/// > You may sacrifice a creature. When you don't, you lose 2 life.
fn withheld_tithe() -> Arc<CardData> {
    let lose_2 = Effect::Atom(Primitive::LoseLife(AmountExpr::Fixed(2)), EffectRecipient::Controller);
    CardDataBuilder::new("Withheld Tithe")
        .mana_cost(ManaCost::build(&[ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Instant)
        .ability(spell_ability(may_sacrifice_then(
            ObjectFilter::ByType(CardType::Creature),
            when_you(ReflexiveForm::Doesnt, lose_2, "When you don't, you lose 2 life."),
        )))
        .build()
}

/// Bloodtithe Rite — a fixture for CR 603.12a, both of its sentences' shapes:
///
/// > Sacrifice two creatures. When a creature is sacrificed this way, you
/// > gain 1 life. When one or more creatures are sacrificed this way, draw
/// > a card.
fn bloodtithe_rite() -> Arc<CardData> {
    let gain_1 = Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller);
    let draw = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller);
    let each = sacrificed(TriggerSubject::Any);
    let one_or_more = sacrificed(TriggerSubject::Any).once_per_event();
    CardDataBuilder::new("Bloodtithe Rite")
        .mana_cost(ManaCost::build(&[ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Instant)
        .ability(spell_ability(Effect::Sequence(vec![
            sacrifice_now(Pick::exactly(2, ObjectFilter::ByType(CardType::Creature))),
            create(reflexive(ReflexiveForm::Does, each.into(), gain_1, "When a creature is sacrificed this way, you gain 1 life.")),
            create(reflexive(ReflexiveForm::Does, one_or_more, draw, "When one or more creatures are sacrificed this way, draw a card.")),
        ])))
        .build()
}

fn spell_ability(effect: Effect) -> AbilityDef {
    AbilityDef {
        rules_text: "".into(),
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Spell,
        costs: Vec::new(),
        effect,
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    }
}

/// Empty `player`'s pool, fill it with exactly `pool`, and cast `card` from
/// hand under `ManaWindowStop`, as a shipped client does; then resolve it.
fn cast_and_resolve(game: &mut GameState, player: PlayerId, card: Arc<CardData>, pool: &[(ManaType, u64)]) -> ObjectId {
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
    game.cast_spell(player, id, &ManaWindowStop::new(RecordingDecisionProvider::picking(0)))
        .expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).unwrap();
    id
}

/// Put the waiting triggers on the stack, asking nothing.
fn place(game: &mut GameState) {
    game.perform_sba_and_triggers(&test_dp()).unwrap();
}

/// Put the one waiting trigger on the stack with `target` as its target.
fn place_targeting(game: &mut GameState, target: ChoiceOption) {
    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(select(), vec![target]);
    game.perform_sba_and_triggers(&dp).unwrap();
    assert!(dp.is_empty(), "its target was chosen as it was put on the stack");
}

/// Resolve the trigger on top, answering its "may" with `yes`, and choosing
/// `sacrifice` where the sacrifice is a choice.
fn resolve_may(game: &mut GameState, yes: bool, sacrifice: Option<ObjectId>) {
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::ApplyOptionalEffect { source: ObjectId::UNASSIGNED }, if yes { vec![0] } else { vec![] });
    if let Some(chosen) = sacrifice {
        dp.expect_choice(select(), vec![ChoiceOption::Object(chosen)]);
    }
    game.resolve_top_of_stack(&dp).unwrap();
    assert!(dp.is_empty(), "the may, and the sacrifice where it was a choice");
}

fn select() -> ChoiceKind {
    ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED }
}

fn zone(game: &GameState, id: ObjectId) -> Zone {
    game.get_object(id).unwrap().zone
}

fn top_targets(game: &GameState) -> Vec<Vec<ResolvedTarget>> {
    let top = *game.stack.last().expect("something on the stack");
    game.stack_entries[&top].chosen_targets.iter().map(|i| i.as_resolved_targets().collect()).collect()
}

/// The words of each reflexive trigger's creation, in the log.
fn reflexive_lines(game: &GameState) -> Vec<String> {
    let names = NamesAsAnnounced::default();
    game.recorded_events()
        .events()
        .filter(|e| matches!(e, GameEvent::DelayedTriggerCreated { duration: DelayedDuration::Reflexive(_), .. }))
        .map(|e| format_event(game, e, &names))
        .collect()
}

/// CR 603.12's example: the Manticore enters, its controller sacrifices a
/// 3-power creature, and the reflexive trigger deals 3 to the target chosen
/// as it goes on the stack.
// COVERS: ATOM-603.12-001
#[test]
fn the_manticore_deals_damage_equal_to_the_sacrificed_creatures_power() {
    let mut game = setup_two_player_game();
    let ogre = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    let manticore = cast_and_resolve(&mut game, 0, spitefang_manticore(), &[(ManaType::Red, 4)]);
    place(&mut game);

    resolve_may(&mut game, true, None);
    assert_eq!(zone(&game, ogre), Zone::Graveyard);
    assert_eq!(game.pending_triggers.len(), 1, "\"when you do\" triggered, checked as it was made");
    assert!(game.delayed_triggers.is_empty(), "and never waits");
    place_targeting(&mut game, ChoiceOption::Player(1));
    resolve_all(&mut game, &test_dp());

    assert_eq!(life(&game, 1), 17);
    assert_eq!(zone(&game, manticore), Zone::Battlefield, "\"another\": the Manticore is no candidate");
    let lines = reflexive_lines(&game);
    assert_eq!(lines.len(), 1, "its creation is logged: {lines:?}");
    assert!(lines[0].contains("reflexive"));
}

/// Heart-Piercer Manticore's ruling #3, on its fixture, since the card is
/// unregistered and the ledger holds no ruling to link: "When it enters the
/// battlefield, its triggered ability goes on the stack without a target. While
/// that ability is resolving, you may sacrifice a creature. If you do, a second
/// ability triggers and you pick a target that will be dealt damage. This is
/// different from other abilities that say "If you do . . ." in that players
/// may cast spells and activate abilities before a creature is sacrificed and
/// then again after the creature is sacrificed but before damage is dealt."
#[test]
fn the_manticores_first_trigger_has_no_target_and_its_second_waits_for_responses() {
    let mut game = setup_two_player_game();
    let ogre = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    cast_and_resolve(&mut game, 0, spitefang_manticore(), &[(ManaType::Red, 4)]);
    place(&mut game);
    assert!(top_targets(&game).is_empty(), "the enters trigger goes on the stack without a target");

    resolve_may(&mut game, true, None);
    place_targeting(&mut game, ChoiceOption::Player(1));
    assert_eq!(zone(&game, ogre), Zone::Graveyard, "the creature is sacrificed first");
    assert_eq!(top_targets(&game), vec![vec![ResolvedTarget::Player(1)]], "the second ability has the target");
    assert_eq!(life(&game, 1), 20, "and the damage waits on the stack, where players may respond");
}

/// Heart-Piercer Manticore's ruling #4, on its fixture, since the card is
/// unregistered and the ledger holds no ruling to link: "Heart-Piercer
/// Manticore's damage-dealing ability triggers only when you sacrifice a
/// creature as a result of the instruction of its triggered ability. It won't
/// trigger if you sacrifice a creature for any other reason."
#[test]
fn only_the_triggers_own_sacrifice_makes_the_manticore_deal_damage() {
    let mut game = setup_two_player_game();
    let altar = put_on_battlefield(&mut game, bonegrinder_altar(), 0);
    let ogre = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    cast_and_resolve(&mut game, 0, spitefang_manticore(), &[(ManaType::Red, 4)]);
    place(&mut game);

    // In response, the altar's cost sacrifices the ogre: a sacrifice, but
    // not this ability's.
    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id: altar, count: 1 }, vec![ChoiceOption::Object(ogre)]);
    game.activate_ability(0, altar, 0, &dp).unwrap();
    game.resolve_top_of_stack(&dp).unwrap();
    assert_eq!(zone(&game, ogre), Zone::Graveyard);

    resolve_may(&mut game, false, None);
    assert!(game.pending_triggers.is_empty(), "nothing was sacrificed this way");
    assert_eq!(reflexive_lines(&game).len(), 1, "though the trigger was made, and checked");
}

/// Heart-Piercer Manticore's ruling #6, on its fixture, since the card is
/// unregistered and the ledger holds no ruling to link: "The sacrificed
/// creature's last known existence on the battlefield is checked to determine
/// its power."
#[test]
fn the_sacrificed_creatures_power_is_read_as_it_last_existed() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, glorious_anthem(), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    cast_and_resolve(&mut game, 0, spitefang_manticore(), &[(ManaType::Red, 4)]);
    place(&mut game);

    resolve_may(&mut game, true, None);
    assert_eq!(zone(&game, bear), Zone::Graveyard);
    place_targeting(&mut game, ChoiceOption::Player(1));
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 1), 17, "3: the anthem's +1/+1 as it last existed");
}

/// Heart-Piercer Manticore's ruling #10, on its fixture, since the card is
/// unregistered and the ledger holds no ruling to link: "You can't sacrifice
/// multiple creatures to deal damage multiple times."
#[test]
fn the_manticore_sacrifices_one_creature_and_triggers_once() {
    let mut game = setup_two_player_game();
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let ogre = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    cast_and_resolve(&mut game, 0, spitefang_manticore(), &[(ManaType::Red, 4)]);
    place(&mut game);

    resolve_may(&mut game, true, Some(ogre));
    assert_eq!((zone(&game, ogre), zone(&game, bear)), (Zone::Graveyard, Zone::Battlefield));
    assert_eq!(game.pending_triggers.len(), 1);
    place_targeting(&mut game, ChoiceOption::Player(1));
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 1), 17);
}

/// `codebase-state.md` item 229: "another creature" at resolution is other
/// than the Manticore, so the Manticore alone has nothing to sacrifice, and
/// nothing triggers.
#[test]
fn a_manticore_alone_has_nothing_to_sacrifice() {
    let mut game = setup_two_player_game();
    let manticore = cast_and_resolve(&mut game, 0, spitefang_manticore(), &[(ManaType::Red, 4)]);
    place(&mut game);

    resolve_may(&mut game, true, None);
    assert_eq!(zone(&game, manticore), Zone::Battlefield);
    assert!(game.pending_triggers.is_empty());
}

/// Cornered Crook, cast from hand from exactly {4}{R}: it enters, its
/// controller sacrifices an artifact, and it deals 3 to the target.
#[test]
fn cornered_crook_cast_from_hand_deals_3_once_an_artifact_is_sacrificed() {
    let mut game = setup_two_player_game();
    let trinket = put_on_battlefield(&mut game, card_of_type("Trinket", CardType::Artifact), 0);
    cast_and_resolve(&mut game, 0, cornered_crook(), &[(ManaType::Red, 5)]);
    place(&mut game);

    resolve_may(&mut game, true, None);
    assert_eq!(zone(&game, trinket), Zone::Graveyard);
    place_targeting(&mut game, ChoiceOption::Player(1));
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 1), 17);
}

// RULING: Cornered Crook #1 - "You don't choose a target for Cornered
//   Crook's ability at the time it triggers. Rather, a second "reflexive"
//   ability triggers when you sacrifice an artifact this way. You choose a
//   target for that ability as it goes on the stack. Each player may respond
//   to this triggered ability as normal."
#[test]
fn cornered_crooks_target_is_chosen_for_its_second_ability() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, card_of_type("Trinket", CardType::Artifact), 0);
    let crook = cast_and_resolve(&mut game, 0, cornered_crook(), &[(ManaType::Red, 5)]);
    place(&mut game);
    assert!(top_targets(&game).is_empty(), "no target as it triggers");

    resolve_may(&mut game, true, None);
    assert!(game.stack.is_empty(), "the second ability has triggered, and is not yet on the stack");
    place_targeting(&mut game, ChoiceOption::Player(1));
    assert_eq!(top_targets(&game), vec![vec![ResolvedTarget::Player(1)]]);
    let top = *game.stack.last().unwrap();
    assert_eq!(game.stack_entries[&top].ability_identity.map(|i| i.source.id), Some(crook), "Cornered Crook's (CR 603.7e)");
}

/// The Crook is killed in response to its reflexive trigger, and still deals
/// its 3 damage, as it last existed (CR 113.7a, 608.2h): with the lifelink
/// Loxodon Warhammer gave it and the deathtouch Venomweave Charm did, though
/// neither reaches the card in the graveyard.
#[test]
fn cornered_crook_killed_in_response_deals_its_damage_as_it_last_existed() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, venomweave_charm(), 0);
    let hammer = put_on_battlefield(&mut game, loxodon_warhammer(), 0);
    let trinket = put_on_battlefield(&mut game, card_of_type("Trinket", CardType::Artifact), 0);
    let wall = put_on_battlefield(&mut game, wall_of_stone(), 1);
    let crook = cast_and_resolve(&mut game, 0, cornered_crook(), &[(ManaType::Red, 5)]);
    game.execute_action(GameAction::Attach { attachment: hammer, host: crook }, &test_ctx()).unwrap();
    place(&mut game);
    // The Warhammer is an artifact too, so the sacrifice is a choice.
    resolve_may(&mut game, true, Some(trinket));
    place_targeting(&mut game, ChoiceOption::Object(wall));

    destroy(&mut game, crook);
    assert_eq!(zone(&game, crook), Zone::Graveyard);
    resolve_all(&mut game, &test_dp());

    assert_eq!(zone(&game, wall), Zone::Graveyard, "deathtouch, as it last existed: 3 damage to a 0/8");
    assert_eq!(life(&game, 0), 23, "lifelink, as it last existed");
}

/// Item 225 on the registered card: Circle of Protection: Red chooses the
/// Crook while its reflexive trigger waits, the Crook is killed, and the
/// damage it deals as it last existed is prevented.
#[test]
fn a_circle_that_chose_cornered_crook_prevents_its_damage_after_it_dies() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, card_of_type("Trinket", CardType::Artifact), 0);
    let crook = cast_and_resolve(&mut game, 0, cornered_crook(), &[(ManaType::Red, 5)]);
    let circle = put_on_battlefield(&mut game, circle_of_protection_red(), 1);
    place(&mut game);
    resolve_may(&mut game, true, None);
    place_targeting(&mut game, ChoiceOption::Player(1));

    let ctx = ResolutionContext {
        source: circle,
        ability_source: game.object_ref(circle),
        controller: 1,
        targets: ChosenTargets::NONE,
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    let dp = ChoosingSource::new(now(crook));
    game.resolve_effect(&circle_of_protection_red().abilities[0].effect, &ctx, &dp).unwrap();
    destroy(&mut game, crook);
    resolve_all(&mut game, &test_dp());

    assert_eq!(life(&game, 1), 20, "the Crook as it last existed is the source the Circle chose");
}

/// CR 603.12's other form, "when you don't": declined, it triggers once,
/// bound to no record; taken, it does not trigger.
#[test]
fn when_you_dont_triggers_only_when_the_action_was_not_taken() {
    let mut game = setup_two_player_game();
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let tithe = put_in_hand(&mut game, withheld_tithe(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);
    game.cast_spell(0, tithe, &ManaWindowStop::new(RecordingDecisionProvider::picking(0))).unwrap();
    resolve_may(&mut game, false, None);
    assert_eq!(game.pending_triggers.len(), 1, "not done: it triggers");
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), 18);

    let tithe = put_in_hand(&mut game, withheld_tithe(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);
    game.cast_spell(0, tithe, &ManaWindowStop::new(RecordingDecisionProvider::picking(0))).unwrap();
    resolve_may(&mut game, true, None);
    assert_eq!(zone(&game, bear), Zone::Graveyard);
    assert!(game.pending_triggers.is_empty(), "done: it does not");
}

/// CR 603.12a: a reflexive trigger whose event occurred twice during the
/// resolution triggers twice; one whose event is "one or more" triggers
/// once for both.
#[test]
fn a_reflexive_trigger_triggers_once_for_each_time_its_event_occurred() {
    let mut game = setup_two_player_game();
    mtgsim::test_support::fill_library(&mut game, 0, 5);
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    let hand = game.players[0].hand.len();
    cast_and_resolve(&mut game, 0, bloodtithe_rite(), &[(ManaType::Black, 1)]);

    assert_eq!(game.pending_triggers.len(), 3, "two for \"a creature\", one for \"one or more\"");
    resolve_all(&mut game, &RecordingDecisionProvider::picking(0));
    assert_eq!(life(&game, 0), 22);
    assert_eq!(game.players[0].hand.len(), hand + 1);
}
