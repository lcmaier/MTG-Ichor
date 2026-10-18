//! CR 107.3a and 601.2b: the values of X a caster may announce, and why
//! (`mana-architecture.md` §3.7, §6.1).
//!
//! Asked at 601.2b, once the alternative and additional costs are announced.
//! The legal values run from the least to the most, and the most is read in
//! closed form, never searched: X adds only generic mana and no cost change
//! reads it, so CR 601.2f's total is the same pips beside a generic part
//! linear in X (`ManaComponent`), and the inventory says once how much
//! generic mana it can pay beside those pips
//! ([`ManaSupply::most_generic_beside`]). Rules that cut a gap, a loyalty
//! ability's −X or a target with mana value X, arrive with their surfaces and
//! offer the set form, [`XOffer::Values`].

use crate::engine::cost_determination::total_in_x;
use crate::oracle::mana_supply::{ManaSupply, NonManaCosts};
use crate::state::game_state::GameState;
use crate::types::costs::{AdditionalCost, AlternativeCost};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::ManaCost;

/// The values of X a caster may announce for a spell, and why they are
/// these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegalXValues {
    pub offered: XOffer,
    pub least: LeastX,
    pub most: MostX,
}

/// What `ChooseXValue` offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XOffer {
    /// Every X from `min` to `max`.
    Range { min: u64, max: u64 },
    /// These, ascending, when a rule cuts a gap: Tamiyo, Compleated Sage's −X
    /// over cards of mana value 0, 1, 4, 6 and 7 offers those.
    Values(Vec<u64>),
}

impl XOffer {
    /// The one value offered, when there is only one: then nothing is asked
    /// (`backlog.md` §2.22's rule 1).
    pub fn only(&self) -> Option<u64> {
        match self {
            XOffer::Range { min, max } => (min == max).then_some(*min),
            XOffer::Values(values) => match values.as_slice() {
                [only] => Some(*only),
                _ => None,
            },
        }
    }

    /// Whether `x` is offered.
    pub fn offers(&self, x: u64) -> bool {
        match self {
            XOffer::Range { min, max } => (*min..=*max).contains(&x),
            XOffer::Values(values) => values.contains(&x),
        }
    }
}

/// Why the least legal X is the least.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeastX {
    Zero,
    /// "X can't be 0."
    CannotBeZero,
    /// The cost being paid has no X, so X is 0: CR 601.2b announces "a
    /// variable cost that will be paid", and 107.3b says the same of an
    /// effect's free cast.
    NotInTheCostPaid,
}

/// Why the most legal X is the most.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MostX {
    /// A larger X leaves the cost unpayable (CR 601.2h). Each X adds `per_x`
    /// generic mana to a generic part of `generic_besides_x` at X = 0, and
    /// beside `beside`, the rest of the cost, the caster can pay at most
    /// `generic_budget` generic mana.
    Payable { generic_budget: u64, generic_besides_x: i64, per_x: u64, beside: ManaCost },
    /// X is held at the least: the cost being paid has no X.
    TheLeast,
}

/// Why no X can be announced, which reverses the cast (CR 601.2e, 732.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoLegalX {
    /// "X can't be 0", and the cost being paid has no X.
    CannotBeZero,
    /// Not even the least X leaves a payable total (CR 601.2h).
    Unpayable { least: u64 },
}

/// The engine's own words, for the error a reversed cast returns.
impl std::fmt::Display for NoLegalX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoLegalX::CannotBeZero => f.write_str("X can't be 0, and the cost being paid has no X"),
            NoLegalX::Unpayable { least } => write!(f, "Not enough mana for X = {least}"),
        }
    }
}

/// CR 601.2b's choices before X, which the values of X depend on.
#[derive(Debug, Clone, Copy)]
pub struct XAnnouncement<'a> {
    pub base_mana_cost: &'a ManaCost,
    pub chosen_alt_cost: Option<&'a AlternativeCost>,
    pub chosen_additional_costs: &'a [&'a AdditionalCost],
    /// The card's "X can't be 0", as printed.
    pub x_cannot_be_zero: bool,
}

