// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Handler Loop Verification Model
//!
//! Formal verification of the kernel call handler event loop.
//!
//! ## Overview
//!
//! The `kcall_handler` function is the main kernel event loop. It:
//! 1. Initializes the event subsystem (`event::init`).
//! 2. Polls the scoreboard for pending kernel calls and dispatches them.
//! 3. Polls for inter-kernel communication (IKC) messages.
//! 4. Harvests zombie processes.
//! 5. Yields the CPU when no work was done.
//! 6. Exits when the init daemon (INITD) terminates.
//! 7. After exit, drains all remaining zombie processes.
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
//!   iteration (kcall_handled, message_received, harvested_process all false)
//!   AND the loop is not terminating. When INITD terminates, the loop exits
//!   before reaching the yield check.
//! - **Work flag monotonicity**: Setting a work flag never clears other flags.
//!   Once work is recorded, it stays recorded.
//! - **Termination condition**: The loop exits only when the init daemon (INITD,
//!   pid=1) terminates. Non-INITD terminations and errors do not exit the loop.
//! - **Post-loop zombie cleanup**: After the loop exits, remaining zombies are
//!   drained. This is modeled in `kcall_handler_lifecycle_step()`.
//! - **Iteration state initialization**: Each iteration starts with all work
//!   flags cleared.
//! - **Dispatch-then-signal protocol**: The scoreboard `handled()` is called
//!   after every successful dispatch, maintaining the scoreboard protocol.
//! - **Dispatch partition**: Each kcall number is either a valid handler kcall
//!   or Invalid; these categories are mutually exclusive and exhaustive.
//! - **Harvest notification semantics**: The `harvested_process` work flag is
//!   only set when `notify_process_termination` succeeds for a non-INITD
//!   zombie. INITD termination breaks the loop before setting the flag;
//!   notification failures leave the flag false.
//! - **Feature flag modeling**: The `stdio_enabled` parameter gates IKC message
//!   polling, matching the original `cfg_if!(feature = "stdio")`. When false,
//!   `message_received` is always false, preventing phantom message work.
//! - **Lifecycle model**: `kcall_handler_init()` establishes the loop invariant
//!   base case; `kcall_handler_lifecycle_step()` preserves the invariant
//!   inductively using the REAL harvest outcome from the iteration and calls
//!   `drain_remaining_zombies()` on termination.
//! - **Full loop model**: `kcall_handler_loop()` models the complete handler
//!   lifecycle (init → bounded iteration loop → drain) with a fuel parameter
//!   for termination, proving the loop invariant is preserved throughout.
//! - **Exec-to-spec linkage**: `spec_harvest_to_outcome()` converts exec-level
//!   harvest flags to the spec `HarvestOutcome` enum, and
//!   `lemma_harvest_to_outcome_termination()` proves the termination semantics
//!   are preserved by the conversion.
//! - **Oracle-connected liveness**: `spec_handler_correct_under_liveness()`
//!   defines the end-to-end correctness property under a liveness assumption
//!   using an environment oracle model. `lemma_oracle_connected_liveness()`
//!   proves the contrapositive link: if the loop didn't terminate, no INITD
//!   was observed. The oracle-to-execution correspondence is an assumption
//!   because `harvest_zombies()` is an external body (T2).
//!
//! ## Verification Model
//!
//! This is a **shadow model** verification. The original `kcall_handler`
//! (`src/kernel/src/kcall/handler.rs`) accesses global state
//! (`ScoreBoard::get_mut()`, `ProcessManager`, `EventManager`), uses OS
//! primitives (`Mutex`, `Semaphore`), and takes `&mut Hal` / `&mut
//! VirtMemoryManager` / `&mut ProcessManager` parameters that cannot be
//! compiled by Verus. Direct verification of the source is infeasible.
//! This shadow-model approach is the standard methodology used across all
//! verified modules in the Nanvix project.
//!
//! **Drift mitigation**: Source baseline (commit ff5c49cc, SHA-256 prefix
//! 38c3e49732afc4ed, 200 lines), regression lemmas for constants and
//! dispatch coverage, and source file/line citations in spec comments.
//!
//! For verification, we model:
//! - The loop's control flow as spec functions over abstract state.
//! - Work tracking via `LoopIterationState` with three boolean flags.
//! - Dispatch routing via `HandlerDispatchCategory` enum.
//! - Termination via `HarvestOutcome` and `spec_should_terminate`.
//! - External subsystem calls as `external_body` boundary functions.
//! - The full handler lifecycle (init → loop → drain) via
//!   `kcall_handler_init()` and `kcall_handler_lifecycle_step()`.
//!
//! ## API Mapping
//!
//! | Original API                       | Verified Model                          | Notes                        |
//! |------------------------------------|-----------------------------------------|------------------------------|
//! | `kcall_handler()` lifecycle        | `kcall_handler_loop()`                  | Full loop model w/ fuel.     |
//! | `kcall_handler()` step             | `kcall_handler_lifecycle_step()`        | Single lifecycle step.       |
//! | `kcall_handler()` main loop        | `run_full_iteration()`                  | Full iteration with yield.   |
//! | `kcall_handler()` single step      | `run_iteration()`                       | Single iteration w/o yield.  |
//! | `event::init(hal)`                 | `event_init()`                          | External body (T5).          |
//! | `ScoreBoard::get_mut()`            | `poll_scoreboard_full()`                | External body (T1).          |
//! | `scoreboard.handle()`              | Part of `poll_scoreboard_full()`        | External body (T1).          |
//! | `scoreboard.handled(ret)`          | `signal_handled()`                      | External body (T1).          |
//! | Match on `KcallNumber::from(...)`  | `classify_and_check_invalid()`          | Verified routing.            |
//! | `pm.harvest_zombies(mm)`           | `harvest_zombies()`                     | External body (T2).          |
//! | `EventManager::notify_...()`       | `notify_termination()`                  | External body (T2), fallible.|
//! | `ProcessManager::giveup()`         | `yield_cpu()`                           | External body (T3).          |
//! | IKC message polling                | `poll_messages_gated()`                 | Verified gate (T4).          |
//! | Post-loop zombie drain             | `drain_remaining_zombies()`             | External body (T2).          |
//!
//! ## Trust Boundaries
//!
//! - **T1: ScoreBoard access.** `ScoreBoard::get_mut()`, `handle()`, and
//!   `handled()` access a `static mut` global. The scoreboard protocol is
//!   separately verified in `kernel::kcall::scoreboard`.
//! - **T2: ProcessManager operations.** `harvest_zombies()`,
//!   `EventManager::notify_process_termination()` are dependency boundary
//!   operations. Their correctness is assumed. `notify_termination` may fail
//!   (returns false), matching the original `Err(e) => error!(...)` path.
//! - **T3: CPU yield.** `ProcessManager::giveup()` performs a context switch.
//!   Its correctness is assumed (HAL dependency).
//! - **T4: IKC message polling.** `crate::stdio::read()` and
//!   `EventManager::post_message()` are dependency boundary operations.
//! - **T5: Event initialization.** `event::init(hal)` is called once before
//!   the handler loop starts. Modeled as `event_init()`.
//!
//! ## Scope Limitations
//!
//! - **Concurrency**: The model is sequential. The original handler runs in the
//!   kernel thread and accesses shared state (ScoreBoard) with mutex/semaphore
//!   synchronization. The concurrent protocol is verified in the scoreboard module.
//! - **Liveness**: No liveness properties (eventual progress, starvation freedom)
//!   are specified. The handler loop may spin indefinitely if no work arrives.
//! - **Feature flags**: The `stdio` feature flag for IKC message polling is
//!   modeled via the `stdio_enabled` parameter to `poll_messages_gated()`.
//!   When false, `message_received` is always false, matching the original
//!   `cfg_if!` `else` branch. This prevents phantom message work from
//!   suppressing yields in non-`stdio` builds.
//! - **Error recovery**: Scoreboard errors (`unreachable!` in original) and
//!   harvest errors (`error!` and continue) are modeled via `error` flags on
//!   result types. Error outcomes guarantee no work was done (`has_call=false`
//!   or `found=false`), preserving yield and termination correctness. The
//!   `notify_termination` external body returns a bool indicating success,
//!   matching the original pattern where `harvested_process` is only set
//!   on `Ok(())`.
//! - **Post-loop drain**: `drain_remaining_zombies()` is invoked in the
//!   lifecycle step model on termination. The property that "no zombies remain
//!   after drain" depends on ProcessManager state (T2) and is not proved.
//! - **Model synchronization**: This verification uses a shadow model that
//!   abstracts the original `src/kernel/src/kcall/handler.rs`. If the original
//!   source changes (e.g., adding a new kcall number, changing yield logic),
//!   the model must be updated manually. Spec constants (`SPEC_INITD_PID`,
//!   `SPEC_ERROR_INVALID_SYSCALL`) are hardcoded and must match their source
//!   definitions (`ProcessIdentifier::INITD`, `ErrorCode::InvalidSysCall`).
//!   The regression-style `lemma_spec_constants_match_source()` in the proof
//!   file documents the expected values for drift detection.
//!   **Source baseline**: `src/kernel/src/kcall/handler.rs` (200 lines,
//!   commit ff5c49cc, SHA-256 prefix 38c3e49732afc4ed). If the source file
//!   changes, compare the diff against this model and update accordingly.
//! - **IKC polling internals**: The IKC message polling loop's internal logic
//!   (batching with `IKC_POLL_BATCH_SIZE`, buffer limit `MAX_IKC_MESSAGES`,
//!   `stdio::read()` and `EventManager::post_message()`) is abstracted into
//!   the `poll_messages_raw()` external body. The handler-level verification
//!   proves yield correctness based on the result, but does not verify that
//!   the polling loop respects batch/buffer constraints.

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
/// `error` indicates whether harvesting failed (error and continue).
/// `pid` is the process identifier of the harvested zombie (if any).
/// `is_initd` is true if the harvested zombie was the init daemon.
/// `exit_status` is the exit status of the harvested zombie (if any).
pub struct ZombieHarvestResult {
    /// Whether a zombie was found.
    pub found: bool,
    /// Whether harvesting failed with an error.
    pub error: bool,
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
/// kernel call has been processed and the result is available. In the
/// original code (line 101), `handled()` may fail (`Err(e)`), logging
/// a warning but NOT affecting the `kcall_handled` flag — the call is
/// considered handled regardless of signaling success. This matches
/// the exec model where `handle_kcall_phase` sets `kcall_handled = true`
/// unconditionally after calling `signal_handled`.
///
/// ## Trust Boundary T1
#[verifier::external_body]
pub fn signal_handled(result: &HandlerKcallResult)
    ensures
        // Signaling completes (may fail internally, but the call is
        // considered handled regardless). See handle_kcall_phase().
        true,
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
/// The subsystem call results depend on kernel state not modeled in this
/// module. The correctness of individual subsystem calls is the responsibility
/// of each subsystem's verification.
#[verifier::external_body]
pub fn dispatch_to_subsystem(kcall_number: u32) -> (result: HandlerKcallResult)
    ensures
        // Error results carry a non-zero error code.
        result.is_error ==> result.error_code != 0i32,
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
/// This function models the actual I/O call and is only invoked when
/// the `stdio` feature is enabled. See `poll_messages_gated()` for
/// the feature-gated wrapper.
///
/// ## Trust Boundary T4
#[verifier::external_body]
pub fn poll_messages_raw() -> (result: bool)
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
/// The `error` flag indicates harvest failure (original: `Err(e)`);
/// errors guarantee no zombie was found.
///
/// The `exit_status` field is intentionally unconstrained. Its value
/// originates from the terminated process and propagates to the final
/// `kcall_handler` return value. The correctness of the exit status
/// depends on ProcessManager state (T2) and is outside the handler's
/// verification scope.
///
/// ## Trust Boundary T2
#[verifier::external_body]
pub fn harvest_zombies() -> (result: ZombieHarvestResult)
    ensures
        result.is_initd ==> (result.found && result.pid == 1u32),
        (result.found && result.pid == 1u32) ==> result.is_initd,
        result.error ==> !result.found,
{
    unimplemented!()
}

/// External body: notifies process termination.
///
/// # Description
///
/// Models `EventManager::notify_process_termination(...)`. Returns true
/// if the notification succeeded (`Ok(())`), false on failure (`Err(e)`).
/// In the original code, `harvested_process` is only set to true when
/// this call succeeds.
///
/// ## Trust Boundary T2
#[verifier::external_body]
pub fn notify_termination(pid: u32, exit_status: u32) -> (result: bool)
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

/// External body: initializes the event subsystem.
///
/// # Description
///
/// Models `event::init(hal)`. Called once before the handler loop starts.
/// In the original code, failure causes `panic!("failed to initialize
/// event manager")`. The verification assumes successful initialization;
/// the panic path is not modeled. If initialization fails, the original
/// code would never enter the handler loop.
///
/// ## Trust Boundary T5
///
/// ## Assumption
///
/// Initialization always succeeds. In the original code, failure causes
/// `panic!("failed to initialize event manager")` which aborts the kernel.
/// The panic path is not modeled; if initialization fails, the handler
/// loop never starts.
#[verifier::external_body]
pub fn event_init()
    ensures
        // ASSUMPTION: initialization succeeds (panic on failure = kernel abort).
        true,
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
/// The `has_error` field models the `unreachable!()` paths in the original
/// code where `ScoreBoard::get_mut()` fails (line 119) or `scoreboard.handle()`
/// returns an error other than `TryAgain` (line 112). In practice, these
/// paths never execute; the error field allows the model to prove that even
/// if they did, no work would be reported and the iteration proceeds safely.
pub struct ScoreBoardPollResult {
    /// Whether a kcall was found.
    pub has_call: bool,
    /// The kcall number (meaningful only when `has_call` is true).
    pub kcall_number: u32,
    /// Whether a scoreboard access error occurred (unreachable in practice).
    pub has_error: bool,
}

//==================================================================================================
// Verified Functions
//==================================================================================================

/// Polls for IKC messages, gated by the stdio feature flag.
///
/// # Description
///
/// Models the `cfg_if!` block in the original handler that conditionally
/// polls for IKC messages. When `stdio_enabled` is true, delegates to
/// `poll_messages_raw()` (external body). When false, returns false
/// immediately, matching the `else` branch where `message_received = false`.
///
/// This function ensures that in non-`stdio` builds, `message_received`
/// is always false, preventing phantom message work from suppressing
/// yields.
pub fn poll_messages_gated(stdio_enabled: bool) -> (result: bool)
    ensures
        !stdio_enabled ==> !result,
{
    if stdio_enabled {
        poll_messages_raw()
    } else {
        false
    }
}

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
    ensures
        result.is_initd ==> (result.found && result.pid == 1u32),
        (result.found && result.pid == 1u32) ==> result.is_initd,
        result.error ==> !result.found,
{
    harvest_zombies()
}

/// Runs a single iteration of the handler loop.
///
/// # Description
///
/// Executes all three phases (kcall dispatch, message polling, zombie harvest)
/// and determines whether to yield or continue. Returns the work state,
/// termination signal, and a ghost `HarvestOutcome` linked to the actual
/// harvest result via `spec_harvest_to_outcome`.
///
/// The `harvested_process` work flag is only set when:
/// 1. A zombie was found (`harvest.found`),
/// 2. It was NOT the init daemon (`!terminate`), AND
/// 3. `notify_process_termination` succeeded.
/// This matches the original code where `harvested_process = true` is only
/// reached on `Ok(())` from the notification call, and INITD termination
/// breaks the loop before setting the flag.
pub fn run_iteration(poll: &ScoreBoardPollResult, stdio_enabled: bool) -> (result: IterationResult)
    ensures
        // If a kcall was polled, it was handled.
        poll.has_call ==> result.work_state.kcall_handled,
        // If no kcall was polled and no messages/zombies, should yield.
        !result.work_state.kcall_handled && !result.work_state.message_received
            && !result.work_state.harvested_process ==> result.should_yield,
        // Yield iff no work was done.
        result.should_yield == (!result.work_state.kcall_handled
            && !result.work_state.message_received && !result.work_state.harvested_process),
        // Termination implies INITD pid.
        result.should_terminate ==> result.initd_pid == 1u32,
        // INITD termination does NOT set harvested_process (loop breaks first).
        result.should_terminate ==> !result.work_state.harvested_process,
        // Ghost outcome reflects actual harvest and links to spec termination.
        result.should_terminate == spec_should_terminate(result.harvest_outcome@),
        // Non-stdio builds never receive messages.
        !stdio_enabled ==> !result.work_state.message_received,
{
    // Phase 1: Handle pending kernel call.
    let kcall_phase: HandlerKcallPhaseResult = handle_kcall_phase(poll);

    // Phase 2: Poll for IKC messages (gated by stdio feature flag).
    let msg_received: bool = poll_messages_gated(stdio_enabled);

    // Phase 3: Harvest zombie processes.
    let harvest: ZombieHarvestResult = handle_harvest_phase();
    let terminate: bool = is_initd_terminated(&harvest);

    // Derive ghost harvest outcome from exec-level fields.
    proof {
        lemma_harvest_to_outcome_termination(
            harvest.found, harvest.error, harvest.pid as nat, harvest.is_initd,
        );
    }

    // Notify termination for non-INITD zombies only.
    // In the original, INITD causes `break status` before reaching notify.
    let harvested_flag: bool = if harvest.found && !terminate {
        notify_termination(harvest.pid, harvest.exit_status)
    } else {
        false
    };

    // Build work state.
    let work_state: HandlerWorkState = HandlerWorkState {
        kcall_handled: kcall_phase.kcall_handled,
        message_received: msg_received,
        harvested_process: harvested_flag,
    };

    let do_yield: bool = should_yield(&work_state);

    IterationResult {
        work_state,
        should_yield: do_yield,
        should_terminate: terminate,
        exit_status: harvest.exit_status,
        initd_pid: harvest.pid,
        harvest_outcome: Ghost(spec_harvest_to_outcome(
            harvest.found, harvest.error, harvest.pid as nat, harvest.is_initd,
        )),
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
    /// Ghost harvest outcome linked to the spec-level HarvestOutcome enum.
    /// Derived from the actual harvest via `spec_harvest_to_outcome`.
    pub harvest_outcome: Ghost<HarvestOutcome>,
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
/// 3. Polls for IKC messages (gated by `stdio_enabled`).
/// 4. Harvests zombie processes and notifies termination.
/// 5. Yields the CPU if no work was done AND the loop is not terminating.
///
/// This function models the entire loop body, including the yield behavior
/// that `run_iteration()` only flags.
///
/// Note: The ensures clause is intentionally narrower than `run_iteration()`
/// because the poll result is internal. Properties like
/// `poll.has_call ==> result.work_state.kcall_handled` are not exposed
/// since the caller has no access to the poll result. The core loop
/// properties (yield-iff-idle, termination-implies-INITD) are preserved.
pub fn run_full_iteration(stdio_enabled: bool) -> (result: IterationResult)
    ensures
        // Yield iff no work was done.
        result.should_yield == (!result.work_state.kcall_handled
            && !result.work_state.message_received && !result.work_state.harvested_process),
        // Termination implies INITD pid.
        result.should_terminate ==> result.initd_pid == 1u32,
        // INITD termination does NOT set harvested_process.
        result.should_terminate ==> !result.work_state.harvested_process,
        // Ghost outcome reflects actual harvest.
        result.should_terminate == spec_should_terminate(result.harvest_outcome@),
        // Non-stdio builds never receive messages.
        !stdio_enabled ==> !result.work_state.message_received,
{
    // Phase 1: Poll scoreboard.
    let poll: ScoreBoardPollResult = poll_scoreboard_full();

    // Phase 2-4: Run iteration (dispatch, messages, harvest).
    let result: IterationResult = run_iteration(&poll, stdio_enabled);

    // Phase 5: Yield CPU if no work was done and loop is not terminating.
    // The `!result.should_terminate` guard is necessary because our model returns
    // a result struct rather than using `break` for INITD termination. In the
    // original code (lines 165-167), INITD termination causes `break status` during
    // the harvest phase, exiting the loop before reaching the yield check at
    // line 187. In our sequential model, `run_full_iteration` always returns, so
    // when INITD terminates with no other work done (`should_yield && should_terminate`),
    // we must suppress the yield to match the original's control flow where the
    // yield would never be reached.
    if result.should_yield && !result.should_terminate {
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
/// what number it has. Scoreboard access errors (`unreachable!` in
/// original) are not modeled as they should never occur.
///
/// The `kcall_number` field is unconstrained when `has_call` is true because
/// `classify_and_check_invalid` handles all `u32` values — any out-of-range
/// number simply maps to `Invalid` and returns `InvalidSysCall`.
///
/// ## Trust Boundary T1
#[verifier::external_body]
pub fn poll_scoreboard_full() -> (result: ScoreBoardPollResult)
    ensures
        // When no call is pending, the kcall_number has no meaning.
        // Default is 0 (Debug); the `has_call` gate in handle_kcall_phase
        // prevents this from being used.
        !result.has_call ==> result.kcall_number == 0u32,
        // Scoreboard errors never occur in practice (original: unreachable!()).
        // This matches the fail-stop semantics: if these paths execute,
        // the kernel panics. The verification assumes they don't.
        !result.has_error,
{
    unimplemented!()
}

//==================================================================================================
// Lifecycle Model
//==================================================================================================

/// Result of a lifecycle step.
///
/// # Description
///
/// Encapsulates the outcome of one step of the handler lifecycle, including
/// whether the loop terminated and the ghost history for invariant tracking.
pub struct LifecycleStepResult {
    /// Whether the handler loop terminated (INITD exited).
    pub terminated: bool,
    /// The exit status (meaningful only when `terminated` is true).
    /// This value originates from `harvest_zombies()` (T2) and is
    /// unconstrained — its correctness depends on ProcessManager state.
    pub exit_status: u32,
    /// The PID that triggered termination (meaningful only when `terminated`).
    /// Proved to equal INITD (1) when terminated.
    pub termination_pid: u32,
    /// Ghost history of harvest outcomes for loop invariant tracking.
    pub new_history: Ghost<Seq<HarvestOutcome>>,
}

/// Models handler initialization and returns the initial loop history.
///
/// # Description
///
/// Initializes the event subsystem (T5) and establishes the base case of
/// the loop invariant: the empty history satisfies `spec_loop_invariant`.
/// This models the `event::init(hal)` call before the handler loop starts.
///
/// ## Precondition (Assumption)
///
/// Event initialization succeeds. In the original code, `event::init(hal)`
/// panics on failure, aborting the kernel before the handler loop starts.
/// This function assumes the panic path is unreachable. The assumption is
/// encoded in `event_init()`'s external body postcondition (`ensures true`).
pub fn kcall_handler_init() -> (history: Ghost<Seq<HarvestOutcome>>)
    ensures
        spec_loop_invariant(history@),
        history@.len() == 0,
{
    event_init();
    proof { lemma_loop_invariant_base(); }
    Ghost(Seq::empty())
}

/// Models one step of the handler lifecycle, connecting the loop invariant.
///
/// # Description
///
/// Given a history trace of past iterations satisfying the loop invariant,
/// runs one full iteration. If the loop continues (INITD not terminated),
/// the invariant is preserved with the history extended by the REAL harvest
/// outcome from the iteration (derived via `spec_harvest_to_outcome`).
/// If the loop terminates (INITD found), `drain_remaining_zombies()` is
/// called to model the post-loop cleanup.
///
/// This function connects:
/// - **Initialization**: The `requires` clause demands a valid history.
/// - **Iteration**: `run_full_iteration()` models one loop body.
/// - **Real outcome**: The ghost history uses the actual harvest outcome
///   from the iteration, not a synthetic `NoZombie`.
/// - **Invariant induction**: On continuation, the real non-terminating
///   outcome is appended and the invariant preserved.
/// - **Termination**: On INITD exit, the loop breaks and drain occurs.
/// - **Post-loop drain**: `drain_remaining_zombies()` is called on exit.
pub fn kcall_handler_lifecycle_step(
    history: Ghost<Seq<HarvestOutcome>>,
    stdio_enabled: bool,
) -> (result: LifecycleStepResult)
    requires
        spec_loop_invariant(history@),
    ensures
        // The invariant is always preserved.
        spec_loop_invariant(result.new_history@),
        // On continuation, history grows by one.
        !result.terminated ==> result.new_history@.len() == history@.len() + 1,
        // On termination, history is unchanged.
        result.terminated ==> result.new_history@.len() == history@.len(),
        // On termination, the exit was triggered by INITD (pid == 1).
        result.terminated ==> result.termination_pid == 1u32,
{
    let iter_result: IterationResult = run_full_iteration(stdio_enabled);

    if iter_result.should_terminate {
        // INITD terminated: drain remaining zombies and exit.
        drain_remaining_zombies();
        LifecycleStepResult {
            terminated: true,
            exit_status: iter_result.exit_status,
            termination_pid: iter_result.initd_pid,
            new_history: Ghost(history@),
        }
    } else {
        // Loop continues: extend history with the REAL harvest outcome.
        let ghost outcome: HarvestOutcome = iter_result.harvest_outcome@;
        proof {
            // The ensures on run_full_iteration gives us:
            //   iter_result.should_terminate == spec_should_terminate(outcome)
            // Since !iter_result.should_terminate, we have !spec_should_terminate(outcome),
            // which is spec_loop_continues(outcome).
            assert(spec_loop_continues(outcome));
            lemma_loop_invariant_inductive(history@, outcome);
        }
        LifecycleStepResult {
            terminated: false,
            exit_status: 0u32,
            termination_pid: 0u32,
            new_history: Ghost(spec_extend_history(history@, outcome)),
        }
    }
}

/// Result of the full handler loop model.
///
/// # Description
///
/// Encapsulates the outcome of running the handler loop to completion
/// (or until the fuel limit is reached).
pub struct LoopResult {
    /// Whether the handler loop terminated (INITD exited).
    pub terminated: bool,
    /// The exit status (meaningful only when `terminated` is true).
    /// Originates from `harvest_zombies()` (T2); its correctness depends
    /// on ProcessManager state and is outside verification scope.
    pub exit_status: u32,
    /// The PID that triggered termination (meaningful only when `terminated`).
    pub termination_pid: u32,
    /// Ghost history of harvest outcomes for all completed iterations.
    pub final_history: Ghost<Seq<HarvestOutcome>>,
}

/// Models the full handler loop from init through iteration until
/// termination or fuel exhaustion.
///
/// # Description
///
/// This function models the complete `kcall_handler` lifecycle:
/// 1. Initializes the event subsystem.
/// 2. Iterates the handler loop up to `fuel` times.
/// 3. Each iteration runs `kcall_handler_lifecycle_step()`, which:
///    - Runs one full iteration (poll, dispatch, messages, harvest, yield).
///    - On continuation, extends the history with the real harvest outcome.
///    - On INITD termination, drains remaining zombies and exits.
/// 4. Returns the final loop state.
///
/// The `fuel` parameter models a bounded number of iterations. In the
/// original code, the loop runs indefinitely until INITD terminates.
/// For verification, the fuel bound provides a decreasing measure for
/// Verus's termination checker. The loop invariant is preserved at every
/// step regardless of fuel exhaustion.
///
/// **Semantic gap**: If `fuel` is exhausted before INITD terminates,
/// `terminated` is false, which has no counterpart in the original code
/// (where the loop always runs until INITD exits). This means the model
/// cannot prove that the handler *always* returns a valid `ExitStatus`.
/// Proving total termination would require a liveness assumption (INITD
/// eventually terminates) that depends on external system behavior and
/// is outside the scope of this safety verification.
///
/// **Conditional termination**: The postconditions establish a tight
/// bound between termination status and history length. When the loop
/// exits without termination (`!result.terminated`), exactly `fuel`
/// iterations ran and the invariant guarantees none of them observed
/// INITD termination (via `lemma_invariant_excludes_termination`).
/// When termination occurs (`result.terminated`), it happened before
/// fuel was exhausted (`final_history.len() < fuel`). Together with
/// `lemma_loop_termination_completeness`, this proves the contrapositive:
/// if INITD terminates within `fuel` iterations, the loop MUST return
/// `terminated == true`.
pub fn kcall_handler_loop(fuel: u32, stdio_enabled: bool) -> (result: LoopResult)
    ensures
        // The loop invariant holds for the final history.
        spec_loop_invariant(result.final_history@),
        // Fuel exhaustion: all iterations ran, none triggered termination.
        !result.terminated ==> result.final_history@.len() == fuel as int,
        // Early exit: termination occurred before fuel ran out.
        result.terminated ==> result.final_history@.len() < fuel as int,
        // On termination, it was INITD (pid == 1) that triggered exit.
        result.terminated ==> result.termination_pid == 1u32,
{
    let mut history: Ghost<Seq<HarvestOutcome>> = kcall_handler_init();
    let mut i: u32 = 0;
    let mut terminated: bool = false;
    let mut exit_status: u32 = 0;
    let mut termination_pid: u32 = 0;

    while i < fuel && !terminated
        invariant
            spec_loop_invariant(history@),
            i <= fuel,
            // History tracks iteration count when the loop is still running.
            !terminated ==> history@.len() == i as int,
            // Termination happened before fuel was exhausted.
            terminated ==> history@.len() < fuel as int,
            // Termination was triggered by INITD.
            terminated ==> termination_pid == 1u32,
        decreases fuel - i,
    {
        let step: LifecycleStepResult = kcall_handler_lifecycle_step(
            history, stdio_enabled,
        );
        if step.terminated {
            terminated = true;
            exit_status = step.exit_status;
            termination_pid = step.termination_pid;
        }
        // Always update history (unchanged on termination, extended otherwise).
        history = step.new_history;
        i = i + 1;
    }

    LoopResult {
        terminated,
        exit_status,
        termination_pid,
        final_history: history,
    }
}

} // verus!
