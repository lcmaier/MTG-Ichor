//! A clone's allocations and bytes, counted exactly: the allocator
//! `clone_bound_test.rs` and `zone_reach_cost_test.rs` both read floors 2 and 3
//! with. Included by `#[path]`, because a `#[global_allocator]` belongs to one
//! binary and each of those tests is its own.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};

/// `System`, counting the allocations a thread asks for while it has
/// [`COUNTING`] on. Requested sizes, so the count is the program's and not an
/// allocator's rounding, and the same under any allocator.
pub struct CountingAllocator;

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
}
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);

fn count(size: usize) {
    // `try_with`: a thread being torn down has no flag left to read.
    if COUNTING.try_with(Cell::get).unwrap_or(false) {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(size as u64, Ordering::Relaxed);
    }
}

// SAFETY: every call is forwarded to `System` unchanged; the counting beside it
// touches only atomics and a const-initialized thread-local, and allocates
// nothing.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count(layout.size());
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        count(new_size);
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// The allocations and bytes of one clone of `value`, held until counted.
pub fn clone_cost<T: Clone>(value: &T) -> (u64, u64) {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    BYTES.store(0, Ordering::Relaxed);
    COUNTING.with(|on| on.set(true));
    let held = value.clone();
    COUNTING.with(|on| on.set(false));
    let cost = (ALLOCATIONS.load(Ordering::Relaxed), BYTES.load(Ordering::Relaxed));
    drop(held);
    cost
}
