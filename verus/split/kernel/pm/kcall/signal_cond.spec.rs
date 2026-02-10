// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Signal Condition Kernel Call Specification.
// Defines View types, spec constants, and spec functions for the signal_cond
// kernel call verification model.
//
// ## Verification Model
//
// The signal_cond kcall converts a user-provided `cond_addr: usize` into a
// `ConditionAddress`, then executes a three-step pipeline:
//   1. ProcessManager::get_cond(cond_addr) → Condvar reference.
//   2. cond.notify_all() or cond.notify_first() → awakened count.
//      (Condvar is dropped at end of block, decrementing reference count.)
//   3. ProcessManager::put_cond(cond_addr) → release condition variable slot.
//
// All three steps use `?` for error propagation (short-circuit on failure).
// Step 3 only executes if step 2 succeeded. The Condvar is dropped (step 2b)
// both on success and on notify error (Rust drops locals on scope exit).
//
// This spec models:
// - The three-step sequential pipeline with short-circuit errors.
// - The broadcast vs. signal (notify_first) choice.
// - Error propagation: each step's error is propagated unchanged.
// - Success returns the number of awakened threads.

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
pub open spec fn USIZE_BITS() -> nat {
    32
}

/// Spec predicate: whether an error code is a valid `ErrorCode` discriminant.
///
/// # Description
///
/// All `ErrorCode` enum variants are positive integers (POSIX errno values).
pub open spec fn spec_is_valid_error_code(code: int) -> bool {
    code > 0
}

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of the outcome of ProcessManager::get_cond.
#[verifier::ext_equal]
pub enum GetCondOutcomeView {
    /// get_cond succeeded, returning a Condvar reference.
    GcOk,
    /// get_cond failed with an error code.
    GcError { error_code: int },
}

/// Abstract view of the outcome of cond.notify_all() or cond.notify_first().
///
/// # Description
///
/// Both notify_all and notify_first return `Result<u32, Error>` where the
/// u32 is the number of awakened threads. The abstract view captures success
/// with the count, or failure with an error code.
#[verifier::ext_equal]
pub enum NotifyOutcomeView {
    /// notify succeeded, awakening `awakened` threads.
    NOk { awakened: nat },
    /// notify failed with an error code.
    NError { error_code: int },
}

/// Abstract view of the outcome of ProcessManager::put_cond.
#[verifier::ext_equal]
pub enum PutCondOutcomeView {
    /// put_cond succeeded.
    PcOk,
    /// put_cond failed with an error code.
    PcError { error_code: int },
}

/// Abstract view of the signal_cond kcall's final result.
///
/// # Description
///
/// The signal_cond function returns `Result<u32, Error>`. This view models
/// all possible outcomes from the three-step pipeline:
/// - Success: all steps succeeded, returns awakened count.
/// - GetCondError: ProcessManager::get_cond failed.
/// - NotifyError: notify_all or notify_first failed.
/// - PutCondError: ProcessManager::put_cond failed.
#[verifier::ext_equal]
pub enum SignalCondResultView {
    /// All steps succeeded; `awakened` threads were woken.
    Success { awakened: nat },
    /// ProcessManager::get_cond failed.
    GetCondError { error_code: int },
    /// notify_all or notify_first failed.
    NotifyError { error_code: int },
    /// ProcessManager::put_cond failed.
    PutCondError { error_code: int },
}

