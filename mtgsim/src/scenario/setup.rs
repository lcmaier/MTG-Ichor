//! Setup actions (`setup-architecture.md` §5.3): a scenario's `then:` lines,
//! each one seat's action, resolved by the loader and played by
//! [`SetupDriver`] before anyone else is asked.

use std::cell::{Cell, RefCell};

use super::board::SetupAction;
use crate::engine::resolve::ResolvedTarget;
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::ManaCost;
use crate::ui::auto_payer::AutoPayer;
use crate::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption, position_of};
use crate::ui::decision::{DecisionProvider, PriorityAction, SeatMode, Stop};
use crate::ui::mana_window_stop::ManaWindowStop;
use crate::ui::random::{mana_window_preference, WindowPreference};

/// A scenario's setup actions, in file order, each name resolved through the
/// objects the loader created (`Scenario::build`), so nothing later matches a
/// name against the board.
#[derive(Debug, Clone, Default)]
pub struct SetupActions {
    pub(super) lines: Vec<ResolvedSetupAction>,
}

impl SetupActions {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }
}

/// One `then:` line, as it is played: the option it picks, by id.
#[derive(Debug, Clone)]
pub(super) struct ResolvedSetupAction {
    pub(super) line: usize,
    /// The line as the file wrote it, which a refusal quotes.
    pub(super) written: SetupAction,
    /// What the line's seat picks when it holds priority, as the engine
    /// offers it: `CastSpell` or `ActivateAbility`.
    pub(super) action: PriorityAction,
    /// `written.targets`, each resolved, in the same order.
    pub(super) targets: Vec<ResolvedTarget>,
}

/// Plays a scenario's setup actions from the board, then hands every prompt
/// to `inner`, the seats' own providers.
///
/// While a line is left, every prompt is the driver's, and every seat stops
/// at every priority point, so none is passed over unseen. The seat holding
/// priority passes until the line's seat holds it, which picks the line's
/// action by id; the action's questions are answered from the line, and its
/// costs paid from the untapped lands, `AutoPayer` over `ManaWindowStop` as a
/// seat stacks them. Anything else is refused, naming the line: an action
/// the engine does not offer when its seat holds priority, one it rewinds
/// once picked, and a question no line answers. A refusal in play raises a
/// [`Stop`], which `Game::until_stopped` returns naming the line.
pub struct SetupDriver<'a> {
    lines: AutoPayer<ManaWindowStop<LineAnswers<'a>>>,
}

impl<'a> SetupDriver<'a> {
    pub fn new(setup: SetupActions, inner: &'a dyn DecisionProvider) -> SetupDriver<'a> {
        let answers = LineAnswers { lines: setup.lines, next: Cell::new(0), picked_at: Cell::new(None), taken: RefCell::new(Vec::new()), inner };
        SetupDriver { lines: AutoPayer::new(ManaWindowStop::new(answers)) }
    }

    fn answers(&self) -> &LineAnswers<'a> {
        self.lines.inner().inner()
    }

    /// Who answers now: the lines while one is left, the seats after.
    fn asked(&self) -> &dyn DecisionProvider {
        if self.answers().playing() { &self.lines } else { self.answers().inner }
    }
}

impl DecisionProvider for SetupDriver<'_> {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        self.asked().pick_n(game, player, context, options, bounds)
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
        self.asked().pick_number(game, player, context, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.asked().allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        self.asked().choose_ordering(game, player, context, items)
    }

    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        if self.answers().playing() {
            SeatMode { stops_at_every_priority_point: true }
        } else {
            self.answers().inner.seat_mode(player)
        }
    }
}

