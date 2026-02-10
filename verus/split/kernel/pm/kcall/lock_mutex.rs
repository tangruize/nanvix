// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Lock Mutex Kernel Call Verification Model
//!
//! Formal verification of the lock_mutex kernel call (`pm::kcall::lock_mutex`).
//!
//! ## Overview
//!
//! The `lock_mutex(pid, tid, mutex_addr, timeout_s, timeout_ns)` function locks
//! a mutex with an optional timeout. It:
//! 1. Converts `mutex_addr` to a `MutexAddress`.
//! 2. Parses `(timeout_s, timeout_ns)` into `Option<SystemTime>`:
//!    - Both `usize::MAX` → `None` (infinite wait).
//!    - `SystemTime::new(s, ns)` returns `Some` → finite timeout.
//!    - `SystemTime::new(s, ns)` returns `None` → `InvalidArgument` error.
//! 3. Calls `ProcessManager::get_mutex(mutex_addr)` to obtain the mutex.
//! 4. Calls `mutex.lock(timeout)` to acquire the lock.
//! 5. Calls `ProcessManager::put_mutex_guard(mutex_addr, guard)` to store the guard.
//!
//! ## Note on Omitted Parameters
//!
//! The original `lock_mutex()` takes `pid: ProcessIdentifier` and
//! `tid: ThreadIdentifier` parameters. These are used only in the `trace!()`
//! diagnostic macro and do not affect control flow or correctness. They are
//! intentionally omitted from the verification model. If a future refactoring
//! uses `pid`/`tid` in the logic (e.g., for ownership checks), the model must
//! be updated to include them.
//!
//! ## Verified Properties
//!
//! - **Timeout parsing correctness**: MAX/MAX → infinite; valid ns → finite;
//!   invalid ns → InvalidArgument error (`lemma_max_max_is_infinite`,
//!   `lemma_invalid_nanos_returns_none`).
//! - **Invalid timeout produces error**: When SystemTime::new fails, the function
//!   returns GenericError(InvalidArgument) regardless of PM outcomes
//!   (`lemma_invalid_timeout_returns_error`).
//! - **Pipeline short-circuit**: Each step's failure makes later steps irrelevant
//!   (`lemma_pipeline_short_circuit`, `lemma_get_mutex_short_circuit`,
//!   `lemma_lock_short_circuit`).
//! - **Error propagation**: get_mutex and put_guard errors wrap in SleepError::Generic;
//!   lock errors pass through unchanged (`lemma_get_mutex_error_propagates`,
//!   `lemma_lock_error_propagates`, `lemma_put_guard_error_propagates`).
//! - **Success requires all steps**: Success iff timeout is valid AND get_mutex
//!   succeeds AND lock succeeds AND put_guard succeeds
//!   (`lemma_success_requires_all_steps`).
//! - **Result exhaustiveness**: Every input produces exactly one result category
//!   (`lemma_result_exhaustive`).
//! - **Finite timeout well-formedness**: Parsed finite timeout has nanoseconds
//!   < 1_000_000_000 (`lemma_finite_timeout_wf`).
//! - **Error code linkage**: `ERROR_CODE_INVALID_ARGUMENT()` == 22
//!   (`lemma_error_code_matches`).
//! - **Exec model correctness**: `lock_mutex_model` matches `spec_lock_mutex_result`
//!   for all inputs and PM outcomes.
//! - **TimedOut requires finite timeout**: With an infinite timeout, TimedOut
//!   is impossible — enforced via `mutex_lock_model` contract and propagated
//!   to the final result (`lemma_infinite_timeout_no_timed_out`).
//! - **Timeout value threading**: The exact parsed timeout value (as
//!   `Option<TimeoutView>`) is threaded through to `mutex_lock_model` via
//!   ghost state, proving that the lock step receives the correct timeout
//!   value derived from the raw (timeout_s, timeout_ns) inputs
//!   (`lemma_timeout_value_reaches_lock`).
//! - **pid/tid independence**: The pipeline result is formally independent of
//!   the `pid` and `tid` parameters, proven via `spec_lock_mutex_result_with_context`
//!   which takes pid/tid in its signature but delegates to the core spec
//!   (`lemma_result_independent_of_pid_tid`). Guards against future signature drift.
//! - **Guard ownership chain**: The lock step produces a ghost guard token on
//!   success, which the put_guard step requires as a precondition. This
//!   formalizes the ownership chain: `Mutex::lock` produces `MutexGuard` →
//!   `ProcessManager::put_mutex_guard` consumes it.
//! - **Timeout view consistency**: The ghost `timeout_view` parameter passed to
//!   `mutex_lock_model` is constrained by `spec_timeout_view_consistent` to be
//!   consistent with the `has_timeout` boolean flag.
//! - **Architecture guard**: The x86-32 assumption (`USIZE_BITS() == 32`,
//!   `USIZE_MAX_X86_32() == u32::MAX`) is explicitly verified
//!   (`lemma_architecture_guard`). If the target changes to x86-64, this
//!   lemma will fail, forcing model updates.
//! - **Finite timeout construction**: Valid non-MAX inputs with valid nanoseconds
//!   produce a `Finite` timeout (`lemma_valid_non_max_is_finite`).
//! - **Safety predicate well-formedness**: The composite safety predicate
//!   correctly decomposes into its three constituents
//!   (`lemma_safety_preconditions_well_formed`).
//! - **Guard consumption on all paths**: After `mutex_lock_model`, the guard
//!   token is either `Some(addr)` (consumed by `put_mutex_guard_model`) or
//!   `None` (no guard exists). No path leaks a guard token
//!   (`lemma_guard_consumed_on_all_paths`). On the `PutGuardError` path,
//!   the original `MutexGuard` is consumed by move semantics inside
//!   `put_mutex_guard`, and `MutexGuard::drop()` unlocks the mutex.
//! - **Guard release on error path**: `put_mutex_guard_model` ensures
//!   `spec_guard_ownership_released(mutex_addr)` on ALL exit paths (success
//!   and error), guaranteeing no lock leak. This postcondition is threaded
//!   through `lock_mutex_model` (`lemma_put_guard_error_releases_guard`).
//!
//! ## Properties NOT Proven Here (Out of Scope)
//!
//! - **Mutex lock correctness**: The internal state machine of Mutex::lock is
//!   verified in the mutex module. This kcall only proves the pipeline logic.
//! - **ProcessManager correctness**: get_mutex and put_mutex_guard internals are
//!   verified in the PM module.
//! - **Concurrency**: Thread scheduling, blocking, and waking are OS-level
//!   concerns verified elsewhere.
//! - **MutexAddress validation**: The conversion from usize to MutexAddress is
//!   a type wrapper; address validity is a PM concern.
//! - **Unsafe safety contract**: The original function's safety documentation
//!   requires: (1) the caller is not the kernel process, (2) the caller does
//!   not hold any resources (e.g., other mutex guards, memory locks), and
//!   (3) the caller does not hold a reference to the ProcessManager. These
//!   constraints are about the *caller's global state* (scheduler context,
//!   resource ownership) rather than about this function's own pipeline logic.
//!   They are modeled as abstract (uninterpreted) spec predicates:
//!   `spec_caller_is_not_kernel_process`, `spec_caller_holds_no_resources`,
//!   and `spec_caller_no_pm_reference`, combined in
//!   `spec_lock_mutex_safety_preconditions`. These predicates provide formal
//!   hooks for the PM module's call-site verification — the PM module should
//!   provide concrete interpretations and prove they hold before invoking
//!   `lock_mutex`. They are not added as `requires` on `lock_mutex_model`
//!   because the concrete definitions depend on ProcessManager state that is
//!   out of scope for this pipeline verification.
//!
//! ## Verification Model
//!
//! The original function uses several external dependencies:
//! - `MutexAddress::from(usize)` → modeled as opaque type construction.
//! - `SystemTime::new(u64, u32)` → modeled via `system_time_new()` with the
//!   well-known invariant: returns None iff nanoseconds >= 1_000_000_000.
//! - `ProcessManager::get_mutex()` → modeled via `get_mutex_model()` `external_body`.
//! - `Mutex::lock()` → modeled via `mutex_lock_model()` `external_body`.
//! - `ProcessManager::put_mutex_guard()` → modeled via `put_mutex_guard_model()`
//!   `external_body`.
//!
//! The exec-level `lock_mutex_model()` mirrors the original control flow and proves
//! that the result matches `spec_lock_mutex_result` for all inputs and all PM outcomes.
//!
//! ## Trust Boundaries
//!
//! - **T1: `SystemTime::new()`**. Returns None iff nanoseconds >= 1_000_000_000.
//!   Modeled as a verified function with this exact behavior.
//! - **T2: `ProcessManager::get_mutex()`**. Returns the mutex for the given address.
//!   Modeled as `external_body`; PM correctness is verified separately.
//! - **T3: `Mutex::lock()`**. Acquires the lock with optional timeout. Returns
//!   `Result<MutexGuard, SleepError>`. Modeled as `external_body`; mutex
//!   correctness is verified in the mutex module.
//! - **T4: `ProcessManager::put_mutex_guard()`**. Stores the guard in the calling
//!   thread. Modeled as `external_body`; PM correctness is verified separately.
//! - **T5: `usize` to `u64`/`u32` cast**. On x86-32, usize is 32 bits. The
//!   `timeout_s as u64` is a widening cast (safe). The `timeout_ns as u32` is
//!   identity (safe). Modeled via `USIZE_MAX_X86_32()` precondition.
//!   **Architecture note**: The model uses `u32` for `mutex_addr`, `timeout_s`,
//!   and `timeout_ns` (which are `usize` in the original). This is correct for
//!   x86-32 — the only currently supported target. If the kernel targets x86-64
//!   in the future, `USIZE_MAX_X86_32()` must be updated to `USIZE_MAX_X86_64()`
//!   and the parameter types widened to `u64`.
//!
//! ## Error Reason Abstraction
//!
//! The original code constructs `Error::new(ErrorCode::InvalidArgument, "invalid timeout")`
//! on invalid timeout. This model abstracts away the reason string, retaining only
//! the numeric error code (22). The reason string is diagnostic and does not affect
//! control flow or semantic correctness.
//!
//! ## API Mapping
//!
//! | Original API                              | Verified Model                    | Notes               |
//! |-------------------------------------------|-----------------------------------|----------------------|
//! | `MutexAddress::from(usize)`               | (not modeled)                     | Type wrapper.        |
//! | `SystemTime::new(u64, u32)`               | `system_time_new(u64, u32)`       | Verified.            |
//! | `ProcessManager::get_mutex(addr)`         | `get_mutex_model(addr)`           | external_body.       |
//! | `Mutex::lock(timeout)`                    | `mutex_lock_model(has, tv, addr)` | external_body.       |
//! | `ProcessManager::put_mutex_guard(a, g)`   | `put_mutex_guard_model(a, tok)`   | external_body.       |
//! | `pub unsafe fn lock_mutex(...)`           | `lock_mutex_model(...)`           | Fully verified.      |

