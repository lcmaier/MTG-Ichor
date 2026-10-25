//! CR 113.7a, 608.2h — the last known information of each existence that has
//! ended, in one store (`triggers-architecture.md` §3.11): written once, by the
//! move that ends it, and read by existence for the rest of the game.
//!
//! A batch frames each mover something reads later before it performs
//! (`capture_lki`); [`GameState::depart`], the before-move half of both
//! performers that take an object out of a zone, moves that capture into the
//! store. The write is this module's alone, so every entry is a departure, and
//! a departure's records write a history row for its object: the store never
//! holds more entries than the history log has rows
//! (`tests/phase_last_known_information_store_integration_test.rs`).

use crate::events::cause::EventCause;
use crate::events::event::UnattachRoute;
use crate::state::game_state::GameState;
use crate::state::shared_stores::ObjectTable;
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::Lki;
use crate::types::zones::Zone;

/// Every existence of one object that has ended with its move framed, each as
/// it last existed, in the order they ended, which is their epochs' order:
/// none for most objects, one for most that left, more for a commander or a
/// card a loop keeps returning. An older existence stays readable after a
/// newer one has ended too: a sacrificed creature's ability deals its damage
/// as the creature was, though the card has since been exiled from the
/// graveyard (CR 608.2h). The type is what the table stores for an object.
#[derive(Debug, Clone, Default)]
pub struct EndedExistences(Vec<Lki>);

/// The LKI store: each ended existence's last known information, by object,
/// so a lookup is the object's slot and that object's few existences. Kept
/// for the game; nothing in play iterates it.
#[derive(Debug, Clone, Default)]
pub struct LkiStore {
    ended: ObjectTable<EndedExistences>,
}

impl LkiStore {
    /// `existence`'s last known information, once it has ended and its move
    /// was framed: the object's slot, then its epoch.
    pub fn of(&self, existence: ObjectRef) -> Option<&Lki> {
        let ended = &self.ended.get(&existence.id)?.0;
        let at = ended.binary_search_by_key(&existence.zone_change_epoch, |lki| lki.object().zone_change_epoch).ok()?;
        Some(&ended[at])
    }

    /// The last known information of `id`'s most recent ended existence.
    fn latest(&self, id: ObjectId) -> Option<&Lki> {
        self.ended.get(&id)?.0.last()
    }

    /// File the last known information of an existence ending now.
    fn write(&mut self, lki: Lki) {
        let id = lki.object().id;
        match self.ended.get_mut(&id) {
            Some(ended) => {
                debug_assert!(
                    ended.0.last().is_none_or(|last| last.object().zone_change_epoch < lki.object().zone_change_epoch),
                    "{id}'s existences ended out of their epochs' order"
                );
                ended.0.push(lki);
            }
            None => {
                self.ended.insert(id, EndedExistences(vec![lki]));
            }
        }
    }

    /// Every entry, for a held-memory reading, which sums them in any order.
    #[cfg(any(test, feature = "test-support"))]
    pub fn entries(&self) -> impl Iterator<Item = &Lki> + '_ {
        self.ended.values().flat_map(|ended| ended.0.iter())
    }

    /// The bytes the store holds beside its entries' bodies and frames: the
    /// table, each object's slot and its list of pointers.
    #[cfg(any(test, feature = "test-support"))]
    pub fn bytes_held(&self) -> usize {
        let slot = std::mem::size_of::<EndedExistences>() + 2 * std::mem::size_of::<usize>();
        let lists: usize = self.ended.values().map(|ended| slot + ended.0.capacity() * std::mem::size_of::<Lki>()).sum();
        self.ended.bytes_held() + lists
    }
}

/// What an object leaving a zone takes with it, read at the last moment
/// before it moves ([`GameState::depart`]) and announced after: its frame,
/// if the move carries one, and each attachment the move ends, as
/// (attachment, former host, route). The attachments are read off the board
/// then, not off the frame, which was taken as the batch began: an earlier
/// member of the same batch may have ended one already (a host destroyed
/// beside its Aura), and the frame would still show it.
pub(crate) struct Departure {
    /// The frame the move's record carries, which only a move CR 603.10a
    /// looks back on has. The store keeps its own copy whenever one was
    /// captured, record or not.
    pub(crate) frame: Option<Lki>,
    /// Empty, and unallocated, when the move ends no attachment.
    unattached: Vec<(ObjectRef, ObjectRef, UnattachRoute)>,
}

