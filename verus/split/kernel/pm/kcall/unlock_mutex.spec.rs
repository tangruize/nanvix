// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Unlock Mutex Kernel Call Specification.
// Defines View types, spec constants, and spec functions for the unlock_mutex
// kernel call verification model.
//
// ## Verification Model
//
// The unlock_mutex kcall converts a user-provided `mutex_addr: usize` into a
// `MutexAddress`, then executes a single-step pipeline:
//   1. ProcessManager::take_mutex_guard(pid, tid, mutex_addr) → ()
//
// On success, the returned guard is immediately dropped (scope exit), which
// triggers `MutexGuard::drop()` and unlocks the mutex. On failure, the error
// from `take_mutex_guard` is propagated via `?`.
//
// This spec models:
// - The single-step pipeline with two outcomes (success or error).
// - Guard drop semantics: on success, the guard is consumed by scope exit.
// - Error propagation: take_mutex_guard errors propagated unchanged.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Spec Constants
//==================================================================================================

/// Maximum value for usize on x86-32.
pub open spec fn USIZE_MAX_X86_32() -> nat {
    u32::MAX as nat
}

/// Number of bits in usize on the target architecture.
///
/// # Description
///
/// On x86-32, usize is 32 bits. This constant makes the architecture
/// assumption explicit so that `lemma_architecture_guard` can verify it.
pub open spec fn USIZE_BITS() -> nat {
    32
}

/// Spec predicate: whether an error code is a valid `ErrorCode` discriminant.
///
/// # Description
///
/// All `ErrorCode` enum variants are positive integers (POSIX errno values).
/// This predicate captures the essential invariant: all error codes are
/// positive (non-zero).
pub open spec fn spec_is_valid_error_code(code: int) -> bool {
    code > 0
}

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of the outcome of ProcessManager::take_mutex_guard.
#[verifier::ext_equal]
pub enum TakeMutexGuardOutcomeView {
    /// take_mutex_guard succeeded, returning a MutexGuard that is dropped.
    TgOk,
    /// take_mutex_guard failed with an error code.
    TgError { error_code: int },
}

/// Abstract view of the unlock_mutex kcall's final result.
///
/// # Description
///
/// The unlock_mutex function returns Result<(), Error>. This view models
/// the two possible outcomes:
/// - Success: take_mutex_guard succeeded and the guard was dropped.
/// - TakeMutexGuardError: take_mutex_guard failed with an error.
#[verifier::ext_equal]
pub enum UnlockMutexResultView {
    /// take_mutex_guard succeeded; guard dropped, mutex unlocked.
    Success,
    /// ProcessManager::take_mutex_guard failed.
    TakeMutexGuardError { error_code: int },
}

//==================================================================================================
// Spec Functions
//==================================================================================================

/// Spec function: models the complete unlock_mutex pipeline.
///
/// # Description
///
/// The unlock_mutex function executes a single-step pipeline:
/// 1. take_mutex_guard → TakeMutexGuardError on failure.
/// 2. Success → guard dropped, mutex unlocked.
pub open spec fn spec_unlock_mutex_result(
    take_guard_outcome: TakeMutexGuardOutcomeView,
) -> UnlockMutexResultView {
    match take_guard_outcome {
        TakeMutexGuardOutcomeView::TgError { error_code } => {
            UnlockMutexResultView::TakeMutexGuardError { error_code }
        },
        TakeMutexGuardOutcomeView::TgOk => {
            UnlockMutexResultView::Success
        },
    }
}

/// Spec function: whether the result is success.
pub open spec fn spec_is_success(result: UnlockMutexResultView) -> bool {
    matches!(result, UnlockMutexResultView::Success)
}

/// Spec function: whether the result is any kind of error.
///
/// # Description
///
/// Defined as the complement of `spec_is_success`.
pub open spec fn spec_is_error(result: UnlockMutexResultView) -> bool {
    !spec_is_success(result)
}

/// Spec function: whether the result is a take_mutex_guard error.
pub open spec fn spec_is_take_guard_error(result: UnlockMutexResultView) -> bool {
    matches!(result, UnlockMutexResultView::TakeMutexGuardError { .. })
}

/// Spec function: pipeline result parameterized by caller context (pid/tid).
///
/// # Description
///
/// Wraps `spec_unlock_mutex_result` to include `pid` and `tid` in the
/// signature, matching the original `unlock_mutex(pid, tid, mutex_addr)`.
/// In the original code, `pid` and `tid` are passed to
/// `ProcessManager::take_mutex_guard`, so they DO affect the PM outcome.
/// However, from the pipeline's perspective, they only influence which
/// `TakeMutexGuardOutcomeView` the PM produces — the pipeline itself
/// maps outcomes identically regardless of pid/tid. This wrapper exists
/// to prove that the pipeline mapping is independent of pid/tid even
/// though the PM outcome may depend on them.
pub open spec fn spec_unlock_mutex_result_with_context(
    pid: nat,
    tid: nat,
    take_guard_outcome: TakeMutexGuardOutcomeView,
) -> UnlockMutexResultView {
    // pid and tid influence PM behavior (which outcome is produced),
    // but the pipeline mapping from outcome to result is the same.
    spec_unlock_mutex_result(take_guard_outcome)
}

//==================================================================================================
// Caller Safety Contract Spec Predicates
//==================================================================================================

/// Spec predicate: the caller does not hold a ProcessManager reference.
///
/// # Description
///
/// Encodes the safety requirement from the original `unlock_mutex` function:
/// "The calling process does not hold a reference to the process manager."
///
/// This prevents re-entrancy issues where `unlock_mutex` internally accesses
/// the ProcessManager (via `take_mutex_guard`) while the caller still holds
/// a mutable reference.
pub uninterp spec fn spec_caller_no_pm_reference() -> bool;

/// Spec predicate: all safety requirements are satisfied.
///
/// # Description
///
/// Convenience predicate for the safety requirement from the original
/// `unlock_mutex` function's `# Safety` documentation. Call sites should
/// establish this predicate before invoking `unlock_mutex`.
pub open spec fn spec_unlock_mutex_safety_preconditions() -> bool {
    spec_caller_no_pm_reference()
}

/// Spec predicate: the guard for the given mutex has been dropped (mutex unlocked).
///
/// # Description
///
/// Models the outcome of `MutexGuard::drop()` — the guard for `mutex_addr`
/// has been consumed and the mutex lock released. In the original code,
/// `take_mutex_guard` returns the `MutexGuard` by value. The guard is then
/// immediately dropped at scope exit (the `?` discards it on success),
/// triggering `MutexGuard::drop()` which unlocks the mutex.
///
/// This predicate is abstract because the concrete unlock semantics are
/// defined in the mutex module, not in this pipeline.
pub uninterp spec fn spec_guard_dropped_and_mutex_unlocked(mutex_addr: nat) -> bool;

} // verus!
