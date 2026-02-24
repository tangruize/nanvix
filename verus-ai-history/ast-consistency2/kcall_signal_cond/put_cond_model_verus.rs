pub fn put_cond_model(cond_addr: u32) -> (result: PutCondOutcomeModel)
    requires
        spec_signal_cond_safety_preconditions(),
        spec_cond_ref_released(cond_addr as nat),
    ensures
        result matches PutCondOutcomeModel::Error { error_code }
            ==> spec_is_valid_error_code(error_code as int),
        result matches PutCondOutcomeModel::Ok
            ==> spec_put_cond_completed(cond_addr as nat),
{
    unimplemented!()
}

//==================================================================================================
// Verified Functions
//==================================================================================================

/// Verified exec model of the `signal_cond` kernel call.
///
/// # Description
///
/// This function mirrors the original `pub unsafe fn signal_cond(pid, tid,
/// cond_addr, broadcast)` control flow. It:
/// 1. Calls get_cond (external) → returns Condvar on success.
/// 2. Calls notify (external) → returns awakened count on success.
/// 3. Drops the Condvar (external) → reference count decreases.
/// 4. Calls put_cond (external) → releases condition variable slot.
/// 5. Returns Ok(awakened) on success.
///
/// All steps short-circuit on error via the `?` operator pattern.
///
/// # Parameters
///
/// - `cond_addr`: Condition variable address (from `ConditionAddress::from(usize)`).
/// - `broadcast`: Whether to wake all waiting threads.
/// - `pid`: Ghost process identifier (for documentation only).
/// - `tid`: Ghost thread identifier (for documentation only).
///
/// # Returns
///
/// A tuple of (result, ghost state) where the ghost state captures
/// all step outcomes for exec-spec linkage.
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

/// Wrapper matching the original `pub unsafe fn signal_cond(...)` signature.
///
/// # Description
///
/// Delegates to `signal_cond_model`, which performs the actual verified pipeline
/// logic. The `pid` and `tid` parameters are forwarded as ghost — they are used
/// only in `trace!()` logging in the original and do not affect control flow.
///
/// # Parameters
///
/// - `pid`: Process identifier (used only in trace logging in original).
/// - `tid`: Thread identifier (used only in trace logging in original).
/// - `cond_addr`: Condition variable address (32-bit on x86).
/// - `broadcast`: Whether to wake all waiting threads.
///
/// # Returns
///
/// The result of the signal_cond pipeline as a `SignalCondResultModel`.
pub fn signal_cond(
    pid: u32,
    tid: u32,
    cond_addr: u32,
    broadcast: bool,
) -> (result: SignalCondResultModel)
    requires
        spec_signal_cond_safety_preconditions(),
        // ABI constraint: inputs originate from 32-bit usize on x86-32.
        cond_addr as nat <= USIZE_MAX_X86_32(),
    ensures
        // On success, put_cond was completed.
        spec_is_success(result.spec_view()) ==>
            spec_put_cond_completed(cond_addr as nat),
{
    // pid and tid are intentionally unused — they affect only trace logging
    // in the original function.
    let ret: (SignalCondResultModel, Ghost<SignalCondGhostState>) =
        signal_cond_model(cond_addr, broadcast, Ghost(pid), Ghost(tid));
    ret.0
}

} // verus!
