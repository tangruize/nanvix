pub fn sleep_model(now: &SystemTimeModel, seconds: u64, nanoseconds: u32) -> (ret: (SleepResultModel, Ghost<PmSleepResultView>))
    requires
        now.spec_wf(),
        // ABI constraint: seconds originates from a 32-bit usize on x86-32.
        seconds as nat <= USIZE_MAX_X86_32(),
        // nanoseconds is already u32, which matches 32-bit usize identity cast.
        // Duration normalization must not overflow.
        seconds as nat + nanoseconds as nat / NANOS_PER_SEC() <= u64::MAX as nat,
    ensures
        // Overflow path: checked_add fails → GenericError(InvalidArgument).
        !spec_sleep_success_condition(now.spec_view(), seconds as nat, nanoseconds as nat)
            ==> ret.0.spec_classified_view() == (SleepResultView::GenericError {
                    error_code: ERROR_CODE_INVALID_ARGUMENT()
                }),
        // Overflow path: result is always GenericError.
        !spec_sleep_success_condition(now.spec_view(), seconds as nat, nanoseconds as nat)
            ==> matches!(ret.0, SleepResultModel::GenericError { .. }),
        // Non-tautological: the ghost captures the original PM result, and the
        // classified view equals spec_sleep_result applied to that PM result.
        // This ties the exec result to the actual PM outcome, not just to the
        // post-classification spec_pm_view (which would be tautological).
        spec_sleep_success_condition(now.spec_view(), seconds as nat, nanoseconds as nat)
            ==> ret.0.spec_classified_view() == spec_sleep_result(
                    now.spec_view(), seconds as nat, nanoseconds as nat, ret.1@),
        // Success path: result is always a classified PM result (Ok, Killed, or GenericError).
        // TimedOut has been folded into Ok by the 3-arm match.
        spec_sleep_success_condition(now.spec_view(), seconds as nat, nanoseconds as nat)
            ==> matches!(ret.0, SleepResultModel::Ok | SleepResultModel::Killed
                    | SleepResultModel::GenericError { .. }),
        // Success path: TimedOut never appears in the output (folded into Ok).
        spec_sleep_success_condition(now.spec_view(), seconds as nat, nanoseconds as nat)
            ==> !matches!(ret.0, SleepResultModel::TimedOut),
{
    // Step 1: Construct the timeout Duration.
    let timeout: DurationModel = duration_new(seconds, nanoseconds);

    // Step 2: Compute the alarm time.
    let alarm_opt: Option<SystemTimeModel> = checked_add_duration(now, &timeout);

    match alarm_opt {
        Some(alarm) => {
            // Step 3: Call ProcessManager::sleep(Some(alarm)).
            let pm_result: SleepResultModel = process_manager_sleep(&alarm);

            // Capture the original PM result as a ghost before classification.
            let ghost orig_pm_view: PmSleepResultView = pm_result.spec_pm_view();

            // Step 4: Classify the result via the 3-arm match.
            // Original: Ok(()) => Ok(()), Interrupted(TimedOut) => Ok(()),
            //           Err(error) => Err(error)
            let classified: SleepResultModel = classify_pm_result(pm_result);
            (classified, Ghost(orig_pm_view))
        },
        None => {
            // Overflow → InvalidArgument.
            proof {
                assert(22i32 as int == ERROR_CODE_INVALID_ARGUMENT());
            }
            // Ghost PM value is arbitrary on the overflow path (ensures are vacuously true).
            (SleepResultModel::GenericError { error_code: 22i32 }, Ghost(PmSleepResultView::PmOk))
        },
    }
}
