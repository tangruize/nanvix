// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Lock Mutex Kernel Call Specification.
// Defines View types, spec constants, and spec functions for the lock_mutex
// kernel call verification model.
//
// ## Verification Model
//
// The lock_mutex kcall converts user-provided (timeout_s, timeout_ns) into an
// optional SystemTime timeout, then executes a three-step pipeline:
//   1. ProcessManager::get_mutex(mutex_addr) → Mutex
//   2. Mutex::lock(timeout) → MutexGuard
//   3. ProcessManager::put_mutex_guard(mutex_addr, guard) → ()
//
// This spec models:
// - Timeout parsing as a pure function with three outcomes (Infinite, Finite, Invalid).
// - The three-step pipeline as a sequential composition with short-circuit on error.
// - Error wrapping: get_mutex and put_guard errors wrapped in SleepError::Generic;
//   lock errors propagated unchanged.

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
/// If Nanvix targets x86-64, this must be updated to 64, which will
/// cause `lemma_architecture_guard` to fail, forcing model updates.
pub open spec fn USIZE_BITS() -> nat {
    32
}

/// Nanoseconds per second boundary for SystemTime validity.
pub open spec fn NANOS_PER_SEC() -> nat {
    1_000_000_000
}

/// The ErrorCode value for InvalidArgument (repr(i32) = 22).
pub open spec fn ERROR_CODE_INVALID_ARGUMENT() -> int {
    22
}

/// Spec predicate: whether an error code is a valid `ErrorCode` discriminant.
///
/// # Description
///
/// The `ErrorCode` enum in the kernel has many variants (all POSIX errno values),
/// all of which are positive integers. Rather than enumerating every variant
/// (which would create a maintenance burden as the enum evolves), this predicate
/// captures the essential invariant: all error codes are positive (non-zero).
///
/// This is the correct abstraction level for external_body trust boundaries:
/// - It prevents confusion with success (error code 0).
/// - It excludes negative values (POSIX errnos are always positive).
/// - It does not over-constrain to a specific subset, which would break
///   semantic equivalence if new ErrorCode variants are added.
/// - The concrete variant identity is a concern of the PM and mutex modules,
///   not this pipeline verification.
pub open spec fn spec_is_valid_error_code(code: int) -> bool {
    code > 0
}

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of the parsed timeout.
///
/// # Description
///
/// Represents the three possible outcomes of parsing the (timeout_s, timeout_ns)
/// parameters:
/// - `Infinite`: both parameters are usize::MAX, meaning wait indefinitely.
/// - `Finite`: a valid SystemTime was constructed from the parameters.
/// - `Invalid`: SystemTime::new returned None (nanoseconds >= 1_000_000_000).
#[verifier::ext_equal]
pub enum TimeoutView {
    /// Both timeout_s and timeout_ns are usize::MAX → no timeout (infinite wait).
    Infinite,
    /// A valid SystemTime was constructed.
    Finite { seconds: nat, nanoseconds: nat },
}

/// Abstract view of the outcome of ProcessManager::get_mutex.
#[verifier::ext_equal]
pub enum GetMutexOutcomeView {
    /// get_mutex succeeded, returning a Mutex.
    GmOk,
    /// get_mutex failed with an error code.
    GmError { error_code: int },
}

/// Abstract view of the outcome of Mutex::lock.
///
/// # Description
///
/// Models the four possible outcomes from Mutex::lock(timeout), which returns
/// Result<MutexGuard, SleepError>:
/// - Ok(guard) → LoOk
/// - Err(Interrupted(TimedOut)) → LoTimedOut
/// - Err(Interrupted(Killed)) → LoKilled
/// - Err(Generic(error)) → LoGenericError
///
/// Unlike the sleep kcall, lock_mutex does NOT re-classify these results.
/// It propagates the SleepError unchanged via the `?` operator.
#[verifier::ext_equal]
pub enum LockOutcomeView {
    /// Mutex::lock succeeded, returning a MutexGuard.
    LoOk,
    /// Mutex::lock returned Err(Interrupted(TimedOut)).
    LoTimedOut,
    /// Mutex::lock returned Err(Interrupted(Killed)).
    LoKilled,
    /// Mutex::lock returned Err(Generic(error)).
    LoGenericError { error_code: int },
}

