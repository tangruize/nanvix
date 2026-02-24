pub fn join_thread(
    pid: u32,
    arg0: u32,
    arg1: u32,
) -> (result: JoinThreadKcallResultModel)
    requires
        spec_is_user_process(pid as nat),
        spec_pm_initialized(),
        spec_mm_initialized(),
        spec_no_resources_held(),
    ensures
        // The result is always success or error.
        spec_is_success(result.spec_view()) || spec_is_error(result.spec_view()),
        // Success and error are mutually exclusive.
        !(spec_is_success(result.spec_view()) && spec_is_error(result.spec_view())),
        // Success implies ExitStatus::ok() (value 0).
        spec_is_success(result.spec_view())
            ==> result.spec_view() == (JoinThreadResultView::Success {
                    exit_status: EXIT_STATUS_OK()
                }),
{
    // Delegate to the fully verified model, discarding ghost witnesses.
    let ret: (JoinThreadKcallResultModel, Ghost<TidParseOutcomeView>, Ghost<JoinThreadOutcomeView>, Ghost<CopyToUserOutcomeView>) =
        join_thread_model(pid, arg0, arg1);
    ret.0
}
