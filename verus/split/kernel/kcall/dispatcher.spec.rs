// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Dispatcher Specification.
// Defines View types, spec constants, and spec functions for the kernel call
// dispatcher verification model.
//
// ## Verification Model
//
// The dispatcher is a routing function that maps KcallNumber values to handler
// paths. This spec models:
// - KcallNumber as u32 constants matching the original `#[repr(u32)]` enum.
// - DispatchCategory classifying each kcall into its handler path.
// - SleepErrorKind and InterruptKind modeling the error types.
// - DispatchResult modeling the KcallResult return type.
// - Spec functions for classification, error handling, and well-formedness.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Spec Constants: KcallNumber Values
//==================================================================================================

/// Kernel call number for Debug.
pub open spec fn KCALL_DEBUG() -> u32 { 0 }

/// Kernel call number for GetPid.
pub open spec fn KCALL_GET_PID() -> u32 { 1 }

/// Kernel call number for GetTid.
pub open spec fn KCALL_GET_TID() -> u32 { 2 }

/// Kernel call number for Exit.
pub open spec fn KCALL_EXIT() -> u32 { 3 }

/// Kernel call number for CapCtl.
pub open spec fn KCALL_CAP_CTL() -> u32 { 4 }

/// Kernel call number for Resume.
pub open spec fn KCALL_RESUME() -> u32 { 5 }

/// Kernel call number for Terminate.
pub open spec fn KCALL_TERMINATE() -> u32 { 6 }

/// Kernel call number for EventCtrl.
pub open spec fn KCALL_EVENT_CTRL() -> u32 { 7 }

/// Kernel call number for Send.
pub open spec fn KCALL_SEND() -> u32 { 8 }

/// Kernel call number for Recv.
pub open spec fn KCALL_RECV() -> u32 { 9 }

/// Kernel call number for MemoryMap.
pub open spec fn KCALL_MEMORY_MAP() -> u32 { 10 }

/// Kernel call number for MemoryUnmap.
pub open spec fn KCALL_MEMORY_UNMAP() -> u32 { 11 }

/// Kernel call number for MemoryCtrl.
pub open spec fn KCALL_MEMORY_CTRL() -> u32 { 12 }

/// Kernel call number for MemoryCopy.
pub open spec fn KCALL_MEMORY_COPY() -> u32 { 13 }

/// Kernel call number for AllocMmio.
pub open spec fn KCALL_ALLOC_MMIO() -> u32 { 14 }

/// Kernel call number for FreeMmio.
pub open spec fn KCALL_FREE_MMIO() -> u32 { 15 }

/// Kernel call number for AllocPmio.
pub open spec fn KCALL_ALLOC_PMIO() -> u32 { 16 }

/// Kernel call number for FreePmio.
pub open spec fn KCALL_FREE_PMIO() -> u32 { 17 }

/// Kernel call number for ReadPmio.
pub open spec fn KCALL_READ_PMIO() -> u32 { 18 }

/// Kernel call number for WritePmio.
pub open spec fn KCALL_WRITE_PMIO() -> u32 { 19 }

/// Kernel call number for SchedulerYield.
pub open spec fn KCALL_SCHEDULER_YIELD() -> u32 { 20 }

/// Kernel call number for CreateThread.
pub open spec fn KCALL_CREATE_THREAD() -> u32 { 21 }

/// Kernel call number for ExitThread.
pub open spec fn KCALL_EXIT_THREAD() -> u32 { 22 }

/// Kernel call number for JoinThread.
pub open spec fn KCALL_JOIN_THREAD() -> u32 { 23 }

/// Kernel call number for MutexLock.
pub open spec fn KCALL_MUTEX_LOCK() -> u32 { 24 }

/// Kernel call number for MutexUnlock.
pub open spec fn KCALL_MUTEX_UNLOCK() -> u32 { 25 }

/// Kernel call number for CondSignal.
pub open spec fn KCALL_COND_SIGNAL() -> u32 { 26 }

/// Kernel call number for CondWait.
pub open spec fn KCALL_COND_WAIT() -> u32 { 27 }

