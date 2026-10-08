//! Objects returning to the battlefield: `Primitive::ReturnToBattlefield`
//! (Flickerwisp's "return that card"), and the Aura's host it chooses as it
//! enters (CR 303.4f/g).

use crate::engine::actions::ActionContext;
use crate::engine::resolve::ResolvedTarget;
use crate::oracle::characteristics::has_subtype;
use crate::state::game_state::GameState;
use crate::types::card_types::{EnchantmentType, Subtype};
use crate::types::effects::{EffectRecipient, SelectionFilter, TargetCount};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::zones::{Zone, ZoneChangeCause};
use crate::ui::decision::DecisionProvider;

impl GameState {
    /// Put each object onto the battlefield under its player's control, as
    /// one event. An Aura's host is chosen first, against the board before
    /// any of them entered, so it cannot be a permanent entering beside it
    /// (Calix, Destiny's Hand's ruling); an Aura with none stays where it is
    /// (CR 303.4g). An object already on the battlefield, or on the stack,
    /// is not returned.
    pub(crate) fn return_to_battlefield(
        &mut self,
        returns: &[(ObjectId, PlayerId)],
        ctx: &ActionContext,
    ) -> Result<(), String> {
        let mut entries = Vec::new();
        let mut hosts = Vec::new();
        for &(object, controller) in returns {
            let from = self.get_object(object)?.zone;
            if matches!(from, Zone::Battlefield | Zone::Stack) {
                continue;
            }
            if has_subtype(self, object, &Subtype::Enchantment(EnchantmentType::Aura)) {
                match self.choose_what_it_enchants(object, controller, ctx.dp)? {
                    Some(host) => hosts.push((object, host)),
                    None => continue,
                }
            }
            entries.extend(self.entry_proposal(object, Some(from), controller, Some(ZoneChangeCause::Returned)));
        }
        if entries.is_empty() {
            return Ok(());
        }
        self.execute_actions(entries, ctx)?;
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
}
