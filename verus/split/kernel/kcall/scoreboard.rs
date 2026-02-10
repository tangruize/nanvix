// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # ScoreBoard Verification Model
//!
//! Formal verification of the kernel call scoreboard dispatch protocol.
//!
//! ## Overview
//!
//! The `ScoreBoard` is a rendezvous channel (bounded buffer of size 1) that
//! coordinates kernel call dispatch between user threads and the kernel thread:
//!
//! 1. **Begin dispatch** (user thread): Acquires mutex, sets args, signals
//!    `dispatched` semaphore (up). Phase: Idle → Signaled.
//! 2. **Handle** (kernel thread): Consumes `dispatched` signal (try_down),
//!    reads args. Phase: Signaled → Dispatched.
//! 3. **Handled** (kernel thread): Sets result, signals `handled` semaphore
//!    (up). Phase: Dispatched → Handled.
//! 4. **Complete dispatch** (user thread): Consumes `handled` signal (down),
//!    reads result, releases mutex. Phase: Handled → Idle.
//!
//! ## Verified Properties
//!
//! - Initialization produces a well-formed scoreboard in the Idle phase.
//! - The four-phase protocol (Idle → Signaled → Dispatched → Handled → Idle)
//!   is the only valid transition sequence.
//! - Semaphore signaling is explicitly modeled: `dispatched_value` transitions
//!   0 → 1 (begin_dispatch) → 0 (handle); `handled_value` transitions
//!   0 → 1 (handled) → 0 (complete_dispatch).
//! - Mutual exclusion is held during all active phases (Signaled, Dispatched,
//!   Handled).
//! - Argument integrity: the handler reads exactly the args the dispatcher set.
//! - Result integrity: the dispatcher reads exactly the result the handler set.
//! - The cycle counter is monotonically increasing.
//! - Well-formedness (`wf()`) is preserved by all transitions.
//! - `KcallResult::wf()` enforces that error values fit in i32, matching the
//!   original `KcallError(i32)` payload constraint.
//! - Multiple consecutive cycles produce predictable state (proved inductively).
//! - Different inputs produce observably different outputs (injectivity).
//!
//! ## Verification Model
//!
//! The original implementation uses `Mutex`, `Semaphore` (with `AtomicUsize` and
//! `Condvar`), and a `static mut` global singleton. For verification, we model:
//! - The protocol phase as a four-state enum (`ScoreBoardPhase`).
//! - The mutex as a boolean `locked` field.
//! - The semaphores as integer counters (`dispatched_value`, `handled_value`)
//!   that faithfully track signal/consume transitions.
//! - All state transitions via `&mut self` methods.
//!
//! This is a sequential model that verifies the state machine protocol (phase
//! transitions, data flow, mutual exclusion, semaphore signaling) without
//! reasoning about atomicity, memory ordering, or concurrent thread scheduling.
//!
//! **Note on field visibility:** All struct fields are `pub` because Verus's
//! `verus!{}` macro requires field access for spec functions and View trait
//! implementations. This is standard practice in Verus verification models
//! (matching the existing verified modules: mutex, semaphore, slab, etc.).
//! These are verification model structs, not production API types; the original
//! struct fields are encapsulated in the production code.
//!
//! ## API Mapping
//!
//! | Original API              | Verified Model          | Notes                            |
//! |---------------------------|-------------------------|----------------------------------|
//! | `ScoreBoard::init()`      | `ScoreBoardSlot::init()`| Idempotent; allows re-init.      |
//! | `ScoreBoard::get_mut()`   | `try_get_board()` /     | Bool check + immutable ref.      |
//! |                           | `get_board()`           | `&mut` return unsupported by     |
//! |                           |                         | Verus; use slot field directly.  |
//! | `ScoreBoard::dispatch()`  | `dispatch()` /          | Full cycle with error paths.     |
//! |                           | `try_begin_dispatch()` +| Split API also available.        |
//! |                           | `complete_dispatch()` / |                                  |
//! |                           | `abandon_dispatch()`    |                                  |
//! | `ScoreBoard::handle()`    | `handle()`/`try_handle()`| With and without error path.    |
//! | `ScoreBoard::handled()`   | `handled()`             | Direct mapping.                  |
//! | *(no original)*           | `get_args()`            | Reference access after handle(). |
//! | *(no original)*           | `completed_cycles`      | Ghost<nat> verification counter. |
//! | `impl Debug for KcallArgs`| *(not modeled)*         | Formatting; out of scope.        |
//! | `pub fn init()`           | *(not modeled)*         | Logging wrapper; out of scope.   |
//!
//! ## API Divergence
//!
//! - `dispatch()` is a verified wrapper that composes the split API into a single
//!   function matching the original `dispatch()` signature. It returns a
//!   `DispatchOutcome` enum modeling `Result<KcallResult, SleepError>` with three
//!   cases: `LockFailed`, `DownInterrupted`, and `Success(KcallResult)`. The handler
//!   steps (handle + handled) are inlined to model the full protocol cycle. The
//!   split API (`try_begin_dispatch`, `complete_dispatch`, `abandon_dispatch`)
//!   remains available for fine-grained reasoning about individual phases.
//! - `handle()` takes `&mut self` (original takes `&self` with atomic try_down).
//!   Returns `Ghost<KcallArgsView>` (original returns `Result<&KcallArgs, Error>`).
//!   The `&mut self` is required to model the dispatched signal consumption.
//!   A separate `get_args(&self)` provides reference access after the transition.
//! - `try_handle()` models the error path: returns `false` when no dispatch is
//!   pending (matching `ErrorCode::TryAgain`), proving state preservation on failure.
//! - `try_begin_dispatch()` models lock acquisition failure: returns `false` when
//!   the lock cannot be acquired (matching `SleepError::Interrupted`), proving
//!   state preservation on failure.
//! - `abandon_dispatch()` models `handled.down()` interruption: releases the
//!   mutex and leaves the board in a characterized stuck state. This is the
//!   liveness-critical error path documented in T5.
//! - `KcallResult` uses `is_success: bool` + `value: i64` (original uses an enum
//!   with `Success(KcallSuccess(i64))` / `Error(KcallError(i32))`).
//! - `completed_cycles: Ghost<nat>` is verification-only ghost state. The original
//!   has no cycle counter. This field tracks protocol progress for inductive proofs.
//!   As a ghost field, it is erased at runtime and introduces no semantic divergence.
//! - `ScoreBoardSlot` models the `Option<ScoreBoard>` global pattern as a regular
//!   struct with an `initialized` flag, avoiding `static mut` and `unsafe`.
//!   Supports idempotent re-initialization matching the original's overwrite behavior.
//!
//! ## Trust Boundaries
//!
//! - **T1: Global singleton.** The `ScoreBoardSlot` models the initialization/access
//!   pattern (`init()` / `get_mut()`). The `try_get_board()` function models the
//!   `ErrorCode::TryAgain` error when uninitialized. `get_board()` provides an
//!   immutable reference when initialized, connecting slot state to scoreboard
//!   operations. The original `get_mut()` returns `&mut ScoreBoard`, but Verus does
//!   not support `&mut` return types; use the slot's `board` field directly for
//!   mutable access. The `static mut` memory safety and lifetime guarantees remain
//!   unverified (Verus cannot reason about `static mut`).
//! - **T2: Mutex correctness.** The model assumes the mutex provides mutual exclusion.
//!   The mutex is separately verified in `kernel::pm::sync::mutex`.
//! - **T3: Semaphore correctness.** The model assumes the semaphores correctly
//!   implement counting and blocking. The semaphore is separately verified in
//!   `kernel::pm::sync::semaphore`. The `up()` operations are proven to have
//!   their *value precondition* satisfied by `lemma_semaphore_up_dispatched_cannot_fail`
//!   and `lemma_semaphore_up_handled_cannot_fail` (semaphore value is always 0
//!   before `up()` is called, so overflow is impossible). Whether `up()` can
//!   fail through internal mechanisms (e.g., `ProcessManager::wakeup()` returning
//!   an error in `notify_first()`) is a property of the semaphore implementation;
//!   the original maps such failures via `map_err(SleepError::Generic)`, which is
//!   propagated through `dispatch()` (modeled by `DispatchOutcome::DownInterrupted`
//!   for the `handled.down()` path; the `dispatched.up()` error is a T3 trust
//!   boundary concern).
//! - **T4: Sequential ordering.** The model assumes sequential execution. The
//!   original relies on mutex + semaphore for thread synchronization.
//!
//!   **Refinement argument:** The sequential model is a sound abstraction of the
//!   concurrent implementation because:
//!   1. The Mutex (separately verified) guarantees that at most one dispatcher
//!      thread accesses the scoreboard fields at a time.
//!   2. The Semaphore pair (separately verified) enforces a strict
//!      signal-then-wait ordering: the dispatcher signals `dispatched` and waits
//!      on `handled`; the handler waits on `dispatched` and signals `handled`.
//!      This creates a total order on field accesses within each cycle.
//!   3. Therefore, all field reads and writes within a single cycle occur in a
//!      fixed sequential order (the same order modeled here), and no two cycles
//!      can overlap (due to the mutex).
//!   4. The only concurrent access is `dispatched.try_down()` in `handle()`,
//!      which is a non-blocking atomic operation modeled by `try_handle()`.
//!
//!   **Note on `handle(&self)`:** The original `handle()` takes `&self` and uses
//!   atomic `try_down()`, meaning it can run concurrently without exclusive access.
//!   The verified model uses `&mut self`, which means the mutual exclusion between
//!   `handle()` and `dispatch()` is *assumed* (via the sequential model), not
//!   *proven*. The semaphore's atomic try_down guarantees this in practice, but
//!   proving it would require a concurrent program logic.
//!
//!   A full mechanized refinement proof would require a concurrent program logic
//!   (e.g., Iris or RustBelt) beyond Verus's current capabilities.
//! - **T5: Error handling.** Error paths are modeled as follows:
//!   - `lock()` failure: modeled by `try_begin_dispatch()` returning false.
//!     State preservation proven.
//!   - `try_down()` failure: modeled by `try_handle()` returning false.
//!     State preservation proven.
//!   - `dispatched.up()` failure: proven impossible by
//!     `lemma_semaphore_up_dispatched_cannot_fail` (value is 0 in Idle phase).
//!   - `handled.up()` failure: proven impossible by
//!     `lemma_semaphore_up_handled_cannot_fail` (value is 0 in Dispatched phase).
//!   - `handled.down()` interruption (`SleepError::Interrupted`): modeled by
//!     `abandon_dispatch()`. Accepts any active phase (Signaled, Dispatched,
//!     or Handled), producing a stuck state (active phase + unlocked) that
//!     violates `wf()`, formally characterizing the liveness failure. Data
//!     preservation is proven by `lemma_abandon_dispatch_preserves_data`.
//!   - `get_mut()` on uninitialized: modeled by `try_get_board()` returning
//!     false.
//!
//! ## Verification Scope
//!
//! This verification proves **sequential state machine correctness** of the
//! scoreboard dispatch protocol, including error path behavior. The following
//! are explicitly **out of scope**:
//! - Concurrent thread scheduling (Verus limitation; see T4 for refinement argument).
//! - The `static mut` memory safety (see T1; init/access pattern is modeled).
//! - The dispatcher and handler modules (`dispatcher.rs`, `handler.rs`).
//! - `impl Debug for KcallArgs` (formatting; no safety implications).
//! - Module-level `pub fn init()` (logging wrapper around `ScoreBoard::init()`).