/// Abstract view of the outcome of ProcessManager::put_mutex_guard.
#[verifier::ext_equal]
pub enum PutGuardOutcomeView {
    /// put_mutex_guard succeeded.
    PgOk,
    /// put_mutex_guard failed with an error code.
    PgError { error_code: int },
}

/// Abstract view of the lock_mutex kcall's final result.
///
/// # Description
///
/// The lock_mutex function returns Result<(), SleepError>. This view models
/// all possible outcomes:
/// - Success: all three pipeline steps completed without error.
/// - InvalidTimeoutError: timeout parsing failed (nanoseconds >= 1B).
/// - GetMutexError: ProcessManager::get_mutex failed.
/// - LockTimedOut/LockKilled/LockGenericError: Mutex::lock failed with SleepError.
/// - PutGuardError: ProcessManager::put_mutex_guard failed.
#[verifier::ext_equal]
pub enum LockMutexResultView {
    /// All steps succeeded: mutex was locked and guard was stored.
    Success,
    /// Timeout parsing failed: SystemTime::new returned None.
    InvalidTimeoutError { error_code: int },
    /// ProcessManager::get_mutex failed (wrapped in SleepError::Generic).
    GetMutexError { error_code: int },
    /// Mutex::lock returned Interrupted(TimedOut).
    LockTimedOut,
    /// Mutex::lock returned Interrupted(Killed).
    LockKilled,
    /// Mutex::lock returned Generic(error).
    LockGenericError { error_code: int },
    /// ProcessManager::put_mutex_guard failed (wrapped in SleepError::Generic).
    PutGuardError { error_code: int },
}

//==================================================================================================
// Spec Functions
//==================================================================================================

/// Spec function: whether timeout parameters indicate an infinite wait.
///
/// # Description
///
/// The original code checks `timeout_s == usize::MAX && timeout_ns == usize::MAX`.
/// On x86-32, usize::MAX == u32::MAX == 0xFFFF_FFFF.
pub open spec fn spec_is_infinite_timeout(timeout_s: nat, timeout_ns: nat) -> bool {
    timeout_s == USIZE_MAX_X86_32() && timeout_ns == USIZE_MAX_X86_32()
}

/// Spec function: whether finite timeout nanoseconds are valid.
///
/// # Description
///
/// SystemTime::new(seconds, nanoseconds) returns None when nanoseconds >= 1_000_000_000.
/// This spec checks the validity of the nanoseconds component.
pub open spec fn spec_timeout_ns_valid(timeout_ns: nat) -> bool {
    timeout_ns < NANOS_PER_SEC()
}

/// Spec function: parse the timeout from raw parameters.
///
/// # Description
///
/// Models the timeout parsing logic from the original code:
/// - Both MAX → None (Infinite timeout).
/// - Valid nanoseconds → Some(Finite { seconds, nanoseconds }).
/// - Invalid nanoseconds → None (error path).
///
/// Returns Some(TimeoutView) on success, None on invalid timeout.
///
/// **Edge case**: When only one of (timeout_s, timeout_ns) is MAX and the
/// other is a valid value, the code falls through to `SystemTime::new`. This
/// can produce extreme but valid timeouts (e.g., `Finite { seconds: 4294967295,
/// nanoseconds: 0 }`). This matches the original code's behavior exactly —
/// the infinite-timeout sentinel requires *both* parameters to be MAX.
pub open spec fn spec_parse_timeout(timeout_s: nat, timeout_ns: nat) -> Option<TimeoutView> {
    if spec_is_infinite_timeout(timeout_s, timeout_ns) {
        Some(TimeoutView::Infinite)
    } else if spec_timeout_ns_valid(timeout_ns) {
        Some(TimeoutView::Finite { seconds: timeout_s, nanoseconds: timeout_ns })
    } else {
        None
    }
}

