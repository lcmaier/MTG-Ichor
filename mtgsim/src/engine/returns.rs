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
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::RememberedObject;
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

    /// CR 610.3 — make the return an "until" exile waits to make, for
    /// `watched`'s leaving the battlefield.
    pub(crate) fn wait_to_return(
        &mut self,
        watched: ObjectRef,
        returns: Vec<(ObjectRef, Zone)>,
        under: ReturnUnder,
        controller: PlayerId,
    ) {
        if returns.is_empty() {
            return;
        }
        let created_at = self.events.next_seq();
        self.until_returns.push(UntilReturn { watched: RememberedObject::now(watched), returns, under, controller, created_at });
    }

    /// The returns the window's departures from the battlefield cause, by
    /// their place on `until_returns`, in the order they were made. A
    /// departure is the watched object's own (`RememberedObject::is`) and
    /// after the return was made.
    pub(crate) fn departures_ending_an_until(&self, window: &[EventSeq]) -> Vec<usize> {
        if self.until_returns.is_empty() {
            return Vec::new();
        }
        let mut ended = Vec::new();
        for seq in window {
            let Some(record) = self.events.record(*seq) else { continue };
            let left = match &record.event {
                GameEvent::ZoneChange { object_id, from: Zone::Battlefield, .. }
                | GameEvent::LeftTheGame { object_id, from: Zone::Battlefield, .. } => *object_id,
                _ => continue,
            };
            for (at, until) in self.until_returns.iter().enumerate() {
                if record.seq >= until.created_at && until.watched.is(self, left, record.seq) && !ended.contains(&at) {
                    ended.push(at);
                }
            }
        }
        ended.sort_unstable();
        ended
    }

    /// CR 610.3 — perform the returns `ended` names, now and as one event,
    /// since one-shot effects created after simultaneous events are
    /// simultaneous (610.3d). Each object goes back to its previous zone
    /// while it is still the object the exile moved (CR 400.7), a permanent
    /// under its owner's control unless the card said otherwise (610.3c).
    /// The return is no part of a resolution whose batch caused the leaving.
    pub(crate) fn return_until(&mut self, ended: Vec<usize>, ctx: Option<&ActionContext>) -> Result<(), String> {
        if ended.is_empty() {
            return Ok(());
        }
        let mut at = 0;
        let mut taken = Vec::new();
        self.until_returns.retain(|until| {
            let keep = !ended.contains(&at);
            if !keep {
                taken.push(until.clone());
            }
            at += 1;
            keep
        });
        let Some(ctx) = ctx else {
            debug_assert!(false, "a departure from the battlefield is performed in a batch, which has a provider");
            return Ok(());
        };
        let mut returns = Vec::new();
        for until in taken {
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