/// Kernel call number for GetTime.
pub open spec fn KCALL_GET_TIME() -> u32 { 28 }

/// Kernel call number for Sleep.
pub open spec fn KCALL_SLEEP() -> u32 { 29 }

/// Kernel call number for SetThreadDataArea.
pub open spec fn KCALL_SET_TDA() -> u32 { 30 }

/// Kernel call number for GetThreadDataArea.
pub open spec fn KCALL_GET_TDA() -> u32 { 31 }

/// Kernel call number for Invalid.
pub open spec fn KCALL_INVALID() -> u32 { u32::MAX as u32 }

/// The number of defined (valid) kernel call numbers (0..=31).
pub open spec fn KCALL_DEFINED_COUNT() -> nat { 32 }

//==================================================================================================
// View Types
//==================================================================================================

/// Classification of kernel call dispatch paths.
///
/// # Description
///
/// Categorizes each kernel call into the handler path taken by `do_kcall`:
/// - `LocalImmediate`: handled locally, returns immediately (GetPid, GetTid).
/// - `LocalTerminal`: handled locally, terminates process/thread (Exit, ExitThread).
/// - `LocalSleepable`: handled locally, may block/sleep (JoinThread, Recv, MutexLock,
///   CondWait, Sleep). On error, goes through `handle_sleep_error`.
/// - `LocalFallible`: handled locally, may fail but does not sleep (MutexUnlock,
///   CondSignal, SchedulerYield). Returns Ok or Error directly.
/// - `LocalDirect`: handled locally, returns KcallResult from subsystem directly (Resume).
/// - `Remote`: dispatched to the scoreboard for remote execution.
#[verifier::ext_equal]
pub enum DispatchCategory {
    /// Returns an identifier value immediately (GetPid, GetTid).
    LocalImmediate,
    /// Terminates process or thread (Exit, ExitThread).
    LocalTerminal,
    /// May block; errors go through handle_sleep_error (JoinThread, Recv, MutexLock, CondWait, Sleep).
    LocalSleepable,
    /// May fail but does not block (MutexUnlock, CondSignal, SchedulerYield).
    LocalFallible,
    /// Returns KcallResult directly from subsystem (Resume).
    LocalDirect,
    /// Dispatched to scoreboard for remote execution.
    Remote,
}

/// The kind of sleep error.
///
/// # Description
///
/// Models `SleepError` from the process manager:
/// - `Generic`: a regular error with an error code.
/// - `InterruptedKilled`: the process was killed.
/// - `InterruptedTimedOut`: a timer expired.
#[verifier::ext_equal]
pub enum SleepErrorKind {
    /// A generic error with an associated error code value.
    Generic,
    /// The thread was interrupted because the process was killed.
    InterruptedKilled,
    /// The thread was interrupted because a timer expired.
    InterruptedTimedOut,
}

/// Abstract view of a dispatch result.
///
/// # Description
///
/// Models the `KcallResult` return value from the dispatcher:
/// - `is_success`: true for success variant, false for error variant.
/// - `value`: the payload (i64 for success, error code as int for error).
#[verifier::ext_equal]
pub struct DispatchResultView {
    /// Whether this result is a success.
    pub is_success: bool,
    /// The payload value.
    pub value: int,
}

/// Abstract view of the dispatch arguments.
///
/// # Description
///
/// Models the five u32 arguments and the kcall number passed to `do_kcall`.
#[verifier::ext_equal]
pub struct DispatchArgsView {
    /// Kernel call number.
    pub number: nat,
    /// First argument.
    pub arg0: nat,
    /// Second argument.
    pub arg1: nat,
    /// Third argument.
    pub arg2: nat,
    /// Fourth argument.
    pub arg3: nat,
}

//==================================================================================================
// Spec Functions: KcallNumber Classification
//==================================================================================================

/// Spec function: checks if a kcall number is a defined (valid) value.
///
/// # Description
///
/// Returns true if the number corresponds to one of the 32 defined kernel calls
/// (0..=31). Returns false for all other values including Invalid (u32::MAX).
pub open spec fn spec_is_defined_kcall(number: u32) -> bool {
    number <= 31
}

