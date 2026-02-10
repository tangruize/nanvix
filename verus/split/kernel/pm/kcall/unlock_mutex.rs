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
//! 3. The returned `MutexGuard` is immediately dropped at the semicolon
//!    (the `?` operator extracts the `MutexGuard` from `Ok`, and since
//!    it is not bound to a variable, it is dropped), triggering
//!    `MutexGuard::drop()` which unlocks the mutex.
//!
//! ## Note on Parameters
//!
//! The original `unlock_mutex()` takes `pid: ProcessIdentifier`,
//! `tid: ThreadIdentifier`, and `mutex_addr: usize`. All three are passed to
//! `ProcessManager::take_mutex_guard`. The `pid` and `tid` affect which PM
//! outcome is produced (e.g., only the owning thread can unlock), but the
//! pipeline mapping from outcome to result is independent of pid/tid.
//! Ghost `pid` and `tid` parameters are included in the model for future
//! enrichment of the PM trust boundary with ownership constraints.
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
//!   is unlocked. Modeled via a separate `drop_guard_model` step that consumes
//!   the ghost guard token and establishes `spec_guard_dropped_and_mutex_unlocked`
//!   (`lemma_guard_dropped_on_success`).
//! - **Ownership on success**: On success, `spec_thread_owns_mutex(pid, tid,
//!   mutex_addr)` is established, proving the calling thread owned the mutex.
//!   This property is propagated from the `take_mutex_guard_model` contract
//!   to the kcall boundary.
//! - **No guard leak on error**: On error, no guard token is returned to the
//!   caller (`lemma_no_guard_leak_on_error`).
//! - **Implicit unlock on error**: On error, the mutex may have been implicitly
//!   unlocked by PM-internal guard drops. The PM's `take_mutex_guard` can fail
//!   at `put_mutex` after extracting the guard, causing `MutexGuard::drop()` to
//!   run inside the PM. This is modeled via `spec_mutex_may_be_unlocked_on_error`.
//! - **Guard token chain**: The guard token produced by `take_mutex_guard_model`
//!   is consumed by `drop_guard_model`, formalizing the ownership chain
//!   (`lemma_guard_token_chain`).
//! - **Success/error complementary**: Success and error are mutually exclusive
//!   and jointly exhaustive (`lemma_success_error_complementary`).
//! - **Pipeline mapping independence**: The pipeline mapping from outcome to
//!   result is independent of pid/tid (`lemma_result_mapping_independent_of_pid_tid`).
//! - **Architecture guard**: x86-32 assumption verified
//!   (`lemma_architecture_guard`).
//! - **Safety precondition enforcement**: The `unsafe` safety contract is
//!   enforced as a `requires` clause on `unlock_mutex_model` and
//!   `take_mutex_guard_model` (`lemma_safety_preconditions_well_formed`).
//! - **Exec model correctness**: `unlock_mutex_model` matches
//!   `spec_unlock_mutex_result` for all inputs and PM outcomes.
//!
//! ## Properties NOT Proven Here (Out of Scope)
//!
//! - **Mutex unlock correctness**: The internal state machine of
//!   MutexGuard::drop() is verified in the mutex module.
//! - **ProcessManager correctness**: take_mutex_guard internals are verified
//!   in the PM module, including which error paths trigger implicit guard drops.
//! - **Ownership validation**: Whether the calling thread actually owns the
//!   mutex is a PM concern.
//! - **MutexAddress validation**: Address validity is a PM concern.
//! - **Liveness / thread notification**: The original code comment notes "The
//!   mutex guard is dropped, causing threads to be notified." Thread wakeup
//!   semantics depend on the mutex module and scheduler, verified separately.
//!
//! ## Verification Model
//!
//! The original function uses:
//! - `MutexAddress::from(usize)` → modeled as opaque type construction.
//! - `ProcessManager::take_mutex_guard()` → modeled via
//!   `take_mutex_guard_model()` `external_body`. Returns a ghost guard token
//!   on success.
//! - `MutexGuard::drop()` → modeled via `drop_guard_model()` `external_body`.
//!   Consumes the ghost guard token and establishes the mutex-unlocked
//!   postcondition. This separates the "acquire guard" and "release guard"
//!   steps for composability.
//!
//! The exec-level `unlock_mutex_model()` mirrors the original control flow and
//! proves that the result matches `spec_unlock_mutex_result` for all inputs
//! and all PM outcomes.
//!
//! ## Trust Boundaries
//!
//! - **T1: `ProcessManager::take_mutex_guard()`**. Returns the mutex guard for
//!   the given (pid, tid, mutex_addr). The PM module verifies this function's
//!   correctness internally. Modeled as `external_body`. Returns a ghost guard
//!   token `Option<u32>` on success (`Some(mutex_addr)`), `None` on failure.
//!   On error, the mutex may have been implicitly unlocked by PM-internal
//!   guard drops (modeled via `spec_mutex_may_be_unlocked_on_error`).
//! - **T2: `MutexGuard::drop()`**. Unlocks the mutex when the guard goes out
//!   of scope. Modeled as `drop_guard_model()` `external_body`. Consumes the
//!   guard token and establishes `spec_guard_dropped_and_mutex_unlocked`.
//!
//! ## API Mapping
//!
//! | Original API                                  | Verified Model                      | Notes          |
//! |-----------------------------------------------|-------------------------------------|----------------|
//! | `MutexAddress::from(usize)`                   | (not modeled)                       | Type wrapper.  |
//! | `ProcessManager::take_mutex_guard(pid,tid,a)`  | `take_mutex_guard_model(pid,tid,a)` | external_body. |
//! | `MutexGuard::drop()`                          | `drop_guard_model(addr, token)`     | external_body. |
//! | `pub unsafe fn unlock_mutex(pid,tid,addr)`    | `unlock_mutex_model(addr,pid,tid)`  | Fully verified.|

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
/// On success, a `MutexGuard` is returned (modeled via a ghost guard token).
/// On failure, an `Error` is returned.
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
/// specified (pid, tid). `take_mutex_guard` returns `Result<MutexGuard, Error>`.
/// On success, the `MutexGuard` is returned by value. On failure, an `Error`
/// is returned. The PM module verifies this function's correctness internally.
///
/// In the original code, pid and tid determine ownership validation.
/// They are accepted as ghost parameters to allow future enrichment of the
/// PM trust boundary with ownership constraints without restructuring.
///
/// The guard token (`Option<u32>`) models the `MutexGuard` ownership:
/// `Some(mutex_addr)` on success, `None` on failure. This token is consumed
/// by `drop_guard_model`, separating the "acquire" and "release" semantics.
///
/// ## PM-Internal Error Path with Implicit Guard Drop
///
/// The PM's `take_mutex_guard` implementation has a two-step internal flow:
/// 1. Extract `MutexGuard` from thread bookkeeping.
/// 2. Call `put_mutex(mutex_addr)` to return the mutex slot.
/// If step 2 fails, the function returns `Err`, but the `MutexGuard` was
/// already extracted in step 1. Rust drops it at scope exit, triggering
/// `MutexGuard::drop()` which unlocks the mutex. On this error path, the
/// mutex IS unlocked as a side effect even though the caller sees `Err`.
///
/// The model captures this via `spec_mutex_may_be_unlocked_on_error`:
/// on error, the mutex may or may not have been unlocked depending on
/// which internal error path was taken. The concrete behavior is a PM
/// concern — the PM module should prove which error paths trigger
/// implicit unlocks.
///
/// For the kcall caller, the guard token is `None` on error because the
/// caller did NOT receive a `MutexGuard` (the `Err` variant carries only
/// the `Error`). The PM-internal implicit drop is invisible to the caller.
///
/// # Parameters
///
/// - `mutex_addr`: The mutex address (from `MutexAddress::from(usize)`).
/// - `pid`: Ghost process identifier for the calling process.
/// - `tid`: Ghost thread identifier for the calling thread.
#[verifier::external_body]
pub fn take_mutex_guard_model(mutex_addr: u32, pid: Ghost<u32>, tid: Ghost<u32>) -> (result: (TakeMutexGuardOutcomeModel, Ghost<Option<u32>>))
    requires
        // Safety: the caller must not hold a PM reference.
        spec_unlock_mutex_safety_preconditions(),
    ensures
        // Error codes from the PM module are valid ErrorCode discriminants.
        result.0 matches TakeMutexGuardOutcomeModel::Error { error_code }
            ==> spec_is_valid_error_code(error_code as int),
        // On success, a guard token is returned for the correct mutex.
        (result.0 matches TakeMutexGuardOutcomeModel::Ok) ==> result.1@.is_some(),
        (result.0 matches TakeMutexGuardOutcomeModel::Ok) ==> result.1@ == Some(mutex_addr),
        // On error, no guard token is returned to the caller (the caller
        // receives Err, not Ok(MutexGuard)). Any PM-internal implicit guard
        // drop is modeled via spec_mutex_may_be_unlocked_on_error.
        !(result.0 matches TakeMutexGuardOutcomeModel::Ok) ==> result.1@.is_none(),
        // On error, the mutex may have been implicitly unlocked by the PM's
        // internal guard drop (if the guard was extracted before the error).
        result.0 matches TakeMutexGuardOutcomeModel::Error { .. }
            ==> spec_mutex_may_be_unlocked_on_error(mutex_addr as nat),
        // Ownership: success implies the thread owned the mutex guard.
        // Concrete interpretation provided by the PM module.
        result.0 matches TakeMutexGuardOutcomeModel::Ok
            ==> spec_thread_owns_mutex(pid@ as nat, tid@ as nat, mutex_addr as nat),
{
    unimplemented!()
}

