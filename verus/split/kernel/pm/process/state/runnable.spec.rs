// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// RunnableProcess Specification.
// This file contains spec functions and View types for the RunnableProcess type
// and boundary types RunningProcess, InterruptedProcess, ZombieProcess.
//
// ## View-Level Abstract State Transitions
//
// The following spec functions on `RunnableProcessView` provide abstract
// state transition descriptions so that downstream modules can write
// postconditions in the form `result@ =~= old(self)@.spec_foo(args)`
// instead of listing every field change individually:
//
// - `spec_new(pid, tid, time)` — constructs the initial view (models `new()`).
// - `spec_from_state(...)` — constructs a view from existing state (models `from_state()`).
// - `spec_run()` — selects the earliest-admission-time thread and returns
//   a `RunningProcessView` (models `run()`).
// - `spec_terminate_has_interrupted()` — predicate for the terminate branch.
// - `spec_terminate_to_interrupted()` — returns `InterruptedProcessView`
//   (models `terminate()` when interrupted/sleeping threads exist).
// - `spec_terminate_to_zombie()` — returns `ZombieProcessView`
//   (models `terminate()` when no interrupted/sleeping threads exist).
// - `spec_wakeup(tid, time)` — moves a sleeping thread to ready, returns
//   updated `RunnableProcessView` (models `wakeup()` success case).
// - `spec_add_thread(tid, time)` — appends a thread to the ready queue,
//   returns updated `RunnableProcessView` (models `add_thread()`).
//
// ## Verification Model
//
// RunnableProcess manages process-level thread scheduling. For verification:
// - Thread lists are modeled as `Vec<i64>` with view `Seq<i64>` of abstract thread IDs.
// - `ready_admission_times` tracks admission times paired with ready thread IDs.
// - `Option<NonEmptyVecDeque<T>>` is modeled as `Vec<i64>`:
//   - Empty Vec represents `None` (no threads in that state).
//   - Vec with `len() >= 1` represents `Some(non_empty_deque)`.
//   This is a sound isomorphism because `NonEmptyVecDeque` always has `len() >= 1`,
//   and `wf()` enforces `ready_thread_ids@.len() >= 1`. The verification checks
//   correct lengths at all transition boundaries.
// - `Box<ProcessState>` is transparent (modeled as ProcessState directly).
// - `ProcessState` uses the verified dependency's `spec_pid()`.
// - `RunningProcess`, `InterruptedProcess`, `ZombieProcess` are boundary models.
//
// ## Key Invariants
//
// - A RunnableProcess always has at least one ready thread (`ready_thread_ids@.len() >= 1`).
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
// - `InterruptReason` from run() return is modeled as i64 tag.
// - `UserTda` (user thread data area) from run() return is omitted (HAL boundary).
//
// ## Exec Coverage Notes
//
// The following original functions are NOT modeled as exec functions:
// - `state()` / `state_mut()`: Return `&ProcessState` / `&mut ProcessState`.
//   Verus cannot express these reference return types. The relevant property
//   (PID access) is modeled via `pid_i32()` and `spec_pid()`.
//   **Cross-module verification obligation:** `state_mut()` allows arbitrary
//   mutation of the inner `ProcessState`. Callers must prove that mutations
//   preserve PID immutability (`spec_pid()` unchanged) and any structural
//   invariants assumed by this module. This obligation is discharged when
//   `ProcessState` and its callers are independently verified.
// - `find_thread()` / `find_thread_mut()`: Return `Option<ThreadRef>` containing
//   references into internal collections. Modeled spec-only via `spec_find_thread`.
//   Callers of these functions should independently verify the correctness of
//   the returned reference's properties against the `spec_find_thread` model.
// - `earliest_admission_time()`: Returns `SystemTime` which maps to `i64`.
//   Modeled via `spec_earliest_admission_time()` with proven bounds.
//   The original has a fallback `unwrap_or(clock::now())` for the case when the
//   iterator returns no minimum. This fallback is dead code given the
//   `NonEmptyVecDeque` invariant (the ready list always has at least one element).
//   The spec model correctly omits this unreachable fallback.
//
// ## Oracle Parameter Notes
//
// `terminate()` does not require an oracle parameter. The branch decision
// is computed from exec-level counters (`interrupted_count`, `sleeping_count`)
// that are tied to concrete vector lengths by the `wf()` invariant.
//
// `wakeup()` no longer requires a `found: bool` oracle parameter because
// the sleeping list is now a concrete `Vec<i64>` that can be searched at
// exec level. The search is performed by `vec_search()`.
//
// `run()` has no oracle parameters. The min-index is computed by a concrete
// exec-level loop and proven to match `spec_earliest_ready_index()`.

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
    pub ready_thread_ids: Seq<i64>,
    /// Sequence of ready thread admission times, parallel to ready_thread_ids.
    pub ready_admission_times: Seq<i64>,
    /// Sequence of interrupted thread IDs (may be empty).
    pub interrupted_thread_ids: Seq<i64>,
    /// Sequence of sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Seq<i64>,
    /// Sequence of zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<i64>,
}

