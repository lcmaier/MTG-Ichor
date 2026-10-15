//! Objects returning: `Primitive::ReturnToBattlefield` (Flickerwisp's
//! "return that card"), CR 610.3's "until" returns, and the Aura's host it
//! chooses as it enters the battlefield (CR 303.4f/g).

use crate::engine::actions::{ActionContext, GameAction};
use crate::engine::resolve::ResolvedTarget;
use crate::events::cause::EventCause;
use crate::events::event::{EventSeq, GameEvent, ResolutionStamp};
use crate::oracle::characteristics::has_subtype;
use crate::state::game_state::GameState;
use crate::types::card_types::{EnchantmentType, Subtype};
use crate::types::effects::{EffectRecipient, ReturnUnder, SelectionFilter, TargetCount};
use crate::types::replacement::{EnterMods, EntryCounters};
use crate::types::ids::{ObjectId, ObjectRef, PlayerId, UntilReturnId};
use crate::types::triggers::{RememberedObject, TriggerEvent, TriggerSubject};
use crate::types::zones::{UntilReturn, Zone, ZoneChangeAct, ZoneChangePattern};
use crate::ui::decision::DecisionProvider;

/// One object an effect is putting back: where to, under whose control and
/// with what counters if that is the battlefield (CR 122.6), and what caused
/// it — the effect returning it.
#[derive(Debug, Clone)]
pub(crate) struct ReturningObject {
    pub object: ObjectId,
    pub to: Zone,
    pub controller: PlayerId,
    pub counters: Vec<EntryCounters>,
    pub cause: EventCause,
}

impl GameState {
    /// Whose control `object` returns under (CR 610.3c's default is its
    /// owner's), where "you" is `you`.
    pub(crate) fn return_controller(&self, object: ObjectId, under: ReturnUnder, you: PlayerId) -> Result<PlayerId, String> {
        Ok(match under {
            ReturnUnder::Owner => self.get_object(object)?.owner,
            ReturnUnder::You => you,
        })
    }

    /// Move each object back as one event. An Aura's host is chosen first,
    /// against the board before any of them entered, so it cannot be a
    /// permanent entering beside it (Calix, Destiny's Hand's ruling), and it
    /// rides on the entry, which attaches it (CR 303.4f); an Aura with none
    /// stays where it is (CR 303.4g). An object already where it is going,
    /// or on the stack, is not moved. Each return is caused by its own
    /// effect, and the event is `ctx`'s.
    pub(crate) fn return_objects(&mut self, returns: &[ReturningObject], ctx: &ActionContext) -> Result<(), String> {
        let mut batch = Vec::new();
        for ReturningObject { object, to, controller, counters, cause } in returns {
            let (object, to, controller, cause) = (*object, *to, *controller, *cause);
            let from = self.get_object(object)?.zone;
            if from == to || from == Zone::Stack {
                continue;
            }
            if to != Zone::Battlefield {
                batch.push((GameAction::ZoneChange { object, from, to, act: ZoneChangeAct::Put }, cause));
                continue;
            }
            let attached_to = if has_subtype(self, object, &Subtype::Enchantment(EnchantmentType::Aura)) {
                match self.choose_what_it_enchants(object, controller, ctx.dp)? {
                    Some(host) => Some(host),
                    None => continue,
                }
            } else {
                None
            };
            let Some(mut entry) = self.entry_proposal(object, Some(from), controller, Some(ZoneChangeAct::Put)) else {
                continue;
            };
            // "With a +1/+1 counter on it" and the Aura's host are the returning
            // effect's own word on how it enters, joining what the rules give
            // the entry, ahead of any replacement (CR 122.6, 303.4f, 614.1c).
            if let GameAction::EnterBattlefield { mods, .. } = &mut entry {
                mods.merge(&EnterMods { counters: counters.clone(), attached_to, ..EnterMods::NONE });
            }
            batch.push((entry, cause));
        }
        if batch.is_empty() {
            return Ok(());
        }
        self.execute_caused_actions(batch, ctx)?;
        Ok(())
    }

