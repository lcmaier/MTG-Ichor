//! An instruction, one verb on one recipient (CR 608.2c), as the words a
//! card file writes: "Target player draws three cards" is
//! `draw_cards(target_player(), 3)`. The recipient is always the first
//! argument and comes from [`super::recipients`]; an amount is a number or
//! an [`AmountExpr`].
//!
//! A verb that makes a continuous effect returns a [`ContinuousChange`], which states
//! its duration as the text does (`.until_end_of_turn()`), so a spell that
//! forgets one does not compile. Only [`super::static_ability`] takes one
//! bare, since a static ability's effect applies while its source does (CR
//! 611.3b). A replacement effect or a "can't" a resolution creates is a
//! [`ContinuousRuleChange`]: it states a duration too, but it is no change
//! to an object, so it joins no `.and_…` chain and no static ability takes
//! it.
//!
//! The words run in the CR's order: CR 701's keyword actions in 701's order,
//! then the plain verbs by their rules, then the continuous changes by
//! layer (CR 613.1), then the effects a resolution creates that are not
//! changes to an object (CR 614.1, 101.2).

use crate::objects::card_data::AbilityDef;
use crate::state::game_state::PhaseType;
use crate::types::costs::Cost;
use crate::types::effects::{
    AmountExpr, ColorChange, Condition, CopyException, CopyRoles, CostAnswer, CounterType, DiscardChooser, Duration, Effect, EffectRecipient,
    ExtraPhasesAfter, ManaOutput, ObjectSet, PatternFill, PlayerRef, Primitive, ReturnUnder, SelectionFilter, TargetCount, TokenDef, TypeChange,
};
use crate::types::mana::ManaCost;
use super::recipients::divided_among;
use crate::types::colors::Color;
use crate::types::keywords::KeywordFlag;
use crate::types::replacement::{AmountRewrite, EventPattern, ReplacementDef, Rewrite};
use crate::types::restriction::{Restriction, RestrictionDef};
use crate::types::triggers::{DelayedTriggerTemplate, TriggerEvent};

// ---------------------------------------------------------------------------
// CR 701's keyword actions
// ---------------------------------------------------------------------------

/// "Attach [this] to [recipient]" (CR 701.3a).
pub fn attach_to(recipient: EffectRecipient) -> Effect {
    Effect::Atom(Primitive::Attach, recipient)
}

/// "Counter target spell" (CR 701.6a): the spell goes to its owner's
/// graveyard.
pub fn counter_spell(recipient: EffectRecipient) -> Effect {
    Effect::Atom(Primitive::CounterSpell, recipient)
}

/// "[Recipient] creates [amount] [token]s" (CR 701.7a): "create a token" is
/// an amount of 1.
pub fn create_tokens(recipient: EffectRecipient, amount: impl Into<AmountExpr>, token: TokenDef) -> Effect {
    Effect::Atom(Primitive::CreateToken(token, amount.into()), recipient)
}

/// "Destroy [recipient]" (CR 701.8a).
pub fn destroy(recipient: EffectRecipient) -> Effect {
    Effect::Atom(Primitive::Destroy, recipient)
}

/// "[Recipient] discards [amount] cards" (CR 701.9a), the cards chosen by
/// `chooser`.
pub fn discard(recipient: EffectRecipient, amount: impl Into<AmountExpr>, chooser: DiscardChooser) -> Effect {
    Effect::Atom(Primitive::Discard(amount.into(), chooser), recipient)
}

/// "Exile [recipient]" (CR 701.13a).
pub fn exile(recipient: EffectRecipient) -> Effect {
    Effect::Atom(Primitive::Exile, recipient)
}


/// "[Recipient] mills [amount] cards" (CR 701.17a).
pub fn mill(recipient: EffectRecipient, amount: impl Into<AmountExpr>) -> Effect {
    Effect::Atom(Primitive::Mill(amount.into()), recipient)
}