/// Spec function: classifies a kernel call number into its dispatch category.
///
/// # Description
///
/// Maps each u32 kernel call number to the dispatch path taken by `do_kcall`.
/// This function mirrors the match statement in the original dispatcher exactly.
pub open spec fn spec_classify_kcall(number: u32) -> DispatchCategory {
    if number == KCALL_GET_PID() || number == KCALL_GET_TID() {
        DispatchCategory::LocalImmediate
    } else if number == KCALL_EXIT() || number == KCALL_EXIT_THREAD() {
        DispatchCategory::LocalTerminal
    } else if number == KCALL_JOIN_THREAD() || number == KCALL_RECV()
        || number == KCALL_MUTEX_LOCK() || number == KCALL_COND_WAIT()
        || number == KCALL_SLEEP() {
        DispatchCategory::LocalSleepable
    } else if number == KCALL_MUTEX_UNLOCK() || number == KCALL_COND_SIGNAL()
        || number == KCALL_SCHEDULER_YIELD() {
        DispatchCategory::LocalFallible
    } else if number == KCALL_RESUME() {
        DispatchCategory::LocalDirect
    } else {
        DispatchCategory::Remote
    }
}

/// Spec function: checks if a kernel call is handled locally.
///
/// # Description
///
/// Returns true if the kcall is handled directly by `do_kcall` without
/// dispatching to the scoreboard. This includes all non-Remote categories.
pub open spec fn spec_is_locally_handled(number: u32) -> bool {
    !matches!(spec_classify_kcall(number), DispatchCategory::Remote)
}

/// Spec function: checks if a kernel call may block (sleep).
///
/// # Description
///
/// Returns true if the kcall may cause the calling thread to sleep.
/// These calls go through `handle_sleep_error` on failure.
pub open spec fn spec_is_sleepable(number: u32) -> bool {
    matches!(spec_classify_kcall(number), DispatchCategory::LocalSleepable)
}

/// Spec function: checks if a kernel call is dispatched remotely.
///
/// # Description
///
/// Returns true if the kcall is dispatched to the scoreboard for remote
/// execution by the kernel thread.
pub open spec fn spec_is_remote(number: u32) -> bool {
    matches!(spec_classify_kcall(number), DispatchCategory::Remote)
}

//==================================================================================================
// Spec Functions: Error Handling
//==================================================================================================

/// Spec function: the error code value for OperationTimedOut.
///
/// # Description
///
/// ErrorCode::OperationTimedOut = 110 (ETIMEDOUT in Linux).
pub open spec fn SPEC_ERROR_TIMED_OUT() -> int { 110 }

/// Spec function: models `handle_sleep_error` at the spec level.
///
/// # Description
///
/// Only models the two non-divergent sleep error kinds:
/// - `Generic(error)` → Error result with the given error code.
/// - `InterruptedTimedOut` → Error result with OperationTimedOut code.
///
/// `InterruptedKilled` is excluded via precondition on `handle_sleep_error`
/// because the original code diverges (calls `ProcessManager::exit` then
/// panics). It is modeled separately as a divergent trust boundary.
pub open spec fn spec_handle_sleep_error(kind: SleepErrorKind, error_code: int) -> DispatchResultView {
    match kind {
        SleepErrorKind::Generic => DispatchResultView {
            is_success: false,
            value: error_code,
        },
        SleepErrorKind::InterruptedTimedOut => DispatchResultView {
            is_success: false,
            value: SPEC_ERROR_TIMED_OUT(),
        },
        // Unreachable: excluded by precondition on handle_sleep_error.
        SleepErrorKind::InterruptedKilled => DispatchResultView {
            is_success: false,
            value: -1,
        },
    }
}

/// Spec function: checks if a sleep error kind is non-divergent.
///
/// # Description
///
/// Returns true for the two sleep error kinds that produce a return value
/// (Generic, InterruptedTimedOut). Returns false for InterruptedKilled,
/// which causes process termination and never returns.
pub open spec fn spec_sleep_error_returns(kind: SleepErrorKind) -> bool {
    !matches!(kind, SleepErrorKind::InterruptedKilled)
}