    /// CR 303.4f — what an Aura entering the battlefield other than by
    /// resolving will enchant, chosen by the player it enters under among the
    /// permanents its enchant ability allows (`can_enchant`, CR 704.5m's own
    /// question). Not targeting. `None` is CR 303.4g: nothing it could
    /// enchant, so it does not enter.
    fn choose_what_it_enchants(
        &self,
        aura: ObjectId,
        chooser: PlayerId,
        dp: &dyn DecisionProvider,
    ) -> Result<Option<ObjectId>, String> {
        let filter = self
            .get_object(aura)?
            .card_data
            .enchant_filter
            .clone()
            .ok_or_else(|| format!("the Aura {aura} has no enchant ability (CR 303.4a)"))?;
        // `PermanentState::attached_to` holds objects, so a Curse has no host
        // to enter attached to (`codebase-state.md` item 227).
        if matches!(filter, SelectionFilter::Player) {
            return Err(format!("the Aura {aura} enchants a player, and an attachment holds only objects"));
        }
        let candidates: Vec<ResolvedTarget> = self
            .battlefield_ids_ordered()
            .into_iter()
            .filter(|&host| host != aura && self.can_enchant(&filter, aura, host, chooser))
            .map(ResolvedTarget::Object)
            .collect();
        let question = EffectRecipient::Choose(filter, TargetCount::Exactly(1));
        let chosen = crate::ui::ask::ask_select_recipients(dp, self, chooser, &question, aura, &candidates, 1, 1);
        Ok(chosen.into_iter().find_map(|t| match t {
            ResolvedTarget::Object(host) => Some(host),
            ResolvedTarget::Player(_) => None,
        }))
    }

    /// What resolution `stamp` exiled from `mark` on and is still in exile,
    /// by identity, each with the zone it came from. A token it created in
    /// exile came from no zone and is never returned (CR 111.8, 610.3).
    pub(crate) fn exiled_since(&self, stamp: ResolutionStamp, mark: EventSeq) -> Vec<(ObjectRef, Zone)> {
        self.put_into_since(stamp, mark, Zone::Exile)
            .into_iter()
            .filter_map(|(object, from)| Some((object, from?)))
            .collect()
    }

    /// CR 610.3 — make the return an "until" exile waits to make: what it
    /// exiled, back when `until` happens. Announced first, so its making is
    /// no record its event can be.
    pub(crate) fn wait_to_return(
        &mut self,
        until: TriggerEvent,
        source: ObjectRef,
        referred: &[ObjectRef],
        returns: Vec<(ObjectRef, Zone)>,
        under: ReturnUnder,
        controller: PlayerId,
        cause: EventCause,
    ) -> Result<(), String> {
        if returns.is_empty() {
            return Ok(());
        }
        let id = UntilReturnId(self.next_until_return_id);
        self.next_until_return_id += 1;
        let made = GameEvent::UntilReturnMade {
            id,
            source: source.id,
            controller,
            returns: returns.iter().map(|(object, _)| object.id).collect(),
        };
        self.emit_event(made, cause)?;
        let created_at = self.events.next_seq();
        let owner = self.owner_of_source(source)?;
        self.until_returns.push(UntilReturn {
            id,
            until,
            source: RememberedObject::now(source),
            referred: referred.iter().copied().map(RememberedObject::now).collect(),
            returns,
            under,
            controller,
            owner,
            created_at,
            cause,
        });
        Ok(())
    }

    /// CR 610.3a, 610.3b — whether `until` has already happened since the
    /// spell was cast or the ability triggered, so nothing moves. A leaving
    /// has, when the object it names is gone or is a new object (CR 400.7):
    /// `source` for "this", `referred` for the target it named. Another event
    /// would need its own history to say, and is refused rather than guessed.
    pub(crate) fn until_has_happened(
        &self,
        until: &TriggerEvent,
        source: Option<ObjectRef>,
        referred: &[ObjectRef],
    ) -> Result<bool, String> {
        let TriggerEvent::ZoneChange {
            subject,
            change: ZoneChangePattern { from: Some(Zone::Battlefield), to: None, act: None },
            by: None,
            owner: None,
            ..
        } = until
        else {
            return Err(format!("CR 610.3a/b asks whether {until:?} has happened since, which needs that event's history"));
        };
        let watched: Vec<ObjectRef> = match subject {
            TriggerSubject::ThisObject => {
                vec![source.ok_or("a spell is never on the battlefield to leave it (CR 610.3)")?]
            }
            TriggerSubject::Referred => referred.to_vec(),
            other => return Err(format!("CR 610.3a/b asks whether {other:?} has left since, which needs its history")),
        };
        let here = |object: &ObjectRef| self.find(*object).is_some() && self.battlefield.contains_key(&object.id);
        Ok(watched.is_empty() || !watched.iter().all(here))
    }