/// "Sacrifice [recipient]" (CR 701.21a).
pub fn sacrifice(recipient: EffectRecipient) -> Effect {
    Effect::Atom(Primitive::Sacrifice, recipient)
}

/// "[Recipient] scries [amount]" (CR 701.22a).
pub fn scry(recipient: EffectRecipient, amount: impl Into<AmountExpr>) -> Effect {
    Effect::Atom(Primitive::Scry(amount.into()), recipient)
}

/// "Shuffle [recipient's owner's] library" (CR 701.24a).
pub fn shuffle_library(recipient: EffectRecipient) -> Effect {
    Effect::Atom(Primitive::ShuffleLibrary, recipient)
}

/// "Tap [recipient]" (CR 701.26a).
pub fn tap(recipient: EffectRecipient) -> Effect {
    Effect::Atom(Primitive::Tap, recipient)
}

/// "Untap [recipient]" (CR 701.26a).
pub fn untap(recipient: EffectRecipient) -> Effect {
    Effect::Atom(Primitive::Untap, recipient)
}

// ---------------------------------------------------------------------------
// The plain verbs, by their rules
// ---------------------------------------------------------------------------

/// "[Recipient] wins the game" (CR 104.2b).
pub fn win_the_game(recipient: EffectRecipient) -> Effect {
    Effect::Atom(Primitive::WinGame, recipient)
}

/// "[Recipient] loses the game" (CR 104.3e).
pub fn lose_the_game(recipient: EffectRecipient) -> Effect {
    Effect::Atom(Primitive::LoseGame, recipient)
}

/// "Add [mana]" (CR 106.4): the ability's controller adds it, so the
/// instruction names no player.
pub fn add_mana(output: ManaOutput) -> Effect {
    Effect::Atom(Primitive::ProduceMana(output), EffectRecipient::Implicit)
}

/// "Its controller adds [mana]" (CR 106.4): the controller of the object the
/// recipient names, Wild Growth's enchanted land.
pub fn its_controller_adds(recipient: EffectRecipient, output: ManaOutput) -> Effect {
    Effect::Atom(Primitive::ProduceMana(output), recipient)
}

/// "[Player] may pay [costs]" (CR 118.12): a cost paid as the effect
/// resolves, offered only when that player can pay (CR 118.3). An "if [they]
/// do" or "if [they] don't" after it reads their answer. The costs are
/// written as the card prints them; their mana is gathered into the one
/// component `Primitive::Pay` holds.
pub fn may_pay(player: PlayerRef, costs: Vec<Cost>) -> Effect {
    let (mana, others) = Cost::split_out_mana_costs(costs);
    offered_to(player, Primitive::Pay { mana: mana.map(ManaCost::from_symbols), others, any_amount_of: None })
}

/// "[Player] may pay any amount of [each]" (CR 107.3f): how many is chosen as
/// they pay, and it is the resolution's X.
pub fn may_pay_any_amount_of(player: PlayerRef, each: ManaCost) -> Effect {
    offered_to(player, Primitive::Pay { mana: None, others: Vec::new(), any_amount_of: Some(each) })
}

/// `pay` offered to `player`: the `Effect::Optional` a payment is the whole
/// of, its chooser the payer and its recipient the instruction's own player.
fn offered_to(player: PlayerRef, pay: Primitive) -> Effect {
    Effect::Optional { chooser: player, effect: Box::new(Effect::Atom(pay, EffectRecipient::Implicit)) }
}

impl Effect {
    /// "[This effect] unless [player] pays [costs]" (CR 118.12a), written in
    /// the order the card prints it: Mana Leak is
    /// `counter_spell(target_spell()).unless(its_controller(the_target()), three)`.
    /// It "means the same thing as" "[player] may pay [costs]. If they don't,
    /// [this effect]", and is built as that sentence.
    pub fn unless(self, player: PlayerRef, costs: Vec<Cost>) -> Effect {
        Effect::Sequence(vec![may_pay(player, costs), Effect::Conditional(Condition::CostAnswer(CostAnswer::Doesnt), Box::new(self))])
    }
}

