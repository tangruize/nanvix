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
