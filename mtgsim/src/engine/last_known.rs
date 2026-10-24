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
//! (`tests/last_known_store_test.rs`).

use crate::events::cause::EventCause;
use crate::events::event::UnattachRoute;
use crate::state::game_state::GameState;
use crate::state::shared_stores::ObjectTable;
use crate::types::ids::{ObjectId, ObjectRef};
use crate::types::triggers::Lki;
use crate::types::zones::Zone;

/// Every existence of one object that has ended with its move framed, each as
/// it last existed, in the order they ended: none for most objects, one for
/// most that left, more for a commander or a card a loop keeps returning.
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
    /// was framed.
    pub fn of(&self, existence: ObjectRef) -> Option<&Lki> {
        self.ended.get(&existence.id)?.0.iter().rev().find(|lki| lki.object() == existence)
    }

    /// File the last known information of an existence ending now.
    fn write(&mut self, lki: Lki) {
        let id = lki.object().id;
        match self.ended.get_mut(&id) {
            Some(ended) => {
                debug_assert!(
                    ended.0.iter().all(|kept| kept.object() != lki.object()),
                    "{id}'s existence ended twice"
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
}

/// What an object leaving a zone takes with it, read at the last moment
/// before it moves ([`GameState::depart`]) and announced after: its frame,
/// if the move carries one, and each attachment the move ends, as
/// (attachment, former host, route). The attachments are read off the board
/// then, not off the frame, which was taken as the batch began: an earlier
/// member of the same batch may have ended one already (a host destroyed
/// beside its Aura), and the frame would still show it.
pub(crate) struct Departure {
    pub(crate) frame: Option<Lki>,
    unattached: Vec<(ObjectRef, ObjectRef, UnattachRoute)>,
}

impl GameState {
    /// How `existence` last existed, once it has ended (CR 113.7a, 608.2h):
    /// `None` while it has not, and where nothing framed its move, as for CR
    /// 601.2a's silent move to the stack.
    pub fn last_known(&self, existence: ObjectRef) -> Option<&Lki> {
        self.last_known.of(existence)
    }

    /// What `id` leaving `from` for `to` (`None`: the game) takes with it,
    /// before it moves: its frame, handed to every keeper that reads it (CR
    /// 603.10a, 113.7a), and what the move unattaches (CR 701.3d). The one
    /// before-move half of both performers that take an object out of a zone,
    /// whose record and unattaches announce it after.
    pub(crate) fn depart(&mut self, id: ObjectId, from: Zone, to: Option<Zone>) -> Departure {
        let captured = self.object_ref(id).and_then(|object| self.frame_taken_for(object).cloned());
        let frame = self.hand_over_before_leaving(id, from, to);
        if let Some(lki) = frame.clone().or(captured) {
            self.last_known.write(lki);
        }
        Departure { frame, unattached: self.unattached_by_leaving(id) }
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
    /// performer that moved it, under the move's cause, each carrying the
    /// leaving object's frame.
    pub(crate) fn announce_unattached_by_leaving(&mut self, departure: Departure, cause: EventCause) -> Result<(), String> {
        let Departure { frame: leaving_frame, unattached } = departure;
        for (attachment, former_host, route) in unattached {
            // The other end, if this event takes it too, as its batch framed it.
            let (attachment_lki, former_host_lki) = match route {
                UnattachRoute::AttachmentLeft => (leaving_frame.clone(), self.frame_taken_for(former_host).cloned()),
                _ => (self.frame_taken_for(attachment).cloned(), leaving_frame.clone()),
            };
            self.announce_unattached(attachment, former_host, route, attachment_lki, former_host_lki, cause)?;
        }
        Ok(())
    }
}