/// "[Recipient] gains [amount] life" (CR 119.3).
pub fn gain_life(recipient: EffectRecipient, amount: impl Into<AmountExpr>) -> Effect {
    Effect::Atom(Primitive::GainLife(amount.into()), recipient)
}

/// "[Recipient] loses [amount] life" (CR 119.3).
pub fn lose_life(recipient: EffectRecipient, amount: impl Into<AmountExpr>) -> Effect {
    Effect::Atom(Primitive::LoseLife(amount.into()), recipient)
}

/// "[Recipient's] life total becomes [amount]" (CR 119.5).
pub fn set_life_total(recipient: EffectRecipient, amount: impl Into<AmountExpr>) -> Effect {
    Effect::Atom(Primitive::SetLifeTotal(amount.into()), recipient)
}

/// "[This] deals [amount] damage to [recipient]" (CR 120.1).
pub fn deal_damage(recipient: EffectRecipient, amount: impl Into<AmountExpr>) -> Effect {
    Effect::Atom(Primitive::DealDamage { amount: amount.into(), unpreventable: false }, recipient)
}

/// "[This] deals [amount] damage to [recipient]. The damage can't be
/// prevented." (CR 120.1, 615.12): a word of its own, so a card that prints
/// the clause says so by the word it picks.
pub fn deal_damage_unpreventable(recipient: EffectRecipient, amount: impl Into<AmountExpr>) -> Effect {
    Effect::Atom(Primitive::DealDamage { amount: amount.into(), unpreventable: true }, recipient)
}

/// What a divided shield's count is written as: each row's is its target's
/// share, read as the row is made, so no row ever holds this.
const EACH_TARGETS_SHARE: u64 = 0;

/// "[This] deals [total] damage divided as you choose among [among] [what]"
/// (CR 601.2d): the atom prints the total, and the clause carries it to the
/// announcement that divides it.
pub fn deal_damage_divided(what: SelectionFilter, among: TargetCount, total: impl Into<AmountExpr>) -> Effect {
    let total = total.into();
    deal_damage(divided_among(what, among, total.clone()), total)
}

/// "Distribute [total] [counter]s among [among] [what]" (CR 601.2d), put by you.
pub fn distribute_counters(what: SelectionFilter, among: TargetCount, counter: CounterType, total: impl Into<AmountExpr>) -> Effect {
    let total = total.into();
    put_counters(divided_among(what, among, total.clone()), counter, total)
}

/// "Prevent the next [total] damage that would be dealt this turn to [among]
/// [what], divided as you choose" (CR 601.2d, 615.7): each target a shield of
/// its own share, so the clause alone holds the total.
pub fn prevent_the_next_damage_divided(what: SelectionFilter, among: TargetCount, total: impl Into<AmountExpr>) -> Effect {
    prevent_the_next_n_damage_this_turn(divided_among(what, among, total.into()), EACH_TARGETS_SHARE)
}

/// "[Recipient] draws [amount] cards" (CR 121.1), one at a time (121.2):
/// "draw a card" is an amount of 1.
pub fn draw_cards(recipient: EffectRecipient, amount: impl Into<AmountExpr>) -> Effect {
    Effect::Atom(Primitive::DrawCards(amount.into()), recipient)
}

/// "Put [amount] [counter]s on [on]" (CR 122.1), put by you.
pub fn put_counters(on: EffectRecipient, counter: CounterType, amount: impl Into<AmountExpr>) -> Effect {
    Effect::Atom(Primitive::AddCounters { counter, amount: amount.into(), by: PlayerRef::You }, on)
}

