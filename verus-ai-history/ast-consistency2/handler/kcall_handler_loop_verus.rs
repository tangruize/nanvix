pub fn kcall_handler_loop(fuel: u32, stdio_enabled: bool) -> (result: (LoopResult, Ghost<Seq<HarvestOutcome>>))
    ensures
        // The loop invariant holds for the final history.
        spec_loop_invariant(result.1@),
        // Fuel exhaustion: all iterations ran, none triggered termination.
        !result.0.terminated ==> result.1@.len() == fuel as int,
        // Early exit: termination occurred before fuel ran out.
        result.0.terminated ==> result.1@.len() < fuel as int,
        // On termination, it was INITD (pid == 1) that triggered exit.
        result.0.terminated ==> result.0.termination_pid == 1u32,
{
    let mut history: Ghost<Seq<HarvestOutcome>> = kcall_handler_init();
    let mut i: u32 = 0;
    let mut terminated: bool = false;
    let mut exit_status: u32 = 0;
    let mut termination_pid: u32 = 0;

    while i < fuel && !terminated
        invariant
            spec_loop_invariant(history@),
            i <= fuel,
            // History tracks iteration count when the loop is still running.
            !terminated ==> history@.len() == i as int,
            // Termination happened before fuel was exhausted.
            terminated ==> history@.len() < fuel as int,
            // Termination was triggered by INITD.
            terminated ==> termination_pid == 1u32,
        decreases fuel - i,
    {
        let (step, new_hist): (LifecycleStepResult, Ghost<Seq<HarvestOutcome>>) =
            kcall_handler_lifecycle_step(history, stdio_enabled);
        if step.terminated {
            terminated = true;
            exit_status = step.exit_status;
            termination_pid = step.termination_pid;
        }
        // Always update history (unchanged on termination, extended otherwise).
        history = new_hist;
        i = i + 1;
    }

    (LoopResult {
        terminated,
        exit_status,
        termination_pid,
    }, history)
}
