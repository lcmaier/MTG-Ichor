//! The stores a fork shares (`clone-architecture.md`, CL-1): a clone of the
//! state copies a pointer per chunk of entries, and a fork copies a chunk, and
//! an entry in it, only when it first writes one.
//!
//! Two shapes, each in chunks of 32 behind `Arc`s, the history log's idea
//! (`state::history_log`):
//! - [`ObjectTable`], keyed by object: an object's id is a per-game counter,
//!   so `ObjectId(n)` lives in chunk n/32 at slot n%32, and a lookup hashes
//!   nothing;
//! - [`SharedList`], in order: the stack's objects, the triggers waiting.
//!
//! **A write goes through `get_mut`, and only there** (decision 3): it copies
//! the chunk if a fork shares it, then the entry if the entry is a
//! [`SharedEntry`]. Neither type offers a mutable walk, since a walk that
//! takes `&mut` of every entry copies every entry in a fork whether or not it
//! writes one.
//!
//! **Sharing is never observed** (decision 7): nothing outside this file asks
//! whether a chunk is shared, where it lives or how many hold it, so a game
//! plays the same whether or not a search cloned it.

use std::fmt;
use std::ops::Index;
use std::sync::Arc;

use crate::types::ids::ObjectId;

/// How many ids a table's chunk spans, and how many items a list's chunk holds.
const CHUNK: usize = 32;

/// How a store holds one entry. A chunk is copied whole on a fork's first
/// write to it, so a small entry with no heap of its own sits inline, and a
/// large one sits behind its own `Arc`, where a chunk's copy is a count.
pub trait StoreEntry: Clone {
    type Value;
    fn hold(value: Self::Value) -> Self;
    fn value(&self) -> &Self::Value;
    /// The value to write: a [`SharedEntry`] copies it first if a fork
    /// shares it.
    fn value_mut(&mut self) -> &mut Self::Value;
    fn release(self) -> Self::Value;
}

/// An entry held in its chunk: an object, a memo frame, a stack position.
#[derive(Clone, PartialEq)]
pub struct InlineEntry<T>(T);

impl<T: Clone> StoreEntry for InlineEntry<T> {
    type Value = T;
    fn hold(value: T) -> Self {
        InlineEntry(value)
    }
    fn value(&self) -> &T {
        &self.0
    }
    fn value_mut(&mut self) -> &mut T {
        &mut self.0
    }
    fn release(self) -> T {
        self.0
    }
}

/// An entry behind its own `Arc`: a permanent, a stack entry, a waiting
/// trigger, each a few hundred bytes with heap of its own. A chunk of 32 of
/// them inline would cost a fork that writes one all 32.
#[derive(Clone, PartialEq)]
pub struct SharedEntry<T>(Arc<T>);

impl<T: Clone> StoreEntry for SharedEntry<T> {
    type Value = T;
    fn hold(value: T) -> Self {
        SharedEntry(Arc::new(value))
    }
    fn value(&self) -> &T {
        &self.0
    }
    fn value_mut(&mut self) -> &mut T {
        Arc::make_mut(&mut self.0)
    }
    fn release(self) -> T {
        Arc::unwrap_or_clone(self.0)
    }
}

/// One table chunk: the entries of 32 consecutive ids, and which are there.
#[derive(Clone)]
struct TableChunk<E> {
    slots: [Option<E>; CHUNK],
    /// Bit `i` set when `slots[i]` holds an entry: a walk of a sparse table,
    /// the battlefield's among every object's ids, visits only those.
    live: u32,
}

/// Entries keyed by object, in chunks a fork shares (the module's doc). Walked
/// in id order, which is the same in every process but is not the rules'
/// order: a choice, a log or a count still sorts by timestamp (`CLAUDE.md`,
/// "Determinism at the decision boundary").
pub struct ObjectTable<E> {
    chunks: Vec<Option<Arc<TableChunk<E>>>>,
    len: usize,
}

impl<E> Clone for ObjectTable<E> {
    fn clone(&self) -> Self {
        ObjectTable { chunks: self.chunks.clone(), len: self.len }
    }
}

impl<E> Default for ObjectTable<E> {
    fn default() -> Self {
        ObjectTable { chunks: Vec::new(), len: 0 }
    }
}

