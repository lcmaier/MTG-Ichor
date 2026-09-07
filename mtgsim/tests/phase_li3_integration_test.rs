//! Phase LI-3 — conditional statics (`layers-architecture.md` §13b).
//!
//! What changed is *what "the effect exists" means*. CR 604.2 already asked
//! whether the source still has the ability; "as long as [X]" is one more
//! clause in the same sentence, evaluated against the pass's live board at
//! the row's layer. So a conditional static registers ordinary rows, and
//! everything interesting happens in `board::static_ability_still_exists`.
//!
//! Four boards. Kird Ape with and without a Forest, which is the condition
//! alone; Kird Ape under Blood Moon, which is a condition reading a *lower
//! layer's* output and involves no dependency at all; the Flight Clause
//! against Humility, which is a conditional layer-6 grant over `Host` in
//! both timestamp orders; and the Simian Clause against Blood Moon, where
//! the condition is flipped by an effect in its own layer and CR 613.8's
//! dependency check is what gets the answer right.

use mtgsim::cards::{basic_lands, creatures, dual_lands, phase_ld_cards, phase_lf_cards, phase_li_cards};
use mtgsim::engine::actions::ZoneChangeCause;
use mtgsim::engine::layers::types::{EffectModification, Layer};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{
    get_effective_power, get_effective_subtypes, get_effective_toughness, has_keyword,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    card_of_type, put_on_battlefield, registered, setup_two_player_game, static_ability, test_ctx,
    vanilla_creature,
};
use mtgsim::types::card_types::{CardType, CreatureType, LandType, Subtype};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AmountExpr, Condition, Duration, Effect, EffectRecipient, PermanentFilter, Primitive, TypeChange,
};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::zones::Zone;
use std::sync::Arc;

fn pt(game: &GameState, id: ObjectId) -> (Option<i32>, Option<i32>) {
    (get_effective_power(game, id), get_effective_toughness(game, id))
}

fn is_ape(game: &GameState, id: ObjectId) -> bool {
    get_effective_subtypes(game, id).contains(&Subtype::Creature(CreatureType::Ape))
}

/// A `TypeChange` that adds subtypes and nothing else.
fn adds_subtypes(subtypes: &[Subtype]) -> TypeChange {
    TypeChange {
        add_types: Vec::new(),
        remove_types: Vec::new(),
        set_types: None,
        add_subtypes: subtypes.to_vec(),
        remove_subtypes: Vec::new(),
        set_subtypes: None,
        add_supertypes: Vec::new(),
        remove_supertypes: Vec::new(),
        set_supertypes: None,
    }
}

// ---------------------------------------------------------------------------
// Kird Ape — the condition alone.
//
// "This creature gets +1/+2 as long as you control a Forest." (Scryfall,
// 2026-09-06.) One layer-7c row, registered when the Ape enters and never
// touched again; what moves is the answer CR 604.2's existence check gives
// for it, once per layer per pass.
// ---------------------------------------------------------------------------

/// The bonus follows the board, with no zone change on the Ape and no
/// registry write at all. And "you control" is CR 109.5's — an opponent's
/// Forest is not yours.
#[test]
fn test_kird_ape_gets_its_bonus_only_while_you_control_a_forest() {
    let mut game = setup_two_player_game();
    let ape = put_on_battlefield(&mut game, phase_li_cards::kird_ape(), 0);
    assert_eq!(pt(&game, ape), (Some(1), Some(1)), "no Forest, no bonus");

    let theirs = put_on_battlefield(&mut game, basic_lands::forest(), 1);
    assert_eq!(pt(&game, ape), (Some(1), Some(1)), "their Forest is not yours (CR 109.5)");

    let mine = put_on_battlefield(&mut game, basic_lands::forest(), 0);
    assert_eq!(pt(&game, ape), (Some(2), Some(3)));

    // The row is still registered — one row, all along. Only its existence
    // moved, which is the whole of §13b decision 5.
    assert_eq!(
        game.continuous_effects.effects_in_layer(Layer::Layer7cModifyPT).len(),
        1,
        "one row throughout"
    );

    game.change_zone(mine, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx()).unwrap();
    assert_eq!(pt(&game, ape), (Some(1), Some(1)), "the Forest left; the bonus went with it");
    let _ = theirs;
}

