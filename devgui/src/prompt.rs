//! The question as owned data: which primitive asked, what it asks, and each
//! option as `ui::display`'s label plus the board things it names. The names
//! are what let the window make exactly those things clickable with no case per
//! `ChoiceKind`.

use mtgsim::state::battlefield::AttackTarget;
use mtgsim::state::game_state::GameState;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::ui::auto_yield::Yield;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceOption};
use mtgsim::ui::decision::PriorityAction;
use mtgsim::ui::display::{option_label, question, rejection};

/// An answer, in the shape of the primitive that asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    Picks(Vec<usize>),
    Number(u64),
    Allocation(Vec<u64>),
    Order(Vec<usize>),
}

/// What the window sends the engine's thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    /// The answer to the open prompt.
    Answer(Answer),
    /// Pass at the open priority prompt, and keep passing until the yield
    /// ends (`mtgsim::ui::auto_yield`). The seat sets it; the log has the pass.
    Yield(Yield),
    /// End the seat's yield. The open prompt stays open.
    StopYielding,
}

/// Which of the four `DecisionProvider` methods asked, with its bounds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Primitive {
    PickN { min: usize, max: usize },
    Number { min: u64, max: u64 },
    Allocate { total: u64, mins: Vec<u64>, maxs: Option<Vec<u64>> },
    Order,
}

/// A thing on the board an option names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BoardRef {
    Object(ObjectId),
    Player(PlayerId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionView {
    pub label: String,
    /// In order: none, one, or a pair's two — attacker then what it attacks,
    /// blocker then what it blocks.
    pub refs: Vec<BoardRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prompt {
    /// The player asked: the window answers at that player's seat.
    pub player: PlayerId,
    /// The `ChoiceKind` variant's name, which the decision log records.
    pub kind: String,
    pub question: String,
    /// `ChoiceKind::subject()`.
    pub subject: Option<ObjectId>,
    /// Where a priority prompt offers `Pass`, which it always does: a
    /// priority prompt is the one a yield answers and Space passes at.
    pub pass: Option<usize>,
    /// Why the seat is asked again, in `ui::display::rejection`'s words: the
    /// answer the engine rejected and the rule.
    pub rejected: Option<String>,
    pub primitive: Primitive,
    /// A pick's or an ordering's options, an allocation's buckets; none for a number.
    pub options: Vec<OptionView>,
}

impl Prompt {
    pub fn pick_n(
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Prompt {
        Prompt::new(game, player, context, Primitive::PickN { min: bounds.0, max: bounds.1 }, options)
    }

    pub fn number(game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> Prompt {
        Prompt::new(game, player, context, Primitive::Number { min, max }, &[])
    }

    pub fn allocate(
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        mins: &[u64],
        maxs: Option<&[u64]>,
    ) -> Prompt {
        let primitive = Primitive::Allocate { total, mins: mins.to_vec(), maxs: maxs.map(<[u64]>::to_vec) };
        Prompt::new(game, player, context, primitive, buckets)
    }

    pub fn order(game: &GameState, player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Prompt {
        Prompt::new(game, player, context, Primitive::Order, items)
    }

    fn new(
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        primitive: Primitive,
        options: &[ChoiceOption],
    ) -> Prompt {
        Prompt {
            player,
            kind: context.kind.as_str().to_string(),
            question: question(game, &context.kind),
            subject: context.kind.subject(),
            pass: options.iter().position(|option| matches!(option, ChoiceOption::Action(PriorityAction::Pass))),
            rejected: context.rejected.as_ref().map(|rejected| rejection(game, rejected)),
            primitive,
            options: options.iter().map(|option| option_view(game, option)).collect(),
        }
    }
}

/// The board things an option names, in the order `OptionView::refs` keeps.
fn option_view(game: &GameState, option: &ChoiceOption) -> OptionView {
    let refs = match option {
        ChoiceOption::Object(id) => vec![BoardRef::Object(*id)],
        ChoiceOption::Player(player) => vec![BoardRef::Player(*player)],
        ChoiceOption::Action(PriorityAction::Pass) => vec![],
        ChoiceOption::Action(
            PriorityAction::CastSpell(id) | PriorityAction::PlayLand(id) | PriorityAction::ActivateAbility(id, _),
        ) => vec![BoardRef::Object(*id)],
        ChoiceOption::AttackerTarget(attacker, target) => vec![BoardRef::Object(*attacker), attack_target_ref(target)],
        ChoiceOption::BlockerAttacker(blocker, attacker) => {
            vec![BoardRef::Object(*blocker), BoardRef::Object(*attacker)]
        }
        ChoiceOption::NormalCost
        | ChoiceOption::AlternativeCost(_)
        | ChoiceOption::AdditionalCost(_)
        | ChoiceOption::Number(_)
        | ChoiceOption::Color(_)
        | ChoiceOption::CounterType(_)
        | ChoiceOption::ManaType(_) => vec![],
    };
    OptionView { label: option_label(game, option), refs }
}

fn attack_target_ref(target: &AttackTarget) -> BoardRef {
    match target {
        AttackTarget::Player(player) => BoardRef::Player(*player),
        AttackTarget::Planeswalker(id) | AttackTarget::Battle(id) => BoardRef::Object(*id),
    }
}

#[cfg(test)]
mod tests {
    use mtgsim::test_support::{put_on_battlefield, setup_two_player_game, vanilla_creature};

    use super::*;

    /// The view model reads a pair's first ref as what the person clicks
    /// first, so an attack and a block name the creature acting first.
    #[test]
    fn a_pair_names_the_creature_acting_first() {
        let mut game = setup_two_player_game();
        let attacker = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        let blocker = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
        let attack = option_view(&game, &ChoiceOption::AttackerTarget(attacker, AttackTarget::Player(1)));
        assert_eq!(attack.refs, [BoardRef::Object(attacker), BoardRef::Player(1)], "{}", attack.label);
        let block = option_view(&game, &ChoiceOption::BlockerAttacker(blocker, attacker));
        assert_eq!(block.refs, [BoardRef::Object(blocker), BoardRef::Object(attacker)], "{}", block.label);
    }
}
