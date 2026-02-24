pub fn do_kcall_context(args: DispatchArgs) -> (result: DispatchResult)
    ensures
        result.wf(),
        // Terminal calls always return error, even if pid/tid retrieval fails.
        spec_classify_kcall(args.number) =~= DispatchCategory::LocalTerminal
            ==> !result@.is_success,
        // GetPid/GetTid: if success, value is non-negative (pids/tids >= 0).
        (args@.number == 1 || args@.number == 2)
            && result@.is_success ==> result@.value >= 0,
        // ok()-returning calls: success value is 0.
        (args@.number == 9 || args@.number == 24
            || args@.number == 27 || args@.number == 29
            || args@.number == 25 || args@.number == 20)
            && result@.is_success ==> result@.value == 0,
        // CondSignal: success value >= 0 (count of woken threads).
        args@.number == 26 && result@.is_success ==> result@.value >= 0,
        // JoinThread: success value >= 0 (u32 exit status).
        args@.number == 23 && result@.is_success ==> result@.value >= 0,
{
    let pid_outcome: FallibleOutcome = pm_get_pid();
    if !pid_outcome.succeeded {
        DispatchResult::error(pid_outcome.error_code)
    } else {
        let tid_outcome: FallibleOutcome = pm_get_tid();
        if !tid_outcome.succeeded {
            DispatchResult::error(tid_outcome.error_code)
        } else {
            do_kcall_dispatch(pid_outcome.value, tid_outcome.value, args)
        }
    }
}