/// The lines' own answers, innermost in the driver's stack.
struct LineAnswers<'a> {
    lines: Vec<ResolvedSetupAction>,
    /// The line being played, or the next to be.
    next: Cell<usize>,
    /// The stack's height when the line's action was picked, until the next
    /// priority prompt shows whether the engine played it: the cast or the
    /// activation puts its object on at this place (CR 601.2a, 602.2a), and
    /// only what it triggers goes above it.
    picked_at: Cell<Option<usize>>,
    /// Which of the line's targets a question has taken.
    taken: RefCell<Vec<bool>>,
    /// The seats' own providers, asked once the last line is played.
    inner: &'a dyn DecisionProvider,
}

/// What in a line answers a question. Every `ChoiceKind` is named, so a new
/// question decides at birth whether a line answers it: the mode prompt
/// `backlog.md` §2.7 adds brings `mode N` here, and the first X spell the
/// priority window offers brings `x N`.
enum LineAnswer<'k> {
    Action,
    Targets(ObjectId),
    /// CR 601.2g's window, for the spell or the activated permanent.
    Window(ObjectId, &'k ManaCost),
    /// CR 601.2h's generic split.
    Split(ObjectId),
    Unanswered,
}

fn line_answer(kind: &ChoiceKind) -> LineAnswer<'_> {
    match kind {
        ChoiceKind::PriorityAction => LineAnswer::Action,
        ChoiceKind::SelectRecipients { spell_id, .. } => LineAnswer::Targets(*spell_id),
        ChoiceKind::ManaAbilityWindow { spell_or_ability_id, remaining_cost } => LineAnswer::Window(*spell_or_ability_id, remaining_cost),
        ChoiceKind::GenericManaAllocation { spell_or_ability_id, .. } => LineAnswer::Split(*spell_or_ability_id),
        // CR 601.2f's order is `AutoPayer`'s, answered before a line is asked.
        ChoiceKind::OrderCostReductions { .. } => LineAnswer::Unanswered,
        // No X spell is offered at priority yet: `ManaPool::can_pay` and
        // `find_mana_sources` read no X (`codebase-state.md`'s CR 107 row).
        ChoiceKind::ChooseXValue { .. } => LineAnswer::Unanswered,
        // Asked while a line plays, with no word to answer them yet: the
        // line's own costs, the order of what it triggers, a replacement on
        // its events, and the state-based actions checked before and after
        // it. A board that needs one adds the word.
        ChoiceKind::ChooseAlternativeCost { .. }
        | ChoiceKind::ChooseAdditionalCosts { .. }
        | ChoiceKind::ChooseSacrificeForCost { .. }
        | ChoiceKind::OrderTriggers { .. }
        | ChoiceKind::ChooseReplacementEffect { .. }
        | ChoiceKind::ApplyOptionalReplacement { .. }
        | ChoiceKind::LegendRule { .. }
        | ChoiceKind::CommanderToCommandZoneSba { .. } => LineAnswer::Unanswered,
        // Never asked while a line plays: each waits for a step to pass
        // (combat's declarations and damage, cleanup's discard) or for a spell
        // or ability to resolve, and setup actions do neither. After the last
        // line the seats answer them as in any game, a trampler's damage too.
        ChoiceKind::DeclareAttackers
        | ChoiceKind::DeclareBlockers
        | ChoiceKind::AssignCombatDamage { .. }
        | ChoiceKind::AssignTrampleDamage { .. }
        | ChoiceKind::Discard { .. }
        | ChoiceKind::Scry { .. }
        | ChoiceKind::ScryOrder { .. }
        | ChoiceKind::ApplyOptionalEffect { .. }
        | ChoiceKind::AllocateNextDamage { .. }
        | ChoiceKind::ChooseDamageSource { .. }
        | ChoiceKind::ChooseEnteringController { .. }
        | ChoiceKind::ChooseAuxiliaryZoneChange { .. }
        | ChoiceKind::ChooseCopySource { .. } => LineAnswer::Unanswered,
    }
}

