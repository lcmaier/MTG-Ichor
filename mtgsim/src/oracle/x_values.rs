//! CR 107.3a and 601.2b: the values of X a caster may announce, and why
//! (`mana-architecture.md` §3.7).
//!
//! Asked at 601.2b, once the alternative and additional costs are announced.
//! The legal values run from the least to the most, and the most is read in
//! closed form, never searched: X adds only generic mana and no cost change
//! reads it, so CR 601.2f's total is the same pips beside a generic part
//! linear in X (`ManaComponent`), and the inventory says once how much
//! generic mana it can pay alongside those pips
//! ([`ManaSupply::max_generic_payable_alongside`]). Rules that cut a gap, a
//! loyalty ability's −X or a target with mana value X, arrive with their
//! surfaces and offer the set form, [`XOffer::Set`].

use std::num::NonZeroU64;

use crate::engine::cost_determination::TotalCostInX;
use crate::oracle::mana_supply::{ManaSupply, NonManaCosts};
use crate::state::game_state::GameState;
use crate::types::ids::PlayerId;
use crate::types::mana::ManaCost;

/// The values of X a caster may announce for a spell, and the facts the why
/// panel says they stop at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegalXValues {
    pub offered: XOffer,
    /// The card says "X can't be 0", which is why the least is 1.
    pub x_cant_be_zero: bool,
    /// Why a larger X is not offered; `None` when the cost being paid has no
    /// X, so X is 0 (CR 601.2b announces "a variable cost that will be
    /// paid", and 107.3b says the same of an effect's free cast).
    pub bound: Option<XBound>,
}

impl LegalXValues {
    /// Every X from 0 through `at_most`: what a one-X cost with no other
    /// generic mana offers when `at_most` generic mana is payable beside its
    /// pips. A provider asked outside a cast, a test's, is handed it.
    pub fn zero_through(at_most: u64) -> LegalXValues {
        LegalXValues {
            offered: XOffer::Interval { at_least: 0, at_most },
            x_cant_be_zero: false,
            bound: Some(XBound {
                pips: ManaCost::zero(),
                max_generic_payable: at_most,
                generic_besides_x: 0,
                x_symbol_count: 1,
            }),
        }
    }
}

/// What `ChooseXValue` offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XOffer {
    /// Every X from `at_least` through `at_most`.
    Interval { at_least: u64, at_most: u64 },
    /// These, ascending, when a rule cuts a gap: Tamiyo, Compleated Sage's −X
    /// over cards of mana value 0, 1, 4, 6 and 7 offers those.
    Set(Vec<u64>),
}

impl XOffer {
    /// Whether exactly one value is offered: then it is announced without
    /// asking (`backlog.md` §2.22's rule 1).
    pub fn has_one_value(&self) -> bool {
        match self {
            XOffer::Interval { at_least, at_most } => at_least == at_most,
            XOffer::Set(values) => values.len() == 1,
        }
    }

    /// The least value offered.
    pub fn at_least(&self) -> u64 {
        match self {
            XOffer::Interval { at_least, .. } => *at_least,
            XOffer::Set(values) => values.first().copied().unwrap_or(0),
        }
    }

    /// The most value offered.
    pub fn at_most(&self) -> u64 {
        match self {
            XOffer::Interval { at_most, .. } => *at_most,
            XOffer::Set(values) => values.last().copied().unwrap_or(0),
        }
    }
}

/// A larger X leaves the cost unpayable (CR 601.2h): beside `pips`, the
/// caster can pay at most `max_generic_payable` generic mana, and the
/// cost's generic part is `generic_besides_x` plus `x_symbol_count` for
/// each X.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XBound {
    /// The cost's symbols other than generic, the same at every X: not the
    /// whole cost. The generic maximum is only true beside them, since a
    /// mana that pays a pip pays no generic, so the why names them with it.
    pub pips: ManaCost,
    pub max_generic_payable: u64,
    pub generic_besides_x: i64,
    pub x_symbol_count: u64,
}

