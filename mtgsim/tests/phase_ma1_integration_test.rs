//! MA-1, the exact mana check (`mana-architecture.md` §3): the boards on
//! which the greedy count hid a play a payment covers (`codebase-state.md`
//! item 162's under-offers), and two payments the window makes for what the
//! check now offers, so the offer agrees with the payment
//! (`cost-architecture.md` §3.6).
//!
//! Written against `can_cast(..).is_ok()` and `activatable_abilities`, which
//! read the same before MA-1's gate and after it, so each under-offer board
//! is shown failing on the tree before the fix.

use mtgsim::cards::artifacts::sol_ring;
use mtgsim::cards::basic_lands::plains;
use mtgsim::cards::dual_lands::{badlands, everywhere};
use mtgsim::cards::keyword_creatures::{knight_of_meadowgrain, thornweald_archer};
use mtgsim::cards::phase_cm_cards::krark_clan_ironworks;
use mtgsim::cards::phase_rc_cards::chainbreaker;
use mtgsim::cards::phase_re9_cards::{doubling_cube, mana_reflection};
use mtgsim::cards::phase_rf_cards::darksteel_colossus;
use mtgsim::cards::phase_tr1_cards::wild_growth;
use mtgsim::oracle::mana_helpers::{activatable_abilities, can_cast};
use mtgsim::state::game_state::{GameState, Phase, PhaseType};
use mtgsim::test_support::{put_in_hand, put_on_battlefield, setup_two_player_game};
use mtgsim::types::ids::PlayerId;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceOption};
use mtgsim::ui::decision::DecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

/// Player 0's precombat main phase, where a creature can be cast.
fn main_phase() -> GameState {
    let mut game = setup_two_player_game();
    game.set_turn_position(Phase::new(PhaseType::Precombat));
    game.active_player = 0;
    game
}

/// Sol Ring's one tap makes {C}{C}: beside a Plains it pays Chainbreaker's
/// {3}, which the greedy count read as two sources and so two mana.
#[test]
fn sol_ring_beside_a_plains_pays_chainbreakers_three() {
    let mut game = main_phase();
    let breaker = put_on_battlefield(&mut game, chainbreaker(), 0);
    put_on_battlefield(&mut game, sol_ring(), 0);
    put_on_battlefield(&mut game, plains(), 0);
    assert!(activatable_abilities(&game, 0).iter().any(|(source, _, _)| *source == breaker));
}

/// Mana Reflection doubles a tap: one Everywhere makes {W}{W}, which pays
/// Knight of Meadowgrain.
#[test]
fn mana_reflection_doubles_an_everywhere_into_two_white() {
    let mut game = main_phase();
    put_on_battlefield(&mut game, mana_reflection(), 0);
    put_on_battlefield(&mut game, everywhere(), 0);
    let knight = put_in_hand(&mut game, knight_of_meadowgrain(), 0);
    assert!(can_cast(&game, 0, knight).is_ok());
}

/// Wild Growth's {G} comes with its land's tap: a Badlands pays Thornweald
/// Archer's {1}{G} though it makes no green itself.
#[test]
fn wild_growths_green_pays_for_thornweald_archer() {
    let mut game = main_phase();
    let land = put_on_battlefield(&mut game, badlands(), 0);
    let aura = put_on_battlefield(&mut game, wild_growth(), 0);
    assert!(game.attach(aura, land));
    let archer = put_in_hand(&mut game, thornweald_archer(), 0);
    assert!(can_cast(&game, 0, archer).is_ok());
}

/// A seat that taps its first offered source until the cost is covered.
struct FirstSource;

impl DecisionProvider for FirstSource {
    fn pick_n(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        (0..bounds.0.max(1).min(options.len())).collect()
    }

    fn pick_number(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, min: u64, _: u64) -> u64 {
        min
    }

    fn allocate(
        &self,
        _: &GameState,
        _: PlayerId,
        _: &ChoiceContext,
        total: u64,
        _: &[ChoiceOption],
        mins: &[u64],
        maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        let mut split = mins.to_vec();
        let mut rest = total - mins.iter().sum::<u64>();
        for (i, amount) in split.iter_mut().enumerate() {
            let more = rest.min(maxs.map_or(u64::MAX, |maxs| maxs[i]) - *amount);
            *amount += more;
            rest -= more;
        }
        split
    }

    fn choose_ordering(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        (0..items.len()).collect()
    }
}

/// Krark-Clan Ironworks sacrificing itself pays Chainbreaker's {2}, and the
/// window makes that payment: the one artifact is a sacrifice with no choice.
#[test]
fn ironworks_alone_pays_for_chainbreaker() {
    let mut game = main_phase();
    put_on_battlefield(&mut game, krark_clan_ironworks(), 0);
    let card = put_in_hand(&mut game, chainbreaker(), 0);
    assert!(can_cast(&game, 0, card).is_ok());
    game.cast_spell(0, card, &ManaWindowStop::new(FirstSource)).unwrap();
    assert_eq!(game.objects[&card].zone, Zone::Stack);
}

/// Doubling Cube: nine Plains and the Cube pay Darksteel Colossus's {11},
/// six white doubled after its {3}, and the window offers the Cube once the
/// pool can pay for it.
#[test]
fn nine_plains_and_doubling_cube_cast_darksteel_colossus() {
    let mut game = main_phase();
    for _ in 0..9 {
        put_on_battlefield(&mut game, plains(), 0);
    }
    let colossus = put_in_hand(&mut game, darksteel_colossus(), 0);
    assert!(can_cast(&game, 0, colossus).is_err(), "nine mana is not eleven");
    put_on_battlefield(&mut game, doubling_cube(), 0);
    assert!(can_cast(&game, 0, colossus).is_ok());
    game.cast_spell(0, colossus, &ManaWindowStop::new(FirstSource)).unwrap();
    assert_eq!(game.objects[&colossus].zone, Zone::Stack);
    assert_eq!(game.players[0].mana_pool.total(), 1, "twelve white, eleven spent");
}
