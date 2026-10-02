// FullControl — a seat's decorators taken off and put back mid-game.
//
// `backlog.md` §2.22 row 3. Off, the seat's prompts go to its decorated stack;
// on, to the raw provider, and the seat stops at every priority point, `Pass`
// alone included (`SeatMode`), which is what a person reads full control as.
// The census's rule 3 is why it is a switch above the stack and never a wrapper
// in it or a handle in each decorator: at most one decorator answers any one
// prompt, so none of them needs to know the switch exists, and a new decorator
// adds nothing here.
//
// **The switch is the client's.** A window flips it from its own thread, and
// `FullControl` reads it once per prompt, so a flip lands at the seat's next
// prompt wherever the engine is. A replay needs to know where it landed. A
// client that flips it between prompts records each answer the switch routes,
// the decorators' too, at the top of the seat where they all pass — never at
// the provider at the bottom, which a decorator's answer never reaches.
// `cli_play` flips it only while the seat answers a prompt, so its input
// stream is already that record.
//
// **Full control supersedes a yield** (the owner, 2026-09-30): while it is on,
// the seat's yields are cancelled, so turning it off again does not bring one
// back. A seat built without `superseding` keeps its yields through the
// switch, which is the other answer a test of the experience might want.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::state::game_state::GameState;
use crate::types::ids::PlayerId;
use crate::ui::auto_yield::Yields;
use crate::ui::choice_types::{ChoiceContext, ChoiceOption};
use crate::ui::decision::{DecisionProvider, SeatMode};

/// Full control's switch, shared by the client that flips it and the
/// [`FullControl`] above its seat that reads it.
#[derive(Clone, Debug, Default)]
pub struct FullControlSwitch(Arc<AtomicBool>);

impl FullControlSwitch {
    /// From any thread; the seat's next prompt is routed by it.
    pub fn set(&self, on: bool) {
        self.0.store(on, Ordering::Relaxed);
    }