/// Where `id` sits: its chunk and its slot.
fn place(id: ObjectId) -> (usize, usize) {
    let raw = id.raw() as usize;
    (raw / CHUNK, raw % CHUNK)
}

impl<E: StoreEntry> ObjectTable<E> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, id: &ObjectId) -> Option<&E::Value> {
        let (chunk, slot) = place(*id);
        self.chunks.get(chunk)?.as_ref()?.slots[slot].as_ref().map(E::value)
    }

    pub fn contains_key(&self, id: &ObjectId) -> bool {
        self.get(id).is_some()
    }

    /// The entry to write, its chunk and the entry copied first if a fork
    /// shares them. A miss copies nothing.
    pub fn get_mut(&mut self, id: &ObjectId) -> Option<&mut E::Value> {
        let (chunk, slot) = place(*id);
        let chunk = self.chunks.get_mut(chunk)?.as_mut()?;
        chunk.slots[slot].as_ref()?;
        Arc::make_mut(chunk).slots[slot].as_mut().map(E::value_mut)
    }

    /// File `value` under `id`, returning what was there.
    pub fn insert(&mut self, id: ObjectId, value: E::Value) -> Option<E::Value> {
        debug_assert!(id.raw() < 1 << 40, "{id} is no id a store minted: an object table is indexed by the counter");
        let (chunk, slot) = place(id);
        if self.chunks.len() <= chunk {
            self.chunks.resize_with(chunk + 1, || None);
        }
        let chunk = Arc::make_mut(
            self.chunks[chunk].get_or_insert_with(|| Arc::new(TableChunk { slots: std::array::from_fn(|_| None), live: 0 })),
        );
        let old = chunk.slots[slot].replace(E::hold(value));
        if old.is_none() {
            chunk.live |= 1 << slot;
            self.len += 1;
        }
        old.map(E::release)
    }

    /// Take `id`'s entry out. A chunk left empty is dropped, and the chunks
    /// past the last live one with it.
    pub fn remove(&mut self, id: &ObjectId) -> Option<E::Value> {
        let (index, slot) = place(*id);
        let chunk = self.chunks.get_mut(index)?.as_mut()?;
        chunk.slots[slot].as_ref()?;
        let chunk = Arc::make_mut(chunk);
        let taken = chunk.slots[slot].take();
        chunk.live &= !(1 << slot);
        self.len -= 1;
        if chunk.live == 0 {
            self.chunks[index] = None;
            while matches!(self.chunks.last(), Some(None)) {
                self.chunks.pop();
            }
        }
        taken.map(E::release)
    }

    /// Every entry, in id order.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (ObjectId, &E::Value)> + '_ {
        self.chunks.iter().enumerate().filter_map(|(index, chunk)| Some((index, chunk.as_ref()?))).flat_map(|(index, chunk)| {
            LiveSlots(chunk.live).map(move |slot| {
                let entry = chunk.slots[slot].as_ref().map(E::value);
                (ObjectId::from_table_slot((index * CHUNK + slot) as u64), entry)
            })
        })
        .filter_map(|(id, entry)| Some((id, entry?)))
    }

    pub fn keys(&self) -> impl DoubleEndedIterator<Item = ObjectId> + '_ {
        self.iter().map(|(id, _)| id)
    }

    pub fn values(&self) -> impl DoubleEndedIterator<Item = &E::Value> + '_ {
        self.iter().map(|(_, value)| value)
    }
}

/// For the fork-copies test alone: no engine code may ask whether a chunk is
/// shared (`clone-architecture.md` decision 7).
#[cfg(any(test, feature = "test-support"))]
impl<E> ObjectTable<E> {
    /// The chunks this table holds that `other` does not hold, the same chunk
    /// and not a copy: what a fork that began as `other` has copied or made
    /// since.
    pub fn chunks_not_shared_with(&self, other: &Self) -> usize {
        let shared = |index: usize, chunk: &Arc<TableChunk<E>>| {
            other.chunks.get(index).and_then(Option::as_ref).is_some_and(|theirs| Arc::ptr_eq(chunk, theirs))
        };
        self.chunks.iter().enumerate().filter(|(index, chunk)| chunk.as_ref().is_some_and(|chunk| !shared(*index, chunk))).count()
    }
}

