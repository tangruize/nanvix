// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Unlock Mutex Kernel Call Proofs.
// Proof lemmas for the unlock_mutex kcall verification model.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Proof Functions
//==================================================================================================

/// Proof: take_mutex_guard error propagates as the final result.
///
/// # Description
///
/// If `ProcessManager::take_mutex_guard` fails, the error is propagated
/// unchanged via the `?` operator. This lemma proves the spec correctly
/// models this as `TakeMutexGuardError`.
pub proof fn lemma_take_guard_error_propagates(error_code: int)
    ensures
        spec_unlock_mutex_result(
            TakeMutexGuardOutcomeView::TgError { error_code },
        ) == (UnlockMutexResultView::TakeMutexGuardError { error_code }),
        spec_is_take_guard_error(
            spec_unlock_mutex_result(
                TakeMutexGuardOutcomeView::TgError { error_code },
            )
        ),
{
}

/// Proof: success requires take_mutex_guard to succeed.
///
/// # Description
///
/// The result is Success if and only if take_mutex_guard returned TgOk.
pub proof fn lemma_success_requires_take_guard_ok(
    take_guard_outcome: TakeMutexGuardOutcomeView,
)
    ensures
        spec_is_success(
            spec_unlock_mutex_result(take_guard_outcome)
        ) <==> matches!(take_guard_outcome, TakeMutexGuardOutcomeView::TgOk),
{
    match take_guard_outcome {
        TakeMutexGuardOutcomeView::TgOk => {},
        TakeMutexGuardOutcomeView::TgError { .. } => {},
    }
}

/// Proof: the result is always exactly one of success or error.
///
/// # Description
///
/// The unlock_mutex result is exhaustive: every possible input produces
/// exactly one result category (success or take_guard error). Success
/// and error are mutually exclusive.
pub proof fn lemma_result_exhaustive(
    take_guard_outcome: TakeMutexGuardOutcomeView,
)
    ensures
        ({
            let result: UnlockMutexResultView = spec_unlock_mutex_result(take_guard_outcome);
            // Exhaustive: every result is either success or take_guard error.
            spec_is_success(result) || spec_is_take_guard_error(result)
        }),
        ({
            let result: UnlockMutexResultView = spec_unlock_mutex_result(take_guard_outcome);
            // Mutual exclusion: success and error are disjoint.
            !(spec_is_success(result) && spec_is_error(result))
        }),
{
    match take_guard_outcome {
        TakeMutexGuardOutcomeView::TgOk => {},
        TakeMutexGuardOutcomeView::TgError { .. } => {},
    }
}

/// Proof: on success, the guard is dropped and the mutex is unlocked.
///
/// # Description
///
/// When `take_mutex_guard` succeeds, the returned `MutexGuard` is immediately
/// dropped at scope exit (the original code uses `?` which discards the `()`
/// on success). The guard's `Drop` implementation unlocks the mutex.
///
/// This lemma connects the pipeline result to the guard-drop guarantee:
/// when the result is `Success`, the `guard_dropped` precondition (which
/// comes from the `take_mutex_guard_model`'s postcondition on the success
/// path) ensures the mutex was unlocked.
pub proof fn lemma_guard_dropped_on_success(
    take_guard_outcome: TakeMutexGuardOutcomeView,
    guard_dropped: bool,
)
    requires
        take_guard_outcome == TakeMutexGuardOutcomeView::TgOk,
        // From take_mutex_guard_model's postcondition on success.
        guard_dropped,
    ensures
        spec_is_success(spec_unlock_mutex_result(take_guard_outcome)),
        guard_dropped,
{
}

/// Proof: the pipeline mapping is independent of pid and tid.
///
/// # Description
///
/// Although `pid` and `tid` are passed to `ProcessManager::take_mutex_guard`
/// and influence which outcome the PM produces, the pipeline's mapping from
/// outcome to result is the same for any (pid, tid) pair. This lemma proves
/// that `spec_unlock_mutex_result_with_context` returns the same result for
/// any two (pid, tid) pairs given the same take_guard outcome.
pub proof fn lemma_result_mapping_independent_of_pid_tid(
    pid1: nat,
    pid2: nat,
    tid1: nat,
    tid2: nat,
    take_guard_outcome: TakeMutexGuardOutcomeView,
)
    ensures
        spec_unlock_mutex_result_with_context(pid1, tid1, take_guard_outcome)
            == spec_unlock_mutex_result_with_context(pid2, tid2, take_guard_outcome),
{
}

/// Proof: architecture guard — USIZE_BITS is 32 and USIZE_MAX matches u32::MAX.
///
/// # Description
///
/// Makes the x86-32 architecture assumption explicit and verifiable.
pub proof fn lemma_architecture_guard()
    ensures
        USIZE_BITS() == 32,
        USIZE_MAX_X86_32() == u32::MAX as nat,
        USIZE_MAX_X86_32() == 4294967295nat,
{
}

/// Proof: the safety preconditions predicate is well-formed.
///
/// # Description
///
/// Verifies that `spec_unlock_mutex_safety_preconditions` correctly composes
/// the uninterpreted predicate. If the predicate changes its arity or type,
/// this lemma will fail to verify.
pub proof fn lemma_safety_preconditions_well_formed()
    ensures
        spec_unlock_mutex_safety_preconditions() ==> spec_caller_no_pm_reference(),
{
}

/// Proof: on the error path, no guard exists to leak.
///
/// # Description
///
/// When `take_mutex_guard` fails, no `MutexGuard` was returned, so there is
/// nothing to drop and no risk of resource leak. The mutex state is unchanged.
pub proof fn lemma_no_guard_leak_on_error(
    take_guard_outcome: TakeMutexGuardOutcomeView,
)
    requires
        !matches!(take_guard_outcome, TakeMutexGuardOutcomeView::TgOk),
    ensures
        spec_is_error(spec_unlock_mutex_result(take_guard_outcome)),
        spec_is_take_guard_error(spec_unlock_mutex_result(take_guard_outcome)),
{
    match take_guard_outcome {
        TakeMutexGuardOutcomeView::TgError { .. } => {},
        TakeMutexGuardOutcomeView::TgOk => {},
    }
}

/// Proof: success and error are complementary and exhaustive.
///
/// # Description
///
/// For any take_guard outcome, the result is either success or error, and
/// these two categories are both mutually exclusive and jointly exhaustive.
pub proof fn lemma_success_error_complementary(
    take_guard_outcome: TakeMutexGuardOutcomeView,
)
    ensures
        ({
            let result: UnlockMutexResultView = spec_unlock_mutex_result(take_guard_outcome);
            spec_is_success(result) <==> !spec_is_error(result)
        }),
{
    match take_guard_outcome {
        TakeMutexGuardOutcomeView::TgOk => {},
        TakeMutexGuardOutcomeView::TgError { .. } => {},
    }
}

/// Proof: error code is preserved through the pipeline.
///
/// # Description
///
/// When `take_mutex_guard` fails with error code `ec`, the final result
/// contains exactly the same error code `ec` — no wrapping or transformation.
pub proof fn lemma_error_code_preserved(error_code: int)
    ensures
        spec_unlock_mutex_result(
            TakeMutexGuardOutcomeView::TgError { error_code },
        ) == (UnlockMutexResultView::TakeMutexGuardError { error_code }),
{
}

} // verus!