    pub fn is_on(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// Routes a seat's prompts to its decorated stack `D`, or to its raw provider
/// `R` while the switch is on.
///
/// `R` and the provider at the bottom of `D` are one seat's: two values that
/// share whatever the provider keeps, as `cli_play`'s two terminals share their
/// handles. A provider that cannot be two values, such as a window's channel,
/// is shared through an `Rc` instead.
pub struct FullControl<D, R> {
    decorated: D,
    raw: R,
    switch: FullControlSwitch,
    /// The seat's yields, cancelled while the switch is on.
    superseded: Option<Yields>,
}

impl<D: DecisionProvider, R: DecisionProvider> FullControl<D, R> {
    pub fn new(decorated: D, raw: R, switch: FullControlSwitch) -> Self {
        FullControl { decorated, raw, switch, superseded: None }
    }

    /// Cancel the seat's yields while the switch is on.
    pub fn superseding(mut self, yields: Yields) -> Self {
        self.superseded = Some(yields);
        self
    }

    /// Whether the switch is on, cancelling the yields it supersedes if so.
    fn on(&self) -> bool {
        let on = self.switch.is_on();
        if on && let Some(yields) = &self.superseded {
            yields.clear();
        }
        on
    }

    fn routed(&self) -> &dyn DecisionProvider {
        if self.on() { &self.raw } else { &self.decorated }
    }
}

impl<D: DecisionProvider, R: DecisionProvider> DecisionProvider for FullControl<D, R> {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        self.routed().pick_n(game, player, context, options, bounds)
    }

    fn pick_number(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        min: u64,
        max: u64,
    ) -> u64 {
        self.routed().pick_number(game, player, context, min, max)
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
        self.routed().allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        items: &[ChoiceOption],
    ) -> Vec<usize> {
        self.routed().choose_ordering(game, player, context, items)
    }

    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        if !self.on() {
            return self.decorated.seat_mode(player);
        }
        let mut mode = self.raw.seat_mode(player);
        mode.stops_at_every_priority_point = true;
        mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::priority::PriorityResult;
    use crate::state::game_state::{Phase, PhaseType};
    use crate::test_support::setup_two_player_game;
    use crate::types::ids::new_object_id;
    use crate::types::mana::ManaCost;
    use crate::ui::choice_types::ChoiceKind;
    use crate::ui::decision::{DispatchDecisionProvider, PriorityAction, ScriptedDecisionProvider};
    use crate::ui::mana_window_stop::ManaWindowStop;

    fn covered_window() -> ChoiceContext {
        ChoiceContext::new(ChoiceKind::ManaAbilityWindow {
            spell_or_ability_id: new_object_id(),
            remaining_cost: ManaCost::zero(),
        })
    }

    fn a_source() -> Vec<ChoiceOption> {
        vec![ChoiceOption::Action(PriorityAction::ActivateAbility(
            new_object_id(),
            crate::types::ids::new_ability_id(),
        ))]
    }

    /// The stop's prompt both ways: off, `ManaWindowStop` declines the covered
    /// window and the provider beneath is never asked; on, the decorator is
    /// off and the raw provider takes the source.
    #[test]
    fn the_switch_takes_the_decorators_off_and_puts_them_back() {
        let game = setup_two_player_game();
        let raw = ScriptedDecisionProvider::new();
        raw.expect_pick_n(covered_window().kind, vec![0]);
        let switch = FullControlSwitch::default();
        let seat = FullControl::new(ManaWindowStop::new(ScriptedDecisionProvider::new()), raw, switch.clone());

        assert!(seat.pick_n(&game, 0, &covered_window(), &a_source(), (0, 1)).is_empty(), "the stop declined");
        switch.set(true);
        assert_eq!(seat.pick_n(&game, 0, &covered_window(), &a_source(), (0, 1)), vec![0], "the person tapped it");
        switch.set(false);
        assert!(seat.pick_n(&game, 0, &covered_window(), &a_source(), (0, 1)).is_empty(), "the stop is back");
    }

    /// A window's thread flips it; the next prompt on the engine's reads it.
    #[test]
    fn a_flip_from_another_thread_routes_the_next_prompt() {
        let switch = FullControlSwitch::default();
        let seat = FullControl::new(ScriptedDecisionProvider::new(), ScriptedDecisionProvider::new(), switch.clone());
        assert!(!seat.seat_mode(0).stops_at_every_priority_point);

        let window = switch.clone();
        std::thread::spawn(move || window.set(true)).join().expect("the window's thread");

        assert!(seat.seat_mode(0).stops_at_every_priority_point);
    }

    /// Turning full control on cancels the yield, so turning it off again
    /// asks the person rather than passing for them.
    #[test]
    fn full_control_cancels_the_yields_it_supersedes() {
        use crate::ui::auto_yield::{AutoYield, Yield};
        let game = setup_two_player_game();
        let priority = ChoiceContext::new(ChoiceKind::PriorityAction);
        let options = vec![
            ChoiceOption::Action(PriorityAction::Pass),
            ChoiceOption::Action(PriorityAction::CastSpell(new_object_id())),
        ];
        let yields = Yields::default();
        assert!(yields.set(&game, Yield::UntilEndOfTurn));
        let person = ScriptedDecisionProvider::new();
        person.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
        let raw = ScriptedDecisionProvider::new();
        raw.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
        let switch = FullControlSwitch::default();
        let seat = FullControl::new(AutoYield::new(person, yields.clone()), raw, switch.clone()).superseding(yields);

        assert_eq!(seat.pick_n(&game, 0, &priority, &options, (1, 1)), vec![0], "yielded");
        switch.set(true);
        assert_eq!(seat.pick_n(&game, 0, &priority, &options, (1, 1)), vec![1], "full control");
        switch.set(false);
        assert_eq!(seat.pick_n(&game, 0, &priority, &options, (1, 1)), vec![1], "the yield is gone");
    }

    /// Under full control the seat is asked where `Pass` is all it can do;
    /// the agent beside it is not.
    #[test]
    fn a_seat_in_full_control_is_asked_where_it_can_only_pass() {
        let mut game = setup_two_player_game();
        game.set_turn_position(Phase::new(PhaseType::Precombat));
        let raw = ScriptedDecisionProvider::new();
        raw.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
        let switch = FullControlSwitch::default();
        switch.set(true);
        let dp = DispatchDecisionProvider::new(vec![
            Box::new(FullControl::new(ScriptedDecisionProvider::new(), raw, switch)),
            Box::new(ScriptedDecisionProvider::new()),
        ]);

        assert_eq!(game.run_priority_round(&dp).unwrap(), PriorityResult::PhaseEnds);
    }
}
