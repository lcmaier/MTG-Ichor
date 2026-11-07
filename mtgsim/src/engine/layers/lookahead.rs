//! CR 614.12 / 614.17d — one object computed as it *would exist* on the
//! battlefield, as a read-side overlay on the layer walk.
//!
//! The walk reads concrete state in two places: the `PermanentState` it
//! seeds from (controller, CR 302.6's clock, counters) and the registry slice
//! it applies per layer. Both go through an accessor on the pass's `Board`
//! (`entity` and `rows_in_layer` in `board.rs`), and for the entering object
//! the accessor answers from this struct — the entity the performer *would*
//! build and the rows `register_static_effects` *would* write. For every
//! other member it answers from the real board, and the entering object's
//! own rows apply to it alone.
//!
//! That one-object asymmetry is the whole design. `replacement-architecture.md`
//! §5d has the rest: CR 614.12's three clauses against the reads they perturb,
//! why this is not a `GameState` clone, why the entering object is invisible
//! to a count over the battlefield, and why the frame may live on the stack.

use std::sync::Arc;

use crate::engine::layers::board::compute_board;
use crate::engine::layers::types::{ContinuousEffect, EffectOrigin, EffectiveCharacteristics, Timestamp};
use crate::objects::card_data::AbilityType;
use crate::state::battlefield::PermanentState;
use crate::state::continuous_effects::RegistryScopeSummary;
use crate::state::game_state::GameState;
use crate::state::layer_memo::{GatedInputs, StoredRunFrame};
use crate::types::effects::Duration;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::EnterMods;

/// The look-ahead for one entering object — see the module docs.
///
/// Built once per frame request from the proposal being decided, and dropped
/// with it. Nothing here is written back anywhere.
pub struct Lookahead {
    pub object: ObjectId,
    /// The entity `perform_entry` would build, from the one
    /// constructor both call (`PermanentState::entering`), out of the pending
    /// `EnterMods` — CR 614.12 clause (1).
    pub(super) entity: PermanentState,
    /// The CR 613.7d timestamp the object *would* receive on entering the
    /// battlefield — `next_timestamp` read without advancing it.
    ///
    /// Carried here rather than read off the object: the object still sits in
    /// its source zone while its entry is decided, so its real timestamp is
    /// that zone's and is *older* than every permanent on the board. What
    /// CR 614.12 asks is what it would be once it has entered, and that is
    /// newer than all of them.
    pub(super) entity_timestamp: Timestamp,
    /// CR 614.12 clause (2): the registry rows its own static abilities would
    /// generate, exactly as `register_static_effects` would write them.
    pub(super) rows: Vec<ContinuousEffect>,
    /// `rows` summarized as the registry summarizes its own, so the walk's
    /// fast-path gates stay exact for rows the registry does not hold.
    pub(super) summary: RegistryScopeSummary,
}

impl Lookahead {
    /// `object` as it would exist on the battlefield under `controller`, having
    /// entered with `mods`.
    pub fn new(game: &GameState, object: ObjectId, controller: PlayerId, mods: &EnterMods) -> Self {
        // Timestamps as the move and `perform_entry` would allocate
        // them — the entity's, then one per counter row in `mods` order — read
        // off `next_timestamp` without advancing it. Only the order matters:
        // later than every registered row, which is where CR 613.7a puts an
        // object's own static-ability effects and CR 613.7c its counters.
        let entity_timestamp = game.next_timestamp;
        let entity = PermanentState::entering(
            game, object, controller, mods, entity_timestamp, entity_timestamp + 1, game.turn_number,
        );

        let rows = would_be_rows(game, object, controller, entity_timestamp, mods);
        let summary = RegistryScopeSummary::of(&rows);

        Lookahead { object, entity, entity_timestamp, rows, summary }
    }
}