/// Abstract view of a RunningProcess (boundary type).
///
/// Note: `ready_admission_times` is intentionally omitted. The exec-level
/// `RunningProcess` does not carry admission times for remaining ready threads.
/// When transitioning back to `RunnableProcess` (e.g., after the running thread
/// blocks), the downstream module must re-supply admission times.
#[verifier::ext_equal]
pub struct RunningProcessView {
    /// Process identifier value.
    pub pid: int,
    /// The running thread ID.
    pub running_thread_id: i64,
    /// Ready thread IDs (may be empty).
    pub ready_thread_ids: Seq<i64>,
    /// Interrupted thread IDs (may be empty).
    pub interrupted_thread_ids: Seq<i64>,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Seq<i64>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<i64>,
    /// Interrupt reason (unconstrained at this abstraction level).
    pub interrupt_reason: i64,
}

/// Abstract view of an InterruptedProcess (boundary type).
#[verifier::ext_equal]
pub struct InterruptedProcessView {
    /// Process identifier value.
    pub pid: int,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Seq<i64>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<i64>,
}

/// Abstract view of a ZombieProcess (boundary type).
#[verifier::ext_equal]
pub struct ZombieProcessView {
    /// Process identifier value.
    pub pid: int,
    /// Zombie thread IDs (non-empty).
    pub zombie_thread_ids: Seq<i64>,
    /// Exit status.
    pub status: i64,
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

/// Spec constant: thread found in the ready list.
pub open spec fn THREAD_REF_READY() -> int { 0 }
/// Spec constant: thread found in the interrupted list.
pub open spec fn THREAD_REF_INTERRUPTED() -> int { 1 }
/// Spec constant: thread found in the sleeping list.
pub open spec fn THREAD_REF_SLEEPING() -> int { 2 }
/// Spec constant: thread found in the zombie list.
pub open spec fn THREAD_REF_ZOMBIE() -> int { 3 }

/// Concrete exit status for interrupted processes (i64 version).
/// Used in exec code where the spec `int` version cannot be used.
pub fn EXIT_STATUS_INTERRUPTED_I64() -> (result: i64)
    ensures result as int == EXIT_STATUS_INTERRUPTED(),
{
    4i64
}

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
    pub open spec fn spec_ready_thread_id(&self, i: int) -> i64
        recommends 0 <= i < self.ready_thread_ids@.len()
    {
        self.ready_thread_ids@[i]
    }

    /// Spec function: returns the i-th ready thread admission time.
    pub open spec fn spec_ready_admission_time(&self, i: int) -> i64
        recommends 0 <= i < self.ready_admission_times@.len()
    {
        self.ready_admission_times@[i]
    }

    /// Spec function: checks if a thread ID is in the ready list.
    pub open spec fn spec_has_ready_thread(&self, tid: i64) -> bool {
        exists|i: int| 0 <= i < self.ready_thread_ids@.len()
            && self.ready_thread_ids@[i] == tid
    }

