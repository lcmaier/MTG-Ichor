//! The clone timer: floor 2's time (`engineering-practices.md#the-floors`),
//! read at the readiness pass by `clone_bound_test`'s and
//! `clone_fixture_test`'s ignored tests. Included by `#[path]`, as
//! `counting_allocator.rs` is.

use std::time::Instant;

/// The median of five means of `n` clone-and-drops of `value`, in
/// microseconds. A single clone is noise on a machine doing anything else; a
/// mean smooths it, and the median drops a round the machine interrupted.
pub fn clone_micros<T: Clone>(value: &T, n: usize) -> f64 {
    let mut means: Vec<f64> = (0..5)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..n {
                drop(std::hint::black_box(value.clone()));
            }
            started.elapsed().as_secs_f64() * 1e6 / n as f64
        })
        .collect();
    means.sort_by(f64::total_cmp);
    means[2]
}
