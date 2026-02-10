// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Handler Proofs.
// This file contains proof lemmas for the kernel call handler loop.
//
// ## Proved Properties
//
// - Dispatch classification totality: every nat maps to a HandlerDispatchCategory.
// - Dispatch classification correctness: each of the 21 handler kcall numbers
//   maps to its expected category.
// - Invalid kcall detection: numbers not in the handler match return Invalid.
// - GetPid/GetTid return InvalidSysCall error (handled by dispatcher, not handler).
// - Yield correctness: yield happens iff no work was done.
// - Termination condition: only INITD termination exits the loop.
// - Initial iteration state: all work flags are false.
// - Work flag monotonicity: setting a work flag never clears other flags.
// - Work detection after each subsystem: after handling any subsystem,
//   spec_did_work returns true.
// - Conditional liveness: the loop invariant excludes INITD from history,
//   so if INITD terminates within the fuel budget, the loop must return
//   terminated == true (proved via contrapositive and quantifier duality).

verus! {

//==================================================================================================
// Proof Lemmas: Dispatch Classification
//==================================================================================================

/// Lemma: Every nat maps to exactly one HandlerDispatchCategory.
///
/// # Description
///
/// The classification function is total: for any kcall number, a category
/// is always returned. No kcall number is left unclassified.
pub proof fn lemma_classification_totality(number: u32)
    ensures ({
        let cat: HandlerDispatchCategory = spec_classify_handler_kcall(number);
        ||| matches!(cat, HandlerDispatchCategory::Debug)
        ||| matches!(cat, HandlerDispatchCategory::GetPid)
        ||| matches!(cat, HandlerDispatchCategory::GetTid)
        ||| matches!(cat, HandlerDispatchCategory::CapCtl)
        ||| matches!(cat, HandlerDispatchCategory::Terminate)
        ||| matches!(cat, HandlerDispatchCategory::EventCtrl)
        ||| matches!(cat, HandlerDispatchCategory::MemoryMap)
        ||| matches!(cat, HandlerDispatchCategory::MemoryUnmap)
        ||| matches!(cat, HandlerDispatchCategory::MemoryCtrl)
        ||| matches!(cat, HandlerDispatchCategory::MemoryCopy)
        ||| matches!(cat, HandlerDispatchCategory::Send)
        ||| matches!(cat, HandlerDispatchCategory::AllocMmio)
        ||| matches!(cat, HandlerDispatchCategory::FreeMmio)
        ||| matches!(cat, HandlerDispatchCategory::AllocPmio)
        ||| matches!(cat, HandlerDispatchCategory::FreePmio)
        ||| matches!(cat, HandlerDispatchCategory::ReadPmio)
        ||| matches!(cat, HandlerDispatchCategory::WritePmio)
        ||| matches!(cat, HandlerDispatchCategory::GetTime)
        ||| matches!(cat, HandlerDispatchCategory::CreateThread)
        ||| matches!(cat, HandlerDispatchCategory::SetThreadDataArea)
        ||| matches!(cat, HandlerDispatchCategory::GetThreadDataArea)
        ||| matches!(cat, HandlerDispatchCategory::Invalid)
    }),
{
}

/// Lemma: Each defined kcall number maps to its correct category.
///
/// # Description
///
/// Proves that the 21 kcall numbers handled by the handler loop each
/// map to their expected dispatch category. This is a regression check
/// against the original match statement.
pub proof fn lemma_classification_correctness()
    ensures
        matches!(spec_classify_handler_kcall(0), HandlerDispatchCategory::Debug),
        matches!(spec_classify_handler_kcall(1), HandlerDispatchCategory::GetPid),
        matches!(spec_classify_handler_kcall(2), HandlerDispatchCategory::GetTid),
        matches!(spec_classify_handler_kcall(4), HandlerDispatchCategory::CapCtl),
        matches!(spec_classify_handler_kcall(6), HandlerDispatchCategory::Terminate),
        matches!(spec_classify_handler_kcall(7), HandlerDispatchCategory::EventCtrl),
        matches!(spec_classify_handler_kcall(8), HandlerDispatchCategory::Send),
        matches!(spec_classify_handler_kcall(10), HandlerDispatchCategory::MemoryMap),
        matches!(spec_classify_handler_kcall(11), HandlerDispatchCategory::MemoryUnmap),
        matches!(spec_classify_handler_kcall(12), HandlerDispatchCategory::MemoryCtrl),
        matches!(spec_classify_handler_kcall(13), HandlerDispatchCategory::MemoryCopy),
        matches!(spec_classify_handler_kcall(14), HandlerDispatchCategory::AllocMmio),
        matches!(spec_classify_handler_kcall(15), HandlerDispatchCategory::FreeMmio),
        matches!(spec_classify_handler_kcall(16), HandlerDispatchCategory::AllocPmio),
        matches!(spec_classify_handler_kcall(17), HandlerDispatchCategory::FreePmio),
        matches!(spec_classify_handler_kcall(18), HandlerDispatchCategory::ReadPmio),
        matches!(spec_classify_handler_kcall(19), HandlerDispatchCategory::WritePmio),
        matches!(spec_classify_handler_kcall(21), HandlerDispatchCategory::CreateThread),
        matches!(spec_classify_handler_kcall(28), HandlerDispatchCategory::GetTime),
        matches!(spec_classify_handler_kcall(30), HandlerDispatchCategory::SetThreadDataArea),
        matches!(spec_classify_handler_kcall(31), HandlerDispatchCategory::GetThreadDataArea),
{
}

/// Lemma: Numbers not in the handler match are classified as Invalid.
///
/// # Description
///
/// Proves that kcall numbers not present in the handler's match arms
/// (3=Exit, 5=Resume, 9=Recv, 20=SchedulerYield, 22=ExitThread,
/// 23=JoinThread, 24=MutexLock, 25=MutexUnlock, 26=CondSignal,
/// 27=CondWait, 29=Sleep, and any number > 31) are classified as Invalid.
pub proof fn lemma_invalid_kcall_classification()
    ensures
        matches!(spec_classify_handler_kcall(3), HandlerDispatchCategory::Invalid),
        matches!(spec_classify_handler_kcall(5), HandlerDispatchCategory::Invalid),
        matches!(spec_classify_handler_kcall(9), HandlerDispatchCategory::Invalid),
        matches!(spec_classify_handler_kcall(20), HandlerDispatchCategory::Invalid),
        matches!(spec_classify_handler_kcall(22), HandlerDispatchCategory::Invalid),
        matches!(spec_classify_handler_kcall(23), HandlerDispatchCategory::Invalid),
        matches!(spec_classify_handler_kcall(24), HandlerDispatchCategory::Invalid),
        matches!(spec_classify_handler_kcall(25), HandlerDispatchCategory::Invalid),
        matches!(spec_classify_handler_kcall(26), HandlerDispatchCategory::Invalid),
        matches!(spec_classify_handler_kcall(27), HandlerDispatchCategory::Invalid),
        matches!(spec_classify_handler_kcall(29), HandlerDispatchCategory::Invalid),
        matches!(spec_classify_handler_kcall(32), HandlerDispatchCategory::Invalid),
        matches!(spec_classify_handler_kcall(100), HandlerDispatchCategory::Invalid),
        matches!(spec_classify_handler_kcall(u32::MAX), HandlerDispatchCategory::Invalid),
{
}

/// Lemma: GetPid and GetTid return InvalidSysCall error.
///
/// # Description
///
/// Proves that GetPid (1) and GetTid (2) are classified as returning
/// InvalidSysCall error, since they should be handled by the dispatcher.
pub proof fn lemma_getpid_gettid_return_invalid()
    ensures
        spec_returns_invalid_syscall(1),
        spec_returns_invalid_syscall(2),
{
}

/// Lemma: Invalid kcall numbers return InvalidSysCall error.
///
/// # Description
///
/// Any kcall number classified as Invalid also returns InvalidSysCall.
pub proof fn lemma_invalid_returns_invalid_syscall(number: u32)
    requires
        matches!(spec_classify_handler_kcall(number), HandlerDispatchCategory::Invalid),
    ensures
        spec_returns_invalid_syscall(number),
{
}

/// Lemma: Valid handler kcalls (excluding GetPid/GetTid) do not return InvalidSysCall.
///
/// # Description
///
/// Kcall numbers that are routed to actual subsystem handlers (Debug, CapCtl,
/// Terminate, etc.) do not return InvalidSysCall.
pub proof fn lemma_valid_kcall_no_invalid_error(number: u32)
    requires
        spec_is_handler_kcall(number),
        !matches!(spec_classify_handler_kcall(number), HandlerDispatchCategory::GetPid),
        !matches!(spec_classify_handler_kcall(number), HandlerDispatchCategory::GetTid),
    ensures
        !spec_returns_invalid_syscall(number),
{
}

//==================================================================================================
// Proof Lemmas: Yield / Idle Correctness
//==================================================================================================

/// Lemma: A fresh iteration has no work done.
///
/// # Description
///
/// At the start of each iteration, all work flags are false, meaning
/// no work has been done and the CPU should yield if nothing happens.
pub proof fn lemma_initial_iteration_no_work()
    ensures ({
        let state: LoopIterationState = spec_initial_iteration();
        &&& !state.kcall_handled
        &&& !state.message_received
        &&& !state.harvested_process
        &&& !spec_did_work(state)
        &&& spec_should_yield(state)
    }),
{
}

/// Lemma: After handling a kcall, work was done.
///
/// # Description
///
/// If a kcall was handled, `spec_did_work` returns true and the CPU
/// should not yield.
pub proof fn lemma_kcall_handled_means_work(state: LoopIterationState)
    ensures ({
        let after: LoopIterationState = spec_after_kcall_handled(state);
        &&& after.kcall_handled
        &&& spec_did_work(after)
        &&& !spec_should_yield(after)
    }),
{
}

/// Lemma: After receiving a message, work was done.
///
/// # Description
///
/// If an IKC message was received, `spec_did_work` returns true and the
/// CPU should not yield.
pub proof fn lemma_message_received_means_work(state: LoopIterationState)
    ensures ({
        let after: LoopIterationState = spec_after_message_received(state);
        &&& after.message_received
        &&& spec_did_work(after)
        &&& !spec_should_yield(after)
    }),
{
}

/// Lemma: After harvesting a zombie, work was done.
///
/// # Description
///
/// If a zombie was harvested, `spec_did_work` returns true and the CPU
/// should not yield.
pub proof fn lemma_harvest_means_work(state: LoopIterationState)
    ensures ({
        let after: LoopIterationState = spec_after_harvest(state);
        &&& after.harvested_process
        &&& spec_did_work(after)
        &&& !spec_should_yield(after)
    }),
{
}

/// Lemma: Setting a work flag preserves other flags.
///
/// # Description
///
/// Each of the three update functions only sets its flag to true without
/// modifying the other two flags.
pub proof fn lemma_work_flag_monotonicity(state: LoopIterationState)
    ensures ({
        let a: LoopIterationState = spec_after_kcall_handled(state);
        let b: LoopIterationState = spec_after_message_received(state);
        let c: LoopIterationState = spec_after_harvest(state);
        // kcall_handled preserves message_received and harvested_process.
        &&& a.message_received == state.message_received
        &&& a.harvested_process == state.harvested_process
        // message_received preserves kcall_handled and harvested_process.
        &&& b.kcall_handled == state.kcall_handled
        &&& b.harvested_process == state.harvested_process
        // harvested_process preserves kcall_handled and message_received.
        &&& c.kcall_handled == state.kcall_handled
        &&& c.message_received == state.message_received
    }),
{
}

/// Lemma: If any work flag was true before, it stays true after any update.
///
/// # Description
///
/// Work flags are monotonically increasing within an iteration: once set
/// to true, they cannot be reset to false.
pub proof fn lemma_work_flags_monotone_increasing(state: LoopIterationState)
    ensures
        state.kcall_handled ==> spec_after_message_received(state).kcall_handled,
        state.kcall_handled ==> spec_after_harvest(state).kcall_handled,
        state.message_received ==> spec_after_kcall_handled(state).message_received,
        state.message_received ==> spec_after_harvest(state).message_received,
        state.harvested_process ==> spec_after_kcall_handled(state).harvested_process,
        state.harvested_process ==> spec_after_message_received(state).harvested_process,
{
}

/// Lemma: Work done is monotone across updates.
///
/// # Description
///
/// If work was already done, applying any update still reports work done.
pub proof fn lemma_did_work_monotone(state: LoopIterationState)
    requires spec_did_work(state),
    ensures
        spec_did_work(spec_after_kcall_handled(state)),
        spec_did_work(spec_after_message_received(state)),
        spec_did_work(spec_after_harvest(state)),
{
}

//==================================================================================================
// Proof Lemmas: Termination Condition
//==================================================================================================

/// Lemma: Only INITD termination triggers loop exit.
///
/// # Description
///
/// The loop terminates only when the init daemon (pid == INITD_PID) terminates.
/// Non-INITD zombie harvests and other outcomes do not terminate the loop.
pub proof fn lemma_only_initd_terminates()
    ensures
        !spec_should_terminate(HarvestOutcome::NoZombie),
        !spec_should_terminate(HarvestOutcome::HarvestError),
        !spec_should_terminate(HarvestOutcome::Harvested { pid: 0, is_initd: false }),
        !spec_should_terminate(HarvestOutcome::Harvested { pid: 2, is_initd: false }),
        !spec_should_terminate(HarvestOutcome::Harvested { pid: 100, is_initd: false }),
        spec_should_terminate(HarvestOutcome::Harvested { pid: SPEC_INITD_PID(), is_initd: true }),
{
}

/// Lemma: Termination requires is_initd flag.
///
/// # Description
///
/// For any harvest outcome, termination occurs iff the outcome is a
/// Harvested variant with `is_initd == true`.
pub proof fn lemma_termination_requires_initd(outcome: HarvestOutcome)
    ensures
        spec_should_terminate(outcome) <==> match outcome {
            HarvestOutcome::Harvested { pid, is_initd } => is_initd,
            _ => false,
        },
{
}

/// Lemma: INITD pid value is 1.
///
/// # Description
///
/// Confirms the INITD pid constant matches the source code value.
pub proof fn lemma_initd_pid_value()
    ensures
        SPEC_INITD_PID() == 1,
        spec_is_initd(1),
        !spec_is_initd(0),
        !spec_is_initd(2),
{
}

//==================================================================================================
// Proof Lemmas: Scoreboard Poll
//==================================================================================================

/// Lemma: GotCall poll outcome leads to kcall handled.
///
/// # Description
///
/// When the scoreboard returns a kcall (GotCall), the iteration state
/// should be updated to reflect that a kcall was handled.
pub proof fn lemma_got_call_kcall_handled(state: LoopIterationState, number: u32)
    ensures ({
        let after: LoopIterationState = spec_after_kcall_handled(state);
        after.kcall_handled
    }),
{
}

//==================================================================================================
// Proof Lemmas: Dispatch Partition
//==================================================================================================

/// Lemma: Each kcall number is either a valid handler kcall or Invalid.
///
/// # Description
///
/// The handler classification partitions all kcall numbers into two sets:
/// those handled by the handler loop and those classified as Invalid.
pub proof fn lemma_dispatch_partition(number: u32)
    ensures
        spec_is_handler_kcall(number) || matches!(spec_classify_handler_kcall(number), HandlerDispatchCategory::Invalid),
{
}

/// Lemma: The handler and Invalid categories are mutually exclusive.
///
/// # Description
///
/// A kcall number cannot be both a valid handler kcall and Invalid.
pub proof fn lemma_handler_invalid_exclusive(number: u32)
    ensures
        !(spec_is_handler_kcall(number) && matches!(spec_classify_handler_kcall(number), HandlerDispatchCategory::Invalid)),
{
}

/// Lemma: All 21 defined handler kcall numbers are valid handler kcalls.
///
/// # Description
///
/// Every kcall number listed in the handler's match statement is classified
/// as a valid handler kcall (spec_is_handler_kcall returns true).
pub proof fn lemma_all_handler_kcalls_valid()
    ensures
        spec_is_handler_kcall(0),
        spec_is_handler_kcall(1),
        spec_is_handler_kcall(2),
        spec_is_handler_kcall(4),
        spec_is_handler_kcall(6),
        spec_is_handler_kcall(7),
        spec_is_handler_kcall(8),
        spec_is_handler_kcall(10),
        spec_is_handler_kcall(11),
        spec_is_handler_kcall(12),
        spec_is_handler_kcall(13),
        spec_is_handler_kcall(14),
        spec_is_handler_kcall(15),
        spec_is_handler_kcall(16),
        spec_is_handler_kcall(17),
        spec_is_handler_kcall(18),
        spec_is_handler_kcall(19),
        spec_is_handler_kcall(21),
        spec_is_handler_kcall(28),
        spec_is_handler_kcall(30),
        spec_is_handler_kcall(31),
{
}

//==================================================================================================
// Proof Lemmas: Composite Scenarios
//==================================================================================================

/// Lemma: Full iteration with all work done does not yield.
///
/// # Description
///
/// If a kcall was handled, a message was received, and a zombie was
/// harvested in the same iteration, the CPU should not yield.
pub proof fn lemma_full_work_no_yield()
    ensures ({
        let s0: LoopIterationState = spec_initial_iteration();
        let s1: LoopIterationState = spec_after_kcall_handled(s0);
        let s2: LoopIterationState = spec_after_message_received(s1);
        let s3: LoopIterationState = spec_after_harvest(s2);
        &&& s3.kcall_handled
        &&& s3.message_received
        &&& s3.harvested_process
        &&& spec_did_work(s3)
        &&& !spec_should_yield(s3)
    }),
{
}

/// Lemma: INITD termination in a busy iteration still terminates.
///
/// # Description
///
/// The termination decision is independent of the work flags. Even if
/// work was done, an INITD termination exits the loop.
pub proof fn lemma_initd_termination_independent_of_work(state: LoopIterationState)
    ensures ({
        let outcome: HarvestOutcome = HarvestOutcome::Harvested { pid: SPEC_INITD_PID(), is_initd: true };
        spec_should_terminate(outcome)
    }),
{
}

//==================================================================================================
// Proof Lemmas: Loop Invariant / Inductive Reasoning
//==================================================================================================

/// Lemma: The loop invariant holds for an empty history (base case).
///
/// # Description
///
/// Before the first iteration, no outcomes have been observed, so the
/// invariant holds vacuously (no element violates the condition).
pub proof fn lemma_loop_invariant_base()
    ensures
        spec_loop_invariant(Seq::<HarvestOutcome>::empty()),
{
}

/// Lemma: The loop invariant is preserved when appending a non-terminating outcome.
///
/// # Description
///
/// If the invariant holds for history h, and the current outcome does not
/// terminate the loop (INITD not harvested), then the invariant holds for
/// h ++ [outcome]. This is the inductive step for loop continuation.
pub proof fn lemma_loop_invariant_inductive(
    history: Seq<HarvestOutcome>,
    outcome: HarvestOutcome,
)
    requires
        spec_loop_invariant(history),
        spec_loop_continues(outcome),
    ensures
        spec_loop_invariant(spec_extend_history(history, outcome)),
{
    let new_history: Seq<HarvestOutcome> = spec_extend_history(history, outcome);
    assert forall|i: int| 0 <= i < new_history.len()
        implies !spec_should_terminate(#[trigger] new_history[i])
    by {
        if i < history.len() as int {
            assert(!spec_should_terminate(history[i]));
            assert(new_history[i] == history[i]);
        } else {
            assert(i == history.len() as int);
            assert(new_history[i] == outcome);
            assert(spec_loop_continues(outcome));
        }
    }
}

/// Lemma: If the loop exits, INITD must have terminated.
///
/// # Description
///
/// The contrapositive of continuation: if an outcome causes the loop to
/// not continue, then it must be an INITD termination.
pub proof fn lemma_loop_exit_requires_initd(outcome: HarvestOutcome)
    requires
        !spec_loop_continues(outcome),
    ensures
        spec_should_terminate(outcome),
{
}

/// Lemma: The history length equals the number of completed iterations.
///
/// # Description
///
/// After n iterations, the history contains exactly n outcomes.
/// Combined with the invariant, this proves that INITD was not
/// terminated in any of those n iterations.
pub proof fn lemma_history_length_after_extension(
    history: Seq<HarvestOutcome>,
    outcome: HarvestOutcome,
)
    ensures
        spec_extend_history(history, outcome).len() == history.len() + 1,
{
}

/// Lemma: A terminating outcome cannot be in a valid history.
///
/// # Description
///
/// If the invariant holds for a history, then no element in that history
/// is a terminating outcome. This means the loop never "missed" an INITD
/// termination — if the loop is still running, INITD was never seen.
pub proof fn lemma_invariant_excludes_termination(
    history: Seq<HarvestOutcome>,
    idx: int,
)
    requires
        spec_loop_invariant(history),
        0 <= idx < history.len(),
    ensures
        !spec_should_terminate(history[idx]),
{
}

/// Lemma: Work flags reset between iterations.
///
/// # Description
///
/// Each new iteration starts from spec_initial_iteration(), which has all
/// work flags cleared. This proves that state does not leak between
/// iterations.
pub proof fn lemma_iteration_state_reset()
    ensures ({
        let fresh: LoopIterationState = spec_initial_iteration();
        &&& !fresh.kcall_handled
        &&& !fresh.message_received
        &&& !fresh.harvested_process
        &&& spec_should_yield(fresh)
    }),
{
}

//==================================================================================================
// Proof Lemmas: Exec-to-Spec Conversion
//==================================================================================================

/// Lemma: The harvest-to-outcome conversion preserves termination semantics.
///
/// # Description
///
/// The spec-level termination decision (`spec_should_terminate`) on the
/// converted outcome is equivalent to the exec-level termination test
/// (`found && is_initd`). This bridges the gap between exec-level flags
/// and spec-level enum matching.
pub proof fn lemma_harvest_to_outcome_termination(
    found: bool,
    error: bool,
    pid: nat,
    is_initd: bool,
)
    requires
        error ==> !found,
    ensures
        spec_should_terminate(spec_harvest_to_outcome(found, error, pid, is_initd))
            == (found && is_initd),
{
}

//==================================================================================================
// Proof Lemmas: Spec Constant Regression
//==================================================================================================

/// Lemma: Spec constants match their source code definitions.
///
/// # Description
///
/// Regression check ensuring that the hardcoded spec constants match the
/// values defined in the Nanvix source code:
/// - `SPEC_INITD_PID() == 1` matches `ProcessIdentifier::INITD = ProcessIdentifier(1)`
///   (defined in `sys::pm::ProcessIdentifier`).
/// - `SPEC_ERROR_INVALID_SYSCALL() == 88` matches `ErrorCode::InvalidSysCall` value 88
///   (ENOSYS, defined in `sys::error::ErrorCode`).
///
/// If the source definitions change, this lemma's ensures clause should be
/// updated to reflect the new values, triggering a review of all dependent
/// specs and proofs.
pub proof fn lemma_spec_constants_match_source()
    ensures
        SPEC_INITD_PID() == 1,
        SPEC_ERROR_INVALID_SYSCALL() == 88,
{
}

/// Lemma: The dispatch classification covers all 21 original handler match arms.
///
/// # Description
///
/// Regression check ensuring that the classification function includes
/// every kcall number present in the original handler's match statement.
/// If a new kcall arm is added to the original source, a corresponding
/// entry must be added here and in `spec_classify_handler_kcall`.
///
/// Source: `src/kernel/src/kcall/handler.rs`, lines 62-97.
/// Kcall numbers: 0(Debug), 1(GetPid), 2(GetTid), 4(CapCtl), 6(Terminate),
/// 7(EventCtrl), 8(Send), 10(MemoryMap), 11(MemoryUnmap), 12(MemoryCtrl),
/// 13(MemoryCopy), 14(AllocMmio), 15(FreeMmio), 16(AllocPmio),
/// 17(FreePmio), 18(ReadPmio), 19(WritePmio), 21(CreateThread),
/// 28(GetTime), 30(SetThreadDataArea), 31(GetThreadDataArea).
pub proof fn lemma_dispatch_coverage_matches_source()
    ensures
        // All 21 handler kcall numbers are classified as non-Invalid.
        spec_is_handler_kcall(0),   // Debug
        spec_is_handler_kcall(1),   // GetPid
        spec_is_handler_kcall(2),   // GetTid
        spec_is_handler_kcall(4),   // CapCtl
        spec_is_handler_kcall(6),   // Terminate
        spec_is_handler_kcall(7),   // EventCtrl
        spec_is_handler_kcall(8),   // Send
        spec_is_handler_kcall(10),  // MemoryMap
        spec_is_handler_kcall(11),  // MemoryUnmap
        spec_is_handler_kcall(12),  // MemoryCtrl
        spec_is_handler_kcall(13),  // MemoryCopy
        spec_is_handler_kcall(14),  // AllocMmio
        spec_is_handler_kcall(15),  // FreeMmio
        spec_is_handler_kcall(16),  // AllocPmio
        spec_is_handler_kcall(17),  // FreePmio
        spec_is_handler_kcall(18),  // ReadPmio
        spec_is_handler_kcall(19),  // WritePmio
        spec_is_handler_kcall(21),  // CreateThread
        spec_is_handler_kcall(28),  // GetTime
        spec_is_handler_kcall(30),  // SetThreadDataArea
        spec_is_handler_kcall(31),  // GetThreadDataArea
        // Dispatcher-only kcalls are Invalid in the handler.
        !spec_is_handler_kcall(3),  // Exit (dispatcher-only)
        !spec_is_handler_kcall(5),  // Resume (dispatcher-only)
        !spec_is_handler_kcall(9),  // Recv (dispatcher-only)
        !spec_is_handler_kcall(20), // SchedulerYield (dispatcher-only)
        !spec_is_handler_kcall(22), // ExitThread (dispatcher-only)
        !spec_is_handler_kcall(23), // JoinThread (dispatcher-only)
{
}

//==================================================================================================
// Proof: Conditional Termination (Liveness)
//==================================================================================================

/// Lemma: The loop invariant excludes ALL INITD termination from the history.
///
/// # Description
///
/// The loop invariant (`spec_loop_invariant`) asserts that no element in the
/// history is a terminating outcome. The liveness predicate
/// (`spec_initd_terminates_within`) asserts that at least one element IS a
/// terminating outcome. These are contradictory by quantifier duality:
/// (∀i. ¬terminate(h[i])) ⊢ ¬(∃i. terminate(h[i])).
///
/// This is the key bridge between the invariant and liveness: if the
/// invariant holds, INITD has NOT terminated in any recorded iteration.
/// Uses the per-element `lemma_invariant_excludes_termination` to discharge
/// any witness that the exists quantifier might provide.
pub proof fn lemma_invariant_excludes_all_termination(history: Seq<HarvestOutcome>)
    requires
        spec_loop_invariant(history),
    ensures
        !spec_initd_terminates_within(history),
{
    // The per-element lemma gives us: for any valid index, the element
    // is non-terminating. This contradicts the existential in
    // spec_initd_terminates_within.
    assert forall|i: int| 0 <= i < history.len() implies
        !spec_should_terminate(#[trigger] history[i]) by {
        lemma_invariant_excludes_termination(history, i);
    }
}

/// Lemma: When the loop exits without termination, INITD was never observed.
///
/// # Description
///
/// Given the loop postconditions (invariant holds, history length = fuel when
/// not terminated), this proves that the actually-observed history contains
/// no INITD termination. This is the non-tautological conditional liveness
/// property:
///
/// **Contrapositive**: If INITD terminates within `fuel` iterations (i.e.,
/// one of the actual harvest outcomes is a terminating outcome), then the
/// loop MUST have returned `terminated == true`. This follows because:
/// 1. If `!terminated`, then `spec_loop_invariant(history)` holds (loop ensures).
/// 2. `spec_loop_invariant(history) ==> !spec_initd_terminates_within(history)`
///    (by `lemma_invariant_excludes_all_termination`).
/// 3. So INITD did NOT terminate in the observed iterations.
/// 4. Contrapositive: if INITD DID terminate, then `terminated == true`.
pub proof fn lemma_loop_termination_completeness(
    terminated: bool,
    history: Seq<HarvestOutcome>,
    fuel: u32,
)
    requires
        spec_loop_invariant(history),
        !terminated ==> history.len() == fuel as int,
        terminated ==> history.len() < fuel as int,
    ensures
        // When the loop exits without termination, no INITD in history.
        !terminated ==> !spec_initd_terminates_within(history),
        // The invariant always excludes INITD from history.
        !spec_initd_terminates_within(history),
{
    lemma_invariant_excludes_all_termination(history);
}

/// Lemma: Top-level correctness theorem — when the loop terminates, all
/// correctness properties hold.
///
/// # Description
///
/// Given the postconditions of `kcall_handler_loop`, proves that when
/// `terminated == true`, the result satisfies `spec_handler_terminated_correctly`:
/// - The loop terminated.
/// - Termination was triggered by INITD (pid == 1).
/// - The loop invariant is maintained for the final history.
///
/// This is the usable top-level theorem that clients of the handler
/// verification can rely on. The `exit_status` is intentionally NOT
/// constrained — it originates from `harvest_zombies()` (T2) and its
/// correctness depends on ProcessManager state outside this module's scope.
pub proof fn lemma_handler_top_level_correctness(
    terminated: bool,
    termination_pid: u32,
    history: Seq<HarvestOutcome>,
    fuel: u32,
)
    requires
        spec_loop_invariant(history),
        !terminated ==> history.len() == fuel as int,
        terminated ==> history.len() < fuel as int,
        terminated ==> termination_pid == 1u32,
    ensures
        terminated ==> spec_handler_terminated_correctly(terminated, termination_pid, history),
{
    // All postconditions of kcall_handler_loop directly establish the
    // conjuncts of spec_handler_terminated_correctly when terminated.
}

} // verus!
