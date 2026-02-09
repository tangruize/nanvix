// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// RunnableProcess Specification.
// This file contains spec functions and View types for the RunnableProcess type
// and boundary types RunningProcess, InterruptedProcess, ZombieProcess.
//
// ## Verification Model
//
// RunnableProcess manages process-level thread scheduling. For verification:
// - Thread lists are modeled as `Seq<int>` of abstract thread IDs.
// - `ready_admission_times` tracks admission times paired with ready thread IDs.
// - `Option<NonEmptyVecDeque<T>>` is modeled as `Seq<int>`:
//   - `Seq::empty()` represents `None` (no threads in that state).
//   - `Seq` with `len() >= 1` represents `Some(non_empty_deque)`.
//   This is a sound isomorphism because `NonEmptyVecDeque` always has `len() >= 1`,
//   and `wf()` enforces `ready_thread_ids.len() >= 1`. The verification checks
//   correct lengths at all transition boundaries.
// - `Box<ProcessState>` is transparent (modeled as ProcessState directly).
// - `ProcessState` uses the verified dependency's `spec_pid()`.
// - `RunningProcess`, `InterruptedProcess`, `ZombieProcess` are boundary models.
//
// ## Key Invariants
//
// - A RunnableProcess always has at least one ready thread (`ready_thread_ids.len() >= 1`).
// - Process identity (PID) is immutable across all operations.
// - Ready thread IDs and admission times sequences have matching lengths.
// - All admission times are non-negative.
//
// ## Ownership Semantics (Trust Assumption)
//
// Thread ID uniqueness across lists is NOT enforced in `wf()`. In the original
// code, the Rust type system ensures ownership semantics — a thread struct can
// only be in one `NonEmptyVecDeque` at a time. This module inherits that
// guarantee as a trust assumption: callers constructing a `RunnableProcess`
// must ensure thread IDs are disjoint across lists. The verification proves
// that *operations* (run, terminate, wakeup, add_thread) move IDs between
// lists correctly (via content-level postconditions), which is the actionable
// property.
//
// ## Trust Assumptions
//
// - Thread ID ownership/disjointness is inherited from Rust's type system
//   (see above).
// - `ContextInformation` and `VirtualAddress` from run() are omitted (HAL boundary).
//   These are hardware abstraction layer types whose values are produced by
//   architecture-specific code (context switching, TDA setup). They carry no
//   protocol-level invariants relevant to process state verification.
// - `InterruptReason` from run() return is modeled as abstract int tag.
// - `UserTda` (user thread data area) from run() return is omitted (HAL boundary).
//
// ## Exec Coverage Notes
//
// The following original functions are NOT modeled as exec functions:
// - `state()` / `state_mut()`: Return `&ProcessState` / `&mut ProcessState`.
//   Verus cannot express these reference return types. The relevant property
//   (PID access) is modeled via `pid_i32()` and `spec_pid()`.
//   Note: `state_mut()` allows arbitrary mutation of the inner `ProcessState`,
//   which could affect invariants (e.g., vmem mapping). Cross-module verification
//   of `ProcessState` mutations should be addressed when verifying callers.
// - `find_thread()` / `find_thread_mut()`: Return `Option<ThreadRef>` containing
//   references into internal collections. Modeled spec-only via `spec_find_thread`.
//   Callers of these functions should independently verify the correctness of
//   the returned reference's properties against the `spec_find_thread` model.
// - `earliest_admission_time()`: Returns `SystemTime` which maps to ghost `int`.
//   Modeled via `spec_earliest_admission_time()` with proven bounds.
//   The original has a fallback `unwrap_or(clock::now())` for the case when the
//   iterator returns no minimum. This fallback is dead code given the
//   `NonEmptyVecDeque` invariant (the ready list always has at least one element).
//   The spec model correctly omits this unreachable fallback.
//
// ## Oracle Parameter Justification
//
// `terminate()` no longer requires an oracle parameter. The branch decision
// is computed from exec-level counters (`interrupted_count`, `sleeping_count`)
// that are tied to ghost sequence lengths by the `wf()` invariant.
//
// `wakeup()` retains a `found: bool` oracle parameter because:
// - The branch decision depends on whether a specific thread ID exists in
//   the ghost sleeping list (`Seq::contains()` is spec-only).
// - There is no exec-level data structure to search — the sleeping list is
//   entirely ghost. The precondition `found == spec_seq_contains(...)` ties
//   the oracle to ghost state, so the branch is fully constrained.
// - The search *index* is derived internally via proof-level `choose`,
//   so only the boolean remains as oracle.
//
// `run()` has no oracle parameters. The min-index is derived internally
// via `lemma_earliest_ready_index_bounds`.

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a RunnableProcess.
#[verifier::ext_equal]
pub struct RunnableProcessView {
    /// Process identifier value.
    pub pid: int,
    /// Sequence of ready thread IDs (non-empty).
    pub ready_thread_ids: Seq<int>,
    /// Sequence of ready thread admission times, parallel to ready_thread_ids.
    pub ready_admission_times: Seq<int>,
    /// Sequence of interrupted thread IDs (may be empty).
    pub interrupted_thread_ids: Seq<int>,
    /// Sequence of sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Seq<int>,
    /// Sequence of zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<int>,
}

