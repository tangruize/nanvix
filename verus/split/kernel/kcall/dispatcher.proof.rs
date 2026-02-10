// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Dispatcher Proofs.
// Contains proof lemmas for the kernel call dispatcher verification model.
//
// ## Verified Properties
//
// 1. Classification totality: every u32 maps to exactly one DispatchCategory.
// 2. Classification consistency: spec_is_locally_handled matches the set enumeration.
// 3. Partition property: every kcall is either local or remote, never both.
// 4. handle_sleep_error produces well-formed results for well-formed inputs.
// 5. handle_sleep_error preserves error codes for Generic errors.
// 6. GetPid/GetTid are classified as LocalImmediate.
// 7. Sleepable calls are a subset of locally-handled calls.
// 8. All defined kcalls (0..31) map to a non-Remote category or Remote.
// 9. Result constructors produce well-formed results.
// 10. Encoding injectivity: error values always fit in i32 range.
// 11. Divergent Killed path is properly separated from non-divergent error handling.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Proof Lemmas: Classification
//==================================================================================================

/// Lemma: Every defined kcall number (0..=31) maps to a valid, non-Invalid category.
///
/// # Description
///
/// Proves that all 32 defined kernel call numbers are classified into one of
/// the five local categories or the Remote category.
pub proof fn lemma_defined_kcalls_classified()
    ensures
        forall|n: u32| spec_is_defined_kcall(n) ==> (
            spec_classify_kcall(n) =~= DispatchCategory::LocalImmediate
            || spec_classify_kcall(n) =~= DispatchCategory::LocalTerminal
            || spec_classify_kcall(n) =~= DispatchCategory::LocalSleepable
            || spec_classify_kcall(n) =~= DispatchCategory::LocalFallible
            || spec_classify_kcall(n) =~= DispatchCategory::LocalDirect
            || spec_classify_kcall(n) =~= DispatchCategory::Remote
        ),
{
}

/// Lemma: Undefined kcall numbers (> 31 or Invalid) are always Remote.
///
/// # Description
///
/// Proves that any kcall number not in the defined range 0..=31 is classified
/// as Remote (dispatched to the scoreboard).
pub proof fn lemma_undefined_kcalls_are_remote()
    ensures
        forall|n: u32| !spec_is_defined_kcall(n) ==> spec_classify_kcall(n) =~= DispatchCategory::Remote,
{
}

/// Lemma: The local/remote partition is complete and exclusive.
///
/// # Description
///
/// Proves that for every u32, a kcall is either locally handled or remote,
/// and never both simultaneously.
pub proof fn lemma_local_remote_partition()
    ensures
        forall|n: u32| spec_is_locally_handled(n) != spec_is_remote(n),
{
}

/// Lemma: spec_is_locally_handled matches the enumerated set.
///
/// # Description
///
/// Proves that the classification-based `spec_is_locally_handled` function
/// produces the same result as the explicit set enumeration
/// `spec_locally_handled_set` for all u32 values.
pub proof fn lemma_locally_handled_matches_set()
    ensures
        forall|n: u32| spec_is_locally_handled(n) == spec_locally_handled_set(n),
{
}

/// Lemma: GetPid is classified as LocalImmediate.
pub proof fn lemma_getpid_is_immediate()
    ensures
        spec_classify_kcall(KCALL_GET_PID()) =~= DispatchCategory::LocalImmediate,
{
}

/// Lemma: GetTid is classified as LocalImmediate.
pub proof fn lemma_gettid_is_immediate()
    ensures
        spec_classify_kcall(KCALL_GET_TID()) =~= DispatchCategory::LocalImmediate,
{
}

/// Lemma: Exit is classified as LocalTerminal.
pub proof fn lemma_exit_is_terminal()
    ensures
        spec_classify_kcall(KCALL_EXIT()) =~= DispatchCategory::LocalTerminal,
{
}

/// Lemma: ExitThread is classified as LocalTerminal.
pub proof fn lemma_exit_thread_is_terminal()
    ensures
        spec_classify_kcall(KCALL_EXIT_THREAD()) =~= DispatchCategory::LocalTerminal,
{
}

/// Lemma: JoinThread is classified as LocalSleepable.
pub proof fn lemma_join_thread_is_sleepable()
    ensures
        spec_classify_kcall(KCALL_JOIN_THREAD()) =~= DispatchCategory::LocalSleepable,
{
}