/// "Remove [amount] [counter]s from [from]" (CR 122.1).
pub fn remove_counters(from: EffectRecipient, counter: CounterType, amount: impl Into<AmountExpr>) -> Effect {
    Effect::Atom(Primitive::RemoveCounters(counter, amount.into()), from)
}

/// "[Recipient] gets [amount] [counter]s", a player's counters such as
/// energy (CR 122.1, 107.14), given by you.
pub fn get_counters(recipient: EffectRecipient, counter: CounterType, amount: impl Into<AmountExpr>) -> Effect {
    Effect::Atom(Primitive::GetCounters { counter, amount: amount.into(), by: PlayerRef::You }, recipient)
}

/// "Return [recipient] to its owner's hand" (CR 400.6).
pub fn return_to_hand(recipient: EffectRecipient) -> Effect {
    Effect::Atom(Primitive::ReturnToHand, recipient)
}

/// "Return [recipient] to the battlefield under [whose] control" (CR 400.6),
/// untapped and with no counters.
pub fn return_to_the_battlefield(recipient: EffectRecipient, under: ReturnUnder) -> Effect {
    Effect::Atom(Primitive::ReturnToBattlefield { under, counters: Vec::new(), tapped: false, attacking: None }, recipient)
}

/// "[Recipient] takes an extra turn after this one" (CR 500.7): the turn
/// it is added after is the one in progress.
pub fn take_an_extra_turn_after_this_one(recipient: EffectRecipient) -> Effect {
    Effect::Atom(Primitive::ExtraTurn, recipient)
}

/// "After [after], there is an additional [phases]" (CR 500.8): `after` is
/// the phase they are added after, and the phases run in their order, in
/// the recipient's turn.
pub fn add_phases(recipient: EffectRecipient, phases: Vec<PhaseType>, after: ExtraPhasesAfter) -> Effect {
    Effect::Atom(Primitive::ExtraPhases { phases, after }, recipient)
}

/// "[Recipient] becomes a blocked creature" (CR 509.1h).
pub fn becomes_a_blocked_creature(recipient: EffectRecipient) -> Effect {
    Effect::Atom(Primitive::BecomeBlocked, recipient)
}

/// "At [the event], [effect]" as an instruction (CR 603.7a): a delayed
/// triggered ability, controlled by the recipient (603.7d).
pub fn create_delayed_trigger(recipient: EffectRecipient, template: DelayedTriggerTemplate) -> Effect {
    Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(template)), recipient)
}

/// "Exile [recipient] until [event]" (CR 610.3): returned as the event
/// happens, under `under`'s control, which is its owner's unless the card
/// says otherwise (610.3c).
pub fn exile_until(recipient: EffectRecipient, until: impl Into<TriggerEvent>, under: ReturnUnder) -> Effect {
    Effect::Atom(Primitive::ExileUntil { until: Box::new(until.into()), refers_to: None, under }, recipient)
}

// ---------------------------------------------------------------------------
// Continuous changes, by layer (CR 613.1)
// ---------------------------------------------------------------------------

/// A continuous effect awaiting its duration (CR 611.2a): one or more
/// changes to one recipient. Each change is a step, a function from the
/// duration to its `Primitive`, since every continuous primitive carries
/// its duration and the text states it last: `gets(target_creature(), 3,
/// 3)` holds one step, `.and_gains_keyword(Trample)` adds a second, and
/// `.until_end_of_turn()` calls each with `UntilEndOfTurn`. End it with the
/// text's duration, or hand it bare to [`super::static_ability`].
///
/// "Gets +1/+0 **and** gains first strike" is one sentence and one
/// recipient, and the engine holds it as two instructions: the second acts
/// on what the first did, so when the first names a target the second is
/// [`super::that_target`] of it rather than a second target. That
/// back-reference is the work the chain does. It names the first instance
/// of "target" its effect declares, so a chain whose target is not the
/// first (a spell's second paragraph after a targeted first) writes its
/// changes as separate instructions on `that_target(n)`;
/// `cards::registry`'s authoring check catches the other spelling.
#[must_use = "a change lasts as long as its text says: end it with its duration"]
pub struct ContinuousChange {
    recipient: EffectRecipient,
    steps: Vec<Box<dyn FnOnce(Duration) -> Primitive>>,
}