/// The values of X `caster` may announce for a spell whose total, with X a
/// variable, is `total`; `None` when no X can be announced, which reverses
/// the cast (CR 601.2e, 732.1).
pub fn legal_x_values(
    game: &GameState,
    caster: PlayerId,
    total: &TotalCostInX,
    x_cant_be_zero: bool,
) -> Option<LegalXValues> {
    let at_least = u64::from(x_cant_be_zero);
    let Some(x_symbol_count) = NonZeroU64::new(total.mana.x_symbol_count()) else {
        return (!x_cant_be_zero).then_some(LegalXValues {
            offered: XOffer::Interval { at_least: 0, at_most: 0 },
            x_cant_be_zero,
            bound: None,
        });
    };
    let non_mana = NonManaCosts { source: None, costs: &total.non_mana };
    let pips = total.mana.pips();
    let max_generic_payable =
        ManaSupply::read(game, caster).max_generic_payable_alongside(game, &pips, &non_mana)?;
    let at_most = total.mana.generic_left_for_x(max_generic_payable)? / x_symbol_count;
    if at_most < at_least {
        return None;
    }
    #[cfg(debug_assertions)]
    audit(game, caster, total, at_most);
    Some(LegalXValues {
        offered: XOffer::Interval { at_least, at_most },
        x_cant_be_zero,
        bound: Some(XBound {
            pips,
            max_generic_payable,
            generic_besides_x: total.mana.generic_besides_x(),
            x_symbol_count: x_symbol_count.get(),
        }),
    })
}

