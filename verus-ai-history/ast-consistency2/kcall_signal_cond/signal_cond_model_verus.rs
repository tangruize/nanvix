pub fn signal_cond_model(
    cond_addr: u32,
    broadcast: bool,
    pid: Ghost<u32>,
    tid: Ghost<u32>,
) -> (ret: (SignalCondResultModel, Ghost<SignalCondGhostState>))
    requires
        spec_signal_cond_safety_preconditions(),
        // ABI constraint: cond_addr originates from 32-bit usize on x86-32.
        // This is trivially satisfied for u32 (documentation-only assertion
        // making the architecture assumption explicit; see lemma_architecture_guard).
        cond_addr as nat <= USIZE_MAX_X86_32(),
    ensures
        ({
            let gs: SignalCondGhostState = ret.1@;
            // The result matches the spec for the captured step outcomes.
            ret.0.spec_view() == spec_signal_cond_result(gs.gc, gs.notify, gs.pc)
        }),
        // Success only when all steps succeed.
        spec_is_success(ret.0.spec_view()) ==> ({
            let gs: SignalCondGhostState = ret.1@;
            gs.gc == GetCondOutcomeView::GcOk
            && gs.pc == PutCondOutcomeView::PcOk
            && gs.notify matches NotifyOutcomeView::NOk { .. }
        }),
        // Error only when at least one step fails.
        spec_is_error(ret.0.spec_view()) ==> ({
            let gs: SignalCondGhostState = ret.1@;
            !(gs.gc matches GetCondOutcomeView::GcOk)
            || !(gs.notify matches NotifyOutcomeView::NOk { .. })
            || !(gs.pc matches PutCondOutcomeView::PcOk)
        }),
        // Condvar ref released whenever get_cond succeeded (not just on overall
        // success). The Condvar is dropped at scope exit regardless of whether
        // notify succeeds or fails — callers can rely on resource cleanup even
        // on NotifyError or PutCondError paths. This subsumes the success case.
        ({
            let gs: SignalCondGhostState = ret.1@;
            gs.gc == GetCondOutcomeView::GcOk
        }) ==> spec_cond_ref_released(cond_addr as nat),
        // On success, put_cond completed successfully.
        spec_is_success(ret.0.spec_view()) ==>
            spec_put_cond_completed(cond_addr as nat),
        // On success, broadcast semantics are satisfied: the awakened count
        // respects the broadcast flag (at most 1 for signal, all waiters for broadcast).
        // Uses ghost state to access the notify outcome's awakened count.
        spec_is_success(ret.0.spec_view()) ==> ({
            let gs: SignalCondGhostState = ret.1@;
            gs.notify matches NotifyOutcomeView::NOk { awakened }
                && spec_broadcast_semantics(broadcast, cond_addr as nat, awakened)
        }),
{
    // Step 1: Get condvar reference (external).
    let gc_result: GetCondOutcomeModel = get_cond_model(cond_addr);
    let ghost gc_view: GetCondOutcomeView = gc_result.spec_view();

    match gc_result {
        GetCondOutcomeModel::Error { error_code } => {
            // get_cond failed; short-circuit return.
            let ghost gs: SignalCondGhostState = SignalCondGhostState {
                gc: gc_view,
                notify: NotifyOutcomeView::NOk { awakened: 0 },  // don't-care
                pc: PutCondOutcomeView::PcOk,  // don't-care
            };
            (
                SignalCondResultModel::GetCondError { error_code },
                Ghost(gs),
            )
        },
        GetCondOutcomeModel::Ok => {
            // Step 2: Notify threads (external).
            let notify_result: NotifyOutcomeModel = notify_model(cond_addr, broadcast);
            let ghost notify_view: NotifyOutcomeView = notify_result.spec_view();

            // Step 3: Drop condvar (always happens when get_cond succeeded).
            drop_cond_model(cond_addr);

            match notify_result {
                NotifyOutcomeModel::Error { error_code } => {
                    // notify failed; short-circuit return (put_cond not called).
                    let ghost gs: SignalCondGhostState = SignalCondGhostState {
                        gc: gc_view,
                        notify: notify_view,
                        pc: PutCondOutcomeView::PcOk,  // don't-care
                    };
                    (
                        SignalCondResultModel::NotifyError { error_code },
                        Ghost(gs),
                    )
                },
                NotifyOutcomeModel::Ok { awakened } => {
                    // Step 4: Put condvar (external).
                    let pc_result: PutCondOutcomeModel = put_cond_model(cond_addr);
                    let ghost pc_view: PutCondOutcomeView = pc_result.spec_view();

                    match pc_result {
                        PutCondOutcomeModel::Error { error_code } => {
                            let ghost gs: SignalCondGhostState = SignalCondGhostState {
                                gc: gc_view,
                                notify: notify_view,
                                pc: pc_view,
                            };
                            (
                                SignalCondResultModel::PutCondError { error_code },
                                Ghost(gs),
                            )
                        },
                        PutCondOutcomeModel::Ok => {
                            let ghost gs: SignalCondGhostState = SignalCondGhostState {
                                gc: gc_view,
                                notify: notify_view,
                                pc: pc_view,
                            };
                            (
                                SignalCondResultModel::Success { awakened },
                                Ghost(gs),
                            )
                        },
                    }
                },
            }
        },
    }
}
