//! Who or what caused an event (`replacement-architecture.md` §2b.3).
//!
//! CR 400.7j sets "an effect causes" beside "the cost of a spell or ability
//! causes", and the cards print the same verb: "a spell or ability an opponent
//! controls causes you to discard", "an effect causes", "can't cause you to
//! sacrifice". Every proposal names one (`ActionContext::cause`) and every
//! record carries it (`EventStamp::cause`), so a "can't", a replacement effect
//! and a trigger ask one value. The act — what a zone change did — is
//! [`ZoneChangeAct`](crate::types::zones::ZoneChangeAct), and is not this.
//!
//! **No catchall, here or in the four rule enums.** Each rule enum has one
//! variant per rule the engine performs, and a new rule's action arrives with
//! its own variant, as a new mover arrives with its act.

use crate::events::event::ResolutionStamp;
use crate::types::effects::CounterType;
use crate::types::ids::{ObjectId, PlayerId};

/// What caused an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventCause {
    /// An effect (CR 609.1): a resolving spell or ability (CR 608.2), a
    /// replacement effect's rider (CR 615.5) or its CR 614.13 moves, a mana
    /// ability (CR 605.3b), or the one-shot effect an "until" exile leaves
    /// behind (CR 610.3). `resolution` names the object and its controller.
    Effect { resolution: ResolutionStamp, generated_by: GeneratedBy },
    /// A cost being paid (CR 118.1, 601.2h, 602.2b): by whom, and the spell or
    /// the ability's source it is paid for. "Costs aren't effects" (Library of
    /// Leng's ruling). A payment a resolution asks for (CR 118.12) is the
    /// resolution's, an `Effect`, by Sigarda's Mogis ruling.
    Cost { player: PlayerId, paying_for: ObjectId },
    /// A player casting a spell (CR 601.2): the 601.2a move, announced at
    /// 601.2i, the cast and its targets' records.
    Casting(PlayerId),
    /// A player activating an ability (CR 602.2): the activation and its
    /// targets' records.
    Activating(PlayerId),
    /// A special action (CR 116.1), which uses no stack and is no effect.
    SpecialAction { player: PlayerId, action: SpecialAction },
    /// A turn-based action (CR 703.4).
    TurnBasedAction(TurnBasedAction),
    /// A state-based action (CR 704.5, 704.6).
    StateBasedAction(StateBasedAction),
    /// A rule acting with no action taken.
    GameRule(GameRule),
    /// A test or a tool moving objects directly; never the engine. Made only by
    /// `ActionContext::new`, which compiles only for tests, and it satisfies no
    /// source filter, so a cause-sensitive test that forgot to name a real
    /// cause fails rather than passing by accident.
    Harness,
}

impl EventCause {
    /// The resolution an effect belongs to; `None` for every other cause.
    pub fn resolution(&self) -> Option<ResolutionStamp> {
        match self {
            EventCause::Effect { resolution, .. } => Some(*resolution),
            EventCause::Cost { .. }
            | EventCause::Casting(_)
            | EventCause::Activating(_)
            | EventCause::SpecialAction { .. }
            | EventCause::TurnBasedAction(_)
            | EventCause::StateBasedAction(_)
            | EventCause::GameRule(_)
            | EventCause::Harness => None,
        }
    }

    /// Who controls the spell or ability that caused this, for "a spell or
    /// ability [someone] controls causes". `None` for anything that is not a
    /// spell's or an ability's effect: a cost, a special action, a turn-based
    /// or state-based action, a rule's move (Sigarda's ruling: "the result of
    /// game rules"), and an effect a counter made (CR 122.1c), which no spell
    /// or ability generated.
    pub fn spell_or_ability_controller(&self) -> Option<PlayerId> {
        match self {
            EventCause::Effect { resolution, generated_by } => match generated_by {
                GeneratedBy::Spell
                | GeneratedBy::ActivatedAbility
                | GeneratedBy::TriggeredAbility
                | GeneratedBy::StaticAbility => Some(resolution.controller),
                GeneratedBy::Counters(_) => None,
            },
            EventCause::Cost { .. }
            | EventCause::Casting(_)
            | EventCause::Activating(_)
            | EventCause::SpecialAction { .. }
            | EventCause::TurnBasedAction(_)
            | EventCause::StateBasedAction(_)
            | EventCause::GameRule(_)
            | EventCause::Harness => None,
        }
    }
}

