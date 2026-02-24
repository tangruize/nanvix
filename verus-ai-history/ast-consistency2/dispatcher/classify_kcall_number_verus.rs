pub fn classify_kcall_number(number: u32) -> (result: DispatchCategory)
    ensures
        result =~= spec_classify_kcall(number),
{
    if number == 1 || number == 2 {
        // GetPid (1), GetTid (2).
        DispatchCategory::LocalImmediate
    } else if number == 3 || number == 22 {
        // Exit (3), ExitThread (22).
        DispatchCategory::LocalTerminal
    } else if number == 23 || number == 9 || number == 24 || number == 27 || number == 29 {
        // JoinThread (23), Recv (9), MutexLock (24), CondWait (27), Sleep (29).
        DispatchCategory::LocalSleepable
    } else if number == 25 || number == 26 || number == 20 {
        // MutexUnlock (25), CondSignal (26), SchedulerYield (20).
        DispatchCategory::LocalFallible
    } else if number == 5 {
        // Resume (5).
        DispatchCategory::LocalDirect
    } else {
        // Everything else: dispatched to scoreboard.
        DispatchCategory::Remote
    }
}