//==================================================================================================
// Spec Functions: Result Well-Formedness
//==================================================================================================

/// Spec function: checks if a dispatch result is well-formed.
///
/// # Description
///
/// A result is well-formed when:
/// - Success values are valid i64 (always true at spec level).
/// - Error values fit in i32 range.
pub open spec fn spec_result_wf(result: DispatchResultView) -> bool {
    if result.is_success {
        result.value >= i64::MIN as int && result.value <= i64::MAX as int
    } else {
        result.value >= i32::MIN as int && result.value <= i32::MAX as int
    }
}

/// Spec function: constructs a success result view.
pub open spec fn spec_success_result(value: int) -> DispatchResultView {
    DispatchResultView {
        is_success: true,
        value: value,
    }
}

/// Spec function: constructs an error result view.
pub open spec fn spec_error_result(code: int) -> DispatchResultView {
    DispatchResultView {
        is_success: false,
        value: code,
    }
}

/// Spec function: the success result with value 0 (ok).
pub open spec fn spec_ok_result() -> DispatchResultView {
    DispatchResultView {
        is_success: true,
        value: 0,
    }
}

//==================================================================================================
// Spec Functions: View Implementations
//==================================================================================================

impl DispatchResult {
    /// Spec function: returns the abstract view.
    pub open spec fn spec_view(&self) -> DispatchResultView {
        DispatchResultView {
            is_success: self.is_success,
            value: self.value as int,
        }
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        if self.is_success {
            true
        } else {
            self.value >= i32::MIN as i64 && self.value <= i32::MAX as i64
        }
    }
}

impl View for DispatchResult {
    type V = DispatchResultView;

    open spec fn view(&self) -> DispatchResultView {
        self.spec_view()
    }
}

impl DispatchArgs {
    /// Spec function: returns the abstract view.
    pub open spec fn spec_view(&self) -> DispatchArgsView {
        DispatchArgsView {
            number: self.number as nat,
            arg0: self.arg0 as nat,
            arg1: self.arg1 as nat,
            arg2: self.arg2 as nat,
            arg3: self.arg3 as nat,
        }
    }

    /// Spec function: well-formedness predicate.
    ///
    /// # Description
    ///
    /// Intentionally `true` for all u32 values: the dispatcher's wildcard
    /// match arm handles any kcall number (including Invalid/undefined) by
    /// dispatching to the scoreboard. No u32 value is rejected at the
    /// argument level.
    pub open spec fn wf(&self) -> bool {
        true
    }
}

impl View for DispatchArgs {
    type V = DispatchArgsView;

    open spec fn view(&self) -> DispatchArgsView {
        self.spec_view()
    }
}

impl SleepError {
    /// Spec function: returns the kind of this sleep error.
    pub open spec fn spec_kind(&self) -> SleepErrorKind {
        self.kind
    }

    /// Spec function: returns the error code (meaningful only for Generic kind).
    pub open spec fn spec_error_code(&self) -> int {
        self.error_code as int
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        match self.kind {
            SleepErrorKind::Generic => {
                self.error_code >= i32::MIN as i64 && self.error_code <= i32::MAX as i64
            },
            _ => true,
        }
    }
}

impl SleepableOutcome {
    /// Spec function: well-formedness predicate.
    ///
    /// # Description
    ///
    /// A SleepableOutcome is well-formed when:
    /// - On success: always true (any i64 value is valid).
    /// - On failure with Generic: the error code fits in i32 range.
    /// - On failure with other kinds: always true.
    pub open spec fn wf(&self) -> bool {
        if self.succeeded {
            true
        } else {
            match self.sleep_error_kind {
                SleepErrorKind::Generic => {
                    self.sleep_error_code >= i32::MIN as i64
                        && self.sleep_error_code <= i32::MAX as i64
                },
                _ => true,
            }
        }
    }
}

