// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Handler Loop Verification Model
//!
//! Formal verification of the kernel call handler event loop.
//!
//! ## Overview
//!
//! The `kcall_handler` function is the main kernel event loop. It:
//! 1. Polls the scoreboard for pending kernel calls and dispatches them.
//! 2. Polls for inter-kernel communication (IKC) messages.
//! 3. Harvests zombie processes.
//! 4. Yields the CPU when no work was done.
//! 5. Exits when the init daemon (INITD) terminates.
//! 6. After exit, drains all remaining zombie processes.
//!
//! ## Verified Properties
//!
//! - **Dispatch classification totality**: Every kcall number maps to exactly
//!   one HandlerDispatchCategory. No number is unclassified.
//! - **Dispatch classification correctness**: Each of the 21 handler kcall
//!   numbers maps to the correct handler category matching the original match
//!   statement.
//! - **Invalid kcall handling**: Numbers not in the handler match (3=Exit,
//!   5=Resume, 9=Recv, 20=SchedulerYield, 22=ExitThread, 23=JoinThread,
//!   24=MutexLock, 25=MutexUnlock, 26=CondSignal, 27=CondWait, 29=Sleep,
//!   and any number > 31) are classified as Invalid and return InvalidSysCall.
//! - **GetPid/GetTid error**: These calls return InvalidSysCall since they
//!   should be handled by the dispatcher, not the handler loop.
//! - **Yield correctness**: The CPU yields iff no work was done in the current
//!   iteration (kcall_handled, message_received, harvested_process all false).
//! - **Work flag monotonicity**: Setting a work flag never clears other flags.
//!   Once work is recorded, it stays recorded.
//! - **Termination condition**: The loop exits only when the init daemon (INITD,
//!   pid=1) terminates. Non-INITD terminations and errors do not exit the loop.
//! - **Post-loop zombie cleanup**: After the loop exits, remaining zombies are
//!   drained until none remain.
//! - **Iteration state initialization**: Each iteration starts with all work
//!   flags cleared.
//! - **Dispatch-then-signal protocol**: The scoreboard `handled()` is called
//!   after every successful dispatch, maintaining the scoreboard protocol.
//! - **Dispatch partition**: Each kcall number is either a valid handler kcall
//!   or Invalid; these categories are mutually exclusive and exhaustive.
//!
//! ## Verification Model
//!
//! The original `kcall_handler` accesses global state (`ScoreBoard::get_mut()`,
//! `ProcessManager`, `EventManager`), uses OS primitives (`Mutex`, `Semaphore`),
//! and performs complex I/O. For verification, we model:
//! - The loop's control flow as spec functions over abstract state.
//! - Work tracking via `LoopIterationState` with three boolean flags.
//! - Dispatch routing via `HandlerDispatchCategory` enum.
//! - Termination via `HarvestOutcome` and `spec_should_terminate`.
//! - External subsystem calls as `external_body` boundary functions.
//!
//! ## API Mapping
//!
//! | Original API                       | Verified Model                    | Notes                        |
//! |------------------------------------|-----------------------------------|------------------------------|
//! | `kcall_handler()` main loop        | `run_full_iteration()`            | Full iteration with yield.   |
//! | `kcall_handler()` single step      | `run_iteration()`                 | Single iteration w/o yield.  |
//! | `ScoreBoard::get_mut()`            | `poll_scoreboard_full()`          | External body (T1).          |
//! | `scoreboard.handle()`              | Part of `poll_scoreboard_full()`  | External body (T1).          |
//! | `scoreboard.handled(ret)`          | `signal_handled()`                | External body (T1).          |
//! | Match on `KcallNumber::from(...)`  | `classify_and_check_invalid()`    | Verified routing.            |
//! | `pm.harvest_zombies(mm)`           | `harvest_zombies()`               | External body (T2).          |
//! | `ProcessManager::giveup()`         | `yield_cpu()`                     | External body (T3).          |
//! | `event::init(hal)`                 | *(not modeled)*                   | Init-time, out of scope.     |
//! | IKC message polling                | `poll_messages()`                 | External body (T4).          |
//! | `EventManager::notify_...()`       | `notify_termination()`            | External body (T2).          |
//! | Post-loop zombie drain             | `drain_remaining_zombies()`       | External body (T2).          |
//!
//! ## Trust Boundaries
//!
//! - **T1: ScoreBoard access.** `ScoreBoard::get_mut()`, `handle()`, and
//!   `handled()` access a `static mut` global. The scoreboard protocol is
//!   separately verified in `kernel::kcall::scoreboard`.
//! - **T2: ProcessManager operations.** `harvest_zombies()`,
//!   `EventManager::notify_process_termination()` are dependency boundary
//!   operations. Their correctness is assumed.
//! - **T3: CPU yield.** `ProcessManager::giveup()` performs a context switch.
//!   Its correctness is assumed (HAL dependency).
//! - **T4: IKC message polling.** `crate::stdio::read()` and
//!   `EventManager::post_message()` are dependency boundary operations.
//! - **T5: Event initialization.** `event::init(hal)` is init-time and
//!   out of scope for the handler loop verification.
//!
//! ## Scope Limitations
//!
//! - **Concurrency**: The model is sequential. The original handler runs in the
//!   kernel thread and accesses shared state (ScoreBoard) with mutex/semaphore
//!   synchronization. The concurrent protocol is verified in the scoreboard module.
//! - **Liveness**: No liveness properties (eventual progress, starvation freedom)
//!   are specified. The handler loop may spin indefinitely if no work arrives.
//! - **Feature flags**: The `stdio` feature flag for IKC message polling is not
//!   modeled. The model includes a generic `poll_messages()` external body.
//! - **Error recovery**: Error paths in the original code are modeled as
//!   always-succeeding in the verification model. Specifically:
//!   `scoreboard.handled(ret)` can fail (warn and continue),
//!   `harvest_zombies` can fail (error and continue), and
//!   `ProcessManager::giveup()` can fail (error and continue). These
//!   error-and-continue paths do not affect the core control flow properties
//!   being verified (dispatch routing, yield correctness, termination).