impl ContinuousChange {
    fn new(recipient: EffectRecipient, step: impl FnOnce(Duration) -> Primitive + 'static) -> Self {
        ContinuousChange { recipient, steps: vec![Box::new(step)] }
    }

    fn and(mut self, step: impl FnOnce(Duration) -> Primitive + 'static) -> Self {
        self.steps.push(Box::new(step));
        self
    }

    /// "… and is [types]", "… and becomes [types]": [`becomes_types`] to the
    /// same recipient.
    pub fn and_becomes_types(self, change: TypeChange) -> Self {
        self.and(move |d| Primitive::ChangeType(change, d))
    }

    /// "… and becomes [colors]": [`becomes_color`] to the same recipient.
    pub fn and_becomes_color(self, colors: impl IntoIterator<Item = Color>) -> Self {
        let colors = colors.into_iter().collect();
        self.and(move |d| Primitive::ChangeColor(ColorChange::Set(colors), d))
    }

    /// "… and gains [keyword]": [`gains_keyword`] to the same recipient.
    pub fn and_gains_keyword(self, keyword: KeywordFlag) -> Self {
        self.and(move |d| Primitive::GrantKeywordFlag(keyword, d))
    }

    /// "… and loses all abilities": [`loses_all_abilities`] to the same
    /// recipient.
    pub fn and_loses_all_abilities(self) -> Self {
        self.and(Primitive::LoseAllAbilities)
    }

    /// "… with base power and toughness [p]/[t]":
    /// [`has_base_power_and_toughness`] to the same recipient.
    pub fn and_has_base_power_and_toughness(self, power: impl Into<AmountExpr>, toughness: impl Into<AmountExpr>) -> Self {
        let (power, toughness) = (power.into(), toughness.into());
        self.and(move |d| Primitive::SetPowerToughness(power, toughness, d))
    }

    /// "… and gets +[p]/+[t]": [`gets`] to the same recipient.
    pub fn and_gets(self, power: impl Into<AmountExpr>, toughness: impl Into<AmountExpr>) -> Self {
        let (power, toughness) = (power.into(), toughness.into());
        self.and(move |d| Primitive::ModifyPowerToughness(power, toughness, d))
    }

    /// "Until end of turn" (CR 514.2).
    pub fn until_end_of_turn(self) -> Effect {
        self.lasting(Duration::UntilEndOfTurn)
    }


    /// No duration printed: CR 611.2a's "until the end of the game".
    pub fn until_end_of_game(self) -> Effect {
        self.lasting(Duration::Indefinite)
    }

    /// The instructions, each lasting `duration`.
    pub(super) fn lasting(self, duration: Duration) -> Effect {
        let again = match &self.recipient {
            EffectRecipient::Target(..) | EffectRecipient::Choose(..) => EffectRecipient::SameInstanceAs(0),
            recipient => recipient.clone(),
        };
        let mut steps = self.steps.into_iter();
        let mut atoms = Vec::new();
        if let Some(first) = steps.next() {
            atoms.push(Effect::Atom(first(duration), self.recipient));
        }
        atoms.extend(steps.map(|step| Effect::Atom(step(duration), again.clone())));
        match atoms.len() {
            1 => atoms.remove(0),
            _ => Effect::Sequence(atoms),
        }
    }
}

/// Layer 1 (CR 707.2): a copy effect, `roles` saying which object copies
/// which, `except` its exceptions (CR 707.9).
pub fn copy(recipient: EffectRecipient, roles: CopyRoles, except: Vec<CopyException>) -> ContinuousChange {
    ContinuousChange::new(recipient, move |duration| Primitive::Copy { roles, except, duration })
}