/// The set bits of a chunk's `live` mask, lowest first.
struct LiveSlots(u32);

impl Iterator for LiveSlots {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.0 == 0 {
            return None;
        }
        let slot = self.0.trailing_zeros() as usize;
        self.0 &= self.0 - 1;
        Some(slot)
    }
}

impl DoubleEndedIterator for LiveSlots {
    fn next_back(&mut self) -> Option<usize> {
        if self.0 == 0 {
            return None;
        }
        let slot = 31 - self.0.leading_zeros() as usize;
        self.0 &= !(1 << slot);
        Some(slot)
    }
}

impl<E: StoreEntry> Index<&ObjectId> for ObjectTable<E> {
    type Output = E::Value;
    /// `HashMap`'s contract: a missing id is the caller's error.
    fn index(&self, id: &ObjectId) -> &E::Value {
        match self.get(id) {
            Some(value) => value,
            None => panic!("{id} is not in the table"),
        }
    }
}

impl<E: StoreEntry> fmt::Debug for ObjectTable<E>
where
    E::Value: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

/// A list chunk, which a fork's first write copies with room for the rest of
/// the chunk, so its later pushes do not reallocate.
struct ListChunk<E>(Vec<E>);

impl<E: Clone> Clone for ListChunk<E> {
    fn clone(&self) -> Self {
        let mut items = Vec::with_capacity(CHUNK);
        items.extend(self.0.iter().cloned());
        ListChunk(items)
    }
}

/// Items in order, in chunks a fork shares: the stack's objects, bottom to
/// top, and the triggers waiting to be put on it. Every chunk but the last
/// is full and none is empty, so a push or a pop at the end copies at most
/// the last chunk in a fork.
pub struct SharedList<E> {
    chunks: Vec<Arc<ListChunk<E>>>,
    len: usize,
}

impl<E> Clone for SharedList<E> {
    fn clone(&self) -> Self {
        SharedList { chunks: self.chunks.clone(), len: self.len }
    }
}

impl<E> Default for SharedList<E> {
    fn default() -> Self {
        SharedList { chunks: Vec::new(), len: 0 }
    }
}

impl<E: StoreEntry> SharedList<E> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, value: E::Value) {
        match self.chunks.last_mut() {
            Some(last) if last.0.len() < CHUNK => Arc::make_mut(last).0.push(E::hold(value)),
            _ => {
                let mut items = Vec::with_capacity(CHUNK);
                items.push(E::hold(value));
                self.chunks.push(Arc::new(ListChunk(items)));
            }
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<E::Value> {
        let last = self.chunks.last_mut()?;
        let popped = Arc::make_mut(last).0.pop();
        if last.0.is_empty() {
            self.chunks.pop();
        }
        self.len -= 1;
        popped.map(E::release)
    }

    pub fn get(&self, index: usize) -> Option<&E::Value> {
        self.chunks.get(index / CHUNK)?.0.get(index % CHUNK).map(E::value)
    }

    /// The item to write, its chunk and the item copied first if a fork
    /// shares them.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut E::Value> {
        let chunk = self.chunks.get_mut(index / CHUNK)?;
        if index % CHUNK >= chunk.0.len() {
            return None;
        }
        Arc::make_mut(chunk).0.get_mut(index % CHUNK).map(E::value_mut)
    }

    pub fn first(&self) -> Option<&E::Value> {
        self.get(0)
    }

    pub fn last(&self) -> Option<&E::Value> {
        self.chunks.last()?.0.last().map(E::value)
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &E::Value> + '_ {
        self.chunks.iter().flat_map(|chunk| chunk.0.iter().map(E::value))
    }

    pub fn contains(&self, value: &E::Value) -> bool
    where
        E::Value: PartialEq,
    {
        self.iter().any(|item| item == value)
    }