/// The closed form, asked again by the check it stands for: every X up to a
/// small bound and eight past it, or a large bound and one past it, each
/// total built by the cast's own arithmetic and asked of the gate's
/// predicate. A cost that a larger X makes cheaper, or a shape the closed
/// form misreads, fails here in a debug test or fuzz run. So does a board
/// past the choice search's cap (`mana_supply`'s `MAX_CHOICE_LEAVES`), where
/// the check leans yes at every X and the closed form only to its upper
/// bound: no fuzz board has come near it.
#[cfg(debug_assertions)]
fn audit(game: &GameState, caster: PlayerId, total: &TotalCostInX, at_most: u64) {
    let supply = std::cell::OnceCell::new();
    let non_mana = NonManaCosts { source: None, costs: &total.non_mana };
    let checked: Vec<u64> = if at_most < 64 { (0..=at_most + 8).collect() } else { vec![at_most, at_most + 1] };
    for x in checked {
        let payable = crate::oracle::mana_helpers::mana_payable(game, caster, &total.mana.at(x), &non_mana, &supply);
        let expected = x <= at_most;
        assert_eq!(payable, expected, "the closed form says X = {x} is {}payable, the check says otherwise", if expected { "" } else { "not " });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::basic_lands::{mountain, plains};
    use crate::engine::cost_determination::total_cost_in_x;
    use crate::cards::phase_cm_cards::{goblin_electromancer, thalia_guardian_of_thraben, trinisphere};
    use crate::cards::phase_re9_cards::doubling_cube;
    use crate::objects::card_data::{CardData, CardDataBuilder};
    use crate::test_support::{put_in_hand, put_on_battlefield, setup_two_player_game};
    use crate::types::card_types::CardType;
    use crate::types::ids::ObjectId;
    use crate::types::costs::{AdditionalCost, AlternativeCost, Cost};
    use crate::types::mana::{ManaSymbol, ManaType};
    use std::sync::Arc;

    const X: ManaSymbol = ManaSymbol::X;
    const R: ManaSymbol = ManaSymbol::Colored(ManaType::Red);
    const W: ManaSymbol = ManaSymbol::Colored(ManaType::White);

    /// A sorcery costing `symbols`, registered nowhere.
    fn sorcery(symbols: &[ManaSymbol]) -> CardDataBuilder {
        CardDataBuilder::new("Ember Lesson").card_type(CardType::Sorcery).mana_cost(ManaCost::from_symbols(symbols.to_vec()))
    }

    /// Player 0 with `mountains` Mountains, and `card` in hand.
    fn board(mountains: usize, card: Arc<CardData>) -> (GameState, ObjectId) {
        let mut game = setup_two_player_game();
        for _ in 0..mountains {
            put_on_battlefield(&mut game, mountain(), 0);
        }
        let spell = put_in_hand(&mut game, card, 0);
        (game, spell)
    }

    fn values(game: &GameState, spell: ObjectId) -> Option<LegalXValues> {
        values_with(game, spell, None, &[])
    }

    fn values_with(
        game: &GameState,
        spell: ObjectId,
        alt: Option<&AlternativeCost>,
        additional: &[&AdditionalCost],
    ) -> Option<LegalXValues> {
        let card = &game.objects[&spell].card_data;
        let total = total_cost_in_x(game, spell, card.mana_cost.as_ref().unwrap(), alt, additional);
        legal_x_values(game, 0, &total, card.x_cant_be_zero)
    }

    fn offered(game: &GameState, spell: ObjectId) -> XOffer {
        values(game, spell).unwrap().offered
    }

    /// Five Mountains pay `{X}{R}` up to X = 4, and the why's numbers are the
    /// closed form's: four generic beside `{R}`, one a X.
    #[test]
    fn the_most_is_what_the_board_pays_beside_the_pips() {
        let (game, spell) = board(5, sorcery(&[X, R]).build());
        assert_eq!(
            values(&game, spell),
            Some(LegalXValues {
                offered: XOffer::Interval { at_least: 0, at_most: 4 },
                x_cant_be_zero: false,
                bound: Some(XBound {
                    pips: ManaCost::from_symbols(vec![R]),
                    max_generic_payable: 4,
                    generic_besides_x: 0,
                    x_symbol_count: 1,
                }),
            })
        );
    }

    /// Two X symbols each pay the announced X (CR 107.3a).
    #[test]
    fn each_x_symbol_pays_x() {
        let (game, spell) = board(5, sorcery(&[X, X, R]).build());
        assert_eq!(offered(&game, spell), XOffer::Interval { at_least: 0, at_most: 2 });
    }

    /// A `{1}` reduction absorbs one X (§3.7): Goblin Electromancer makes
    /// X = 4 cost `{3}{R}`, four Mountains.
    #[test]
    fn a_reduction_absorbs_part_of_x() {
        let (mut game, spell) = board(4, sorcery(&[X, R]).build());
        put_on_battlefield(&mut game, goblin_electromancer(), 0);
        assert_eq!(offered(&game, spell), XOffer::Interval { at_least: 0, at_most: 4 });
    }

    /// Thalia's `{1}` more takes one X back.
    #[test]
    fn an_increase_takes_part_of_x() {
        let (mut game, spell) = board(4, sorcery(&[X, R]).build());
        put_on_battlefield(&mut game, thalia_guardian_of_thraben(), 1);
        assert_eq!(offered(&game, spell), XOffer::Interval { at_least: 0, at_most: 2 });
    }

    /// Trinisphere raises `{X}{R}` to three mana below X = 2, so three
    /// Mountains pay X from 0 to 2, and two pay none.
    #[test]
    fn a_floor_on_the_total_bounds_x_from_below_too() {
        let (mut game, spell) = board(3, sorcery(&[X, R]).build());
        put_on_battlefield(&mut game, trinisphere(), 1);
        assert_eq!(offered(&game, spell), XOffer::Interval { at_least: 0, at_most: 2 });
        let (mut game, spell) = board(2, sorcery(&[X, R]).build());
        put_on_battlefield(&mut game, trinisphere(), 1);
        assert_eq!(values(&game, spell), None);
    }

    /// Doubling Cube under a Plains and six Mountains: the seven mana less
    /// the Cube's `{3}` leave `{W}{R}{R}{R}`, doubled to eight, so `{X}{W}`
    /// is paid up to X = 7, one more than the lands alone.
    #[test]
    fn a_pool_multiplier_raises_the_most() {
        let (mut game, spell) = board(6, sorcery(&[X, W]).build());
        put_on_battlefield(&mut game, plains(), 0);
        assert_eq!(offered(&game, spell), XOffer::Interval { at_least: 0, at_most: 6 });
        put_on_battlefield(&mut game, doubling_cube(), 0);
        assert_eq!(offered(&game, spell), XOffer::Interval { at_least: 0, at_most: 7 });
    }

    /// "X can't be 0" sets the least, and a board that pays only `{R}` pays
    /// no legal X.
    #[test]
    fn x_cant_be_zero_sets_the_least() {
        let (game, spell) = board(1, sorcery(&[X, R]).x_cant_be_zero().build());
        assert_eq!(values(&game, spell), None);
        let (game, spell) = board(2, sorcery(&[X, R]).x_cant_be_zero().build());
        let legal = values(&game, spell).unwrap();
        assert_eq!((legal.offered, legal.x_cant_be_zero), (XOffer::Interval { at_least: 1, at_most: 1 }, true));
    }

    /// CR 601.2b announces X only for "a variable cost that will be paid",
    /// and 107.3b says the same of a free cast: an alternative cost with no
    /// X leaves 0 the only legal value, and none under "X can't be 0".
    // COVERS-PARTIAL: ATOM-107.3b-001
    #[test]
    fn an_alternative_cost_with_no_x_holds_x_at_zero() {
        let free = AlternativeCost::Custom("Free".to_string(), Vec::new());
        let (game, spell) = board(5, sorcery(&[X, R]).build());
        assert_eq!(
            values_with(&game, spell, Some(&free), &[]),
            Some(LegalXValues { offered: XOffer::Interval { at_least: 0, at_most: 0 }, x_cant_be_zero: false, bound: None })
        );
        let (game, spell) = board(5, sorcery(&[X, R]).x_cant_be_zero().build());
        assert_eq!(values_with(&game, spell, Some(&free), &[]), None);
    }

    /// An alternative cost's own X is bounded by its own total: miracle's
    /// `{X}{R}` on a `{X}{R}{R}` card, three Mountains.
    #[test]
    fn an_alternative_costs_x_is_bounded_by_that_cost() {
        let miracle = AlternativeCost::Custom("Miracle".to_string(), vec![Cost::Mana(ManaCost::from_symbols(vec![X, R]))]);
        let (game, spell) = board(3, sorcery(&[X, R, R]).build());
        assert_eq!(offered(&game, spell), XOffer::Interval { at_least: 0, at_most: 1 });
        assert_eq!(values_with(&game, spell, Some(&miracle), &[]).unwrap().offered, XOffer::Interval { at_least: 0, at_most: 2 });
    }

    /// A kicker announced before X takes its share first, and a kicker the
    /// board cannot pay beside the least X leaves no legal X.
    #[test]
    fn an_announced_kicker_narrows_the_most() {
        let kicker = |generic| AdditionalCost::Kicker(vec![Cost::Mana(ManaCost::build(&[], generic))]);
        let (game, spell) = board(5, sorcery(&[X, R]).build());
        assert_eq!(values_with(&game, spell, None, &[&kicker(2)]).unwrap().offered, XOffer::Interval { at_least: 0, at_most: 2 });
        assert_eq!(values_with(&game, spell, None, &[&kicker(5)]), None);
    }

    #[test]
    fn the_offer_says_whether_it_has_one_value_and_where_it_stops() {
        assert!(XOffer::Interval { at_least: 2, at_most: 2 }.has_one_value());
        assert!(!XOffer::Interval { at_least: 0, at_most: 2 }.has_one_value());
        assert!(XOffer::Set(vec![4]).has_one_value());
        let tamiyo = XOffer::Set(vec![0, 1, 4, 6, 7]);
        assert!(!tamiyo.has_one_value());
        assert_eq!((tamiyo.at_least(), tamiyo.at_most()), (0, 7));
    }
}