/// Spec function: models the complete lock_mutex pipeline.
///
/// # Description
///
/// The lock_mutex function executes a sequential pipeline:
/// 1. Parse timeout → InvalidTimeoutError on failure.
/// 2. get_mutex → GetMutexError on failure.
/// 3. lock(timeout) → LockError variant on failure.
/// 4. put_mutex_guard → PutGuardError on failure.
/// 5. All succeeded → Success.
///
/// Each step only executes if all previous steps succeeded (short-circuit).
pub open spec fn spec_lock_mutex_result(
    timeout_s: nat,
    timeout_ns: nat,
    get_mutex_outcome: GetMutexOutcomeView,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
) -> LockMutexResultView {
    match spec_parse_timeout(timeout_s, timeout_ns) {
        None => LockMutexResultView::InvalidTimeoutError {
            error_code: ERROR_CODE_INVALID_ARGUMENT(),
        },
        Some(_timeout) => {
            match get_mutex_outcome {
                GetMutexOutcomeView::GmError { error_code } => {
                    LockMutexResultView::GetMutexError { error_code }
                },
                GetMutexOutcomeView::GmOk => {
                    match lock_outcome {
                        LockOutcomeView::LoTimedOut => LockMutexResultView::LockTimedOut,
                        LockOutcomeView::LoKilled => LockMutexResultView::LockKilled,
                        LockOutcomeView::LoGenericError { error_code } => {
                            LockMutexResultView::LockGenericError { error_code }
                        },
                        LockOutcomeView::LoOk => {
                            match put_guard_outcome {
                                PutGuardOutcomeView::PgError { error_code } => {
                                    LockMutexResultView::PutGuardError { error_code }
                                },
                                PutGuardOutcomeView::PgOk => LockMutexResultView::Success,
                            }
                        },
                    }
                },
            }
        },
    }
}

/// Spec function: whether the result is success.
pub open spec fn spec_is_success(result: LockMutexResultView) -> bool {
    matches!(result, LockMutexResultView::Success)
}

/// Spec function: whether the result is any kind of error.
///
/// # Description
///
/// Defined as the complement of `spec_is_success`. This is correct because
/// `lemma_result_exhaustive` proves the result enum is exhaustive: every
/// result is either `Success` or one of the error variants — there is no
/// "unknown" category.
pub open spec fn spec_is_error(result: LockMutexResultView) -> bool {
    !spec_is_success(result)
}

/// Spec function: whether the result is a timeout parsing error.
pub open spec fn spec_is_timeout_error(result: LockMutexResultView) -> bool {
    matches!(result, LockMutexResultView::InvalidTimeoutError { .. })
}

/// Spec function: whether the result is a get_mutex error.
pub open spec fn spec_is_get_mutex_error(result: LockMutexResultView) -> bool {
    matches!(result, LockMutexResultView::GetMutexError { .. })
}

/// Spec function: whether the result is a lock error (any variant).
pub open spec fn spec_is_lock_error(result: LockMutexResultView) -> bool {
    matches!(result, LockMutexResultView::LockTimedOut
        | LockMutexResultView::LockKilled
        | LockMutexResultView::LockGenericError { .. })
}

/// Spec function: whether the result is a put_guard error.
pub open spec fn spec_is_put_guard_error(result: LockMutexResultView) -> bool {
    matches!(result, LockMutexResultView::PutGuardError { .. })
}

/// Spec function: whether the timeout was successfully parsed.
pub open spec fn spec_timeout_parsed_ok(timeout_s: nat, timeout_ns: nat) -> bool {
    spec_parse_timeout(timeout_s, timeout_ns).is_some()
}

/// Spec function: well-formedness of a finite timeout.
pub open spec fn spec_finite_timeout_wf(seconds: nat, nanoseconds: nat) -> bool {
    nanoseconds < NANOS_PER_SEC()
}

