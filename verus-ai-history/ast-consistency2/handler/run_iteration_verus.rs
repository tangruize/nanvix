pub fn run_iteration(poll: &ScoreBoardPollResult, stdio_enabled: bool) -> (result: IterationResult)
    ensures
        // If a kcall was polled, it was handled.
        poll.has_call ==> result.work_state.kcall_handled,
        // If no kcall was polled and no messages/zombies, should yield.
        !result.work_state.kcall_handled && !result.work_state.message_received
            && !result.work_state.harvested_process ==> result.should_yield,
        // Yield iff no work was done.
        result.should_yield == (!result.work_state.kcall_handled
            && !result.work_state.message_received && !result.work_state.harvested_process),
        // Termination implies INITD pid.
        result.should_terminate ==> result.initd_pid == 1u32,
        // INITD termination does NOT set harvested_process (loop breaks first).
        result.should_terminate ==> !result.work_state.harvested_process,
        // Concrete harvest fields link to spec termination via spec_harvest_to_outcome.
        result.should_terminate == spec_should_terminate(spec_harvest_to_outcome(
            result.harvest_found, result.harvest_error, result.harvest_pid as nat, result.harvest_is_initd)),
        // Non-stdio builds never receive messages.
        !stdio_enabled ==> !result.work_state.message_received,
{
    // Phase 1: Handle pending kernel call.
    let kcall_phase: HandlerKcallPhaseResult = handle_kcall_phase(poll);

    // Phase 2: Poll for IKC messages (gated by stdio feature flag).
    let msg_received: bool = poll_messages_gated(stdio_enabled);

    // Phase 3: Harvest zombie processes.
    let harvest: ZombieHarvestResult = handle_harvest_phase();
    let terminate: bool = is_initd_terminated(&harvest);

    // Derive ghost harvest outcome from exec-level fields.
    proof {
        lemma_harvest_to_outcome_termination(
            harvest.found, harvest.error, harvest.pid as nat, harvest.is_initd,
        );
    }

    // Notify termination for non-INITD zombies only.
    // In the original, INITD causes `break status` before reaching notify.
    let harvested_flag: bool = if harvest.found && !terminate {
        notify_termination(harvest.pid, harvest.exit_status)
    } else {
        false
    };

    // Build work state.
    let work_state: HandlerWorkState = HandlerWorkState {
        kcall_handled: kcall_phase.kcall_handled,
        message_received: msg_received,
        harvested_process: harvested_flag,
    };

    let do_yield: bool = should_yield(&work_state);

    IterationResult {
        work_state,
        should_yield: do_yield,
        should_terminate: terminate,
        exit_status: harvest.exit_status,
        initd_pid: harvest.pid,
        harvest_found: harvest.found,
        harvest_error: harvest.error,
        harvest_pid: harvest.pid,
        harvest_is_initd: harvest.is_initd,
    }
}