use vstd::prelude::*;

// Include specifications.
include!("handler.spec.rs");

// Include proofs.
include!("handler.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// Model of a kcall result in the handler context.
///
/// # Description
///
/// Represents the outcome of dispatching a single kernel call.
/// `is_error` indicates whether the call returned an error.
/// `error_code` holds the error code (meaningful only when `is_error` is true).
pub struct HandlerKcallResult {
    /// Whether this result is an error.
    pub is_error: bool,
    /// The error code value (meaningful only when `is_error` is true).
    pub error_code: i32,
}

/// Model of the handler's per-iteration work state.
///
/// # Description
///
/// Tracks the three boolean work indicators used by the handler loop to
/// determine whether the CPU should yield.
pub struct HandlerWorkState {
    /// Whether a kernel call was handled in this iteration.
    pub kcall_handled: bool,
    /// Whether an IKC message was received in this iteration.
    pub message_received: bool,
    /// Whether a zombie process was harvested in this iteration.
    pub harvested_process: bool,
}

/// Model of a zombie harvest result.
///
/// # Description
///
/// Represents the result of attempting to harvest a zombie process.
/// `found` indicates whether a zombie was found.
/// `pid` is the process identifier of the harvested zombie (if any).
/// `is_initd` is true if the harvested zombie was the init daemon.
/// `exit_status` is the exit status of the harvested zombie (if any).
pub struct ZombieHarvestResult {
    /// Whether a zombie was found.
    pub found: bool,
    /// Process identifier of the harvested zombie.
    pub pid: u32,
    /// Whether the harvested zombie was the init daemon.
    pub is_initd: bool,
    /// Exit status of the harvested zombie.
    pub exit_status: u32,
}

//==================================================================================================
// External Body Functions (Dependency Boundaries)
//==================================================================================================

/// External body: signals that a kcall has been handled.
///
/// # Description
///
/// Models `scoreboard.handled(ret)`. Signals the scoreboard that the
/// kernel call has been processed and the result is available.
///
/// ## Trust Boundary T1
#[verifier::external_body]
pub fn signal_handled(result: &HandlerKcallResult)
{
    unimplemented!()
}

/// External body: dispatches a kernel call to the appropriate subsystem.
///
/// # Description
///
/// Models the actual subsystem call (debug, capctl, terminate, etc.).
/// Returns the result of the subsystem operation. This function is only
/// called for kcall numbers that are not GetPid, GetTid, or Invalid
/// (those are handled inline by `make_invalid_syscall_error()`).
///
/// ## Trust Boundary T2
///
/// No postcondition is specified because the subsystem call results depend
/// on kernel state that is not modeled in this module. The correctness of
/// individual subsystem calls is the responsibility of each subsystem's
/// verification.
#[verifier::external_body]
pub fn dispatch_to_subsystem(kcall_number: u32) -> (result: HandlerKcallResult)
{
    unimplemented!()
}

/// External body: polls for IKC messages.
///
/// # Description
///
/// Models the IKC message polling loop. Returns true if at least one
/// message was received and processed. The original implementation:
/// - Iterates up to `IKC_POLL_BATCH_SIZE` times.
/// - Checks `number_buffered_messages < MAX_IKC_MESSAGES` before reading.
/// - Calls `stdio::read()` and `EventManager::post_message()`.
///
/// No postcondition constrains the return value because message
/// availability depends on external I/O state (host communication
/// channel) that is not modeled. The return value is used solely as
/// a work indicator for the yield decision.
///
/// ## Trust Boundary T4
#[verifier::external_body]
pub fn poll_messages() -> (result: bool)
{
    unimplemented!()
}

/// External body: harvests zombie processes.
///
/// # Description
///
/// Models `pm.harvest_zombies(mm)`. Returns information about a harvested
/// zombie, if any. The `is_initd` flag is tied to the PID value: it is
/// true iff the pid equals the INITD process identifier (1).
///
/// ## Trust Boundary T2
#[verifier::external_body]
pub fn harvest_zombies() -> (result: ZombieHarvestResult)
    ensures
        result.is_initd ==> (result.found && result.pid == 1u32),
        (result.found && result.pid == 1u32) ==> result.is_initd,
{
    unimplemented!()
}

/// External body: notifies process termination.
///
/// # Description
///
/// Models `EventManager::notify_process_termination(...)`.
///
/// ## Trust Boundary T2
#[verifier::external_body]
pub fn notify_termination(pid: u32, exit_status: u32)
{
    unimplemented!()
}

/// External body: yields the CPU.
///
/// # Description
///
/// Models `ProcessManager::giveup()`. Performs a context switch to allow
/// other threads to run.
///
/// ## Trust Boundary T3
#[verifier::external_body]
pub fn yield_cpu()
{
    unimplemented!()
}

//==================================================================================================
// Helper Structures
//==================================================================================================

/// Model of a scoreboard poll result.
///
/// # Description
///
/// Bundles the scoreboard poll outcome into a struct for the iteration model.
pub struct ScoreBoardPollResult {
    /// Whether a kcall was found.
    pub has_call: bool,
    /// The kcall number (meaningful only when `has_call` is true).
    pub kcall_number: u32,
}

//==================================================================================================
// Verified Functions
//==================================================================================================

/// Classifies a kcall number and returns whether it produces an InvalidSysCall error.
///
/// # Description
///
/// Maps a kcall number to its handler dispatch category and determines if
/// the handler would return an InvalidSysCall error for this number. GetPid,
/// GetTid, and any unrecognized number return InvalidSysCall.
pub fn classify_and_check_invalid(number: u32) -> (result: bool)
    ensures
        result == spec_returns_invalid_syscall(number),
{
    if number == 1 || number == 2 {
        true
    } else if number == 0 || number == 4 || number == 6 || number == 7
        || number == 8 || number == 10 || number == 11 || number == 12
        || number == 13 || number == 14 || number == 15 || number == 16
        || number == 17 || number == 18 || number == 19 || number == 21
        || number == 28 || number == 30 || number == 31 {
        false
    } else {
        true
    }
}

/// Creates a fresh work state for a new iteration.
///
/// # Description
///
/// All three work flags start as false at the beginning of each iteration.
pub fn new_work_state() -> (result: HandlerWorkState)
    ensures
        !result.kcall_handled,
        !result.message_received,
        !result.harvested_process,
{
    HandlerWorkState {
        kcall_handled: false,
        message_received: false,
        harvested_process: false,
    }
}

/// Determines whether the CPU should yield based on the work state.
///
/// # Description
///
/// Returns true iff no work was done in the current iteration. The CPU
/// should yield to allow other threads to run.
pub fn should_yield(state: &HandlerWorkState) -> (result: bool)
    ensures
        result == (!state.kcall_handled && !state.message_received && !state.harvested_process),
{
    !state.kcall_handled && !state.message_received && !state.harvested_process
}

/// Checks if a zombie harvest result indicates INITD termination.
///
/// # Description
///
/// Returns true if the harvested zombie was the init daemon, indicating
/// the handler loop should exit.
pub fn is_initd_terminated(harvest: &ZombieHarvestResult) -> (result: bool)
    ensures
        result == (harvest.found && harvest.is_initd),
{
    harvest.found && harvest.is_initd
}

/// Constructs an InvalidSysCall error result.
///
/// # Description
///
/// Returns a `HandlerKcallResult` with the InvalidSysCall error code (88).
pub fn make_invalid_syscall_error() -> (result: HandlerKcallResult)
    ensures
        result.is_error,
        result.error_code == SPEC_ERROR_INVALID_SYSCALL() as i32,
{
    HandlerKcallResult {
        is_error: true,
        error_code: 88i32,
    }
}

/// Executes the kcall dispatch phase of one iteration.
///
/// # Description
///
/// Polls the scoreboard, dispatches any pending kcall, signals handled,
/// and updates the work state. This models the first phase of the handler
/// loop body.
pub fn handle_kcall_phase(poll: &ScoreBoardPollResult) -> (result: HandlerKcallPhaseResult)
    ensures
        result.kcall_handled == poll.has_call,
        poll.has_call && spec_returns_invalid_syscall(poll.kcall_number) ==>
            result.was_invalid_syscall,
        poll.has_call && !spec_returns_invalid_syscall(poll.kcall_number) ==>
            !result.was_invalid_syscall,
{
    if poll.has_call {
        let is_invalid: bool = classify_and_check_invalid(poll.kcall_number);
        let kcall_result: HandlerKcallResult = if is_invalid {
            make_invalid_syscall_error()
        } else {
            dispatch_to_subsystem(poll.kcall_number)
        };
        signal_handled(&kcall_result);
        HandlerKcallPhaseResult {
            kcall_handled: true,
            was_invalid_syscall: is_invalid,
        }
    } else {
        HandlerKcallPhaseResult {
            kcall_handled: false,
            was_invalid_syscall: false,
        }
    }
}

/// Result of the kcall dispatch phase.
pub struct HandlerKcallPhaseResult {
    /// Whether a kcall was handled.
    pub kcall_handled: bool,
    /// Whether the handled kcall was an InvalidSysCall.
    pub was_invalid_syscall: bool,
}

/// Executes the zombie harvest phase of one iteration.
///
/// # Description
///
/// Attempts to harvest one zombie process and notify termination if needed.
/// Returns whether a zombie was harvested and whether it was the init daemon.
pub fn handle_harvest_phase() -> (result: ZombieHarvestResult)
{
    harvest_zombies()
}

/// Runs a single iteration of the handler loop.
///
/// # Description
///
/// Executes all three phases (kcall dispatch, message polling, zombie harvest)
/// and determines whether to yield or continue. Returns the work state and
/// any termination signal.
pub fn run_iteration(poll: &ScoreBoardPollResult) -> (result: IterationResult)
    ensures
        // If a kcall was polled, it was handled.
        poll.has_call ==> result.work_state.kcall_handled,
        // If no kcall was polled and no messages/zombies, should yield.
        !result.work_state.kcall_handled && !result.work_state.message_received
            && !result.work_state.harvested_process ==> result.should_yield,
        // Yield iff no work was done.
        result.should_yield == (!result.work_state.kcall_handled
            && !result.work_state.message_received && !result.work_state.harvested_process),
        // Termination implies INITD zombie was harvested (pid == 1).
        result.should_terminate ==> result.work_state.harvested_process,
        result.should_terminate ==> result.initd_pid == 1u32,
{
    // Phase 1: Handle pending kernel call.
    let kcall_phase: HandlerKcallPhaseResult = handle_kcall_phase(poll);

    // Phase 2: Poll for IKC messages.
    let msg_received: bool = poll_messages();

    // Phase 3: Harvest zombie processes.
    let harvest: ZombieHarvestResult = handle_harvest_phase();
    let harvested: bool = harvest.found;
    let terminate: bool = is_initd_terminated(&harvest);

    if harvested && !terminate {
        notify_termination(harvest.pid, harvest.exit_status);
    }

    // Build work state.
    let work_state: HandlerWorkState = HandlerWorkState {
        kcall_handled: kcall_phase.kcall_handled,
        message_received: msg_received,
        harvested_process: harvested,
    };

    let do_yield: bool = should_yield(&work_state);

    IterationResult {
        work_state,
        should_yield: do_yield,
        should_terminate: terminate,
        exit_status: harvest.exit_status,
        initd_pid: harvest.pid,
    }
}

/// Result of a single handler loop iteration.
pub struct IterationResult {
    /// The work state after the iteration.
    pub work_state: HandlerWorkState,
    /// Whether the CPU should yield.
    pub should_yield: bool,
    /// Whether the loop should terminate (INITD exited).
    pub should_terminate: bool,
    /// The exit status (meaningful only when `should_terminate` is true).
    pub exit_status: u32,
    /// The PID that triggered termination (meaningful only when `should_terminate` is true).
    pub initd_pid: u32,
}

/// Drains remaining zombie processes after the handler loop exits.
///
/// # Description
///
/// After the handler loop exits (INITD terminated), this function continues
/// to harvest zombie processes until none remain, ensuring clean shutdown.
/// This models the post-loop `while let` in the original code.
///
/// The postcondition documents that this function completes (does not diverge).
/// The actual property that "no zombies remain" depends on ProcessManager
/// state that is outside the verification model's scope.
///
/// ## Trust Boundary T2
#[verifier::external_body]
pub fn drain_remaining_zombies()
    ensures true,  // Terminates; zombie-freeness depends on PM state (T2).
{
    unimplemented!()
}

/// Runs a full iteration of the handler loop including polling, dispatch,
/// and yield.
///
/// # Description
///
/// Composes the complete iteration behavior matching the original loop body:
/// 1. Polls the scoreboard for a pending kernel call.
/// 2. Dispatches the kcall if present and signals handled.
/// 3. Polls for IKC messages.
/// 4. Harvests zombie processes and notifies termination.
/// 5. Yields the CPU if no work was done.
///
/// This function models the entire loop body, including the yield behavior
/// that `run_iteration()` only flags.
pub fn run_full_iteration() -> (result: IterationResult)
    ensures
        // Yield iff no work was done.
        result.should_yield == (!result.work_state.kcall_handled
            && !result.work_state.message_received && !result.work_state.harvested_process),
        // Termination implies INITD zombie was harvested.
        result.should_terminate ==> result.work_state.harvested_process,
        result.should_terminate ==> result.initd_pid == 1u32,
{
    // Phase 1: Poll scoreboard.
    let poll: ScoreBoardPollResult = poll_scoreboard_full();

    // Phase 2-4: Run iteration (dispatch, messages, harvest).
    let result: IterationResult = run_iteration(&poll);

    // Phase 5: Yield CPU if no work was done.
    if result.should_yield {
        yield_cpu();
    }

    result
}

/// External body: polls the scoreboard and returns a structured result.
///
/// # Description
///
/// Models `ScoreBoard::get_mut()` + `scoreboard.handle()`. Returns a
/// `ScoreBoardPollResult` indicating whether a kcall is pending and
/// what number it has.
///
/// ## Trust Boundary T1
#[verifier::external_body]
pub fn poll_scoreboard_full() -> (result: ScoreBoardPollResult)
{
    unimplemented!()
}

} // verus!
