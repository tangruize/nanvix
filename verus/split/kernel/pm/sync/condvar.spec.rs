// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Condvar Specification.
// This file contains spec functions for the Condvar type.

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a Condvar.
///
/// # Description
///
/// Represents the observable state of a condition variable: the queue of waiting
/// threads, each identified by a (pid, tid) pair stored as (int, int).
#[verifier::ext_equal]
pub struct CondvarView {
    /// The sequence of sleeping (pid, tid) pairs, in FIFO order.
    pub sleeping: Seq<(int, int)>,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl Condvar {
    /// Spec function: well-formedness predicate.
    ///
    /// # Description
    ///
    /// Enforces that the concrete length counter matches the ghost sequence
    /// length, that all queue entries are unique (trust assumption T1), and
    /// that no kernel process entry exists in the queue (safety invariant
    /// from the original `wait()` panic guard).
    pub open spec fn wf(&self) -> bool {
        &&& self.len as nat == self.sleeping@.len()
        &&& self.spec_all_unique()
        &&& self.spec_no_kernel_pid()
    }

    /// Spec function: returns whether the sleeping queue is empty.
    pub open spec fn spec_is_empty(&self) -> bool {
        self@.sleeping.len() == 0
    }

    /// Spec function: returns whether the sleeping queue is non-empty.
    pub open spec fn spec_is_nonempty(&self) -> bool {
        self@.sleeping.len() > 0
    }

    /// Spec function: returns the queue length.
    pub open spec fn spec_len(&self) -> nat {
        self@.sleeping.len()
    }

    /// Spec function: the view of a newly created condvar.
    pub open spec fn spec_new_view() -> CondvarView {
        CondvarView { sleeping: Seq::empty() }
    }

    /// Spec function: returns whether the queue contains an entry with the given pid.
    pub open spec fn spec_contains_pid(&self, pid_val: int) -> bool {
        exists|i: int|
            #![trigger self@.sleeping[i]]
            0 <= i < self@.sleeping.len() as int && self@.sleeping[i].0 == pid_val
    }

    /// Spec function: returns whether the queue contains an entry with the given tid.
    pub open spec fn spec_contains_tid(&self, tid_val: int) -> bool {
        exists|i: int|
            #![trigger self@.sleeping[i]]
            0 <= i < self@.sleeping.len() as int && self@.sleeping[i].1 == tid_val
    }

    /// Spec function: returns whether the queue contains a specific (pid, tid) pair.
    pub open spec fn spec_contains_entry(&self, pid_val: int, tid_val: int) -> bool {
        exists|i: int|
            #![trigger self@.sleeping[i]]
            0 <= i < self@.sleeping.len() as int
            && self@.sleeping[i].0 == pid_val
            && self@.sleeping[i].1 == tid_val
    }

    /// Spec function: returns whether all queue entries are unique.
    ///
    /// # Description
    ///
    /// Formalizes trust assumption T1: each thread appears at most once in the
    /// sleeping queue. This predicate can be used as a precondition to prove
    /// stronger postconditions (e.g., after `remove_entry`, the entry is absent).
    pub open spec fn spec_all_unique(&self) -> bool {
        forall|i: int, j: int|
            #![trigger self@.sleeping[i], self@.sleeping[j]]
            0 <= i < self@.sleeping.len() as int
            && 0 <= j < self@.sleeping.len() as int
            && i != j
            ==> self@.sleeping[i] != self@.sleeping[j]
    }

    /// Spec function: returns whether the queue contains no kernel process entry.
    ///
    /// # Description
    ///
    /// Formalizes the safety invariant from the original `wait()` which panics
    /// if `pid == ProcessIdentifier::KERNEL`. Including this in `wf()` makes
    /// it a global invariant preserved by all operations.
    pub open spec fn spec_no_kernel_pid(&self) -> bool {
        forall|i: int|
            #![trigger self@.sleeping[i]]
            0 <= i < self@.sleeping.len() as int
            ==> self@.sleeping[i].0 != Condvar::spec_kernel_pid()
    }

    /// Spec function: returns the front element of the queue.
    pub open spec fn spec_front(&self) -> (int, int)
        recommends !self.spec_is_empty()
    {
        self@.sleeping[0]
    }

    /// Spec function: returns the back element of the queue.
    pub open spec fn spec_back(&self) -> (int, int)
        recommends !self.spec_is_empty()
    {
        self@.sleeping[self@.sleeping.len() as int - 1]
    }

    /// Spec function: returns the sequence after removing the element at index `idx`.
    pub open spec fn spec_remove_at_seq(s: Seq<(int, int)>, idx: int) -> Seq<(int, int)>
        recommends 0 <= idx < s.len()
    {
        s.subrange(0, idx) + s.subrange(idx + 1, s.len() as int)
    }

    /// Spec function: returns whether the condvar is safe to drop.
    ///
    /// # Description
    ///
    /// Formalizes trust assumption T2: the queue must be empty when the condvar
    /// is dropped. The original enforces this via a panic in `Drop::drop`.
    /// Callers can use this predicate to reason about drop safety.
    pub open spec fn spec_drop_safe(&self) -> bool {
        self.spec_is_empty()
    }

    /// Spec constant: the raw pid value of the kernel process.
    ///
    /// # Description
    ///
    /// Matches `ProcessIdentifier::KERNEL_RAW` (value 0) from the original.
    /// The original `wait()` panics if this pid tries to sleep.
    pub open spec fn spec_kernel_pid() -> int {
        0
    }

    /// Spec predicate: constrains the possible return value of `notify_all()`.
    ///
    /// # Description
    ///
    /// The original `notify_all()` returns the count of *successful* wakeups,
    /// which is at most the total number of entries drained. This predicate
    /// captures the invariant `0 <= awakened <= total` without modeling
    /// `ProcessManager::wakeup()` outcomes.
    pub open spec fn spec_notify_all_result(awakened: nat, total: nat) -> bool {
        awakened <= total
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for Condvar {
    type V = CondvarView;

    open spec fn view(&self) -> CondvarView {
        CondvarView { sleeping: self.sleeping@ }
    }
}

} // verus!
