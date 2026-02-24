pub fn unlock_mutex_model(mutex_addr: u32, pid: Ghost<u32>, tid: Ghost<u32>) -> (ret: (
    UnlockMutexResultModel,
    Ghost<TakeMutexGuardOutcomeView>,
    Ghost<bool>,
))
    requires
        // Safety: the caller must not hold a PM reference.
        spec_unlock_mutex_safety_preconditions(),
        // The supplied pid/tid must be the currently-running thread.
        // Guaranteed by the kcall dispatch layer.
        spec_is_currently_running(pid@ as nat, tid@ as nat),
        // ABI constraint: mutex_addr originates from 32-bit usize on x86-32.
        // This is always true for u32 values (documentation-only constraint
        // making the architecture assumption explicit).
        mutex_addr as nat <= USIZE_MAX_X86_32(),
    ensures
        // The result matches the spec for the captured PM outcome.
        ret.0.spec_view() == spec_unlock_mutex_result(ret.1@),
        // Success only when take_mutex_guard succeeds.
        spec_is_success(ret.0.spec_view()) ==>
            ret.1@ == TakeMutexGuardOutcomeView::TgOk,
        // Error only when take_mutex_guard fails.
        spec_is_error(ret.0.spec_view()) ==>
            !(ret.1@ == TakeMutexGuardOutcomeView::TgOk),
        // On success, guard was dropped and mutex was unlocked.
        spec_is_success(ret.0.spec_view()) ==>
            spec_guard_dropped_and_mutex_unlocked(mutex_addr as nat),
        // On success, the calling thread owned the mutex (from PM contract).
        spec_is_success(ret.0.spec_view()) ==>
            spec_thread_owns_mutex(pid@ as nat, tid@ as nat, mutex_addr as nat),
        // On success, PM did not internally drop the guard (caller dropped it).
        spec_is_success(ret.0.spec_view()) ==> !ret.2@,
        // On error, ghost flag indicates whether PM internally dropped the guard.
        // When true, the mutex was unlocked as a side effect of the PM error path.
        (spec_is_error(ret.0.spec_view()) && ret.2@) ==>
            spec_guard_dropped_and_mutex_unlocked(mutex_addr as nat),
{
    // Step 1: Take mutex guard (external), threading ghost pid/tid.
    let tg_pair: (TakeMutexGuardOutcomeModel, Ghost<Option<u32>>, Ghost<bool>) = take_mutex_guard_model(mutex_addr, pid, tid);
    let tg_result: TakeMutexGuardOutcomeModel = tg_pair.0;
    let guard_token: Ghost<Option<u32>> = tg_pair.1;
    let pm_dropped_guard: Ghost<bool> = tg_pair.2;
    let ghost tg_view: TakeMutexGuardOutcomeView = tg_result.spec_view();

    match tg_result {
        TakeMutexGuardOutcomeModel::Error { error_code } => {
            // No guard was returned; nothing to drop.
            // Forward the pm_dropped_guard flag to the caller.
            (
                UnlockMutexResultModel::TakeMutexGuardError { error_code },
                Ghost(tg_view),
                pm_dropped_guard,
            )
        },
        TakeMutexGuardOutcomeModel::Ok => {
            // Step 2: Drop the guard (models MutexGuard going out of scope).
            drop_guard_model(mutex_addr, guard_token);

            // pm_dropped_guard is guaranteed false on success path.
            (
                UnlockMutexResultModel::Success,
                Ghost(tg_view),
                pm_dropped_guard,
            )
        },
    }
}