/// The rows `register_static_effects` would write for `object` entering under
/// `controller` — the same lowering, with no side effects and no assertions.
///
/// The abilities it would arrive with, as the real registration reads them:
/// the printed ones, or a copy's when `mods` carries one (CR 707.5). Whether
/// the object still *has* each ability on the battlefield is CR 604.2's
/// question, and `static_ability_still_exists` re-asks it at every layer
/// against this object's own frame — which is how Humility or Blood Moon strip
/// an entering permanent's static ability before it can apply to itself.
///
/// The lowering is registration's own, loud arms included. A card that
/// lowers to nothing contributes nothing here, which is also what it will
/// contribute once it has entered. Each row carries the id that
/// registration will give it, read off the registry as the timestamp is read
/// off its counter, because a grant mints its instance's id from its row.
fn would_be_rows(
    game: &GameState,
    object: ObjectId,
    controller: PlayerId,
    timestamp: Timestamp,
    mods: &EnterMods,
) -> Vec<ContinuousEffect> {
    let Some(obj) = game.objects.get(&object) else {
        return Vec::new();
    };
    // PRE-LAYER ZONE: what it enters with, printed or a copy's, before it is a
    // permanent (CR 614.12).
    let (abilities, name) = match &mods.copy {
        Some(copy) => (&copy.values.abilities, copy.values.name.as_str()),
        None => (&obj.card_data.abilities, obj.card_data.name.as_str()),
    };
    let mut rows = Vec::new();

    for ability in abilities.iter() {
        if ability.ability_type != AbilityType::Static {
            continue;
        }
        // CR 604.3a(3) — a CDA is never a registry row (`layers::cda`).
        if ability.is_characteristic_defining {
            continue;
        }
        // `Effect::Replacement`, `Effect::Restriction` and
        // `Effect::CostModification` lower to no atoms:
        // they are discovered off the effective ability list, not registered.
        for (clause, affected, lowered) in GameState::static_ability_rows(ability, name) {
            for (layer, modification) in lowered {
                rows.push(ContinuousEffect {
                    id: game.continuous_effects.next_id_after(rows.len()),
                    source: object,
                    origin: EffectOrigin::StaticAbility { ability: ability.id, clause },
                    layer,
                    duration: Duration::WhileSourceOnBattlefield,
                    controller,
                    created_on_turn: game.turn_number,
                    timestamp,
                    affected_objects: affected.clone(),
                    modification,
                });
            }
        }
    }

    rows
}

/// CR 614.12 / 614.17d — `id`'s characteristics as it *would exist* on the
/// battlefield under `controller`, with `pending` already applied.
///
/// The read-side counterpart of `compute_characteristics`: one pass over the
/// working set plus `id`, with the [`Lookahead`] threaded through the board,
/// so that the reads CR 614.12 names answer for the would-be permanent and
/// every read about any other object answers off the real board. Counted as
/// a layer walk, like every other top-level frame; fills no memo.
///
/// `id` may be anywhere: in the battlefield zone with no entity yet (the
/// replacement pipeline's case), or still in a hand, graveyard or on the stack
/// (a CR 614.17d "can't enter" asked at the zone change). The overlay makes it
/// a permanent for the pass's duration either way, and answers ahead of a real
/// entity if one exists, because the caller asked about the proposal.
pub fn compute_as_entering(
    game: &GameState,
    id: ObjectId,
    controller: PlayerId,
    pending: &EnterMods,
) -> Option<EffectiveCharacteristics> {
    walk_as_entering(game, id, controller, pending).map(|(frame, _)| frame)
}

/// [`compute_as_entering`]'s walk, with what it read of the gated inputs.
fn walk_as_entering(
    game: &GameState,
    id: ObjectId,
    controller: PlayerId,
    pending: &EnterMods,
) -> Option<(EffectiveCharacteristics, GatedInputs)> {
    game.diagnostics.record_layer_walk();
    let frames_before = game.diagnostics.layer_frames();
    let lookahead = Lookahead::new(game, id, controller, pending);
    let board = compute_board(game, Some(&lookahead));
    let read = board.gated_reads();
    let frame = board.take(id)?;
    game.trace(|| {
        crate::engine::trace_records::layer_walk(
            game,
            id,
            crate::engine::trace_records::WalkKind::Entering,
            frames_before,
            &frame,
        )
    });
    Some((frame, read))
}