/// Ghost state bundle capturing all step outcomes for exec-spec linkage.
///
/// # Description
///
/// Returned as a ghost value from `signal_cond_model` to prove the exec result
/// matches `spec_signal_cond_result` applied to the actual step outcomes.
/// When a step is not reached (due to prior error), its field is set to an
/// arbitrary don't-care value — the spec ignores it due to short-circuit.
#[verifier::ext_equal]
pub struct SignalCondGhostState {
    /// Outcome of ProcessManager::get_cond.
    pub gc: GetCondOutcomeView,
    /// Outcome of notify_all or notify_first (don't-care when get_cond fails).
    pub notify: NotifyOutcomeView,
    /// Outcome of ProcessManager::put_cond (don't-care when notify fails).
    pub pc: PutCondOutcomeView,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

/// Spec function: models the complete signal_cond pipeline.
///
/// # Description
///
/// The signal_cond function executes the following sequential pipeline:
/// 1. get_cond → GetCondError on failure (short-circuit).
/// 2. notify (broadcast or signal) → NotifyError on failure (short-circuit).
///    (Condvar dropped at end of block regardless.)
/// 3. put_cond → PutCondError on failure (short-circuit).
/// 4. Return Ok(awakened).
///
/// All errors short-circuit: later steps are not reached.
pub open spec fn spec_signal_cond_result(
    get_cond_outcome: GetCondOutcomeView,
    notify_outcome: NotifyOutcomeView,
    put_cond_outcome: PutCondOutcomeView,
) -> SignalCondResultView {
    match get_cond_outcome {
        GetCondOutcomeView::GcError { error_code } => {
            SignalCondResultView::GetCondError { error_code }
        },
        GetCondOutcomeView::GcOk => {
            match notify_outcome {
                NotifyOutcomeView::NError { error_code } => {
                    SignalCondResultView::NotifyError { error_code }
                },
                NotifyOutcomeView::NOk { awakened } => {
                    match put_cond_outcome {
                        PutCondOutcomeView::PcError { error_code } => {
                            SignalCondResultView::PutCondError { error_code }
                        },
                        PutCondOutcomeView::PcOk => {
                            SignalCondResultView::Success { awakened }
                        },
                    }
                },
            }
        },
    }
}

/// Spec function: whether the result is success.
pub open spec fn spec_is_success(result: SignalCondResultView) -> bool {
    matches!(result, SignalCondResultView::Success { .. })
}

/// Spec function: whether the result is any kind of error.
pub open spec fn spec_is_error(result: SignalCondResultView) -> bool {
    !spec_is_success(result)
}

/// Spec function: whether the result is a get_cond error.
pub open spec fn spec_is_get_cond_error(result: SignalCondResultView) -> bool {
    matches!(result, SignalCondResultView::GetCondError { .. })
}

/// Spec function: whether the result is a notify error.
pub open spec fn spec_is_notify_error(result: SignalCondResultView) -> bool {
    matches!(result, SignalCondResultView::NotifyError { .. })
}

/// Spec function: whether the result is a put_cond error.
pub open spec fn spec_is_put_cond_error(result: SignalCondResultView) -> bool {
    matches!(result, SignalCondResultView::PutCondError { .. })
}

/// Spec function: pipeline result parameterized by caller context (pid/tid/broadcast).
///
/// # Description
///
/// Wraps `spec_signal_cond_result` to include `pid`, `tid`, and `broadcast`
/// in the signature, matching the original `signal_cond(pid, tid, cond_addr, broadcast)`.
/// The `broadcast` flag selects between notify_all and notify_first, but both
/// produce the same `NotifyOutcomeView` type. The `pid` and `tid` only affect
/// which PM outcome is produced — the pipeline mapping is independent of them.
///
/// This wrapper exists purely for traceability to the original API signature
/// and to support `lemma_result_mapping_independent_of_context`, which proves
/// the pipeline mapping is context-independent.
pub open spec fn spec_signal_cond_result_with_context(
    pid: nat,
    tid: nat,
    broadcast: bool,
    get_cond_outcome: GetCondOutcomeView,
    notify_outcome: NotifyOutcomeView,
    put_cond_outcome: PutCondOutcomeView,
) -> SignalCondResultView {
    spec_signal_cond_result(get_cond_outcome, notify_outcome, put_cond_outcome)
}

/// Spec function: on success, the awakened count matches the notify outcome.
pub open spec fn spec_success_awakened_count(
    result: SignalCondResultView,
    notify_outcome: NotifyOutcomeView,
) -> bool {
    spec_is_success(result) ==> (
        notify_outcome matches NotifyOutcomeView::NOk { awakened }
        && result == SignalCondResultView::Success { awakened }
    )
}

//==================================================================================================
// Caller Safety Contract Spec Predicates
//==================================================================================================

/// Spec predicate: the caller does not hold a ProcessManager reference.
///
/// # Description
///
/// Encodes the safety requirement from the original `signal_cond` function:
/// "The calling process does not hold a reference to the process manager."
pub uninterp spec fn spec_caller_no_pm_reference() -> bool;

/// Spec predicate: all safety requirements are satisfied.
pub open spec fn spec_signal_cond_safety_preconditions() -> bool {
    spec_caller_no_pm_reference()
}

/// Spec predicate: a valid Condvar reference has been acquired for cond_addr.
///
/// # Description
///
/// Models the fact that `get_cond` has previously succeeded for this
/// address, meaning a valid Condvar reference exists. Used as a
/// precondition on `notify_model` and `drop_cond_model` to document
/// the dependency on a prior successful `get_cond` call.
pub uninterp spec fn spec_condvar_acquired(cond_addr: nat) -> bool;

/// Spec predicate: the condition variable reference count was decremented.
///
/// # Description
///
/// Abstract postcondition token established by `drop_cond_model` (trust
/// boundary T3). Models the effect of dropping the Condvar at the end of
/// the block. In the original code, the Condvar goes out of scope after
/// the notify call, causing its reference count to decrease.
///
/// This predicate is intentionally uninterpreted at the kcall level. Its
/// concrete semantics (refcount decrement) are the responsibility of the
/// condvar module behind trust boundary T3. Here it serves as an abstract
/// token that the pipeline propagates to callers, allowing them to compose
/// resource-release reasoning across module boundaries.
pub uninterp spec fn spec_cond_ref_released(cond_addr: nat) -> bool;

/// Spec predicate: put_cond completed successfully.
///
/// # Description
///
/// Abstract postcondition token established by `put_cond_model` (trust
/// boundary T4). Models the effect of ProcessManager::put_cond returning
/// Ok(()). Note: "completed" means the put_cond call returned successfully,
/// NOT that the condvar entry was necessarily removed from the PM's map.
/// The real `put_cond` only removes the entry if `reference_count() <= 1`;
/// otherwise it returns Ok(()) without removal. The concrete reclamation
/// semantics are the responsibility of the PM module behind trust boundary
/// T4. Here it serves as an abstract token that the pipeline propagates
/// to callers.
pub uninterp spec fn spec_put_cond_completed(cond_addr: nat) -> bool;

/// Spec predicate: the number of threads waiting on a condvar.
///
/// # Description
///
/// Abstract predicate modeling the number of threads blocked on the condition
/// variable at `cond_addr`. Used to specify broadcast semantics: `notify_all`
/// awakens all waiters, while `notify_first` awakens at most one.
/// The concrete waiter count is managed by the condvar module; this is an
/// uninterpreted abstraction at the kcall trust boundary.
pub uninterp spec fn spec_num_waiters(cond_addr: nat) -> nat;

/// Spec predicate: broadcast semantics constraint on awakened count.
///
/// # Description
///
/// When `broadcast` is false (`notify_first`), at most one thread is awakened,
/// and the count cannot exceed the number of waiters (i.e., 0 when no waiters).
/// When `broadcast` is true (`notify_all`), the implementation uses best-effort
/// wakeup: it attempts to wake all waiters but individual `wakeup(tid)` calls
/// may fail. The count reflects successfully awakened threads, which is at most
/// the total number of waiters. The real implementation returns `Ok(count)`
/// as long as at least one thread was awakened (or there were no waiters).
///
/// Both cases enforce `awakened <= spec_num_waiters(cond_addr)`: you cannot
/// awaken more threads than are actually waiting.
pub open spec fn spec_broadcast_semantics(
    broadcast: bool,
    cond_addr: nat,
    awakened: nat,
) -> bool {
    // Common bound: cannot awaken more threads than are waiting.
    awakened <= spec_num_waiters(cond_addr) && (
        if broadcast {
            // notify_all: best-effort wakeup of all waiters.
            true
        } else {
            // notify_first: awakens at most one waiter.
            awakened <= 1
        }
    )
}

} // verus!