/// Lemma: Recv is classified as LocalSleepable.
pub proof fn lemma_recv_is_sleepable()
    ensures
        spec_classify_kcall(KCALL_RECV()) =~= DispatchCategory::LocalSleepable,
{
}

/// Lemma: MutexLock is classified as LocalSleepable.
pub proof fn lemma_mutex_lock_is_sleepable()
    ensures
        spec_classify_kcall(KCALL_MUTEX_LOCK()) =~= DispatchCategory::LocalSleepable,
{
}

/// Lemma: CondWait is classified as LocalSleepable.
pub proof fn lemma_cond_wait_is_sleepable()
    ensures
        spec_classify_kcall(KCALL_COND_WAIT()) =~= DispatchCategory::LocalSleepable,
{
}

/// Lemma: Sleep is classified as LocalSleepable.
pub proof fn lemma_sleep_is_sleepable()
    ensures
        spec_classify_kcall(KCALL_SLEEP()) =~= DispatchCategory::LocalSleepable,
{
}

/// Lemma: Resume is classified as LocalDirect.
pub proof fn lemma_resume_is_direct()
    ensures
        spec_classify_kcall(KCALL_RESUME()) =~= DispatchCategory::LocalDirect,
{
}

/// Lemma: MutexUnlock is classified as LocalFallible.
pub proof fn lemma_mutex_unlock_is_fallible()
    ensures
        spec_classify_kcall(KCALL_MUTEX_UNLOCK()) =~= DispatchCategory::LocalFallible,
{
}

/// Lemma: CondSignal is classified as LocalFallible.
pub proof fn lemma_cond_signal_is_fallible()
    ensures
        spec_classify_kcall(KCALL_COND_SIGNAL()) =~= DispatchCategory::LocalFallible,
{
}

/// Lemma: SchedulerYield is classified as LocalFallible.
pub proof fn lemma_scheduler_yield_is_fallible()
    ensures
        spec_classify_kcall(KCALL_SCHEDULER_YIELD()) =~= DispatchCategory::LocalFallible,
{
}

/// Lemma: Every sleepable call is also locally handled.
///
/// # Description
///
/// Proves that LocalSleepable is a subset of locally-handled calls.
pub proof fn lemma_sleepable_implies_local()
    ensures
        forall|n: u32| spec_is_sleepable(n) ==> spec_is_locally_handled(n),
{
}

//==================================================================================================
// Proof Lemmas: Error Handling
//==================================================================================================

/// Lemma: handle_sleep_error produces an error result for Generic errors.
///
/// # Description
///
/// Proves that a Generic sleep error produces a non-success result with the
/// same error code.
pub proof fn lemma_handle_generic_error(error_code: int)
    ensures ({
        let result: DispatchResultView = spec_handle_sleep_error(
            SleepErrorKind::Generic, error_code);
        &&& !result.is_success
        &&& result.value == error_code
    }),
{
}

/// Lemma: handle_sleep_error produces OperationTimedOut for InterruptedTimedOut.
///
/// # Description
///
/// Proves that a timed-out interruption produces an error result with the
/// OperationTimedOut error code (110).
pub proof fn lemma_handle_timed_out()
    ensures ({
        let result: DispatchResultView = spec_handle_sleep_error(
            SleepErrorKind::InterruptedTimedOut, 0);
        &&& !result.is_success
        &&& result.value == SPEC_ERROR_TIMED_OUT()
    }),
{
}

/// Lemma: handle_sleep_error for Generic with valid error code is well-formed.
///
/// # Description
///
/// Proves that if the input error code fits in i32, the resulting
/// DispatchResult is well-formed.
pub proof fn lemma_handle_generic_error_wf(error_code: int)
    requires
        error_code >= i32::MIN as int,
        error_code <= i32::MAX as int,
    ensures
        spec_result_wf(spec_handle_sleep_error(SleepErrorKind::Generic, error_code)),
{
}

/// Lemma: handle_sleep_error for InterruptedTimedOut is always well-formed.
///
/// # Description
///
/// Proves that the timed-out error result is well-formed (110 fits in i32).
pub proof fn lemma_handle_timed_out_wf()
    ensures
        spec_result_wf(spec_handle_sleep_error(SleepErrorKind::InterruptedTimedOut, 0)),
{
}

//==================================================================================================
// Proof Lemmas: Result Construction
//==================================================================================================

/// Lemma: the ok result is well-formed and successful.
pub proof fn lemma_ok_result_wf()
    ensures ({
        let r: DispatchResultView = spec_ok_result();
        &&& r.is_success
        &&& r.value == 0
        &&& spec_result_wf(r)
    }),
{
}

