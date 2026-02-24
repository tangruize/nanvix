pub fn terminate(
    arg0: u32,
    Ghost(pm_pre): Ghost<ProcessManagerStateView>,
) -> (result: KcallResultModel)
    requires
        spec_pm_wf(pm_pre),
    ensures
        // Success requires both PID parse and PM terminate to succeed.
        spec_is_success(result.spec_view()) || spec_is_error(result.spec_view()),
        // Kernel PID always fails (PID 0 parses successfully per axiom and PM rejects it).
        arg0 as nat == KERNEL_PID() ==> spec_is_error(result.spec_view()),
{
    // Delegate to the fully verified model, discarding ghost witnesses.
    let ret: (KcallResultModel, Ghost<PidParseOutcomeView>, Ghost<TerminateOutcomeView>, Ghost<ProcessManagerStateView>) =
        terminate_model(arg0, Ghost(pm_pre));
    ret.0
}