impl GameState {
    /// How `existence` last existed, once it has ended (CR 113.7a, 608.2h):
    /// `None` while it has not, and where nothing framed its move, as for CR
    /// 601.2a's silent move to the stack.
    pub fn lki(&self, existence: ObjectRef) -> Option<&Lki> {
        self.lki_store.of(existence)
    }

    /// `id`'s owner (CR 108.3), the same in every existence: the object's
    /// while the store holds it, and once the store has lost it (a token that
    /// ceased to exist, CR 704.5d; an object whose owner left the game, CR
    /// 800.4a), its last known information's. `None` for one that left with
    /// no frame.
    pub fn owner_of(&self, id: ObjectId) -> Option<PlayerId> {
        self.objects.get(&id).map(|object| object.owner).or_else(|| self.lki_store.latest(id).map(Lki::owner))
    }

    /// What `id` leaving `from` for `to` (`None`: the game) takes with it,
    /// before it moves: the frame its batch took of it, filed in the LKI store
    /// (CR 113.7a), which its record carries too for a move in one of CR
    /// 603.10a's classes; and what the move unattaches (CR 701.3d). The one
    /// before-move half of both performers that take an object out of a zone,
    /// whose record and unattaches announce it after. A token whose entry is
    /// still being decided has no entity and nothing to look back at
    /// (`create_tokens`).
    pub(crate) fn depart(&mut self, id: ObjectId, from: Zone, to: Option<Zone>) -> Departure {
        let looks_back = self.zone_change_looks_back_at(id, from, to);
        let captured = if looks_back { self.take_captured_lki(id, from) } else { self.take_named_lki(id) };
        if let Some(lki) = &captured {
            self.lki_store.write(lki.clone());
        }
        Departure { frame: captured.filter(|_| looks_back), unattached: self.unattached_by_leaving(id) }
    }

    /// CR 701.3d — what a permanent's leaving the battlefield unattaches,
    /// read before it moves, since the move drops every link it is in
    /// (`cleanup_zone_state`): itself from its host, then whatever is attached
    /// to it, in the order they were attached. Each end is the existence it is
    /// before the move. Empty for anything not on the battlefield.
    pub(crate) fn unattached_by_leaving(&self, id: ObjectId) -> Vec<(ObjectRef, ObjectRef, UnattachRoute)> {
        let (Some(entry), Some(leaving)) = (self.battlefield.get(&id), self.object_ref(id)) else {
            return Vec::new();
        };
        let from_host = entry
            .attached_to
            .and_then(|host| self.object_ref(host))
            .map(|host| (leaving, host, UnattachRoute::AttachmentLeft));
        let from_it = entry
            .attached_by
            .iter()
            .filter_map(|&attachment| self.object_ref(attachment))
            .map(|attachment| (attachment, leaving, UnattachRoute::HostLeft));
        from_host.into_iter().chain(from_it).collect()
    }

    /// A departure's unattaches, announced after its own record by the
    /// performer that moved it, under the move's cause, each saying which ends
    /// left in the event: the leaving one where its move was framed (CR
    /// 603.10c), and the other end where its batch takes it later.
    pub(crate) fn announce_unattached_by_leaving(&mut self, departure: Departure, cause: EventCause) -> Result<(), String> {
        let Departure { frame: leaving_frame, unattached } = departure;
        let left = leaving_frame.is_some();
        for (attachment, former_host, route) in unattached {
            let (attachment_left, former_host_left) = match route {
                UnattachRoute::AttachmentLeft => (left, self.leaves_later_in_this_batch(former_host)),
                _ => (self.leaves_later_in_this_batch(attachment), left),
            };
            self.announce_unattached(attachment, former_host, route, attachment_left, former_host_left, cause)?;
        }
        Ok(())
    }
}
