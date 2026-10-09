//! Objects returning: `Primitive::ReturnToBattlefield` (Flickerwisp's
//! "return that card"), CR 610.3's "until" returns, and the Aura's host it
//! chooses as it enters the battlefield (CR 303.4f/g).

use crate::engine::actions::{ActionContext, GameAction};
use crate::engine::resolve::ResolvedTarget;
use crate::events::event::{EventSeq, GameEvent, ResolutionStamp};
use crate::oracle::characteristics::has_subtype;
use crate::state::game_state::GameState;
use crate::types::card_types::{EnchantmentType, Subtype};
use crate::types::effects::{EffectRecipient, ReturnUnder, SelectionFilter, TargetCount};
use crate::types::ids::{ObjectId, ObjectRef, PlayerId, UntilReturnId};
use crate::types::triggers::{RememberedObject, TriggerEvent, TriggerSubject};
use crate::types::zones::{UntilReturn, Zone, ZoneChangeCause};
use crate::ui::decision::DecisionProvider;

/// One object going back: where to, and under whose control if that is the
/// battlefield.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Return {
    pub object: ObjectId,
    pub to: Zone,
    pub controller: PlayerId,
}

impl GameState {
    /// Move each object back as one event. An Aura's host is chosen first,
    /// against the board before any of them entered, so it cannot be a
    /// permanent entering beside it (Calix, Destiny's Hand's ruling); an Aura
    /// with none stays where it is (CR 303.4g). An object already where it
    /// is going, or on the stack, is not moved.
    pub(crate) fn return_objects(&mut self, returns: &[Return], ctx: &ActionContext) -> Result<(), String> {
        let mut batch = Vec::new();
        let mut hosts = Vec::new();
        for &Return { object, to, controller } in returns {
            let from = self.get_object(object)?.zone;
            if from == to || from == Zone::Stack {
                continue;
            }
            if to != Zone::Battlefield {
                batch.push(GameAction::ZoneChange { object, from, to, cause: ZoneChangeCause::Returned });
                continue;
            }
            if has_subtype(self, object, &Subtype::Enchantment(EnchantmentType::Aura)) {
                match self.choose_what_it_enchants(object, controller, ctx.dp)? {
                    Some(host) => hosts.push((object, host)),
                    None => continue,
                }
            }
            batch.extend(self.entry_proposal(object, Some(from), controller, Some(ZoneChangeCause::Returned)));
        }
        if batch.is_empty() {
            return Ok(());
        }
        self.execute_actions(batch, ctx)?;
        // As the Aura spell's attach (`engine/stack.rs`): an Aura whose entry
        // happened, to a host still there.
        for (aura, host) in hosts {
            self.attach(aura, host);
        }
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
    /// by identity, each with the zone it came from.
    pub(crate) fn exiled_since(&self, stamp: ResolutionStamp, mark: EventSeq) -> Vec<(ObjectRef, Zone)> {
        let mut exiled: Vec<(ObjectRef, Zone)> = Vec::new();
        for record in self.events.resolution_records(stamp, mark) {
            if let GameEvent::ZoneChange { object_id, from, to: Zone::Exile, .. } = &record.event
                && let Some(object) = self.object_ref(*object_id)
                && self.get_object(*object_id).is_ok_and(|o| o.zone == Zone::Exile)
                && !exiled.iter().any(|(seen, _)| *seen == object)
            {
                exiled.push((object, *from));
            }
        }
        exiled
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
    ) {
        if returns.is_empty() {
            return;
        }
        let id = UntilReturnId(self.next_until_return_id);
        self.next_until_return_id += 1;
        self.emit_event(GameEvent::UntilReturnMade {
            id,
            source: source.id,
            controller,
            returns: returns.iter().map(|(object, _)| object.id).collect(),
        });
        let created_at = self.events.next_seq();
        let owner = self.owner_now_or(source.id, controller);
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
        });
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
        let TriggerEvent::ZoneChange { subject, from: Some(Zone::Battlefield), to: None, cause: None, owner: None, .. } =
            until
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
        let here = |object: &ObjectRef| self.object_ref(object.id) == Some(*object) && self.battlefield.contains_key(&object.id);
        Ok(watched.is_empty() || !watched.iter().all(here))
    }

    /// CR 610.3 — perform the returns that are `due`, now and as one event,
    /// since one-shot effects created after simultaneous events are
    /// simultaneous (610.3d). Each object goes back to its previous zone
    /// while it is still the object the exile moved (CR 400.7), a permanent
    /// under its owner's control unless the card said otherwise (610.3c).
    /// The return is no part of a resolution whose batch caused the event.
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
                let controller = match until.under {
                    ReturnUnder::Owner => self.get_object(object)?.owner,
                    ReturnUnder::You => until.controller,
                };
                returns.push(Return { object, to, controller });
            }
        }
        self.return_objects(&returns, &ActionContext::new(ctx.dp))
    }
}

#[cfg(test)]
mod tests {
    use crate::cards::creatures::grizzly_bears;
    use crate::engine::actions::GameAction;
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
        game.wait_to_return(gains, source, &[], vec![(card, Zone::Battlefield)], ReturnUnder::Owner, 0);

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
        game.wait_to_return(gains, source_ref, &[], vec![(card, Zone::Battlefield)], ReturnUnder::Owner, 1);
        game.remove_from_game(source).unwrap();

        game.execute_action(GameAction::GainLife { player: 0, amount: 1, source: exiled }, &test_ctx()).unwrap();
        assert_eq!(game.get_object(exiled).unwrap().zone, Zone::Battlefield, "the owner, P0, gained life");
    }
}