use vstd::prelude::*;

// Include specifications.
include!("scoreboard.spec.rs");

// Include proofs.
include!("scoreboard.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// Kernel call arguments passed from user threads to the kernel thread.
///
/// # Description
///
/// Contains the process/thread identifiers, kernel call number, and up to
/// four arguments. In the original, `ProcessIdentifier` and `ThreadIdentifier`
/// are `#[repr(C)]` wrappers around `i32`. Here they are modeled as plain `i32`.
/// `ProcessIdentifier` and `ThreadIdentifier` accept any `i32` via `From<i32>`,
/// so no non-negativity restriction is imposed.
pub struct KcallArgs {
    /// Process identifier of the calling process.
    pub pid: i32,
    /// Thread identifier of the calling thread.
    pub tid: i32,
    /// Kernel call number.
    pub number: u32,
    /// First kernel call argument.
    pub arg0: u32,
    /// Second kernel call argument.
    pub arg1: u32,
    /// Third kernel call argument.
    pub arg2: u32,
    /// Fourth kernel call argument.
    pub arg3: u32,
}

/// Kernel call result returned from the kernel thread to user threads.
///
/// # Description
///
/// In the original, `KcallResult` is an enum with `Success(KcallSuccess(i64))`
/// and `Error(KcallError(i32))`. For verification, we model the variant tag as
/// `is_success: bool` and the payload as `value: i64`. Error payloads must fit
/// in i32 (matching `KcallError(i32)`).
///
/// The original `KcallResult` derives `Copy`, so `Ok(self.ret)` in `dispatch()`
/// returns a copy without moving. This flat struct model inherently has copy
/// semantics in Verus, matching the original behavior.
#[derive(PartialEq, Eq)]
pub struct KcallResult {
    /// Whether this result represents the Success variant.
    pub is_success: bool,
    /// Payload value: any i64 for success, must fit i32 for error.
    pub value: i64,
}

/// Outcome of a full dispatch cycle, modeling `Result<KcallResult, SleepError>`.
///
/// # Description
///
/// Captures the three observable outcomes of the original `dispatch()`:
/// - `LockFailed`: `lock()` returned `Err(SleepError::Interrupted)`.
/// - `DownInterrupted`: `handled.down()` returned `Err(SleepError::Interrupted)`.
/// - `Success(KcallResult)`: `Ok(self.ret)` with the handler's result.
#[derive(PartialEq, Eq)]
pub enum DispatchOutcome {
    /// Lock acquisition failed (models `SleepError::Interrupted` from `lock()`).
    LockFailed,
    /// `handled.down()` was interrupted (models `SleepError::Interrupted` from `down()`).
    DownInterrupted,
    /// Dispatch completed successfully with the handler's result.
    Success(KcallResult),
}

/// The scoreboard: a rendezvous channel for kernel call dispatch.
///
/// # Description
///
/// Coordinates kernel call dispatch between user threads (dispatchers)
/// and the kernel thread (handler). The protocol is a four-phase handshake:
/// Idle → Signaled → Dispatched → Handled → Idle.
///
/// In the original, synchronization uses `Mutex`, two `Semaphore`s, and
/// a `static mut` global. The verified model uses plain fields with
/// `&mut self` state transitions, faithfully tracking semaphore signal
/// and consume operations.
pub struct ScoreBoard {
    /// Whether the mutex is currently held.
    pub locked: bool,
    /// Dispatched semaphore value: 0 (idle/dispatched/handled) or 1 (signaled).
    pub dispatched_value: u8,
    /// Handled semaphore value: 0 (idle/signaled/dispatched) or 1 (handled).
    pub handled_value: u8,
    /// Current kernel call arguments.
    pub args: KcallArgs,
    /// Current kernel call result.
    pub result: KcallResult,
    /// Current protocol phase.
    pub phase: ScoreBoardPhase,
    /// Count of completed cycles (verification-only; no original counterpart).
    /// Ghost field: exists only at the spec level, erased at runtime.
    /// Uses `Ghost<nat>` so no overflow guard is needed (nat is unbounded).
    pub completed_cycles: Ghost<nat>,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl KcallArgs {
    /// Creates new kernel call arguments.
    ///
    /// # Parameters
    ///
    /// - `pid`: Process identifier (any i32; `ProcessIdentifier` accepts full range).
    /// - `tid`: Thread identifier (any i32; `ThreadIdentifier` accepts full range).
    /// - `number`: Kernel call number.
    /// - `arg0`: First argument.
    /// - `arg1`: Second argument.
    /// - `arg2`: Third argument.
    /// - `arg3`: Fourth argument.
    ///
    /// # Returns
    ///
    /// A new `KcallArgs` with the specified values.
    pub fn new(
        pid: i32,
        tid: i32,
        number: u32,
        arg0: u32,
        arg1: u32,
        arg2: u32,
        arg3: u32,
    ) -> (result: Self)
        ensures
            result.pid == pid,
            result.tid == tid,
            result.number == number,
            result.arg0 == arg0,
            result.arg1 == arg1,
            result.arg2 == arg2,
            result.arg3 == arg3,
            result@ == (KcallArgsView {
                pid: pid as int,
                tid: tid as int,
                number: number as nat,
                arg0: arg0 as nat,
                arg1: arg1 as nat,
                arg2: arg2 as nat,
                arg3: arg3 as nat,
            }),
    {
        KcallArgs { pid, tid, number, arg0, arg1, arg2, arg3 }
    }
}

impl KcallResult {
    /// Creates a successful (ok) result with value 0.
    ///
    /// # Returns
    ///
    /// A `KcallResult` representing success with value 0.
    pub fn ok() -> (result: Self)
        ensures
            result.is_success,
            result.value == 0,
            result.wf(),
            result@ == KcallResult::spec_ok_view(),
    {
        KcallResult { is_success: true, value: 0 }
    }

    /// Creates a success result with a specific value.
    ///
    /// # Parameters
    ///
    /// - `value`: The success payload (any i64).
    ///
    /// # Returns
    ///
    /// A `KcallResult` success variant with the specified value.
    pub fn success(value: i64) -> (result: Self)
        ensures
            result.is_success,
            result.value == value,
            result.wf(),
            result@ == (KcallResultView { is_success: true, value: value as int }),
    {
        KcallResult { is_success: true, value }
    }

    /// Creates an error result with a specific error code.
    ///
    /// # Parameters
    ///
    /// - `code`: The error code (must fit in i32, matching `KcallError(i32)`).
    ///
    /// # Returns
    ///
    /// A `KcallResult` error variant with the specified code.
    pub fn error(code: i32) -> (result: Self)
        ensures
            !result.is_success,
            result.value == code as i64,
            result.wf(),
            result@ == (KcallResultView { is_success: false, value: code as int }),
    {
        KcallResult { is_success: false, value: code as i64 }
    }
}

impl ScoreBoard {
    /// Creates a new scoreboard in the Idle phase.
    ///
    /// # Description
    ///
    /// Corresponds to `ScoreBoard::init()` in the original. Creates the
    /// scoreboard with default arguments, ok result, unlocked mutex, and
    /// both semaphores at 0.
    ///
    /// # Returns
    ///
    /// A new `ScoreBoard` in the Idle phase.
    pub fn new() -> (result: Self)
        ensures
            result.wf(),
            result.spec_is_idle(),
            !result.locked,
            result.dispatched_value == 0,
            result.handled_value == 0,
            result.completed_cycles@ == 0nat,
            result@ == ScoreBoard::spec_initial_view(),
    {
        ScoreBoard {
            locked: false,
            dispatched_value: 0,
            handled_value: 0,
            args: KcallArgs {
                pid: i32::MAX,
                tid: i32::MAX,
                number: 0,
                arg0: 0,
                arg1: 0,
                arg2: 0,
                arg3: 0,
            },
            result: KcallResult::ok(),
            phase: ScoreBoardPhase::Idle,
            completed_cycles: Ghost(0nat),
        }
    }

    /// Begins a dispatch: acquires mutex, sets arguments, signals handler.
    ///
    /// # Description
    ///
    /// Models the first part of the original `dispatch()`:
    /// 1. Acquires the mutex (modeled by setting `locked = true`).
    /// 2. Stores the kernel call arguments.
    /// 3. Signals the handler via `dispatched.up()` (sets `dispatched_value = 1`).
    ///
    /// The `dispatched.up()` call cannot fail because the semaphore value is 0
    /// in the Idle phase (proven by `lemma_semaphore_up_dispatched_cannot_fail`).
    ///
    /// Phase transitions from `Idle` to `Signaled`. The handler must call
    /// `handle()` to consume the signal before the protocol can proceed.
    ///
    /// # Parameters
    ///
    /// - `args`: The kernel call arguments to dispatch.
    ///
    /// # Returns
    ///
    /// Ghost copy of the arguments view, for spec-level integrity checking.
    pub fn begin_dispatch(&mut self, args: KcallArgs) -> (result: Ghost<KcallArgsView>)
        requires
            old(self).wf(),
            old(self).spec_is_idle(),
        ensures
            self.wf(),
            self.spec_is_signaled(),
            self.locked,
            self.dispatched_value == 1,
            self.args@ == args@,
            self@ == ScoreBoard::spec_begin_dispatch(old(self)@, args@),
            result@ == args@,
    {
        self.locked = true;
        self.args = args;
        self.dispatched_value = 1;
        self.phase = ScoreBoardPhase::Signaled;
        Ghost(self.args@)
    }

    /// Handles a dispatched kernel call: consumes signal, reads arguments.
    ///
    /// # Description
    ///
    /// Models the original `handle()`:
    /// 1. Consumes the dispatched signal via `dispatched.try_down()`
    ///    (sets `dispatched_value = 0`).
    /// 2. The handler reads the arguments (accessible via `get_args()`).
    ///
    /// Phase transitions from `Signaled` to `Dispatched`. The handler
    /// should process the call and then invoke `handled()`.
    ///
    /// Note: Takes `&mut self` (original takes `&self` with atomic try_down).
    /// Returns `Ghost<KcallArgsView>` since `&mut self` prevents returning
    /// a reference. Use `get_args()` for reference access after this call.
    ///
    /// # Returns
    ///
    /// Ghost copy of the arguments view.
    pub fn handle(&mut self) -> (result: Ghost<KcallArgsView>)
        requires
            old(self).wf(),
            old(self).spec_is_signaled(),
        ensures
            self.wf(),
            self.spec_is_dispatched(),
            self.dispatched_value == 0,
            self.args@ == old(self).args@,
            self@ == ScoreBoard::spec_handle(old(self)@),
            result@ == self.args@,
    {
        self.dispatched_value = 0;
        self.phase = ScoreBoardPhase::Dispatched;
        Ghost(self.args@)
    }

    /// Attempts to handle a dispatched kernel call; models try_down success/failure.
    ///
    /// # Description
    ///
    /// Models the original `handle()` with its `try_down()` error path:
    /// - If the phase is `Signaled` (dispatched semaphore > 0), the signal is
    ///   consumed and the phase transitions to `Dispatched`. Returns `true`.
    /// - If the phase is not `Signaled` (dispatched semaphore == 0), no state
    ///   change occurs. Returns `false` (models `ErrorCode::TryAgain`).
    ///
    /// This proves that the error path preserves the well-formedness invariant
    /// and does not corrupt scoreboard state.
    ///
    /// # Returns
    ///
    /// `true` if a dispatch was pending and consumed; `false` otherwise.
    pub fn try_handle(&mut self) -> (success: bool)
        requires
            old(self).wf(),
        ensures
            success == old(self).spec_is_signaled(),
            success ==> (
                self.wf()
                && self.spec_is_dispatched()
                && self.dispatched_value == 0
                && self.args@ == old(self).args@
                && self@ == ScoreBoard::spec_handle(old(self)@)
            ),
            !success ==> self@ == old(self)@,
    {
        if matches!(self.phase, ScoreBoardPhase::Signaled) {
            self.dispatched_value = 0;
            self.phase = ScoreBoardPhase::Dispatched;
            true
        } else {
            false
        }
    }

    /// Attempts to begin a dispatch; models lock() success/failure.
    ///
    /// # Description
    ///
    /// Models the original `dispatch()` lock acquisition with error path:
    /// - If `lock_acquired` is true: acquires mutex, sets args, signals handler.
    ///   Phase transitions from Idle to Signaled (same as `begin_dispatch`).
    /// - If `lock_acquired` is false: models `lock()` returning
    ///   `Err(SleepError::Interrupted)`. No state change occurs.
    ///
    /// This proves that the lock failure error path preserves the well-formedness
    /// invariant and does not corrupt scoreboard state.
    ///
    /// # Parameters
    ///
    /// - `args`: The kernel call arguments to dispatch.
    /// - `lock_acquired`: Whether the lock was successfully acquired.
    ///
    /// # Returns
    ///
    /// `true` if dispatch began successfully; `false` on lock failure.
    pub fn try_begin_dispatch(&mut self, args: KcallArgs, lock_acquired: bool) -> (success: bool)
        requires
            old(self).wf(),
            old(self).spec_is_idle(),
        ensures
            success == lock_acquired,
            success ==> (
                self.wf()
                && self.spec_is_signaled()
                && self.locked
                && self.dispatched_value == 1
                && self.args@ == args@
                && self@ == ScoreBoard::spec_begin_dispatch(old(self)@, args@)
            ),
            !success ==> self@ == old(self)@,
    {
        if lock_acquired {
            self.locked = true;
            self.args = args;
            self.dispatched_value = 1;
            self.phase = ScoreBoardPhase::Signaled;
            true
        } else {
            false
        }
    }

    /// Abandons a dispatch cycle after an error interrupts an active phase.
    ///
    /// # Description
    ///
    /// Models error paths where the mutex guard drops during an active phase:
    /// - `Signaled`: `handled.down()` interrupted before handler consumed signal.
    /// - `Dispatched`: `handled.down()` interrupted while handler is processing.
    /// - `Handled`: `handled.down()` interrupted after handler completed.
    ///
    /// In all cases, the mutex guard drops, releasing the lock. The scoreboard
    /// is left in its current phase with the mutex unlocked, violating `wf()`.
    /// This reflects a genuine stuck state in the original implementation.
    pub fn abandon_dispatch(&mut self)
        requires
            old(self).wf(),
            !old(self).spec_is_idle(),
        ensures
            !self.locked,
            self.phase == old(self).phase,
            self.result@ == old(self).result@,
            self.args@ == old(self).args@,
            self.dispatched_value == old(self).dispatched_value,
            self.handled_value == old(self).handled_value,
            self@ == ScoreBoard::spec_abandon_dispatch(old(self)@),
    {
        self.locked = false;
    }

    /// Returns a reference to the current kernel call arguments.
    ///
    /// # Description
    ///
    /// Provides reference access to the arguments after `handle()` has
    /// consumed the dispatched signal. This is separated from `handle()`
    /// because `handle()` requires `&mut self` (to model signal consumption)
    /// and cannot return a reference.
    ///
    /// # Returns
    ///
    /// Reference to the current `KcallArgs`.
    pub fn get_args(&self) -> (result: &KcallArgs)
        requires
            self.wf(),
            self.spec_is_dispatched(),
        ensures
            (*result)@ == self.args@,
    {
        &self.args
    }

    /// Signals that the kernel call has been handled and stores the result.
    ///
    /// # Description
    ///
    /// Models the original `handled()`:
    /// 1. Stores the kernel call result.
    /// 2. Signals the dispatcher via `handled.up()` (sets `handled_value = 1`).
    ///
    /// Phase transitions from `Dispatched` to `Handled`.
    ///
    /// # Parameters
    ///
    /// - `ret`: The kernel call result.
    pub fn handled(&mut self, ret: KcallResult)
        requires
            old(self).wf(),
            old(self).spec_is_dispatched(),
            ret.wf(),
        ensures
            self.wf(),
            self.spec_is_handled(),
            self.handled_value == 1,
            self.result@ == ret@,
            self.args@ == old(self).args@,
            self@ == ScoreBoard::spec_handled(old(self)@, ret@),
    {
        self.result = ret;
        self.handled_value = 1;
        self.phase = ScoreBoardPhase::Handled;
    }

    /// Completes a dispatch cycle: reads the result and releases the mutex.
    ///
    /// # Description
    ///
    /// Models the second half of the original `dispatch()`:
    /// 1. Consumes the handled signal via `handled.down()`
    ///    (sets `handled_value = 0`).
    /// 2. Reads the result.
    /// 3. Releases the mutex (guard drop).
    ///
    /// Phase transitions from `Handled` to `Idle`.
    ///
    /// # Returns
    ///
    /// Copy of the kernel call result (matches original `Ok(self.ret)`).
    pub fn complete_dispatch(&mut self) -> (result: KcallResult)
        requires
            old(self).wf(),
            old(self).spec_is_handled(),
        ensures
            self.wf(),
            self.spec_is_idle(),
            !self.locked,
            self.dispatched_value == 0,
            self.handled_value == 0,
            self.result@ == old(self).result@,
            result@ == old(self).result@,
            result.wf(),
            self@ == ScoreBoard::spec_complete_dispatch(old(self)@),
            self.completed_cycles@ == old(self).completed_cycles@ + 1,
    {
        let ret: KcallResult = KcallResult { is_success: self.result.is_success, value: self.result.value };
        self.handled_value = 0;
        self.locked = false;
        self.completed_cycles = Ghost(self.completed_cycles@ + 1);
        self.phase = ScoreBoardPhase::Idle;
        ret
    }

    /// Performs a complete dispatch cycle, modeling the original `dispatch()`.
    ///
    /// # Description
    ///
    /// Composes the split API to model the original `dispatch()` which returns
    /// `Result<KcallResult, SleepError>`. The method handles three cases:
    ///
    /// 1. **Lock failure** (`lock_acquired == false`): Models `lock()` returning
    ///    `Err(SleepError::Interrupted)`. Returns `DispatchOutcome::LockFailed`.
    ///    State is preserved.
    /// 2. **Down interrupted** (`down_interrupted == true`): Models `handled.down()`
    ///    returning `Err(SleepError::Interrupted)`. The interrupt is modeled from
    ///    the Signaled phase (before the handler runs), which is the worst case.
    ///    In the real implementation, `handled.down()` blocks until the handler
    ///    signals, so the interrupt can occur while the board is in any active
    ///    phase (Signaled, Dispatched, or Handled). Modeling from Signaled
    ///    captures the most conservative stuck state. The `abandon_dispatch()`
    ///    function covers interruption from other active phases separately.
    ///    Returns `DispatchOutcome::DownInterrupted`.
    /// 3. **Success**: Lock acquired and `handled.down()` succeeds. Returns
    ///    `DispatchOutcome::Success(result)`, modeling `Ok(self.ret)`.
    ///
    /// The parameters model nondeterministic environmental choices in a
    /// sequential verification setting:
    /// - `lock_acquired`: Whether the OS scheduler granted the lock (environment).
    /// - `down_interrupted`: Whether the OS interrupted the blocking `down()` call.
    /// - `ret`: The handler thread's eventual result (determined by the handler,
    ///   not the dispatcher). Only meaningful on the success path.
    ///
    /// # Parameters
    ///
    /// - `args`: The kernel call arguments to dispatch.
    /// - `ret`: The kernel call result (set by the handler; environment input).
    /// - `lock_acquired`: Whether the lock was successfully acquired (environment).
    /// - `down_interrupted`: Whether `handled.down()` was interrupted (environment).
    ///
    /// # Returns
    ///
    /// `DispatchOutcome` modeling the original `Result<KcallResult, SleepError>`.
    pub fn dispatch(
        &mut self,
        args: KcallArgs,
        ret: KcallResult,
        lock_acquired: bool,
        down_interrupted: bool,
    ) -> (outcome: DispatchOutcome)
        requires
            old(self).wf(),
            old(self).spec_is_idle(),
            ret.wf(),
        ensures
            // Lock failure: state preserved.
            !lock_acquired ==> (
                outcome == DispatchOutcome::LockFailed
                && self@ == old(self)@
                && self.wf()
            ),
            // Down interrupted: stuck state from Signaled phase.
            // Args are preserved, result unchanged, semaphore state known.
            (lock_acquired && down_interrupted) ==> (
                outcome == DispatchOutcome::DownInterrupted
                && !self.locked
                && self.phase == ScoreBoardPhase::Signaled
                && self.args@ == args@
                && self.result@ == old(self).result@
                && self.dispatched_value == 1
                && self.handled_value == 0
                && self.completed_cycles@ == old(self).completed_cycles@
                && self@ == ScoreBoard::spec_dispatch_interrupted(old(self)@, args@)
            ),
            // Success: full cycle completed, result returned.
            (lock_acquired && !down_interrupted) ==> (
                outcome == DispatchOutcome::Success(KcallResult { is_success: ret.is_success, value: ret.value })
                && self.wf()
                && self.spec_is_idle()
                && !self.locked
                && self.completed_cycles@ == old(self).completed_cycles@ + 1
                && self@ == ScoreBoard::spec_dispatch_success(old(self)@, args@, ret@)
            ),
    {
        if !lock_acquired {
            return DispatchOutcome::LockFailed;
        }
        // Begin dispatch: acquire lock, set args, signal handler.
        self.locked = true;
        self.args = args;
        self.dispatched_value = 1;
        self.phase = ScoreBoardPhase::Signaled;

        if down_interrupted {
            // handled.down() interrupted: mutex guard drops from Signaled phase.
            // The handler has not run yet; args are preserved, result unchanged.
            self.locked = false;
            return DispatchOutcome::DownInterrupted;
        }

        // Handler processes: consume dispatched signal, set result, signal handled.
        self.dispatched_value = 0;
        self.phase = ScoreBoardPhase::Dispatched;
        self.result = ret;
        self.handled_value = 1;
        self.phase = ScoreBoardPhase::Handled;

        // Complete: consume handled signal, read result, release mutex.
        let dispatch_result: KcallResult = KcallResult {
            is_success: self.result.is_success,
            value: self.result.value,
        };
        self.handled_value = 0;
        self.locked = false;
        self.completed_cycles = Ghost(self.completed_cycles@ + 1);
        self.phase = ScoreBoardPhase::Idle;
        DispatchOutcome::Success(dispatch_result)
    }

    /// Returns whether the scoreboard is in the idle phase.
    ///
    /// # Returns
    ///
    /// `true` if the scoreboard is idle.
    pub fn is_idle(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self.spec_is_idle(),
    {
        matches!(self.phase, ScoreBoardPhase::Idle)
    }

    /// Returns whether the scoreboard is in the signaled phase.
    ///
    /// # Returns
    ///
    /// `true` if the scoreboard has a pending signal for the handler.
    pub fn is_signaled(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self.spec_is_signaled(),
    {
        matches!(self.phase, ScoreBoardPhase::Signaled)
    }

    /// Returns whether the scoreboard is in the dispatched phase.
    ///
    /// # Returns
    ///
    /// `true` if the handler has consumed the signal and is processing.
    pub fn is_dispatched(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self.spec_is_dispatched(),
    {
        matches!(self.phase, ScoreBoardPhase::Dispatched)
    }

    /// Returns whether the scoreboard is in the handled phase.
    ///
    /// # Returns
    ///
    /// `true` if the handler has finished and the result is ready.
    pub fn is_handled(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self.spec_is_handled(),
    {
        matches!(self.phase, ScoreBoardPhase::Handled)
    }
}

//==================================================================================================
// ScoreBoardSlot
//==================================================================================================

/// Models the global `static mut SCOREBOARD: Option<ScoreBoard>` pattern.
///
/// # Description
///
/// The original kernel uses a `static mut Option<ScoreBoard>` with unsafe access.
/// `ScoreBoardSlot` models this initialization/access pattern:
/// - Before `init()`: the slot is uninitialized, and `try_get_board()` returns false
///   (modeling `get_mut()` returning `Err(ErrorCode::TryAgain)`).
/// - After `init()`: the slot is initialized with a well-formed idle scoreboard,
///   and `try_get_board()` returns true (modeling `get_mut()` returning `Ok(&mut sb)`).
pub struct ScoreBoardSlot {
    /// Whether the scoreboard has been initialized.
    pub initialized: bool,
    /// The scoreboard (only meaningful when `initialized` is true).
    pub board: ScoreBoard,
}

impl ScoreBoardSlot {
    /// Creates a new uninitialized scoreboard slot.
    ///
    /// # Description
    ///
    /// Models the initial state of `static mut SCOREBOARD: Option<ScoreBoard> = None`.
    ///
    /// # Returns
    ///
    /// An uninitialized `ScoreBoardSlot`.
    pub fn new() -> (result: Self)
        ensures
            !result.spec_is_initialized(),
            result.wf(),
            result@ == ScoreBoardSlot::spec_initial_slot_view(),
    {
        ScoreBoardSlot {
            initialized: false,
            board: ScoreBoard::new(),
        }
    }

    /// Initializes (or re-initializes) the scoreboard slot.
    ///
    /// # Description
    ///
    /// Models `ScoreBoard::init()` which sets `SCOREBOARD = Some(ScoreBoard { ... })`.
    /// The original unconditionally overwrites the global, so this accepts any
    /// prior state (including already-initialized slots) for idempotent re-init.
    pub fn init(&mut self)
        ensures
            self.spec_is_initialized(),
            self.wf(),
            self.board.wf(),
            self.board.spec_is_idle(),
            !self.board.locked,
            self.board.completed_cycles@ == 0nat,
    {
        self.board = ScoreBoard::new();
        self.initialized = true;
    }

    /// Checks whether the scoreboard has been initialized.
    ///
    /// # Description
    ///
    /// Models the `SCOREBOARD.as_mut()` check in `get_mut()`.
    ///
    /// # Returns
    ///
    /// `true` if initialized (modeling `Some`), `false` otherwise (`None`).
    pub fn is_initialized(&self) -> (result: bool)
        ensures
            result == self.spec_is_initialized(),
    {
        self.initialized
    }

    /// Attempts to access the scoreboard; models `get_mut()` error behavior.
    ///
    /// # Description
    ///
    /// Models `ScoreBoard::get_mut()`:
    /// - Returns `true` if initialized (modeling `Ok(&mut sb)`).
    /// - Returns `false` if uninitialized (modeling `Err(ErrorCode::TryAgain)`).
    ///
    /// When returning `true`, the board is guaranteed well-formed.
    ///
    /// # Returns
    ///
    /// `true` if the scoreboard is accessible; `false` with TryAgain semantics.
    pub fn try_get_board(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self.spec_is_initialized(),
            result ==> self.board.wf(),
    {
        self.initialized
    }

    /// Returns a reference to the contained scoreboard.
    ///
    /// # Description
    ///
    /// Models the success path of `get_mut()`: when the slot is initialized,
    /// provides a reference to the well-formed scoreboard. This connects
    /// the slot's initialization state with the ability to perform scoreboard
    /// operations (dispatch, handle, handled, complete_dispatch).
    ///
    /// # Returns
    ///
    /// Reference to the contained `ScoreBoard`.
    pub fn get_board(&self) -> (result: &ScoreBoard)
        requires
            self.wf(),
            self.spec_is_initialized(),
        ensures
            (*result).wf(),
            (*result)@ == self.board@,
    {
        &self.board
    }
}

} // verus!