/// Spec function: whether a lock outcome is valid given the timeout type.
///
/// # Description
///
/// `Interrupted(TimedOut)` can only occur when a finite timeout is provided
/// (`timeout.is_some()` in the original). With an infinite timeout (None),
/// the `Condvar::wait()` path has no timer, so TimedOut is impossible.
/// This constraint tightens the external_body contract for `Mutex::lock()`.
pub open spec fn spec_lock_outcome_valid_for_timeout(has_timeout: bool, outcome: LockOutcomeView) -> bool {
    // TimedOut requires a finite timeout.
    matches!(outcome, LockOutcomeView::LoTimedOut) ==> has_timeout
}

/// Spec function: whether a parsed timeout is finite.
///
/// # Description
///
/// Returns true when the parsed timeout is a `Finite` variant (not Infinite).
/// The timeout value (seconds/nanoseconds) is captured in `TimeoutView::Finite`
/// and is threaded through the exec model via ghost state to prove that
/// `mutex_lock_model` receives the correct parsed timeout value.
pub open spec fn spec_is_finite_timeout(timeout_s: nat, timeout_ns: nat) -> bool {
    spec_parse_timeout(timeout_s, timeout_ns) matches Some(TimeoutView::Finite { .. })
}

/// Spec function: extract the parsed timeout value for the lock step.
///
/// # Description
///
/// Returns the `Option<TimeoutView>` that the exec model should pass to the
/// lock step. This connects the raw (timeout_s, timeout_ns) inputs to the
/// concrete timeout value that `Mutex::lock` receives:
/// - Infinite → None (no timeout).
/// - Finite { s, ns } → Some(Finite { s, ns }) with well-formed nanoseconds.
///
/// Used in the exec model's postcondition to prove that the correct timeout
/// value reaches `mutex_lock_model`.
pub open spec fn spec_parsed_timeout_for_lock(timeout_s: nat, timeout_ns: nat) -> Option<TimeoutView> {
    match spec_parse_timeout(timeout_s, timeout_ns) {
        Some(TimeoutView::Infinite) => None,
        Some(tv @ TimeoutView::Finite { .. }) => Some(tv),
        None => None,  // Don't-care: this path returns InvalidTimeoutError before lock.
    }
}

//==================================================================================================
// Caller Safety Contract Spec Predicates
//==================================================================================================

/// Spec predicate: the calling process is not the kernel process.
///
/// # Description
///
/// Encodes the first safety requirement from the original `lock_mutex` function:
/// "The calling process is not the kernel process."
///
/// This is an abstract predicate over the caller's process identity. It cannot
/// be verified within this module because process identity is global scheduler
/// state managed by the ProcessManager. The PM module's verification should
/// establish this predicate before invoking `lock_mutex`.
///
/// The `pid` parameter corresponds to the `pid: ProcessIdentifier` from the
/// original function signature, which is omitted from the exec model because
/// it only appears in trace logging.
pub uninterp spec fn spec_caller_is_not_kernel_process(pid: nat) -> bool;

/// Spec predicate: the calling thread holds no resources.
///
/// # Description
///
/// Encodes the second safety requirement from the original `lock_mutex` function:
/// "This function is invoked without holding any resources."
///
/// This is an abstract predicate over the calling thread's resource ownership
/// (e.g., other mutex guards, memory locks). It cannot be verified within this
/// module because resource tracking is global mutable state managed by the
/// ProcessManager's thread bookkeeping. The PM module's verification should
/// establish this predicate before invoking `lock_mutex`.
pub uninterp spec fn spec_caller_holds_no_resources(tid: nat) -> bool;

/// Spec predicate: the caller does not hold a ProcessManager reference.
///
/// # Description
///
/// Encodes the third safety requirement from the original `lock_mutex` function:
/// "The calling process does not hold a reference to the process manager."
///
/// This prevents re-entrancy issues where `lock_mutex` internally accesses the
/// ProcessManager (via `get_mutex`/`put_mutex_guard`) while the caller still
/// holds a mutable reference. This is an abstract predicate over borrow state
/// that cannot be verified here — Rust's borrow checker enforces it at compile
/// time for safe code, and the PM module should verify it for unsafe contexts.
pub uninterp spec fn spec_caller_no_pm_reference() -> bool;