/// [`compute_as_entering`] for the pipeline, walked once for a run of equal
/// tokens: the frame CR 614.12 asks of one token is the frame of every token
/// of its run decided at this epoch, under the same controller and `EnterMods`.
///
/// Nothing the walk reads tells two of them apart. A run is one definition of
/// one creation (`tokens_being_created`), so one printed text and one owner;
/// nothing on the board names an id just minted, and no entity exists for a
/// count or a filter to find (`Board::seed`). One thing it writes does: an
/// ability derived on the object, a planeswalker's loyalty or a basic land
/// type's mana, so a frame holding one stays that token's own, at a walk each.
///
/// Stored with what the walk read of the gated inputs, as the memo stores a
/// frame, so a write to one moves the epoch and ends the sharing.
pub(crate) fn shared_entering_frame(
    game: &GameState,
    id: ObjectId,
    controller: PlayerId,
    pending: &EnterMods,
) -> Option<Arc<EffectiveCharacteristics>> {
    let Some(run) = game.tokens_being_created.get(&id) else {
        return compute_as_entering(game, id, controller, pending).map(Arc::new);
    };
    let epoch = game.layer_epoch();
    if let Some(frame) = game.layer_memo.run_frame(epoch, run, controller, pending) {
        if audits_a_served_frame(game) {
            let walked = game.unobserved(|| compute_as_entering(game, id, controller, pending));
            assert!(
                walked.as_ref() == Some(&*frame),
                "{id:?} was served its run's CR 614.12 frame {frame:?}, and its own walk found {walked:?}: \
                 something the walk reads tells two equal tokens apart",
            );
        }
        return Some(frame);
    }
    let (frame, read) = walk_as_entering(game, id, controller, pending)?;
    let frame = Arc::new(frame);
    if !frame.abilities.iter().any(|ability| ability.id.is_derived_on_an_object()) {
        game.layer_memo.insert_gated_reads(epoch, read);
        game.layer_memo.insert_run_frame(epoch, StoredRunFrame {
            run: Arc::clone(run),
            controller,
            mods: pending.clone(),
            frame: Arc::clone(&frame),
        });
    }
    Some(frame)
}

/// Whether a served frame is walked again: under `--audit`, and in a debug
/// build while the layer memo's audit is not paused, as a memo hit is.
fn audits_a_served_frame(game: &GameState) -> bool {
    #[cfg(debug_assertions)]
    if game.layer_memo.audited() {
        return true;
    }
    game.audit_on()
}

#[cfg(test)]
mod tests {
    use crate::test_support::place_on_battlefield;
    use std::sync::Arc;

    use super::{compute_as_entering, shared_entering_frame, Lookahead, StoredRunFrame};
    use crate::cards::printings::grizzly_bears;
    use crate::engine::layers::copiable_values;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::{CastFacts, CostChoices};
    use crate::state::game_state::{GameState, ResolvingObject};
    use crate::test_support::{put_in_hand, put_on_battlefield, setup_two_player_game};
    use crate::types::card_types::{CreatureType, Subtype};
    use crate::types::costs::AdditionalCost;
    use crate::types::effects::{CharacteristicEdit, CounterType, TokenDef, TypeChange};
    use crate::types::ids::ObjectId;
    use crate::types::mana::ManaSpent;
    use crate::types::replacement::{CopyAdditions, EnterMods, EntryCopy, EntryCounters, TapStatus};
    use crate::types::zones::Zone;

    /// A token being created, as `create_tokens` leaves it: in the zone with no
    /// entity, its creation naming it. Each call makes one more of `run`.
    fn token_of_run(game: &mut GameState, run: &Arc<TokenDef>) -> ObjectId {
        let mut token = GameObject::new(run.card_data(), 0, Zone::Battlefield);
        token.is_token = true;
        let id = game.add_object(token);
        game.tokens_being_created.insert(id, Arc::clone(run));
        id
    }

    fn bears_run() -> Arc<TokenDef> {
        Arc::new(TokenDef::creature(2, 2, &[], &[CreatureType::Bear]))
    }

    /// The share's audit is the walk it saved: a frame no walk would find is a
    /// panic, whatever the frame's reader would have made of it.
    #[test]
    #[should_panic(expected = "something the walk reads tells two equal tokens apart")]
    fn a_shared_frame_the_walk_disagrees_with_panics() {
        let mut game = setup_two_player_game();
        game.enable_audit();
        let run = bears_run();
        let first = token_of_run(&mut game, &run);
        let mut lie = compute_as_entering(&game, first, 0, &EnterMods::NONE).unwrap();
        lie.power = Some(99);
        game.layer_memo.insert_run_frame(game.layer_epoch(), StoredRunFrame {
            run: Arc::clone(&run),
            controller: 0,
            mods: EnterMods::NONE,
            frame: Arc::new(lie),
        });

        shared_entering_frame(&game, first, 0, &EnterMods::NONE);
    }

