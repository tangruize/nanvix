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
//! - `enqueue` requires the entry is not already present (uniqueness).
//! - `enqueue` requires the pid is not the kernel process (safety invariant).
//! - `try_enqueue` models `wait(alarm)` with alarm-expired guard: if expired,
//!   the queue is unchanged; otherwise, enqueue proceeds.
//! - `dequeue_first` removes the front entry (FIFO) and decreases length by 1.
//! - `remove_at` removes the entry at a given index and preserves remaining order.
//! - `remove_entry` removes a specific (pid, tid) entry by index, models
//!   `wait()` failure cleanup. Under uniqueness, the entry is absent afterward.
//! - `remove_by_pid` removes the first entry matching a pid (first-match).
//! - `remove_by_tid` removes the first entry matching a tid (first-match).
//! - `try_remove_by_pid` models `notify_process` with both found/not-found cases.
//! - `try_remove_by_tid` models `notify_thread` with both found/not-found cases.
//! - `clear` empties the queue and returns the previous length.
//! - `is_empty` and `get_len` are faithful observers of the queue state.
//! - Well-formedness (`wf()`) is preserved by all operations and includes
//!   length consistency, queue element uniqueness (`spec_all_unique()`), and
//!   the kernel process exclusion invariant (`spec_no_kernel_pid()`).
//! - FIFO ordering: enqueue A then B, dequeue returns A first.
//! - Enqueue-then-dequeue round-trip on empty queue restores empty state.
//! - **Wait protocol**: `lemma_wait_cleanup_restores_state` proves that enqueue
//!   followed by remove_entry (modeling `wait()`'s `retain()` cleanup on failure)
//!   restores the original queue state. `lemma_wait_protocol_preserves_wf` proves
//!   the cleanup path preserves well-formedness.
//! - **Drop safety**: `spec_drop_safe()` formalizes that the queue must be empty
//!   for safe drop. Proof lemmas establish `new()` and `clear()` produce
//!   drop-safe condvars.
//!
//! ## Verification Model
//!
//! The original implementation uses `Arc<CondvarInner>` with
//! `RefCell<LinkedList<(ProcessIdentifier, ThreadIdentifier)>>` for interior
//! mutability and shared ownership. For verification, we model the sleeping
//! queue as a concrete `Vec<(i32, i32)>` field and a concrete `len: usize`
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
//!   trust assumption (T2). The `spec_drop_safe()` predicate formalizes the
//!   precondition; proof lemmas establish that `new()` and `clear()` produce
//!   drop-safe condvars.
//!
//! ## API Mapping
//!
//! | Original API            | Verified Model         | Notes                         |
//! |-------------------------|------------------------|-------------------------------|
//! | `Condvar::new()`        | `new()`                | Direct correspondence.        |
//! | `wait(alarm)`           | `try_enqueue(pid, tid, expired)` | Models alarm guard + queue insertion. |
//! | `wait()` failure cleanup| `remove_entry()`       | Models `retain()` cleanup.    |
//! | `wait()` full protocol  | (proof lemma)          | `lemma_wait_cleanup_restores_state`. |
//! | `notify_first()`        | `dequeue_first()`      | Models queue removal only.     |
//! | `notify_process(pid)`   | `try_remove_by_pid()`  | Handles found and not-found.   |
//! | `notify_thread(tid)`    | `try_remove_by_tid()`  | Handles found and not-found.   |
//! | `notify_all()`          | `clear()`              | Returns total entries, not successful wakeup count. |
//! | `reference_count()`     | (not modeled)          | Arc-specific, out of scope.    |
//!
//! ## API Divergence
//!
//! The original `notify_process` and `notify_thread` search the queue by pid
//! or tid using `LinkedList::iter().position()`. In the verified model, the
//! search result is provided as a concrete index parameter. The wrapper
//! functions `remove_by_pid` and `remove_by_tid` connect the search predicate
//! to the index via preconditions that assert the entry at the index matches
//! the search criterion. The lower-level `remove_at` is also retained for
//! generality. The spec functions `spec_contains_pid` and `spec_contains_tid`
//! allow callers to reason about whether a matching entry exists.
//!
//! The original `notify_process(pid)` documentation says "Wakes up all threads
//! of a process" but the implementation only wakes the *first* thread found
//! with matching `pid` (uses `position()` which returns the first match, then
//! removes one entry). The verified model matches the *implementation*
//! (removes one entry), not the documentation.
//!
//! The original `notify_all()` has iterative partial-failure semantics: it
//! calls `ProcessManager::wakeup()` for each entry, continues on error, and
//! returns `Err` only if zero wakeups succeeded. The model's `clear()`
//! atomically empties the queue. This is consistent with the documented
//! verification scope: error handling and `ProcessManager` interaction are out
//! of scope. The queue state transition (drain all entries) is correctly
//! modeled. Similarly, the original `notify_first()` returns
//! `Result<u32, Error>` with wakeup errors; the model's `dequeue_first()`
//! returns `bool` capturing only whether the queue was non-empty. These are
//! refinement-safe: the model captures all queue state transitions, while
//! error behavior depends on the external `ProcessManager` dependency.
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
//!   This is a valid protocol invariant: a thread that calls `wait()` blocks
//!   in `ProcessManager::sleep()`, so it cannot call `wait()` again until it
//!   is woken up and the entry is removed. The original does not check for
//!   duplicates because the protocol makes duplicates unreachable. The verified
//!   model enforces this structurally: `wf()` includes `spec_all_unique()`, so
//!   every exec operation requires and ensures uniqueness. The `enqueue`
//!   function explicitly requires the entry is not already present, matching
//!   the protocol invariant.
//! - **T2: Drop discipline.** The queue must be empty when the condvar is
//!   dropped. The original enforces this via a panic in `Drop::drop`. The
//!   verified model formalizes this with `spec_drop_safe()` and proves that
//!   `new()` and `clear()` produce drop-safe condvars. The actual enforcement
//!   at all call sites requires whole-program reasoning beyond Verus's scope.
//! - **T3: Queue length bound.** The queue length never reaches `usize::MAX`.
//!   The verified `enqueue()` requires `len < usize::MAX` to prevent overflow.
//!   The original has no explicit check but this is practically guaranteed
//!   since the number of threads is bounded by system resources. In Nanvix,
//!   thread creation is managed by the ProcessManager with finite limits,
//!   making `usize::MAX` threads unreachable. A runtime guard is unnecessary
//!   given these system-level constraints.
//! - **T4: Sequential wait protocol.** The wait protocol lemmas
//!   (`lemma_wait_cleanup_restores_state`, `lemma_wait_protocol_preserves_wf`)
//!   prove correctness assuming no concurrent modifications between enqueue
//!   and cleanup. At runtime, the original `wait()` calls
//!   `ProcessManager::sleep()` between `push_back` and `retain`, during which
//!   other threads may call `notify_*()` and modify the queue. The sequential
//!   model cannot capture this interleaving. The cleanup `retain()` operates
//!   on the actual queue state at cleanup time, which may differ from the
//!   enqueue-time state.
//! - **T5: Search result correctness.** In `try_remove_by_pid` and
//!   `try_remove_by_tid`, the `has_match` parameter is concrete (not ghost)
//!   because the exec code branches on it (Verus cannot branch on ghost values
//!   in exec mode). The caller must supply the correct value; the preconditions
//!   constrain `has_match` to be consistent with
//!   `spec_contains_pid`/`spec_contains_tid`, so an incorrect value makes
//!   the precondition unsatisfiable. At runtime, the original computes this
//!   via `position()`. A concrete call-site wrapper performing the search
//!   would supply the correct value.
//! - **T6: Arc lifetime management.** The original `reference_count()` returns
//!   `Arc::strong_count()`. The verified model does not model reference
//!   counting. Correct lifetime management (i.e., the condvar outlives all
//!   waiters and is not dropped while threads reference it) is assumed.

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
/// `RefCell<LinkedList<...>>`. Here we use a concrete `Vec<(i32, i32)>` and
/// a concrete length counter for verification.
///
/// # Representation
///
/// The fields are `pub` as required by Verus for `pub open spec fn` access.
/// The `sleeping` field is a concrete Vec storing the FIFO queue of
/// (pid, tid) pairs. The `len` field tracks the queue length.
pub struct Condvar {
    /// Concrete queue length.
    pub len: usize,
    /// Concrete FIFO queue of (pid_value, tid_value) pairs.
    pub sleeping: Vec<(i32, i32)>,
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
            result@.spec_is_empty(),
            result@ == CondvarView::spec_new(),
            result.wf(),
    {
        Condvar { len: 0, sleeping: Vec::new() }
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
            old(self)@.spec_len() < usize::MAX,
            !old(self)@.spec_contains_entry(pid_val as int, tid_val as int),
            // The kernel process must not sleep (matches original panic guard).
            pid_val as int != CondvarView::spec_kernel_pid(),
        ensures
            self@.spec_len() == old(self)@.spec_len() + 1,
            self@.sleeping =~= old(self)@.sleeping.push((pid_val as int, tid_val as int)),
            !self@.spec_is_empty(),
            self.wf(),
    {
        proof {
            // Bridge from wf() to raw-Seq uniqueness on concrete fields.
            let entry: (i32, i32) = (pid_val, tid_val);
            let s: Seq<(i32, i32)> = self.sleeping@;
            // Uniqueness of s follows from wf() which includes concrete_all_unique().
            assert forall|i: int, j: int|
                #![trigger s[i], s[j]]
                0 <= i < s.len() as int
                && 0 <= j < s.len() as int
                && i != j
            implies s[i] != s[j] by {
                assert(self.sleeping@[i] == s[i]);
                assert(self.sleeping@[j] == s[j]);
            }
            // No existing element equals the new entry (from !spec_contains_entry).
            assert forall|i: int|
                #![trigger s[i]]
                0 <= i < s.len() as int
            implies s[i] != entry by {
                if s[i] == entry {
                    assert(s[i].0 == pid_val && s[i].1 == tid_val);
                    // Bridge: concrete equality implies abstract equality.
                    assert(self@.sleeping[i] == (s[i].0 as int, s[i].1 as int));
                    assert(self@.sleeping[i] == (pid_val as int, tid_val as int));
                }
            }
            Condvar::lemma_enqueue_preserves_unique(s, entry);
            // Prove no-kernel-pid is preserved after push.
            let new_s: Seq<(i32, i32)> = s.push(entry);
            assert forall|i: int|
                #![trigger new_s[i]]
                0 <= i < new_s.len() as int
            implies new_s[i].0 as int != CondvarView::spec_kernel_pid() by {
                if i < s.len() as int {
                    assert(new_s[i] == s[i]);
                    assert(self.sleeping@[i] == s[i]);
                } else {
                    assert(new_s[i] == entry);
                }
            }
        }
        self.len = self.len + 1;
        self.sleeping.push((pid_val, tid_val));
    }

    /// Conditionally enqueues a (pid, tid) entry, modeling `wait()` with alarm.
    ///
    /// # Description
    ///
    /// Models the original `wait(alarm)` conditional: if the alarm has already
    /// expired, the queue is not modified and `false` is returned. Otherwise,
    /// the entry is enqueued and `true` is returned. The `alarm_expired`
    /// parameter is concrete because the exec code branches on it. At runtime,
    /// the original computes this by comparing `clock::now() >= alarm`.
    ///
    /// # Parameters
    ///
    /// - `pid_val`: Process identifier value.
    /// - `tid_val`: Thread identifier value.
    /// - `alarm_expired`: Whether the alarm has already expired.
    ///
    /// # Returns
    ///
    /// `true` if the entry was enqueued, `false` if the alarm was expired.
    pub fn try_enqueue(
        &mut self,
        pid_val: i32,
        tid_val: i32,
        alarm_expired: bool,
    ) -> (enqueued: bool)
        requires
            old(self).wf(),
            old(self)@.spec_len() < usize::MAX,
            !old(self)@.spec_contains_entry(pid_val as int, tid_val as int),
            pid_val as int != CondvarView::spec_kernel_pid(),
        ensures
            enqueued == !alarm_expired,
            // If enqueued: queue grew by one.
            enqueued ==> self@.spec_len() == old(self)@.spec_len() + 1,
            enqueued ==> self@.sleeping =~= old(self)@.sleeping.push(
                (pid_val as int, tid_val as int),
            ),
            // If alarm expired: queue unchanged.
            !enqueued ==> self@ == old(self)@,
            self.wf(),
    {
        if !alarm_expired {
            self.enqueue(pid_val, tid_val);
            true
        } else {
            false
        }
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
            dequeued == !old(self)@.spec_is_empty(),
            dequeued ==> self@.spec_len() == old(self)@.spec_len() - 1,
            dequeued ==> self@.sleeping =~= old(self)@.sleeping.subrange(
                1,
                old(self)@.sleeping.len() as int,
            ),
            // When dequeued, the removed entry was the front of the queue.
            dequeued ==> old(self)@.sleeping[0] == old(self)@.spec_front(),
            !dequeued ==> self@ == old(self)@,
            self.wf(),
    {
        if self.len > 0 {
            self.len = self.len - 1;
            let _removed: (i32, i32) = self.sleeping.remove(0);
            true
        } else {
            false
        }
    }

    /// Removes the thread at the given index from the sleeping queue.
    ///
    /// # Description
    ///
    /// Models the find-and-remove operation used by the original
    /// `notify_process()` and `notify_thread()`. The index represents
    /// the result of `LinkedList::iter().position()` in the original.
    ///
    /// # Parameters
    ///
    /// - `idx`: Index of the entry to remove. Must be a valid index
    ///   into the sleeping queue.
    ///
    /// # Returns
    ///
    /// Always returns `true` (removal always succeeds when preconditions hold).
    pub fn remove_at(&mut self, idx: usize) -> (removed: bool)
        requires
            old(self).wf(),
            (idx as int) < old(self)@.sleeping.len() as int,
        ensures
            removed,
            self@.spec_len() == old(self)@.spec_len() - 1,
            self@.sleeping =~= CondvarView::spec_remove_at_seq(old(self)@.sleeping, idx as int),
            self.wf(),
    {
        proof {
            // Uniqueness preservation (uses _cv variant to avoid trigger mismatch).
            self.lemma_remove_at_preserves_unique_cv(idx as int);
            // No-kernel-pid preservation: map result elements back to originals.
            let s: Seq<(i32, i32)> = self.sleeping@;
            let idx_int: int = idx as int;
            let sub1: Seq<(i32, i32)> = s.subrange(0, idx_int);
            let sub2: Seq<(i32, i32)> = s.subrange(idx_int + 1, s.len() as int);
            let result: Seq<(i32, i32)> = sub1 + sub2;
            assert(result =~= concrete_remove_at_seq(s, idx_int));
            assert forall|i: int|
                #![trigger result[i]]
                0 <= i < result.len() as int
            implies result[i].0 as int != CondvarView::spec_kernel_pid() by {
                if i < sub1.len() as int {
                    assert(result[i] == sub1[i]);
                    assert(sub1[i] == s[i]);
                } else {
                    let j: int = i - sub1.len() as int;
                    assert(result[i] == sub2[j]);
                    assert(sub2[j] == s[idx_int + 1 + j]);
                }
            }
        }
        self.len = self.len - 1;
        let _removed: (i32, i32) = self.sleeping.remove(idx);
        true
    }

    /// Removes a specific (pid, tid) entry from the sleeping queue.
    ///
    /// # Description
    ///
    /// Models the `wait()` failure cleanup path in the original, where
    /// `self.sleeping.borrow_mut().retain(|&mut (p, t)| p != pid || t != tid)`
    /// removes the entry after `ProcessManager::sleep()` fails. The caller
    /// provides the index where the entry is located.
    ///
    /// **Note:** The original `retain()` removes *all* entries matching
    /// `(pid, tid)`, while this model removes exactly one entry at the given
    /// index. Under trust assumption T1 (queue element uniqueness), these are
    /// equivalent — see `lemma_remove_entry_equivalent_to_retain` in the
    /// proof file.
    ///
    /// # Parameters
    ///
    /// - `pid_val`: Process identifier value to remove.
    /// - `tid_val`: Thread identifier value to remove.
    /// - `idx`: Index of the (pid, tid) entry in the queue.
    ///
    /// # Returns
    ///
    /// Always returns `true` (removal succeeds when preconditions hold).
    pub fn remove_entry(&mut self, pid_val: i32, tid_val: i32, idx: usize) -> (removed: bool)
        requires
            old(self).wf(),
            (idx as int) < old(self)@.sleeping.len() as int,
            old(self)@.sleeping[idx as int] == (pid_val as int, tid_val as int),
        ensures
            removed,
            self@.spec_len() == old(self)@.spec_len() - 1,
            self@.sleeping =~= CondvarView::spec_remove_at_seq(old(self)@.sleeping, idx as int),
            self.wf(),
    {
        self.remove_at(idx)
    }

    /// Removes the first entry matching a given process identifier.
    ///
    /// # Description
    ///
    /// Models the original `notify_process(pid)` which uses
    /// `LinkedList::iter().position()` to find the first entry with matching
    /// pid, then removes it. The index must point to the first entry
    /// whose pid component matches `pid_val`.
    ///
    /// # Parameters
    ///
    /// - `pid_val`: Process identifier value to search for.
    /// - `idx`: Index of the first matching entry in the queue.
    ///
    /// # Returns
    ///
    /// Always returns `true` (removal succeeds when preconditions hold).
    pub fn remove_by_pid(&mut self, pid_val: i32, idx: usize) -> (removed: bool)
        requires
            old(self).wf(),
            (idx as int) < old(self)@.sleeping.len() as int,
            old(self)@.sleeping[idx as int].0 == pid_val as int,
            // The index is the first match, modeling `position()` semantics.
            forall|k: int|
                #![trigger old(self)@.sleeping[k]]
                0 <= k < idx as int ==> old(self)@.sleeping[k].0 != pid_val as int,
        ensures
            removed,
            self@.spec_len() == old(self)@.spec_len() - 1,
            old(self)@.sleeping[idx as int].0 == pid_val as int,
            self@.sleeping =~= CondvarView::spec_remove_at_seq(old(self)@.sleeping, idx as int),
            self.wf(),
    {
        self.remove_at(idx)
    }

    /// Removes the first entry matching a given thread identifier.
    ///
    /// # Description
    ///
    /// Models the original `notify_thread(tid)` which uses
    /// `LinkedList::iter().position()` to find the first entry with matching
    /// tid, then removes it. The index must point to the first entry
    /// whose tid component matches `tid_val`.
    ///
    /// # Parameters
    ///
    /// - `tid_val`: Thread identifier value to search for.
    /// - `idx`: Index of the first matching entry in the queue.
    ///
    /// # Returns
    ///
    /// Always returns `true` (removal succeeds when preconditions hold).
    pub fn remove_by_tid(&mut self, tid_val: i32, idx: usize) -> (removed: bool)
        requires
            old(self).wf(),
            (idx as int) < old(self)@.sleeping.len() as int,
            old(self)@.sleeping[idx as int].1 == tid_val as int,
            // The index is the first match, modeling `position()` semantics.
            forall|k: int|
                #![trigger old(self)@.sleeping[k]]
                0 <= k < idx as int ==> old(self)@.sleeping[k].1 != tid_val as int,
        ensures
            removed,
            self@.spec_len() == old(self)@.spec_len() - 1,
            old(self)@.sleeping[idx as int].1 == tid_val as int,
            self@.sleeping =~= CondvarView::spec_remove_at_seq(old(self)@.sleeping, idx as int),
            self.wf(),
    {
        self.remove_at(idx)
    }

    /// Attempts to remove the first entry matching a given process identifier.
    ///
    /// # Description
    ///
    /// Models the original `notify_process(pid)` including the "not found"
    /// case where `position()` returns `None` and the queue is unchanged.
    /// The `has_match` parameter indicates whether a matching entry exists.
    /// If `true`, the index must point to the first matching entry.
    ///
    /// **Note:** `has_match` is concrete (not ghost) because the exec code
    /// branches on it. In the original, this is computed internally by
    /// `position()`. The preconditions constrain `has_match` to be consistent
    /// with `spec_contains_pid`, so an incorrect value makes the precondition
    /// unsatisfiable. A concrete call-site wrapper performing the search would
    /// supply the correct value. See trust assumption T5.
    ///
    /// # Parameters
    ///
    /// - `pid_val`: Process identifier value to search for.
    /// - `has_match`: Whether a matching entry exists in the queue.
    /// - `match_idx`: Index of the first matching entry (when found).
    ///
    /// # Returns
    ///
    /// `true` if an entry was found and removed, `false` if no match existed.
    pub fn try_remove_by_pid(&mut self, pid_val: i32, has_match: bool, match_idx: usize) -> (found: bool)
        requires
            old(self).wf(),
            // If match exists: match_idx is the first valid match.
            has_match ==> (
                (match_idx as int) < old(self)@.sleeping.len() as int
                && old(self)@.sleeping[match_idx as int].0 == pid_val as int
                && forall|k: int|
                    #![trigger old(self)@.sleeping[k]]
                    0 <= k < match_idx as int ==> old(self)@.sleeping[k].0 != pid_val as int
            ),
            // If no match: no entry has matching pid.
            !has_match ==> !old(self)@.spec_contains_pid(pid_val as int),
        ensures
            found == has_match,
            // If found: removal happened.
            found ==> self@.spec_len() == old(self)@.spec_len() - 1,
            found ==> self@.sleeping =~= CondvarView::spec_remove_at_seq(
                old(self)@.sleeping, match_idx as int,
            ),
            // If not found: state unchanged.
            !found ==> self@ == old(self)@,
            self.wf(),
    {
        if has_match {
            self.remove_at(match_idx);
            true
        } else {
            false
        }
    }

    /// Attempts to remove the first entry matching a given thread identifier.
    ///
    /// # Description
    ///
    /// Models the original `notify_thread(tid)` including the "not found"
    /// case where `position()` returns `None` and the queue is unchanged.
    /// The `has_match` parameter indicates whether a matching entry exists.
    /// If `true`, the index must point to the first matching entry.
    /// See `try_remove_by_pid` for the rationale on `has_match` being concrete.
    ///
    /// # Parameters
    ///
    /// - `tid_val`: Thread identifier value to search for.
    /// - `has_match`: Whether a matching entry exists in the queue.
    /// - `match_idx`: Index of the first matching entry (when found).
    ///
    /// # Returns
    ///
    /// `true` if an entry was found and removed, `false` if no match existed.
    pub fn try_remove_by_tid(&mut self, tid_val: i32, has_match: bool, match_idx: usize) -> (found: bool)
        requires
            old(self).wf(),
            // If match exists: match_idx is the first valid match.
            has_match ==> (
                (match_idx as int) < old(self)@.sleeping.len() as int
                && old(self)@.sleeping[match_idx as int].1 == tid_val as int
                && forall|k: int|
                    #![trigger old(self)@.sleeping[k]]
                    0 <= k < match_idx as int ==> old(self)@.sleeping[k].1 != tid_val as int
            ),
            // If no match: no entry has matching tid.
            !has_match ==> !old(self)@.spec_contains_tid(tid_val as int),
        ensures
            found == has_match,
            // If found: removal happened.
            found ==> self@.spec_len() == old(self)@.spec_len() - 1,
            found ==> self@.sleeping =~= CondvarView::spec_remove_at_seq(
                old(self)@.sleeping, match_idx as int,
            ),
            // If not found: state unchanged.
            !found ==> self@ == old(self)@,
            self.wf(),
    {
        if has_match {
            self.remove_at(match_idx);
            true
        } else {
            false
        }
    }

    /// Removes all threads from the sleeping queue.
    ///
    /// # Description
    ///
    /// Models the original `notify_all()` which drains the entire queue. The
    /// original also calls `ProcessManager::wakeup()` for each entry, which
    /// is not modeled here. **Note:** The returned count represents the total
    /// number of entries removed from the queue, not the number of successful
    /// wakeups. The original `notify_all()` returns the count of *successful*
    /// `ProcessManager::wakeup()` calls, which may be fewer if some fail.
    /// The `spec_notify_all_result` predicate constrains the relationship:
    /// `0 <= awakened <= total`.
    ///
    /// # Returns
    ///
    /// The total number of entries that were in the queue before clearing.
    pub fn clear(&mut self) -> (count: usize)
        requires
            old(self).wf(),
        ensures
            count as nat == old(self)@.spec_len(),
            // Any actual awakened count from the original satisfies this.
            CondvarView::spec_notify_all_result(0, count as nat),
            CondvarView::spec_notify_all_result(count as nat, count as nat),
            self@.spec_is_empty(),
            self@ == CondvarView::spec_new(),
            self.wf(),
    {
        let old_len: usize = self.len;
        self.len = 0;
        self.sleeping.clear();
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
            result == self@.spec_is_empty(),
    {
        self.len == 0
    }

    /// Returns the number of sleeping threads.
    ///
    /// # Returns
    ///
    /// The current queue length.
    pub fn get_len(&self) -> (result: usize)
        requires
            self.wf(),
        ensures
            result as nat == self@.spec_len(),
    {
        self.len
    }
}

} // verus!
