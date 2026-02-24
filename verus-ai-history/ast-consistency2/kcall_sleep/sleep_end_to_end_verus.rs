pub fn sleep_end_to_end(seconds: u64, nanoseconds: u32) -> (ret: (SleepResultModel, Ghost<PmSleepResultView>, Ghost<SystemTimeView>))
    requires
        // ABI constraint: seconds originates from a 32-bit usize on x86-32.
        seconds as nat <= USIZE_MAX_X86_32(),
        // nanoseconds is already u32, which matches 32-bit usize identity cast.
        seconds as nat + nanoseconds as nat / NANOS_PER_SEC() <= u64::MAX as nat,
    ensures
        // TimedOut never appears in the output (core invariant from the 3-arm match).
        !matches!(ret.0, SleepResultModel::TimedOut),
        // The ghost now value is well-formed (from clock module guarantee).
        spec_system_time_wf(ret.2@),
        // Overflow path: checked_add fails → GenericError(InvalidArgument).
        !spec_sleep_success_condition(ret.2@, seconds as nat, nanoseconds as nat)
            ==> ret.0.spec_classified_view() == (SleepResultView::GenericError {
                    error_code: ERROR_CODE_INVALID_ARGUMENT()
                }),
        !spec_sleep_success_condition(ret.2@, seconds as nat, nanoseconds as nat)
            ==> matches!(ret.0, SleepResultModel::GenericError { .. }),
        // Success path: result tied to spec_sleep_result via the ghost PM outcome.
        spec_sleep_success_condition(ret.2@, seconds as nat, nanoseconds as nat)
            ==> ret.0.spec_classified_view() == spec_sleep_result(
                    ret.2@, seconds as nat, nanoseconds as nat, ret.1@),
        // Success path: only classified variants (no TimedOut).
        spec_sleep_success_condition(ret.2@, seconds as nat, nanoseconds as nat)
            ==> matches!(ret.0, SleepResultModel::Ok | SleepResultModel::Killed
                    | SleepResultModel::GenericError { .. }),
{
    // Step 1: Get the current time (Trust Boundary T1).
    let now: SystemTimeModel = clock_now();

    // Capture the clock value as a ghost for postcondition exposure.
    let ghost now_view: SystemTimeView = now.spec_view();

    // Delegate to the verified model.
    let result: (SleepResultModel, Ghost<PmSleepResultView>) = sleep_model(&now, seconds, nanoseconds);
    (result.0, result.1, Ghost(now_view))
}