/// Trust Boundary T2: Models `MutexGuard::drop()`.
///
/// # Description
///
/// Consumes the guard token and unlocks the mutex. In the original code,
/// `MutexGuard::drop()` is called implicitly when the guard goes out of
/// scope (at the semicolon after the `?` operator extracts it from `Ok`).
///
/// This is modeled as a separate step to keep the "acquire guard" and
/// "release guard" semantics composable. If `take_mutex_guard` were reused
/// in a context where the guard is NOT immediately dropped, this separation
/// would be essential.
///
/// # Parameters
///
/// - `mutex_addr`: The mutex address being unlocked.
/// - `guard_token`: Ghost token proving the caller holds a valid guard.
#[verifier::external_body]
pub fn drop_guard_model(mutex_addr: u32, guard_token: Ghost<Option<u32>>)
    requires
        // The caller must hold a valid guard token for this specific mutex.
        guard_token@ == Some(mutex_addr),
    ensures
        // After drop, the guard is consumed and the mutex is unlocked.
        spec_guard_dropped_and_mutex_unlocked(mutex_addr as nat),
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
/// 1. Calls take_mutex_guard (external) → returns guard token on success.
/// 2. On success, drops the guard (external) → mutex unlocked.
/// 3. Returns Ok(()) on success (guard was dropped).
/// 4. Returns the error on failure.
///
/// # Parameters
///
/// - `mutex_addr`: Mutex address (from `MutexAddress::from(usize)`).
/// - `pid`: Ghost process identifier for the calling process.
/// - `tid`: Ghost thread identifier for the calling thread.
///
/// # Returns
///
/// A tuple of (result, ghost take_guard_outcome) where the ghost captures
/// the PM outcome for postcondition linking.
pub fn unlock_mutex_model(mutex_addr: u32, pid: Ghost<u32>, tid: Ghost<u32>) -> (ret: (
    UnlockMutexResultModel,
    Ghost<TakeMutexGuardOutcomeView>,
))
    requires
        // Safety: the caller must not hold a PM reference.
        spec_unlock_mutex_safety_preconditions(),
        // ABI constraint: mutex_addr originates from 32-bit usize on x86-32.
        // This is always true for u32 values (documentation-only constraint
        // making the architecture assumption explicit).
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
        // On success, the calling thread owned the mutex (from PM contract).
        spec_is_success(ret.0.spec_view()) ==>
            spec_thread_owns_mutex(pid@ as nat, tid@ as nat, mutex_addr as nat),
        // On error, the mutex may have been implicitly unlocked by PM internals.
        spec_is_error(ret.0.spec_view()) ==>
            spec_mutex_may_be_unlocked_on_error(mutex_addr as nat),
{
    // Step 1: Take mutex guard (external), threading ghost pid/tid.
    let tg_pair: (TakeMutexGuardOutcomeModel, Ghost<Option<u32>>) = take_mutex_guard_model(mutex_addr, pid, tid);
    let tg_result: TakeMutexGuardOutcomeModel = tg_pair.0;
    let guard_token: Ghost<Option<u32>> = tg_pair.1;
    let ghost tg_view: TakeMutexGuardOutcomeView = tg_result.spec_view();

    match tg_result {
        TakeMutexGuardOutcomeModel::Error { error_code } => {
            // No guard was returned; nothing to drop.
            (
                UnlockMutexResultModel::TakeMutexGuardError { error_code },
                Ghost(tg_view),
            )
        },
        TakeMutexGuardOutcomeModel::Ok => {
            // Step 2: Drop the guard (models MutexGuard going out of scope).
            drop_guard_model(mutex_addr, guard_token);

            (
                UnlockMutexResultModel::Success,
                Ghost(tg_view),
            )
        },
    }
}

} // verus!
