pub fn lock_mutex(pid: u32, tid: u32, mutex_addr: u32, timeout_s: u32, timeout_ns: u32) -> (result: LockMutexResultModel)
    requires
        // ABI constraint: inputs originate from 32-bit usize on x86-32.
        timeout_s as nat <= USIZE_MAX_X86_32(),
        timeout_ns as nat <= USIZE_MAX_X86_32(),
    ensures
        // Invalid timeout always produces InvalidTimeoutError.
        !spec_timeout_parsed_ok(timeout_s as nat, timeout_ns as nat) ==>
            spec_is_timeout_error(result.spec_view()),
        // TimedOut impossible with infinite timeout.
        !spec_is_finite_timeout(timeout_s as nat, timeout_ns as nat) ==>
            !matches!(result, LockMutexResultModel::LockTimedOut),
{
    // pid and tid are intentionally unused — they affect only trace logging
    // in the original function. See lemma_result_independent_of_pid_tid.
    let ret: (
        LockMutexResultModel,
        Ghost<GetMutexOutcomeView>,
        Ghost<LockOutcomeView>,
        Ghost<PutGuardOutcomeView>,
        Ghost<Option<TimeoutView>>,
    ) = lock_mutex_model(mutex_addr, timeout_s, timeout_ns);
    ret.0
}