    /// Spec function: checks if a thread ID is in the sleeping list.
    pub open spec fn spec_has_sleeping_thread(&self, tid: i64) -> bool {
        exists|i: int| 0 <= i < self.sleeping_thread_ids@.len()
            && self.sleeping_thread_ids@[i] == tid
    }

    /// Spec function: checks if a thread ID is in any list.
    pub open spec fn spec_has_thread(&self, tid: i64) -> bool {
        self.spec_has_ready_thread(tid)
        || self.spec_has_sleeping_thread(tid)
        || self.spec_has_interrupted_thread(tid)
        || self.spec_has_zombie_thread(tid)
    }

    /// Spec function: checks if a thread ID is in the interrupted list.
    pub open spec fn spec_has_interrupted_thread(&self, tid: i64) -> bool {
        exists|i: int| 0 <= i < self.interrupted_thread_ids@.len()
            && self.interrupted_thread_ids@[i] == tid
    }

    /// Spec function: checks if a thread ID is in the zombie list.
    pub open spec fn spec_has_zombie_thread(&self, tid: i64) -> bool {
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
    /// The search order matches the original: ready -> interrupted -> sleeping -> zombie.
    /// This models the exhaustive search and correct variant selection.
    pub open spec fn spec_find_thread(&self, tid: i64) -> Option<int> {
        if self.spec_has_ready_thread(tid) {
            Some(THREAD_REF_READY())
        } else if self.spec_has_interrupted_thread(tid) {
            Some(THREAD_REF_INTERRUPTED())
        } else if self.spec_has_sleeping_thread(tid) {
            Some(THREAD_REF_SLEEPING())
        } else if self.spec_has_zombie_thread(tid) {
            Some(THREAD_REF_ZOMBIE())
        } else {
            None
        }
    }

    /// Spec helper: checks if a sequence contains a given value.
    pub open spec fn spec_seq_contains(s: Seq<i64>, tid: i64) -> bool {
        exists|i: int| 0 <= i < s.len() && s[i] == tid
    }

    /// Spec helper: computes the sequence resulting from removing index `idx`
    /// from sequence `s`.
    pub open spec fn spec_remove_at(s: Seq<i64>, idx: int) -> Seq<i64>
        recommends 0 <= idx < s.len()
    {
        s.subrange(0, idx).add(s.subrange(idx + 1, s.len() as int))
    }

    /// Spec function: recursively finds the index of minimum in `s[0..n]`.
    pub open spec fn spec_min_index_rec(s: Seq<i64>, n: int) -> int
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
    pub open spec fn spec_earliest_admission_time(&self) -> i64
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
    /// - Exec-level counters match concrete vector lengths.
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
                ==> #[trigger] self.ready_admission_times@[i] >= 0i64
        // Exec counters match concrete vector lengths.
        &&& self.interrupted_count as nat == self.interrupted_thread_ids@.len()
        &&& self.sleeping_count as nat == self.sleeping_thread_ids@.len()
    }

    /// Spec helper: checks whether two sequences share no common elements.
    pub open spec fn spec_seqs_disjoint(a: Seq<i64>, b: Seq<i64>) -> bool {
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
        self.pid.spec_value()
    }

    /// Spec function: returns the running thread ID.
    pub open spec fn spec_running_thread_id(&self) -> i64 {
        self.running_thread_id
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
        self.pid.spec_value()
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
        self.pid.spec_value()
    }

    /// Spec function: returns the exit status.
    pub open spec fn spec_status(&self) -> int {
        self.status as int
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        self.zombie_thread_ids@.len() >= 1
    }
}

//==================================================================================================
// Abstract State Transitions: RunnableProcessView
//==================================================================================================

impl RunnableProcessView {
    /// View-level well-formedness predicate.
    ///
    /// A RunnableProcessView is well-formed when:
    /// - There is at least one ready thread.
    /// - Ready thread IDs and admission times have equal length.
    /// - All admission times are non-negative.
    pub open spec fn wf(&self) -> bool {
        &&& self.ready_thread_ids.len() >= 1
        &&& self.ready_thread_ids.len() == self.ready_admission_times.len()
        &&& forall|i: int| 0 <= i < self.ready_admission_times.len()
                ==> #[trigger] self.ready_admission_times[i] >= 0i64
    }

