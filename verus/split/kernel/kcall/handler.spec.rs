// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Handler Specification.
// Defines View types, spec constants, and spec functions for the kernel call
// handler loop verification model.
//
// ## Verification Model
//
// The handler is the main kernel event loop that:
// 1. Polls the scoreboard for pending kernel calls and dispatches them.
// 2. Polls for inter-kernel communication (IKC) messages.
// 3. Harvests zombie processes.
// 4. Yields the CPU when no work was done.
// 5. Exits when the init daemon (INITD) terminates.
//
// This spec models the loop's control flow, work tracking, termination
// condition, and dispatch routing at an abstract level.

verus! {

//==================================================================================================
// Spec Constants
//==================================================================================================

/// The process identifier of the init daemon (INITD).
/// Matches `ProcessIdentifier::INITD = ProcessIdentifier(1)`.
pub open spec fn SPEC_INITD_PID() -> nat { 1 }

/// The error code for InvalidSysCall (ENOSYS = 88 in Nanvix).
pub open spec fn SPEC_ERROR_INVALID_SYSCALL() -> int { 88 }

//==================================================================================================
// View Types
//==================================================================================================

/// Outcome of polling the scoreboard for a pending kernel call.
///
/// # Description
///
/// Models the result of `ScoreBoard::get_mut()` + `scoreboard.handle()`:
/// - `NoPending`: No kernel call is pending (TryAgain).
/// - `GotCall(number)`: A kernel call with the given number was retrieved.
/// - `AccessError`: The scoreboard could not be accessed (unreachable in practice).
#[verifier::ext_equal]
pub enum ScoreBoardPollOutcome {
    /// No kernel call is pending (TryAgain).
    NoPending,
    /// A kernel call was retrieved with the given kcall number.
    GotCall { number: nat },
    /// The scoreboard could not be accessed.
    AccessError,
}

/// Outcome of dispatching a single kernel call.
///
/// # Description
///
/// Models the `KcallResult` returned by the handler dispatch match:
/// - `Success`: The kernel call was handled successfully.
/// - `Error(code)`: The kernel call resulted in an error with the given code.
#[verifier::ext_equal]
pub enum DispatchOutcome {
    /// The kernel call was handled successfully.
    Success,
    /// The kernel call resulted in an error.
    Error { code: int },
}

/// Outcome of harvesting zombie processes.
///
/// # Description
///
/// Models the result of `pm.harvest_zombies(mm)`:
/// - `NoZombie`: No zombie processes available.
/// - `Harvested(pid, is_initd)`: A zombie was harvested; `is_initd` indicates
///   whether the terminated process was the init daemon.
/// - `HarvestError`: Harvesting failed.
#[verifier::ext_equal]
pub enum HarvestOutcome {
    /// No zombie processes to harvest.
    NoZombie,
    /// A zombie was harvested. `pid` is the process ID, `is_initd` is true
    /// if this was the init daemon.
    Harvested { pid: nat, is_initd: bool },
    /// Harvesting failed with an error.
    HarvestError,
}

/// Abstract state of one iteration of the handler loop.
///
/// # Description
///
/// Tracks the three work indicators that determine whether the CPU should
/// yield at the end of each iteration.
#[verifier::ext_equal]
pub struct LoopIterationState {
    /// Whether a kernel call was handled in this iteration.
    pub kcall_handled: bool,
    /// Whether an IKC message was received in this iteration.
    pub message_received: bool,
    /// Whether a zombie process was harvested in this iteration.
    pub harvested_process: bool,
}

/// Abstract view of the handler loop termination.
///
/// # Description
///
/// Models why the handler loop exited and what status it returned.
#[verifier::ext_equal]
pub struct HandlerTermination {
    /// The exit status value from the init daemon.
    pub exit_status: nat,
}

/// Classification of a kcall number for the handler dispatch.
///
/// # Description
///
/// Models the match arms in the handler's dispatch logic. Each kcall
/// number routes to one of these handler categories. Numbers that don't
/// match any defined handler return InvalidSysCall error.
#[verifier::ext_equal]
pub enum HandlerDispatchCategory {
    /// Debug kernel call.
    Debug,
    /// GetPid - should be handled by dispatcher, returns InvalidSysCall.
    GetPid,
    /// GetTid - should be handled by dispatcher, returns InvalidSysCall.
    GetTid,
    /// Capability control.
    CapCtl,
    /// Process termination.
    Terminate,
    /// Event control.
    EventCtrl,
    /// Memory map.
    MemoryMap,
    /// Memory unmap.
    MemoryUnmap,
    /// Memory control.
    MemoryCtrl,
    /// Memory copy.
    MemoryCopy,
    /// IPC send.
    Send,
    /// MMIO allocation.
    AllocMmio,
    /// MMIO free.
    FreeMmio,
    /// PMIO allocation.
    AllocPmio,
    /// PMIO free.
    FreePmio,
    /// PMIO read.
    ReadPmio,
    /// PMIO write.
    WritePmio,
    /// Get system time.
    GetTime,
    /// Create thread.
    CreateThread,
    /// Set thread data area.
    SetThreadDataArea,
    /// Get thread data area.
    GetThreadDataArea,
    /// Invalid/unknown kernel call number.
    Invalid,
}

//==================================================================================================
// Spec Functions: Dispatch Classification
//==================================================================================================

/// Spec function: classifies a kcall number for handler dispatch.
///
/// # Description
///
/// Maps each kcall number to its handler category, mirroring the match
/// statement in `kcall_handler`. Uses the same kcall number constants
/// as the dispatcher spec. Numbers outside the defined range (0..=31)
/// or not handled (e.g., Exit, ExitThread, which are locally handled
/// by the dispatcher) map to `Invalid`.
///
/// Source: `src/kernel/src/kcall/handler.rs` lines 62-97.
/// Constants imported from the dispatcher module's spec, which mirrors
/// `src/libs/sys/src/sys/number.rs` `KcallNumber` enum values.
pub open spec fn spec_classify_handler_kcall(number: u32) -> HandlerDispatchCategory {
    if number == super::dispatcher::KCALL_DEBUG() { HandlerDispatchCategory::Debug }
    else if number == super::dispatcher::KCALL_GET_PID() { HandlerDispatchCategory::GetPid }
    else if number == super::dispatcher::KCALL_GET_TID() { HandlerDispatchCategory::GetTid }
    else if number == super::dispatcher::KCALL_CAP_CTL() { HandlerDispatchCategory::CapCtl }
    else if number == super::dispatcher::KCALL_TERMINATE() { HandlerDispatchCategory::Terminate }
    else if number == super::dispatcher::KCALL_EVENT_CTRL() { HandlerDispatchCategory::EventCtrl }
    else if number == super::dispatcher::KCALL_MEMORY_MAP() { HandlerDispatchCategory::MemoryMap }
    else if number == super::dispatcher::KCALL_MEMORY_UNMAP() { HandlerDispatchCategory::MemoryUnmap }
    else if number == super::dispatcher::KCALL_MEMORY_CTRL() { HandlerDispatchCategory::MemoryCtrl }
    else if number == super::dispatcher::KCALL_MEMORY_COPY() { HandlerDispatchCategory::MemoryCopy }
    else if number == super::dispatcher::KCALL_SEND() { HandlerDispatchCategory::Send }
    else if number == super::dispatcher::KCALL_ALLOC_MMIO() { HandlerDispatchCategory::AllocMmio }
    else if number == super::dispatcher::KCALL_FREE_MMIO() { HandlerDispatchCategory::FreeMmio }
    else if number == super::dispatcher::KCALL_ALLOC_PMIO() { HandlerDispatchCategory::AllocPmio }
    else if number == super::dispatcher::KCALL_FREE_PMIO() { HandlerDispatchCategory::FreePmio }
    else if number == super::dispatcher::KCALL_READ_PMIO() { HandlerDispatchCategory::ReadPmio }
    else if number == super::dispatcher::KCALL_WRITE_PMIO() { HandlerDispatchCategory::WritePmio }
    else if number == super::dispatcher::KCALL_GET_TIME() { HandlerDispatchCategory::GetTime }
    else if number == super::dispatcher::KCALL_CREATE_THREAD() { HandlerDispatchCategory::CreateThread }
    else if number == super::dispatcher::KCALL_SET_TDA() { HandlerDispatchCategory::SetThreadDataArea }
    else if number == super::dispatcher::KCALL_GET_TDA() { HandlerDispatchCategory::GetThreadDataArea }
    else { HandlerDispatchCategory::Invalid }
}

/// Spec function: checks if a kcall number is handled by the handler loop.
///
/// # Description
///
/// Returns true if the kcall number corresponds to a handler arm in the
/// handler's match statement (including GetPid/GetTid which return
/// InvalidSysCall). Returns false for numbers not present in the match.
pub open spec fn spec_is_handler_kcall(number: u32) -> bool {
    !matches!(spec_classify_handler_kcall(number), HandlerDispatchCategory::Invalid)
}

/// Spec function: checks if a kcall returns InvalidSysCall error.
///
/// # Description
///
/// GetPid and GetTid are expected to be handled by the dispatcher, not
/// the handler loop. If they reach the handler, they return InvalidSysCall.
/// Invalid/unknown kcall numbers also return InvalidSysCall.
pub open spec fn spec_returns_invalid_syscall(number: u32) -> bool {
    let cat: HandlerDispatchCategory = spec_classify_handler_kcall(number);
    ||| matches!(cat, HandlerDispatchCategory::GetPid)
    ||| matches!(cat, HandlerDispatchCategory::GetTid)
    ||| matches!(cat, HandlerDispatchCategory::Invalid)
}

//==================================================================================================
// Spec Functions: Loop Iteration Logic
//==================================================================================================

/// Spec function: creates a fresh loop iteration state.
///
/// # Description
///
/// At the start of each iteration, no work has been done yet.
pub open spec fn spec_initial_iteration() -> LoopIterationState {
    LoopIterationState {
        kcall_handled: false,
        message_received: false,
        harvested_process: false,
    }
}

/// Spec function: determines if any work was done in an iteration.
///
/// # Description
///
/// Returns true if at least one of the three work indicators is set.
pub open spec fn spec_did_work(state: LoopIterationState) -> bool {
    state.kcall_handled || state.message_received || state.harvested_process
}

/// Spec function: determines if the CPU should yield.
///
/// # Description
///
/// The CPU yields iff no work was done in the current iteration.
/// This is the negation of `spec_did_work`.
pub open spec fn spec_should_yield(state: LoopIterationState) -> bool {
    !spec_did_work(state)
}

/// Spec function: updates iteration state after handling a kcall.
///
/// # Description
///
/// Sets the `kcall_handled` flag to true, preserving other flags.
pub open spec fn spec_after_kcall_handled(state: LoopIterationState) -> LoopIterationState {
    LoopIterationState {
        kcall_handled: true,
        ..state
    }
}

/// Spec function: updates iteration state after receiving a message.
///
/// # Description
///
/// Sets the `message_received` flag to true, preserving other flags.
pub open spec fn spec_after_message_received(state: LoopIterationState) -> LoopIterationState {
    LoopIterationState {
        message_received: true,
        ..state
    }
}

/// Spec function: updates iteration state after harvesting a zombie.
///
/// # Description
///
/// Sets the `harvested_process` flag to true, preserving other flags.
pub open spec fn spec_after_harvest(state: LoopIterationState) -> LoopIterationState {
    LoopIterationState {
        harvested_process: true,
        ..state
    }
}

//==================================================================================================
// Spec Functions: Termination Condition
//==================================================================================================

/// Spec function: checks if a harvest outcome should terminate the loop.
///
/// # Description
///
/// The handler loop exits when the init daemon (INITD) terminates.
/// This is the only termination condition for the main kernel loop.
pub open spec fn spec_should_terminate(outcome: HarvestOutcome) -> bool {
    match outcome {
        HarvestOutcome::Harvested { pid, is_initd } => is_initd,
        _ => false,
    }
}

/// Spec function: checks if a pid is the init daemon.
///
/// # Description
///
/// Returns true if the pid matches the init daemon identifier (1).
pub open spec fn spec_is_initd(pid: nat) -> bool {
    pid == SPEC_INITD_PID()
}

//==================================================================================================
// Spec Functions: Loop Invariant
//==================================================================================================

/// Spec function: the loop invariant for the handler event loop.
///
/// # Description
///
/// The invariant states that in all harvest outcomes observed during
/// iterations 0..n, none indicated INITD termination. This captures
/// the fact that the loop is still running precisely because INITD
/// has not terminated in any prior iteration.
///
/// `history` is a sequence of harvest outcomes, one per completed iteration.
/// The invariant holds when every entry in the history is a non-terminating
/// outcome.
pub open spec fn spec_loop_invariant(history: Seq<HarvestOutcome>) -> bool {
    forall|i: int| 0 <= i < history.len() ==> !spec_should_terminate(#[trigger] history[i])
}

/// Spec function: models the loop exit condition.
///
/// # Description
///
/// The loop exits iff a harvest outcome indicates INITD termination.
pub open spec fn spec_loop_exits(outcome: HarvestOutcome) -> bool {
    spec_should_terminate(outcome)
}

/// Spec function: models the loop continuation condition.
///
/// # Description
///
/// The loop continues iff INITD has not terminated. After each iteration,
/// the work flags are reset for the next iteration.
pub open spec fn spec_loop_continues(outcome: HarvestOutcome) -> bool {
    !spec_should_terminate(outcome)
}

/// Spec function: extends the loop history with a new non-terminating outcome.
///
/// # Description
///
/// When the loop continues (INITD not terminated), the outcome is appended
/// to the history trace.
pub open spec fn spec_extend_history(
    history: Seq<HarvestOutcome>,
    outcome: HarvestOutcome,
) -> Seq<HarvestOutcome> {
    history.push(outcome)
}

//==================================================================================================
// Spec Functions: Exec-to-Spec Conversion
//==================================================================================================

/// Spec function: converts exec-level harvest fields to the spec HarvestOutcome enum.
///
/// # Description
///
/// Bridges the exec-level `ZombieHarvestResult` fields (found, error, pid, is_initd)
/// to the spec-level `HarvestOutcome` enum. This function is used to derive the ghost
/// harvest outcome from actual iteration results, linking exec behavior to spec
/// reasoning about termination and loop invariants.
///
/// Mapping:
/// - `error == true` → `HarvestError` (error implies !found)
/// - `found == true` → `Harvested { pid, is_initd }`
/// - otherwise → `NoZombie`
pub open spec fn spec_harvest_to_outcome(
    found: bool,
    error: bool,
    pid: nat,
    is_initd: bool,
) -> HarvestOutcome {
    if error {
        HarvestOutcome::HarvestError
    } else if found {
        HarvestOutcome::Harvested { pid, is_initd }
    } else {
        HarvestOutcome::NoZombie
    }
}

//==================================================================================================
// Spec Functions: Liveness Assumption
//==================================================================================================

/// Spec function: models the liveness assumption that INITD eventually terminates.
///
/// # Description
///
/// This predicate expresses the assumption that within a sequence of harvest
/// outcomes, at least one is a terminating outcome (INITD exited). This is
/// a fairness/liveness assumption about the external environment — the init
/// daemon will eventually exit, causing the handler loop to terminate.
///
/// This assumption cannot be proved within the handler's verification model
/// because it depends on the behavior of the process manager and the init
/// daemon, which are external to the handler loop.
pub open spec fn spec_initd_terminates_within(outcomes: Seq<HarvestOutcome>) -> bool {
    exists|i: int| 0 <= i < outcomes.len() && spec_should_terminate(#[trigger] outcomes[i])
}

//==================================================================================================
// Spec Functions: Top-Level Correctness Theorem
//==================================================================================================

/// Spec function: top-level correctness predicate for the handler loop.
///
/// # Description
///
/// States the expected end-to-end correctness property of `kcall_handler`
/// under the liveness assumption that INITD terminates within `fuel` iterations:
///
/// 1. The loop terminates (`terminated == true`).
/// 2. Termination was triggered by INITD (pid == 1).
/// 3. The loop invariant (no INITD in history) is maintained.
///
/// The `exit_status` value is intentionally unconstrained by this predicate
/// because it originates from `harvest_zombies()` (T2 trust boundary) and
/// its correctness depends on ProcessManager state outside the handler's
/// verification scope.
///
/// Usage: Given `result = kcall_handler_loop(fuel, stdio_enabled)` and
/// the assumption `spec_initd_terminates_within(actual_outcomes)` where
/// `actual_outcomes` is the sequence of harvest outcomes that WOULD occur,
/// the contrapositive argument from `lemma_loop_termination_completeness`
/// proves `spec_handler_terminated_correctly(result)`.
pub open spec fn spec_handler_terminated_correctly(
    terminated: bool,
    termination_pid: u32,
    history: Seq<HarvestOutcome>,
) -> bool {
    &&& terminated
    &&& termination_pid == SPEC_INITD_PID() as u32
    &&& spec_loop_invariant(history)
}

//==================================================================================================
// Spec Functions: Environment Oracle Model
//==================================================================================================

/// Spec function: models a single environment harvest outcome at a given iteration.
///
/// # Description
///
/// An "environment oracle" is a mapping from iteration index to the
/// `HarvestOutcome` that `harvest_zombies()` would produce at that iteration.
/// This concept is separate from the recorded history (which only contains
/// non-terminating outcomes by loop invariant).
///
/// The oracle represents the external environment's behavior and is used
/// to connect the liveness assumption to the exec model: if the oracle
/// produces a terminating outcome at iteration `k`, the lifecycle step at
/// iteration `k` will return `terminated = true` and the loop will exit.
///
/// Since `harvest_zombies()` is an `external_body` with unconstrained output,
/// the oracle cannot be mechanically tied to actual execution. The connection
/// is by assumption: the real system behaves consistently with the oracle.
pub open spec fn spec_oracle_terminates_at(oracle: Seq<HarvestOutcome>, k: int) -> bool {
    0 <= k < oracle.len() && spec_should_terminate(oracle[k])
}

/// Spec function: an oracle contains a terminating outcome within the first
/// `n` iterations.
pub open spec fn spec_oracle_has_termination(oracle: Seq<HarvestOutcome>, n: int) -> bool {
    exists|k: int| 0 <= k < n && k < oracle.len() && spec_should_terminate(#[trigger] oracle[k])
}

/// Spec function: the recorded history matches the oracle's non-terminating prefix.
///
/// # Description
///
/// This predicate encodes the oracle-to-execution correspondence assumption:
/// the recorded history (which only contains non-terminating outcomes by
/// invariant) matches the first `history.len()` entries of the oracle, and
/// those entries are all non-terminating. This is the key assumption that
/// cannot be mechanically verified because `harvest_zombies()` is external.
///
/// When this holds and the oracle contains a terminating outcome at index `k`,
/// then `k >= history.len()` (because history entries are non-terminating).
/// If the loop ran for `fuel` iterations without terminating, history has
/// `fuel` entries, so `k >= fuel`. Contrapositive: if the oracle terminates
/// at `k < fuel`, the loop must have terminated at or before iteration `k`.
pub open spec fn spec_oracle_matches_history(
    oracle: Seq<HarvestOutcome>,
    history: Seq<HarvestOutcome>,
) -> bool {
    &&& oracle.len() >= history.len()
    &&& forall|i: int| 0 <= i < history.len() ==>
        (#[trigger] history[i]) == oracle[i]
}

/// Spec function: correctness under liveness assumption.
///
/// # Description
///
/// States the expected end-to-end property: if the environment oracle
/// produces a terminating outcome within `fuel` iterations, then the loop
/// returns `terminated == true` with all correctness properties.
///
/// This connects the liveness assumption (INITD eventually terminates)
/// to the exec model (the loop detects it and exits). The connection
/// relies on two assumptions:
/// 1. The oracle faithfully represents `harvest_zombies()` outputs.
/// 2. The loop runs with sufficient fuel (fuel >= termination iteration + 1).
///
/// These assumptions cannot be discharged within the handler module because
/// `harvest_zombies()` is an external body (T2). The proof is by the
/// contrapositive: `lemma_loop_termination_completeness` shows that if
/// `!terminated`, then no INITD was observed in the history. Combined with
/// the oracle assumption (iteration outcomes match the oracle), INITD in
/// the oracle implies INITD was observed, yielding a contradiction.
pub open spec fn spec_handler_correct_under_liveness(
    terminated: bool,
    termination_pid: u32,
    history: Seq<HarvestOutcome>,
    oracle: Seq<HarvestOutcome>,
    fuel: nat,
) -> bool {
    // If the oracle contains a terminating outcome within fuel iterations,
    // and the loop ran with that fuel, then:
    spec_oracle_has_termination(oracle, fuel as int) ==>
        spec_handler_terminated_correctly(terminated, termination_pid, history)
}

} // verus!