/// A Taiga is a Forest until Blood Moon sets it to Mountain in layer 4, and
/// the Ape's condition is read at 7c against a board where that has already
/// happened. Two layers apart, so CR 613.8a(a) never applies: **no
/// dependency is involved**, and the pass runs no hypothetical on this board
/// at all.
#[test]
fn test_blood_moon_takes_kird_apes_forest_away_two_layers_earlier() {
    let mut game = setup_two_player_game();
    let ape = put_on_battlefield(&mut game, phase_li_cards::kird_ape(), 0);
    let taiga = put_on_battlefield(&mut game, dual_lands::taiga(), 0);
    assert!(
        get_effective_subtypes(&game, taiga).contains(&Subtype::Land(LandType::Forest)),
        "a Taiga is a nonbasic Mountain Forest"
    );
    assert_eq!(pt(&game, ape), (Some(2), Some(3)));

    put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 1);
    assert!(!get_effective_subtypes(&game, taiga).contains(&Subtype::Land(LandType::Forest)));

    let before = game.counters.dependency_checks();
    assert_eq!(pt(&game, ape), (Some(1), Some(1)), "no Forest at layer 4, no bonus at 7c");
    assert_eq!(
        game.counters.dependency_checks(),
        before,
        "the condition reads a lower layer's output; CR 613.8 is confined to one layer"
    );

    // A *basic* Forest is out of Blood Moon's reach ("Nonbasic lands are
    // Mountains"), so the same board with one gives the bonus back.
    put_on_battlefield(&mut game, basic_lands::forest(), 0);
    assert_eq!(pt(&game, ape), (Some(2), Some(3)));
}

// ---------------------------------------------------------------------------
// The Flight Clause — a conditional layer-6 grant over `Host`.
//
// Rune of Flight's third line, as a fixture (`phase_li_cards`): "As long as
// enchanted permanent is a creature, it has flying." The condition is
// `HostMatches`, read off `attached_to` the way `AffectedSet::Host` is, so
// reattaching the Aura moves both the condition and the grant.
// ---------------------------------------------------------------------------

#[test]
fn test_the_flight_clause_grants_only_while_its_host_is_a_creature() {
    let mut game = setup_two_player_game();
    let aura = put_on_battlefield(&mut game, phase_li_cards::flight_clause(), 0);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let rock = put_on_battlefield(&mut game, card_of_type("Plain Rock", CardType::Artifact), 0);

    assert!(!has_keyword(&game, bears, KeywordFlag::Flying), "unattached: the effect does not exist");

    assert!(game.attach(aura, rock));
    assert!(!has_keyword(&game, rock, KeywordFlag::Flying), "an artifact is not a creature");
    assert!(!has_keyword(&game, bears, KeywordFlag::Flying));

    assert!(game.attach(aura, bears));
    assert!(has_keyword(&game, bears, KeywordFlag::Flying));
    assert!(!has_keyword(&game, rock, KeywordFlag::Flying), "and the old host keeps nothing");
}

/// Humility ("All creatures lose all abilities and have base power and
/// toughness 1/1") strips at layer 6; the Flight Clause grants at layer 6.
/// Neither reads what the other writes — Humility reads types, the Clause
/// reads its host's types — so CR 613.8 finds no dependency and CR 613.7's
/// timestamps decide, in both directions.
#[test]
fn test_the_flight_clause_and_humility_in_both_orders() {
    // Humility first: it strips, and the later grant lands on top.
    let mut game = setup_two_player_game();
    let humility = put_on_battlefield(&mut game, phase_lf_cards::humility(), 1);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Vigilance]), 0);
    let aura = put_on_battlefield(&mut game, phase_li_cards::flight_clause(), 0);
    assert!(game.attach(aura, bears));

    assert!(has_keyword(&game, bears, KeywordFlag::Flying), "granted after Humility applied");
    assert!(!has_keyword(&game, bears, KeywordFlag::Vigilance), "and the printed keyword is gone");
    assert_eq!(pt(&game, bears), (Some(1), Some(1)), "Humility's 7b part still applies");
    let _ = humility;

    // The Aura first: the grant applies, and Humility takes it away again.
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let aura = put_on_battlefield(&mut game, phase_li_cards::flight_clause(), 0);
    assert!(game.attach(aura, bears));
    assert!(has_keyword(&game, bears, KeywordFlag::Flying));

    put_on_battlefield(&mut game, phase_lf_cards::humility(), 1);
    assert!(!has_keyword(&game, bears, KeywordFlag::Flying), "Humility applied later and cleared it");
    assert_eq!(pt(&game, bears), (Some(1), Some(1)));
}

// ---------------------------------------------------------------------------
// The Simian Clause + Blood Moon — a condition another effect in the *same*
// layer flips, which is CR 613.8a(b)'s existence clause on a conditional
// static.
//
// "As long as you control a Forest, each creature you control is an Ape in
// addition to its other types" is layer 4; so is Blood Moon. Applying Blood
// Moon turns the Taiga into a Mountain, the condition goes false, and the
// Clause's effect stops existing — so the Clause waits for Blood Moon
// whatever the timestamps say, and by then has nothing to say.
// ---------------------------------------------------------------------------