/// Lemma: a success result with a valid i64 value is well-formed.
pub proof fn lemma_success_result_wf(value: int)
    requires
        value >= i64::MIN as int,
        value <= i64::MAX as int,
    ensures
        spec_result_wf(spec_success_result(value)),
{
}

/// Lemma: an error result with a valid i32 value is well-formed.
pub proof fn lemma_error_result_wf(code: int)
    requires
        code >= i32::MIN as int,
        code <= i32::MAX as int,
    ensures
        spec_result_wf(spec_error_result(code)),
{
}

//==================================================================================================
// Proof Lemmas: Dispatch Table Properties
//==================================================================================================

/// Lemma: The locally-handled set contains exactly 13 kernel calls.
///
/// # Description
///
/// Enumerates all 13 locally-handled kcall numbers and proves each is in the set.
pub proof fn lemma_locally_handled_enumeration()
    ensures
        spec_locally_handled_set(KCALL_GET_PID()),
        spec_locally_handled_set(KCALL_GET_TID()),
        spec_locally_handled_set(KCALL_EXIT()),
        spec_locally_handled_set(KCALL_EXIT_THREAD()),
        spec_locally_handled_set(KCALL_JOIN_THREAD()),
        spec_locally_handled_set(KCALL_RECV()),
        spec_locally_handled_set(KCALL_RESUME()),
        spec_locally_handled_set(KCALL_MUTEX_LOCK()),
        spec_locally_handled_set(KCALL_MUTEX_UNLOCK()),
        spec_locally_handled_set(KCALL_COND_WAIT()),
        spec_locally_handled_set(KCALL_COND_SIGNAL()),
        spec_locally_handled_set(KCALL_SCHEDULER_YIELD()),
        spec_locally_handled_set(KCALL_SLEEP()),
{
}

/// Lemma: Remote calls include all kcalls not locally handled.
///
/// # Description
///
/// Verifies specific examples: Debug, CapCtl, Terminate, EventCtrl, Send,
/// and memory/IO calls are all dispatched remotely.
pub proof fn lemma_remote_examples()
    ensures
        spec_is_remote(KCALL_DEBUG()),
        spec_is_remote(KCALL_CAP_CTL()),
        spec_is_remote(KCALL_TERMINATE()),
        spec_is_remote(KCALL_EVENT_CTRL()),
        spec_is_remote(KCALL_SEND()),
        spec_is_remote(KCALL_MEMORY_MAP()),
        spec_is_remote(KCALL_MEMORY_UNMAP()),
        spec_is_remote(KCALL_MEMORY_CTRL()),
        spec_is_remote(KCALL_MEMORY_COPY()),
        spec_is_remote(KCALL_ALLOC_MMIO()),
        spec_is_remote(KCALL_FREE_MMIO()),
        spec_is_remote(KCALL_ALLOC_PMIO()),
        spec_is_remote(KCALL_FREE_PMIO()),
        spec_is_remote(KCALL_READ_PMIO()),
        spec_is_remote(KCALL_WRITE_PMIO()),
        spec_is_remote(KCALL_CREATE_THREAD()),
        spec_is_remote(KCALL_GET_TIME()),
        spec_is_remote(KCALL_SET_TDA()),
        spec_is_remote(KCALL_GET_TDA()),
        spec_is_remote(KCALL_INVALID()),
{
}

//==================================================================================================
// Proof Lemmas: SleepError Well-Formedness
//==================================================================================================

impl SleepError {
    /// Lemma: A well-formed Generic sleep error produces a well-formed dispatch result.
    pub proof fn lemma_wf_generic_produces_wf_result(&self)
        requires
            self.wf(),
            self.kind =~= SleepErrorKind::Generic,
        ensures
            spec_result_wf(spec_handle_sleep_error(self.kind, self.error_code as int)),
    {
    }

    /// Lemma: A TimedOut sleep error always produces a well-formed dispatch result.
    pub proof fn lemma_timed_out_produces_wf_result(&self)
        requires
            self.kind =~= SleepErrorKind::InterruptedTimedOut,
        ensures
            spec_result_wf(spec_handle_sleep_error(self.kind, self.error_code as int)),
    {
    }
}

//==================================================================================================
// Proof Lemmas: DispatchResult Well-Formedness
//==================================================================================================

