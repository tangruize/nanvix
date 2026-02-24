pub fn handle_sleep_error(sleep_error: SleepError) -> (result: DispatchResult)
    requires
        sleep_error.wf(),
        spec_sleep_error_returns(sleep_error.spec_kind()),
    ensures
        result@ == spec_handle_sleep_error(sleep_error.spec_kind(), sleep_error.spec_error_code()),
        !result@.is_success,
        result.wf(),
{
    match sleep_error.kind {
        SleepErrorKind::Generic => {
            DispatchResult::error(sleep_error.error_code as i32)
        },
        SleepErrorKind::InterruptedTimedOut => {
            DispatchResult::error(116i32)
        },
        SleepErrorKind::InterruptedKilled => {
            // Unreachable: excluded by precondition spec_sleep_error_returns.
            DispatchResult::error(-1i32)
        },
    }
}
