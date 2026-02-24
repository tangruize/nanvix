fn remote_dispatch_verified(number: u32, pid: i64, tid: i64, arg0: u32, arg1: u32, arg2: u32, arg3: u32) -> (result: DispatchResult)
    ensures
        result.wf(),
{
    let sb_outcome: FallibleOutcome = scoreboard_get_mut();
    if !sb_outcome.succeeded {
        DispatchResult::error(sb_outcome.error_code)
    } else {
        let dispatch_outcome: ScoreboardDispatchOutcome =
            scoreboard_dispatch_call(number, pid, tid, arg0, arg1, arg2, arg3);
        if dispatch_outcome.succeeded {
            if dispatch_outcome.result_is_success {
                DispatchResult::success(dispatch_outcome.result_value)
            } else {
                DispatchResult::error(dispatch_outcome.result_value as i32)
            }
        } else {
            match dispatch_outcome.sleep_error_kind {
                SleepErrorKind::InterruptedKilled => {
                    // SOUNDNESS NOTE: handle_sleep_error_killed calls
                    // pm_exit_interrupted() then diverge_after_exit().
                    // If the original Killed path changes, update T4a/T4b.
                    handle_sleep_error_killed()
                },
                SleepErrorKind::Generic => {
                    handle_sleep_error(SleepError {
                        kind: SleepErrorKind::Generic,
                        error_code: dispatch_outcome.sleep_error_code,
                    })
                },
                SleepErrorKind::InterruptedTimedOut => {
                    handle_sleep_error(SleepError {
                        kind: SleepErrorKind::InterruptedTimedOut,
                        error_code: dispatch_outcome.sleep_error_code,
                    })
                },
            }
        }
    }
}
