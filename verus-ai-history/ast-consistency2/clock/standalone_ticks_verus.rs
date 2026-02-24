pub fn standalone_ticks(timer: &TimerTicks) -> (result: u64)
    requires
        timer.wf(),
        TimerTicks::spec_no_concurrent_writer_assumption(),
    ensures
        result as nat == timer@.ticks,
{
    let (major, minor): (u32, u32) = timer.get();
    proof {
        assert(major <= u32::MAX);
        assert(minor <= u32::MAX);
        assert(u32::MAX as nat * TimerTicks::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
        TimerTicks::lemma_nat_mul_le_mono(major as nat, u32::MAX as nat, TimerTicks::MINOR_MODULUS());
    }
    (major as u64) * 0x1_0000_0000u64 + (minor as u64)
}
