// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Unlock Mutex Kernel Call Verification Model
//!
//! Formal verification of the unlock_mutex kernel call (`pm::kcall::unlock_mutex`).
//!
//! ## Overview
//!
//! The `unlock_mutex(pid, tid, mutex_addr)` function unlocks a mutex by
//! retrieving and dropping the mutex guard. It:
//! 1. Converts `mutex_addr` to a `MutexAddress` (type wrapper, not modeled).
//! 2. Calls `ProcessManager::take_mutex_guard(pid, tid, mutex_addr)`.
//! 3. The returned `MutexGuard` is immediately dropped (via `?` discarding `()`),
//!    which triggers `MutexGuard::drop()` and unlocks the mutex.
//!
//! ## Note on Parameters
//!
//! The original `unlock_mutex()` takes `pid: ProcessIdentifier`,
//! `tid: ThreadIdentifier`, and `mutex_addr: usize`. All three are passed to
//! `ProcessManager::take_mutex_guard`. The `pid` and `tid` affect which PM
//! outcome is produced (e.g., only the owning thread can unlock), but the
//! pipeline mapping from outcome to result is independent of pid/tid.
//!
//! ## Verified Properties
//!
//! - **Error propagation**: take_mutex_guard errors propagate unchanged
//!   (`lemma_take_guard_error_propagates`, `lemma_error_code_preserved`).
//! - **Success requires take_guard OK**: Success iff take_mutex_guard succeeds
//!   (`lemma_success_requires_take_guard_ok`).
//! - **Result exhaustiveness**: Every input produces exactly one result category
//!   (success or take_guard error) (`lemma_result_exhaustive`).
//! - **Guard drop on success**: On success, the guard is dropped and the mutex
//!   is unlocked (`lemma_guard_dropped_on_success`).
//! - **No guard leak on error**: On error, no guard exists to leak
//!   (`lemma_no_guard_leak_on_error`).
//! - **Success/error complementary**: Success and error are mutually exclusive
//!   and jointly exhaustive (`lemma_success_error_complementary`).
//! - **Pipeline mapping independence**: The pipeline mapping from outcome to
//!   result is independent of pid/tid (`lemma_result_mapping_independent_of_pid_tid`).
//! - **Architecture guard**: x86-32 assumption verified
//!   (`lemma_architecture_guard`).
//! - **Safety predicate well-formedness**: The composite safety predicate
//!   correctly decomposes (`lemma_safety_preconditions_well_formed`).
//! - **Exec model correctness**: `unlock_mutex_model` matches
//!   `spec_unlock_mutex_result` for all inputs and PM outcomes.
//!
//! ## Properties NOT Proven Here (Out of Scope)
//!
//! - **Mutex unlock correctness**: The internal state machine of
//!   MutexGuard::drop() is verified in the mutex module.
//! - **ProcessManager correctness**: take_mutex_guard internals are verified
//!   in the PM module.
//! - **Ownership validation**: Whether the calling thread actually owns the
//!   mutex is a PM concern.
//! - **MutexAddress validation**: Address validity is a PM concern.
//! - **Unsafe safety contract**: The original function requires "the calling
//!   process does not hold a reference to the process manager." This is
//!   modeled as an abstract predicate `spec_caller_no_pm_reference`.
//!
//! ## Verification Model
//!
//! The original function uses:
//! - `MutexAddress::from(usize)` → modeled as opaque type construction.
//! - `ProcessManager::take_mutex_guard()` → modeled via
//!   `take_mutex_guard_model()` `external_body`.
//!
//! The exec-level `unlock_mutex_model()` mirrors the original control flow and
//! proves that the result matches `spec_unlock_mutex_result` for all inputs
//! and all PM outcomes.
//!
//! ## Trust Boundaries
//!
//! - **T1: `ProcessManager::take_mutex_guard()`**. Returns the mutex guard for
//!   the given (pid, tid, mutex_addr). The PM module verifies this function's
//!   correctness internally. Modeled as `external_body`.
//!
//! ## API Mapping
//!
//! | Original API                                  | Verified Model                    | Notes          |
//! |-----------------------------------------------|-----------------------------------|----------------|
//! | `MutexAddress::from(usize)`                   | (not modeled)                     | Type wrapper.  |
//! | `ProcessManager::take_mutex_guard(pid,tid,a)`  | `take_mutex_guard_model(addr)`    | external_body. |
//! | `pub unsafe fn unlock_mutex(pid,tid,addr)`    | `unlock_mutex_model(addr)`        | Fully verified.|

use vstd::prelude::*;

// Include specifications.
include!("unlock_mutex.spec.rs");

// Include proofs.
include!("unlock_mutex.proof.rs");

