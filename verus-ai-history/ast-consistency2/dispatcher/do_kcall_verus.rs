pub fn do_kcall(args: DispatchArgs) -> (result: DispatchResult)
    ensures
        result.wf(),
        // Terminal calls (Exit, ExitThread) always produce error results.
        spec_classify_kcall(args.number) =~= DispatchCategory::LocalTerminal ==> !result@.is_success,
        // General category constraint.
        spec_dispatch_result_constrained(spec_classify_kcall(args.number), result@),
        // GetPid/GetTid: if success, value is non-negative.
        (args@.number == 1 || args@.number == 2)
            && result@.is_success ==> result@.value >= 0,
        // ok()-returning calls: success value is 0.
        (args@.number == 9 || args@.number == 24
            || args@.number == 27 || args@.number == 29
            || args@.number == 25 || args@.number == 20)
            && result@.is_success ==> result@.value == 0,
        // CondSignal: success value >= 0 (count of woken threads).
        args@.number == 26 && result@.is_success ==> result@.value >= 0,
        // JoinThread: success value >= 0.
        args@.number == 23 && result@.is_success ==> result@.value >= 0,
{
    do_kcall_context(args)
}
