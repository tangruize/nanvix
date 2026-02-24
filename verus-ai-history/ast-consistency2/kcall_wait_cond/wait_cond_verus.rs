pub fn wait_cond(
    pid: u32,
    tid: u32,
    cond_addr: u32,
    mutex_addr: u32,
    timeout_s: u32,
    timeout_ns: u32,
) -> (result: WaitCondResultModel)
    requires
        // ABI constraint: inputs originate from 32-bit usize on x86-32.
        USIZE_BITS() == 32,
        // Safety: caller must satisfy the wait_cond safety preconditions.
        spec_wait_cond_safety_preconditions(pid as nat, tid as nat),
        // The supplied pid/tid must be the currently-running thread.
        spec_is_currently_running(pid as nat, tid as nat),
    ensures
        // Mutex protocol: on stored-result return, the mutex was released
        // before the wait and reacquired afterward.
        spec_is_stored_result_return(result.spec_view()) ==> (
            spec_mutex_released(mutex_addr as nat)
            && spec_cond_ref_released(cond_addr as nat)
            && spec_mutex_reacquired(mutex_addr as nat)
        ),
{
    // pid and tid are forwarded as ghost — they affect only trace logging
    // and PM-internal ownership validation, not the pipeline control flow.
    let ret: (WaitCondResultModel, Ghost<WaitCondGhostState>) =
        wait_cond_model(cond_addr, mutex_addr, timeout_s, timeout_ns, Ghost(pid), Ghost(tid));
    ret.0
}
