fn convert_sleepable(outcome: SleepableOutcome) -> (result: DispatchResult)
    requires
        outcome.wf(),
    ensures
        result.wf(),
        outcome.succeeded ==> (result@.is_success && result@.value == outcome.value as int),
        !outcome.succeeded ==> !result@.is_success,
{
    if outcome.succeeded {
        DispatchResult::success(outcome.value)
    } else {
        match outcome.sleep_error_kind {
            SleepErrorKind::InterruptedKilled => {
                // SOUNDNESS NOTE: handle_sleep_error_killed calls
                // pm_exit_interrupted() then diverge_after_exit().
                // If the original Killed path changes, update T4a/T4b.
                handle_sleep_error_killed()
            },
            SleepErrorKind::Generic => {
                handle_sleep_error(SleepError {
                    kind: SleepErrorKind::Generic,
                    error_code: outcome.sleep_error_code,
                })
            },
            SleepErrorKind::InterruptedTimedOut => {
                handle_sleep_error(SleepError {
                    kind: SleepErrorKind::InterruptedTimedOut,
                    error_code: outcome.sleep_error_code,
                })
            },
        }
    }
}
