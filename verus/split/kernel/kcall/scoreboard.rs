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
//! 1. **Dispatch** (user thread): Acquires mutex, sets args, signals `dispatched`
//!    semaphore, waits on `handled` semaphore, reads result, releases mutex.
//! 2. **Handle** (kernel thread): Polls `dispatched` semaphore (try_down), reads args.
//! 3. **Handled** (kernel thread): Sets result, signals `handled` semaphore.
//!
//! ## Verified Properties
//!
//! - Initialization produces a well-formed scoreboard in the Idle phase.
//! - The three-phase protocol (Idle → Dispatched → Handled → Idle) is the only
//!   valid transition sequence.
//! - Mutual exclusion is held during the Dispatched and Handled phases.
//! - Argument integrity: the handler reads exactly the args the dispatcher set.
//! - Result integrity: the dispatcher reads exactly the result the handler set.
//! - The cycle counter is monotonically increasing.
//! - Well-formedness (`wf()`) is preserved by all transitions.
//! - The handle step is read-only and idempotent.
//! - Phase transitions are deterministic.
//! - Multiple consecutive cycles produce predictable state.
//!
//! ## Verification Model
//!
//! The original implementation uses `Mutex`, `Semaphore` (with `AtomicUsize` and
//! `Condvar`), and a `static mut` global singleton. For verification, we model:
//! - The protocol phase as an enum (`ScoreBoardPhase`).
//! - The mutex as a boolean `locked` field.
//! - The semaphores as integer counters (`dispatched_value`, `handled_value`).
//! - All state transitions via `&mut self` methods.
//!
//! This is a sequential model that verifies the state machine protocol (phase
//! transitions, data flow, mutual exclusion) without reasoning about atomicity,
//! memory ordering, or concurrent thread scheduling.
//!
//! ## API Mapping
//!
//! | Original API              | Verified Model          | Notes                            |
//! |---------------------------|-------------------------|----------------------------------|
//! | `ScoreBoard::init()`      | `new()`                 | Returns struct instead of global.|
//! | `ScoreBoard::get_mut()`   | *(not modeled)*         | Global access; see Trust T1.     |
//! | `ScoreBoard::dispatch()`  | `begin_dispatch()` +    | Split into two phases for        |
//! |                           | `complete_dispatch()`   | handler interleaving.            |
//! | `ScoreBoard::handle()`    | `handle()`              | Direct mapping.                  |
//! | `ScoreBoard::handled()`   | `handled()`             | Direct mapping.                  |
//!
//! ## Trust Boundaries
//!
//! - **T1: Global singleton.** The original uses `static mut SCOREBOARD: Option<ScoreBoard>`
//!   with `unsafe` access. The verified model uses a regular struct. The safety of
//!   the global mutable state is not verified.
//! - **T2: Mutex correctness.** The model assumes the mutex provides mutual exclusion.
//!   The mutex is separately verified in `kernel::pm::sync::mutex`.
//! - **T3: Semaphore correctness.** The model assumes the semaphores correctly
//!   implement counting and blocking. The semaphore is separately verified in
//!   `kernel::pm::sync::semaphore`.
//! - **T4: Sequential ordering.** The model assumes sequential execution. The
//!   original relies on mutex + semaphore for thread synchronization.
//! - **T5: Error handling.** The original returns `Result` types with various
//!   error codes. The verified model uses preconditions to guarantee success.
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
/// and `Error(KcallError(i32))`. For verification, we model it as a single `i64`
/// value where non-negative values represent success and negative values
/// represent errors.
pub struct KcallResult {
    /// Result value (>= 0 for success, < 0 for error).
    pub value: i64,
}

/// The scoreboard: a rendezvous channel for kernel call dispatch.
///
/// # Description
///
/// Coordinates kernel call dispatch between user threads (dispatchers)
/// and the kernel thread (handler). The protocol is a three-phase handshake:
/// Idle → Dispatched → Handled → Idle.
///
/// In the original, synchronization uses `Mutex`, two `Semaphore`s, and
/// a `static mut` global. The verified model uses plain fields with
/// `&mut self` state transitions.
pub struct ScoreBoard {
    /// Whether the mutex is currently held.
    pub locked: bool,
    /// Dispatched semaphore value (0 or transiently 1).
    pub dispatched_value: u8,
    /// Handled semaphore value (0 or 1).
    pub handled_value: u8,
    /// Current kernel call arguments.
    pub args: KcallArgs,
    /// Current kernel call result.
    pub result: KcallResult,
    /// Current protocol phase.
    pub phase: ScoreBoardPhase,
    /// Count of completed dispatch-handle-handled cycles.
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
    /// - `pid`: Process identifier.
    /// - `tid`: Thread identifier.
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
        requires
            0 <= pid,
            0 <= tid,
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
            result.value == 0,
            result@ == KcallResult::spec_ok_view(),
    {
        KcallResult { value: 0 }
    }

