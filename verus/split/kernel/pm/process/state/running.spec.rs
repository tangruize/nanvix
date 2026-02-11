// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications for RunningProcess and boundary process types.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Constants
//==================================================================================================

/// Tag for try_join_thread: thread was zombie and removed.
pub const JOIN_TAG_ZOMBIE: u8 = 0;

/// Tag for try_join_thread: thread is the running thread (error).
pub const JOIN_TAG_RUNNING: u8 = 1;

/// Tag for try_join_thread: thread is live (ready/sleeping/interrupted).
pub const JOIN_TAG_LIVE: u8 = 2;

/// Tag for try_join_thread: thread not found (error).
pub const JOIN_TAG_NOT_FOUND: u8 = 3;

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of `RunningProcess`.
pub struct RunningProcessView {
    /// Process identifier.
    pub pid: u64,
    /// Running thread ID.
    pub running_thread_id: u64,
    /// Ready thread IDs.
    pub ready_thread_ids: Seq<u64>,
    /// Interrupted thread IDs.
    pub interrupted_thread_ids: Seq<u64>,
    /// Sleeping thread IDs.
    pub sleeping_thread_ids: Seq<u64>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Seq<u64>,
}

/// Abstract view of `RunnableProcess`.
pub struct RunnableProcessView {
    /// Process identifier.
    pub pid: u64,
    /// Ready thread IDs.
    pub ready_thread_ids: Seq<u64>,
    /// Interrupted thread IDs.
    pub interrupted_thread_ids: Seq<u64>,
    /// Sleeping thread IDs.
    pub sleeping_thread_ids: Seq<u64>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Seq<u64>,
}

/// Abstract view of `SleepingProcess`.
pub struct SleepingProcessView {
    /// Process identifier.
    pub pid: u64,
    /// Sleeping thread IDs.
    pub sleeping_thread_ids: Seq<u64>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Seq<u64>,
}

/// Abstract view of `InterruptedProcess`.
pub struct InterruptedProcessView {
    /// Process identifier.
    pub pid: u64,
    /// Interrupted thread IDs.
    pub interrupted_thread_ids: Seq<u64>,
    /// Sleeping thread IDs.
    pub sleeping_thread_ids: Seq<u64>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Seq<u64>,
}

/// Abstract view of `ZombieProcess`.
pub struct ZombieProcessView {
    /// Process identifier.
    pub pid: u64,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Seq<u64>,
    /// Exit status.
    pub status: u64,
}

//==================================================================================================
// Spec Functions — RunningProcess
//==================================================================================================

impl RunningProcess {
    /// Returns the process identifier.
    pub open spec fn spec_pid(&self) -> u64 {
        self.pid
    }

    /// Returns the running thread's identifier.
    pub open spec fn spec_running_thread_id(&self) -> u64 {
        self.running_thread_id
    }

    /// Returns the number of ready threads.
    pub open spec fn spec_ready_count(&self) -> nat {
        self.ready_thread_ids@.len()
    }

    /// Returns the number of interrupted threads.
    pub open spec fn spec_interrupted_count(&self) -> nat {
        self.interrupted_thread_ids@.len()
    }

    /// Returns the number of sleeping threads.
    pub open spec fn spec_sleeping_count(&self) -> nat {
        self.sleeping_thread_ids@.len()
    }

    /// Returns the number of zombie threads.
    pub open spec fn spec_zombie_count(&self) -> nat {
        self.zombie_thread_ids@.len()
    }

    /// Returns the total number of threads (running + all lists).
    pub open spec fn spec_total_thread_count(&self) -> nat {
        1 + self.spec_ready_count() + self.spec_interrupted_count()
            + self.spec_sleeping_count() + self.spec_zombie_count()
    }

    /// Returns whether `tid` is in the ready list.
    pub open spec fn spec_has_ready_thread(&self, tid: u64) -> bool {
        Self::spec_seq_contains(self.ready_thread_ids@, tid)
    }

    /// Returns whether `tid` is in the interrupted list.
    pub open spec fn spec_has_interrupted_thread(&self, tid: u64) -> bool {
        Self::spec_seq_contains(self.interrupted_thread_ids@, tid)
    }

    /// Returns whether `tid` is in the sleeping list.
    pub open spec fn spec_has_sleeping_thread(&self, tid: u64) -> bool {
        Self::spec_seq_contains(self.sleeping_thread_ids@, tid)
    }