impl FallibleOutcome {
    /// Spec function: well-formedness predicate.
    ///
    /// # Description
    ///
    /// Always true because `error_code` is i32 (automatically in range).
    pub open spec fn wf(&self) -> bool {
        true
    }
}

impl ScoreboardDispatchOutcome {
    /// Spec function: well-formedness predicate.
    ///
    /// # Description
    ///
    /// A ScoreboardDispatchOutcome is well-formed when:
    /// - On success with error result: the value fits in i32 range.
    /// - On failure with Generic: the sleep error code fits in i32 range.
    /// - Otherwise: always true.
    pub open spec fn wf(&self) -> bool {
        if self.succeeded {
            if self.result_is_success {
                true
            } else {
                self.result_value >= i32::MIN as i64 && self.result_value <= i32::MAX as i64
            }
        } else {
            match self.sleep_error_kind {
                SleepErrorKind::Generic => {
                    self.sleep_error_code >= i32::MIN as i64
                        && self.sleep_error_code <= i32::MAX as i64
                },
                _ => true,
            }
        }
    }
}

//==================================================================================================
// Spec Functions: KcallResult → i64 Encoding
//==================================================================================================

/// Spec function: models the `Into<i64>` conversion from KcallResult.
///
/// # Description
///
/// The original `KcallResult` converts to i64 as follows:
/// - `Success(KcallSuccess(v))` → `v` (the i64 value directly).
/// - `Error(KcallError(e))` → `e as i64` (i32 sign-extended to i64).
///
/// The encoded i64 loses the success/error type tag. The caller must know
/// the call's outcome from context (e.g., checking an errno convention).
/// This is a representation gap in the C ABI — the verified model uses
/// `DispatchResult` with an explicit `is_success` flag while the actual
/// ABI uses a raw `i64`. See trust boundary T5 in dispatcher.rs.
pub open spec fn spec_encode_result(r: DispatchResultView) -> int {
    r.value
}

/// Spec function: checks if an encoded i64 value is in the error range.
///
/// # Description
///
/// Error payloads originate from `KcallError(i32)`, so they always fit in
/// i32 range. This is a necessary (but not sufficient) condition for
/// distinguishing errors from success values that exceed i32 range.
pub open spec fn spec_in_error_range(encoded: int) -> bool {
    encoded >= i32::MIN as int && encoded <= i32::MAX as int
}

/// Spec function: models the encoded result for specific dispatch categories.
///
/// # Description
///
/// Constrains the encoded i64 result based on the dispatch category:
/// - `LocalImmediate` (GetPid, GetTid): value is non-negative (pid/tid ≥ 0).
/// - `LocalTerminal` (Exit, ExitThread): value is in error range (always fails).
/// - Others: no structural constraint beyond well-formedness.
pub open spec fn spec_dispatch_result_constrained(
    category: DispatchCategory,
    result: DispatchResultView,
) -> bool {
    match category {
        // LocalImmediate may fail if pid/tid retrieval fails;
        // success is only guaranteed when ProcessManager is accessible
        // (see do_kcall_dispatch for the stronger conditional postcondition).
        DispatchCategory::LocalTerminal => !result.is_success,
        _ => true,
    }
}

//==================================================================================================
// Spec Functions: Dispatch Table Properties
//==================================================================================================

/// Spec function: enumerates all locally-handled kcall numbers.
///
/// # Description
///
/// Returns true for exactly the 13 kernel call numbers that are handled
/// locally by `do_kcall` (not dispatched to the scoreboard).
pub open spec fn spec_locally_handled_set(number: u32) -> bool {
    number == KCALL_GET_PID()
    || number == KCALL_GET_TID()
    || number == KCALL_EXIT()
    || number == KCALL_EXIT_THREAD()
    || number == KCALL_JOIN_THREAD()
    || number == KCALL_RECV()
    || number == KCALL_RESUME()
    || number == KCALL_MUTEX_LOCK()
    || number == KCALL_MUTEX_UNLOCK()
    || number == KCALL_COND_WAIT()
    || number == KCALL_COND_SIGNAL()
    || number == KCALL_SCHEDULER_YIELD()
    || number == KCALL_SLEEP()
}

} // verus!