/// Spec predicate: all three caller safety requirements are satisfied.
///
/// # Description
///
/// Convenience predicate combining the three safety requirements from the
/// original `lock_mutex` function's `# Safety` documentation. Call sites
/// (in the PM/kcall dispatch layer) should establish this predicate before
/// invoking `lock_mutex`.
///
/// These predicates are intentionally abstract (uninterpreted) in this module
/// because the concrete definitions depend on ProcessManager state, which is
/// out of scope. The PM module should provide concrete interpretations.
pub open spec fn spec_lock_mutex_safety_preconditions(pid: nat, tid: nat) -> bool {
    spec_caller_is_not_kernel_process(pid)
    && spec_caller_holds_no_resources(tid)
    && spec_caller_no_pm_reference()
}

/// Spec function: pipeline result parameterized by caller context (pid/tid).
///
/// # Description
///
/// Wraps `spec_lock_mutex_result` to include `pid` and `tid` in the signature,
/// matching the original `lock_mutex(pid, tid, mutex_addr, timeout_s, timeout_ns)`
/// function. The implementation delegates entirely to `spec_lock_mutex_result`,
/// ignoring pid/tid — reflecting the fact that the original uses them only in
/// `trace!()` logging.
///
/// This wrapper exists to give `lemma_result_independent_of_pid_tid` a
/// non-trivial proof obligation: it proves that two calls with different
/// (pid, tid) values yield identical results. If a future refactoring makes
/// the pipeline depend on pid/tid, this function's body must change, and the
/// lemma will require a new proof (or fail to verify).
pub open spec fn spec_lock_mutex_result_with_context(
    pid: nat,
    tid: nat,
    timeout_s: nat,
    timeout_ns: nat,
    get_mutex_outcome: GetMutexOutcomeView,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
) -> LockMutexResultView {
    // pid and tid are intentionally unused — they affect only trace logging.
    spec_lock_mutex_result(timeout_s, timeout_ns, get_mutex_outcome, lock_outcome, put_guard_outcome)
}

/// Spec function: whether a timeout_view ghost is consistent with has_timeout.
///
/// # Description
///
/// Encodes the consistency invariant between the boolean `has_timeout` flag
/// and the ghost `Option<TimeoutView>` value passed to `mutex_lock_model`:
/// - `has_timeout == true` iff the ghost is `Some(Finite { .. })`.
/// - `has_timeout == false` iff the ghost is `None`.
///
/// This constraint ensures that `mutex_lock_model`'s external_body contract
/// is self-consistent: the boolean flag and the ghost value agree.
pub open spec fn spec_timeout_view_consistent(has_timeout: bool, timeout_view: Option<TimeoutView>) -> bool {
    &&& (has_timeout <==> timeout_view matches Some(TimeoutView::Finite { .. }))
    &&& (!has_timeout ==> timeout_view.is_none())
}

/// Spec predicate: the guard ownership for the given mutex has been released.
///
/// # Description
///
/// Models the outcome of `MutexGuard::drop()` — the guard for `mutex_addr`
/// has been consumed and the mutex lock released. In the original code,
/// `put_mutex_guard` takes `MutexGuard` by value (move semantics):
/// - On success, the guard is stored in the thread's bookkeeping.
/// - On failure, the guard is dropped at scope exit, and `MutexGuard::drop()`
///   unlocks the mutex.
///
/// Either way, the guard ownership is transferred out of the caller. This
/// predicate is abstract (uninterpreted) because the concrete unlock
/// semantics are defined in the mutex module, not in this pipeline.
/// The mutex module should provide a concrete interpretation and prove
/// that `MutexGuard::drop()` always releases the lock.
pub uninterp spec fn spec_guard_ownership_released(mutex_addr: nat) -> bool;

} // verus!