/// Layer 2 (CR 613.1b): "gain control of [recipient]".
pub fn gain_control_of(recipient: EffectRecipient) -> ContinuousChange {
    ContinuousChange::new(recipient, |duration| Primitive::GainControl { to: PlayerRef::You, duration })
}

/// Layer 4 (CR 613.1d): "[recipient] is a [type] in addition to its other
/// types", "becomes an artifact creature".
pub fn becomes_types(recipient: EffectRecipient, change: TypeChange) -> ContinuousChange {
    ContinuousChange::new(recipient, move |d| Primitive::ChangeType(change, d))
}

/// Layer 5 (CR 613.1e): "[recipient] becomes [colors]", which replaces its
/// colors.
pub fn becomes_color(recipient: EffectRecipient, colors: impl IntoIterator<Item = Color>) -> ContinuousChange {
    let colors = colors.into_iter().collect();
    ContinuousChange::new(recipient, move |d| Primitive::ChangeColor(ColorChange::Set(colors), d))
}

/// Layer 5 (CR 613.1e): "[recipient] is [color] in addition to its other
/// colors".
pub fn is_also_color(recipient: EffectRecipient, color: Color) -> ContinuousChange {
    ContinuousChange::new(recipient, move |d| Primitive::ChangeColor(ColorChange::Add(color), d))
}

/// Layer 5 (CR 613.1e, 105.2c): "[recipient] becomes colorless".
pub fn becomes_colorless(recipient: EffectRecipient) -> ContinuousChange {
    ContinuousChange::new(recipient, |d| Primitive::ChangeColor(ColorChange::RemoveAll, d))
}

/// Layer 6 (CR 613.1f): "[recipient] gains [keyword]", "has [keyword]", for
/// a keyword held as a flag; any other ability is [`gains_ability`].
pub fn gains_keyword(recipient: EffectRecipient, keyword: KeywordFlag) -> ContinuousChange {
    ContinuousChange::new(recipient, move |d| Primitive::GrantKeywordFlag(keyword, d))
}

/// Layer 6 (CR 613.1f): "[recipient] gains '[ability]'", "has '[ability]'".
pub fn gains_ability(recipient: EffectRecipient, ability: AbilityDef) -> ContinuousChange {
    let ability = std::sync::Arc::new(ability);
    ContinuousChange::new(recipient, move |d| Primitive::GrantAbility(ability, d))
}

/// Layer 6 (CR 613.1f): "[recipient] loses all abilities".
pub fn loses_all_abilities(recipient: EffectRecipient) -> ContinuousChange {
    ContinuousChange::new(recipient, Primitive::LoseAllAbilities)
}

/// Layer 7b (CR 613.4b): "[recipient] has base power and toughness [p]/[t]".
pub fn has_base_power_and_toughness(recipient: EffectRecipient, power: impl Into<AmountExpr>, toughness: impl Into<AmountExpr>) -> ContinuousChange {
    let (power, toughness) = (power.into(), toughness.into());
    ContinuousChange::new(recipient, move |d| Primitive::SetPowerToughness(power, toughness, d))
}

/// Layer 7c (CR 613.4c): "[recipient] gets +[p]/+[t]".
pub fn gets(recipient: EffectRecipient, power: impl Into<AmountExpr>, toughness: impl Into<AmountExpr>) -> ContinuousChange {
    let (power, toughness) = (power.into(), toughness.into());
    ContinuousChange::new(recipient, move |d| Primitive::ModifyPowerToughness(power, toughness, d))
}

/// Layer 7d (CR 613.4d): "switch [recipient's] power and toughness".
pub fn switch_power_and_toughness(recipient: EffectRecipient) -> ContinuousChange {
    ContinuousChange::new(recipient, Primitive::SwitchPowerToughness)
}

// ---------------------------------------------------------------------------
// What a resolution creates that changes no object
// ---------------------------------------------------------------------------

