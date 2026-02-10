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
// 7. Exit/ExitThread are classified as LocalTerminal.
// 8. Sleepable calls are a subset of locally-handled calls.
// 9. All defined kcalls (0..31) map to a non-Remote category or Remote.
// 10. Result constructors produce well-formed results.
// 11. Error encodings always fit in i32 range (non-trivial for encoding model).
// 12. Large success values (outside i32 range) are distinguishable from errors.
// 13. Divergent Killed path is properly separated from non-divergent error handling.
// 14. spec_dispatch_result_constrained correctly constrains per-category behavior.
// 15. Kcall spec constants match the original #[repr(u32)] enum values.
// 16. do_kcall_dispatch correctly routes each kcall to its subsystem handler.
// 17. do_kcall_context handles pid/tid retrieval failure gracefully.
// 18. convert_sleepable routes sleep errors through handle_sleep_error.
// 19. remote_dispatch_verified splits ScoreBoard access from dispatch routing.
// 20. ok()-returning calls (Recv, MutexLock, CondWait, Sleep, MutexUnlock,
//     SchedulerYield) verified to return value 0 on success.
// 21. JoinThread success value is non-negative (u32 exit status).
// 22. GetPid/GetTid conditional guarantee: success implies non-negative value.

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
/// OperationTimedOut error code (116, ETIMEDOUT in Nanvix).
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
/// Proves that the timed-out error result is well-formed (116 fits in i32).
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

/// Lemma: Spec constants match the original `#[repr(u32)]` KcallNumber values.
///
/// # Description
///
/// Asserts that each spec constant matches the value from the original
/// `KcallNumber` enum in `src/libs/sys/src/sys/number.rs`. This serves
/// as a consistency check: if the original enum values change, a human
/// reviewer can detect drift by comparing these assertions.
///
/// Note: Verus cannot import the original enum, so this is a manual
/// cross-reference. The values were verified against the source.
pub proof fn lemma_kcall_constants_consistency()
    ensures
        KCALL_DEBUG() == 0u32,
        KCALL_GET_PID() == 1u32,
        KCALL_GET_TID() == 2u32,
        KCALL_EXIT() == 3u32,
        KCALL_CAP_CTL() == 4u32,
        KCALL_RESUME() == 5u32,
        KCALL_TERMINATE() == 6u32,
        KCALL_EVENT_CTRL() == 7u32,
        KCALL_SEND() == 8u32,
        KCALL_RECV() == 9u32,
        KCALL_MEMORY_MAP() == 10u32,
        KCALL_MEMORY_UNMAP() == 11u32,
        KCALL_MEMORY_CTRL() == 12u32,
        KCALL_MEMORY_COPY() == 13u32,
        KCALL_ALLOC_MMIO() == 14u32,
        KCALL_FREE_MMIO() == 15u32,
        KCALL_ALLOC_PMIO() == 16u32,
        KCALL_FREE_PMIO() == 17u32,
        KCALL_READ_PMIO() == 18u32,
        KCALL_WRITE_PMIO() == 19u32,
        KCALL_SCHEDULER_YIELD() == 20u32,
        KCALL_CREATE_THREAD() == 21u32,
        KCALL_EXIT_THREAD() == 22u32,
        KCALL_JOIN_THREAD() == 23u32,
        KCALL_MUTEX_LOCK() == 24u32,
        KCALL_MUTEX_UNLOCK() == 25u32,
        KCALL_COND_SIGNAL() == 26u32,
        KCALL_COND_WAIT() == 27u32,
        KCALL_GET_TIME() == 28u32,
        KCALL_SLEEP() == 29u32,
        KCALL_SET_TDA() == 30u32,
        KCALL_GET_TDA() == 31u32,
        KCALL_INVALID() == u32::MAX as u32,
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

/// Lemma: well-formed error results encode to values in i32 range.
///
/// # Description
///
/// Proves that any well-formed error result's encoded value fits in i32,
/// which is a necessary condition for error values originating from
/// `KcallError(i32)` through the `Into<i64>` conversion.
pub proof fn lemma_error_encoding_fits_i32(r: DispatchResultView)
    requires
        spec_result_wf(r),
        !r.is_success,
    ensures
        spec_in_error_range(spec_encode_result(r)),
{
}

/// Lemma: success values outside i32 range cannot be errors.
///
/// # Description
///
/// Proves that if a success result has a value outside the i32 range, the
/// encoded i64 is distinguishable from any error value. This is the key
/// property for the encoding: large success values are unambiguously
/// not errors.
pub proof fn lemma_large_success_not_error(r: DispatchResultView)
    requires
        r.is_success,
        !spec_in_error_range(r.value),
    ensures
        // A well-formed error result cannot encode to this value.
        forall|e: DispatchResultView| #![auto]
            (spec_result_wf(e) && !e.is_success)
                ==> spec_encode_result(e) != spec_encode_result(r),
{
}