/// Abstract view of a RunningProcess (boundary type).
#[verifier::ext_equal]
pub struct RunningProcessView {
    /// Process identifier value.
    pub pid: int,
    /// The running thread ID.
    pub running_thread_id: int,
    /// Ready thread IDs (may be empty).
    pub ready_thread_ids: Seq<int>,
    /// Interrupted thread IDs (may be empty).
    pub interrupted_thread_ids: Seq<int>,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Seq<int>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<int>,
    /// Interrupt reason (unconstrained at this abstraction level).
    pub interrupt_reason: int,
}

/// Abstract view of an InterruptedProcess (boundary type).
#[verifier::ext_equal]
pub struct InterruptedProcessView {
    /// Process identifier value.
    pub pid: int,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Seq<int>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<int>,
}

/// Abstract view of a ZombieProcess (boundary type).
#[verifier::ext_equal]
pub struct ZombieProcessView {
    /// Process identifier value.
    pub pid: int,
    /// Zombie thread IDs (non-empty).
    pub zombie_thread_ids: Seq<int>,
    /// Exit status.
    pub status: int,
}

//==================================================================================================
// Spec Constants
//==================================================================================================

/// Abstract exit status for interrupted processes.
/// Models `ErrorCode::Interrupted.into()` from the original code.
/// Value 4 corresponds to EINTR: see `src/libs/sysapi/src/errno.rs:21`
/// and `ErrorCode::Interrupted` at `src/libs/sysapi/src/error.rs`.
/// TODO (cross-module/CI): Add a CI check or cross-module assertion that
/// validates this constant against the actual `ErrorCode::Interrupted` value.
/// If the error code numbering changes, this must be updated accordingly.
pub open spec fn EXIT_STATUS_INTERRUPTED() -> int { 4 }

//==================================================================================================
// Spec Functions: RunnableProcess
//==================================================================================================