/// The values of X `caster` may announce for `spell`, after `announced`.
pub fn legal_x_values(
    game: &GameState,
    caster: PlayerId,
    spell: ObjectId,
    announced: &XAnnouncement<'_>,
) -> Result<LegalXValues, NoLegalX> {
    let (mana, others) = total_in_x(
        game,
        spell,
        announced.base_mana_cost,
        announced.chosen_alt_cost,
        announced.chosen_additional_costs,
    );
    if mana.x_symbols() == 0 {
        if announced.x_cannot_be_zero {
            return Err(NoLegalX::CannotBeZero);
        }
        return Ok(LegalXValues {
            offered: XOffer::Range { min: 0, max: 0 },
            least: LeastX::NotInTheCostPaid,
            most: MostX::TheLeast,
        });
    }
    let (least, least_because) =
        if announced.x_cannot_be_zero { (1, LeastX::CannotBeZero) } else { (0, LeastX::Zero) };
    let non_mana = NonManaCosts { source: None, costs: &others };
    let beside = mana.pips();
    let generic_budget = ManaSupply::read(game, caster).most_generic_beside(game, &beside, &non_mana);
    let bounded = generic_budget.and_then(|budget| mana.most_x_within(budget).map(|most| (budget, most)));
    let Some((generic_budget, most)) = bounded.filter(|(_, most)| *most >= least) else {
        return Err(NoLegalX::Unpayable { least });
    };
    #[cfg(debug_assertions)]
    audit(game, caster, &mana, &others, most);
    Ok(LegalXValues {
        offered: XOffer::Range { min: least, max: most },
        least: least_because,
        most: MostX::Payable {
            generic_budget,
            generic_besides_x: mana.generic_besides_x(),
            per_x: mana.x_symbols(),
            beside,
        },
    })
}

