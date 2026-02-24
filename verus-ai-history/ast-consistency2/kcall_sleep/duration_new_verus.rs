pub fn duration_new(seconds: u64, nanoseconds: u32) -> (result: DurationModel)
    requires
        // The carry from nanosecond normalization won't overflow u64 seconds.
        // Since nanoseconds is u32, carry <= 4. On Nanvix's x86-32, seconds
        // comes from usize (32-bit), so this is always satisfied.
        seconds as nat + nanoseconds as nat / NANOS_PER_SEC() <= u64::MAX as nat,
    ensures
        result.spec_wf(),
        result.spec_view() == spec_duration_new(seconds as nat, nanoseconds as nat),
        result.nanoseconds < 1_000_000_000u32,
{
    let carry: u64 = (nanoseconds / 1_000_000_000u32) as u64;
    let remainder: u32 = nanoseconds % 1_000_000_000u32;

    proof {
        lemma_duration_new_wf(seconds as nat, nanoseconds as nat);
        assert(NANOS_PER_SEC() == 1_000_000_000nat);
        assert(remainder as nat == nanoseconds as nat % NANOS_PER_SEC());
        assert(carry as nat == nanoseconds as nat / NANOS_PER_SEC());
    }

    let total_seconds: u64 = seconds + carry;

    DurationModel { seconds: total_seconds, nanoseconds: remainder }
}
