//! Phase TR-5h — one history of what happened (`triggers-architecture.md`
//! §12, TR-5h).
//!
//! Every count a card reads is a sum over the history log's rows, and each
//! row carries its object as the event left it (CR 603.10). The fixtures are
//! invented and carry their own names (`engineering-practices.md` §3).

use mtgsim::engine::actions::GameAction;
use mtgsim::events::event::DamageTarget;
use mtgsim::objects::card_data::CardDataBuilder;
use mtgsim::oracle::characteristics::get_effective_power;
use mtgsim::state::history_log::RowFact;
use mtgsim::test_support::{put_on_battlefield, setup_two_player_game, static_ability, test_ctx, vanilla_creature};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{AmountExpr, Condition, Duration, Effect, EffectRecipient, ObjectFilter, PlayerSet, Primitive};
use mtgsim::types::history::{CountIs, HistoryCount, TurnFact};
use mtgsim::types::keywords::KeywordFlag;

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
            Condition::ThisTurn(HistoryCount { players: PlayerSet::You, fact: TurnFact::LifeGainEvents, is: CountIs::AtLeast(1) }),
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
    let power = row.and_then(|row| row.frame.as_ref()).and_then(|chars| chars.power);
    assert_eq!(power, Some(3), "the lifelinker as the event left it");
}