verus! {

//==================================================================================================
// Dependency Models (External Bodies)
//==================================================================================================

/// Model of the TakeMutexGuard step outcome.
///
/// # Description
///
/// Represents the result of `ProcessManager::take_mutex_guard(pid, tid, mutex_addr)`.
/// On success, a `MutexGuard` is returned. On failure, an `Error` is returned.
/// The exec model uses this enum to capture the PM outcome.
pub enum TakeMutexGuardOutcomeModel {
    /// take_mutex_guard succeeded, returning a MutexGuard.
    Ok,
    /// take_mutex_guard failed with an error code.
    Error { error_code: i32 },
}

impl TakeMutexGuardOutcomeModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> TakeMutexGuardOutcomeView {
        match self {
            TakeMutexGuardOutcomeModel::Ok => TakeMutexGuardOutcomeView::TgOk,
            TakeMutexGuardOutcomeModel::Error { error_code } => {
                TakeMutexGuardOutcomeView::TgError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the unlock_mutex overall result.
///
/// # Description
///
/// Represents the final result of the unlock_mutex kcall, mirroring
/// `Result<(), Error>` from the original.
pub enum UnlockMutexResultModel {
    /// take_mutex_guard succeeded; guard dropped, mutex unlocked.
    Success,
    /// ProcessManager::take_mutex_guard failed.
    TakeMutexGuardError { error_code: i32 },
}

impl UnlockMutexResultModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> UnlockMutexResultView {
        match self {
            UnlockMutexResultModel::Success => UnlockMutexResultView::Success,
            UnlockMutexResultModel::TakeMutexGuardError { error_code } => {
                UnlockMutexResultView::TakeMutexGuardError { error_code: *error_code as int }
            },
        }
    }
}

//==================================================================================================
// External Body Functions (Trust Boundaries)
//==================================================================================================

/// Trust Boundary T1: Models `ProcessManager::take_mutex_guard(pid, tid, mutex_addr)`.
///
/// # Description
///
/// Retrieves the mutex guard for the given mutex address, owned by the
/// specified (pid, tid). The PM module verifies this function's correctness
/// internally.
///
/// In the original code, pid and tid determine ownership validation.
/// The model abstracts these away since ownership checking is a PM concern.
///
/// ## Guard Drop Semantics
///
/// On success, the original `take_mutex_guard` returns `MutexGuard` by value
/// (inside `Ok`). The caller (unlock_mutex) uses `?` which extracts the `()`
/// (the guard is returned but the `?` on Result<(), Error> discards it).
/// Actually, looking more carefully: `take_mutex_guard` returns
/// `Result<(), Error>` — it takes the guard internally and drops it.
/// The comment in the original says "The mutex guard is dropped, causing
/// threads to be notified."
///
/// # Parameters
///
/// - `mutex_addr`: The mutex address (from `MutexAddress::from(usize)`).
#[verifier::external_body]
pub fn take_mutex_guard_model(mutex_addr: u32) -> (result: TakeMutexGuardOutcomeModel)
    ensures
        // Error codes from the PM module are valid ErrorCode discriminants.
        result matches TakeMutexGuardOutcomeModel::Error { error_code }
            ==> spec_is_valid_error_code(error_code as int),
        // On success, the guard was dropped and the mutex is unlocked.
        result matches TakeMutexGuardOutcomeModel::Ok
            ==> spec_guard_dropped_and_mutex_unlocked(mutex_addr as nat),
{
    unimplemented!()
}

//==================================================================================================
// Verified Functions
//==================================================================================================

/// Verified exec model of the `unlock_mutex` kernel call.
///
/// # Description
///
/// This function mirrors the original `pub unsafe fn unlock_mutex(pid, tid, mutex_addr)`
/// control flow. It:
/// 1. Calls take_mutex_guard (external).
/// 2. Returns Ok(()) on success (guard was dropped by PM).
/// 3. Returns the error on failure.
///
/// The original also takes `pid` and `tid` parameters, which are passed to
/// `ProcessManager::take_mutex_guard`. They affect the PM outcome but not the
/// pipeline mapping. They are omitted from this model; the PM's outcome is
/// captured via the ghost return.
///
/// # Parameters
///
/// - `mutex_addr`: Mutex address (from `MutexAddress::from(usize)`).
///
/// # Returns
///
/// A tuple of (result, ghost take_guard_outcome) where the ghost captures
/// the PM outcome for postcondition linking.
pub fn unlock_mutex_model(mutex_addr: u32) -> (ret: (
    UnlockMutexResultModel,
    Ghost<TakeMutexGuardOutcomeView>,
))
    requires
        // ABI constraint: mutex_addr originates from 32-bit usize on x86-32.
        mutex_addr as nat <= USIZE_MAX_X86_32(),
    ensures
        // The result matches the spec for the captured PM outcome.
        ret.0.spec_view() == spec_unlock_mutex_result(ret.1@),
        // Success only when take_mutex_guard succeeds.
        spec_is_success(ret.0.spec_view()) ==>
            ret.1@ == TakeMutexGuardOutcomeView::TgOk,
        // Error only when take_mutex_guard fails.
        spec_is_error(ret.0.spec_view()) ==>
            !(ret.1@ == TakeMutexGuardOutcomeView::TgOk),
        // On success, guard was dropped and mutex was unlocked.
        spec_is_success(ret.0.spec_view()) ==>
            spec_guard_dropped_and_mutex_unlocked(mutex_addr as nat),
{
    // Step 1: Take mutex guard (external).
    let tg_result: TakeMutexGuardOutcomeModel = take_mutex_guard_model(mutex_addr);
    let ghost tg_view: TakeMutexGuardOutcomeView = tg_result.spec_view();

    match tg_result {
        TakeMutexGuardOutcomeModel::Error { error_code } => {
            (
                UnlockMutexResultModel::TakeMutexGuardError { error_code },
                Ghost(tg_view),
            )
        },
        TakeMutexGuardOutcomeModel::Ok => {
            (
                UnlockMutexResultModel::Success,
                Ghost(tg_view),
            )
        },
    }
}

} // verus!