    /// Take out the item at `index`, closing the gap: the items after it are
    /// taken and put back, so the cost is the tail's. `None` past the end.
    pub fn remove(&mut self, index: usize) -> Option<E::Value> {
        if index >= self.len {
            return None;
        }
        let mut tail = Vec::with_capacity(self.len - index - 1);
        while self.len > index + 1 {
            tail.extend(self.pop());
        }
        let removed = self.pop();
        while let Some(value) = tail.pop() {
            self.push(value);
        }
        removed
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.len = 0;
    }
}

/// For the fork-copies test alone, as [`ObjectTable::chunks_not_shared_with`].
#[cfg(any(test, feature = "test-support"))]
impl<E> SharedList<E> {
    /// The chunks this list holds that `other` does not hold, the same chunk
    /// at the same place.
    pub fn chunks_not_shared_with(&self, other: &Self) -> usize {
        self.chunks.iter().enumerate().filter(|(index, chunk)| !other.chunks.get(*index).is_some_and(|theirs| Arc::ptr_eq(chunk, theirs))).count()
    }
}

impl<E: StoreEntry> Extend<E::Value> for SharedList<E> {
    fn extend<I: IntoIterator<Item = E::Value>>(&mut self, items: I) {
        for item in items {
            self.push(item);
        }
    }
}

impl<E: StoreEntry> FromIterator<E::Value> for SharedList<E> {
    fn from_iter<I: IntoIterator<Item = E::Value>>(items: I) -> Self {
        let mut list = SharedList::default();
        list.extend(items);
        list
    }
}

impl<E: StoreEntry> IntoIterator for SharedList<E> {
    type Item = E::Value;
    type IntoIter = std::vec::IntoIter<E::Value>;
    /// Each item by value: a fork that shares a chunk copies its items out.
    fn into_iter(self) -> Self::IntoIter {
        let mut items = Vec::with_capacity(self.len);
        for chunk in self.chunks {
            items.extend(Arc::unwrap_or_clone(chunk).0.into_iter().map(E::release));
        }
        items.into_iter()
    }
}

impl<'a, E: StoreEntry> IntoIterator for &'a SharedList<E> {
    type Item = &'a E::Value;
    type IntoIter = Box<dyn DoubleEndedIterator<Item = &'a E::Value> + 'a>;
    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

impl<E: StoreEntry> Index<usize> for SharedList<E> {
    type Output = E::Value;
    /// `Vec`'s contract: an index past the end is the caller's error.
    fn index(&self, index: usize) -> &E::Value {
        match self.get(index) {
            Some(value) => value,
            None => panic!("index {index} past a list of {}", self.len),
        }
    }
}

impl<E: StoreEntry> PartialEq for SharedList<E>
where
    E::Value: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().eq(other.iter())
    }
}

impl<E: StoreEntry> PartialEq<Vec<E::Value>> for SharedList<E>
where
    E::Value: PartialEq,
{
    fn eq(&self, other: &Vec<E::Value>) -> bool {
        self.len == other.len() && self.iter().eq(other.iter())
    }
}

