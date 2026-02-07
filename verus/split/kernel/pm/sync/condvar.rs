// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Condvar Implementation
//!
//! A condition variable for thread synchronization, allowing threads to wait
//! for a condition to be signaled by other threads.
//!
//! ## Verified Properties
//!
//! - A new condvar has an empty sleeping queue and is well-formed.
//! - `enqueue` adds exactly one entry to the back of the queue (FIFO).
//! - `dequeue_first` removes the front entry (FIFO) and decreases length by 1.
//! - `remove_at` removes the entry at a given index and preserves remaining order.
//! - `clear` empties the queue and returns the previous length.
//! - `is_empty` and `get_len` are faithful observers of the queue state.
//! - Well-formedness (`wf()`) is preserved by all operations.
//! - FIFO ordering: enqueue A then B, dequeue returns A first.
//! - Enqueue-then-dequeue round-trip on empty queue restores empty state.
//!
//! ## Verification Model
//!
//! The original implementation uses `Arc<CondvarInner>` with
//! `RefCell<LinkedList<(ProcessIdentifier, ThreadIdentifier)>>` for interior
//! mutability and shared ownership. For verification, we model the sleeping
//! queue as a ghost `Seq<(int, int)>` field and a concrete `len: usize`
//! counter, using `&mut self` for state transitions. This is a sequential
//! model that verifies the queue protocol (state machine correctness) without
//! reasoning about interior mutability or shared ownership.
//!
//! **This verified code is a specification model, not a runtime replacement.**
//! The kernel uses the original `src/kernel/src/pm/sync/condvar.rs` (with
//! `Arc`, `RefCell`, and `LinkedList`) at runtime. The verified model proves
//! the queue management protocol is correct: every reachable state satisfies
//! `wf()`, FIFO ordering is maintained, and queue transitions are sound.
//!
//! ## Verification Scope
//!
//! This verification proves **sequential queue state machine correctness** of
//! the condvar protocol. The following are explicitly **out of scope**:
//! - **Concurrency and interior mutability**: The sequential `&mut self` model
//!   does not capture `RefCell` borrow checking or `Arc` reference counting.
//! - **ProcessManager interaction**: `ProcessManager::wakeup()` and
//!   `ProcessManager::sleep()` are external dependencies not modeled here.
//!   The verification proves queue management correctness independent of
//!   whether wakeup/sleep succeed or fail.
//! - **Error handling**: The original returns `Result<_, Error>` from notify
//!   operations. The verified model returns simpler types, focusing on queue
//!   state transitions rather than error propagation.
//! - **Alarm/timer handling**: The `wait()` alarm parameter and clock checks
//!   are ProcessManager concerns, not queue management.
//! - **Drop safety**: The original `CondvarInner::drop` panics if the queue
//!   is non-empty. This is a design discipline requirement documented as a
//!   trust assumption (T2), not verified structurally.
//!
//! ## API Mapping
//!
//! | Original API            | Verified Model       | Notes                         |
//! |-------------------------|----------------------|-------------------------------|
//! | `Condvar::new()`        | `new()`              | Direct correspondence.        |
//! | `wait(alarm)`           | `enqueue(pid, tid)`  | Models queue insertion only.   |
//! | `notify_first()`        | `dequeue_first()`    | Models queue removal only.     |
//! | `notify_process(pid)`   | `remove_at(idx)`     | Caller provides ghost index.   |
//! | `notify_thread(tid)`    | `remove_at(idx)`     | Caller provides ghost index.   |
//! | `notify_all()`          | `clear()`            | Models complete queue drain.   |
//! | `reference_count()`     | (not modeled)        | Arc-specific, out of scope.    |
//!
//! ## API Divergence
//!
//! The original `notify_process` and `notify_thread` search the queue by pid
//! or tid using `LinkedList::iter().position()`. In the verified model, the
//! search result is provided as a ghost index parameter to `remove_at()`.
//! This decouples the search (which depends on the LinkedList implementation)
//! from the removal (which is the core protocol operation). The spec functions
//! `spec_contains_pid` and `spec_contains_tid` allow callers to reason about
//! whether a matching entry exists.
//!
//! ## Trust Boundaries
//!
//! - `ProcessManager::wakeup(tid)`: External dependency. The original calls
//!   this after removing an entry from the queue. The verified model does not
//!   model the wakeup side effect.
//! - `ProcessManager::sleep(alarm)`: External dependency. The original calls
//!   this after adding an entry to the queue. The verified model does not
//!   model the blocking behavior.
//! - `ProcessManager::get()`: External dependency for obtaining current
//!   pid/tid. Not modeled; the verified `enqueue` takes pid/tid as parameters.
//!
//! ## Trust Assumptions
//!
//! - **T1: Queue element uniqueness.** The original code assumes each thread
//!   appears at most once in the sleeping queue (a thread cannot wait twice).
//!   The verified model does not enforce this but documents it as a protocol
//!   requirement.
//! - **T2: Drop discipline.** The queue must be empty when the condvar is
//!   dropped. The original enforces this via a panic in `Drop::drop`. The
//!   verified model documents this requirement but cannot enforce it
//!   structurally (Verus does not model `Drop`).

use vstd::prelude::*;

// Include specifications.
include!("condvar.spec.rs");