    /// Returns whether `tid` is in the zombie list.
    pub open spec fn spec_has_zombie_thread(&self, tid: u64) -> bool {
        Self::spec_seq_contains(self.zombie_thread_ids@, tid)
    }

    /// Returns whether `tid` is anywhere in this process (running or any list).
    pub open spec fn spec_has_thread(&self, tid: u64) -> bool {
        self.running_thread_id == tid
        || self.spec_has_ready_thread(tid)
        || self.spec_has_interrupted_thread(tid)
        || self.spec_has_sleeping_thread(tid)
        || self.spec_has_zombie_thread(tid)
    }

    /// Returns which list `tid` belongs to, or None if not found.
    ///
    /// - `Some(0)`: running.
    /// - `Some(1)`: ready.
    /// - `Some(2)`: interrupted.
    /// - `Some(3)`: sleeping.
    /// - `Some(4)`: zombie.
    /// - `None`: not found.
    pub open spec fn spec_find_thread(&self, tid: u64) -> Option<int> {
        if self.running_thread_id == tid {
            Some(0int)
        } else if self.spec_has_ready_thread(tid) {
            Some(1int)
        } else if self.spec_has_interrupted_thread(tid) {
            Some(2int)
        } else if self.spec_has_sleeping_thread(tid) {
            Some(3int)
        } else if self.spec_has_zombie_thread(tid) {
            Some(4int)
        } else {
            None
        }
    }

    /// Returns the try_join_thread result tag for `tid`.
    ///
    /// Priority: running → zombie → live → not found.
    /// This matches the original code's search order.
    pub open spec fn spec_try_join_thread(&self, tid: u64) -> int {
        if self.running_thread_id == tid {
            JOIN_TAG_RUNNING as int
        } else if self.spec_has_zombie_thread(tid) {
            JOIN_TAG_ZOMBIE as int
        } else if self.spec_has_ready_thread(tid)
            || self.spec_has_sleeping_thread(tid)
            || self.spec_has_interrupted_thread(tid) {
            JOIN_TAG_LIVE as int
        } else {
            JOIN_TAG_NOT_FOUND as int
        }
    }

    /// Returns the zombie list after removing one occurrence of `tid`.
    ///
    /// Uses `choose` to pick a valid index. The exec code removes the first
    /// occurrence; the ensures clause uses an existential to match.
    pub open spec fn spec_try_join_zombie_post(&self, tid: u64) -> Seq<u64> {
        let s: Seq<u64> = self.zombie_thread_ids@;
        let idx: int = choose|i: int|
            0 <= i < s.len()
            && #[trigger] s[i] == tid;
        Self::spec_remove_at(s, idx)
    }

    /// Returns whether `s` contains `tid`.
    pub open spec fn spec_seq_contains(s: Seq<u64>, tid: u64) -> bool {
        exists|i: int| 0 <= i < s.len() && s[i] == tid
    }

    /// Returns `s` with the element at `idx` removed.
    pub open spec fn spec_remove_at(s: Seq<u64>, idx: int) -> Seq<u64> {
        s.subrange(0, idx).add(s.subrange(idx + 1, s.len() as int))
    }

    /// Returns whether two sequences have no element in common.
    pub open spec fn spec_seqs_disjoint(a: Seq<u64>, b: Seq<u64>) -> bool {
        forall|i: int, j: int|
            0 <= i < a.len() && 0 <= j < b.len()
            ==> a[i] != b[j]
    }

    /// Well-formedness: exec counts match ghost sequence lengths.
    pub open spec fn wf(&self) -> bool {
        &&& self.ready_count as nat == self.ready_thread_ids@.len()
        &&& self.interrupted_count as nat == self.interrupted_thread_ids@.len()
        &&& self.sleeping_count as nat == self.sleeping_thread_ids@.len()
        &&& self.zombie_count as nat == self.zombie_thread_ids@.len()
    }

    /// Strict well-formedness: wf plus disjointness of all thread lists
    /// and uniqueness of running thread ID across all lists.
    pub open spec fn wf_strict(&self) -> bool {
        &&& self.wf()
        &&& !self.spec_has_ready_thread(self.running_thread_id)
        &&& !self.spec_has_interrupted_thread(self.running_thread_id)
        &&& !self.spec_has_sleeping_thread(self.running_thread_id)
        &&& !self.spec_has_zombie_thread(self.running_thread_id)
        &&& Self::spec_seqs_disjoint(self.ready_thread_ids@, self.interrupted_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.ready_thread_ids@, self.sleeping_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.ready_thread_ids@, self.zombie_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.interrupted_thread_ids@, self.sleeping_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.interrupted_thread_ids@, self.zombie_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.sleeping_thread_ids@, self.zombie_thread_ids@)
    }