impl RunnableProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid.spec_value()
    }

    /// Spec function: returns the number of ready threads.
    pub open spec fn spec_ready_count(&self) -> nat {
        self.ready_thread_ids@.len()
    }

    /// Spec function: returns the number of interrupted threads.
    pub open spec fn spec_interrupted_count(&self) -> nat {
        self.interrupted_thread_ids@.len()
    }

    /// Spec function: returns the number of sleeping threads.
    pub open spec fn spec_sleeping_count(&self) -> nat {
        self.sleeping_thread_ids@.len()
    }

    /// Spec function: returns the number of zombie threads.
    pub open spec fn spec_zombie_count(&self) -> nat {
        self.zombie_thread_ids@.len()
    }

    /// Spec function: returns the total number of threads.
    pub open spec fn spec_total_thread_count(&self) -> nat {
        self.spec_ready_count()
        + self.spec_interrupted_count()
        + self.spec_sleeping_count()
        + self.spec_zombie_count()
    }

    /// Spec function: returns the i-th ready thread ID.
    pub open spec fn spec_ready_thread_id(&self, i: int) -> int
        recommends 0 <= i < self.ready_thread_ids@.len()
    {
        self.ready_thread_ids@[i]
    }

    /// Spec function: returns the i-th ready thread admission time.
    pub open spec fn spec_ready_admission_time(&self, i: int) -> int
        recommends 0 <= i < self.ready_admission_times@.len()
    {
        self.ready_admission_times@[i]
    }

    /// Spec function: checks if a thread ID is in the ready list.
    pub open spec fn spec_has_ready_thread(&self, tid: int) -> bool {
        exists|i: int| 0 <= i < self.ready_thread_ids@.len()
            && self.ready_thread_ids@[i] == tid
    }

    /// Spec function: checks if a thread ID is in the sleeping list.
    pub open spec fn spec_has_sleeping_thread(&self, tid: int) -> bool {
        exists|i: int| 0 <= i < self.sleeping_thread_ids@.len()
            && self.sleeping_thread_ids@[i] == tid
    }

    /// Spec function: checks if a thread ID is in any list.
    pub open spec fn spec_has_thread(&self, tid: int) -> bool {
        self.spec_has_ready_thread(tid)
        || self.spec_has_sleeping_thread(tid)
        || self.spec_has_interrupted_thread(tid)
        || self.spec_has_zombie_thread(tid)
    }

    /// Spec function: checks if a thread ID is in the interrupted list.
    pub open spec fn spec_has_interrupted_thread(&self, tid: int) -> bool {
        exists|i: int| 0 <= i < self.interrupted_thread_ids@.len()
            && self.interrupted_thread_ids@[i] == tid
    }

    /// Spec function: checks if a thread ID is in the zombie list.
    pub open spec fn spec_has_zombie_thread(&self, tid: int) -> bool {
        exists|i: int| 0 <= i < self.zombie_thread_ids@.len()
            && self.zombie_thread_ids@[i] == tid
    }

    /// Spec function: models `find_thread()` — returns which list a thread is in.
    ///
    /// The original `find_thread()` returns `Option<ThreadRef>` with a variant
    /// tag indicating which list (Ready, Interrupted, Sleeping, Zombie) the
    /// thread was found in. Verus cannot express reference types, so we model
    /// the result as an `Option<int>` tag:
    /// - `None` if the thread is not found in any list.
    /// - `Some(0)` if found in ready threads.
    /// - `Some(1)` if found in interrupted threads.
    /// - `Some(2)` if found in sleeping threads.
    /// - `Some(3)` if found in zombie threads.
    ///
    /// The search order matches the original: ready → interrupted → sleeping → zombie.
    /// This models the exhaustive search and correct variant selection.
    pub open spec fn spec_find_thread(&self, tid: int) -> Option<int> {
        if self.spec_has_ready_thread(tid) {
            Some(0int)
        } else if self.spec_has_interrupted_thread(tid) {
            Some(1int)
        } else if self.spec_has_sleeping_thread(tid) {
            Some(2int)
        } else if self.spec_has_zombie_thread(tid) {
            Some(3int)
        } else {
            None
        }
    }

    /// Spec helper: checks if a sequence contains a given value.
    /// Used for searching thread lists without oracle parameters.
    pub open spec fn spec_seq_contains(s: Seq<int>, tid: int) -> bool {
        exists|i: int| 0 <= i < s.len() && s[i] == tid
    }

    /// Spec helper: computes the sequence resulting from removing index `idx`
    /// from sequence `s`.
    pub open spec fn spec_remove_at(s: Seq<int>, idx: int) -> Seq<int>
        recommends 0 <= idx < s.len()
    {
        s.subrange(0, idx).add(s.subrange(idx + 1, s.len() as int))
    }

    /// Spec function: recursively finds the index of minimum in `s[0..n]`.
    pub open spec fn spec_min_index_rec(s: Seq<int>, n: int) -> int
        recommends 1 <= n <= s.len()
        decreases n
    {
        if n <= 1 {
            0int
        } else {
            let prev: int = Self::spec_min_index_rec(s, n - 1);
            if 0 <= prev < s.len() && s[n - 1] < s[prev] {
                n - 1
            } else {
                prev
            }
        }
    }

    /// Spec function: finds the index of the ready thread with earliest admission time.
    pub open spec fn spec_earliest_ready_index(&self) -> int
        recommends self.ready_thread_ids@.len() > 0
    {
        Self::spec_min_index_rec(
            self.ready_admission_times@,
            self.ready_admission_times@.len() as int,
        )
    }

    /// Spec function: returns the earliest admission time among ready threads.
    pub open spec fn spec_earliest_admission_time(&self) -> int
        recommends self.ready_thread_ids@.len() > 0
    {
        self.ready_admission_times@[self.spec_earliest_ready_index()]
    }

    /// Spec function: well-formedness predicate.
    ///
    /// A RunnableProcess is well-formed when:
    /// - There is at least one ready thread (modeling NonEmptyVecDeque).
    /// - Ready thread IDs and admission times sequences have equal length.
    /// - All admission times are non-negative.
    /// - Exec-level counters match ghost sequence lengths.
    ///
    /// Note: Thread ID uniqueness/disjointness across lists is NOT enforced here.
    /// In the original code, Rust's ownership type system ensures a thread struct
    /// can only be in one collection. This is a trust assumption inherited from
    /// the type system. The content-level postconditions on run(), terminate(),
    /// wakeup(), and add_thread() verify that operations move IDs correctly.
    /// See `spec_ids_disjoint()` for an optional disjointness predicate available
    /// to downstream cross-module proofs.
    pub open spec fn wf(&self) -> bool {
        // At least one ready thread (NonEmptyVecDeque invariant).
        &&& self.ready_thread_ids@.len() >= 1
        // Parallel arrays have matching lengths.
        &&& self.ready_thread_ids@.len() == self.ready_admission_times@.len()
        // Admission times are non-negative.
        &&& forall|i: int| 0 <= i < self.ready_admission_times@.len()
                ==> #[trigger] self.ready_admission_times@[i] >= 0
        // Exec counters match ghost sequence lengths.
        &&& self.interrupted_count as nat == self.interrupted_thread_ids@.len()
        &&& self.sleeping_count as nat == self.sleeping_thread_ids@.len()
    }

    /// Spec helper: checks whether two sequences share no common elements.
    pub open spec fn spec_seqs_disjoint(a: Seq<int>, b: Seq<int>) -> bool {
        forall|i: int, j: int|
            0 <= i < a.len() && 0 <= j < b.len()
            ==> a[i] != b[j]
    }

    /// Spec function: pairwise thread ID disjointness across all four lists.
    ///
    /// NOT part of `wf()` — this is a trust assumption inherited from Rust's
    /// ownership model. Provided as an optional predicate for downstream
    /// cross-module proofs that need to assert thread ID exclusivity.
    pub open spec fn spec_ids_disjoint(&self) -> bool {
        // ready vs interrupted.
        Self::spec_seqs_disjoint(self.ready_thread_ids@, self.interrupted_thread_ids@)
        // ready vs sleeping.
        && Self::spec_seqs_disjoint(self.ready_thread_ids@, self.sleeping_thread_ids@)
        // ready vs zombie.
        && Self::spec_seqs_disjoint(self.ready_thread_ids@, self.zombie_thread_ids@)
        // interrupted vs sleeping.
        && Self::spec_seqs_disjoint(self.interrupted_thread_ids@, self.sleeping_thread_ids@)
        // interrupted vs zombie.
        && Self::spec_seqs_disjoint(self.interrupted_thread_ids@, self.zombie_thread_ids@)
        // sleeping vs zombie.
        && Self::spec_seqs_disjoint(self.sleeping_thread_ids@, self.zombie_thread_ids@)
    }
}