/// Both orders, and the two controls that say the dependency is what decided
/// it: without Blood Moon the Bears is an Ape, and with a basic Forest —
/// which Blood Moon cannot reach — it is an Ape again.
// COVERS-PARTIAL: ATOM-613.8a-002
#[test]
fn test_a_conditional_static_waits_for_what_can_falsify_its_condition() {
    // Control: the Clause on its own does what it says.
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, dual_lands::taiga(), 0);
    let clause = put_on_battlefield(&mut game, phase_li_cards::simian_clause(), 0);
    let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    assert!(is_ape(&game, bears), "a Taiga is a Forest, so the effect exists");
    let _ = clause;

    // The Clause first, Blood Moon second. Timestamp order alone would apply
    // the Clause while the Taiga was still a Forest.
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, dual_lands::taiga(), 0);
    put_on_battlefield(&mut game, phase_li_cards::simian_clause(), 0);
    let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let before = game.counters.dependency_checks();
    put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 1);
    assert!(
        !is_ape(&game, bears),
        "the Clause waits for Blood Moon (CR 613.8a(b)), and finds no Forest when its turn comes"
    );
    assert!(
        game.counters.dependency_checks() > before,
        "the pair reached the hypothetical, which is what `condition_reads` is for"
    );

    // Blood Moon first, the Clause second: timestamp order gives the same
    // answer here, and the CR's reason is still the dependency.
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, dual_lands::taiga(), 0);
    put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 1);
    put_on_battlefield(&mut game, phase_li_cards::simian_clause(), 0);
    let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    assert!(!is_ape(&game, bears));

    // A basic Forest is out of Blood Moon's reach, so applying it changes
    // nothing the Clause reads: no dependency, and the Clause applies.
    put_on_battlefield(&mut game, basic_lands::forest(), 0);
    assert!(is_ape(&game, bears));
}

// ---------------------------------------------------------------------------
// CR 613.6 — "if an effect starts to apply in one layer, it continues to
// apply to those objects in later layers". A condition is read once, when
// the effect starts; a later layer consults the locked set instead.
// ---------------------------------------------------------------------------

/// A white enchantment whose condition is "you control a white permanent" —
/// itself, until layer 5 takes its color away. The layer-4 half has already
/// started by then, so the layer-7b half applies to the set it locked even
/// though the condition is false when 7b arrives.
// COVERS-PARTIAL: ATOM-613.6-003
#[test]
fn test_a_conditional_effect_that_has_started_keeps_applying_in_later_layers() {
    /// "As long as you control a white permanent, each creature is an Ape
    /// and has base power and toughness 4/4" — one ability, two atoms, two
    /// layers, which is the shape CR 613.6 is written about. `white` decides
    /// whether the card is itself the white permanent its condition asks for.
    fn pale_rites(white: bool) -> Arc<CardData> {
        let each_creature =
            EffectRecipient::FilteredPermanents(PermanentFilter::ByType(CardType::Creature));
        let mut builder = CardDataBuilder::new("Pale Rites").card_type(CardType::Enchantment);
        if white {
            builder = builder.color(Color::White);
        }
        builder
            .ability(static_ability(Effect::Conditional(
                Condition::ControlPermanent(PermanentFilter::ByColor(Color::White)),
                Box::new(Effect::Sequence(vec![
                    Effect::Atom(
                        Primitive::ChangeType(
                            adds_subtypes(&[Subtype::Creature(CreatureType::Ape)]),
                            Duration::WhileSourceOnBattlefield,
                        ),
                        each_creature.clone(),
                    ),
                    Effect::Atom(
                        Primitive::SetPowerToughness(
                            AmountExpr::Fixed(4),
                            AmountExpr::Fixed(4),
                            Duration::WhileSourceOnBattlefield,
                        ),
                        each_creature,
                    ),
                ])),
            )))
            .build()
    }

    let mut game = setup_two_player_game();
    let rites = put_on_battlefield(&mut game, pale_rites(true), 0);
    let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    assert!(is_ape(&game, bears));
    assert_eq!(pt(&game, bears), (Some(4), Some(4)));

    // Layer 5 takes the only white permanent's color, so the condition is
    // false from layer 5 onwards — after layer 4 has already started.
    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(registered(
        rites,
        Layer::Layer5Color,
        timestamp,
        EffectModification::RemoveAllColors,
    ));
    assert!(
        is_ape(&game, bears),
        "layer 4 ran while the condition held, and CR 613.6 does not re-ask"
    );
    assert_eq!(
        pt(&game, bears),
        (Some(4), Some(4)),
        "so the 7b half applies to the set layer 4 locked, condition or no condition"
    );

    // The control: the same card, not white, so the condition is false at
    // layer 4 too — and then neither half applies. It is the condition that
    // gates both, and the lock that carries the second one.
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, pale_rites(false), 0);
    let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    assert!(!is_ape(&game, bears), "no white permanent, no effect");
    assert_eq!(pt(&game, bears), (Some(2), Some(2)));
}