    /// CR 610.3 — perform the returns that are `due`, now and as one event,
    /// since one-shot effects created after simultaneous events are
    /// simultaneous (610.3d). Each object goes back to its previous zone
    /// while it is still the object the exile moved (CR 400.7), a permanent
    /// under its owner's control unless the card said otherwise (610.3c).
    /// The return is no part of a resolution whose batch caused the event:
    /// each is the effect its exile left behind (610.3), and carries that
    /// exile's cause.
    pub(crate) fn return_until(&mut self, due: Vec<UntilReturn>, ctx: Option<&ActionContext>) -> Result<(), String> {
        if due.is_empty() {
            return Ok(());
        }
        let Some(ctx) = ctx else {
            debug_assert!(false, "a leaving is performed in a batch, which has a provider (item 230 for any other event)");
            return Ok(());
        };
        let mut returns = Vec::new();
        for until in due {
            for (object, to) in until.returns {
                let Some(object) = RememberedObject::now(object).found(self) else { continue };
                let controller = self.return_controller(object, until.under, until.controller)?;
                returns.push(ReturningObject { object, to, controller, counters: Vec::new(), cause: until.cause });
            }
        }
        let Some(first) = returns.first().map(|r| r.cause) else { return Ok(()) };
        self.return_objects(&returns, &ctx.caused_by(first))
    }
}

#[cfg(test)]
mod tests {
    use crate::cards::creatures::grizzly_bears;
    use crate::engine::actions::GameAction;
    use crate::events::cause::EventCause;
    use crate::test_support::{put_in_exile, put_on_battlefield, put_on_battlefield_under, setup_two_player_game, test_ctx};
    use crate::types::effects::{PlayerRef, ReturnUnder};
    use crate::types::triggers::{Multiplicity, TriggerEvent};
    use crate::types::zones::Zone;

    /// CR 610.3's event is any event: a return waiting on "you gain life"
    /// is made when its controller gains life, as a dispatch reads the
    /// record, and not before.
    #[test]
    fn an_until_waits_on_whatever_event_it_names() {
        let mut game = setup_two_player_game();
        let source = put_on_battlefield(&mut game, grizzly_bears(), 0);
        let exiled = put_in_exile(&mut game, grizzly_bears(), 1);
        let gains = TriggerEvent::GainsLife { player: Some(PlayerRef::You), multiplicity: Multiplicity::PerOccurrence };
        let (source, card) = (game.object_ref(source).unwrap(), game.object_ref(exiled).unwrap());
        game.wait_to_return(gains, source, &[], vec![(card, Zone::Battlefield)], ReturnUnder::Owner, 0, EventCause::Harness).unwrap();

        game.execute_action(GameAction::GainLife { player: 1, amount: 1, source: source.id }, &test_ctx()).unwrap();
        assert_eq!(game.get_object(exiled).unwrap().zone, Zone::Exile, "another player's gain is not \"you\"");
        game.execute_action(GameAction::GainLife { player: 0, amount: 1, source: source.id }, &test_ctx()).unwrap();
        assert_eq!(game.get_object(exiled).unwrap().zone, Zone::Battlefield);
        assert!(game.until_returns.is_empty());
    }

    /// CR 108.3: "its owner" is the source's owner, fixed as the return is
    /// made, and stays so once the store has lost the source (item 235). The
    /// source is P0's under P1's control, so a fallback to the controller
    /// waits on the wrong player.
    #[test]
    fn an_until_reads_its_sources_owner_after_the_source_is_gone() {
        let mut game = setup_two_player_game();
        let source = put_on_battlefield_under(&mut game, grizzly_bears(), 0, 1);
        let exiled = put_in_exile(&mut game, grizzly_bears(), 1);
        let gains = TriggerEvent::GainsLife { player: Some(PlayerRef::Owner), multiplicity: Multiplicity::PerOccurrence };
        let (source_ref, card) = (game.object_ref(source).unwrap(), game.object_ref(exiled).unwrap());
        game.wait_to_return(gains, source_ref, &[], vec![(card, Zone::Battlefield)], ReturnUnder::Owner, 1, EventCause::Harness).unwrap();
        game.remove_from_game(source).unwrap();

        game.execute_action(GameAction::GainLife { player: 0, amount: 1, source: exiled }, &test_ctx()).unwrap();
        assert_eq!(game.get_object(exiled).unwrap().zone, Zone::Battlefield, "the owner, P0, gained life");
    }
}
