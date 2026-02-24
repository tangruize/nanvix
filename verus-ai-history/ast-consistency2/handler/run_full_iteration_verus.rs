pub fn run_full_iteration(stdio_enabled: bool) -> (result: IterationResult)
    ensures
        // Yield iff no work was done.
        result.should_yield == (!result.work_state.kcall_handled
            && !result.work_state.message_received && !result.work_state.harvested_process),
        // Termination implies INITD pid.
        result.should_terminate ==> result.initd_pid == 1u32,
        // INITD termination does NOT set harvested_process.
        result.should_terminate ==> !result.work_state.harvested_process,
        // Concrete harvest fields link to spec termination.
        result.should_terminate == spec_should_terminate(spec_harvest_to_outcome(
            result.harvest_found, result.harvest_error, result.harvest_pid as nat, result.harvest_is_initd)),
        // Non-stdio builds never receive messages.
        !stdio_enabled ==> !result.work_state.message_received,
{
    // Phase 1: Poll scoreboard.
    let poll: ScoreBoardPollResult = poll_scoreboard_full();

    // Phase 2-4: Run iteration (dispatch, messages, harvest).
    let result: IterationResult = run_iteration(&poll, stdio_enabled);

    // Phase 5: Yield CPU if no work was done and loop is not terminating.
    // The `!result.should_terminate` guard is necessary because our model returns
    // a result struct rather than using `break` for INITD termination. In the
    // original code (lines 165-167), INITD termination causes `break status` during
    // the harvest phase, exiting the loop before reaching the yield check at
    // line 187. In our sequential model, `run_full_iteration` always returns, so
    // when INITD terminates with no other work done (`should_yield && should_terminate`),
    // we must suppress the yield to match the original's control flow where the
    // yield would never be reached.
    if result.should_yield && !result.should_terminate {
        yield_cpu();
    }

    result
}