/// What generated an effect: one of CR 113.3's four kinds of ability's
/// sources, or counters, which CR 122.1 lets create effects of their own.
///
/// Read by "triggered abilities you control can't cause" (The Master,
/// Multiplied) and "a white instant or sorcery spell causes" (Firesong and
/// Sunspeaker, which a cycled Renewed Faith's triggered ability must not
/// satisfy).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedBy {
    /// A spell's own instructions (CR 113.3a).
    Spell,
    /// CR 113.3b.
    ActivatedAbility,
    /// CR 113.3c.
    TriggeredAbility,
    /// CR 113.3d — a static ability's replacement effect, whose rider or
    /// CR 614.13 moves are effects of it.
    StaticAbility,
    /// CR 122.1c/d/h: shield, stun and finality counters create replacement
    /// and prevention effects. The shield's prevention half has a rider (it
    /// removes the counter), and no spell or ability generated it.
    Counters(CounterType),
}

/// A special action (CR 116.2). One variant per special action the engine
/// performs; face-down play, suspend, foretell, plot and the rest of 116.2
/// join with their mechanics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialAction {
    /// CR 116.2a, 305.1.
    PlayLand,
}

/// A turn-based action (CR 703.4), one variant per action the engine
/// performs that proposes or announces an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnBasedAction {
    /// CR 703.4c, 502.3.
    Untap,
    /// CR 703.4d, 504.1.
    Draw,
    /// CR 703.4i, 508.1: the attackers tap (508.1f) and are declared.
    DeclareAttackers,
    /// CR 703.4j, 509.1.
    DeclareBlockers,
    /// CR 703.4m, 510.2.
    CombatDamage,
    /// CR 703.4n, 514.1.
    CleanupDiscard,
}

/// A state-based action (CR 704.5, 704.6), one variant per subrule the sweep
/// performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateBasedAction {
    /// CR 704.5a.
    ZeroLife,
    /// CR 704.5b.
    DrewFromEmptyLibrary,
    /// CR 704.5c.
    TenPoisonCounters,
    /// CR 704.5d: the token ceases to exist.
    TokenOutsideTheBattlefield,
    /// CR 704.5f — not a destruction (CR 701.8b), so regeneration and shield
    /// counters do not answer it.
    ZeroToughness,
    /// CR 704.5g and 704.5h: one variant, since CR 701.8b calls both
    /// "destroyed" and no card tells them apart.
    LethalDamage,
    /// CR 704.5i.
    ZeroLoyalty,
    /// CR 704.5j.
    LegendRule,
    /// CR 704.5m.
    AuraAttachedIllegally,
    /// CR 704.5n.
    EquipmentAttachedIllegally,
    /// CR 704.5p: a permanent that is no Aura, Equipment or Fortification, or
    /// that is a creature or a battle, is attached.
    NonAttachmentAttached,
    /// CR 704.5q.
    CountersAnnihilate,
    /// CR 704.6c.
    CommanderDamage,
    /// CR 704.6d: its owner puts a commander into the command zone.
    CommanderToCommandZone,
}

/// A rule acting with no action taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameRule {
    /// CR 103.5: each player draws their opening hand.
    OpeningHand,
    /// CR 500.11, 614.10: a turn begins.
    TurnBegins,
    /// CR 500.11, 614.10: a phase begins.
    PhaseBegins,
    /// CR 500.11, 614.10: a step begins.
    StepBegins,
    /// CR 603.2: an ability triggers.
    AbilityTriggers,
    /// CR 603.3: a triggered ability is put on the stack, its targets chosen
    /// (603.3d).
    TriggeredAbilityPutOnStack,
    /// CR 608.2n: an instant or sorcery is put into its owner's graveyard as
    /// the final part of its resolution.
    SpellResolves,
    /// CR 608.2n: an ability is removed from the stack as the final part of
    /// its resolution.
    AbilityResolves,
    /// CR 608.3a, 608.3c: a permanent spell enters the battlefield.
    PermanentSpellResolves,
    /// CR 608.3e: a permanent spell that can't enter is put into its owner's
    /// graveyard.
    PermanentSpellCantEnter,
    /// CR 608.2b: every target is illegal, so it doesn't resolve.
    TargetsIllegal,
    /// CR 800.4a, 800.4c: what no player in the game controls is exiled.
    /// CR 800.4a says itself that it "is not a state-based action".
    ControllerLeftTheGame,
    /// CR 704.3: a state-based check performed something, one event.
    StateBasedCheck,
    /// CR 613.1b: a permanent's controller changed, which a continuous effect
    /// did by starting or ending, and which no performer proposes. The sweep
    /// that announces it cannot tell which effect it was.
    ControlChanges,
    /// CR 511.3: the end of combat step ended, removing everything from
    /// combat.
    CombatEnds,
}
