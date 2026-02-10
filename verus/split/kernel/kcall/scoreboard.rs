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
//! ## API Mapping
//!
//! | Original API              | Verified Model          | Notes                            |
//! |---------------------------|-------------------------|----------------------------------|
//! | `ScoreBoard::init()`      | `new()`                 | Returns struct instead of global.|
//! | `ScoreBoard::get_mut()`   | *(not modeled)*         | Global access; see Trust T1.     |
//! | `ScoreBoard::dispatch()`  | `begin_dispatch()` +    | Split into four phases for       |
//! |                           | `complete_dispatch()`   | handler interleaving.            |
//! | `ScoreBoard::handle()`    | `handle()`              | `&mut self`; consumes signal.    |
//! | `ScoreBoard::handled()`   | `handled()`             | Direct mapping.                  |
//! | *(no original)*           | `get_args()`            | Reference access after handle(). |
//! | *(no original)*           | `completed_cycles`      | Verification-only ghost counter. |
//!
//! ## API Divergence
//!
//! - `handle()` takes `&mut self` (original takes `&self` with atomic try_down).
//!   Returns `Ghost<KcallArgsView>` (original returns `Result<&KcallArgs, Error>`).
//!   The `&mut self` is required to model the dispatched signal consumption.
//!   A separate `get_args(&self)` provides reference access after the transition.
//! - `KcallResult` uses `is_success: bool` + `value: i64` (original uses an enum
//!   with `Success(KcallSuccess(i64))` / `Error(KcallError(i32))`).
//! - `completed_cycles: u64` is verification-only state. The original has no
//!   cycle counter. This field tracks protocol progress for inductive proofs.
//!   It has a `u64::MAX` overflow guard in `complete_dispatch()` that does not
//!   correspond to original behavior.
//!
//! ## Trust Boundaries
//!
//! - **T1: Global singleton.** The original uses `static mut SCOREBOARD: Option<ScoreBoard>`
//!   with `unsafe` access. The verified model uses a regular struct. The safety of
//!   the global mutable state is not verified. The `get_mut()` function that returns
//!   `Err(ErrorCode::TryAgain)` when uninitialized is not modeled; the `new()`
//!   constructor guarantees a valid initial state.
//! - **T2: Mutex correctness.** The model assumes the mutex provides mutual exclusion.
//!   The mutex is separately verified in `kernel::pm::sync::mutex`.
//! - **T3: Semaphore correctness.** The model assumes the semaphores correctly
//!   implement counting and blocking. The semaphore is separately verified in
//!   `kernel::pm::sync::semaphore`.
//! - **T4: Sequential ordering.** The model assumes sequential execution. The
//!   original relies on mutex + semaphore for thread synchronization.
//! - **T5: Error handling.** The original returns `Result` types with various
//!   error codes. The verified model uses preconditions to guarantee success.
//!   Error conditions by criticality:
//!   - *Safety-critical*: None. All error paths in the original lead to error
//!     propagation, not undefined behavior.
//!   - *Liveness-critical*: `SleepError::Interrupted` from `handled.down()` in
//!     `dispatch()` could leave the scoreboard in a stuck state (mutex held, no
//!     handler response). `Error` from `handled.up()` in `handled()` could leave
//!     the dispatcher waiting indefinitely.
//!   - *Non-critical*: `ErrorCode::TryAgain` from `handle()` when no dispatch is
//!     pending (normal polling behavior).
//!
//! ## Verification Scope
//!
//! This verification proves **sequential state machine correctness** of the
//! scoreboard dispatch protocol. The following are explicitly **out of scope**:
//! - Concurrency and thread scheduling.
//! - The global singleton pattern (`static mut` safety).
//! - Error propagation paths (`SleepError`, `Error`).
//! - The dispatcher and handler modules (`dispatcher.rs`, `handler.rs`).

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
pub struct KcallResult {
    /// Whether this result represents the Success variant.
    pub is_success: bool,
    /// Payload value: any i64 for success, must fit i32 for error.
    pub value: i64,
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
    /// Uses `u64` rather than `Ghost<nat>` because Verus Ghost fields inside exec
    /// structs complicate View trait derivation and pattern matching.
    pub completed_cycles: u64,
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
            result.completed_cycles == 0,
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
            completed_cycles: 0,
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
    /// Ghost copy of the result view.
    pub fn complete_dispatch(&mut self) -> (result: Ghost<KcallResultView>)
        requires
            old(self).wf(),
            old(self).spec_is_handled(),
            old(self).completed_cycles < u64::MAX,
        ensures
            self.wf(),
            self.spec_is_idle(),
            !self.locked,
            self.dispatched_value == 0,
            self.handled_value == 0,
            self.result@ == old(self).result@,
            result@ == old(self).result@,
            self@ == ScoreBoard::spec_complete_dispatch(old(self)@),
            self.completed_cycles == old(self).completed_cycles + 1,
    {
        let ret: Ghost<KcallResultView> = Ghost(self.result@);
        self.handled_value = 0;
        self.locked = false;
        self.completed_cycles = self.completed_cycles + 1;
        self.phase = ScoreBoardPhase::Idle;
        ret
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

} // verus!