    // Note: The helpers `spec_seq_contains`, `spec_remove_at`, and
    // `spec_min_index_rec` are duplicated from `RunnableProcess` because Verus
    // requires them on each impl block. The bridging lemma
    // `lemma_view_min_index_eq` in the proof file proves equivalence for
    // `spec_min_index_rec`; the other two are structurally identical.

    /// View-level helper: checks if a sequence contains a given value.
    pub open spec fn spec_seq_contains(s: Seq<i64>, tid: i64) -> bool {
        exists|i: int| 0 <= i < s.len() && s[i] == tid
    }

    /// View-level helper: removes element at index `idx` from sequence `s`.
    pub open spec fn spec_remove_at(s: Seq<i64>, idx: int) -> Seq<i64>
        recommends 0 <= idx < s.len()
    {
        s.subrange(0, idx).add(s.subrange(idx + 1, s.len() as int))
    }

    /// View-level helper: recursively finds the index of the minimum in `s[0..n]`.
    pub open spec fn spec_min_index_rec(s: Seq<i64>, n: int) -> int
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

    /// View-level helper: finds the index of the ready thread with earliest admission time.
    pub open spec fn spec_earliest_ready_index(&self) -> int
        recommends self.ready_thread_ids.len() > 0
    {
        Self::spec_min_index_rec(
            self.ready_admission_times,
            self.ready_admission_times.len() as int,
        )
    }

    /// View-level helper: selects a witness index for `tid` in sequence `s`.
    pub open spec fn spec_find_index(s: Seq<i64>, tid: i64) -> int
        recommends Self::spec_seq_contains(s, tid)
    {
        choose|i: int| 0 <= i < s.len() && s[i] == tid
    }

    /// Abstract state transition: constructs initial view (models `new()`).
    pub open spec fn spec_new(pid: int, ready_tid: i64, ready_time: i64) -> RunnableProcessView {
        RunnableProcessView {
            pid: pid,
            ready_thread_ids: seq![ready_tid],
            ready_admission_times: seq![ready_time],
            interrupted_thread_ids: Seq::<i64>::empty(),
            sleeping_thread_ids: Seq::<i64>::empty(),
            zombie_thread_ids: Seq::<i64>::empty(),
        }
    }

    /// Abstract state transition: constructs a view from existing state
    /// (models `from_state()`). Used by sibling modules when reconstituting
    /// a `RunnableProcess` after a state transition (e.g., returning from
    /// running to runnable).
    pub open spec fn spec_from_state(
        pid: int,
        ready_ids: Seq<i64>,
        ready_times: Seq<i64>,
        interrupted_ids: Seq<i64>,
        sleeping_ids: Seq<i64>,
        zombie_ids: Seq<i64>,
    ) -> RunnableProcessView {
        RunnableProcessView {
            pid: pid,
            ready_thread_ids: ready_ids,
            ready_admission_times: ready_times,
            interrupted_thread_ids: interrupted_ids,
            sleeping_thread_ids: sleeping_ids,
            zombie_thread_ids: zombie_ids,
        }
    }

    /// Abstract state transition: selects earliest-admission-time thread
    /// and returns `RunningProcessView` (models `run()`).
    ///
    /// Note: `interrupt_reason` is set to `0i64` as a placeholder. The real
    /// `run()` returns an opaque `Option<InterruptReason>` from the thread's
    /// previous state, which is unconstrained at this abstraction level.
    /// Downstream proofs must not rely on this specific value; the bridging
    /// lemma `lemma_run_view_eq` ties the exec result to this spec via a
    /// matching precondition.
    pub open spec fn spec_run(&self) -> RunningProcessView
        recommends self.ready_thread_ids.len() >= 1
    {
        let sel: int = self.spec_earliest_ready_index();
        RunningProcessView {
            pid: self.pid,
            running_thread_id: self.ready_thread_ids[sel],
            ready_thread_ids: Self::spec_remove_at(self.ready_thread_ids, sel),
            interrupted_thread_ids: self.interrupted_thread_ids,
            sleeping_thread_ids: self.sleeping_thread_ids,
            zombie_thread_ids: self.zombie_thread_ids,
            interrupt_reason: 0i64,
        }
    }