impl<E: StoreEntry> fmt::Debug for SharedList<E>
where
    E::Value: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(raw: u64) -> ObjectId {
        ObjectId::from_table_slot(raw)
    }

    fn table(ids: impl IntoIterator<Item = u64>) -> ObjectTable<SharedEntry<u64>> {
        let mut table = ObjectTable::default();
        for raw in ids {
            table.insert(id(raw), raw);
        }
        table
    }

    /// Whether two tables hold the same chunk at `index`, not a copy.
    fn same_chunk<E>(a: &ObjectTable<E>, b: &ObjectTable<E>, index: usize) -> bool {
        match (&a.chunks[index], &b.chunks[index]) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }

    #[test]
    fn a_table_files_reads_and_removes_by_id_and_walks_in_id_order() {
        let mut table = table([100, 1, 33]);
        assert_eq!(table.len(), 3);
        assert_eq!(table.get(&id(33)), Some(&33));
        assert_eq!(table[&id(100)], 100);
        assert!(!table.contains_key(&id(2)));
        assert_eq!(table.keys().map(|id| id.raw()).collect::<Vec<_>>(), [1, 33, 100]);
        assert_eq!(table.remove(&id(33)), Some(33));
        assert_eq!(table.remove(&id(33)), None);
        assert_eq!(table.values().copied().collect::<Vec<_>>(), [1, 100]);
    }

    #[test]
    fn an_emptied_chunk_is_dropped_and_the_table_ends_at_its_last_live_one() {
        let mut table = table([1, 200]);
        assert_eq!(table.chunks.len(), 7);
        table.remove(&id(200));
        assert_eq!(table.chunks.len(), 1, "the trailing chunks go with the emptied one");
        table.remove(&id(1));
        assert!(table.is_empty() && table.chunks.is_empty());
    }

    /// A clone shares every chunk; a write copies the one chunk it lands in
    /// and the entry, and the other side reads what it had.
    #[test]
    fn a_fork_copies_only_the_chunk_and_the_entry_it_writes() {
        let mut parent = table(1..=64);
        let mut fork = parent.clone();
        assert!((0..3).all(|index| same_chunk(&parent, &fork, index)));
        *fork.get_mut(&id(3)).unwrap() = 99;
        assert!(!same_chunk(&parent, &fork, 0), "the written chunk is the fork's own");
        assert!(same_chunk(&parent, &fork, 1) && same_chunk(&parent, &fork, 2));
        assert_eq!((parent[&id(3)], fork[&id(3)]), (3, 99));
        // Within the copied chunk, the entries not written are still shared.
        let (Some(a), Some(b)) = (&parent.chunks[0], &fork.chunks[0]) else { unreachable!() };
        let entry = |chunk: &TableChunk<SharedEntry<u64>>, slot: usize| chunk.slots[slot].clone().map(|entry| entry.0);
        assert!(Arc::ptr_eq(&entry(a, 4).unwrap(), &entry(b, 4).unwrap()));
        assert!(!Arc::ptr_eq(&entry(a, 3).unwrap(), &entry(b, 3).unwrap()));
        // The parent writes its own copy and leaves the fork as it was.
        *parent.get_mut(&id(40)).unwrap() = 7;
        assert_eq!((parent[&id(40)], fork[&id(40)]), (7, 40));
    }

    #[test]
    fn a_miss_copies_nothing() {
        let mut parent = table([1, 2]);
        let fork = parent.clone();
        assert!(parent.get_mut(&id(5)).is_none());
        assert!(same_chunk(&parent, &fork, 0), "an absent id in a shared chunk is no write");
    }

    #[test]
    fn a_list_keeps_its_order_through_pushes_pops_and_removals() {
        let mut list: SharedList<InlineEntry<u32>> = (0..70).collect();
        let mut model: Vec<u32> = (0..70).collect();
        assert_eq!(list, model);
        assert_eq!((list.first(), list.last(), list.get(40)), (Some(&0), Some(&69), Some(&40)));
        assert_eq!(list.remove(5), Some(model.remove(5)));
        assert_eq!(list.remove(64), Some(model.remove(64)));
        assert_eq!(list.remove(500), None);
        assert_eq!(list.pop(), model.pop());
        list.push(1000);
        model.push(1000);
        assert_eq!(list, model);
        assert_eq!(list.iter().rev().copied().collect::<Vec<_>>(), model.iter().rev().copied().collect::<Vec<_>>());
        assert!(list.chunks.iter().rev().skip(1).all(|chunk| chunk.0.len() == CHUNK), "every chunk but the last is full");
        assert_eq!(list.into_iter().collect::<Vec<_>>(), model);
    }

    /// A push or a pop at the end copies the last chunk and no other.
    #[test]
    fn a_fork_of_a_list_copies_only_the_chunk_it_writes() {
        let parent: SharedList<SharedEntry<u32>> = (0..64).collect();
        let mut fork = parent.clone();
        fork.push(64);
        assert!(parent.chunks.iter().zip(&fork.chunks).all(|(a, b)| Arc::ptr_eq(a, b)), "a push into a new chunk");
        fork.pop();
        fork.pop();
        assert!(Arc::ptr_eq(&parent.chunks[0], &fork.chunks[0]));
        assert!(!Arc::ptr_eq(&parent.chunks[1], &fork.chunks[1]));
        assert_eq!((parent.len(), fork.len()), (64, 63));
        *fork.get_mut(0).unwrap() = 99;
        assert_eq!((parent[0], fork[0]), (0, 99));
    }
}