//==================================================================================================
// Spec Functions: RunningProcess (Boundary)
//==================================================================================================

impl RunningProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid@
    }

    /// Spec function: returns the running thread ID.
    pub open spec fn spec_running_thread_id(&self) -> int {
        self.running_thread_id@
    }

    /// Spec function: well-formedness predicate.
    ///
    /// This boundary model has minimal invariants. PID consistency is
    /// asserted in the run() postcondition (`result.spec_pid() == self.spec_pid()`).
    /// Thread list structural invariants (running thread not in remaining ready
    /// list) would require thread ID uniqueness, which is a trust assumption
    /// from Rust's ownership model (see Ownership Semantics in spec file).
    ///
    /// TODO (cross-module): When RunningProcess verification is complete,
    /// add a cross-module linking assertion confirming this boundary model's
    /// postconditions are implied by the real RunningProcess module's specs.
    pub open spec fn wf(&self) -> bool {
        true
    }
}

//==================================================================================================
// Spec Functions: InterruptedProcess (Boundary)
//==================================================================================================

impl InterruptedProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid@
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        self.interrupted_thread_ids@.len() >= 1
    }
}

//==================================================================================================
// Spec Functions: ZombieProcess (Boundary)
//==================================================================================================

impl ZombieProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid@
    }

    /// Spec function: returns the exit status.
    pub open spec fn spec_status(&self) -> int {
        self.status@
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        self.zombie_thread_ids@.len() >= 1
    }
}

//==================================================================================================
// View Implementations
//==================================================================================================

impl View for RunnableProcess {
    type V = RunnableProcessView;

    open spec fn view(&self) -> RunnableProcessView {
        RunnableProcessView {
            pid: self.pid.spec_value(),
            ready_thread_ids: self.ready_thread_ids@,
            ready_admission_times: self.ready_admission_times@,
            interrupted_thread_ids: self.interrupted_thread_ids@,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

impl View for RunningProcess {
    type V = RunningProcessView;

    open spec fn view(&self) -> RunningProcessView {
        RunningProcessView {
            pid: self.pid@,
            running_thread_id: self.running_thread_id@,
            ready_thread_ids: self.ready_thread_ids@,
            interrupted_thread_ids: self.interrupted_thread_ids@,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
            interrupt_reason: self.interrupt_reason@,
        }
    }
}

impl View for InterruptedProcess {
    type V = InterruptedProcessView;

    open spec fn view(&self) -> InterruptedProcessView {
        InterruptedProcessView {
            pid: self.pid@,
            interrupted_thread_ids: self.interrupted_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

impl View for ZombieProcess {
    type V = ZombieProcessView;

    open spec fn view(&self) -> ZombieProcessView {
        ZombieProcessView {
            pid: self.pid@,
            zombie_thread_ids: self.zombie_thread_ids@,
            status: self.status@,
        }
    }
}

} // verus!
