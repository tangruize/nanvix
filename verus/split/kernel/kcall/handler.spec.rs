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
pub open spec fn spec_classify_handler_kcall(number: u32) -> HandlerDispatchCategory {
    if number == 0 { HandlerDispatchCategory::Debug }
    else if number == 1 { HandlerDispatchCategory::GetPid }
    else if number == 2 { HandlerDispatchCategory::GetTid }
    else if number == 4 { HandlerDispatchCategory::CapCtl }
    else if number == 6 { HandlerDispatchCategory::Terminate }
    else if number == 7 { HandlerDispatchCategory::EventCtrl }
    else if number == 10 { HandlerDispatchCategory::MemoryMap }
    else if number == 11 { HandlerDispatchCategory::MemoryUnmap }
    else if number == 12 { HandlerDispatchCategory::MemoryCtrl }
    else if number == 13 { HandlerDispatchCategory::MemoryCopy }
    else if number == 8 { HandlerDispatchCategory::Send }
    else if number == 14 { HandlerDispatchCategory::AllocMmio }
    else if number == 15 { HandlerDispatchCategory::FreeMmio }
    else if number == 16 { HandlerDispatchCategory::AllocPmio }
    else if number == 17 { HandlerDispatchCategory::FreePmio }
    else if number == 18 { HandlerDispatchCategory::ReadPmio }
    else if number == 19 { HandlerDispatchCategory::WritePmio }
    else if number == 28 { HandlerDispatchCategory::GetTime }
    else if number == 21 { HandlerDispatchCategory::CreateThread }
    else if number == 30 { HandlerDispatchCategory::SetThreadDataArea }
    else if number == 31 { HandlerDispatchCategory::GetThreadDataArea }
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
// Spec Functions: Well-Formedness
//==================================================================================================

/// Spec function: well-formedness of an iteration state.
///
/// # Description
///
/// An iteration state is always well-formed (all fields are booleans).
/// This is a trivial predicate included for uniformity with other modules.
pub open spec fn spec_iteration_wf(state: LoopIterationState) -> bool {
    true
}

/// Spec function: well-formedness of a handler dispatch category.
///
/// # Description
///
/// All dispatch categories are well-formed. Included for uniformity.
pub open spec fn spec_dispatch_category_wf(cat: HandlerDispatchCategory) -> bool {
    true
}

} // verus!