    /// Abstract predicate: does terminate produce an InterruptedProcess?
    pub open spec fn spec_terminate_has_interrupted(&self) -> bool {
        self.interrupted_thread_ids.len() > 0 || self.sleeping_thread_ids.len() > 0
    }

    /// Abstract state transition: terminate to `InterruptedProcessView`
    /// (models `terminate()` when interrupted/sleeping threads exist).
    pub open spec fn spec_terminate_to_interrupted(&self) -> InterruptedProcessView
        recommends self.spec_terminate_has_interrupted()
    {
        InterruptedProcessView {
            pid: self.pid,
            interrupted_thread_ids: self.interrupted_thread_ids.add(self.sleeping_thread_ids),
            zombie_thread_ids: self.ready_thread_ids.add(self.zombie_thread_ids),
        }
    }

    /// Abstract state transition: terminate to `ZombieProcessView`
    /// (models `terminate()` when no interrupted/sleeping threads exist).
    pub open spec fn spec_terminate_to_zombie(&self) -> ZombieProcessView
        recommends !self.spec_terminate_has_interrupted()
    {
        ZombieProcessView {
            pid: self.pid,
            zombie_thread_ids: self.ready_thread_ids.add(self.zombie_thread_ids),
            status: EXIT_STATUS_INTERRUPTED() as i64,
        }
    }

    /// Abstract state transition: moves a sleeping thread to the ready queue
    /// (models `wakeup()` success case). The `time` parameter represents the
    /// admission time assigned by `clock_now()` at exec level. The `idx`
    /// parameter is the index of `tid` in the sleeping list, determined by
    /// the concrete search at exec level.
    pub open spec fn spec_wakeup(&self, tid: i64, time: i64, idx: int) -> RunnableProcessView
        recommends
            0 <= idx < self.sleeping_thread_ids.len(),
            self.sleeping_thread_ids[idx] == tid,
    {
        RunnableProcessView {
            pid: self.pid,
            ready_thread_ids: self.ready_thread_ids.push(tid),
            ready_admission_times: self.ready_admission_times.push(time),
            interrupted_thread_ids: self.interrupted_thread_ids,
            sleeping_thread_ids: Self::spec_remove_at(self.sleeping_thread_ids, idx),
            zombie_thread_ids: self.zombie_thread_ids,
        }
    }

    /// Abstract state transition: appends a thread to the ready queue
    /// (models `add_thread()`).
    pub open spec fn spec_add_thread(&self, ready_tid: i64, ready_time: i64) -> RunnableProcessView {
        RunnableProcessView {
            ready_thread_ids: self.ready_thread_ids.push(ready_tid),
            ready_admission_times: self.ready_admission_times.push(ready_time),
            ..*self
        }
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
            pid: self.pid.spec_value(),
            running_thread_id: self.running_thread_id,
            ready_thread_ids: self.ready_thread_ids@,
            interrupted_thread_ids: self.interrupted_thread_ids@,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
            interrupt_reason: self.interrupt_reason,
        }
    }
}

impl View for InterruptedProcess {
    type V = InterruptedProcessView;

    open spec fn view(&self) -> InterruptedProcessView {
        InterruptedProcessView {
            pid: self.pid.spec_value(),
            interrupted_thread_ids: self.interrupted_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

impl View for ZombieProcess {
    type V = ZombieProcessView;

    open spec fn view(&self) -> ZombieProcessView {
        ZombieProcessView {
            pid: self.pid.spec_value(),
            zombie_thread_ids: self.zombie_thread_ids@,
            status: self.status,
        }
    }
}

} // verus!