    /// Creates a result with a specific value.
    ///
    /// # Parameters
    ///
    /// - `value`: The result value.
    ///
    /// # Returns
    ///
    /// A `KcallResult` with the specified value.
    pub fn from_value(value: i64) -> (result: Self)
        ensures
            result.value == value,
            result@ == (KcallResultView { value: value as int }),
    {
        KcallResult { value }
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
    /// Models the first half of the original `dispatch()`:
    /// 1. Acquires the mutex (modeled by setting `locked = true`).
    /// 2. Stores the kernel call arguments.
    /// 3. Signals the handler via `dispatched.up()` (transiently sets
    ///    `dispatched_value = 1`, then handler consumes it).
    ///
    /// After this call, the scoreboard is in the `Dispatched` phase and
    /// the dispatcher should wait for the handler (via `complete_dispatch`).
    ///
    /// # Parameters
    ///
    /// - `args`: The kernel call arguments to dispatch.
    ///
    /// # Returns
    ///
    /// The arguments view, for the caller to verify integrity.
    pub fn begin_dispatch(&mut self, args: KcallArgs) -> (result: Ghost<KcallArgsView>)
        requires
            old(self).wf(),
            old(self).spec_is_idle(),
            args.wf(),
        ensures
            self.wf(),
            self.spec_is_dispatched(),
            self.locked,
            self.args@ == args@,
            self@ == ScoreBoard::spec_begin_dispatch(old(self)@, args@),
            result@ == args@,
    {
        self.locked = true;
        self.args = args;
        // Signal handler: up dispatched semaphore (transiently 1, consumed by handler).
        // In the model, we keep it at 0 since the handler will consume it immediately.
        self.dispatched_value = 0;
        self.phase = ScoreBoardPhase::Dispatched;
        Ghost(self.args@)
    }

    /// Handles a dispatched kernel call: reads the arguments.
    ///
    /// # Description
    ///
    /// Models the original `handle()`:
    /// 1. Consumes the dispatched signal via `dispatched.try_down()`.
    /// 2. Returns a reference to the arguments.
    ///
    /// The scoreboard remains in the `Dispatched` phase after this call.
    /// The handler should process the call and then invoke `handled()`.
    ///
    /// # Returns
    ///
    /// A ghost copy of the arguments view.
    pub fn handle(&self) -> (result: Ghost<KcallArgsView>)
        requires
            self.wf(),
            self.spec_is_dispatched(),
        ensures
            result@ == self.args@,
            self@ == ScoreBoard::spec_handle(self@),
    {
        Ghost(self.args@)
    }

    /// Signals that the kernel call has been handled and stores the result.
    ///
    /// # Description
    ///
    /// Models the original `handled()`:
    /// 1. Stores the kernel call result.
    /// 2. Signals the dispatcher via `handled.up()`.
    ///
    /// The scoreboard transitions to the `Handled` phase.
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
    /// 1. Consumes the handled signal via `handled.down()`.
    /// 2. Reads the result.
    /// 3. Releases the mutex (guard drop).
    ///
    /// The scoreboard returns to the `Idle` phase.
    ///
    /// # Returns
    ///
    /// A ghost copy of the result view.
    pub fn complete_dispatch(&mut self) -> (result: Ghost<KcallResultView>)
        requires
            old(self).wf(),
            old(self).spec_is_handled(),
            old(self).completed_cycles < u64::MAX,
        ensures
            self.wf(),
            self.spec_is_idle(),
            !self.locked,
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

    /// Returns whether the scoreboard is in the dispatched phase.
    ///
    /// # Returns
    ///
    /// `true` if the scoreboard has a pending dispatch.
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
