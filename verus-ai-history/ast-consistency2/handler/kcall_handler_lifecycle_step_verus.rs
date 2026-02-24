pub fn kcall_handler_lifecycle_step(
    history: Ghost<Seq<HarvestOutcome>>,
    stdio_enabled: bool,
) -> (result: (LifecycleStepResult, Ghost<Seq<HarvestOutcome>>))
    requires
        spec_loop_invariant(history@),
    ensures
        // The invariant is always preserved.
        spec_loop_invariant(result.1@),
        // On continuation, history grows by one.
        !result.0.terminated ==> result.1@.len() == history@.len() + 1,
        // On termination, history is unchanged.
        result.0.terminated ==> result.1@.len() == history@.len(),
        // On termination, the exit was triggered by INITD (pid == 1).
        result.0.terminated ==> result.0.termination_pid == 1u32,
{
    let iter_result: IterationResult = run_full_iteration(stdio_enabled);

    if iter_result.should_terminate {
        // INITD terminated: drain remaining zombies and exit.
        drain_remaining_zombies();
        (LifecycleStepResult {
            terminated: true,
            exit_status: iter_result.exit_status,
            termination_pid: iter_result.initd_pid,
        }, Ghost(history@))
    } else {
        // Loop continues: extend history with the REAL harvest outcome.
        let ghost outcome: HarvestOutcome = spec_harvest_to_outcome(
            iter_result.harvest_found,
            iter_result.harvest_error,
            iter_result.harvest_pid as nat,
            iter_result.harvest_is_initd,
        );
        proof {
            // The ensures on run_full_iteration gives us:
            //   iter_result.should_terminate == spec_should_terminate(outcome)
            // Since !iter_result.should_terminate, we have !spec_should_terminate(outcome),
            // which is spec_loop_continues(outcome).
            assert(spec_loop_continues(outcome));
            lemma_loop_invariant_inductive(history@, outcome);
        }
        (LifecycleStepResult {
            terminated: false,
            exit_status: 0u32,
            termination_pid: 0u32,
        }, Ghost(spec_extend_history(history@, outcome)))
    }
}
