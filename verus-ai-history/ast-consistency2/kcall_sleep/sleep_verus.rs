pub fn sleep(seconds: u64, nanoseconds: u32) -> (result: SleepResultModel)
    requires
        // ABI constraint: seconds originates from a 32-bit usize on x86-32.
        seconds as nat <= USIZE_MAX_X86_32(),
        // Duration normalization must not overflow.
        seconds as nat + nanoseconds as nat / NANOS_PER_SEC() <= u64::MAX as nat,
    ensures
        // TimedOut is never in the output (folded into Ok by the 3-arm match).
        !matches!(result, SleepResultModel::TimedOut),
{
    unimplemented!()
}