/// A replacement effect or a "can't" a resolution creates (CR 614.1,
/// 101.2), awaiting its duration (CR 611.2a): CR 611.1's continuous effect
/// that affects players or the rules of the game, where a
/// [`ContinuousChange`] changes an object's characteristics or control. One
/// about an object, as Plague Spores' "They can't be regenerated", changes
/// what may happen to it, not it, so nothing chains to it, and a static
/// ability, whose effects last while it does, never writes one.
#[must_use = "a rule change lasts as long as its text says: end it with its duration"]
pub struct ContinuousRuleChange(ContinuousChange);

impl ContinuousRuleChange {
    /// "This turn", "until end of turn" (CR 514.2).
    pub fn until_end_of_turn(self) -> Effect {
        self.0.lasting(Duration::UntilEndOfTurn)
    }

    /// No duration printed: CR 611.2a's "until the end of the game".
    pub fn until_end_of_game(self) -> Effect {
        self.0.lasting(Duration::Indefinite)
    }
}

/// A replacement or prevention effect a resolution creates (CR 614.1,
/// 615.1), applying to what its def names: "prevent all combat damage that
/// would be dealt this turn".
pub fn create_replacement(def: ReplacementDef) -> ContinuousRuleChange {
    create_replacement_for(EffectRecipient::Implicit, def)
}

/// [`create_replacement`] with a recipient: a target or a filter, which the
/// resolution writes into the def's objects ("prevent the next 3 damage
/// that would be dealt to any target this turn"). A def that names its own
/// players is complete as authored, and is `create_replacement`'s.
pub fn create_replacement_for(recipient: EffectRecipient, def: ReplacementDef) -> ContinuousRuleChange {
    let def = Box::new(def);
    ContinuousRuleChange(ContinuousChange::new(recipient, move |d| Primitive::CreateReplacement(def, d, PatternFill::Authored)))
}

/// [`create_replacement`] for "a source of your choice" (CR 609.7a): the
/// source is chosen as the effect is created and written into the def.
pub fn create_replacement_from_a_chosen_source(def: ReplacementDef) -> ContinuousRuleChange {
    let def = Box::new(def);
    ContinuousRuleChange(ContinuousChange::new(EffectRecipient::Implicit, move |d| Primitive::CreateReplacement(def, d, PatternFill::ChosenDamageSource)))
}

/// "Prevent the next `n` damage that would be dealt to … this turn" — the def
/// every card that counts the damage it prevents shares.
///
/// [`AmountRewrite::PreventRemaining`] reads the count off
/// `Uses::NextDamage(n)`, so the number is written once; the object set is the
/// empty `Fixed` the resolution fills with its target — or, for Samite
/// Censer-Bearer, with each creature its filter finds — and the duration is the
/// card's "this turn", authored here because a duration the text did not state
/// would last the game (CR 611.2a).
pub fn prevent_the_next_n_damage_this_turn(recipient: EffectRecipient, n: u64) -> Effect {
    create_replacement_for(
        recipient,
        ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            ObjectSet::NO_OBJECTS,
            Rewrite::Amount(AmountRewrite::PreventRemaining),
        )
        .next_damage(n),
    )
    .until_end_of_turn()
}

/// "[What] can't happen" from a resolution (CR 101.2, 614.17), complete as
/// authored: "players can't gain life this turn".
pub fn cant(what: Restriction) -> ContinuousRuleChange {
    cant_for(EffectRecipient::Implicit, what)
}

/// [`cant`] with a recipient: a target's or a filter's objects, which the
/// resolution writes into what can't happen ("target creature can't be
/// regenerated").
pub fn cant_for(recipient: EffectRecipient, what: Restriction) -> ContinuousRuleChange {
    let def = RestrictionDef::new(what);
    ContinuousRuleChange(ContinuousChange::new(recipient, move |d| Primitive::Restrict(def, d)))
}
