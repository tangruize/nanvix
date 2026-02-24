pub fn unlock_mutex(pid: u32, tid: u32, mutex_addr: u32) -> (result: UnlockMutexResultModel)
    requires
        // Safety: the caller must not hold a PM reference.
        spec_unlock_mutex_safety_preconditions(),
        // The supplied pid/tid must be the currently-running thread.
        spec_is_currently_running(pid as nat, tid as nat),
        // ABI constraint: mutex_addr originates from 32-bit usize on x86-32.
        mutex_addr as nat <= USIZE_MAX_X86_32(),
    ensures
        // On success, the guard was dropped and the mutex was unlocked.
        spec_is_success(result.spec_view()) ==>
            spec_guard_dropped_and_mutex_unlocked(mutex_addr as nat),
        // On success, the calling thread owned the mutex.
        spec_is_success(result.spec_view()) ==>
            spec_thread_owns_mutex(pid as nat, tid as nat, mutex_addr as nat),
{
    // pid and tid are forwarded as ghost — they affect only PM-internal
    // ownership validation, not the pipeline mapping.
    // See lemma_result_mapping_independent_of_pid_tid.
    let ret: (
        UnlockMutexResultModel,
        Ghost<TakeMutexGuardOutcomeView>,
        Ghost<bool>,
    ) = unlock_mutex_model(mutex_addr, Ghost(pid), Ghost(tid));
    ret.0
}