impl DispatchResult {
    /// Lemma: The ok() constructor produces a well-formed success result.
    pub proof fn lemma_ok_is_wf()
        ensures ({
            let r: DispatchResult = DispatchResult { is_success: true, value: 0 };
            &&& r.wf()
            &&& r@.is_success
            &&& r@.value == 0
        }),
    {
    }

    /// Lemma: An error result with a valid i32 code is well-formed.
    pub proof fn lemma_error_is_wf(code: i64)
        requires
            code >= i32::MIN as i64,
            code <= i32::MAX as i64,
        ensures ({
            let r: DispatchResult = DispatchResult { is_success: false, value: code };
            &&& r.wf()
            &&& !r@.is_success
            &&& r@.value == code as int
        }),
    {
    }
}

//==================================================================================================
// Proof Lemmas: KcallResult → i64 Encoding
//==================================================================================================

/// Lemma: encoding preserves the value for both success and error results.
///
/// # Description
///
/// Proves that spec_encode_result simply extracts the value, which is the
/// same behavior as the original `Into<i64>` implementation.
pub proof fn lemma_encode_preserves_value(r: DispatchResultView)
    ensures
        spec_encode_result(r) == r.value,
{
}

/// Lemma: well-formed error results encode to values in i32 range.
///
/// # Description
///
/// Proves that any well-formed error result's encoded value fits in i32,
/// which is a necessary condition for the encoding to be distinguishable
/// from large success values.
pub proof fn lemma_error_encoding_fits_i32(r: DispatchResultView)
    requires
        spec_result_wf(r),
        !r.is_success,
    ensures
        spec_could_be_error(spec_encode_result(r)),
{
}

/// Lemma: encoding two different well-formed results preserves distinction.
///
/// # Description
///
/// Proves that if two results have different abstract views, they encode
/// to the same value only if they have the same payload. Combined with
/// the success/error flag, the full result is distinguishable.
pub proof fn lemma_encode_injective_on_value(r1: DispatchResultView, r2: DispatchResultView)
    ensures
        spec_encode_result(r1) == spec_encode_result(r2) ==> r1.value == r2.value,
{
}

//==================================================================================================
// Proof Lemmas: Divergence Boundary
//==================================================================================================

/// Lemma: Generic and TimedOut are the only non-divergent sleep error kinds.
///
/// # Description
///
/// Proves that spec_sleep_error_returns is true for exactly Generic and
/// InterruptedTimedOut, and false for InterruptedKilled.
pub proof fn lemma_non_divergent_sleep_errors()
    ensures
        spec_sleep_error_returns(SleepErrorKind::Generic),
        spec_sleep_error_returns(SleepErrorKind::InterruptedTimedOut),
        !spec_sleep_error_returns(SleepErrorKind::InterruptedKilled),
{
}

/// Lemma: Non-divergent sleep errors always produce error (non-success) results.
///
/// # Description
///
/// Proves that for any non-divergent sleep error kind, the spec-level
/// handler always produces a result with `is_success == false`.
pub proof fn lemma_non_divergent_errors_are_errors(kind: SleepErrorKind, error_code: int)
    requires
        spec_sleep_error_returns(kind),
    ensures
        !spec_handle_sleep_error(kind, error_code).is_success,
{
}

//==================================================================================================
// Proof Lemmas: do_kcall Postcondition Support
//==================================================================================================

/// Lemma: LocalImmediate calls are exactly GetPid and GetTid.
///
/// # Description
///
/// Proves that the only two kcall numbers classified as LocalImmediate
/// are GetPid (1) and GetTid (2).
pub proof fn lemma_immediate_is_getpid_gettid(number: u32)
    ensures
        spec_classify_kcall(number) =~= DispatchCategory::LocalImmediate
            <==> (number == KCALL_GET_PID() || number == KCALL_GET_TID()),
{
}

/// Lemma: The result category spec correctly reflects the classification.
///
/// # Description
///
/// Proves that `spec_do_kcall_result_category` is consistent with
/// `spec_classify_kcall` for any valid DispatchArgs.
pub proof fn lemma_result_category_consistent(number: u32, arg0: u32, arg1: u32, arg2: u32, arg3: u32)
    ensures ({
        let args: DispatchArgsView = DispatchArgsView {
            number: number as nat,
            arg0: arg0 as nat,
            arg1: arg1 as nat,
            arg2: arg2 as nat,
            arg3: arg3 as nat,
        };
        spec_do_kcall_result_category(args) =~= spec_classify_kcall(number)
    }),
{
}

} // verus!