    /// A walk that read a life total is stored with that read, so the write
    /// moves the epoch, and the next token of the run is walked afresh: the
    /// memo's rule for a gated input, kept by the share too.
    #[test]
    fn a_gated_input_the_frame_read_ends_the_sharing_when_it_is_written() {
        use crate::cards::authoring::{as_long_as, creatures_you_control, each, gets, static_ability, you_have};
        use crate::engine::actions::GameAction;
        use crate::objects::card_data::CardDataBuilder;
        use crate::test_support::test_ctx;
        use crate::types::card_types::CardType;
        use crate::types::effects::{AmountExpr, PlayerFact};

        let mut game = setup_two_player_game();
        let while_hale = as_long_as(you_have(PlayerFact::LifeAtLeast(AmountExpr::Fixed(20))), gets(each(creatures_you_control()), 1, 1));
        let anthem = CardDataBuilder::new("Path of Bravery Fixture")
            .card_type(CardType::Enchantment)
            .ability(static_ability("As long as you have 20 or more life, creatures you control get +1/+1.", while_hale))
            .build();
        put_on_battlefield(&mut game, anthem, 0);
        let run = bears_run();
        let (first, second) = (token_of_run(&mut game, &run), token_of_run(&mut game, &run));

        let hale = shared_entering_frame(&game, first, 0, &EnterMods::NONE).unwrap();
        assert_eq!(hale.power, Some(3));

        game.execute_action(GameAction::LoseLife { player: 0, amount: 1, damage_source: None }, &test_ctx()).unwrap();

        let wounded = shared_entering_frame(&game, second, 0, &EnterMods::NONE).unwrap();
        assert_eq!(wounded.power, Some(2));
    }

    /// `replacement-architecture.md` §3.5's constructor, field by field: the permanent
    /// the frame predicts is the one the performer places, for every field
    /// the proposal or the resolving spell supplies, timestamps included.
    #[test]
    fn the_look_ahead_predicts_the_permanent_the_performer_places() {
        let mut game = setup_two_player_game();
        let donor = put_on_battlefield(&mut game, grizzly_bears(), 1);
        let card = put_in_hand(&mut game, grizzly_bears(), 0);
        let mutant = CharacteristicEdit::Types(TypeChange {
            add_subtypes: vec![Subtype::Creature(CreatureType::Mutant)],
            ..TypeChange::NONE
        });
        let mods = EnterMods {
            status: Some(TapStatus::Tapped),
            counters: vec![
                EntryCounters { counter: CounterType::PlusOnePlusOne, n: 2, by: None },
                EntryCounters { counter: CounterType::Charge, n: 1, by: Some(1) },
            ],
            copy: Some(EntryCopy {
                values: Arc::new(copiable_values(&game, donor).expect("the donor is on the battlefield")),
                added: CopyAdditions::default(),
            }),
            edits: Some(Arc::from(vec![mutant])),
            attached_to: None,
            attacking: None,
        };
        game.resolving = Some(ResolvingObject {
            id: card,
            default_controller: 0,
            cast: Some(CastFacts { by: 0, from: Zone::Hand, mana_spent: ManaSpent::NONE }),
            cost_choices: CostChoices { additional: vec![AdditionalCost::Bargain], ..CostChoices::NONE },
            x_value: Some(3),
            chosen_targets: Vec::new(),
            identity: None,
            trigger: None,
            began_at: game.events.next_seq(),
            #[cfg(debug_assertions)]
            binding_reads_noted: std::cell::Cell::default(),
        });

        let predicted = Lookahead::new(&game, card, 0, &mods).entity;
        game.move_object(card, Zone::Battlefield).expect("a hand card moves");
        let placed = place_on_battlefield(&mut game, card, 0, &mods).clone();

        assert_eq!(predicted.timestamp, placed.timestamp);
        assert!(placed.tapped && predicted.tapped);
        for kind in [CounterType::PlusOnePlusOne, CounterType::Charge] {
            let stack = |p: &crate::state::battlefield::PermanentState| p.counters().get(&kind).map(|s| (s.count, s.timestamp));
            assert_eq!(stack(&predicted), stack(&placed), "{kind:?} and its CR 613.7c timestamp");
        }
        let copy = |p: &crate::state::battlefield::PermanentState| p.entered_as.copy.clone().expect("entered as a copy");
        assert!(Arc::ptr_eq(&copy(&predicted), &copy(&placed)));
        assert_eq!(predicted.entered_as.edits, placed.entered_as.edits);
        assert!(placed.entered_as.edits.is_some());
        assert_eq!((predicted.cast, placed.cast.is_some()), (placed.cast, true));
        assert_eq!(predicted.cost_choices, placed.cost_choices);
        assert_eq!((predicted.x_value, placed.x_value), (Some(3), Some(3)));
        assert_eq!(predicted.controller, placed.controller);
        assert_eq!(predicted.entered_battlefield_turn, placed.entered_battlefield_turn);
    }
}