    /// Frame condition for mutations through state_mut / running_mut.
    pub open spec fn mutation_frame_preserved(old_self: &Self, new_self: &Self) -> bool {
        &&& new_self.spec_pid() == old_self.spec_pid()
        &&& new_self.spec_running_thread_id() == old_self.spec_running_thread_id()
        &&& new_self.ready_thread_ids@ == old_self.ready_thread_ids@
        &&& new_self.interrupted_thread_ids@ == old_self.interrupted_thread_ids@
        &&& new_self.sleeping_thread_ids@ == old_self.sleeping_thread_ids@
        &&& new_self.zombie_thread_ids@ == old_self.zombie_thread_ids@
        &&& new_self.ready_count == old_self.ready_count
        &&& new_self.interrupted_count == old_self.interrupted_count
        &&& new_self.sleeping_count == old_self.sleeping_count
        &&& new_self.zombie_count == old_self.zombie_count
    }
}

//==================================================================================================
// Spec Functions — RunnableProcess
//==================================================================================================

impl RunnableProcess {
    /// Returns the process identifier.
    pub open spec fn spec_pid(&self) -> u64 {
        self.pid
    }

    /// Well-formedness: ready list is non-empty.
    pub open spec fn wf(&self) -> bool {
        self.ready_thread_ids@.len() >= 1
    }
}

//==================================================================================================
// Spec Functions — SleepingProcess
//==================================================================================================

impl SleepingProcess {
    /// Returns the process identifier.
    pub open spec fn spec_pid(&self) -> u64 {
        self.pid
    }

    /// Well-formedness: sleeping list is non-empty.
    pub open spec fn wf(&self) -> bool {
        self.sleeping_thread_ids@.len() >= 1
    }
}

//==================================================================================================
// Spec Functions — InterruptedProcess
//==================================================================================================

impl InterruptedProcess {
    /// Returns the process identifier.
    pub open spec fn spec_pid(&self) -> u64 {
        self.pid
    }

    /// Well-formedness: interrupted list is non-empty.
    pub open spec fn wf(&self) -> bool {
        self.interrupted_thread_ids@.len() >= 1
    }
}

//==================================================================================================
// Spec Functions — ZombieProcess
//==================================================================================================

impl ZombieProcess {
    /// Returns the process identifier.
    pub open spec fn spec_pid(&self) -> u64 {
        self.pid
    }

    /// Returns the exit status.
    pub open spec fn spec_status(&self) -> u64 {
        self.status
    }

    /// Well-formedness: zombie list is non-empty.
    pub open spec fn wf(&self) -> bool {
        self.zombie_thread_ids@.len() >= 1
    }
}

//==================================================================================================
// View Implementations
//==================================================================================================

impl View for RunningProcess {
    type V = RunningProcessView;
    open spec fn view(&self) -> RunningProcessView {
        RunningProcessView {
            pid: self.pid,
            running_thread_id: self.running_thread_id,
            ready_thread_ids: self.ready_thread_ids@,
            interrupted_thread_ids: self.interrupted_thread_ids@,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

impl View for RunnableProcess {
    type V = RunnableProcessView;
    open spec fn view(&self) -> RunnableProcessView {
        RunnableProcessView {
            pid: self.pid,
            ready_thread_ids: self.ready_thread_ids@,
            interrupted_thread_ids: self.interrupted_thread_ids@,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

impl View for SleepingProcess {
    type V = SleepingProcessView;
    open spec fn view(&self) -> SleepingProcessView {
        SleepingProcessView {
            pid: self.pid,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

impl View for InterruptedProcess {
    type V = InterruptedProcessView;
    open spec fn view(&self) -> InterruptedProcessView {
        InterruptedProcessView {
            pid: self.pid,
            interrupted_thread_ids: self.interrupted_thread_ids@,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

impl View for ZombieProcess {
    type V = ZombieProcessView;
    open spec fn view(&self) -> ZombieProcessView {
        ZombieProcessView {
            pid: self.pid,
            zombie_thread_ids: self.zombie_thread_ids@,
            status: self.status,
        }
    }
}

} // verus!