use crate::libs::error::ErrorCode;
use vstd::prelude::*;

// Architecture documentation: This model assumes usize is 32 bits (x86-32).
// The Verus verifier runs on the host (typically x86-64), so compile-time
// `#[cfg(target_pointer_width)]` checks would be misleading. Instead, the
// x86-32 assumption is enforced via:
// 1. `USIZE_BITS()` spec constant (= 32) in the spec file.
// 2. `lemma_architecture_guard()` in the proof file, which verifies
//    USIZE_MAX_X86_32 == u32::MAX == 4294967295.
// 3. All model parameters use `u32` (not `usize`), matching x86-32 width.
// If the kernel targets x86-64, update USIZE_BITS to 64, widen parameters
// to u64, and update USIZE_MAX to u64::MAX. The lemma will fail until fixed.

// Include specifications.
include!("lock_mutex.spec.rs");

// Include proofs.
include!("lock_mutex.proof.rs");

verus! {

//==================================================================================================
// Dependency Models (External Bodies)
//==================================================================================================

/// Model of a SystemTime value for verification.
///
/// # Description
///
/// Represents a SystemTime with accessible seconds and nanoseconds fields.
/// Reused from the sleep kcall model for consistency.
pub struct SystemTimeModel {
    /// Seconds since epoch.
    pub seconds: u64,
    /// Nanoseconds since last second.
    pub nanoseconds: u32,
}

/// Model of the GetMutex step outcome.
pub enum GetMutexOutcomeModel {
    /// get_mutex succeeded.
    Ok,
    /// get_mutex failed with an error code.
    Error { error_code: i32 },
}

impl GetMutexOutcomeModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> GetMutexOutcomeView {
        match self {
            GetMutexOutcomeModel::Ok => GetMutexOutcomeView::GmOk,
            GetMutexOutcomeModel::Error { error_code } => {
                GetMutexOutcomeView::GmError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the Mutex::lock step outcome.
pub enum LockOutcomeModel {
    /// lock succeeded.
    Ok,
    /// lock returned Interrupted(TimedOut).
    TimedOut,
    /// lock returned Interrupted(Killed).
    Killed,
    /// lock returned Generic(error).
    GenericError { error_code: i32 },
}

impl LockOutcomeModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> LockOutcomeView {
        match self {
            LockOutcomeModel::Ok => LockOutcomeView::LoOk,
            LockOutcomeModel::TimedOut => LockOutcomeView::LoTimedOut,
            LockOutcomeModel::Killed => LockOutcomeView::LoKilled,
            LockOutcomeModel::GenericError { error_code } => {
                LockOutcomeView::LoGenericError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the PutMutexGuard step outcome.
pub enum PutGuardOutcomeModel {
    /// put_mutex_guard succeeded.
    Ok,
    /// put_mutex_guard failed with an error code.
    Error { error_code: i32 },
}

impl PutGuardOutcomeModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> PutGuardOutcomeView {
        match self {
            PutGuardOutcomeModel::Ok => PutGuardOutcomeView::PgOk,
            PutGuardOutcomeModel::Error { error_code } => {
                PutGuardOutcomeView::PgError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the lock_mutex overall result.
///
/// # Description
///
/// Represents the final result of the lock_mutex kcall, mirroring
/// Result<(), SleepError> from the original.
pub enum LockMutexResultModel {
    /// All steps succeeded.
    Success,
    /// Timeout parsing failed: SystemTime::new returned None.
    InvalidTimeoutError { error_code: i32 },
    /// ProcessManager::get_mutex failed.
    GetMutexError { error_code: i32 },
    /// Mutex::lock returned Interrupted(TimedOut).
    LockTimedOut,
    /// Mutex::lock returned Interrupted(Killed).
    LockKilled,
    /// Mutex::lock returned Generic(error).
    LockGenericError { error_code: i32 },
    /// ProcessManager::put_mutex_guard failed.
    PutGuardError { error_code: i32 },
}

impl LockMutexResultModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> LockMutexResultView {
        match self {
            LockMutexResultModel::Success => LockMutexResultView::Success,
            LockMutexResultModel::InvalidTimeoutError { error_code } => {
                LockMutexResultView::InvalidTimeoutError { error_code: *error_code as int }
            },
            LockMutexResultModel::GetMutexError { error_code } => {
                LockMutexResultView::GetMutexError { error_code: *error_code as int }
            },
            LockMutexResultModel::LockTimedOut => LockMutexResultView::LockTimedOut,
            LockMutexResultModel::LockKilled => LockMutexResultView::LockKilled,
            LockMutexResultModel::LockGenericError { error_code } => {
                LockMutexResultView::LockGenericError { error_code: *error_code as int }
            },
            LockMutexResultModel::PutGuardError { error_code } => {
                LockMutexResultView::PutGuardError { error_code: *error_code as int }
            },
        }
    }
}

//==================================================================================================
// External Body Functions (Trust Boundaries)
//==================================================================================================

/// Trust Boundary T2: Models `ProcessManager::get_mutex(mutex_addr)`.
///
/// # Description
///
/// Returns the mutex associated with the given address.
/// The PM module verifies this function's correctness internally.
///
/// # Parameters
///
/// - `mutex_addr`: The mutex address (from `MutexAddress::from(usize)`).
///   Address validity is a PM concern; this model preserves the interface
///   shape for future strengthening (e.g., `result is Ok ==> addr_is_valid(addr)`).
#[verifier::external_body]
pub fn get_mutex_model(mutex_addr: u32) -> (result: GetMutexOutcomeModel)
    ensures
        // Error codes from the PM module are valid ErrorCode discriminants.
        result matches GetMutexOutcomeModel::Error { error_code } ==> spec_is_valid_error_code(error_code as int),
        // Future strengthening hook (PM module responsibility):
        // result matches GetMutexOutcomeModel::Ok ==> spec_addr_is_valid(mutex_addr),
{
    unimplemented!()
}

/// Trust Boundary T3: Models `Mutex::lock(timeout)`.
///
/// # Description
///
/// Acquires the mutex with the given timeout. Returns one of the four possible
/// outcomes matching SleepError variants.
///
/// The postcondition encodes: `Interrupted(TimedOut)` can only occur when a
/// finite timeout is provided. With an infinite timeout (`None`), the
/// `Condvar::wait()` path has no timer, so TimedOut is impossible.
///
/// # Parameters
///
/// - `has_timeout`: Whether a finite timeout was provided (true = `Some(t)`,
///   false = `None` in the original).
/// - `timeout_view`: Ghost of the parsed timeout value. When `has_timeout` is
///   true, this carries the exact `TimeoutView::Finite { seconds, nanoseconds }`
///   that was derived from the raw `(timeout_s, timeout_ns)` inputs. This
///   parameter is accepted to prove value-level correctness: the exact parsed
///   timeout reaches the lock step. The actual lock behavior (blocking duration,
///   etc.) is verified in the mutex module.
/// - `mutex_addr`: Ghost of the mutex address, threaded from `get_mutex_model`.
///   Used to associate the guard token with a specific mutex, formalizing the
///   ownership chain: get_mutex(addr) → lock produces guard(addr) → put_guard
///   requires guard(addr).
#[verifier::external_body]
pub fn mutex_lock_model(has_timeout: bool, timeout_view: Ghost<Option<TimeoutView>>, mutex_addr: Ghost<u32>) -> (result: (LockOutcomeModel, Ghost<Option<u32>>))
    requires
        // The ghost timeout_view must be consistent with the has_timeout flag.
        spec_timeout_view_consistent(has_timeout, timeout_view@),
    ensures
        // TimedOut can only occur with a finite timeout. With an infinite
        // timeout, the Condvar::wait() path has no timer.
        spec_lock_outcome_valid_for_timeout(has_timeout, result.0.spec_view()),
        // Error codes from lock failures are valid ErrorCode discriminants.
        result.0 matches LockOutcomeModel::GenericError { error_code } ==> spec_is_valid_error_code(error_code as int),
        // Guard token: Some(addr) iff lock succeeded; None otherwise.
        // The contained address identifies which mutex the guard belongs to.
        (result.0 matches LockOutcomeModel::Ok) <==> result.1@.is_some(),
        // Guard token carries the correct mutex address on success.
        result.1@.is_some() ==> result.1@ == Some(mutex_addr@),
{
    unimplemented!()
}

/// Trust Boundary T4: Models `ProcessManager::put_mutex_guard(mutex_addr, guard)`.
///
/// # Description
///
/// Stores the mutex guard in the calling thread.
/// The PM module verifies this function's correctness internally.
///
/// **Guard consumption semantics**: In the original code, `put_mutex_guard`
/// takes the `MutexGuard` by value (move semantics). On success, the guard
/// is stored in the thread's bookkeeping. On failure, the guard is dropped
/// at the end of `put_mutex_guard`, and `MutexGuard::drop()` unlocks the
/// mutex. Either way, the guard is consumed — no guard leak is possible.
/// The unlock-on-drop behavior is a mutex-module concern, modeled here as
/// a commented-out postcondition hook for future PM-level strengthening.
///
/// # Parameters
///
/// - `mutex_addr`: The mutex address, same as passed to `get_mutex_model`.
///   Preserved for interface fidelity and future strengthening.
/// - `guard_token`: Ghost token proving the caller holds a valid guard for
///   the specific mutex identified by `mutex_addr`. Models the `MutexGuard`
///   parameter from the original `ProcessManager::put_mutex_guard(mutex_addr,
///   guard)`, formalizing the ownership chain: lock produces a guard for a
///   specific mutex → put_guard consumes a guard for the same mutex.
#[verifier::external_body]
pub fn put_mutex_guard_model(mutex_addr: u32, guard_token: Ghost<Option<u32>>) -> (result: PutGuardOutcomeModel)
    requires
        // The caller must hold a valid guard token for this specific mutex.
        guard_token@ == Some(mutex_addr),
    ensures
        // Error codes from the PM module are valid ErrorCode discriminants.
        result matches PutGuardOutcomeModel::Error { error_code } ==> spec_is_valid_error_code(error_code as int),
        // Guard ownership is always released, whether put_mutex_guard succeeds
        // or fails. On success the guard is stored; on failure MutexGuard::drop()
        // unlocks the mutex. The concrete interpretation of this predicate is
        // provided by the mutex/PM module.
        spec_guard_ownership_released(mutex_addr as nat),
{
    unimplemented!()
}

//==================================================================================================
// Verified Functions
//==================================================================================================

/// Trust Boundary T1: Models `SystemTime::new(seconds, nanoseconds)`.
///
/// # Description
///
/// Creates a SystemTime if nanoseconds < 1_000_000_000, returns None otherwise.
/// This is a verified function (not external_body) because the logic is simple
/// and well-specified.
///
/// # Parameters
///
/// - `seconds`: Seconds component (from usize cast to u64).
/// - `nanoseconds`: Nanoseconds component (from usize cast to u32).
///
/// # Returns
///
/// Some(SystemTimeModel) if valid, None if nanoseconds >= 1_000_000_000.
pub fn system_time_new(seconds: u64, nanoseconds: u32) -> (result: Option<SystemTimeModel>)
    ensures
        nanoseconds < 1_000_000_000u32 ==> result.is_some(),
        nanoseconds < 1_000_000_000u32 ==> result.unwrap().seconds == seconds,
        nanoseconds < 1_000_000_000u32 ==> result.unwrap().nanoseconds == nanoseconds,
        nanoseconds >= 1_000_000_000u32 ==> result.is_none(),
{
    if nanoseconds >= 1_000_000_000u32 {
        None
    } else {
        Some(SystemTimeModel { seconds, nanoseconds })
    }
}

/// Verified model of timeout parsing.
///
/// # Description
///
/// Implements the original's timeout parsing logic:
/// ```ignore
/// let timeout = if timeout_s == usize::MAX && timeout_ns == usize::MAX {
///     None
/// } else {
///     match SystemTime::new(timeout_s as u64, timeout_ns as u32) {
///         Some(t) => Some(t),
///         None => return Err(SleepError::Generic(Error::new(InvalidArgument, ...)))
///     }
/// };
/// ```
///
/// Returns Ok(has_timeout) where has_timeout indicates whether a finite timeout
/// was parsed, or Err with the error model for invalid timeout.
///
/// # Parameters
///
/// - `timeout_s`: Timeout seconds (from usize, 32-bit on x86).
/// - `timeout_ns`: Timeout nanoseconds (from usize, 32-bit on x86).
///
/// # Returns
///
/// Ok(bool): true if finite timeout, false if infinite.
/// Err(LockMutexResultModel): InvalidTimeoutError on failure.
pub fn parse_timeout(timeout_s: u32, timeout_ns: u32) -> (result: Result<bool, LockMutexResultModel>)
    ensures
        // Infinite timeout case.
        spec_is_infinite_timeout(timeout_s as nat, timeout_ns as nat)
            ==> result == Ok::<bool, LockMutexResultModel>(false),
        // Valid finite timeout case.
        (!spec_is_infinite_timeout(timeout_s as nat, timeout_ns as nat)
            && spec_timeout_ns_valid(timeout_ns as nat))
            ==> result == Ok::<bool, LockMutexResultModel>(true),
        // Invalid timeout case.
        (!spec_is_infinite_timeout(timeout_s as nat, timeout_ns as nat)
            && !spec_timeout_ns_valid(timeout_ns as nat))
            ==> result.is_err(),
        // Error case produces correct error code.
        result matches Err(err) ==> err.spec_view() == (LockMutexResultView::InvalidTimeoutError {
            error_code: ERROR_CODE_INVALID_ARGUMENT(),
        }),
        // Result matches spec_parse_timeout.
        result.is_ok() <==> spec_timeout_parsed_ok(timeout_s as nat, timeout_ns as nat),
{
    if timeout_s == u32::MAX && timeout_ns == u32::MAX {
        // Both MAX → infinite timeout (None in original).
        Ok(false)
    } else {
        // Try to construct SystemTime.
        let st: Option<SystemTimeModel> = system_time_new(timeout_s as u64, timeout_ns);
        match st {
            Some(_) => Ok(true),
            None => {
                proof {
                    assert(22i32 as int == ERROR_CODE_INVALID_ARGUMENT());
                }
                Err(LockMutexResultModel::InvalidTimeoutError { error_code: 22i32 })
            },
        }
    }
}

/// Verified exec model of the `lock_mutex` kernel call.
///
/// # Description
///
/// This function mirrors the original `pub unsafe fn lock_mutex(...)` control
/// flow. It:
/// 1. Parses timeout from (timeout_s, timeout_ns).
/// 2. Calls get_mutex (external).
/// 3. Calls lock (external).
/// 4. Calls put_mutex_guard (external).
///
/// The postconditions prove that the result matches `spec_lock_mutex_result`
/// for all inputs and all PM outcomes.
///
/// The original also takes `pid` and `tid` parameters, which are used only
/// in the `trace!()` diagnostic macro and are omitted from this model.
///
/// # Parameters
///
/// - `mutex_addr`: Mutex address (from `MutexAddress::from(usize)`).
/// - `timeout_s`: Timeout seconds (from usize, 32-bit on x86).
/// - `timeout_ns`: Timeout nanoseconds (from usize, 32-bit on x86).
///
/// # Returns
///
/// A tuple of (result, ghost get_mutex_outcome, ghost lock_outcome, ghost put_guard_outcome)
/// where the ghosts capture the PM outcomes for postcondition linking.
pub fn lock_mutex_model(mutex_addr: u32, timeout_s: u32, timeout_ns: u32) -> (ret: (
    LockMutexResultModel,
    Ghost<GetMutexOutcomeView>,
    Ghost<LockOutcomeView>,
    Ghost<PutGuardOutcomeView>,
    Ghost<Option<TimeoutView>>,
))
    requires
        // ABI constraint: inputs originate from 32-bit usize on x86-32.
        timeout_s as nat <= USIZE_MAX_X86_32(),
        timeout_ns as nat <= USIZE_MAX_X86_32(),
    ensures
        // The result matches the spec for the captured PM outcomes.
        ret.0.spec_view() == spec_lock_mutex_result(
            timeout_s as nat,
            timeout_ns as nat,
            ret.1@,
            ret.2@,
            ret.3@,
        ),
        // Invalid timeout → always InvalidTimeoutError.
        !spec_timeout_parsed_ok(timeout_s as nat, timeout_ns as nat) ==>
            spec_is_timeout_error(ret.0.spec_view()),
        // Success only when all steps succeed.
        spec_is_success(ret.0.spec_view()) ==> (
            spec_timeout_parsed_ok(timeout_s as nat, timeout_ns as nat)
            && ret.1@ == GetMutexOutcomeView::GmOk
            && ret.2@ == LockOutcomeView::LoOk
            && ret.3@ == PutGuardOutcomeView::PgOk
        ),
        // TimedOut impossible with infinite timeout (from mutex_lock_model contract).
        !spec_is_finite_timeout(timeout_s as nat, timeout_ns as nat) ==>
            !matches!(ret.0, LockMutexResultModel::LockTimedOut),
        // Timeout value correctness: the parsed timeout is threaded to the lock step.
        // When the lock step is reached (timeout parsed OK, get_mutex OK), the
        // timeout value passed to mutex_lock_model matches spec_parsed_timeout_for_lock.
        ret.4@ == spec_parsed_timeout_for_lock(timeout_s as nat, timeout_ns as nat),
        // Guard ownership release: when the pipeline reaches the put_guard step
        // (i.e., timeout valid, get_mutex OK, lock OK), the guard ownership for
        // mutex_addr is released regardless of whether put_guard succeeds or fails.
        // This follows from put_mutex_guard_model's postcondition.
        (ret.2@ == LockOutcomeView::LoOk && ret.1@ == GetMutexOutcomeView::GmOk
            && spec_timeout_parsed_ok(timeout_s as nat, timeout_ns as nat))
            ==> spec_guard_ownership_released(mutex_addr as nat),
{
    // Step 1: Parse timeout.
    let parsed: Result<bool, LockMutexResultModel> = parse_timeout(timeout_s, timeout_ns);

    match parsed {
        Err(err) => {
            // Invalid timeout → return error.
            // Ghost PM outcomes are don't-care values: the spec
            // `spec_lock_mutex_result` ignores them on the timeout-error
            // path (the first `match` arm returns `InvalidTimeoutError`
            // regardless of get_mutex/lock/put_guard outcomes). Any
            // ghost values satisfy the postcondition vacuously.
            (err, Ghost(GetMutexOutcomeView::GmOk), Ghost(LockOutcomeView::LoOk), Ghost(PutGuardOutcomeView::PgOk), Ghost(spec_parsed_timeout_for_lock(timeout_s as nat, timeout_ns as nat)))
        },
        Ok(has_timeout) => {
            // Compute ghost timeout value for the lock step.
            let ghost timeout_for_lock: Option<TimeoutView> = spec_parsed_timeout_for_lock(timeout_s as nat, timeout_ns as nat);

            // Step 2: Get mutex (external).
            let gm_result: GetMutexOutcomeModel = get_mutex_model(mutex_addr);
            let ghost gm_view: GetMutexOutcomeView = gm_result.spec_view();

            match gm_result {
                GetMutexOutcomeModel::Error { error_code } => {
                    // get_mutex failed → wrap in SleepError::Generic.
                    // Ghost lock/put_guard outcomes are don't-cares (pipeline short-circuited).
                    (
                        LockMutexResultModel::GetMutexError { error_code },
                        Ghost(gm_view),
                        Ghost(LockOutcomeView::LoOk),
                        Ghost(PutGuardOutcomeView::PgOk),
                        Ghost(timeout_for_lock),
                    )
                },
                GetMutexOutcomeModel::Ok => {
                    // Step 3: Lock mutex (external), threading the parsed timeout value.
                    let lock_pair: (LockOutcomeModel, Ghost<Option<u32>>) = mutex_lock_model(has_timeout, Ghost(timeout_for_lock), Ghost(mutex_addr));
                    let lock_result: LockOutcomeModel = lock_pair.0;
                    let guard_token: Ghost<Option<u32>> = lock_pair.1;
                    let ghost lo_view: LockOutcomeView = lock_result.spec_view();

                    match lock_result {
                        LockOutcomeModel::TimedOut => {
                            // Ghost put_guard outcome is don't-care (pipeline short-circuited).
                            (
                                LockMutexResultModel::LockTimedOut,
                                Ghost(gm_view),
                                Ghost(lo_view),
                                Ghost(PutGuardOutcomeView::PgOk),
                                Ghost(timeout_for_lock),
                            )
                        },
                        LockOutcomeModel::Killed => {
                            // Ghost put_guard outcome is don't-care (pipeline short-circuited).
                            (
                                LockMutexResultModel::LockKilled,
                                Ghost(gm_view),
                                Ghost(lo_view),
                                Ghost(PutGuardOutcomeView::PgOk),
                                Ghost(timeout_for_lock),
                            )
                        },
                        LockOutcomeModel::GenericError { error_code } => {
                            // Ghost put_guard outcome is don't-care (pipeline short-circuited).
                            (
                                LockMutexResultModel::LockGenericError { error_code },
                                Ghost(gm_view),
                                Ghost(lo_view),
                                Ghost(PutGuardOutcomeView::PgOk),
                                Ghost(timeout_for_lock),
                            )
                        },
                        LockOutcomeModel::Ok => {
                            // Step 4: Put mutex guard (external), passing the guard token.
                            let pg_result: PutGuardOutcomeModel = put_mutex_guard_model(mutex_addr, guard_token);
                            let ghost pg_view: PutGuardOutcomeView = pg_result.spec_view();

                            match pg_result {
                                PutGuardOutcomeModel::Error { error_code } => {
                                    (
                                        LockMutexResultModel::PutGuardError { error_code },
                                        Ghost(gm_view),
                                        Ghost(lo_view),
                                        Ghost(pg_view),
                                        Ghost(timeout_for_lock),
                                    )
                                },
                                PutGuardOutcomeModel::Ok => {
                                    (
                                        LockMutexResultModel::Success,
                                        Ghost(gm_view),
                                        Ghost(lo_view),
                                        Ghost(pg_view),
                                        Ghost(timeout_for_lock),
                                    )
                                },
                            }
                        },
                    }
                },
            }
        },
    }
}

} // verus!