// Include proofs.
include!("condvar.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A condition variable for thread synchronization.
///
/// # Description
///
/// Manages a FIFO queue of sleeping threads, each identified by a (pid, tid)
/// pair. In the original implementation, this uses `Arc<CondvarInner>` with
/// `RefCell<LinkedList<...>>`. Here we use a ghost `Seq` and a concrete length
/// counter for verification.
///
/// # Representation
///
/// The fields are `pub` as required by Verus for `pub open spec fn` access.
/// The `sleeping` field is ghost (erased at runtime) and tracks the abstract
/// queue state. The `len` field is the concrete queue length.
pub struct Condvar {
    /// Concrete queue length.
    pub len: usize,
    /// Ghost FIFO queue of (pid_value, tid_value) pairs.
    pub sleeping: Ghost<Seq<(int, int)>>,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Condvar {
    /// Creates a new condition variable with an empty sleeping queue.
    ///
    /// # Returns
    ///
    /// A new `Condvar` with no sleeping threads.
    pub fn new() -> (result: Self)
        ensures
            result.len == 0,
            result.spec_is_empty(),
            result@ == Condvar::spec_new_view(),
            result.wf(),
    {
        Condvar { len: 0, sleeping: Ghost(Seq::empty()) }
    }

    /// Adds a thread to the back of the sleeping queue.
    ///
    /// # Description
    ///
    /// Models the queue insertion part of the original `wait()`. The original
    /// also calls `ProcessManager::sleep()` which is not modeled here.
    ///
    /// # Parameters
    ///
    /// - `pid_val`: Process identifier value (i32 raw value).
    /// - `tid_val`: Thread identifier value (i32 raw value).
    pub fn enqueue(&mut self, pid_val: i32, tid_val: i32)
        requires
            old(self).wf(),
            old(self).len < usize::MAX,
        ensures
            self.len as nat == old(self).len as nat + 1,
            self@.sleeping =~= old(self)@.sleeping.push((pid_val as int, tid_val as int)),
            !self.spec_is_empty(),
            self.wf(),
    {
        self.len = self.len + 1;
        self.sleeping = Ghost(self.sleeping@.push((pid_val as int, tid_val as int)));
    }

    /// Removes the front thread from the sleeping queue.
    ///
    /// # Description
    ///
    /// Models the queue removal part of the original `notify_first()`. The
    /// original also calls `ProcessManager::wakeup()` which is not modeled.
    ///
    /// # Returns
    ///
    /// `true` if a thread was dequeued (queue was non-empty), `false` otherwise.
    pub fn dequeue_first(&mut self) -> (dequeued: bool)
        requires
            old(self).wf(),
        ensures
            dequeued == !old(self).spec_is_empty(),
            dequeued ==> self.len as nat == old(self).len as nat - 1,
            dequeued ==> self@.sleeping =~= old(self)@.sleeping.subrange(
                1,
                old(self)@.sleeping.len() as int,
            ),
            !dequeued ==> self@ == old(self)@,
            self.wf(),
    {
        if self.len > 0 {
            let ghost old_sleeping: Seq<(int, int)> = self.sleeping@;
            self.len = self.len - 1;
            self.sleeping = Ghost(old_sleeping.subrange(1, old_sleeping.len() as int));
            true
        } else {
            false
        }
    }

    /// Removes the thread at the given ghost index from the sleeping queue.
    ///
    /// # Description
    ///
    /// Models the find-and-remove operation used by the original
    /// `notify_process()` and `notify_thread()`. The ghost index represents
    /// the result of `LinkedList::iter().position()` in the original.
    ///
    /// # Parameters
    ///
    /// - `idx`: Ghost index of the entry to remove. Must be a valid index
    ///   into the sleeping queue.
    ///
    /// # Returns
    ///
    /// Always returns `true` (removal always succeeds when preconditions hold).
    pub fn remove_at(&mut self, Ghost(idx): Ghost<int>) -> (removed: bool)
        requires
            old(self).wf(),
            0 <= idx < old(self).len as int,
        ensures
            removed,
            self.len as nat == old(self).len as nat - 1,
            self@.sleeping =~= Condvar::spec_remove_at_seq(old(self)@.sleeping, idx),
            self.wf(),
    {
        let ghost old_sleeping: Seq<(int, int)> = self.sleeping@;
        self.len = self.len - 1;
        self.sleeping = Ghost(
            old_sleeping.subrange(0, idx) + old_sleeping.subrange(
                idx + 1,
                old_sleeping.len() as int,
            ),
        );
        true
    }

    /// Removes all threads from the sleeping queue.
    ///
    /// # Description
    ///
    /// Models the original `notify_all()` which drains the entire queue. The
    /// original also calls `ProcessManager::wakeup()` for each entry, which
    /// is not modeled here.
    ///
    /// # Returns
    ///
    /// The number of threads that were in the queue before clearing.
    pub fn clear(&mut self) -> (count: usize)
        requires
            old(self).wf(),
        ensures
            count == old(self).len,
            self.len == 0,
            self.spec_is_empty(),
            self@ == Condvar::spec_new_view(),
            self.wf(),
    {
        let old_len: usize = self.len;
        self.len = 0;
        self.sleeping = Ghost(Seq::empty());
        old_len
    }

    /// Checks if the sleeping queue is empty.
    ///
    /// # Returns
    ///
    /// `true` if no threads are sleeping, `false` otherwise.
    pub fn is_empty(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self.spec_is_empty(),
            result == (self.len == 0),
    {
        self.len == 0
    }

    /// Returns the number of sleeping threads.
    ///
    /// # Returns
    ///
    /// The current queue length.
    pub fn get_len(&self) -> (result: usize)
        ensures
            result == self.len,
    {
        self.len
    }
}

} // verus!
