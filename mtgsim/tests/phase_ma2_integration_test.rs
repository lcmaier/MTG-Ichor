//! MA-2, the legal values of X (`mana-architecture.md` §6.1), and the random
//! agent's payment through a pool multiplier it rides on (`codebase-state.md`
//! item 214): what MA-1's check offers, the agent now pays.

use mtgsim::cards::basic_lands::{mountain, plains};
use mtgsim::cards::phase_lf_cards::humility;
use mtgsim::cards::phase_re9_cards::doubling_cube;
use mtgsim::cards::phase_rf_cards::darksteel_colossus;
use mtgsim::oracle::mana_helpers::can_cast;
use mtgsim::state::game_state::{GameState, Phase, PhaseType};
use mtgsim::test_support::{put_in_hand, put_on_battlefield, setup_two_player_game};
use mtgsim::types::zones::Zone;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;

/// Player 0's precombat main phase, where a sorcery-speed spell can be cast.
fn main_phase() -> GameState {
    let mut game = setup_two_player_game();
    game.set_turn_position(Phase::new(PhaseType::Precombat));
    game.active_player = 0;
    game
}

/// Whether the random agent, from `seed`, completes the cast of `card`.
fn the_agent_casts(game: &GameState, card: mtgsim::types::ids::ObjectId, seed: u64) -> bool {
    let mut game = game.clone();
    let cast = game.cast_spell(0, card, &ManaWindowStop::new(RandomDecisionProvider::seeded(seed)));
    cast.is_ok() && game.objects[&card].zone == Zone::Stack
}

/// Item 214's board: Humility's `{2}{W}{W}` from one Plains, four Mountains
/// and Doubling Cube, where the Cube's doubling is the only second `{W}`. The
/// agent taps everything else first, keeps the white out of the Cube's
/// `{3}`, and doubles `{W}{R}` into the cost. Before, it tapped the Plains,
/// found no source of a second `{W}`, and declined, so the cast was reversed.
#[test]
fn the_agent_pays_humility_through_doubling_cube() {
    let mut game = main_phase();
    put_on_battlefield(&mut game, plains(), 0);
    for _ in 0..4 {
        put_on_battlefield(&mut game, mountain(), 0);
    }
    put_on_battlefield(&mut game, doubling_cube(), 0);
    let card = put_in_hand(&mut game, humility(), 0);
    assert!(can_cast(&game, 0, card).is_ok(), "the check counts the doubling");
    for seed in 0..32 {
        assert!(the_agent_casts(&game, card, seed), "seed {seed}");
    }
}

/// Nine Plains and the Cube against Darksteel Colossus's `{11}`: the Cube is
/// activated once nothing else is offered, so it doubles the six white its
/// `{3}` leaves. Before, the agent counted it among the sources any of which
/// would do, and from some seeds tapped it with three white in the pool.
#[test]
fn the_agent_activates_the_cube_last() {
    let mut game = main_phase();
    for _ in 0..9 {
        put_on_battlefield(&mut game, plains(), 0);
    }
    put_on_battlefield(&mut game, doubling_cube(), 0);
    let card = put_in_hand(&mut game, darksteel_colossus(), 0);
    assert!(can_cast(&game, 0, card).is_ok());
    for seed in 0..32 {
        assert!(the_agent_casts(&game, card, seed), "seed {seed}");
    }
}