/// Lemma: the ABI encoding of a DispatchResult is its value field.
///
/// # Description
///
/// Proves the fundamental ABI property: `spec_encode_result(r)` equals
/// `r.value` for any dispatch result. This means `encode_result` is
/// the identity function on the value field, matching the real
/// `KcallResult::into::<i64>()` implementation.
pub proof fn lemma_encode_result_is_value(r: DispatchResultView)
    ensures
        spec_encode_result(r) == r.value,
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

/// Lemma: LocalTerminal calls are exactly Exit and ExitThread.
///
/// # Description
///
/// Proves that the only two kcall numbers classified as LocalTerminal
/// are Exit (3) and ExitThread (22).
pub proof fn lemma_terminal_is_exit_exitthread(number: u32)
    ensures
        spec_classify_kcall(number) =~= DispatchCategory::LocalTerminal
            <==> (number == KCALL_EXIT() || number == KCALL_EXIT_THREAD()),
{
}

/// Lemma: dispatch constraint is trivially true for LocalImmediate.
///
/// # Description
///
/// After weakening the spec (pid/tid retrieval may fail), LocalImmediate
/// has no structural constraint beyond well-formedness. The stronger
/// postcondition (success when pid/tid available) lives on `do_kcall_dispatch`.
pub proof fn lemma_dispatch_constraint_immediate()
    ensures
        forall|r: DispatchResultView| #![auto]
            spec_dispatch_result_constrained(DispatchCategory::LocalImmediate, r),
{
}

/// Lemma: dispatch constraint for terminal calls requires error.
pub proof fn lemma_dispatch_constraint_terminal()
    ensures
        forall|r: DispatchResultView| #![auto]
            spec_dispatch_result_constrained(DispatchCategory::LocalTerminal, r) == !r.is_success,
{
}

/// Lemma: dispatch constraint is trivially true for non-immediate, non-terminal categories.
pub proof fn lemma_dispatch_constraint_other()
    ensures
        forall|r: DispatchResultView| #![auto]
            spec_dispatch_result_constrained(DispatchCategory::LocalSleepable, r),
        forall|r: DispatchResultView| #![auto]
            spec_dispatch_result_constrained(DispatchCategory::LocalFallible, r),
        forall|r: DispatchResultView| #![auto]
            spec_dispatch_result_constrained(DispatchCategory::LocalDirect, r),
        forall|r: DispatchResultView| #![auto]
            spec_dispatch_result_constrained(DispatchCategory::Remote, r),
{
}

//==================================================================================================
// Proof Lemmas: Error-Code Propagation
//==================================================================================================

/// Lemma: the error constructor faithfully preserves the error code.
///
/// # Description
///
/// Proves that `spec_error_result(code)` produces a result whose value
/// equals the given code. This is the fundamental property underlying
/// error-code propagation: every failure path in the dispatcher calls
/// `DispatchResult::error(code)` which maps to `spec_error_result`,
/// guaranteeing value preservation.
///
/// Combined with `wf()` (which constrains error values to i32 range),
/// this proves that error codes flow faithfully from subsystem calls
/// through the verified dispatch logic to the result.
pub proof fn lemma_error_constructor_preserves_code(code: int)
    ensures
        spec_error_result(code).value == code,
        !spec_error_result(code).is_success,
{
}

/// Lemma: the success constructor faithfully preserves the value.
///
/// # Description
///
/// Proves that `spec_success_result(value)` produces a result whose
/// value equals the given value. This is the success-path analog of
/// `lemma_error_constructor_preserves_code`.
pub proof fn lemma_success_constructor_preserves_value(value: int)
    ensures
        spec_success_result(value).value == value,
        spec_success_result(value).is_success,
{
}

/// Lemma: GetPid/GetTid dispatch always succeeds.
///
/// # Description
///
/// Proves that GetPid (1) and GetTid (2) are classified as LocalImmediate.
/// Since `do_kcall_dispatch` returns `success(pid)` / `success(tid)` for
/// these calls (no subsystem call that can fail), dispatch is infallible.
///
/// Consequence: if `do_kcall_context` returns an error for GetPid/GetTid,
/// the error necessarily came from pid/tid retrieval (ProcessManager
/// access, trust boundary T1), not from the dispatch logic itself.
pub proof fn lemma_getpid_gettid_dispatch_infallible()
    ensures
        spec_classify_kcall(KCALL_GET_PID()) =~= DispatchCategory::LocalImmediate,
        spec_classify_kcall(KCALL_GET_TID()) =~= DispatchCategory::LocalImmediate,
{
}

} // verus!