impl LineAnswers<'_> {
    fn playing(&self) -> bool {
        self.next.get() < self.lines.len()
    }

    /// The line being played; asked only while one is.
    fn line(&self) -> &ResolvedSetupAction {
        &self.lines[self.next.get()]
    }

    fn refuse(&self, why: impl std::fmt::Display) -> ! {
        let line = self.line();
        Stop::SetupRefused { line: line.line, written: line.written.to_string(), why: why.to_string() }.raise()
    }

    fn refuse_question(&self, player: PlayerId, kind: &ChoiceKind) -> ! {
        self.refuse(format!(
            "player {player} is asked {}, which no line answers: a trigger's question, a replacement's, a \"may\", or a cost \
             a line has no word for",
            kind.as_str()
        ))
    }

    /// Is `subject` the picked line's own: its object on the stack, or for an
    /// activation the permanent, whose costs CR 602.2b pays as a spell's?
    fn asks_for_the_line(&self, game: &GameState, subject: ObjectId) -> bool {
        let Some(depth) = self.picked_at.get() else { return false };
        game.stack.get(depth) == Some(&subject) || matches!(self.line().action, PriorityAction::ActivateAbility(source, _) if source == subject)
    }

    fn priority(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        self.confirm_played(game);
        if !self.playing() {
            // The last line is played. `Pass` alone is the engine's to take
            // unless the seat stops at every priority point, as it would
            // have been had the driver not been stopping every seat.
            if options.len() == 1 && !self.inner.seat_mode(player).stops_at_every_priority_point {
                return vec![0];
            }
            return self.inner.pick_n(game, player, context, options, bounds);
        }
        let line = self.line();
        let offered = |wanted: &PriorityAction| position_of(options, game, &ChoiceOption::Action(wanted.clone()).as_logged(game), &[]);
        if player != line.written.seat {
            // The seat holding priority passes until the line's seat holds it.
            return match offered(&PriorityAction::Pass) {
                Some(pass) => vec![pass],
                None => self.refuse(format!("player {player} holds priority and is not offered a pass")),
            };
        }
        let Some(index) = offered(&line.action) else {
            self.refuse(format!(
                "player {player} holds priority, and the engine does not offer this: the card is not where it can be cast or \
                 activated from now, its timing does not allow it, no target it needs is legal, or the untapped lands cannot pay"
            ));
        };
        self.picked_at.set(Some(game.stack.len()));
        *self.taken.borrow_mut() = vec![false; line.targets.len()];
        vec![index]
    }

    /// The first priority prompt after a line's action was picked shows
    /// whether the engine played it: its object is where it was picked for,
    /// with its stack entry written. A target no question took was the
    /// engine's choice, made where a choice has one legal answer, and must be
    /// among the targets it chose.
    fn confirm_played(&self, game: &GameState) {
        let Some(depth) = self.picked_at.take() else { return };
        let line = self.line();
        let entry = game.stack.get(depth).and_then(|id| game.stack_entries.get(id)).filter(|entry| match line.action {
            PriorityAction::CastSpell(card) => entry.object_id == card,
            PriorityAction::ActivateAbility(source, ability) => {
                entry.ability_identity.is_some_and(|identity| identity.source.id == source && identity.ability == ability)
            }
            PriorityAction::Pass | PriorityAction::PlayLand(_) => false,
        });
        let Some(entry) = entry else {
            self.refuse("the engine rewound it once picked (CR 732.1), most often for a cost the untapped lands cannot pay");
        };
        let chosen: Vec<ResolvedTarget> = entry.chosen_targets.iter().flat_map(|instance| instance.as_resolved_targets()).collect();
        let untaken = line.targets.iter().zip(self.taken.borrow().iter()).position(|(target, taken)| !taken && !chosen.contains(target));
        if let Some(i) = untaken {
            self.refuse(format!("it names {} as a target, and no target it was cast or activated with is that", line.written.targets[i]));
        }
        self.next.set(self.next.get() + 1);
    }

    /// One CR 601.2c choice: the line's targets not yet taken, in the line's
    /// order, as many of those the choice offers as it takes.
    fn targets(&self, game: &GameState, player: PlayerId, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        let mut picks: Vec<usize> = Vec::new();
        for (target, taken) in self.line().targets.iter().zip(self.taken.borrow_mut().iter_mut()) {
            if picks.len() == bounds.1 {
                break;
            }
            let wanted = match target {
                ResolvedTarget::Object(id) => ChoiceOption::Object(*id),
                ResolvedTarget::Player(target) => ChoiceOption::Player(*target),
            };
            if let Some(index) = position_of(options, game, &wanted.as_logged(game), &picks).filter(|_| !*taken) {
                picks.push(index);
                *taken = true;
            }
        }
        if picks.len() < bounds.0 {
            self.refuse(format!("player {player} is asked for a target (CR 601.2c), and no `targeting` segment left names one the choice offers"));
        }
        picks
    }

    /// A source that makes a pip still owed, by the random agent's
    /// preference taking its first source rather than a random one, and the
    /// line's own permanent last, since its ability may need to tap it.
    fn tap(&self, game: &GameState, player: PlayerId, remaining: &ManaCost, options: &[ChoiceOption]) -> Vec<usize> {
        let allowed: Vec<usize> = match mana_window_preference(game, player, remaining, options) {
            WindowPreference::Useful(indices) => indices,
            WindowPreference::AnyWillDo => (0..options.len()).collect(),
            // Nothing offered pays: the payment fails, the engine rewinds the
            // action, and the next priority prompt refuses the line.
            WindowPreference::Hopeless => return Vec::new(),
        };
        let own = match self.line().action {
            PriorityAction::ActivateAbility(source, _) => Some(source),
            PriorityAction::CastSpell(_) | PriorityAction::Pass | PriorityAction::PlayLand(_) => None,
        };
        let on_own = |index: &usize| matches!(options[*index], ChoiceOption::Action(PriorityAction::ActivateAbility(id, _)) if Some(id) == own);
        allowed.iter().copied().find(|index| !on_own(index)).or(allowed.first().copied()).into_iter().collect()
    }
}

