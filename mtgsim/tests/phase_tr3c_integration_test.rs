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

use std::cell::RefCell;
use std::sync::Arc;

use mtgsim::cards::authoring::{dies, triggered_ability, whenever};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_rd_cards::circle_of_protection_red;
use mtgsim::cards::phase_tr3b_cards::flickerwisp;
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::event::DamageTarget;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::legality::damage_sources;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_hand, put_on_battlefield, setup_two_player_game, test_ctx, test_dp, RecordingDecisionProvider,
};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, Primitive, SelectionFilter, TargetCount};
use mtgsim::types::ids::{ObjectId, ObjectRef, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::triggers::TriggerSubject;
use mtgsim::types::zones::{DestructionSource, Zone, ZoneChangeCause};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::DecisionProvider;
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
