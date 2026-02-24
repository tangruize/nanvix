fn do_kcall_dispatch(pid: i64, tid: i64, args: DispatchArgs) -> (result: DispatchResult)
    requires
        pid >= 0,
        tid >= 0,
    ensures
        result.wf(),
        // GetPid returns the pid value.
        args@.number == 1 ==> (result@.is_success && result@.value == pid as int),
        // GetTid returns the tid value.
        args@.number == 2 ==> (result@.is_success && result@.value == tid as int),
        // Terminal calls always return error.
        spec_classify_kcall(args.number) =~= DispatchCategory::LocalTerminal
            ==> !result@.is_success,
        // ok()-returning sleepable calls: success value is 0.
        (args@.number == 9 || args@.number == 24
            || args@.number == 27 || args@.number == 29)
            && result@.is_success ==> result@.value == 0,
        // JoinThread: success value >= 0 (u32 exit status).
        args@.number == 23 && result@.is_success ==> result@.value >= 0,
        // ok()-returning fallible calls: success value is 0.
        (args@.number == 25 || args@.number == 20)
            && result@.is_success ==> result@.value == 0,
        // CondSignal: success value >= 0 (count of woken threads).
        args@.number == 26 && result@.is_success ==> result@.value >= 0,
        // Sleepable/fallible error paths produce well-formed error results.
        (args@.number == 9 || args@.number == 23 || args@.number == 24
            || args@.number == 27 || args@.number == 29
            || args@.number == 25 || args@.number == 26
            || args@.number == 20)
            && !result@.is_success ==> (result@.value >= i32::MIN as int
                                        && result@.value <= i32::MAX as int),
{
    let number: u32 = args.number;
    if number == 1u32 {
        // GetPid: return pid directly.
        DispatchResult::success(pid)
    } else if number == 2u32 {
        // GetTid: return tid directly.
        DispatchResult::success(tid)
    } else if number == 3u32 {
        // Exit: always returns error (process terminates).
        pm_exit(args.arg0)
    } else if number == 22u32 {
        // ExitThread: always returns error (thread terminates).
        pm_exit_thread(args.arg0)
    } else if number == 23u32 {
        // JoinThread: sleepable.
        convert_sleepable(pm_join_thread(pid, args.arg0, args.arg1))
    } else if number == 9u32 {
        // Recv: sleepable.
        convert_sleepable(ipc_recv(tid, pid, args.arg0))
    } else if number == 5u32 {
        // Resume: direct result from event subsystem.
        event_resume(args.arg0)
    } else if number == 24u32 {
        // MutexLock: sleepable.
        convert_sleepable(pm_lock_mutex(pid, tid, args.arg0, args.arg1, args.arg2))
    } else if number == 25u32 {
        // MutexUnlock: fallible.
        convert_fallible(pm_unlock_mutex(pid, tid, args.arg0))
    } else if number == 27u32 {
        // CondWait: sleepable.
        convert_sleepable(pm_wait_cond(pid, tid, args.arg0, args.arg1, args.arg2, args.arg3))
    } else if number == 26u32 {
        // CondSignal: fallible. arg1 != 0 is the broadcast flag.
        convert_fallible(pm_signal_cond(pid, tid, args.arg0, args.arg1 != 0))
    } else if number == 20u32 {
        // SchedulerYield: fallible.
        convert_fallible(pm_giveup())
    } else if number == 29u32 {
        // Sleep: sleepable.
        convert_sleepable(pm_sleep(args.arg0, args.arg1))
    } else {
        // Remote: dispatched to scoreboard (verified routing).
        remote_dispatch_verified(args.number, pid, tid, args.arg0, args.arg1, args.arg2, args.arg3)
    }
}