/// Each bucket's minimum, then the rest in bucket order up to each bucket's
/// maximum: the caps leave every pip its own mana
/// (`ask_choose_generic_mana_allocation`), so any split they allow pays.
fn generic_split(total: u64, mins: &[u64], maxs: Option<&[u64]>) -> Vec<u64> {
    let mut split = mins.to_vec();
    let mut rest = total.saturating_sub(mins.iter().sum());
    for (i, amount) in split.iter_mut().enumerate() {
        let more = rest.min(maxs.map_or(u64::MAX, |maxs| maxs[i]).saturating_sub(*amount));
        *amount += more;
        rest -= more;
    }
    split
}

impl DecisionProvider for LineAnswers<'_> {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        match line_answer(&context.kind) {
            LineAnswer::Action => self.priority(game, player, context, options, bounds),
            LineAnswer::Targets(spell) if self.asks_for_the_line(game, spell) => self.targets(game, player, options, bounds),
            LineAnswer::Window(subject, remaining) if self.asks_for_the_line(game, subject) => self.tap(game, player, remaining, options),
            _ => self.refuse_question(player, &context.kind),
        }
    }

    fn pick_number(&self, _game: &GameState, player: PlayerId, context: &ChoiceContext, _min: u64, _max: u64) -> u64 {
        self.refuse_question(player, &context.kind)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        _buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        match line_answer(&context.kind) {
            LineAnswer::Split(subject) if self.asks_for_the_line(game, subject) => generic_split(total, per_bucket_mins, per_bucket_maxs),
            _ => self.refuse_question(player, &context.kind),
        }
    }

    fn choose_ordering(&self, _game: &GameState, player: PlayerId, context: &ChoiceContext, _items: &[ChoiceOption]) -> Vec<usize> {
        self.refuse_question(player, &context.kind)
    }
}