/// The closed form, asked again by the check it stands for: every X up to a
/// small bound and eight past it, or a large bound and one past it, each
/// total built by the cast's own arithmetic and asked of the gate's
/// predicate. A cost that a larger X makes cheaper, or a shape the closed
/// form misreads, fails here in a debug test or fuzz run.
#[cfg(debug_assertions)]
fn audit(
    game: &GameState,
    caster: PlayerId,
    mana: &crate::engine::cost_determination::ManaComponent,
    others: &[crate::types::costs::Cost],
    most: u64,
) {
    let supply = std::cell::OnceCell::new();
    let non_mana = NonManaCosts { source: None, costs: others };
    let checked: Vec<u64> = if most < 64 { (0..=most + 8).collect() } else { vec![most, most + 1] };
    for x in checked {
        let payable = crate::oracle::mana_helpers::mana_payable(game, caster, &mana.at(x), &non_mana, &supply);
        assert_eq!(payable, x <= most, "the closed form says X = {x} is {}payable, the check says otherwise", if x <= most { "" } else { "not " });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::basic_lands::{mountain, plains};
    use crate::cards::phase_cm_cards::{goblin_electromancer, thalia_guardian_of_thraben, trinisphere};
    use crate::cards::phase_re9_cards::doubling_cube;
    use crate::objects::card_data::{CardData, CardDataBuilder};
    use crate::test_support::{put_in_hand, put_on_battlefield, setup_two_player_game};
    use crate::types::card_types::CardType;
    use crate::types::costs::Cost;
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

    fn values(game: &GameState, spell: ObjectId) -> Result<LegalXValues, NoLegalX> {
        values_with(game, spell, None, &[])
    }

    fn values_with(
        game: &GameState,
        spell: ObjectId,
        alt: Option<&AlternativeCost>,
        additional: &[&AdditionalCost],
    ) -> Result<LegalXValues, NoLegalX> {
        let card = &game.objects[&spell].card_data;
        let announced = XAnnouncement {
            base_mana_cost: card.mana_cost.as_ref().unwrap(),
            chosen_alt_cost: alt,
            chosen_additional_costs: additional,
            x_cannot_be_zero: card.x_cannot_be_zero,
        };
        legal_x_values(game, 0, spell, &announced)
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
            Ok(LegalXValues {
                offered: XOffer::Range { min: 0, max: 4 },
                least: LeastX::Zero,
                most: MostX::Payable {
                    generic_budget: 4,
                    generic_besides_x: 0,
                    per_x: 1,
                    beside: ManaCost::from_symbols(vec![R]),
                },
            })
        );
    }

    /// Two X symbols each pay the announced X (CR 107.3a).
    #[test]
    fn each_x_symbol_pays_x() {
        let (game, spell) = board(5, sorcery(&[X, X, R]).build());
        assert_eq!(offered(&game, spell), XOffer::Range { min: 0, max: 2 });
    }

    /// A `{1}` reduction absorbs one X (§3.7): Goblin Electromancer makes
    /// X = 4 cost `{3}{R}`, four Mountains.
    #[test]
    fn a_reduction_absorbs_part_of_x() {
        let (mut game, spell) = board(4, sorcery(&[X, R]).build());
        put_on_battlefield(&mut game, goblin_electromancer(), 0);
        assert_eq!(offered(&game, spell), XOffer::Range { min: 0, max: 4 });
    }

    /// Thalia's `{1}` more takes one X back.
    #[test]
    fn an_increase_takes_part_of_x() {
        let (mut game, spell) = board(4, sorcery(&[X, R]).build());
        put_on_battlefield(&mut game, thalia_guardian_of_thraben(), 1);
        assert_eq!(offered(&game, spell), XOffer::Range { min: 0, max: 2 });
    }

    /// Trinisphere raises `{X}{R}` to three mana below X = 2, so three
    /// Mountains pay X from 0 to 2, and two pay none.
    #[test]
    fn a_floor_on_the_total_bounds_x_from_below_too() {
        let (mut game, spell) = board(3, sorcery(&[X, R]).build());
        put_on_battlefield(&mut game, trinisphere(), 1);
        assert_eq!(offered(&game, spell), XOffer::Range { min: 0, max: 2 });
        let (mut game, spell) = board(2, sorcery(&[X, R]).build());
        put_on_battlefield(&mut game, trinisphere(), 1);
        assert_eq!(values(&game, spell), Err(NoLegalX::Unpayable { least: 0 }));
    }

    /// Doubling Cube under a Plains and six Mountains: the seven mana less
    /// the Cube's `{3}` leave `{W}{R}{R}{R}`, doubled to eight, so `{X}{W}`
    /// is paid up to X = 7, one more than the lands alone.
    #[test]
    fn a_pool_multiplier_raises_the_most() {
        let (mut game, spell) = board(6, sorcery(&[X, W]).build());
        put_on_battlefield(&mut game, plains(), 0);
        assert_eq!(offered(&game, spell), XOffer::Range { min: 0, max: 6 });
        put_on_battlefield(&mut game, doubling_cube(), 0);
        assert_eq!(offered(&game, spell), XOffer::Range { min: 0, max: 7 });
    }

    /// "X can't be 0" sets the least, and a board that pays only `{R}` pays
    /// no legal X.
    #[test]
    fn x_cannot_be_zero_sets_the_least() {
        let (game, spell) = board(1, sorcery(&[X, R]).x_cannot_be_zero().build());
        assert_eq!(values(&game, spell), Err(NoLegalX::Unpayable { least: 1 }));
        let (game, spell) = board(2, sorcery(&[X, R]).x_cannot_be_zero().build());
        let legal = values(&game, spell).unwrap();
        assert_eq!((legal.offered, legal.least), (XOffer::Range { min: 1, max: 1 }, LeastX::CannotBeZero));
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
            Ok(LegalXValues { offered: XOffer::Range { min: 0, max: 0 }, least: LeastX::NotInTheCostPaid, most: MostX::TheLeast })
        );
        let (game, spell) = board(5, sorcery(&[X, R]).x_cannot_be_zero().build());
        assert_eq!(values_with(&game, spell, Some(&free), &[]), Err(NoLegalX::CannotBeZero));
    }

    /// An alternative cost's own X is bounded by its own total: miracle's
    /// `{X}{R}` on a `{X}{R}{R}` card, three Mountains.
    #[test]
    fn an_alternative_costs_x_is_bounded_by_that_cost() {
        let miracle = AlternativeCost::Custom("Miracle".to_string(), vec![Cost::Mana(ManaCost::from_symbols(vec![X, R]))]);
        let (game, spell) = board(3, sorcery(&[X, R, R]).build());
        assert_eq!(offered(&game, spell), XOffer::Range { min: 0, max: 1 });
        assert_eq!(values_with(&game, spell, Some(&miracle), &[]).unwrap().offered, XOffer::Range { min: 0, max: 2 });
    }

    /// A kicker announced before X takes its share first, and a kicker the
    /// board cannot pay beside the least X leaves no legal X.
    #[test]
    fn an_announced_kicker_narrows_the_most() {
        let kicker = |generic| AdditionalCost::Kicker(vec![Cost::Mana(ManaCost::build(&[], generic))]);
        let (game, spell) = board(5, sorcery(&[X, R]).build());
        assert_eq!(values_with(&game, spell, None, &[&kicker(2)]).unwrap().offered, XOffer::Range { min: 0, max: 2 });
        assert_eq!(values_with(&game, spell, None, &[&kicker(5)]), Err(NoLegalX::Unpayable { least: 0 }));
    }

    #[test]
    fn the_offer_names_its_one_value_and_what_it_offers() {
        assert_eq!(XOffer::Range { min: 2, max: 2 }.only(), Some(2));
        assert_eq!(XOffer::Range { min: 0, max: 2 }.only(), None);
        assert_eq!(XOffer::Values(vec![4]).only(), Some(4));
        let tamiyo = XOffer::Values(vec![0, 1, 4, 6, 7]);
        assert!(tamiyo.offers(4) && !tamiyo.offers(2));
        assert!(XOffer::Range { min: 1, max: 3 }.offers(3) && !XOffer::Range { min: 1, max: 3 }.offers(0));
    }
}
